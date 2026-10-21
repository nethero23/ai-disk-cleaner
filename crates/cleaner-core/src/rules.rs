//! Rule library: declarative descriptions of where junk lives.
//!
//! Rules ship inside the binary (`rules/*.toml`, embedded at compile time) so
//! a fresh install works offline; a later version will also load newer rule
//! files from the app data folder.

use crate::model::{Category, SafetyLevel};
use serde::{Deserialize, Serialize};

/// Which inspector names the contents of a location. `Plain` just sizes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Inspector {
    #[default]
    Plain,
    /// One item per immediate child folder (e.g. each Cursor workspace, each conda env).
    Children,
    OllamaModels,
    HuggingFaceHub,
    LmStudioModels,
    CursorWorkspaces,
    CondaEnvs,
    ClaudeProjects,
    /// Walks the user's folders for node_modules / .venv / target next to a manifest.
    ProjectArtifacts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    /// Stable, dotted: `ollama.models`, `pip.cache`.
    pub id: String,
    pub tool: String,
    pub category: Category,
    /// Display name, Chinese.
    pub name_zh: String,
    /// Display name, English.
    pub name_en: String,
    /// Path templates, tried in order; the first that exists wins unless `all_paths`.
    pub paths: Vec<String>,
    #[serde(default)]
    pub all_paths: bool,
    pub safety: SafetyLevel,
    #[serde(default)]
    pub inspector: Inspector,
    /// Consequence of deletion, Chinese ("工具会自动重建").
    pub consequence_zh: String,
    pub consequence_en: String,
    /// Only report when at least this big, to keep the list readable.
    #[serde(default)]
    pub min_bytes: u64,
    /// Windows-only rule (skipped on other platforms even if the path exists).
    #[serde(default)]
    pub windows_only: bool,
    /// Executable names (without path; `.exe` optional) that own this data.
    /// While one is running the items are marked in use and not deleted.
    #[serde(default)]
    pub processes: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RuleSet {
    #[serde(rename = "rule")]
    pub rules: Vec<Rule>,
}

/// The rule files compiled into the binary.
const BUILTIN: &[(&str, &str)] = &[
    ("models.toml", include_str!("../rules/models.toml")),
    ("python.toml", include_str!("../rules/python.toml")),
    ("ai-editors.toml", include_str!("../rules/ai-editors.toml")),
    ("ai-agents.toml", include_str!("../rules/ai-agents.toml")),
    ("ai-apps.toml", include_str!("../rules/ai-apps.toml")),
    ("javascript.toml", include_str!("../rules/javascript.toml")),
    ("containers.toml", include_str!("../rules/containers.toml")),
    ("gpu.toml", include_str!("../rules/gpu.toml")),
    ("browsers.toml", include_str!("../rules/browsers.toml")),
    ("system.toml", include_str!("../rules/system.toml")),
    ("projects.toml", include_str!("../rules/projects.toml")),
];

impl RuleSet {
    pub fn parse(text: &str) -> crate::Result<Self> {
        Ok(toml::from_str(text)?)
    }

    /// All rules shipped with this build.
    pub fn builtin() -> Self {
        let mut all = RuleSet::default();
        for (name, text) in BUILTIN {
            let set = RuleSet::parse(text)
                .unwrap_or_else(|e| panic!("builtin rule file {name} is invalid: {e}"));
            all.rules.extend(set.rules);
        }
        all
    }

    pub fn get(&self, id: &str) -> Option<&Rule> {
        self.rules.iter().find(|r| r.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn builtin_rules_parse_and_have_unique_ids() {
        let set = RuleSet::builtin();
        assert!(
            set.rules.len() >= 30,
            "expected a real rule library, got {}",
            set.rules.len()
        );
        let mut seen = HashSet::new();
        for r in &set.rules {
            assert!(seen.insert(&r.id), "duplicate rule id {}", r.id);
            assert!(!r.paths.is_empty(), "rule {} has no paths", r.id);
            assert!(r.id.contains('.'), "rule id {} should be dotted", r.id);
        }
    }

    #[test]
    fn review_rules_never_claim_to_be_harmless() {
        for r in RuleSet::builtin().rules {
            if r.safety == SafetyLevel::Review {
                assert!(
                    !r.consequence_zh.contains("自动重建"),
                    "{} is Review but says it auto-rebuilds",
                    r.id
                );
            }
        }
    }
}
