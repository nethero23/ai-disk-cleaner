//! Big folders under the user's profile that no rule claims. The rule
//! library grows from these: the user sees "AppData\Local\SomeTool is 4 GB
//! and nobody knows what it is" and can open a prefilled GitHub issue.
//! Nothing is uploaded by itself; the issue body holds the folder name,
//! size and date, never a file list.

use crate::paths::PathContext;
use crate::rules::RuleSet;
use crate::scan::tree_stats;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Unrecognized {
    /// `AppData\Local\SomeTool`, relative to the profile, for display and issues.
    pub name: String,
    pub path: PathBuf,
    pub bytes: u64,
    pub files: u64,
    pub newest: Option<DateTime<Utc>>,
}

/// Profile folders that are the user's own documents or Windows' own: never "unrecognized".
const SKIP_NAMES: &[&str] = &[
    "desktop",
    "documents",
    "downloads",
    "pictures",
    "videos",
    "music",
    "saved games",
    "contacts",
    "favorites",
    "links",
    "searches",
    "3d objects",
    "appdata",
    "application data",
    "local settings",
    "microsoft",
    "windows",
    "temp",
    "comms",
    "connecteddevicesplatform",
    "d3dscache",
    "packages",
    "publishers",
    "squirreltemp",
];

/// Level-1 folders that hold many tools: look one level deeper instead.
const CONTAINERS: &[&str] = &["programs", "jetbrains", "google", "npm", "pip", "pypoetry"];

/// Every path template of every rule, expanded, whether or not it exists.
/// Templates with unset variables (optional overrides) are skipped, and so
/// are walk-style rules (the project-artifacts walker is rooted at the whole
/// profile) and any path that is one of our scan roots or above them:
/// those "cover" everything and would hide every unrecognized folder.
fn covered_paths(rules: &RuleSet, ctx: &PathContext) -> Vec<PathBuf> {
    let roots: Vec<PathBuf> = ["${USERPROFILE}", "${LOCALAPPDATA}", "${APPDATA}"]
        .iter()
        .filter_map(|v| ctx.expand(v))
        .collect();
    let mut out = Vec::new();
    for r in &rules.rules {
        if r.inspector == crate::rules::Inspector::ProjectArtifacts {
            continue;
        }
        for t in &r.paths {
            if let Some(p) = ctx.expand(t) {
                if roots.iter().any(|root| root.starts_with(&p)) {
                    continue;
                }
                out.push(p);
            }
        }
    }
    out
}

fn norm(p: &Path) -> String {
    p.to_string_lossy().to_ascii_lowercase().replace('/', "\\")
}

/// Is `dir` itself covered, or is something deeper inside it covered?
fn coverage(dir: &Path, covered: &[String]) -> (bool, bool) {
    let d = norm(dir);
    let mut exact = false;
    let mut inside = false;
    for c in covered {
        if *c == d {
            exact = true;
        } else if c.starts_with(&format!("{d}\\")) {
            inside = true;
        } else if d.starts_with(&format!("{c}\\")) {
            // A rule covers a parent of this folder: the rule owns it.
            exact = true;
        }
    }
    (exact, inside)
}

fn display_name(profile: &Path, p: &Path) -> String {
    p.strip_prefix(profile)
        .map(|r| r.to_string_lossy().replace('/', "\\"))
        .unwrap_or_else(|_| p.to_string_lossy().to_string())
}

fn subdirs(p: &Path) -> Vec<PathBuf> {
    let Ok(rd) = std::fs::read_dir(p) else {
        return Vec::new();
    };
    let mut v: Vec<PathBuf> = rd
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir() && !p.is_symlink())
        .collect();
    v.sort();
    v
}

fn skip(p: &Path) -> bool {
    let n = p
        .file_name()
        .map(|n| n.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    SKIP_NAMES.contains(&n.as_str()) || n.starts_with("onedrive")
}

fn is_container(p: &Path) -> bool {
    let n = p
        .file_name()
        .map(|n| n.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    CONTAINERS.contains(&n.as_str())
}

/// Walks the profile, `AppData\Local` and `AppData\Roaming` two levels deep
/// and returns folders of at least `min_bytes` that no rule covers, biggest first.
pub fn find(
    rules: &RuleSet,
    ctx: &PathContext,
    min_bytes: u64,
    excluded: &[PathBuf],
) -> Vec<Unrecognized> {
    let covered: Vec<String> = covered_paths(rules, ctx).iter().map(|p| norm(p)).collect();
    let Some(profile) = ctx.expand("${USERPROFILE}") else {
        return Vec::new();
    };
    let mut roots = vec![profile.clone()];
    for v in ["${LOCALAPPDATA}", "${APPDATA}"] {
        if let Some(p) = ctx.expand(v) {
            if !roots.contains(&p) {
                roots.push(p);
            }
        }
    }
    let mut seen: HashSet<PathBuf> = HashSet::new();
    let mut out = Vec::new();
    let mut consider = |p: &Path, out: &mut Vec<Unrecognized>| {
        if !seen.insert(p.to_path_buf()) || excluded.iter().any(|e| p.starts_with(e)) {
            return;
        }
        let st = tree_stats(p);
        if st.bytes >= min_bytes {
            out.push(Unrecognized {
                name: display_name(&profile, p),
                path: p.to_path_buf(),
                bytes: st.bytes,
                files: st.files,
                newest: st.newest,
            });
        }
    };
    for root in &roots {
        for d in subdirs(root) {
            if skip(&d) {
                continue;
            }
            let (exact, inside) = coverage(&d, &covered);
            if exact {
                continue;
            }
            if inside || is_container(&d) {
                for c in subdirs(&d) {
                    if skip(&c) {
                        continue;
                    }
                    let (cexact, _) = coverage(&c, &covered);
                    if !cexact {
                        consider(&c, &mut out);
                    }
                }
            } else {
                consider(&d, &mut out);
            }
        }
    }
    out.sort_by_key(|u| std::cmp::Reverse(u.bytes));
    out
}

/// A GitHub "new issue" URL with title and body filled in. The username is
/// replaced by `<user>` so the issue never carries it.
pub fn issue_url(repo: &str, u: &Unrecognized, lang: &str) -> String {
    let path = u.path.to_string_lossy().replace('/', "\\");
    let anon = match std::env::var("USERNAME").or_else(|_| std::env::var("USER")) {
        Ok(user) if !user.is_empty() => path.replace(&user, "<user>"),
        _ => path,
    };
    let newest = u
        .newest
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "?".into());
    let size = crate::model::human_size(u.bytes);
    let (title, body) = if lang == "en" {
        (
            format!("Rule suggestion: {}", u.name),
            format!(
                "**Folder:** `{anon}`\n**Size:** {size} ({} files)\n**Newest file:** {newest}\n\n**Which tool writes here?** \n**Is it safe to delete (cache), a re-download (models/packages), or user data?** \n\n_No file list is attached; add detail if you can._\n",
                u.files
            ),
        )
    } else {
        (
            format!("规则建议：{}", u.name),
            format!(
                "**目录：** `{anon}`\n**大小：** {size}（{} 个文件）\n**最新文件：** {newest}\n\n**这是哪个工具写的？** \n**删了会怎样：纯缓存 / 需要重新下载 / 用户数据？** \n\n_没有附带文件列表；方便的话补充一下。_\n",
                u.files
            ),
        )
    };
    format!(
        "https://github.com/{repo}/issues/new?labels=rule-suggestion&title={}&body={}",
        urlencode(&title),
        urlencode(&body)
    )
}

fn urlencode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn ctx(home: &Path) -> PathContext {
        let mut vars = HashMap::new();
        vars.insert("USERPROFILE".to_string(), home.to_path_buf());
        vars.insert("LOCALAPPDATA".to_string(), home.join("AppData/Local"));
        vars.insert("APPDATA".to_string(), home.join("AppData/Roaming"));
        PathContext::with_vars(vars)
    }

    fn rule(id: &str, path: &str) -> crate::rules::Rule {
        crate::rules::Rule {
            id: id.into(),
            tool: "t".into(),
            category: crate::model::Category::PackageCache,
            name_zh: id.into(),
            name_en: id.into(),
            paths: vec![path.into()],
            all_paths: false,
            safety: crate::model::SafetyLevel::Safe,
            inspector: Default::default(),
            consequence_zh: String::new(),
            consequence_en: String::new(),
            min_bytes: 0,
            windows_only: false,
            processes: vec![],
            require: vec![],
            auto_clean: true,
        }
    }

    #[test]
    fn finds_uncovered_folders_and_descends_into_partly_covered_ones() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();
        let mk = |rel: &str, bytes: usize| {
            let p = home.join(rel);
            std::fs::create_dir_all(&p).unwrap();
            std::fs::write(p.join("f.bin"), vec![0u8; bytes]).unwrap();
        };
        mk("AppData/Local/Known/cache", 500);
        mk("AppData/Local/Mystery", 300);
        mk("AppData/Local/Partly/sub-known", 200);
        mk("AppData/Local/Partly/sub-mystery", 400);
        mk("AppData/Local/Tiny", 10);
        mk("Documents/big", 900);
        mk(".weirdtool", 250);
        let rules = RuleSet {
            rules: vec![
                rule("a", "${LOCALAPPDATA}/Known"),
                rule("b", "${LOCALAPPDATA}/Partly/sub-known"),
                rule("c", "${OLLAMA_MODELS}"),
            ],
        };
        let found = find(&rules, &ctx(home), 100, &[]);
        let names: Vec<&str> = found.iter().map(|u| u.name.as_str()).collect();
        assert_eq!(
            names,
            vec![
                r"AppData\Local\Partly\sub-mystery",
                r"AppData\Local\Mystery",
                ".weirdtool"
            ]
        );
        assert_eq!(found[0].bytes, 400);
        // Excluded folders are respected.
        let found = find(
            &rules,
            &ctx(home),
            100,
            &[home.join("AppData/Local/Mystery")],
        );
        assert!(found.iter().all(|u| u.name != r"AppData\Local\Mystery"));
    }

    #[test]
    fn profile_wide_rules_do_not_hide_everything() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();
        let p = home.join("AppData/Local/Huawei");
        std::fs::create_dir_all(&p).unwrap();
        std::fs::write(p.join("f.bin"), vec![0u8; 300]).unwrap();
        let mut walker = rule("projects.artifacts", "${USERPROFILE}");
        walker.inspector = crate::rules::Inspector::ProjectArtifacts;
        let rules = RuleSet {
            rules: vec![
                walker,
                rule("root", "${LOCALAPPDATA}"),
                rule("known", "${LOCALAPPDATA}/Known"),
            ],
        };
        let found = find(&rules, &ctx(home), 100, &[]);
        let names: Vec<&str> = found.iter().map(|u| u.name.as_str()).collect();
        assert_eq!(names, vec![r"AppData\Local\Huawei"]);
    }

    #[test]
    fn issue_url_is_prefilled_and_anonymised() {
        let u = Unrecognized {
            name: r"AppData\Local\Mystery".into(),
            path: PathBuf::from(r"C:\Users\alice\AppData\Local\Mystery"),
            bytes: 4 * 1024 * 1024 * 1024,
            files: 12,
            newest: None,
        };
        std::env::set_var("USERNAME", "alice");
        let url = issue_url("o/r", &u, "en");
        assert!(url.starts_with(
            "https://github.com/o/r/issues/new?labels=rule-suggestion&title=Rule%20suggestion"
        ));
        assert!(url.contains("%3Cuser%3E"));
        assert!(!url.contains("alice"));
        assert!(url.contains("4.0%20GB"));
    }
}
