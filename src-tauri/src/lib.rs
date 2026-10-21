//! Tauri shell: exposes the core engine to the web UI as commands.
//! All heavy work runs off the main thread so the window stays responsive.

use cleaner_core::clean::{self, CleanRecord, CleanSummary};
use cleaner_core::dedup::{self, DuplicateGroup};
use cleaner_core::paths::PathContext;
use cleaner_core::relocate::{self, RelocateRecord, RelocateResult};
use cleaner_core::{scan_all, RuleSet, ScanItem, ScanOptions, ScanReport};
use serde::Serialize;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager};

/// Where the clean log lives: the app's own data folder, never the user's documents.
fn log_path(app: &AppHandle) -> PathBuf {
    let dir = app.path().app_data_dir().unwrap_or_else(|_| {
        dirs::data_local_dir()
            .unwrap_or_else(std::env::temp_dir)
            .join("ai-disk-cleaner")
    });
    let _ = std::fs::create_dir_all(&dir);
    dir.join("clean-log.jsonl")
}

fn relocate_log_path(app: &AppHandle) -> PathBuf {
    log_path(app).with_file_name("relocate-log.jsonl")
}

/// Throttled progress emitter shared by move and move-back.
fn progress_emitter(app: AppHandle, event: &'static str) -> impl FnMut(u64, u64) {
    let mut last_emit = std::time::Instant::now();
    move |copied: u64, total: u64| {
        if last_emit.elapsed().as_millis() > 200 || copied == total {
            let _ = app.emit(event, RelocateProgress { copied, total });
            last_emit = std::time::Instant::now();
        }
    }
}

#[tauri::command]
async fn scan(lang: String) -> Result<ScanReport, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let rules = RuleSet::builtin();
        let opts = ScanOptions {
            lang,
            ..ScanOptions::default()
        };
        scan_all(&rules, &opts)
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
async fn clean_items(app: AppHandle, items: Vec<ScanItem>) -> Result<CleanSummary, String> {
    let log = log_path(&app);
    tauri::async_runtime::spawn_blocking(move || {
        // The scan may be minutes old: re-check which owning tools are running now.
        let rules = RuleSet::builtin();
        let running = cleaner_core::process::running_names();
        let mut items = items;
        for item in &mut items {
            if let Some(rule) = rules.get(&item.rule_id) {
                item.in_use_by = cleaner_core::process::first_running(&rule.processes, &running)
                    .map(str::to_string);
            }
        }
        clean::clean(&items, &log)
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn clean_log(app: AppHandle) -> Vec<CleanRecord> {
    clean::read_log(&log_path(&app))
}

#[derive(Serialize)]
struct RuleInfo {
    id: String,
    tool: String,
    name_zh: String,
    name_en: String,
    safety: cleaner_core::SafetyLevel,
}

/// The rule library, so the UI can show "we check N locations" and a settings page later.
#[tauri::command]
fn rules() -> Vec<RuleInfo> {
    RuleSet::builtin()
        .rules
        .into_iter()
        .map(|r| RuleInfo {
            id: r.id,
            tool: r.tool,
            name_zh: r.name_zh,
            name_en: r.name_en,
            safety: r.safety,
        })
        .collect()
}

#[derive(Serialize, Clone)]
struct RelocateProgress {
    copied: u64,
    total: u64,
}

/// Which rule ids can be moved to another drive (the UI shows a Move button for them).
#[tauri::command]
fn relocatable() -> Vec<String> {
    RuleSet::builtin()
        .rules
        .into_iter()
        .filter(|r| relocate::redirect_for(&r.id).is_some())
        .map(|r| r.id)
        .collect()
}

/// The folder a move would actually take (a whole store for env-var tools),
/// so the confirmation shows it before anything happens.
#[tauri::command]
fn move_source(rule_id: String, path: PathBuf) -> PathBuf {
    relocate::move_source(&rule_id, &path, &PathContext::from_env())
}

#[tauri::command]
async fn relocate_item(
    app: AppHandle,
    rule_id: String,
    path: PathBuf,
    dest_root: PathBuf,
) -> Result<RelocateResult, String> {
    let log = relocate_log_path(&app);
    tauri::async_runtime::spawn_blocking(move || {
        let mut progress = progress_emitter(app, "relocate-progress");
        let src = relocate::move_source(&rule_id, &path, &PathContext::from_env());
        let r = relocate::relocate(&rule_id, &src, &dest_root, &mut progress)
            .map_err(|e| e.to_string())?;
        relocate::log_relocation(&log, &rule_id, &r);
        Ok(r)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Relocations still in effect, newest first.
#[tauri::command]
fn relocations(app: AppHandle) -> Vec<RelocateRecord> {
    relocate::read_relocations(&relocate_log_path(&app))
}

#[tauri::command]
async fn move_back(app: AppHandle, record: RelocateRecord) -> Result<(), String> {
    let log = relocate_log_path(&app);
    tauri::async_runtime::spawn_blocking(move || {
        let mut progress = progress_emitter(app, "relocate-progress");
        relocate::move_back(&log, &record, &mut progress).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Rule ids whose items can be compacted.
#[tauri::command]
fn compactable() -> Vec<String> {
    vhdx::COMPACTABLE_RULES
        .iter()
        .map(|s| s.to_string())
        .collect()
}

#[tauri::command]
fn vhdx_files(path: PathBuf) -> Vec<Vhdx> {
    vhdx::find(&path)
}

#[tauri::command]
async fn compact_vhdx(path: PathBuf) -> Result<CompactResult, String> {
    tauri::async_runtime::spawn_blocking(move || vhdx::compact(&path).map_err(|e| e.to_string()))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn find_duplicates(items: Vec<ScanItem>) -> Result<Vec<DuplicateGroup>, String> {
    tauri::async_runtime::spawn_blocking(move || dedup::find_duplicates(&items))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn merge_duplicates(group: DuplicateGroup, keep: PathBuf) -> Result<u64, String> {
    tauri::async_runtime::spawn_blocking(move || {
        dedup::merge_group(&group, &keep).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            scan,
            clean_items,
            clean_log,
            rules,
            relocatable,
            move_source,
            relocate_item,
            relocations,
            move_back,
            find_duplicates,
            merge_duplicates
        ])
        .run(tauri::generate_context!())
        .expect("error while running ai-disk-cleaner");
}
