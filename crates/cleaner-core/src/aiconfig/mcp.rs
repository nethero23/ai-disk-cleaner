//! Parses the MCP server lists of the various tools into one shape.
//!
//! Dialects seen in the wild: `mcpServers` / `servers` / `mcp` /
//! `context_servers` as the root key; `command` as a string or (opencode) an
//! array; Zed's `command: { path, args }`; remote servers under `url`,
//! `serverUrl`, `httpUrl` or `uri`; Codex in TOML. Comments and trailing
//! commas are tolerated because VS Code and Kilo write JSONC.

use super::catalog::McpSource;
use super::{looks_secret, mask};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfigFile {
    pub tool: String,
    pub path: PathBuf,
    pub exists: bool,
    pub bytes: u64,
    /// Parse error, if any. The file is still listed so the user can see it.
    pub error: Option<String>,
    pub servers: usize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum McpTransport {
    Stdio,
    Remote,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SecretHit {
    /// `env.GITHUB_TOKEN`, `args[2]`, `headers.Authorization`, `url`.
    pub location: String,
    pub masked: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct McpServer {
    pub tool: String,
    pub config_path: PathBuf,
    /// `user`, `project:<dir>` or `extension:<name>`.
    pub scope: String,
    pub name: String,
    pub transport: McpTransport,
    pub command: Option<String>,
    /// Arguments with anything secret-looking already masked.
    pub args: Vec<String>,
    pub url: Option<String>,
    /// npm / PyPI / Docker image name when the launcher is recognised.
    pub package: Option<String>,
    /// Pinned version, if one is written (`@1.2.3`, `==1.2.3`, `:tag`).
    pub version: Option<String>,
    /// `npm:<package>`, `pypi:<package>`, `docker:<image>`, `url:<url>`,
    /// `script:<path>` or `cmd:<command line>`: equal means the same server.
    pub identity: String,
    pub env_keys: Vec<String>,
    pub secrets: Vec<SecretHit>,
    pub disabled: bool,
}

/// Reads one config file. Never fails: a missing file is `exists: false`,
/// a broken one carries `error` and no servers.
pub fn read_config(tool: &str, path: &Path, src: &McpSource) -> (ConfigFile, Vec<McpServer>) {
    let mut file = ConfigFile {
        tool: tool.to_string(),
        path: path.to_path_buf(),
        exists: false,
        bytes: 0,
        error: None,
        servers: 0,
    };
    let Ok(md) = std::fs::metadata(path) else {
        return (file, Vec::new());
    };
    file.exists = true;
    file.bytes = md.len();
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            file.error = Some(e.to_string());
            return (file, Vec::new());
        }
    };
    match parse_text(tool, path, src, &text) {
        Ok(servers) => {
            file.servers = servers.len();
            (file, servers)
        }
        Err(e) => {
            file.error = Some(e);
            (file, Vec::new())
        }
    }
}

/// The parsing half of [`read_config`], for tests and for extension manifests.
pub fn parse_text(
    tool: &str,
    path: &Path,
    src: &McpSource,
    text: &str,
) -> Result<Vec<McpServer>, String> {
    let root: Value = if src.format == "toml" {
        let t: toml::Value = toml::from_str(text).map_err(|e| e.to_string())?;
        serde_json::to_value(t).map_err(|e| e.to_string())?
    } else {
        serde_json::from_str(&strip_jsonc(text)).map_err(|e| e.to_string())?
    };
    let mut out = Vec::new();
    if let Some(map) = root.get(&src.root).and_then(Value::as_object) {
        for (name, v) in map {
            if let Some(s) = server_from_value(tool, path, "user", name, v) {
                out.push(s);
            }
        }
    }
    if src.projects {
        if let Some(projects) = root.get("projects").and_then(Value::as_object) {
            for (dir, pv) in projects {
                if let Some(map) = pv.get(&src.root).and_then(Value::as_object) {
                    for (name, v) in map {
                        if let Some(s) =
                            server_from_value(tool, path, &format!("project:{dir}"), name, v)
                        {
                            out.push(s);
                        }
                    }
                }
            }
        }
    }
    Ok(out)
}

/// Servers listed under `mcpServers` in a plugin or extension manifest.
pub fn servers_in_manifest(tool: &str, path: &Path, scope: &str, root: &Value) -> Vec<McpServer> {
    let mut out = Vec::new();
    for key in ["mcpServers", "servers", "mcp"] {
        if let Some(map) = root.get(key).and_then(Value::as_object) {
            for (name, v) in map {
                if let Some(s) = server_from_value(tool, path, scope, name, v) {
                    out.push(s);
                }
            }
        }
    }
    out
}

fn str_of(v: &Value) -> Option<String> {
    v.as_str().map(str::to_string)
}

fn string_list(v: Option<&Value>) -> Vec<String> {
    v.and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

pub fn server_from_value(
    tool: &str,
    path: &Path,
    scope: &str,
    name: &str,
    v: &Value,
) -> Option<McpServer> {
    let obj = v.as_object()?;
    // Command: string, array (opencode) or object (Zed).
    let (mut command, mut args) = match obj.get("command") {
        Some(Value::String(s)) => (Some(s.clone()), string_list(obj.get("args"))),
        Some(Value::Array(a)) => {
            let mut parts: Vec<String> = a
                .iter()
                .filter_map(|x| x.as_str().map(str::to_string))
                .collect();
            if parts.is_empty() {
                (None, Vec::new())
            } else {
                let c = parts.remove(0);
                parts.extend(string_list(obj.get("args")));
                (Some(c), parts)
            }
        }
        Some(Value::Object(o)) => (
            o.get("path")
                .and_then(str_of)
                .or_else(|| o.get("command").and_then(str_of)),
            string_list(o.get("args")),
        ),
        _ => (None, Vec::new()),
    };
    // A single "command" string holding the whole line ("npx -y pkg").
    if args.is_empty() {
        if let Some(c) = command.clone() {
            if c.contains(' ') && !Path::new(&c).exists() {
                let mut parts = c.split_whitespace().map(str::to_string);
                command = parts.next();
                args = parts.collect();
            }
        }
    }
    let url = ["url", "serverUrl", "httpUrl", "uri", "endpoint"]
        .iter()
        .find_map(|k| obj.get(*k).and_then(str_of));
    if command.is_none() && url.is_none() {
        // Zed `{"source": "extension"}` and the like: nothing to launch.
        return None;
    }
    let disabled = obj
        .get("disabled")
        .and_then(Value::as_bool)
        .unwrap_or(false)
        || obj.get("enabled").and_then(Value::as_bool) == Some(false)
        || obj.get("enable").and_then(Value::as_bool) == Some(false);

    let mut secrets = Vec::new();
    let mut env_keys = Vec::new();
    for (section, key_name) in [
        ("env", "env"),
        ("headers", "headers"),
        ("environment", "env"),
    ] {
        if let Some(map) = obj.get(section).and_then(Value::as_object) {
            for (k, val) in map {
                if key_name == "env" {
                    env_keys.push(k.clone());
                }
                let s = match val {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                if looks_secret(k, &s) {
                    secrets.push(SecretHit {
                        location: format!("{section}.{k}"),
                        masked: mask(&s),
                    });
                }
            }
        }
    }
    let mut masked_args = Vec::with_capacity(args.len());
    let mut prev_flag_secret = false;
    for (i, a) in args.iter().enumerate() {
        let (flag, val) = match a.split_once('=') {
            Some((f, v)) if f.starts_with('-') => (f, Some(v)),
            _ => (a.as_str(), None),
        };
        let hit = if let Some(v) = val {
            looks_secret(flag, v)
        } else if prev_flag_secret {
            !a.starts_with('-')
        } else {
            looks_secret("", a)
        };
        if hit {
            secrets.push(SecretHit {
                location: format!("args[{i}]"),
                masked: mask(val.unwrap_or(a)),
            });
            masked_args.push(match val {
                Some(v) => format!("{flag}={}", mask(v)),
                None => mask(a),
            });
        } else {
            masked_args.push(a.clone());
        }
        prev_flag_secret =
            val.is_none() && a.starts_with('-') && looks_secret(a, "placeholder-value");
    }
    let mut url_out = url.clone();
    if let Some(u) = &url {
        if let Some((base, query)) = u.split_once('?') {
            let has = query.split('&').any(|kv| {
                let (k, v) = kv.split_once('=').unwrap_or((kv, ""));
                looks_secret(k, v)
            });
            if has {
                secrets.push(SecretHit {
                    location: "url".into(),
                    masked: format!("{base}?…"),
                });
                url_out = Some(format!("{base}?…"));
            }
        }
    }

    let (transport, identity, package, version) = match (&command, &url) {
        (Some(c), _) => {
            let (id, pkg, ver) = stdio_identity(c, &args);
            (McpTransport::Stdio, id, pkg, ver)
        }
        (None, Some(u)) => (
            McpTransport::Remote,
            format!("url:{}", normalise_url(u)),
            None,
            None,
        ),
        (None, None) => unreachable!(),
    };
    Some(McpServer {
        tool: tool.to_string(),
        config_path: path.to_path_buf(),
        scope: scope.to_string(),
        name: name.to_string(),
        transport,
        command,
        args: masked_args,
        url: url_out,
        package,
        version,
        identity,
        env_keys,
        secrets,
        disabled,
    })
}

fn normalise_url(u: &str) -> String {
    let u = u
        .trim()
        .split('?')
        .next()
        .unwrap_or("")
        .trim_end_matches('/');
    match u.split_once("://") {
        Some((scheme, rest)) => {
            let (host, path) = rest.split_once('/').unwrap_or((rest, ""));
            let mut s = format!(
                "{}://{}",
                scheme.to_ascii_lowercase(),
                host.to_ascii_lowercase()
            );
            if !path.is_empty() {
                s.push('/');
                s.push_str(path);
            }
            s
        }
        None => u.to_string(),
    }
}

fn basename(cmd: &str) -> String {
    let b = cmd
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(cmd)
        .to_ascii_lowercase();
    b.trim_end_matches(".exe")
        .trim_end_matches(".cmd")
        .trim_end_matches(".bat")
        .to_string()
}

/// Splits `pkg@1.2.3`, `@scope/pkg@1.2.3`, `pkg==1.2.3` into name and version.
fn split_version(spec: &str) -> (String, Option<String>) {
    if let Some((n, v)) = spec.split_once("==") {
        return (n.to_string(), Some(v.to_string()));
    }
    let start = if spec.starts_with('@') { 1 } else { 0 };
    match spec[start..].find('@') {
        Some(i) => {
            let i = i + start;
            (spec[..i].to_string(), Some(spec[i + 1..].to_string()))
        }
        None => (spec.to_string(), None),
    }
}

/// Picks the first argument that is a package or script, skipping launcher flags.
fn first_positional(args: &[String], value_flags: &[&str]) -> Option<String> {
    let mut skip = false;
    for a in args {
        if skip {
            skip = false;
            continue;
        }
        if a.starts_with('-') {
            if value_flags.contains(&a.as_str()) {
                skip = true;
            }
            continue;
        }
        return Some(a.clone());
    }
    None
}

/// (identity, package, version) for a stdio server.
fn stdio_identity(command: &str, args: &[String]) -> (String, Option<String>, Option<String>) {
    let b = basename(command);
    let joined = |pre: &str| {
        let mut s = String::from(pre);
        s.push_str(&b);
        for a in args {
            s.push(' ');
            s.push_str(a);
        }
        s
    };
    match b.as_str() {
        "npx" | "bunx" | "pnpx" | "npm" | "pnpm" | "yarn" | "cmd" => {
            // `cmd /c npx -y pkg` on Windows; `npm exec`, `pnpm dlx`, `yarn dlx`.
            let mut rest: Vec<String> = args.to_vec();
            if b == "cmd" {
                rest.retain(|a| !a.eq_ignore_ascii_case("/c"));
                let inner = rest.first().map(|c| basename(c)).unwrap_or_default();
                if !["npx", "bunx", "pnpx", "npm", "pnpm", "yarn", "uvx", "uv"]
                    .contains(&inner.as_str())
                {
                    return (joined("cmd:"), None, None);
                }
                let c = rest.remove(0);
                return stdio_identity(&c, &rest);
            }
            if ["npm", "pnpm", "yarn"].contains(&b.as_str()) {
                if !rest
                    .first()
                    .is_some_and(|a| a == "exec" || a == "dlx" || a == "x")
                {
                    return (joined("cmd:"), None, None);
                }
                rest.remove(0);
            }
            match first_positional(&rest, &["-p", "--package", "--registry", "--cache", "-c"]) {
                Some(spec) => {
                    let (name, ver) = split_version(&spec);
                    (
                        format!("npm:{}", name.to_ascii_lowercase()),
                        Some(name),
                        ver,
                    )
                }
                None => (joined("cmd:"), None, None),
            }
        }
        "uvx" | "uv" | "pipx" => {
            let mut rest: Vec<String> = args.to_vec();
            if b == "uv" {
                if rest.len() >= 2 && rest[0] == "tool" && rest[1] == "run" {
                    rest.drain(..2);
                } else if rest.first().is_some_and(|a| a == "run" || a == "x") {
                    rest.remove(0);
                } else {
                    return (joined("cmd:"), None, None);
                }
            }
            if b == "pipx" {
                if rest.first().is_some_and(|a| a == "run") {
                    rest.remove(0);
                } else {
                    return (joined("cmd:"), None, None);
                }
            }
            // `uvx --from pkg cmd` names the package in the flag value.
            let from = rest
                .windows(2)
                .find(|w| w[0] == "--from")
                .map(|w| w[1].clone());
            let spec = from.or_else(|| {
                first_positional(
                    &rest,
                    &[
                        "--python",
                        "-p",
                        "--with",
                        "--index",
                        "--index-url",
                        "--from",
                    ],
                )
            });
            match spec {
                Some(spec) => {
                    let (name, ver) = split_version(&spec);
                    let name = name.to_ascii_lowercase().replace('_', "-");
                    (format!("pypi:{name}"), Some(name), ver)
                }
                None => (joined("cmd:"), None, None),
            }
        }
        "docker" | "podman" => {
            let image = args
                .iter()
                .rev()
                .find(|a| !a.starts_with('-') && (a.contains('/') || a.contains(':')))
                .cloned();
            match image {
                Some(img) => {
                    let (name, tag) = match img.rsplit_once(':') {
                        Some((n, t)) if !t.contains('/') => (n.to_string(), Some(t.to_string())),
                        _ => (img.clone(), None),
                    };
                    (
                        format!("docker:{}", name.to_ascii_lowercase()),
                        Some(name),
                        tag,
                    )
                }
                None => (joined("cmd:"), None, None),
            }
        }
        "node" | "python" | "python3" | "deno" | "bun" | "tsx" | "ts-node" => {
            let script = args.iter().find(|a| {
                let l = a.to_ascii_lowercase();
                [".js", ".mjs", ".cjs", ".ts", ".py"]
                    .iter()
                    .any(|e| l.ends_with(e))
            });
            match script {
                Some(s) => (
                    format!("script:{}", s.replace('\\', "/").to_ascii_lowercase()),
                    None,
                    None,
                ),
                None => (joined("cmd:"), None, None),
            }
        }
        _ => {
            let l = command.to_ascii_lowercase().replace('\\', "/");
            if l.contains('/') {
                (format!("script:{l}"), None, None)
            } else {
                (joined("cmd:"), None, None)
            }
        }
    }
}

/// Removes `//` and `/* */` comments outside strings and trailing commas
/// before `}` or `]`, so JSONC parses with serde_json.
pub fn strip_jsonc(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    let mut in_str = false;
    while i < chars.len() {
        let c = chars[i];
        if in_str {
            out.push(c);
            if c == '\\' && i + 1 < chars.len() {
                out.push(chars[i + 1]);
                i += 2;
                continue;
            }
            if c == '"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        match c {
            '"' => {
                in_str = true;
                out.push(c);
                i += 1;
            }
            '/' if chars.get(i + 1) == Some(&'/') => {
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
            }
            '/' if chars.get(i + 1) == Some(&'*') => {
                i += 2;
                while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                    i += 1;
                }
                i += 2;
            }
            ',' => {
                // Trailing comma: look past whitespace for a closing bracket.
                let mut j = i + 1;
                while j < chars.len() && chars[j].is_whitespace() {
                    j += 1;
                }
                if j < chars.len() && (chars[j] == '}' || chars[j] == ']') {
                    i += 1;
                } else {
                    out.push(c);
                    i += 1;
                }
            }
            _ => {
                out.push(c);
                i += 1;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn src(root: &str) -> McpSource {
        McpSource {
            path: String::new(),
            root: root.into(),
            format: "json".into(),
            projects: false,
        }
    }

    fn parse(root: &str, text: &str) -> Vec<McpServer> {
        parse_text("t", Path::new("/c.json"), &src(root), text).unwrap()
    }

    #[test]
    fn npx_and_uvx_servers_get_package_identities() {
        let s = parse(
            "mcpServers",
            r#"{"mcpServers": {
              "github": {"command": "npx", "args": ["-y", "@modelcontextprotocol/server-github@1.2.0"], "env": {"GITHUB_TOKEN": "ghp_abcdefghijklmnopqrstuvwxyz"}},
              "fetch": {"command": "uvx", "args": ["mcp-server-fetch==0.6"]},
              "win": {"command": "cmd", "args": ["/c", "npx", "-y", "@upstash/context7-mcp"]},
              "off": {"command": "node", "args": ["C:\\mcp\\Server.js"], "disabled": true}
            }}"#,
        );
        assert_eq!(s.len(), 4);
        let by = |n: &str| s.iter().find(|x| x.name == n).unwrap();
        let gh = by("github");
        assert_eq!(gh.identity, "npm:@modelcontextprotocol/server-github");
        assert_eq!(gh.version.as_deref(), Some("1.2.0"));
        assert_eq!(gh.env_keys, vec!["GITHUB_TOKEN"]);
        assert_eq!(gh.secrets.len(), 1);
        assert_eq!(gh.secrets[0].location, "env.GITHUB_TOKEN");
        assert_eq!(gh.secrets[0].masked, "ghp…wxyz");
        assert_eq!(by("fetch").identity, "pypi:mcp-server-fetch");
        assert_eq!(by("fetch").version.as_deref(), Some("0.6"));
        assert_eq!(by("win").identity, "npm:@upstash/context7-mcp");
        assert_eq!(by("off").identity, "script:c:/mcp/server.js");
        assert!(by("off").disabled);
    }

    #[test]
    fn remote_servers_and_array_commands() {
        let s = parse(
            "mcp",
            r#"{"mcp": {
              "ctx": {"type": "remote", "url": "https://MCP.Context7.com/mcp/?apiKey=abcdefghijklmnop"},
              "local": {"type": "local", "command": ["npx", "-y", "some-pkg"], "enabled": false}
            }}"#,
        );
        let ctx = s.iter().find(|x| x.name == "ctx").unwrap();
        let local = s.iter().find(|x| x.name == "local").unwrap();
        assert_eq!(ctx.transport, McpTransport::Remote);
        assert_eq!(ctx.identity, "url:https://mcp.context7.com/mcp");
        assert_eq!(ctx.url.as_deref(), Some("https://MCP.Context7.com/mcp/?…"));
        assert_eq!(ctx.secrets[0].location, "url");
        assert_eq!(local.identity, "npm:some-pkg");
        assert!(local.disabled);
    }

    #[test]
    fn codex_toml_and_claude_projects() {
        let toml_src = McpSource {
            path: String::new(),
            root: "mcp_servers".into(),
            format: "toml".into(),
            projects: false,
        };
        let s = parse_text(
            "codex",
            Path::new("/config.toml"),
            &toml_src,
            "model = \"o3\"\n[mcp_servers.fs]\ncommand = \"npx\"\nargs = [\"-y\", \"@modelcontextprotocol/server-filesystem\", \"/tmp\"]\n",
        )
        .unwrap();
        assert_eq!(s[0].identity, "npm:@modelcontextprotocol/server-filesystem");

        let cc = McpSource {
            path: String::new(),
            root: "mcpServers".into(),
            format: "json".into(),
            projects: true,
        };
        let s = parse_text(
            "claude_code",
            Path::new("/.claude.json"),
            &cc,
            r#"{"numStartups": 5, "mcpServers": {"a": {"command": "npx", "args": ["x"]}},
                "projects": {"C:\\proj": {"mcpServers": {"b": {"url": "http://localhost:3000/sse"}}, "allowedTools": []}}}"#,
        )
        .unwrap();
        assert_eq!(s.len(), 2);
        assert!(s.iter().any(|x| x.scope == "project:C:\\proj"));
    }

    #[test]
    fn jsonc_comments_and_trailing_commas() {
        let cleaned = strip_jsonc("{\n // c\n \"a\": \"x//y\", /* z */ \"b\": [1,2,],\n}");
        let v: Value = serde_json::from_str(&cleaned).unwrap();
        assert_eq!(v["a"], "x//y");
        assert_eq!(v["b"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn secret_args_are_masked() {
        let s = parse(
            "servers",
            r#"{"servers": {"x": {"command": "npx", "args": ["pkg", "--api-key", "sk-1234567890abcdefgh", "--token=ghp_zzzzzzzzzzzzzzzzzz"]}}}"#,
        );
        assert_eq!(s[0].args[2], "sk-…efgh");
        assert_eq!(s[0].args[3], "--token=ghp…zzzz");
        assert_eq!(s[0].secrets.len(), 2);
        assert!(!serde_json::to_string(&s[0]).unwrap().contains("1234567890"));
    }

    #[test]
    fn missing_file_is_not_an_error() {
        let (f, s) = read_config("t", Path::new("/nope/none.json"), &src("mcpServers"));
        assert!(!f.exists && f.error.is_none() && s.is_empty());
    }
}
