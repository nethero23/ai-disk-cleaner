//! Claude Code keeps one folder per project under `~/.claude/projects/`,
//! named after the project path with separators replaced by `-`, holding
//! `.jsonl` transcripts. These are conversation history: always Review, but
//! the user deserves to know which project each belongs to and whether that
//! project still exists.

use crate::model::ScanItem;
use crate::rules::Rule;
use crate::scan::{make_item, tree_stats, ScanOptions};
use std::path::Path;

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
        // "C--Users-me-proj" → a readable tail; the exact original path is not
        // recoverable because '-' is ambiguous, so show the last two segments.
        let tail: Vec<&str> = folder
            .rsplit('-')
            .filter(|s| !s.is_empty())
            .take(2)
            .collect();
        let name = tail.iter().rev().cloned().collect::<Vec<_>>().join("/");
        let stats = tree_stats(&p);
        let sessions = std::fs::read_dir(&p)
            .map(|r| {
                r.flatten()
                    .filter(|e| e.file_name().to_string_lossy().ends_with(".jsonl"))
                    .count()
            })
            .unwrap_or(0);
        let detail = Some(if opts.lang == "en" {
            format!("{sessions} session transcript(s)")
        } else {
            format!("{sessions} 个会话记录")
        });
        items.push(make_item(rule, &p, name, stats, opts, detail));
    }
    items
}
