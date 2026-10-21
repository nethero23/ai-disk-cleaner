//! Tool-specific inspectors: turn an opaque cache folder into named items.
//!
//! Each inspector takes the rule, the resolved root and the options, and
//! returns one [`ScanItem`] per thing the user would recognise. They never
//! delete anything and tolerate half-written caches (a model mid-download,
//! a workspace folder with no `workspace.json`).

pub mod artifacts;
pub mod claude;
pub mod conda;
pub mod cursor;
pub mod drives;
pub mod huggingface;
pub mod lmstudio;
pub mod ollama;
pub mod versions;

use crate::model::ScanItem;
use crate::rules::Rule;
use crate::scan::{make_item, tree_stats, ScanOptions};
use std::path::Path;

/// One item per immediate subdirectory, named after the folder.
pub fn children(rule: &Rule, root: &Path, opts: &ScanOptions) -> Vec<ScanItem> {
    let mut items = Vec::new();
    let Ok(rd) = std::fs::read_dir(root) else {
        return items;
    };
    for entry in rd.flatten() {
        let p = entry.path();
        if !p.is_dir() || p.is_symlink() {
            continue;
        }
        if !rule.require.is_empty() && !rule.require.iter().any(|r| p.join(r).exists()) {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let stats = tree_stats(&p);
        items.push(make_item(rule, &p, name, stats, opts, None));
    }
    items
}

/// Reads a small text/JSON file, returning `None` on any problem.
pub(crate) fn read_small(p: &Path) -> Option<String> {
    let md = std::fs::metadata(p).ok()?;
    if md.len() > 4 * 1024 * 1024 {
        return None;
    }
    std::fs::read_to_string(p).ok()
}
