//! Inventory of what each AI tool on this machine has configured: MCP
//! servers, skills, plugins and extensions. v0.8.0 is read-only: nothing
//! here writes a config file. Config files often hold API keys, so every
//! value that leaves this module is masked ([`mask`]) and the raw text is
//! never kept.
//!
//! Flow: the catalogue ([`catalog`], `aitools.toml`) says where each tool
//! keeps its files → [`mcp`] parses them into [`McpServer`]s with a
//! normalised identity ("the same server" across tools) → [`skills`] lists
//! skill folders, plugins and extensions with a content hash → [`analyze`]
//! turns all that into duplicates, conflicts, stale entries and exposed keys.

pub mod analyze;
pub mod catalog;
pub mod mcp;
pub mod skills;
pub mod versions;

pub use analyze::{analyze, Analysis, Finding, FindingKind};
pub use catalog::{Catalog, McpSource, ToolDef};
pub use mcp::{ConfigFile, McpServer, McpTransport, SecretHit};
pub use skills::{Extension, Plugin, Skill};
pub use versions::{latest_from_response, registry_url, VersionInfo};

use crate::paths::PathContext;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Everything found for one tool.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolInventory {
    pub id: String,
    pub name: String,
    /// At least one config file, skills folder, plugin or extension exists.
    pub present: bool,
    /// A catalogue path not yet confirmed on a real machine.
    pub verified: bool,
    pub running: bool,
    pub configs: Vec<ConfigFile>,
    pub mcp: Vec<McpServer>,
    pub skills: Vec<Skill>,
    pub plugins: Vec<Plugin>,
    pub extensions: Vec<Extension>,
}

/// The whole picture plus what [`analyze`] made of it.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Inventory {
    pub tools: Vec<ToolInventory>,
    pub analysis: Analysis,
    pub computed_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Reads every catalogued location and analyses the result.
pub fn inventory(catalog: &Catalog, ctx: &PathContext) -> Inventory {
    let running = crate::process::running_names();
    let tools: Vec<ToolInventory> = catalog
        .tools
        .iter()
        .map(|t| inventory_tool(t, ctx, &running))
        .collect();
    let analysis = analyze(&tools);
    Inventory {
        tools,
        analysis,
        computed_at: Some(chrono::Utc::now()),
    }
}

fn inventory_tool(def: &ToolDef, ctx: &PathContext, running: &HashSet<String>) -> ToolInventory {
    let mut inv = ToolInventory {
        id: def.id.clone(),
        name: def.name.clone(),
        verified: def.verified,
        running: crate::process::first_running(&def.processes, running).is_some(),
        ..Default::default()
    };
    for src in &def.mcp {
        for path in catalog::expand_glob(ctx, &src.path) {
            let (file, servers) = mcp::read_config(&def.id, &path, src);
            if file.exists {
                inv.present = true;
            }
            inv.configs.push(file);
            inv.mcp.extend(servers);
        }
    }
    for tpl in &def.skills {
        if let Some(dir) = ctx.expand(tpl) {
            let found = skills::list_skills(&def.id, &dir);
            if !found.is_empty() {
                inv.present = true;
            }
            inv.skills.extend(found);
        }
    }
    if let Some(tpl) = &def.plugins {
        if let Some(dir) = ctx.expand(tpl) {
            let found = skills::list_plugins(&def.id, &dir);
            if !found.is_empty() {
                inv.present = true;
            }
            inv.plugins.extend(found);
        }
    }
    for tpl in &def.extensions {
        if let Some(dir) = ctx.expand(tpl) {
            let (exts, embedded) = skills::list_extensions(&def.id, &dir);
            if !exts.is_empty() {
                inv.present = true;
            }
            inv.extensions.extend(exts);
            inv.mcp.extend(embedded);
        }
    }
    inv
}

/// Shows `sk-…a1b2` instead of a key. Short values become `…` entirely so
/// nothing can be reassembled from the UI or a log.
pub fn mask(value: &str) -> String {
    let chars: Vec<char> = value.chars().collect();
    if chars.len() < 12 {
        return "…".to_string();
    }
    let head: String = chars[..3].iter().collect();
    let tail: String = chars[chars.len() - 4..].iter().collect();
    format!("{head}…{tail}")
}

/// Whether a config key or value looks like a credential.
pub fn looks_secret(key: &str, value: &str) -> bool {
    let k = key.to_ascii_uppercase();
    let by_key = [
        "KEY",
        "TOKEN",
        "SECRET",
        "PASSWORD",
        "PASSWD",
        "CREDENTIAL",
        "AUTH",
    ]
    .iter()
    .any(|w| k.contains(w));
    let v = value.trim();
    // `${ENV}` / `$ENV` references are fine: the secret lives elsewhere.
    if v.is_empty() || v.starts_with("${") || v.starts_with('$') || v.len() < 8 {
        return false;
    }
    let by_value = [
        "sk-",
        "ghp_",
        "gho_",
        "github_pat_",
        "glpat-",
        "xoxb-",
        "xoxp-",
        "AKIA",
        "ya29.",
        "AIza",
        "Bearer ",
        "pplx-",
        "hf_",
        "npm_",
        "sk_live_",
    ]
    .iter()
    .any(|p| v.starts_with(p));
    by_key || by_value
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[test]
    fn inventory_over_a_fake_home_finds_servers_skills_and_findings() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();
        let w = |rel: &str, body: &str| {
            let p = home.join(rel);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, body).unwrap();
        };
        w(
            ".cursor/mcp.json",
            r#"{"mcpServers": {"github": {"command": "npx", "args": ["-y", "@modelcontextprotocol/server-github@1.0.0"], "env": {"GITHUB_TOKEN": "ghp_0123456789abcdefghij"}}}}"#,
        );
        w(
            ".codex/config.toml",
            "[mcp_servers.gh]\ncommand = \"npx\"\nargs = [\"-y\", \"@modelcontextprotocol/server-github@2.0.0\"]\n",
        );
        w(
            ".claude/skills/web/SKILL.md",
            "---\ndescription: web\n---\n",
        );
        w(".codex/skills/web/SKILL.md", "---\ndescription: web\n---\n");
        w(
            ".agents/skills/web/SKILL.md",
            "---\ndescription: web (edited)\n---\n",
        );
        let mut vars = HashMap::new();
        vars.insert("USERPROFILE".to_string(), home.to_path_buf());
        vars.insert("APPDATA".to_string(), home.join("AppData/Roaming"));
        let ctx = PathContext::with_vars(vars);
        let inv = inventory(&Catalog::builtin(), &ctx);
        let present: Vec<&str> = inv
            .tools
            .iter()
            .filter(|t| t.present)
            .map(|t| t.id.as_str())
            .collect();
        assert_eq!(
            present,
            vec!["claude_code", "cursor", "codex", "agents_shared"]
        );
        assert_eq!(inv.analysis.mcp_total, 2);
        assert_eq!(inv.analysis.mcp_unique, 1);
        let kinds: Vec<FindingKind> = inv.analysis.findings.iter().map(|f| f.kind).collect();
        assert!(kinds.contains(&FindingKind::McpVersionConflict));
        assert!(kinds.contains(&FindingKind::SecretExposed));
        assert!(kinds.contains(&FindingKind::SkillDiverged));
        // The raw token never appears anywhere in the serialised inventory.
        let json = serde_json::to_string(&inv).unwrap();
        assert!(!json.contains("0123456789abcdefghij"));
        assert!(json.contains("ghp…ghij"));
        // Missing config files are listed but not errors.
        let trae = inv.tools.iter().find(|t| t.id == "trae").unwrap();
        assert!(!trae.present && !trae.verified && trae.configs.iter().all(|c| !c.exists));
        let _: PathBuf = trae.configs[0].path.clone();
    }

    #[test]
    fn mask_keeps_only_the_ends() {
        assert_eq!(mask("sk-abcdefghijklmnop1234"), "sk-…1234");
        assert_eq!(mask("short"), "…");
    }

    #[test]
    fn secret_detection_by_key_and_by_value() {
        assert!(looks_secret("GITHUB_TOKEN", "ghp_xxxxxxxxxxxxxxxx"));
        assert!(looks_secret("whatever", "sk-ant-api03-xxxxxxxx"));
        assert!(!looks_secret("GITHUB_TOKEN", "${GITHUB_TOKEN}"));
        assert!(!looks_secret("PATH", "/usr/bin"));
        assert!(!looks_secret("API_KEY", ""));
    }
}
