//! Build artifacts inside the user's own project folders: `node_modules`,
//! `.venv`, Cargo `target`, `__pycache__`… They are not in any fixed cache
//! location, so this inspector walks the user's folders (bounded depth,
//! skipping AppData and known caches) and reports one item per artifact
//! folder, named after the project it belongs to.
//!
//! A project untouched for [`STALE_DAYS`] is Safe: its artifacts rebuild on
//! the next install. A project edited recently is Review: the user is
//! probably still working in it.

use crate::model::{SafetyLevel, ScanItem};
use crate::rules::Rule;
use crate::scan::{make_item, tree_stats, ScanOptions};
use chrono::{Duration, Utc};
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

/// Folder names that are artifacts, and the sibling file that proves it
/// (so a random folder called `target` is not reported).
const ARTIFACTS: &[(&str, &[&str])] = &[
    ("node_modules", &["package.json"]),
    (
        ".venv",
        &[
            "pyproject.toml",
            "requirements.txt",
            "setup.py",
            "Pipfile",
            "pyvenv.cfg",
        ],
    ),
    (
        "venv",
        &["pyproject.toml", "requirements.txt", "setup.py", "Pipfile"],
    ),
    ("target", &["Cargo.toml"]),
    (
        "build",
        &[
            "CMakeLists.txt",
            "build.gradle",
            "build.gradle.kts",
            "setup.py",
        ],
    ),
    (
        ".gradle",
        &["build.gradle", "build.gradle.kts", "settings.gradle"],
    ),
    (".next", &["package.json"]),
    (".nuxt", &["package.json"]),
    ("dist", &["package.json"]),
    (
        ".pytest_cache",
        &["pyproject.toml", "setup.py", "pytest.ini", "setup.cfg"],
    ),
    (
        ".mypy_cache",
        &["pyproject.toml", "setup.py", "setup.cfg", "mypy.ini"],
    ),
];

/// Folders never entered: other rules cover them, or they are not projects.
const SKIP_DIRS: &[&str] = &[
    "AppData",
    "Application Data",
    "Library",
    "Windows",
    "Program Files",
    "Program Files (x86)",
    "ProgramData",
    "$Recycle.Bin",
    "System Volume Information",
    ".cache",
    ".conda",
    ".ollama",
    ".lmstudio",
    ".cargo",
    ".rustup",
    ".npm",
    ".nvm",
    ".pnpm-store",
    ".local",
    ".config",
    ".vscode",
    ".cursor",
    ".claude",
    ".codex",
    ".gemini",
    ".docker",
    ".git",
    "miniconda3",
    "anaconda3",
    "miniforge3",
    "scoop",
    "go",
    "OneDrive",
    "iCloudDrive",
    "Dropbox",
];

/// Deeper than this and we are inside something, not looking for projects.
const MAX_DEPTH: usize = 7;
pub const STALE_DAYS: i64 = 90;

fn is_artifact_dir(entry: &DirEntry) -> bool {
    if !entry.file_type().is_dir() {
        return false;
    }
    let name = entry.file_name().to_string_lossy();
    let Some(parent) = entry.path().parent() else {
        return false;
    };
    ARTIFACTS
        .iter()
        .any(|(n, markers)| *n == name && markers.iter().any(|m| parent.join(m).exists()))
}

fn should_skip(entry: &DirEntry) -> bool {
    if !entry.file_type().is_dir() || entry.file_type().is_symlink() {
        return false;
    }
    let name = entry.file_name().to_string_lossy();
    // Dot folders in the home directory are tool installs, not projects:
    // `~/.qoder/extensions/x/dist` or `~/.some-cli/node_modules` is the tool
    // itself, and an old mtime there means "installed a while ago", not
    // "abandoned". Artifact folders that start with a dot are still matched.
    if name.starts_with('.') && !ARTIFACTS.iter().any(|(n, _)| *n == name) {
        return true;
    }
    SKIP_DIRS.iter().any(|s| *s == name)
}

/// Newest modification time among the project's own files (not the artifact),
/// sampled at the top level only: cheap, and good enough to tell "edited this
/// week" from "untouched since spring".
fn project_last_edit(project: &Path) -> Option<chrono::DateTime<Utc>> {
    let mut newest: Option<chrono::DateTime<Utc>> = None;
    let rd = std::fs::read_dir(project).ok()?;
    for e in rd.flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        if ARTIFACTS.iter().any(|(n, _)| *n == name) || name == ".git" {
            continue;
        }
        if let Ok(md) = e.metadata() {
            if let Ok(t) = md.modified() {
                let t: chrono::DateTime<Utc> = t.into();
                newest = Some(newest.map_or(t, |n| n.max(t)));
            }
        }
    }
    newest
}

pub fn artifacts(rule: &Rule, root: &Path, opts: &ScanOptions) -> Vec<ScanItem> {
    let mut items = Vec::new();
    let mut it = WalkDir::new(root)
        .follow_links(false)
        .max_depth(MAX_DEPTH)
        .into_iter();
    // An unreadable folder yields an Err; skip it rather than ending the walk,
    // or every project after the first locked folder goes unreported.
    while let Some(next) = it.next() {
        let Ok(entry) = next else { continue };
        if entry.depth() == 0 {
            continue;
        }
        if should_skip(&entry) {
            it.skip_current_dir();
            continue;
        }
        if !is_artifact_dir(&entry) {
            continue;
        }
        // Found one: size it, don't descend into it.
        it.skip_current_dir();
        let path = entry.path().to_path_buf();
        let project = path.parent().unwrap_or(&path).to_path_buf();
        let project_name = project
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| project.to_string_lossy().to_string());
        let artifact_name = entry.file_name().to_string_lossy().to_string();
        let mut stats = tree_stats(&path);
        let last_edit = project_last_edit(&project);
        stats.newest = last_edit.or(stats.newest);
        let stale = last_edit
            .map(|t| Utc::now() - t > Duration::days(STALE_DAYS))
            .unwrap_or(false);
        let name = format!("{project_name}/{artifact_name}");
        let detail = Some(if opts.lang == "en" {
            format!("Project: {}", project.display())
        } else {
            format!("项目：{}", project.display())
        });
        let mut item = make_item(rule, &path, name, stats, opts, detail);
        item.safety = if stale {
            SafetyLevel::Safe
        } else {
            SafetyLevel::Review
        };
        item.consequence = match (stale, opts.lang.as_str()) {
            (true, "en") => {
                format!("Project untouched for {STALE_DAYS}+ days; rebuilds on next install/build")
            }
            (true, _) => format!("项目 {STALE_DAYS} 天以上没改过；下次安装或构建时会重建"),
            (false, "en") => "Project edited recently; you probably still need this".into(),
            (false, _) => "项目最近改过，可能还在用".into(),
        };
        items.push(item);
    }
    items
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Category;
    use crate::paths::PathContext;
    use crate::rules::Inspector;
    use std::collections::HashMap;
    use std::fs;

    #[test]
    fn finds_node_modules_next_to_package_json_only() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();
        fs::create_dir_all(home.join("proj/node_modules/x")).unwrap();
        fs::write(home.join("proj/package.json"), "{}").unwrap();
        fs::write(home.join("proj/node_modules/x/a.js"), vec![0u8; 500]).unwrap();
        // A "node_modules" with no package.json beside it is not a project artifact.
        fs::create_dir_all(home.join("random/node_modules")).unwrap();
        fs::write(home.join("random/node_modules/b"), vec![0u8; 500]).unwrap();
        // AppData is never entered.
        fs::create_dir_all(home.join("AppData/proj2/node_modules")).unwrap();
        fs::write(home.join("AppData/proj2/package.json"), "{}").unwrap();
        fs::write(home.join("AppData/proj2/node_modules/c"), vec![0u8; 500]).unwrap();
        // Nor are dot folders: an editor extension's dist is the extension itself.
        fs::create_dir_all(home.join(".qoder/extensions/ext/dist")).unwrap();
        fs::write(home.join(".qoder/extensions/ext/package.json"), "{}").unwrap();
        fs::write(home.join(".qoder/extensions/ext/dist/d.js"), vec![0u8; 500]).unwrap();

        let rule = Rule {
            id: "projects.artifacts".into(),
            tool: "Projects".into(),
            category: Category::ProjectArtifacts,
            name_zh: "x".into(),
            name_en: "x".into(),
            paths: vec!["${USERPROFILE}".into()],
            all_paths: false,
            safety: SafetyLevel::Review,
            inspector: Inspector::ProjectArtifacts,
            consequence_zh: String::new(),
            consequence_en: String::new(),
            min_bytes: 0,
            windows_only: false,
            require: vec![],
            processes: vec![],
            auto_clean: false,
        };
        let mut vars = HashMap::new();
        vars.insert("USERPROFILE".to_string(), home.to_path_buf());
        let opts = ScanOptions {
            ctx: PathContext::with_vars(vars),
            lang: "zh".into(),
            global_min_bytes: 0,
            running: Default::default(),
            excluded: Vec::new(),
        };
        let items = artifacts(&rule, home, &opts);
        assert_eq!(items.len(), 1, "{items:?}");
        assert_eq!(items[0].name, "proj/node_modules");
        assert_eq!(items[0].size_bytes, 500);
        // Just created, so the project counts as active.
        assert_eq!(items[0].safety, SafetyLevel::Review);
    }
}
