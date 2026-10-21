//! The Hugging Face hub cache: `hub/models--<org>--<name>/{blobs,refs,snapshots}`.
//! On Windows without Developer Mode there are no symlinks and files live
//! straight in `snapshots/`, possibly duplicated across revisions.

use crate::model::ScanItem;
use crate::rules::Rule;
use crate::scan::{make_item, tree_stats, ScanOptions};
use std::path::Path;

/// `models--Qwen--Qwen2.5-7B` → `Qwen/Qwen2.5-7B`; datasets/spaces keep a prefix.
pub fn repo_display_name(folder: &str) -> Option<String> {
    let (kind, rest) = folder.split_once("--")?;
    let name = rest.replace("--", "/");
    Some(match kind {
        "models" => name,
        other => format!("{other}: {name}"),
    })
}

pub fn hub(rule: &Rule, root: &Path, opts: &ScanOptions) -> Vec<ScanItem> {
    let mut items = Vec::new();
    let Ok(rd) = std::fs::read_dir(root) else {
        return items;
    };
    for entry in rd.flatten() {
        let p = entry.path();
        if !p.is_dir() || p.is_symlink() {
            continue;
        }
        let folder = entry.file_name().to_string_lossy().to_string();
        let Some(name) = repo_display_name(&folder) else {
            continue;
        };
        let stats = tree_stats(&p);
        let snapshots = std::fs::read_dir(p.join("snapshots"))
            .map(|r| r.flatten().count())
            .unwrap_or(0);
        let incomplete = std::fs::read_dir(p.join("blobs"))
            .map(|r| {
                r.flatten()
                    .filter(|e| e.file_name().to_string_lossy().ends_with(".incomplete"))
                    .count()
            })
            .unwrap_or(0);
        let mut notes = Vec::new();
        if snapshots > 1 {
            notes.push(if opts.lang == "en" {
                format!("{snapshots} revisions cached")
            } else {
                format!("缓存了 {snapshots} 个版本")
            });
        }
        if incomplete > 0 {
            notes.push(if opts.lang == "en" {
                format!("{incomplete} incomplete download(s)")
            } else {
                format!("{incomplete} 个未完成的下载")
            });
        }
        let detail = if notes.is_empty() {
            None
        } else {
            Some(notes.join("; "))
        };
        items.push(make_item(rule, &p, name, stats, opts, detail));
    }
    items
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folder_names_become_repo_ids() {
        assert_eq!(
            repo_display_name("models--Qwen--Qwen2.5-7B").as_deref(),
            Some("Qwen/Qwen2.5-7B")
        );
        assert_eq!(repo_display_name("models--gpt2").as_deref(), Some("gpt2"));
        assert_eq!(
            repo_display_name("datasets--squad").as_deref(),
            Some("datasets: squad")
        );
        assert_eq!(repo_display_name("version.txt"), None);
    }
}
