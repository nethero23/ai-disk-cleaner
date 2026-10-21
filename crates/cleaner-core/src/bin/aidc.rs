//! Tiny CLI over the engine, for development and for scripting:
//!   aidc scan            # table of what a scan finds
//!   aidc scan --json     # the same as JSON
//!   aidc rules           # the rule library

use cleaner_core::model::human_size;
use cleaner_core::{scan_all, RuleSet, ScanOptions};

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
            let opts = ScanOptions {
                lang: lang.into(),
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
        Some(other) => {
            eprintln!("unknown command {other}; use scan or rules");
            std::process::exit(2);
        }
    }
}
