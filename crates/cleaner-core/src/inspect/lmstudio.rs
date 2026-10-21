//! LM Studio stores `models/<publisher>/<model>/<file>.gguf`. One item per
//! `<publisher>/<model>` folder so the user sees "lmstudio-community/Qwen2.5-7B".

use crate::model::ScanItem;
use crate::rules::Rule;
use crate::scan::{make_item, tree_stats, ScanOptions};
use std::path::Path;

pub fn models(rule: &Rule, root: &Path, opts: &ScanOptions) -> Vec<ScanItem> {
    let mut items = Vec::new();
    let Ok(publishers) = std::fs::read_dir(root) else {
        return items;
    };
    for pubdir in publishers.flatten() {
        let pp = pubdir.path();
        if !pp.is_dir() || pp.is_symlink() {
            continue;
        }
        let publisher = pubdir.file_name().to_string_lossy().to_string();
        let Ok(models) = std::fs::read_dir(&pp) else {
            continue;
        };
        let mut any_child = false;
        for m in models.flatten() {
            let mp = m.path();
            if !mp.is_dir() || mp.is_symlink() {
                continue;
            }
            any_child = true;
            let name = format!("{publisher}/{}", m.file_name().to_string_lossy());
            let stats = tree_stats(&mp);
            let files: Vec<String> = std::fs::read_dir(&mp)
                .map(|r| {
                    r.flatten()
                        .map(|e| e.file_name().to_string_lossy().to_string())
                        .filter(|n| n.ends_with(".gguf") || n.ends_with(".safetensors"))
                        .collect()
                })
                .unwrap_or_default();
            let detail = if files.is_empty() {
                None
            } else {
                Some(files.join(", "))
            };
            items.push(make_item(rule, &mp, name, stats, opts, detail));
        }
        // A publisher folder holding loose files (older layout): treat it as one model.
        if !any_child {
            let stats = tree_stats(&pp);
            if stats.bytes > 0 {
                items.push(make_item(rule, &pp, publisher, stats, opts, None));
            }
        }
    }
    items
}
