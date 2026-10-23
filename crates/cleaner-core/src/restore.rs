//! Undo: put things back from the Recycle Bin.
//!
//! Deletion always goes through the Recycle Bin, so undo is "find the bin
//! entry whose original path matches the clean record and move it back".
//! Matching is by original path, newest entry first, so a path deleted
//! twice restores the most recent copy. Nothing here touches files that the
//! app did not delete.

use crate::clean::CleanRecord;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
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

/// The deleted tail no longer exists, but an existing ancestor can still
/// resolve Windows short-name / verbatim aliases to the same original path.
fn match_path(path: &Path) -> PathBuf {
    // Never turn a redirected ancestor into permission to restore an unrelated
    // original path. Preserve literal matching when aliases cannot be trusted.
    if path
        .ancestors()
        .any(|p| match std::fs::symlink_metadata(p) {
            Ok(metadata) => {
                #[cfg(windows)]
                {
                    use std::os::windows::fs::MetadataExt;
                    metadata.file_attributes() & 0x400 != 0
                }
                #[cfg(not(windows))]
                {
                    metadata.file_type().is_symlink()
                }
            }
            Err(error) => error.kind() != std::io::ErrorKind::NotFound,
        })
    {
        return path.to_path_buf();
    }
    let mut ancestor = path;
    let mut tail = Vec::new();
    loop {
        if let Ok(mut resolved) = std::fs::canonicalize(ancestor) {
            for name in tail.iter().rev() {
                resolved.push(name);
            }
            return resolved;
        }
        let Some(name) = ancestor.file_name() else {
            return path.to_path_buf();
        };
        tail.push(name.to_os_string());
        let Some(parent) = ancestor.parent() else {
            return path.to_path_buf();
        };
        ancestor = parent;
    }
}

/// Picks, among bin entries, the newest one whose original path is `path`.
/// `not_before` (unix seconds) drops entries older than the clean record, so a
/// file the user deleted by hand last year is never mistaken for ours.
fn pick<'a>(
    items: &'a [trash::TrashItem],
    path: &Path,
    not_before: i64,
) -> Option<&'a trash::TrashItem> {
    let wanted = match_path(path);
    items
        .iter()
        .filter(|t| t.time_deleted >= not_before && match_path(&t.original_path()) == wanted)
        .max_by_key(|t| t.time_deleted)
}

fn list_bin() -> Vec<trash::TrashItem> {
    trash::os_limited::list().unwrap_or_default()
}

/// Whether each path is back in place, still in the bin, or gone for good.
/// Lists the bin once, so calling it for a whole history page is cheap.
pub fn status(paths: &[PathBuf]) -> Vec<RestoreStatus> {
    let in_bin: HashSet<PathBuf> = list_bin()
        .iter()
        .map(|t| match_path(&t.original_path()))
        .collect();
    paths
        .iter()
        .map(|p| RestoreStatus {
            path: p.clone(),
            state: if p.exists() {
                Whereabouts::Present
            } else if in_bin.contains(&match_path(p)) {
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
    let mut seen = HashSet::new();
    for rec in records {
        if !seen.insert(match_path(&rec.path)) {
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
    fn deleted_paths_match_canonical_ancestor_aliases_without_weakening_cutoff() {
        let tmp = tempfile::tempdir().unwrap();
        let canonical = std::fs::canonicalize(tmp.path()).unwrap();
        let parent = canonical.join("missing-parent");
        let items = vec![item(parent.to_str().unwrap(), "cache", 300)];
        let original = tmp.path().join("missing-parent/cache");
        assert!(!original.exists(), "the deleted tail must stay absent");
        assert!(pick(&items, &original, 200).is_some());
        assert!(pick(&items, &original, 301).is_none());
        assert!(pick(&items, &tmp.path().join("other/cache"), 0).is_none());
        assert!(pick(&items, &tmp.path().join("missing-parent/other"), 0).is_none());
    }

    #[test]
    fn redirected_ancestor_never_matches_another_original_bin_path() {
        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path().join("target");
        let alias = tmp.path().join("alias");
        std::fs::create_dir(&target).unwrap();
        #[cfg(windows)]
        assert!(std::process::Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(&alias)
            .arg(&target)
            .status()
            .unwrap()
            .success());
        #[cfg(unix)]
        std::os::unix::fs::symlink(&target, &alias).unwrap();
        let items = vec![item(target.to_str().unwrap(), "cache", 300)];
        assert!(pick(&items, &alias.join("cache"), 0).is_none());
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
