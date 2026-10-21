//! "What is actually in there?" The largest files under a location, so the
//! user can judge an item without opening Explorer.

use chrono::{DateTime, Utc};
use serde::Serialize;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize)]
pub struct PreviewEntry {
    /// Path relative to the item, with the OS separator.
    pub rel: String,
    pub path: PathBuf,
    pub size_bytes: u64,
    pub modified: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Preview {
    pub files: u64,
    pub dirs: u64,
    /// The `limit` largest files, largest first.
    pub largest: Vec<PreviewEntry>,
}

/// Walks `root` (a single file previews as itself) and keeps the `limit`
/// largest files. Symlinks and junctions are never followed.
pub fn preview(root: &Path, limit: usize) -> Preview {
    let mut out = Preview::default();
    if root.is_symlink() {
        return out;
    }
    let mut heap: Vec<PreviewEntry> = Vec::new();
    for e in WalkDir::new(root).follow_links(false).into_iter().flatten() {
        let ft = e.file_type();
        if ft.is_dir() {
            if e.depth() > 0 {
                out.dirs += 1;
            }
            continue;
        }
        if ft.is_symlink() || !ft.is_file() {
            continue;
        }
        let Ok(md) = e.metadata() else { continue };
        out.files += 1;
        let size = md.len();
        // Keep the vector small: only consider the file when it beats the
        // smallest kept one (or there is still room).
        if heap.len() == limit && heap.last().is_some_and(|l| l.size_bytes >= size) {
            continue;
        }
        let rel = e
            .path()
            .strip_prefix(root)
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default();
        let rel = if rel.is_empty() {
            e.file_name().to_string_lossy().into_owned()
        } else {
            rel
        };
        heap.push(PreviewEntry {
            rel,
            path: e.path().to_path_buf(),
            size_bytes: size,
            modified: md.modified().ok().map(DateTime::<Utc>::from),
        });
        heap.sort_by_key(|p| std::cmp::Reverse(p.size_bytes));
        heap.truncate(limit);
    }
    out.largest = heap;
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn keeps_the_largest_files_in_order() {
        let tmp = tempfile::tempdir().unwrap();
        let d = tmp.path().join("cache");
        fs::create_dir_all(d.join("sub")).unwrap();
        fs::write(d.join("small"), [0u8; 10]).unwrap();
        fs::write(d.join("sub/big"), [0u8; 300]).unwrap();
        fs::write(d.join("mid"), [0u8; 100]).unwrap();
        fs::write(d.join("tiny"), [0u8; 1]).unwrap();
        let p = preview(&d, 2);
        assert_eq!(p.files, 4);
        assert_eq!(p.dirs, 1);
        let rels: Vec<&str> = p.largest.iter().map(|e| e.rel.as_str()).collect();
        assert_eq!(
            rels,
            vec![
                format!("sub{}big", std::path::MAIN_SEPARATOR),
                "mid".to_string()
            ]
        );
    }

    #[test]
    fn a_single_file_previews_as_itself() {
        let tmp = tempfile::tempdir().unwrap();
        let f = tmp.path().join("model.gguf");
        fs::write(&f, [0u8; 50]).unwrap();
        let p = preview(&f, 5);
        assert_eq!(p.files, 1);
        assert_eq!(p.largest[0].rel, "model.gguf");
    }
}
