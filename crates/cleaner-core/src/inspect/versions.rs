//! Versioned folders: Playwright's `chromium-1140`, VS Code-family
//! `publisher.name-1.2.3`, Gemini/Claude Desktop extensions and Claude Code's
//! plugin cache `cache/<marketplace>/<plugin>/<version>`. After an upgrade
//! the old version folder usually stays behind; it is Safe to delete, while
//! the newest version of each thing keeps the rule's own safety and is never
//! listed by [`stale`].

use super::read_small;
use crate::aiconfig::skills::{group_versions, sorted_subdirs};
use crate::model::{SafetyLevel, ScanItem};
use crate::rules::Rule;
use crate::scan::{make_item, tree_stats, ScanOptions};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

fn old_detail(opts: &ScanOptions, current: Option<&str>) -> String {
    match (opts.lang.as_str(), current) {
        ("en", Some(c)) => format!("Old version; current is {c}"),
        ("en", None) => "Old version".to_string(),
        (_, Some(c)) => format!("旧版本；当前版本 {c}"),
        (_, None) => "旧版本".to_string(),
    }
}

fn old_consequence(opts: &ScanOptions) -> String {
    if opts.lang == "en" {
        "Nothing uses it any more; the current version stays".to_string()
    } else {
        "已没有东西在用它，当前版本保留".to_string()
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

/// One item per child; older versions are Safe, the newest keeps the rule's safety.
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

/// Only the old versions; the current version of each thing is never listed.
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
                let (newest, older) = split_newest(versions);
                let cur = newest.file_name().map(|n| n.to_string_lossy().to_string());
                for p in older {
                    items.push(old_item(rule, &p, &name, cur.as_deref(), opts));
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

/// Newest by numeric version of the folder name, else by modification time.
fn split_newest(mut dirs: Vec<PathBuf>) -> (PathBuf, Vec<PathBuf>) {
    use crate::aiconfig::skills::version_key;
    dirs.sort_by(|a, b| {
        let ka = version_key(&a.file_name().unwrap_or_default().to_string_lossy());
        let kb = version_key(&b.file_name().unwrap_or_default().to_string_lossy());
        if ka != kb {
            return kb.cmp(&ka);
        }
        let ma = std::fs::metadata(a).and_then(|m| m.modified()).ok();
        let mb = std::fs::metadata(b).and_then(|m| m.modified()).ok();
        mb.cmp(&ma)
    });
    let newest = dirs.remove(0);
    (newest, dirs)
}

fn old_item(
    rule: &Rule,
    p: &Path,
    name: &str,
    current: Option<&str>,
    opts: &ScanOptions,
) -> ScanItem {
    let mut item = make_item(
        rule,
        p,
        label(name, p),
        tree_stats(p),
        opts,
        Some(old_detail(opts, current)),
    );
    item.safety = SafetyLevel::Safe;
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
    fn playwright_old_builds_are_safe_and_current_is_not() {
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
        assert_eq!(old.safety, SafetyLevel::Safe);
        assert_eq!(old.detail.as_deref(), Some("旧版本；当前版本 1148"));
        let cur = items
            .iter()
            .find(|i| i.path.ends_with("chromium-1148"))
            .unwrap();
        assert_eq!(cur.safety, SafetyLevel::Redownload);
        assert!(
            items
                .iter()
                .filter(|i| i.safety == SafetyLevel::Safe)
                .count()
                == 1
        );
    }

    #[test]
    fn stale_lists_only_old_extension_versions_and_obsolete_ones() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("extensions");
        for d in [
            "ms-python.python-2024.1.0",
            "ms-python.python-2024.3.1-win32-x64",
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
        assert!(items.iter().all(|i| i.safety == SafetyLevel::Safe));
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
        assert_eq!(items[0].detail.as_deref(), Some("旧版本；当前版本 1.10.0"));
    }
}
