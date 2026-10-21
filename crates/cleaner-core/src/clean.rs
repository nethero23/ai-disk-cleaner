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
use std::io::Write;
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
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CleanSummary {
    pub freed_bytes: u64,
    pub records: Vec<CleanRecord>,
}

/// Refuses paths that no rule should ever hand us, as a last line of defence
/// against a bad rule file or a corrupted scan result.
pub fn guard(path: &Path) -> Result<()> {
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

/// Moves one item to the Recycle Bin.
pub fn delete_item(item: &ScanItem) -> Result<()> {
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

/// Deletes each item, logging to `log_path`, and never stops on one failure.
pub fn clean(items: &[ScanItem], log_path: &Path) -> CleanSummary {
    let mut summary = CleanSummary::default();
    let mut log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .ok();
    for item in items {
        let result = delete_item(item);
        let rec = CleanRecord {
            at: Utc::now(),
            rule_id: item.rule_id.clone(),
            name: item.name.clone(),
            path: item.path.clone(),
            size_bytes: item.size_bytes,
            safety: item.safety,
            ok: result.is_ok(),
            error: result.err().map(|e| e.to_string()),
        };
        if rec.ok {
            summary.freed_bytes += rec.size_bytes;
        }
        if let Some(f) = log.as_mut() {
            if let Ok(line) = serde_json::to_string(&rec) {
                let _ = writeln!(f, "{line}");
            }
        }
        summary.records.push(rec);
    }
    summary
}

/// Reads the whole log, newest last.
pub fn read_log(log_path: &Path) -> Vec<CleanRecord> {
    std::fs::read_to_string(log_path)
        .map(|t| {
            t.lines()
                .filter_map(|l| serde_json::from_str(l).ok())
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

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
        };
        std::fs::write(&log, format!("{}\n", serde_json::to_string(&rec).unwrap())).unwrap();
        let back = read_log(&log);
        assert_eq!(back.len(), 1);
        assert_eq!(back[0].size_bytes, 5);
    }
}
