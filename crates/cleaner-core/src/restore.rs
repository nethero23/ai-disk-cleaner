//! Undo: put things back from the Recycle Bin.
//!
//! Deletion always goes through the Recycle Bin, so undo is "find the bin
//! entry whose original path matches the clean record and move it back".
//! Matching is by original path, newest entry first, so a path deleted
//! twice restores the most recent copy. Nothing here touches files that the
//! app did not delete.

use crate::clean::CleanRecord;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Where a cleaned item is now.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Whereabouts {
    /// Something exists at the original path again (restored, or rebuilt by the tool).
    Present,
    /// Still in the Recycle Bin: can be restored.
    InBin,
    /// Neither: the bin was emptied, or the entry was purged.
    Gone,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreStatus {
    pub path: PathBuf,
    pub state: Whereabouts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreResult {
    pub path: PathBuf,
    pub ok: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RestoreSummary {
    pub restored: usize,
    pub restored_bytes: u64,
    pub results: Vec<RestoreResult>,
}

/// Picks, among bin entries, the newest one whose original path is `path`.
/// `not_before` (unix seconds) drops entries older than the clean record, so a
/// file the user deleted by hand last year is never mistaken for ours.
fn pick<'a>(
    items: &'a [trash::TrashItem],
    path: &Path,
    not_before: i64,
) -> Option<&'a trash::TrashItem> {
    items
        .iter()
        .filter(|t| t.original_path() == path && t.time_deleted >= not_before)
        .max_by_key(|t| t.time_deleted)
}

fn list_bin() -> Vec<trash::TrashItem> {
    trash::os_limited::list().unwrap_or_default()
}

/// Whether each path is back in place, still in the bin, or gone for good.
/// Lists the bin once, so calling it for a whole history page is cheap.
pub fn status(paths: &[PathBuf]) -> Vec<RestoreStatus> {
    let in_bin: HashSet<PathBuf> = list_bin().iter().map(|t| t.original_path()).collect();
    paths
        .iter()
        .map(|p| RestoreStatus {
            path: p.clone(),
            state: if p.exists() {
                Whereabouts::Present
            } else if in_bin.contains(p) {
                Whereabouts::InBin
            } else {
                Whereabouts::Gone
            },
        })
        .collect()
}

/// Restores the given clean records, one bin entry each, and never stops on
/// one failure. A record whose path exists again is reported as a failure
/// ("already present") rather than overwritten.
pub fn restore(records: &[CleanRecord]) -> RestoreSummary {
    let bin = list_bin();
    let mut summary = RestoreSummary::default();
    // Deduplicate by path: the same path cleaned twice restores once.
    let mut seen: HashMap<PathBuf, ()> = HashMap::new();
    for rec in records {
        if seen.insert(rec.path.clone(), ()).is_some() {
            continue;
        }
        let result = restore_one(&bin, rec);
        if result.is_ok() {
            summary.restored += 1;
            summary.restored_bytes += rec.size_bytes;
        }
        summary.results.push(RestoreResult {
            path: rec.path.clone(),
            ok: result.is_ok(),
            error: result.err(),
        });
    }
    summary
}

fn restore_one(bin: &[trash::TrashItem], rec: &CleanRecord) -> std::result::Result<(), String> {
    if rec.path.exists() {
        return Err("already present at the original path".into());
    }
    // Allow a little clock skew between the log and the bin's timestamp.
    let not_before = rec.at.timestamp() - 120;
    let item = pick(bin, &rec.path, not_before)
        .ok_or_else(|| "not in the Recycle Bin any more".to_string())?;
    trash::os_limited::restore_all([item.clone()]).map_err(|e| match e {
        trash::Error::RestoreCollision { path, .. } => format!("{} already exists", path.display()),
        other => other.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    fn item(parent: &str, name: &str, t: i64) -> trash::TrashItem {
        trash::TrashItem {
            id: OsString::from(format!("{parent}/{name}@{t}")),
            name: OsString::from(name),
            original_parent: PathBuf::from(parent),
            time_deleted: t,
        }
    }

    #[test]
    fn pick_prefers_newest_matching_entry_after_cutoff() {
        let items = vec![
            item("/u/cache", "a", 100),
            item("/u/cache", "a", 300),
            item("/u/cache", "b", 400),
            item("/u/other", "a", 500),
        ];
        let got = pick(&items, Path::new("/u/cache/a"), 200).unwrap();
        assert_eq!(got.time_deleted, 300);
        assert!(pick(&items, Path::new("/u/cache/a"), 350).is_none());
        assert!(pick(&items, Path::new("/u/cache/zzz"), 0).is_none());
    }

    #[test]
    fn status_reports_present_paths_without_touching_the_bin() {
        let tmp = tempfile::tempdir().unwrap();
        let here = tmp.path().join("here");
        std::fs::create_dir(&here).unwrap();
        let gone = tmp.path().join("gone");
        let st = status(&[here.clone(), gone.clone()]);
        assert_eq!(st[0].state, Whereabouts::Present);
        // Not in this process's bin (nothing was deleted), so gone.
        assert_eq!(st[1].state, Whereabouts::Gone);
    }
}
