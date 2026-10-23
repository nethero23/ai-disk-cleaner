//! Windows integration: only a freshly created temporary fixture enters the bin.
#![cfg(windows)]

use cleaner_core::clean;
use cleaner_core::paths::PathContext;
use cleaner_core::restore::{self, Whereabouts};
use cleaner_core::{scan_all, RuleSet, ScanOptions};
use std::collections::HashMap;

#[test]
fn scanned_fixture_can_be_recycled_and_restored_byte_for_byte() {
    let tmp = tempfile::tempdir().unwrap();
    let cache = tmp.path().join("acceptance-cache");
    std::fs::create_dir(&cache).unwrap();
    let file = cache.join("sentinel.txt");
    let original = b"Clearwisp synthetic Recycle Bin acceptance fixture";
    std::fs::write(&file, original).unwrap();
    let rules = RuleSet::parse(
        r#"
[[rule]]
id = "acceptance.fixture"
tool = "Acceptance Fixture"
category = "package_cache"
name_zh = "验收缓存"
name_en = "Acceptance cache"
paths = ["${FIXTURE}/acceptance-cache"]
safety = "safe"
"#,
    )
    .unwrap();
    let opts = ScanOptions {
        ctx: PathContext::with_vars(HashMap::from([("FIXTURE".into(), tmp.path().into())])),
        global_min_bytes: 0,
        ..Default::default()
    };
    let report = scan_all(&rules, &opts);
    assert_eq!(report.items.len(), 1);
    assert_eq!(report.items[0].path, cache);
    assert!(cache.starts_with(tmp.path()));
    assert_eq!(std::fs::read(&file).unwrap(), original);

    let log = tmp.path().join("clean-log.jsonl");
    let cleaned = clean::clean(&report.items, &log);
    assert_eq!(cleaned.records.len(), 1);
    assert!(cleaned.records[0].ok, "{:?}", cleaned.records[0].error);
    assert!(!cache.exists());
    let status = restore::status(std::slice::from_ref(&cache));
    let restored = restore::restore(&cleaned.records);
    assert_eq!(restored.restored, 1, "{:?}", restored.results);
    assert_eq!(status[0].state, Whereabouts::InBin);
    assert_eq!(restored.restored_bytes, original.len() as u64);
    assert_eq!(std::fs::read(&file).unwrap(), original);
    assert_eq!(clean::read_log(&log).unwrap().len(), 1);
}
