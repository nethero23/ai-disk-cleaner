//! Tauri shell: exposes the core engine to the web UI as commands.
//! All heavy work runs off the main thread so the window stays responsive.

use cleaner_core::clean::{self, CleanRecord, CleanSummary};
use cleaner_core::{scan_all, RuleSet, ScanItem, ScanOptions, ScanReport};
use serde::Serialize;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            scan,
            clean_items,
            clean_log,
            rules
        ])
        .run(tauri::generate_context!())
        .expect("error while running ai-disk-cleaner");
}
