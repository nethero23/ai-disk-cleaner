//! VS Code-family editors (Cursor, Windsurf, VS Code) keep
//! `User/workspaceStorage/<hash>/workspace.json` pointing at the folder the
//! storage belongs to. When that folder no longer exists, the storage is an
//! orphan and safe to drop; otherwise it may hold chat state and is Review.

use super::read_small;
use crate::model::{SafetyLevel, ScanItem};
use crate::rules::Rule;
use crate::scan::{make_item, tree_stats, ScanOptions};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
struct WorkspaceJson {
    #[serde(default)]
    folder: Option<String>,
    #[serde(default)]
    workspace: Option<String>,
}

/// `file:///c%3A/Users/me/proj` → `c:\Users\me\proj` (best effort).
pub fn uri_to_path(uri: &str) -> Option<PathBuf> {
    let rest = uri
        .strip_prefix("file:///")
        .or_else(|| uri.strip_prefix("file://"))?;
    let decoded = percent_decode(rest);
    let s = if cfg!(windows) {
        decoded.replace('/', "\\")
    } else {
        format!("/{decoded}")
    };
    Some(PathBuf::from(s))
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

pub fn workspaces(rule: &Rule, root: &Path, opts: &ScanOptions) -> Vec<ScanItem> {
    let mut items = Vec::new();
    let Ok(rd) = std::fs::read_dir(root) else {
        return items;
    };
    for entry in rd.flatten() {
        let p = entry.path();
        if !p.is_dir() || p.is_symlink() {
            continue;
        }
        let target = read_small(&p.join("workspace.json"))
            .and_then(|t| serde_json::from_str::<WorkspaceJson>(&t).ok())
            .and_then(|w| w.folder.or(w.workspace))
            .and_then(|u| uri_to_path(&u));
        let stats = tree_stats(&p);
        let (name, safety, detail) = match target {
            Some(t) if t.exists() => (
                t.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| t.to_string_lossy().to_string()),
                SafetyLevel::Review,
                Some(if opts.lang == "en" {
                    format!("Project still exists: {}", t.display())
                } else {
                    format!("项目仍存在：{}", t.display())
                }),
            ),
            Some(t) => (
                t.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| t.to_string_lossy().to_string()),
                SafetyLevel::Safe,
                Some(if opts.lang == "en" {
                    format!("Project folder is gone: {}", t.display())
                } else {
                    format!("项目文件夹已不存在：{}", t.display())
                }),
            ),
            None => (
                entry.file_name().to_string_lossy().to_string(),
                SafetyLevel::Review,
                None,
            ),
        };
        let mut item = make_item(rule, &p, name, stats, opts, detail);
        item.safety = safety;
        if safety == SafetyLevel::Safe {
            item.consequence = if opts.lang == "en" {
                "Storage for a deleted project; nothing references it".into()
            } else {
                "属于已删除的项目，没有东西再引用它".into()
            };
        }
        items.push(item);
    }
    items
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_file_uris() {
        let p = uri_to_path("file:///c%3A/Users/me/my%20proj").unwrap();
        let s = p.to_string_lossy();
        assert!(s.ends_with("my proj"), "{s}");
        assert!(s.to_lowercase().contains("c:"), "{s}");
    }
}
