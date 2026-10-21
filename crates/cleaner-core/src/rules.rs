//! Rule library: declarative descriptions of where junk lives.
//!
//! Rules ship inside the binary (`rules/*.toml`, embedded at compile time) so
//! a fresh install works offline; a later version will also load newer rule
//! files from the app data folder.

use crate::model::{Category, SafetyLevel};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

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
    /// Like `Children`, but folders named `<name>-<version>` are grouped and
    /// every version except the newest is marked Safe ("old version").
    VersionedChildren,
    /// Only the old versions of `<name>-<version>` folders (and of Claude
    /// Code's `cache/<marketplace>/<plugin>/<version>` layout); the current
    /// version of anything is never listed.
    StaleVersions,
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
    #[serde(default)]
    pub consequence_zh: String,
    #[serde(default)]
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
    /// For the `children` inspector: only report subfolders containing one of
    /// these relative paths (e.g. `LocalState/ext4.vhdx` picks WSL distros out
    /// of every Store app's package folder).
    #[serde(default)]
    pub require: Vec<String>,
    /// False for things the app must never delete unattended (a project's
    /// `.venv`, editor workspace state): still shown, never auto-cleaned and
    /// not pre-ticked by "clean safe items".
    #[serde(default = "default_true")]
    pub auto_clean: bool,
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

    /// File names of the builtin rule files, for the updater.
    pub fn builtin_file_names() -> Vec<&'static str> {
        BUILTIN.iter().map(|(n, _)| *n).collect()
    }

    /// Every `*.toml` in `dir`, in name order. A file that fails to parse is
    /// reported in the second half and the rest still load.
    pub fn from_dir(dir: &Path) -> (Self, Vec<(PathBuf, String)>) {
        let mut set = RuleSet::default();
        let mut errors = Vec::new();
        let Ok(rd) = std::fs::read_dir(dir) else {
            return (set, errors);
        };
        let mut files: Vec<PathBuf> = rd
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "toml"))
            .collect();
        files.sort();
        for f in files {
            match std::fs::read_to_string(&f)
                .map_err(|e| e.to_string())
                .and_then(|t| RuleSet::parse(&t).map_err(|e| e.to_string()))
            {
                Ok(s) => set.rules.extend(s.rules),
                Err(e) => errors.push((f, e)),
            }
        }
        (set, errors)
    }

    /// Adds `other`'s rules; one with an id already present replaces it in
    /// place, so a downloaded or custom file can fix a builtin rule.
    pub fn merge(&mut self, other: RuleSet) {
        for r in other.rules {
            match self.rules.iter_mut().find(|x| x.id == r.id) {
                Some(slot) => *slot = r,
                None => self.rules.push(r),
            }
        }
    }

    /// Builtin, then the downloaded folder, then the user's folder; then the
    /// rules the user switched off are dropped.
    pub fn effective(
        downloaded_dir: Option<&Path>,
        settings: &crate::settings::Settings,
    ) -> (Self, Vec<(PathBuf, String)>) {
        let mut set = RuleSet::builtin();
        let mut errors = Vec::new();
        for dir in [downloaded_dir, settings.custom_rules_dir.as_deref()]
            .into_iter()
            .flatten()
        {
            let (extra, errs) = RuleSet::from_dir(dir);
            set.merge(extra);
            errors.extend(errs);
        }
        set.rules
            .retain(|r| !settings.disabled_rules.iter().any(|d| d == &r.id));
        (set, errors)
    }
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_and_workspace_rules_are_never_auto_cleaned() {
        let set = RuleSet::builtin();
        for id in [
            "projects.artifacts",
            "vscode.workspaces",
            "cursor.workspaces",
        ] {
            assert!(!set.get(id).unwrap().auto_clean, "{id}");
        }
        assert!(set.get("pip.cache").unwrap().auto_clean);
        assert!(!set
            .rules
            .iter()
            .any(|r| r.paths.iter().any(|p| p.ends_with(".obsolete"))));
    }
    use std::collections::HashSet;

    #[test]
    fn merge_overrides_by_id_and_dir_loads_toml() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("mine.toml"),
            r#"
[[rule]]
id = "pip.cache"
tool = "pip"
category = "package_cache"
name_zh = "改过的 pip"
name_en = "patched pip"
paths = ["${USERPROFILE}/nowhere"]
safety = "review"

[[rule]]
id = "my.tool"
tool = "Mine"
category = "system"
name_zh = "我的"
name_en = "mine"
paths = ["${USERPROFILE}/mine"]
safety = "safe"
"#,
        )
        .unwrap();
        std::fs::write(tmp.path().join("broken.toml"), "not toml [[").unwrap();
        let settings = crate::settings::Settings {
            disabled_rules: vec!["uv.cache".into()],
            ..Default::default()
        };
        let (set, errors) = RuleSet::effective(Some(tmp.path()), &settings);
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert_eq!(set.get("pip.cache").unwrap().name_en, "patched pip");
        assert!(set.get("my.tool").is_some());
        assert!(set.get("uv.cache").is_none());
        let n = set.rules.iter().filter(|r| r.id == "pip.cache").count();
        assert_eq!(n, 1);
    }

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
