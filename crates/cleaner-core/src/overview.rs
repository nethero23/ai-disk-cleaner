//! "Where did my drive go?": how the space on a drive splits across its
//! top-level folders, with `Users\<name>` opened one level further so the
//! usual suspects (`AppData\Local`, `Downloads`…) show up on their own.
//!
//! One walk over the drive, file sizes only. Symlinks and junctions are not
//! followed, so a junction into `Program Files` is not counted twice, and
//! unreadable folders are skipped rather than aborting the walk.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use walkdir::WalkDir;

/// One folder (or a loose file at the root such as `pagefile.sys`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OverviewEntry {
    pub name: String,
    pub path: PathBuf,
    pub bytes: u64,
    pub files: u64,
    #[serde(default)]
    pub children: Vec<OverviewEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Overview {
    pub root: PathBuf,
    pub total_bytes: u64,
    pub free_bytes: u64,
    /// Bytes the walk attributed to entries (less than used space when some
    /// folders were unreadable).
    pub scanned_bytes: u64,
    pub entries: Vec<OverviewEntry>,
    pub computed_at: DateTime<Utc>,
    pub cancelled: bool,
}

pub type Progress<'a> = &'a mut dyn FnMut(u64, u64, &Path);

/// How many path components below the root a file is attributed to.
/// `Users\alice\AppData` is three; anything else is one.
fn attribute_depth(first: &str) -> usize {
    if first.eq_ignore_ascii_case("users") || first.eq_ignore_ascii_case("home") {
        3
    } else {
        1
    }
}

pub fn overview(
    root: &Path,
    total_bytes: u64,
    free_bytes: u64,
    cancel: &AtomicBool,
    progress: Progress,
) -> Overview {
    let mut sums: HashMap<Vec<String>, (u64, u64)> = HashMap::new();
    let mut seen = 0u64;
    let mut scanned = 0u64;
    let mut last_report = std::time::Instant::now();
    let mut cancelled = false;
    let walker = WalkDir::new(root)
        .follow_links(false)
        .same_file_system(true)
        .into_iter();
    for e in walker.flatten() {
        if cancel.load(Ordering::Relaxed) {
            cancelled = true;
            break;
        }
        if e.file_type().is_dir() {
            if last_report.elapsed().as_millis() > 150 {
                progress(seen, scanned, e.path());
                last_report = std::time::Instant::now();
            }
            continue;
        }
        if !e.file_type().is_file() {
            continue;
        }
        let Ok(md) = e.metadata() else { continue };
        let Ok(rel) = e.path().strip_prefix(root) else {
            continue;
        };
        let comps: Vec<String> = rel
            .components()
            .filter_map(|c| match c {
                Component::Normal(s) => Some(s.to_string_lossy().to_string()),
                _ => None,
            })
            .collect();
        if comps.is_empty() {
            continue;
        }
        seen += 1;
        scanned += md.len();
        // A loose file at the root is its own entry; a file inside a folder
        // is credited to that folder (and to the deeper keys under Users).
        let depth = if comps.len() == 1 {
            1
        } else {
            attribute_depth(&comps[0]).min(comps.len() - 1)
        };
        for k in 1..=depth {
            let entry = sums.entry(comps[..k].to_vec()).or_insert((0, 0));
            entry.0 += md.len();
            entry.1 += 1;
        }
    }

    let mut entries = build(root, &[], &sums);
    entries.sort_by_key(|e| std::cmp::Reverse(e.bytes));
    Overview {
        root: root.to_path_buf(),
        total_bytes,
        free_bytes,
        scanned_bytes: scanned,
        entries,
        computed_at: Utc::now(),
        cancelled,
    }
}

fn build(
    root: &Path,
    prefix: &[String],
    sums: &HashMap<Vec<String>, (u64, u64)>,
) -> Vec<OverviewEntry> {
    let mut out: Vec<OverviewEntry> = sums
        .iter()
        .filter(|(k, _)| k.len() == prefix.len() + 1 && k.starts_with(prefix))
        .map(|(k, (bytes, files))| {
            let mut path = root.to_path_buf();
            for c in k {
                path.push(c);
            }
            let mut children = build(root, k, sums);
            children.sort_by_key(|e| std::cmp::Reverse(e.bytes));
            OverviewEntry {
                name: k.last().cloned().unwrap_or_default(),
                path,
                bytes: *bytes,
                files: *files,
                children,
            }
        })
        .collect();
    out.sort_by_key(|e| std::cmp::Reverse(e.bytes));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn splits_space_by_top_level_folder_and_opens_users() {
        let tmp = tempfile::tempdir().unwrap();
        let d = tmp.path();
        fs::create_dir_all(d.join("Users/alice/AppData/Local/x")).unwrap();
        fs::create_dir_all(d.join("Users/alice/Downloads")).unwrap();
        fs::create_dir_all(d.join("Windows/System32")).unwrap();
        fs::write(d.join("Users/alice/AppData/Local/x/a"), vec![0u8; 300]).unwrap();
        fs::write(d.join("Users/alice/Downloads/b"), vec![0u8; 200]).unwrap();
        fs::write(d.join("Windows/System32/c"), vec![0u8; 600]).unwrap();
        fs::write(d.join("pagefile.sys"), vec![0u8; 1000]).unwrap();
        let cancel = AtomicBool::new(false);
        let ov = overview(d, 10_000, 5_000, &cancel, &mut |_, _, _| {});
        assert_eq!(ov.scanned_bytes, 2100);
        assert!(!ov.cancelled);
        let names: Vec<&str> = ov.entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, vec!["pagefile.sys", "Windows", "Users"]);
        let users = ov.entries.iter().find(|e| e.name == "Users").unwrap();
        assert_eq!(users.bytes, 500);
        assert_eq!(users.children.len(), 1);
        let alice = &users.children[0];
        assert_eq!(alice.name, "alice");
        let folders: Vec<(&str, u64)> = alice
            .children
            .iter()
            .map(|e| (e.name.as_str(), e.bytes))
            .collect();
        assert_eq!(folders, vec![("AppData", 300), ("Downloads", 200)]);
        // AppData is not opened further than the attribution depth.
        assert!(alice.children[0].children.is_empty());
        // Windows is not opened at all.
        let windows = ov.entries.iter().find(|e| e.name == "Windows").unwrap();
        assert_eq!(windows.bytes, 600);
        assert!(windows.children.is_empty());
    }

    #[test]
    fn cancel_stops_the_walk() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("a"), vec![0u8; 10]).unwrap();
        let cancel = AtomicBool::new(true);
        let ov = overview(tmp.path(), 1, 1, &cancel, &mut |_, _, _| {});
        assert!(ov.cancelled);
        assert!(ov.entries.is_empty());
    }
}
