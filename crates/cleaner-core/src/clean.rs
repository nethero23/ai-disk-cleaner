//! Reversible deletion. Everything goes to the Recycle Bin (via the `trash`
//! crate) and every action is appended to a JSON-lines log so the UI can
//! show history and restore.
//!
//! Guards, in order: the path must be one the current scan reported (the
//! caller passes the item), must still exist, must not be a filesystem root
//! or the user's home, and must not be a symlink/junction.

use crate::model::{SafetyLevel, ScanItem};
use crate::{Error, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanRecord {
    pub at: DateTime<Utc>,
    pub rule_id: String,
    pub name: String,
    pub path: PathBuf,
    pub size_bytes: u64,
    pub safety: SafetyLevel,
    pub ok: bool,
    pub error: Option<String>,
    /// All records of one `clean` call share a batch id, so "undo the last
    /// clean" knows what belongs together. Empty on records from older versions.
    #[serde(default)]
    pub batch: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CleanSummary {
    pub freed_bytes: u64,
    pub records: Vec<CleanRecord>,
    /// Audit persistence failed. Actual successes remain successes, but no
    /// subsequent item is touched and the UI must show this warning.
    #[serde(default)]
    pub log_error: Option<String>,
}

/// Refuses paths that no rule should ever hand us, as a last line of defence
/// against a bad rule file or a corrupted scan result.
pub fn guard(path: &Path) -> Result<()> {
    if path
        .to_string_lossy()
        .split(std::path::is_separator)
        .any(|part| matches!(part, "." | ".."))
    {
        return Err(Error::Refused(
            path.into(),
            "path contains dot segments; rescan without path aliases".into(),
        ));
    }
    if crate::system::is_windows_old(path) {
        return Err(windows_old_refused(path));
    }
    if !path.exists() {
        return Err(Error::Refused(path.into(), "no longer exists".into()));
    }
    if path.is_symlink() {
        return Err(Error::Refused(
            path.into(),
            "is a symlink or junction".into(),
        ));
    }
    let depth = path.components().count();
    if depth <= 1 {
        return Err(Error::Refused(path.into(), "is a drive root".into()));
    }
    if let Some(home) = dirs::home_dir() {
        if path == home {
            return Err(Error::Refused(path.into(), "is the home folder".into()));
        }
        for sub in ["Desktop", "Documents", "Pictures", "Videos", "Downloads"] {
            if path == home.join(sub) {
                return Err(Error::Refused(path.into(), format!("is the {sub} folder")));
            }
        }
    }
    if let Some(sr) = std::env::var_os("SYSTEMROOT") {
        let sr = PathBuf::from(sr);
        if path == sr || path == sr.join("System32") {
            return Err(Error::Refused(
                path.into(),
                "is a Windows system folder".into(),
            ));
        }
    }
    Ok(())
}

fn windows_old_refused(path: &Path) -> Error {
    Error::Refused(path.into(), "Windows.old 需要单独的系统深度清理，不能移入回收站 / Windows.old requires separate system cleanup, not the Recycle Bin".into())
}

/// Moves one item to the Recycle Bin.
pub fn delete_item(item: &ScanItem) -> Result<()> {
    if item.rule_id == "windows.old" {
        return Err(windows_old_refused(&item.path));
    }
    if let Some(proc_name) = &item.in_use_by {
        return Err(Error::Refused(
            item.path.clone(),
            format!("{proc_name} is running; close it first"),
        ));
    }
    guard(&item.path)?;
    trash::delete(&item.path)?;
    Ok(())
}

/// Deletes each item, logging to `log_path`. Item failures do not stop the
/// batch; unavailable or failed audit persistence stops all further deletion.
pub fn clean(items: &[ScanItem], log_path: &Path) -> CleanSummary {
    // Foreground and scheduled cleanup must not interleave journal records.
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _lock = LOCK.lock().unwrap_or_else(|poison| poison.into_inner());
    if items.is_empty() {
        return CleanSummary::default();
    }
    let opened = open_log(log_path);
    let error = opened.as_ref().err().map(ToString::to_string);
    let mut log = opened.ok();
    clean_with(items, error, delete_item, |rec| {
        let f = log
            .as_mut()
            .expect("only append after successful preflight");
        let line = serde_json::to_vec(rec).map_err(|e| e.to_string())?;
        f.write_all(&line)
            .and_then(|_| f.write_all(b"\n"))
            .and_then(|_| f.sync_data())
            .map_err(|e| e.to_string())
    })
}

fn open_log(path: &Path) -> Result<std::fs::File> {
    read_log(path)?;
    let open = || -> std::io::Result<std::fs::File> {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(path)?;
        // Legacy JSONL may end without a newline. Separate before appending,
        // never concatenate two valid records into one invalid line.
        if file.metadata()?.len() > 0 {
            file.seek(SeekFrom::End(-1))?;
            let mut last = [0];
            file.read_exact(&mut last)?;
            if last[0] != b'\n' {
                file.write_all(b"\n")?;
            }
        }
        file.sync_data()?;
        Ok(file)
    };
    open().map_err(|source| Error::Io {
        path: path.into(),
        source,
    })
}

fn clean_with(
    items: &[ScanItem],
    log_error: Option<String>,
    mut delete: impl FnMut(&ScanItem) -> Result<()>,
    mut append: impl FnMut(&CleanRecord) -> std::result::Result<(), String>,
) -> CleanSummary {
    let mut summary = CleanSummary {
        log_error,
        ..Default::default()
    };
    let batch = Utc::now().timestamp_millis().to_string();
    for item in items {
        let result = match &summary.log_error {
            Some(error) => Err(Error::Refused(
                item.path.clone(),
                format!("cleanup stopped: history unavailable: {error}"),
            )),
            None => delete(item),
        };
        let rec = CleanRecord {
            at: Utc::now(),
            rule_id: item.rule_id.clone(),
            name: item.name.clone(),
            path: item.path.clone(),
            size_bytes: item.size_bytes,
            safety: item.safety,
            ok: result.is_ok(),
            error: result.err().map(|e| e.to_string()),
            batch: batch.clone(),
        };
        if rec.ok {
            summary.freed_bytes += rec.size_bytes;
        }
        if summary.log_error.is_none() {
            if let Err(error) = append(&rec) {
                summary.log_error = Some(format!(
                    "History could not be saved; further items were left untouched: {error}"
                ));
            }
        }
        summary.records.push(rec);
    }
    summary
}

/// Reads the whole log, newest last.
pub fn read_log(log_path: &Path) -> Result<Vec<CleanRecord>> {
    crate::journal::read(log_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_old_never_enters_the_generic_recycle_path() {
        let tmp = tempfile::tempdir().unwrap();
        // Deliberately absent: the regression must never delete even a fixture.
        let mut item = fixture(&tmp.path().join("Windows.old"));
        item.rule_id = "windows.old".into();
        let error = delete_item(&item).unwrap_err().to_string();
        assert!(error.contains("system cleanup"), "{error}");
    }

    #[test]
    fn guard_refuses_system_old_root_and_dot_segment_aliases_without_deletion() {
        let root = crate::paths::PathContext::from_env()
            .expand("${SYSTEMDRIVE}/Windows.old")
            .unwrap();
        assert!(guard(&root)
            .unwrap_err()
            .to_string()
            .contains("system cleanup"));
        // Only call the read-only guard, never the deletion entry point.
        let tmp = tempfile::tempdir().unwrap();
        for alias in [root.join("."), tmp.path().join("absent").join("..")] {
            let error = guard(&alias).unwrap_err().to_string();
            assert!(error.contains("dot segments"), "{error}");
        }
    }

    #[test]
    fn unreadable_history_is_not_an_empty_success() {
        let tmp = tempfile::tempdir().unwrap();
        let got = read_log(tmp.path());
        assert!(got.is_err(), "read errors must reach the UI");
    }

    #[test]
    fn corrupt_history_is_not_an_empty_success() {
        let tmp = tempfile::tempdir().unwrap();
        let log = tmp.path().join("clean.log");
        std::fs::write(&log, b"{broken record}\n").unwrap();
        let got = read_log(&log);
        assert!(got.is_err(), "parse errors must reach the UI");
    }

    fn fixture(path: &Path) -> ScanItem {
        ScanItem {
            id: path.display().to_string(),
            rule_id: "fixture".into(),
            tool: "Fixture".into(),
            category: crate::Category::PackageCache,
            name: "fixture".into(),
            path: path.into(),
            size_bytes: 5,
            file_count: 1,
            last_used: None,
            safety: SafetyLevel::Safe,
            consequence: String::new(),
            detail: None,
            in_use_by: None,
            auto_clean: true,
        }
    }

    #[test]
    fn unavailable_or_corrupt_journal_prevents_any_deletion() {
        let tmp = tempfile::tempdir().unwrap();
        let source = tmp.path().join("sentinel");
        std::fs::write(&source, b"alive").unwrap();
        let corrupt = tmp.path().join("corrupt.jsonl");
        std::fs::write(&corrupt, b"invalid-private-record").unwrap();
        for log in [tmp.path().to_path_buf(), corrupt.clone()] {
            let result = clean(&[fixture(&source)], &log);
            assert!(result.log_error.is_some());
            assert_eq!(result.freed_bytes, 0);
            assert!(!result.records[0].ok);
            assert_eq!(std::fs::read(&source).unwrap(), b"alive");
        }
        assert_eq!(std::fs::read(corrupt).unwrap(), b"invalid-private-record");
    }

    #[test]
    fn mid_batch_write_failure_preserves_actual_success_and_stops_next_items() {
        let items = [fixture(Path::new("first")), fixture(Path::new("second"))];
        let mut deletions = 0;
        let sum = clean_with(
            &items,
            None,
            |_| {
                deletions += 1;
                Ok(())
            },
            |_| Err("disk full".into()),
        );
        assert_eq!(deletions, 1);
        assert_eq!(sum.freed_bytes, 5);
        assert!(sum.records[0].ok);
        assert!(!sum.records[1].ok);
        assert!(sum.log_error.unwrap().contains("disk full"));
    }

    #[test]
    fn legacy_record_without_final_newline_stays_readable_after_append() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("history.jsonl");
        let record = r#"{"at":"2026-01-01T00:00:00Z","rule_id":"fixture","name":"x","path":"x","size_bytes":5,"safety":"safe","ok":true,"error":null}"#;
        std::fs::write(&path, record).unwrap();
        let file = open_log(&path).unwrap();
        drop(file);
        assert_eq!(read_log(&path).unwrap().len(), 1);
        assert_eq!(read_log(&path).unwrap()[0].batch, "");
        let item = fixture(&tmp.path().join("missing"));
        let sum = clean(&[item], &path);
        assert!(sum.log_error.is_none());
        assert_eq!(read_log(&path).unwrap().len(), 2);
    }

    #[test]
    fn guard_refuses_roots_and_home() {
        assert!(guard(Path::new("/")).is_err());
        if let Some(h) = dirs::home_dir() {
            assert!(guard(&h).is_err());
        }
    }

    #[test]
    fn guard_refuses_missing_and_symlinks() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(guard(&tmp.path().join("missing")).is_err());
        let real = tmp.path().join("real");
        std::fs::create_dir(&real).unwrap();
        assert!(guard(&real).is_ok());
        #[cfg(unix)]
        {
            let link = tmp.path().join("link");
            std::os::unix::fs::symlink(&real, &link).unwrap();
            assert!(guard(&link).is_err());
        }
    }

    #[test]
    fn log_round_trips() {
        let tmp = tempfile::tempdir().unwrap();
        let log = tmp.path().join("clean.log");
        let rec = CleanRecord {
            at: Utc::now(),
            rule_id: "a.b".into(),
            name: "x".into(),
            path: tmp.path().join("x"),
            size_bytes: 5,
            safety: SafetyLevel::Safe,
            ok: true,
            error: None,
            batch: "1".into(),
        };
        std::fs::write(&log, format!("{}\n", serde_json::to_string(&rec).unwrap())).unwrap();
        let back = read_log(&log).unwrap();
        assert_eq!(back.len(), 1);
        assert_eq!(back[0].size_bytes, 5);
    }
}
