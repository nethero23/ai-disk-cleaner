//! Tiny CLI over the engine, for development and for scripting:
//!   aidc scan            # table of what a scan finds
//!   aidc scan --json     # the same as JSON
//!   aidc scan --drive D:\\  # also walk a whole drive for caches by shape
//!   aidc rules           # the rule library
//!   aidc dupes           # duplicate model files across tools (read only)
//!   aidc move <rule_id> <path> <dest_folder>   # move a cache to another drive

use cleaner_core::model::human_size;
use cleaner_core::paths::PathContext;
use cleaner_core::{bigfiles, dedup, relocate, scan_all, vhdx, RuleSet, ScanOptions};
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let json = args.iter().any(|a| a == "--json");
    let lang = if args.iter().any(|a| a == "--en") {
        "en"
    } else {
        "zh"
    };
    match args.first().map(String::as_str) {
        Some("rules") => {
            for r in RuleSet::builtin().rules {
                println!("{:<32} {:<14} {:?}", r.id, r.tool, r.safety);
            }
        }
        Some("scan") | None => {
            let rules = RuleSet::builtin();
            let extra_roots = args
                .windows(2)
                .filter(|w| w[0] == "--drive")
                .map(|w| std::path::PathBuf::from(&w[1]))
                .collect();
            let opts = ScanOptions {
                lang: lang.into(),
                extra_roots,
                ..ScanOptions::default()
            };
            let report = scan_all(&rules, &opts);
            if json {
                println!("{}", serde_json::to_string_pretty(&report).unwrap());
                return;
            }
            for it in &report.items {
                println!(
                    "{:>9}  {:<10}  {:<16} {:<40} {}{}",
                    human_size(it.size_bytes),
                    format!("{:?}", it.safety),
                    it.tool,
                    it.name,
                    it.path.display(),
                    it.in_use_by
                        .as_deref()
                        .map(|p| format!("  [in use: {p}]"))
                        .unwrap_or_default()
                );
            }
            println!(
                "\ntotal {}  (safe {}, redownload {}, review {})",
                human_size(report.total_bytes),
                human_size(report.safe_bytes),
                human_size(report.redownload_bytes),
                human_size(report.review_bytes)
            );
            for p in &report.unreadable {
                println!("unreadable: {}", p.display());
            }
        }
        Some("dupes") => {
            let report = scan_all(&RuleSet::builtin(), &ScanOptions::default());
            let groups = dedup::find_duplicates(&report.items);
            for g in &groups {
                println!(
                    "{} x{}  reclaimable {}{}",
                    human_size(g.size_bytes),
                    g.files.len(),
                    human_size(g.reclaimable_bytes),
                    if g.same_volume {
                        ""
                    } else {
                        "  (different drives)"
                    }
                );
                for f in &g.files {
                    println!("    {:<14} {}", f.tool, f.path.display());
                }
            }
            println!("{} duplicate group(s)", groups.len());
        }
        Some("big") => {
            // aidc big [--min <MB>] [root...]
            let mut min_mb = 500u64;
            let mut roots = Vec::new();
            let mut it = args[1..].iter();
            while let Some(a) = it.next() {
                if a == "--min" {
                    min_mb = it.next().and_then(|v| v.parse().ok()).unwrap_or(min_mb);
                } else {
                    roots.push(std::path::PathBuf::from(a));
                }
            }
            if roots.is_empty() {
                roots = bigfiles::drives().into_iter().map(|d| d.path).collect();
            }
            let cancel = std::sync::atomic::AtomicBool::new(false);
            let items = bigfiles::scan(
                &roots,
                min_mb * 1024 * 1024,
                &[],
                "en",
                &cancel,
                &mut |_, _, _| {},
            );
            for i in &items {
                println!(
                    "{:>10}  {:<14} {}",
                    human_size(i.size_bytes),
                    i.tool,
                    i.path.display()
                );
            }
            println!("{} file(s) over {min_mb} MB", items.len());
        }
        Some("compact") => {
            let Some(path) = args.get(1) else {
                eprintln!("usage: aidc compact <file.vhdx>");
                std::process::exit(2);
            };
            match vhdx::compact(Path::new(path)) {
                Ok(r) => println!(
                    "{}: {} -> {}",
                    r.path.display(),
                    human_size(r.before),
                    human_size(r.after)
                ),
                Err(e) => {
                    eprintln!("error: {e}");
                    std::process::exit(1);
                }
            }
        }
        Some("move") => {
            let (Some(rule_id), Some(path), Some(dest)) = (args.get(1), args.get(2), args.get(3))
            else {
                eprintln!("usage: aidc move <rule_id> <path> <dest_folder>");
                std::process::exit(2);
            };
            let src = relocate::move_source(rule_id, Path::new(path), &PathContext::from_env());
            match relocate::relocate(rule_id, &src, Path::new(dest), &mut |_, _| {}) {
                Ok(r) => println!(
                    "moved {} ({} files, {}) to {} via {:?} {}",
                    r.from.display(),
                    r.files,
                    human_size(r.bytes),
                    r.to.display(),
                    r.redirect,
                    r.note
                ),
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(1);
                }
            }
        }
        Some(other) => {
            eprintln!("unknown command {other}; use scan, rules, dupes or move");
            std::process::exit(2);
        }
    }
}
