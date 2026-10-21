//! Claude Code keeps one folder per project under `~/.claude/projects/`,
//! named after the project path with separators replaced by `-`, holding
//! `.jsonl` transcripts. These are conversation history: always Review, but
//! the user deserves to know which project each belongs to and whether that
//! project still exists.

use crate::model::ScanItem;
use crate::rules::Rule;
use crate::scan::{make_item, tree_stats, ScanOptions};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

pub fn projects(rule: &Rule, root: &Path, opts: &ScanOptions) -> Vec<ScanItem> {
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
        // The folder name turns every separator and every non-ASCII character
        // into '-', so "D:\AI\小说" becomes "D--AI---". The transcripts record
        // the real working directory; use it, and fall back to the folder name.
        let cwd = transcript_cwd(&p);
        let name = match &cwd {
            Some(c) => last_two(c.components().filter_map(|c| match c {
                std::path::Component::Normal(s) => Some(s.to_string_lossy().to_string()),
                _ => None,
            })),
            None => last_two(
                folder
                    .split('-')
                    .filter(|s| !s.is_empty())
                    .map(str::to_string),
            ),
        };
        let stats = tree_stats(&p);
        let sessions = std::fs::read_dir(&p)
            .map(|r| {
                r.flatten()
                    .filter(|e| e.file_name().to_string_lossy().ends_with(".jsonl"))
                    .count()
            })
            .unwrap_or(0);
        let mut detail = if opts.lang == "en" {
            format!("{sessions} session transcript(s)")
        } else {
            format!("{sessions} 个会话记录")
        };
        if let Some(c) = &cwd {
            let gone = !c.exists();
            detail = match (opts.lang.as_str(), gone) {
                ("en", true) => format!("{detail} · {} (folder no longer exists)", c.display()),
                ("en", false) => format!("{detail} · {}", c.display()),
                (_, true) => format!("{detail} · {}（项目文件夹已不存在）", c.display()),
                (_, false) => format!("{detail} · {}", c.display()),
            };
        }
        items.push(make_item(rule, &p, name, stats, opts, Some(detail)));
    }
    items
}

fn last_two(parts: impl Iterator<Item = String>) -> String {
    let parts: Vec<String> = parts.collect();
    parts[parts.len().saturating_sub(2)..].join("/")
}

/// The `cwd` recorded in the first lines of any transcript in the folder.
fn transcript_cwd(dir: &Path) -> Option<PathBuf> {
    let rd = std::fs::read_dir(dir).ok()?;
    for e in rd.flatten() {
        if !e.file_name().to_string_lossy().ends_with(".jsonl") {
            continue;
        }
        let Ok(f) = std::fs::File::open(e.path()) else {
            continue;
        };
        for line in BufReader::new(f).lines().take(20).map_while(|l| l.ok()) {
            let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) else {
                continue;
            };
            if let Some(c) = v.get("cwd").and_then(|c| c.as_str()) {
                return Some(PathBuf::from(c));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_real_project_path_from_transcript() {
        let tmp = tempfile::tempdir().unwrap();
        let d = tmp.path().join("D--AI---");
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(
            d.join("s.jsonl"),
            "{\"type\":\"summary\"}\n{\"cwd\":\"D:\\\\AI\\\\小说\",\"type\":\"user\"}\n",
        )
        .unwrap();
        assert_eq!(transcript_cwd(&d), Some(PathBuf::from(r"D:\AI\小说")));
        assert_eq!(
            last_two(["a", "b", "c"].into_iter().map(String::from)),
            "b/c"
        );
    }
}
