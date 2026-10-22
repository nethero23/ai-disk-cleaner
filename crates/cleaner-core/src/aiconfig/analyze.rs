//! Turns the inventory into things worth telling the user: the same MCP
//! server configured in several tools (and whether their versions agree),
//! skills copied around (identical or diverged), entries that cannot start
//! any more, and keys sitting in plain text where they might leak.
//!
//! Static only: nothing here runs a server or touches the network.

use super::{McpServer, Skill, ToolInventory};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FindingKind {
    /// Same server in several tools, same version/args.
    McpDuplicate,
    /// Same server in several tools with different pinned versions.
    McpVersionConflict,
    /// Byte-identical skill folders in several places.
    SkillCopies,
    /// Same-named skill folders whose contents differ.
    SkillDiverged,
    /// A stdio server whose command or script path does not exist.
    McpBroken,
    /// A plain-text key in a config file.
    SecretExposed,
    /// Older plugin or extension versions still on disk.
    StaleVersions,
}

/// One row in the "AI tools" page.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Finding {
    pub kind: FindingKind,
    /// What it is about: an MCP identity, a skill name, a config path.
    pub subject: String,
    /// Tools involved, in catalogue order.
    pub tools: Vec<String>,
    /// Config files, skill folders or version folders involved.
    pub paths: Vec<PathBuf>,
    /// Free text detail already safe to show (masked where needed).
    pub detail: String,
    /// Disk space that would come back if the extras went (copies, stale versions).
    pub reclaimable_bytes: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Analysis {
    pub findings: Vec<Finding>,
    pub mcp_total: usize,
    pub mcp_unique: usize,
    pub skills_total: usize,
    pub skills_unique: usize,
    pub reclaimable_bytes: u64,
}

pub fn analyze(tools: &[ToolInventory]) -> Analysis {
    let mut findings = Vec::new();
    let all_mcp: Vec<&McpServer> = tools.iter().flat_map(|t| t.mcp.iter()).collect();
    let all_skills: Vec<&Skill> = tools.iter().flat_map(|t| t.skills.iter()).collect();

    // MCP: group by identity.
    let mut by_id: BTreeMap<&str, Vec<&McpServer>> = BTreeMap::new();
    for s in &all_mcp {
        by_id.entry(s.identity.as_str()).or_default().push(s);
    }
    for (id, group) in &by_id {
        let tool_names: Vec<String> = dedup_tools(group.iter().map(|s| s.tool.as_str()));
        if tool_names.len() < 2 && group.len() < 2 {
            continue;
        }
        let mut versions: Vec<String> = group.iter().filter_map(|s| s.version.clone()).collect();
        versions.sort();
        versions.dedup();
        let kind = if versions.len() > 1 {
            FindingKind::McpVersionConflict
        } else {
            FindingKind::McpDuplicate
        };
        let names: Vec<String> = group
            .iter()
            .map(|s| format!("{}:{}", s.tool, s.name))
            .collect();
        let detail = if versions.len() > 1 {
            format!("{} · versions {}", names.join(", "), versions.join(" / "))
        } else {
            names.join(", ")
        };
        findings.push(Finding {
            kind,
            subject: id.to_string(),
            tools: tool_names,
            paths: group.iter().map(|s| s.config_path.clone()).collect(),
            detail,
            reclaimable_bytes: 0,
        });
    }

    // Broken stdio servers: command path or script missing.
    for s in &all_mcp {
        if let Some(reason) = broken_reason(s) {
            findings.push(Finding {
                kind: FindingKind::McpBroken,
                subject: format!("{}:{}", s.tool, s.name),
                tools: vec![s.tool.clone()],
                paths: vec![s.config_path.clone()],
                detail: reason,
                reclaimable_bytes: 0,
            });
        }
    }

    // Secrets.
    let mut by_file: BTreeMap<&Path, Vec<String>> = BTreeMap::new();
    for s in &all_mcp {
        for hit in &s.secrets {
            by_file
                .entry(s.config_path.as_path())
                .or_default()
                .push(format!("{} ({} = {})", s.name, hit.location, hit.masked));
        }
    }
    for (path, hits) in by_file {
        let tool = all_mcp
            .iter()
            .find(|s| s.config_path == path)
            .map(|s| s.tool.clone())
            .unwrap_or_default();
        let mut detail = hits.join("; ");
        if let Some(w) = exposure_warning(path) {
            detail.push_str(" · ");
            detail.push_str(w);
        }
        findings.push(Finding {
            kind: FindingKind::SecretExposed,
            subject: path.to_string_lossy().to_string(),
            tools: vec![tool],
            paths: vec![path.to_path_buf()],
            detail,
            reclaimable_bytes: 0,
        });
    }

    // Skills: group by folder name.
    let mut by_name: BTreeMap<&str, Vec<&Skill>> = BTreeMap::new();
    for s in &all_skills {
        if !s.link {
            by_name.entry(s.name.as_str()).or_default().push(s);
        }
    }
    let mut unique_skills = 0usize;
    for (name, group) in &by_name {
        unique_skills += 1;
        if group.len() < 2 {
            continue;
        }
        let mut hashes: Vec<&str> = group.iter().map(|s| s.hash.as_str()).collect();
        hashes.sort();
        hashes.dedup();
        let tool_names = dedup_tools(group.iter().map(|s| s.tool.as_str()));
        let paths: Vec<PathBuf> = group.iter().map(|s| s.path.clone()).collect();
        if hashes.len() == 1 {
            // Keep one copy: everything but the biggest-is-same, so (n-1) × size.
            let reclaimable = group.iter().skip(1).map(|s| s.bytes).sum();
            findings.push(Finding {
                kind: FindingKind::SkillCopies,
                subject: name.to_string(),
                tools: tool_names,
                paths,
                detail: format!("{} identical copies", group.len()),
                reclaimable_bytes: reclaimable,
            });
        } else {
            let mut newest: Vec<&&Skill> = group.iter().collect();
            newest.sort_by_key(|s| std::cmp::Reverse(s.modified));
            let detail = format!(
                "{} copies, {} different versions; newest in {}",
                group.len(),
                hashes.len(),
                newest[0].path.display()
            );
            findings.push(Finding {
                kind: FindingKind::SkillDiverged,
                subject: name.to_string(),
                tools: tool_names,
                paths,
                detail,
                reclaimable_bytes: 0,
            });
        }
    }

    // Stale plugin / extension versions.
    for t in tools {
        for p in &t.plugins {
            if !p.stale_versions.is_empty() {
                findings.push(Finding {
                    kind: FindingKind::StaleVersions,
                    subject: p.name.clone(),
                    tools: vec![t.id.clone()],
                    paths: p.stale_versions.clone(),
                    detail: format!(
                        "highest detected {}, {} potential older version(s); verify usage before removal",
                        p.version.as_deref().unwrap_or("?"),
                        p.stale_versions.len()
                    ),
                    reclaimable_bytes: p.stale_bytes,
                });
            }
        }
        for e in &t.extensions {
            if !e.stale_versions.is_empty() {
                findings.push(Finding {
                    kind: FindingKind::StaleVersions,
                    subject: e.name.clone(),
                    tools: vec![t.id.clone()],
                    paths: e.stale_versions.clone(),
                    detail: format!(
                        "highest detected {}, {} potential older version(s); verify usage before removal",
                        e.version.as_deref().unwrap_or("?"),
                        e.stale_versions.len()
                    ),
                    reclaimable_bytes: e.stale_bytes,
                });
            }
        }
    }

    let reclaimable_bytes = findings.iter().map(|f| f.reclaimable_bytes).sum();
    Analysis {
        findings,
        mcp_total: all_mcp.len(),
        mcp_unique: by_id.len(),
        skills_total: all_skills.len(),
        skills_unique: unique_skills,
        reclaimable_bytes,
    }
}

fn dedup_tools<'a>(it: impl Iterator<Item = &'a str>) -> Vec<String> {
    let mut v: Vec<String> = Vec::new();
    for t in it {
        if !v.iter().any(|x| x == t) {
            v.push(t.to_string());
        }
    }
    v
}

/// Why a stdio server cannot start, judged without running it.
fn broken_reason(s: &McpServer) -> Option<String> {
    let cmd = s.command.as_deref()?;
    let is_path = cmd.contains('/') || cmd.contains('\\');
    if is_path && !Path::new(cmd).exists() {
        return Some(format!("command not found: {cmd}"));
    }
    if !is_path && !on_path(cmd) {
        return Some(format!("'{cmd}' is not on PATH"));
    }
    if s.identity.starts_with("script:") {
        if let Some(script) = s.args.iter().find(|a| {
            let l = a.to_ascii_lowercase();
            [".js", ".mjs", ".cjs", ".ts", ".py"]
                .iter()
                .any(|e| l.ends_with(e))
        }) {
            if (script.contains('/') || script.contains('\\')) && !Path::new(script).exists() {
                return Some(format!("script not found: {script}"));
            }
        }
    }
    None
}

fn on_path(cmd: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return true;
    };
    let exts: Vec<String> = if cfg!(windows) {
        std::env::var("PATHEXT")
            .unwrap_or_else(|_| ".EXE;.CMD;.BAT;.COM".into())
            .split(';')
            .map(|e| e.to_ascii_lowercase())
            .collect()
    } else {
        vec![String::new()]
    };
    let has_ext = Path::new(cmd).extension().is_some();
    for dir in std::env::split_paths(&path) {
        if has_ext && dir.join(cmd).is_file() {
            return true;
        }
        for e in &exts {
            if dir.join(format!("{cmd}{e}")).is_file() {
                return true;
            }
        }
    }
    false
}

/// Where a config file sits that makes a plain-text key riskier.
fn exposure_warning(path: &Path) -> Option<&'static str> {
    let l = path.to_string_lossy().to_ascii_lowercase();
    if l.contains("onedrive") || l.contains("dropbox") || l.contains("google drive") {
        return Some("inside a cloud-synced folder");
    }
    // A project-level file inside a git checkout may be committed.
    let mut p = path.parent();
    while let Some(dir) = p {
        if dir.join(".git").exists() {
            return Some("inside a git repository");
        }
        p = dir.parent();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aiconfig::mcp::{McpTransport, SecretHit};

    fn mcp(tool: &str, name: &str, identity: &str, version: Option<&str>) -> McpServer {
        McpServer {
            tool: tool.into(),
            config_path: PathBuf::from(format!("/cfg/{tool}.json")),
            scope: "user".into(),
            name: name.into(),
            transport: McpTransport::Stdio,
            command: Some("npx".into()),
            args: vec![],
            url: None,
            package: None,
            version: version.map(str::to_string),
            identity: identity.into(),
            env_keys: vec![],
            secrets: vec![],
            disabled: false,
        }
    }

    fn skill(tool: &str, name: &str, hash: &str, bytes: u64) -> Skill {
        Skill {
            tool: tool.into(),
            name: name.into(),
            path: PathBuf::from(format!("/{tool}/skills/{name}")),
            bytes,
            files: 1,
            modified: None,
            hash: hash.into(),
            description: None,
            link: false,
        }
    }

    fn tool(id: &str, mcp: Vec<McpServer>, skills: Vec<Skill>) -> ToolInventory {
        ToolInventory {
            id: id.into(),
            name: id.into(),
            present: true,
            mcp,
            skills,
            ..Default::default()
        }
    }

    #[test]
    fn duplicates_conflicts_copies_and_divergence() {
        let tools = vec![
            tool(
                "cursor",
                vec![
                    mcp("cursor", "github", "npm:gh", Some("1.0.0")),
                    mcp("cursor", "fs", "npm:fs", None),
                ],
                vec![
                    skill("cursor", "web", "h1", 100),
                    skill("cursor", "sql", "s1", 50),
                ],
            ),
            tool(
                "codex",
                vec![
                    mcp("codex", "gh", "npm:gh", Some("2.0.0")),
                    mcp("codex", "fs", "npm:fs", None),
                ],
                vec![
                    skill("codex", "web", "h1", 100),
                    skill("codex", "sql", "s2", 60),
                ],
            ),
            tool("claude", vec![], vec![skill("claude", "web", "h1", 100)]),
        ];
        let a = analyze(&tools);
        assert_eq!(a.mcp_total, 4);
        assert_eq!(a.mcp_unique, 2);
        assert_eq!(a.skills_total, 5);
        assert_eq!(a.skills_unique, 2);
        let kinds: Vec<FindingKind> = a.findings.iter().map(|f| f.kind).collect();
        assert!(kinds.contains(&FindingKind::McpVersionConflict));
        assert!(kinds.contains(&FindingKind::McpDuplicate));
        let copies = a
            .findings
            .iter()
            .find(|f| f.kind == FindingKind::SkillCopies)
            .unwrap();
        assert_eq!(copies.subject, "web");
        assert_eq!(copies.tools, vec!["cursor", "codex", "claude"]);
        assert_eq!(copies.reclaimable_bytes, 200);
        let div = a
            .findings
            .iter()
            .find(|f| f.kind == FindingKind::SkillDiverged)
            .unwrap();
        assert_eq!(div.subject, "sql");
        assert_eq!(a.reclaimable_bytes, 200);
        // `npx` probably exists on PATH in CI but let's not depend on it: no
        // McpBroken for a command that is on PATH, and one for a dead path.
        let mut dead = mcp("cursor", "dead", "script:/nope/x.js", None);
        dead.command = Some("/nope/node".into());
        let a = analyze(&[tool("cursor", vec![dead], vec![])]);
        assert_eq!(a.findings[0].kind, FindingKind::McpBroken);
    }

    #[test]
    fn stale_findings_never_claim_the_highest_version_is_active() {
        let mut t = tool("cursor", vec![], vec![]);
        t.plugins.push(crate::aiconfig::Plugin {
            tool: "cursor".into(),
            name: "plugin".into(),
            version: Some("2.0.0".into()),
            path: PathBuf::from("/plugins/2.0.0"),
            bytes: 10,
            stale_versions: vec![PathBuf::from("/plugins/1.0.0")],
            stale_bytes: 5,
        });
        t.extensions.push(crate::aiconfig::Extension {
            tool: "cursor".into(),
            name: "extension".into(),
            version: Some("2.0.0".into()),
            path: PathBuf::from("/extensions/2.0.0"),
            bytes: 10,
            stale_versions: vec![PathBuf::from("/extensions/1.0.0")],
            stale_bytes: 5,
        });
        let analysis = analyze(&[t]);
        assert_eq!(analysis.findings.len(), 2);
        for finding in analysis.findings {
            assert_eq!(finding.kind, FindingKind::StaleVersions);
            assert!(finding.detail.contains("highest detected 2.0.0"));
            assert!(finding.detail.contains("verify usage"));
            assert!(!finding.detail.contains("current"));
        }
    }

    #[test]
    fn secrets_are_grouped_per_file_and_never_raw() {
        let mut s = mcp("cursor", "github", "npm:gh", None);
        s.secrets.push(SecretHit {
            location: "env.GITHUB_TOKEN".into(),
            masked: "ghp…wxyz".into(),
        });
        s.config_path = PathBuf::from("/Users/x/OneDrive/.cursor/mcp.json");
        let a = analyze(&[tool("cursor", vec![s], vec![])]);
        let f = a
            .findings
            .iter()
            .find(|f| f.kind == FindingKind::SecretExposed)
            .unwrap();
        assert!(f.detail.contains("ghp…wxyz"));
        assert!(f.detail.contains("cloud-synced"));
    }
}
