//! Walks the locations the rules point at and builds a [`ScanReport`].

use crate::inspect;
use crate::model::{human_size, ScanItem, ScanReport};
use crate::paths::PathContext;
use crate::rules::{Inspector, Rule, RuleSet};
use chrono::{DateTime, Utc};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct ScanOptions {
    pub ctx: PathContext,
    /// `zh` or `en`; picks which rule strings go into the items.
    pub lang: String,
    /// Skip rules below this size even if the rule itself has no floor.
    pub global_min_bytes: u64,
    /// Lower-cased names of running processes, sampled once per scan.
    pub running: std::collections::HashSet<String>,
    /// Folders the user asked to leave alone; items at or under them are dropped.
    pub excluded: Vec<PathBuf>,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            ctx: PathContext::from_env(),
            lang: "zh".into(),
            global_min_bytes: 1024 * 1024,
            running: crate::process::running_names(),
            excluded: Vec::new(),
        }
    }
}

/// Size, count and newest timestamp of a tree. Never follows symlinks or
/// junctions, so a junction into `C:\Program Files` is counted as 0 bytes
/// rather than as something we might delete.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TreeStats {
    pub bytes: u64,
    pub files: u64,
    pub newest: Option<DateTime<Utc>>,
}

pub fn tree_stats(root: &Path) -> TreeStats {
    let mut s = TreeStats::default();
    if root.is_symlink() {
        return s;
    }
    for entry in WalkDir::new(root).follow_links(false).into_iter().flatten() {
        let ft = entry.file_type();
        if ft.is_symlink() || !ft.is_file() {
            continue;
        }
        if let Ok(md) = entry.metadata() {
            s.bytes += md.len();
            s.files += 1;
            let t = md.modified().ok().or_else(|| md.accessed().ok());
            if let Some(t) = t {
                let t: DateTime<Utc> = t.into();
                s.newest = Some(s.newest.map_or(t, |n| n.max(t)));
            }
        }
    }
    s
}

/// Reports whether a directory is readable at all, so permission problems
/// surface as "unreadable" rather than as "nothing here".
fn readable_dir(p: &Path) -> Option<bool> {
    if !p.exists() {
        return None;
    }
    Some(std::fs::read_dir(p).is_ok())
}

/// Scans every rule in `rules` and returns the report, largest items first.
pub fn scan_all(rules: &RuleSet, opts: &ScanOptions) -> ScanReport {
    scan_all_with(rules, opts, &AtomicBool::new(false), &mut |_| {})
}

/// What the scan reports as it goes: a rule just finished.
#[derive(Debug, Clone)]
pub struct ScanProgress {
    pub done: usize,
    pub total: usize,
    /// The rule that just finished and how long it took.
    pub rule_id: String,
    pub rule_name: String,
    pub millis: u64,
}

/// Number of rules walked at the same time. Walking is I/O bound, so a few
/// threads hide most of the latency without thrashing a laptop disk.
const WORKERS: usize = 4;

/// Like [`scan_all`], but walks rules in parallel, reports each finished
/// rule through `progress`, and stops early when `cancel` is set (the
/// report then holds what was found so far and `cancelled` is true).
pub fn scan_all_with(
    rules: &RuleSet,
    opts: &ScanOptions,
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(ScanProgress),
) -> ScanReport {
    let mut report = ScanReport {
        started_at: Some(Utc::now()),
        ..Default::default()
    };
    let todo: Vec<&Rule> = rules
        .rules
        .iter()
        .filter(|r| !(r.windows_only && !cfg!(windows)))
        .collect();
    let total = todo.len();
    let next = AtomicUsize::new(0);
    let (tx, rx) = mpsc::channel::<(usize, ScanReport, u64)>();

    std::thread::scope(|scope| {
        for _ in 0..WORKERS.min(total.max(1)) {
            let tx = tx.clone();
            let todo = &todo;
            let next = &next;
            scope.spawn(move || loop {
                if cancel.load(Ordering::Relaxed) {
                    break;
                }
                let i = next.fetch_add(1, Ordering::Relaxed);
                let Some(rule) = todo.get(i) else { break };
                let started = std::time::Instant::now();
                let mut part = ScanReport::default();
                scan_rule(rule, opts, &mut part);
                if tx
                    .send((i, part, started.elapsed().as_millis() as u64))
                    .is_err()
                {
                    break;
                }
            });
        }
        drop(tx);
        for (n, (i, part, millis)) in rx.into_iter().enumerate() {
            let done = n + 1;
            let rule = todo[i];
            for item in part.items {
                report.push(item);
            }
            report.unreadable.extend(part.unreadable);
            report.timings.push((rule.id.clone(), millis));
            progress(ScanProgress {
                done,
                total,
                rule_id: rule.id.clone(),
                rule_name: rule_name(rule, opts),
                millis,
            });
        }
    });

    report.cancelled = cancel.load(Ordering::Relaxed) && report.timings.len() < total;
    report.timings.sort_by_key(|(_, ms)| std::cmp::Reverse(*ms));
    report.sort_by_size();
    report.finished_at = Some(Utc::now());
    report
}

/// Resolves the rule's templates to the locations that exist on this machine.
pub fn resolve_paths(rule: &Rule, ctx: &PathContext) -> Vec<PathBuf> {
    let mut found = Vec::new();
    for t in &rule.paths {
        let Some(p) = ctx.expand(t) else { continue };
        if p.exists() && !found.contains(&p) {
            found.push(p);
            if !rule.all_paths {
                break;
            }
        }
    }
    found
}

pub fn scan_rule(rule: &Rule, opts: &ScanOptions, report: &mut ScanReport) {
    let excluded = |p: &Path| opts.excluded.iter().any(|ex| p.starts_with(ex));
    for root in resolve_paths(rule, &opts.ctx) {
        if excluded(&root) {
            continue;
        }
        match readable_dir(&root) {
            Some(false) => {
                report.unreadable.push(root);
                continue;
            }
            None => continue,
            Some(true) => {}
        }
        let items = match rule.inspector {
            Inspector::Plain => vec![plain_item(rule, &root, opts)],
            Inspector::Children => inspect::children(rule, &root, opts),
            Inspector::OllamaModels => inspect::ollama::models(rule, &root, opts),
            Inspector::HuggingFaceHub => inspect::huggingface::hub(rule, &root, opts),
            Inspector::LmStudioModels => inspect::lmstudio::models(rule, &root, opts),
            Inspector::CursorWorkspaces => inspect::cursor::workspaces(rule, &root, opts),
            Inspector::CondaEnvs => inspect::conda::envs(rule, &root, opts),
            Inspector::ClaudeProjects => inspect::claude::projects(rule, &root, opts),
            Inspector::ProjectArtifacts => inspect::artifacts::artifacts(rule, &root, opts),
        };
        let floor = rule.min_bytes.max(opts.global_min_bytes);
        for item in items
            .into_iter()
            .filter(|i| i.size_bytes >= floor && !excluded(&i.path))
        {
            report.push(item);
        }
    }
}

/// Builds a single item for a location, the default for most rules.
pub fn plain_item(rule: &Rule, path: &Path, opts: &ScanOptions) -> ScanItem {
    let stats = tree_stats(path);
    make_item(rule, path, rule_name(rule, opts), stats, opts, None)
}

pub fn rule_name(rule: &Rule, opts: &ScanOptions) -> String {
    if opts.lang == "en" {
        rule.name_en.clone()
    } else {
        rule.name_zh.clone()
    }
}

/// Shared constructor so every inspector fills the item the same way.
pub fn make_item(
    rule: &Rule,
    path: &Path,
    name: String,
    stats: TreeStats,
    opts: &ScanOptions,
    detail: Option<String>,
) -> ScanItem {
    let consequence = if opts.lang == "en" {
        rule.consequence_en.clone()
    } else {
        rule.consequence_zh.clone()
    };
    ScanItem {
        id: format!("{}::{}", rule.id, path.to_string_lossy()),
        rule_id: rule.id.clone(),
        tool: rule.tool.clone(),
        category: rule.category,
        name,
        path: path.to_path_buf(),
        size_bytes: stats.bytes,
        file_count: stats.files,
        last_used: stats.newest,
        safety: rule.safety,
        consequence: consequence.replace("{size}", &human_size(stats.bytes)),
        detail,
        in_use_by: crate::process::first_running(&rule.processes, &opts.running)
            .map(str::to_string),
        auto_clean: rule.auto_clean,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Category, SafetyLevel};
    use std::collections::HashMap;
    use std::fs;

    fn rule(id: &str, paths: &[&str], inspector: Inspector) -> Rule {
        Rule {
            id: id.into(),
            tool: "Test".into(),
            category: Category::PackageCache,
            name_zh: "测试".into(),
            name_en: "Test".into(),
            paths: paths.iter().map(|s| s.to_string()).collect(),
            all_paths: false,
            safety: SafetyLevel::Safe,
            inspector,
            consequence_zh: "会重建 {size}".into(),
            consequence_en: "rebuilds {size}".into(),
            min_bytes: 0,
            windows_only: false,
            require: vec![],
            processes: vec![],
            auto_clean: true,
        }
    }

    fn opts(home: &Path) -> ScanOptions {
        let mut vars = HashMap::new();
        vars.insert("USERPROFILE".to_string(), home.to_path_buf());
        ScanOptions {
            ctx: PathContext::with_vars(vars),
            lang: "zh".into(),
            global_min_bytes: 0,
            running: Default::default(),
            excluded: Vec::new(),
        }
    }

    #[test]
    fn excluded_folders_are_not_reported() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();
        fs::create_dir_all(home.join("ws/a")).unwrap();
        fs::create_dir_all(home.join("ws/b")).unwrap();
        fs::write(home.join("ws/a/f"), [0u8; 10]).unwrap();
        fs::write(home.join("ws/b/f"), [0u8; 10]).unwrap();
        let r = rule("c.k", &["${USERPROFILE}/ws"], Inspector::Children);
        let mut o = opts(home);
        o.excluded = vec![home.join("ws/a")];
        let mut report = ScanReport::default();
        scan_rule(&r, &o, &mut report);
        assert_eq!(report.items.len(), 1);
        assert!(report.items[0].path.ends_with("b"));
        o.excluded = vec![home.join("ws")];
        let mut report = ScanReport::default();
        scan_rule(&r, &o, &mut report);
        assert!(report.items.is_empty());
    }

    #[test]
    fn parallel_scan_reports_every_rule_and_can_be_cancelled() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();
        for i in 0..6 {
            fs::create_dir_all(home.join(format!("c{i}"))).unwrap();
            fs::write(home.join(format!("c{i}/f")), [0u8; 10]).unwrap();
        }
        let set = RuleSet {
            rules: (0..6)
                .map(|i| {
                    rule(
                        &format!("r{i}"),
                        &[&format!("${{USERPROFILE}}/c{i}")],
                        Inspector::Plain,
                    )
                })
                .collect(),
        };
        let o = opts(home);
        let mut seen = Vec::new();
        let report = scan_all_with(&set, &o, &AtomicBool::new(false), &mut |p| {
            seen.push((p.done, p.total));
        });
        assert_eq!(report.items.len(), 6);
        assert_eq!(report.timings.len(), 6);
        assert!(!report.cancelled);
        assert_eq!(seen.last(), Some(&(6, 6)));
        // Items stay largest-first across workers.
        assert!(report
            .items
            .windows(2)
            .all(|w| w[0].size_bytes >= w[1].size_bytes));

        // A cancel set before the start finishes with nothing walked.
        let report = scan_all_with(&set, &o, &AtomicBool::new(true), &mut |_| {});
        assert!(report.cancelled);
        assert!(report.items.is_empty());
    }

    #[test]
    fn tree_stats_counts_files_not_symlinks() {
        let tmp = tempfile::tempdir().unwrap();
        let d = tmp.path().join("cache");
        fs::create_dir_all(d.join("sub")).unwrap();
        fs::write(d.join("a.bin"), vec![0u8; 1000]).unwrap();
        fs::write(d.join("sub/b.bin"), vec![0u8; 24]).unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(tmp.path(), d.join("loop")).unwrap();
        let s = tree_stats(&d);
        assert_eq!(s.bytes, 1024);
        assert_eq!(s.files, 2);
        assert!(s.newest.is_some());
    }

    #[test]
    fn plain_rule_yields_one_item_with_substituted_consequence() {
        let tmp = tempfile::tempdir().unwrap();
        let d = tmp.path().join("pipcache");
        fs::create_dir_all(&d).unwrap();
        fs::write(d.join("x"), vec![0u8; 2048]).unwrap();
        let r = rule("pip.cache", &["${USERPROFILE}/pipcache"], Inspector::Plain);
        let set = RuleSet { rules: vec![r] };
        let report = scan_all(&set, &opts(tmp.path()));
        assert_eq!(report.items.len(), 1);
        assert_eq!(report.items[0].size_bytes, 2048);
        assert_eq!(report.items[0].consequence, "会重建 2.0 KB");
        assert_eq!(report.safe_bytes, 2048);
    }

    #[test]
    fn missing_paths_produce_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let r = rule(
            "x.y",
            &["${USERPROFILE}/nope", "${NOT_SET}/a"],
            Inspector::Plain,
        );
        let set = RuleSet { rules: vec![r] };
        let report = scan_all(&set, &opts(tmp.path()));
        assert!(report.items.is_empty());
        assert!(report.unreadable.is_empty());
    }

    #[test]
    fn children_inspector_lists_each_subfolder() {
        let tmp = tempfile::tempdir().unwrap();
        let d = tmp.path().join("ws");
        fs::create_dir_all(d.join("one")).unwrap();
        fs::create_dir_all(d.join("two")).unwrap();
        fs::write(d.join("one/a"), vec![0u8; 10]).unwrap();
        fs::write(d.join("two/a"), vec![0u8; 20]).unwrap();
        let r = rule("c.k", &["${USERPROFILE}/ws"], Inspector::Children);
        let set = RuleSet { rules: vec![r] };
        let report = scan_all(&set, &opts(tmp.path()));
        assert_eq!(report.items.len(), 2);
        assert_eq!(report.items[0].name, "two");
        assert_eq!(report.items[0].size_bytes, 20);
    }
}
