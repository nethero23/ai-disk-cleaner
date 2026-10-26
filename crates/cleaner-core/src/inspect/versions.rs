//! Versioned folders: Playwright's `chromium-1140`, VS Code-family
//! `publisher.name-1.2.3`, Gemini/Claude Desktop extensions and Claude Code's
//! plugin cache `cache/<marketplace>/<plugin>/<version>`. A lower version or
//! `.obsolete` marker is only a cleanup hint, not proof that a folder is unused.
//! Candidates require review and never qualify for automatic cleanup.

use super::read_small;
use crate::aiconfig::skills::{group_plugin_versions, group_versions, sorted_subdirs};
use crate::model::{SafetyLevel, ScanItem};
use crate::rules::Rule;
use crate::scan::{make_item, tree_stats, ScanOptions};
use std::collections::HashSet;
use std::path::Path;

fn old_detail(opts: &ScanOptions, newest: Option<&str>) -> String {
    match (opts.lang.as_str(), newest) {
        ("en", Some(v)) => {
            format!("Potential old version; highest detected version is {v}; review usage")
        }
        ("en", None) => "Marked in .obsolete; review usage before removal".to_string(),
        (_, Some(v)) => format!("可能的旧版本；检测到的最高版本 {v}；请确认是否仍在使用"),
        (_, None) => "被 .obsolete 标记；删除前请确认是否仍在使用".to_string(),
    }
}

fn old_consequence(opts: &ScanOptions) -> String {
    if opts.lang == "en" {
        "May still be pinned or used by a project or tool; removal can break it. Verify usage first.".to_string()
    } else {
        "可能仍被项目或工具固定版本或使用；删除可能导致功能失效，请先确认使用情况。".to_string()
    }
}

fn label(name: &str, path: &Path) -> String {
    let dir = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    if dir == name {
        dir
    } else {
        format!("{name} ({dir})")
    }
}

/// VS Code writes `.obsolete` (`{"<dir>": true}`) for extensions it has
/// already replaced but could not delete while running.
fn obsolete_dirs(root: &Path) -> HashSet<String> {
    read_small(&root.join(".obsolete"))
        .and_then(|t| serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&t).ok())
        .map(|m| {
            m.into_iter()
                .filter(|(_, v)| v.as_bool() == Some(true))
                .map(|(k, _)| k)
                .collect()
        })
        .unwrap_or_default()
}

/// One item per child; potential older versions require review, others keep the rule's safety.
pub fn versioned(rule: &Rule, root: &Path, opts: &ScanOptions) -> Vec<ScanItem> {
    let mut items = Vec::new();
    for (name, (newest, version), older) in group_versions(sorted_subdirs(root)) {
        items.push(make_item(
            rule,
            &newest,
            label(&name, &newest),
            tree_stats(&newest),
            opts,
            version.as_ref().map(|v| {
                if opts.lang == "en" {
                    format!("Version {v}")
                } else {
                    format!("版本 {v}")
                }
            }),
        ));
        for p in older {
            items.push(old_item(rule, &p, &name, version.as_deref(), opts));
        }
    }
    items
}

/// Only potential older versions and `.obsolete` hints, always requiring review.
/// Numeric ordering does not identify the active version; a marker can include the highest version.
pub fn stale(rule: &Rule, root: &Path, opts: &ScanOptions) -> Vec<ScanItem> {
    let mut items = Vec::new();
    let cache = root.join("cache");
    if cache.is_dir() {
        // Claude Code plugins: cache/<marketplace>/<plugin>/<version>/
        for market in sorted_subdirs(&cache) {
            for plugin in sorted_subdirs(&market) {
                let versions = sorted_subdirs(&plugin);
                if versions.len() < 2 {
                    continue;
                }
                let name = format!(
                    "{}/{}",
                    market.file_name().unwrap_or_default().to_string_lossy(),
                    plugin.file_name().unwrap_or_default().to_string_lossy()
                );
                for (_, (_, version), older) in group_plugin_versions(&name, versions) {
                    for p in older {
                        items.push(old_item(rule, &p, &name, version.as_deref(), opts));
                    }
                }
            }
        }
        return items;
    }
    let obsolete = obsolete_dirs(root);
    for (name, (newest, version), older) in group_versions(sorted_subdirs(root)) {
        let newest_dir = newest
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        if obsolete.contains(&newest_dir) {
            items.push(old_item(rule, &newest, &name, None, opts));
        }
        for p in older {
            items.push(old_item(rule, &p, &name, version.as_deref(), opts));
        }
    }
    items
}

fn old_item(
    rule: &Rule,
    p: &Path,
    name: &str,
    newest: Option<&str>,
    opts: &ScanOptions,
) -> ScanItem {
    let mut item = make_item(
        rule,
        p,
        label(name, p),
        tree_stats(p),
        opts,
        Some(old_detail(opts, newest)),
    );
    item.safety = SafetyLevel::Review;
    item.auto_clean = false;
    item.consequence = old_consequence(opts);
    item
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Category;
    use crate::paths::PathContext;
    use std::collections::HashMap;

    fn rule(id: &str) -> Rule {
        Rule {
            id: id.into(),
            tool: "t".into(),
            category: Category::PackageCache,
            name_zh: "n".into(),
            name_en: "n".into(),
            paths: vec![],
            all_paths: false,
            safety: SafetyLevel::Redownload,
            inspector: Default::default(),
            consequence_zh: "c".into(),
            consequence_en: "c".into(),
            min_bytes: 0,
            windows_only: false,
            processes: vec![],
            require: vec![],
            auto_clean: true,
        }
    }

    fn opts(home: &Path) -> ScanOptions {
        let mut vars = HashMap::new();
        vars.insert("USERPROFILE".to_string(), home.to_path_buf());
        ScanOptions {
            ctx: PathContext::with_vars(vars),
            lang: "zh".into(),
            ui_lang: String::new(),
            global_min_bytes: 0,
            running: Default::default(),
            excluded: vec![],
            extra_roots: vec![],
        }
    }

    fn mk(root: &Path, rel: &str) {
        let p = root.join(rel);
        std::fs::create_dir_all(&p).unwrap();
        std::fs::write(p.join("f"), "data").unwrap();
    }

    #[test]
    fn playwright_old_builds_require_review_and_highest_keeps_rule_safety() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("ms-playwright");
        for d in [
            "chromium-1140",
            "chromium-1148",
            "chromium_headless_shell-1148",
            "ffmpeg-1010",
        ] {
            mk(&root, d);
        }
        let items = versioned(&rule("playwright.browsers"), &root, &opts(tmp.path()));
        assert_eq!(items.len(), 4);
        let old = items
            .iter()
            .find(|i| i.path.ends_with("chromium-1140"))
            .unwrap();
        assert_eq!(old.safety, SafetyLevel::Review);
        assert!(!old.auto_clean);
        assert_eq!(
            old.detail.as_deref(),
            Some("可能的旧版本；检测到的最高版本 1148；请确认是否仍在使用")
        );
        let cur = items
            .iter()
            .find(|i| i.path.ends_with("chromium-1148"))
            .unwrap();
        assert_eq!(cur.safety, SafetyLevel::Redownload);
        assert!(cur.auto_clean);
        assert_eq!(
            items
                .iter()
                .filter(|i| i.safety == SafetyLevel::Review)
                .count(),
            1
        );
    }

    #[test]
    fn stale_lists_only_old_extension_versions_and_obsolete_ones() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("extensions");
        for d in [
            "ms-python.python-2024.1.0",
            "ms-python.python-2024.3.1",
            "other.ext-1.0.0",
            "dead.ext-0.1.0",
        ] {
            mk(&root, d);
        }
        std::fs::write(root.join(".obsolete"), r#"{"dead.ext-0.1.0": true}"#).unwrap();
        let items = stale(&rule("cursor.extensions_stale"), &root, &opts(tmp.path()));
        let mut names: Vec<String> = items
            .iter()
            .map(|i| i.path.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        names.sort();
        assert_eq!(names, vec!["dead.ext-0.1.0", "ms-python.python-2024.1.0"]);
        assert!(items
            .iter()
            .all(|i| i.safety == SafetyLevel::Review && !i.auto_clean));
    }

    #[test]
    fn stale_handles_claude_plugin_cache_layout() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join(".claude/plugins");
        mk(&root, "cache/anthropic/design/1.2.0");
        mk(&root, "cache/anthropic/design/1.10.0");
        mk(&root, "cache/anthropic/single/2.0.0");
        let items = stale(&rule("claude_code.plugins_stale"), &root, &opts(tmp.path()));
        assert_eq!(items.len(), 1);
        assert!(items[0].path.ends_with("design/1.2.0"));
        assert_eq!(items[0].name, "anthropic/design (1.2.0)");
        assert_eq!(
            items[0].detail.as_deref(),
            Some("可能的旧版本；检测到的最高版本 1.10.0；请确认是否仍在使用")
        );
        assert_eq!(items[0].safety, SafetyLevel::Review);
        assert!(!items[0].auto_clean);
    }

    #[test]
    fn pinned_older_plugin_is_only_a_manual_review_candidate() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("plugins");
        mk(&root, "cache/market/pinned/1.0.0");
        mk(&root, "cache/market/pinned/2.0.0");
        // The inspector does not resolve every project's pins. A higher folder
        // number must never be treated as proof this installation is unused.
        std::fs::write(root.join("installed_plugins.json"), r#"{"version":2,"plugins":{"pinned@market":[{"scope":"user","version":"1.0.0","installPath":"cache/market/pinned/1.0.0"}]}}"#).unwrap();
        let mut options = opts(tmp.path());
        options.lang = "en".into();
        let mut permissive = rule("plugins_stale");
        permissive.safety = SafetyLevel::Safe;
        let items = stale(&permissive, &root, &options);
        assert_eq!(items.len(), 1);
        let item = &items[0];
        assert!(item.path.ends_with("pinned/1.0.0"));
        assert_eq!(item.safety, SafetyLevel::Review);
        assert!(!item.auto_clean);
        assert_eq!(
            item.detail.as_deref(),
            Some("Potential old version; highest detected version is 2.0.0; review usage")
        );
        assert!(item.consequence.contains("pinned or used"));
        assert!(!item.consequence.contains("Nothing uses"));
    }

    #[test]
    fn platform_variants_are_not_stale_replacements_for_each_other() {
        let tmp = tempfile::tempdir().unwrap();
        for d in [
            "ext-1.0.0-win32-x64",
            "ext-2.0.0-win32-x64",
            "ext-2.0.0-linux-x64",
            "ext-1.0.0-darwin-arm64",
            "ext-2.0.0",
        ] {
            mk(tmp.path(), d);
        }
        let items = stale(&rule("extensions_stale"), tmp.path(), &opts(tmp.path()));
        assert_eq!(items.len(), 1);
        assert!(items[0].path.ends_with("ext-1.0.0-win32-x64"));
        assert_eq!(items[0].safety, SafetyLevel::Review);
        assert!(!items[0].auto_clean);
        assert_eq!(
            versioned(&rule("extensions"), tmp.path(), &opts(tmp.path())).len(),
            5
        );
    }

    #[test]
    fn obsolete_highest_and_unknown_names_require_review_without_current_claim() {
        let tmp = tempfile::tempdir().unwrap();
        for d in ["ext-1.0.0", "ext-2.0.0", "unknown-build", "unmarked-build"] {
            mk(tmp.path(), d);
        }
        std::fs::write(
            tmp.path().join(".obsolete"),
            r#"{"ext-2.0.0":true,"unknown-build":true,"unmarked-build":false,"../outside":true}"#,
        )
        .unwrap();
        let mut options = opts(tmp.path());
        options.lang = "en".into();
        let items = stale(&rule("extensions_stale"), tmp.path(), &options);
        assert_eq!(items.len(), 3);
        assert!(items
            .iter()
            .all(|i| i.safety == SafetyLevel::Review && !i.auto_clean));
        for name in ["ext-2.0.0", "unknown-build"] {
            let item = items.iter().find(|i| i.path.ends_with(name)).unwrap();
            assert_eq!(
                item.detail.as_deref(),
                Some("Marked in .obsolete; review usage before removal")
            );
        }
        assert!(items
            .iter()
            .all(|i| !i.detail.as_deref().unwrap().contains("current")));
    }

    #[test]
    fn unknown_and_equal_plugin_versions_are_not_inferred_stale() {
        let tmp = tempfile::tempdir().unwrap();
        for version in ["1.0", "1.0.0", "v1.0.0", "abcdef12", "latest", "2.0.0-beta"] {
            mk(tmp.path(), &format!("cache/market/plugin/{version}"));
        }
        assert!(stale(&rule("plugins_stale"), tmp.path(), &opts(tmp.path())).is_empty());
    }
}
