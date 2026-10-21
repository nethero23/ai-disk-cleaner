//! The tool catalogue: `aitools.toml` embedded at build time, same shape as
//! the cleaning rules so it can be served from the rules URL later.

use crate::paths::PathContext;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpSource {
    /// Path template; may contain one `*` segment (`connectors/*/mcp.json`).
    pub path: String,
    /// Key the servers live under.
    pub root: String,
    /// "json" (default) or "toml".
    #[serde(default = "default_format")]
    pub format: String,
    /// Claude Code's `~/.claude.json` also nests `projects.<dir>.mcpServers`.
    #[serde(default)]
    pub projects: bool,
}

fn default_format() -> String {
    "json".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDef {
    pub id: String,
    pub name: String,
    #[serde(default = "default_true")]
    pub verified: bool,
    #[serde(default)]
    pub processes: Vec<String>,
    #[serde(default)]
    pub mcp: Vec<McpSource>,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub plugins: Option<String>,
    #[serde(default)]
    pub extensions: Vec<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Catalog {
    #[serde(default, rename = "tool")]
    pub tools: Vec<ToolDef>,
}

pub const BUILTIN: &str = include_str!("../../aitools.toml");

impl Catalog {
    pub fn builtin() -> Self {
        toml::from_str(BUILTIN).expect("aitools.toml is valid")
    }

    pub fn parse(text: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(text)
    }
}

/// Expands a template, resolving a single `*` directory segment against the
/// filesystem. Without a star this is `ctx.expand` in a one-element vector.
pub fn expand_glob(ctx: &PathContext, template: &str) -> Vec<PathBuf> {
    let Some(star) = template.find('*') else {
        return ctx.expand(template).into_iter().collect();
    };
    let (head, tail) = template.split_at(star);
    let tail = tail.trim_start_matches('*').trim_start_matches(['/', '\\']);
    let Some(base) = ctx.expand(head.trim_end_matches(['/', '\\'])) else {
        return Vec::new();
    };
    let Ok(rd) = std::fs::read_dir(&base) else {
        return Vec::new();
    };
    let mut out: Vec<PathBuf> = rd
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| e.path().join(tail))
        .collect();
    out.sort();
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn builtin_catalog_parses_and_has_unique_ids() {
        let c = Catalog::builtin();
        assert!(c.tools.len() >= 20);
        let mut ids: Vec<&str> = c.tools.iter().map(|t| t.id.as_str()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), c.tools.len());
        for t in &c.tools {
            for m in &t.mcp {
                assert!(["json", "toml"].contains(&m.format.as_str()), "{}", t.id);
                assert!(m.path.contains("${"), "{}", t.id);
            }
        }
    }

    #[test]
    fn glob_expands_one_star_segment() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("c/a")).unwrap();
        std::fs::create_dir_all(tmp.path().join("c/b")).unwrap();
        std::fs::write(tmp.path().join("c/file"), "").unwrap();
        let mut vars = HashMap::new();
        vars.insert("USERPROFILE".to_string(), tmp.path().to_path_buf());
        let ctx = PathContext::with_vars(vars);
        let got = expand_glob(&ctx, "${USERPROFILE}/c/*/mcp.json");
        assert_eq!(got.len(), 2);
        assert!(got[0].ends_with("a/mcp.json"));
        assert_eq!(expand_glob(&ctx, "${USERPROFILE}/plain.json").len(), 1);
        assert!(expand_glob(&ctx, "${NOPE}/x/*/y").is_empty());
    }
}
