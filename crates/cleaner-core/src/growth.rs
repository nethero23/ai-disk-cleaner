//! Growth monitoring: remember how much each tool held at every scan and
//! say who is growing. Snapshots are small (one line per scan with the
//! per-tool totals) and appended to a JSON-lines file next to the clean log.

use crate::model::ScanReport;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Keep at most this many snapshots so the file never grows without bound.
const KEEP: usize = 2000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub at: DateTime<Utc>,
    pub total_bytes: u64,
    pub by_tool: BTreeMap<String, u64>,
    /// What else was scanned besides the system drive (extra roots, sorted,
    /// lower-case). Two scans with different scopes are not comparable:
    /// ticking `D:\` adds that drive's project folders to "项目" and would
    /// look like 12 GB of growth overnight.
    #[serde(default)]
    pub scope: Vec<String>,
}

/// The scope key for a set of extra roots.
pub fn scope_key(extra_roots: &[PathBuf]) -> Vec<String> {
    let mut v: Vec<String> = extra_roots
        .iter()
        .map(|p| {
            p.to_string_lossy()
                .to_ascii_lowercase()
                .trim_end_matches(['\\', '/'])
                .to_string()
        })
        .collect();
    v.sort();
    v.dedup();
    v
}

impl Snapshot {
    pub fn from_report(report: &ScanReport, extra_roots: &[PathBuf]) -> Self {
        let mut by_tool = BTreeMap::new();
        for i in &report.items {
            *by_tool.entry(i.tool.clone()).or_insert(0) += i.size_bytes;
        }
        Self {
            at: report.finished_at.unwrap_or_else(Utc::now),
            total_bytes: report.total_bytes,
            by_tool,
            scope: scope_key(extra_roots),
        }
    }
}

/// One tool's change over the window.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Growth {
    pub tool: String,
    pub now_bytes: u64,
    pub before_bytes: u64,
    /// Positive means it grew.
    pub delta_bytes: i64,
    /// When the "before" snapshot was taken.
    pub since: DateTime<Utc>,
}

/// Appends a snapshot of this scan, trimming the file to the newest `KEEP`.
pub fn record(path: &Path, report: &ScanReport, extra_roots: &[PathBuf]) {
    // A cancelled scan under-reports every tool; recording it would look like a drop.
    if report.cancelled {
        return;
    }
    let snap = Snapshot::from_report(report, extra_roots);
    let mut all = read(path);
    all.push(snap);
    if all.len() > KEEP {
        let drop = all.len() - KEEP;
        all.drain(..drop);
    }
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Ok(mut f) = std::fs::File::create(path) {
        for s in &all {
            if let Ok(line) = serde_json::to_string(s) {
                let _ = writeln!(f, "{line}");
            }
        }
    }
}

/// All snapshots, oldest first.
pub fn read(path: &Path) -> Vec<Snapshot> {
    std::fs::read_to_string(path)
        .map(|t| {
            t.lines()
                .filter_map(|l| serde_json::from_str(l).ok())
                .collect()
        })
        .unwrap_or_default()
}

/// Compares the newest snapshot with the oldest one taken within `window`
/// days of it and with the same scope, per tool, biggest growth first.
/// Tools absent from the earlier snapshot count as 0 before (they are new).
/// Needs two comparable snapshots.
pub fn growth(snapshots: &[Snapshot], window_days: i64) -> Vec<Growth> {
    let Some(latest) = snapshots.last() else {
        return Vec::new();
    };
    let cutoff = latest.at - Duration::days(window_days);
    let Some(before) = snapshots
        .iter()
        .filter(|s| s.at >= cutoff && s.at < latest.at && s.scope == latest.scope)
        .min_by_key(|s| s.at)
    else {
        return Vec::new();
    };
    let mut out: Vec<Growth> = latest
        .by_tool
        .iter()
        .map(|(tool, now)| {
            let was = before.by_tool.get(tool).copied().unwrap_or(0);
            Growth {
                tool: tool.clone(),
                now_bytes: *now,
                before_bytes: was,
                delta_bytes: *now as i64 - was as i64,
                since: before.at,
            }
        })
        .filter(|g| g.delta_bytes != 0)
        .collect();
    out.sort_by_key(|g| std::cmp::Reverse(g.delta_bytes));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(days_ago: i64, pairs: &[(&str, u64)]) -> Snapshot {
        Snapshot {
            at: Utc::now() - Duration::days(days_ago),
            total_bytes: pairs.iter().map(|p| p.1).sum(),
            by_tool: pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
            scope: Vec::new(),
        }
    }

    #[test]
    fn scans_of_different_scope_are_not_compared() {
        let mut wide = snap(1, &[("项目", 12_000)]);
        wide.scope = scope_key(&[PathBuf::from("D:\\")]);
        let snaps = vec![snap(2, &[("项目", 100)]), wide.clone()];
        // Only one snapshot in the D: scope: nothing to compare with.
        assert!(growth(&snaps, 7).is_empty());
        let mut wide2 = snap(0, &[("项目", 12_500)]);
        wide2.scope = scope_key(&[PathBuf::from("d:")]);
        let snaps = vec![snap(2, &[("项目", 100)]), wide, wide2];
        let g = growth(&snaps, 7);
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].delta_bytes, 500);
        // Older files without a scope load as "system drive only".
        let old: Snapshot =
            serde_json::from_str(r#"{"at":"2026-01-01T00:00:00Z","total_bytes":1,"by_tool":{}}"#)
                .unwrap();
        assert!(old.scope.is_empty());
    }

    #[test]
    fn growth_compares_with_oldest_snapshot_inside_the_window() {
        let snaps = vec![
            snap(30, &[("Cursor", 100)]), // outside the 7-day window
            snap(6, &[("Cursor", 200), ("pip", 50)]),
            snap(3, &[("Cursor", 250), ("pip", 50)]),
            snap(0, &[("Cursor", 500), ("Ollama", 900)]),
        ];
        let g = growth(&snaps, 7);
        assert_eq!(g[0].tool, "Ollama");
        assert_eq!(g[0].delta_bytes, 900);
        assert_eq!(g[1].tool, "Cursor");
        assert_eq!(g[1].before_bytes, 200);
        assert_eq!(g[1].delta_bytes, 300);
        // pip disappeared from the latest snapshot: not reported (it is not growing).
        assert!(g.iter().all(|x| x.tool != "pip"));
        assert!(growth(&snaps[..1], 7).is_empty());
    }

    #[test]
    fn record_round_trips_and_skips_cancelled() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().join("growth.jsonl");
        let mut r = ScanReport::default();
        r.push(crate::model::ScanItem {
            id: "a::x".into(),
            rule_id: "a".into(),
            tool: "Cursor".into(),
            category: crate::model::Category::AiEditor,
            name: "x".into(),
            path: "/x".into(),
            size_bytes: 7,
            file_count: 1,
            last_used: None,
            safety: crate::model::SafetyLevel::Safe,
            consequence: String::new(),
            detail: None,
            in_use_by: None,
            auto_clean: true,
        });
        record(&p, &r, &[]);
        r.cancelled = true;
        record(&p, &r, &[]);
        let back = read(&p);
        assert_eq!(back.len(), 1);
        assert_eq!(back[0].by_tool["Cursor"], 7);
    }
}
