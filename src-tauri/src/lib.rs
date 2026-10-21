//! Tauri shell: exposes the core engine to the web UI as commands.
//! All heavy work runs off the main thread so the window stays responsive.

use cleaner_core::bigfiles::{self, Drive};
use cleaner_core::clean::{self, CleanRecord, CleanSummary};
use cleaner_core::dedup::{self, DuplicateGroup};
use cleaner_core::export::{self, ExportResult};
use cleaner_core::paths::PathContext;
use cleaner_core::preview::{self, Preview};
use cleaner_core::relocate::{self, RelocateRecord, RelocateResult};
use cleaner_core::scan::{scan_all_with, ScanProgress};
use cleaner_core::settings::Settings;
use cleaner_core::vhdx::{self, CompactResult, Vhdx};
use cleaner_core::{scan_all, RuleSet, ScanItem, ScanOptions, ScanReport};
use serde::Serialize;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager};

mod background;

/// Portable build: a file named `portable` next to the exe makes the app
/// keep settings and logs in `data` beside it instead of AppData.
pub(crate) fn portable_dir() -> Option<PathBuf> {
    let exe_dir = std::env::current_exe().ok()?.parent()?.to_path_buf();
    exe_dir
        .join("portable")
        .is_file()
        .then(|| exe_dir.join("data"))
}

#[tauri::command]
fn is_portable() -> bool {
    portable_dir().is_some()
}

/// Where the clean log lives: the app's own data folder, never the user's documents.
pub(crate) fn log_path(app: &AppHandle) -> PathBuf {
    let dir = portable_dir().unwrap_or_else(|| {
        app.path().app_data_dir().unwrap_or_else(|_| {
            dirs::data_local_dir()
                .unwrap_or_else(std::env::temp_dir)
                .join("ai-disk-cleaner")
        })
    });
    let _ = std::fs::create_dir_all(&dir);
    dir.join("clean-log.jsonl")
}

pub(crate) fn growth_path(app: &AppHandle) -> PathBuf {
    log_path(app).with_file_name("growth.jsonl")
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

pub(crate) fn settings_path(app: &AppHandle) -> PathBuf {
    log_path(app).with_file_name("settings.json")
}

/// Where "update rules" stores the downloaded rule files.
fn downloaded_rules_dir(app: &AppHandle) -> PathBuf {
    log_path(app).with_file_name("rules")
}

/// The rule set the scan actually uses: builtin + downloaded + custom, minus
/// disabled. Parse errors in user files come back so the UI can show them.
pub(crate) fn effective_rules(app: &AppHandle) -> (RuleSet, Vec<(PathBuf, String)>, Settings) {
    let settings = Settings::load(&settings_path(app));
    let dir = downloaded_rules_dir(app);
    let (set, errors) = RuleSet::effective(dir.is_dir().then_some(dir.as_path()), &settings);
    (set, errors, settings)
}

static SCAN_CANCEL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[derive(Serialize, Clone)]
struct ScanStep {
    done: usize,
    total: usize,
    rule_id: String,
    rule_name: String,
    millis: u64,
}

/// Full rule scan; emits `scan-progress` after every rule.
#[tauri::command]
async fn scan(app: AppHandle, lang: String) -> Result<ScanReport, String> {
    let (rules, _, settings) = effective_rules(&app);
    SCAN_CANCEL.store(false, std::sync::atomic::Ordering::Relaxed);
    let growth = growth_path(&app);
    tauri::async_runtime::spawn_blocking(move || {
        let opts = ScanOptions {
            lang,
            global_min_bytes: settings.min_item_bytes,
            excluded: settings.excluded_paths,
            extra_roots: settings.scan_drives.clone(),
            ..ScanOptions::default()
        };
        let report = scan_all_with(&rules, &opts, &SCAN_CANCEL, &mut |p: ScanProgress| {
            let _ = app.emit(
                "scan-progress",
                ScanStep {
                    done: p.done,
                    total: p.total,
                    rule_id: p.rule_id,
                    rule_name: p.rule_name,
                    millis: p.millis,
                },
            );
        });
        cleaner_core::growth::record(&growth, &report, &settings.scan_drives);
        report
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn cancel_scan() {
    SCAN_CANCEL.store(true, std::sync::atomic::Ordering::Relaxed);
}

#[tauri::command]
async fn clean_items(app: AppHandle, items: Vec<ScanItem>) -> Result<CleanSummary, String> {
    let log = log_path(&app);
    let (rules_for_check, _, _) = effective_rules(&app);
    tauri::async_runtime::spawn_blocking(move || {
        // The scan may be minutes old: re-check which owning tools are running now.
        let rules = rules_for_check;
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

/// The largest files inside an item, for the "what's inside" expander.
#[tauri::command]
async fn preview_item(path: PathBuf, limit: usize) -> Result<Preview, String> {
    tauri::async_runtime::spawn_blocking(move || preview::preview(&path, limit.clamp(1, 50)))
        .await
        .map_err(|e| e.to_string())
}

/// Who grew over the last week, biggest first (needs at least two scans).
#[tauri::command]
fn growth_report(app: AppHandle) -> Vec<cleaner_core::growth::Growth> {
    cleaner_core::growth::growth(&cleaner_core::growth::read(&growth_path(&app)), 7)
}

#[tauri::command]
fn clean_log(app: AppHandle) -> Vec<CleanRecord> {
    clean::read_log(&log_path(&app))
}

/// For the history page: which cleaned paths can still be restored.
#[tauri::command]
async fn restore_status(
    paths: Vec<PathBuf>,
) -> Result<Vec<cleaner_core::restore::RestoreStatus>, String> {
    tauri::async_runtime::spawn_blocking(move || cleaner_core::restore::status(&paths))
        .await
        .map_err(|e| e.to_string())
}

/// Puts cleaned items back from the Recycle Bin.
#[tauri::command]
async fn restore_items(
    records: Vec<CleanRecord>,
) -> Result<cleaner_core::restore::RestoreSummary, String> {
    tauri::async_runtime::spawn_blocking(move || cleaner_core::restore::restore(&records))
        .await
        .map_err(|e| e.to_string())
}

#[derive(Serialize)]
struct RuleInfo {
    id: String,
    tool: String,
    name_zh: String,
    name_en: String,
    safety: cleaner_core::SafetyLevel,
    enabled: bool,
}

#[derive(Serialize)]
struct RuleLibrary {
    rules: Vec<RuleInfo>,
    /// User rule files that failed to parse: (path, error).
    errors: Vec<(PathBuf, String)>,
}

/// Every rule known to this install (builtin + downloaded + custom), with
/// whether the user has it switched on, so the settings page can list them.
#[tauri::command]
fn rules(app: AppHandle) -> RuleLibrary {
    let settings = Settings::load(&settings_path(&app));
    let dir = downloaded_rules_dir(&app);
    let (set, errors) = RuleSet::effective(
        dir.is_dir().then_some(dir.as_path()),
        &Settings {
            disabled_rules: Vec::new(),
            ..settings.clone()
        },
    );
    RuleLibrary {
        rules: set
            .rules
            .into_iter()
            .map(|r| RuleInfo {
                enabled: !settings.disabled_rules.contains(&r.id),
                id: r.id,
                tool: r.tool,
                name_zh: r.name_zh,
                name_en: r.name_en,
                safety: r.safety,
            })
            .collect(),
        errors,
    }
}

#[tauri::command]
fn get_settings(app: AppHandle) -> Settings {
    Settings::load(&settings_path(&app))
}

#[tauri::command]
fn save_settings(app: AppHandle, settings: Settings) -> Result<(), String> {
    let before = Settings::load(&settings_path(&app));
    settings
        .save(&settings_path(&app))
        .map_err(|e| e.to_string())?;
    if before.autostart != settings.autostart {
        background::apply_autostart(&app, settings.autostart);
    }
    Ok(())
}

/// The UI's language, remembered so tray menu and notifications match it.
#[tauri::command]
fn set_lang(app: AppHandle, lang: String) -> Result<(), String> {
    let mut settings = Settings::load(&settings_path(&app));
    if settings.lang != lang {
        settings.lang = lang.clone();
        settings
            .save(&settings_path(&app))
            .map_err(|e| e.to_string())?;
        background::relabel(&app, &lang);
    }
    Ok(())
}

#[derive(Serialize)]
struct RulesUpdate {
    files: usize,
    rules: usize,
    updated_at: chrono::DateTime<chrono::Utc>,
}

/// Downloads the rule files from `settings.rules_update_url` (a folder URL)
/// into the app's rules dir. Every file is parsed before anything is written,
/// so a broken download never replaces a working set.
#[tauri::command]
async fn update_rules(app: AppHandle) -> Result<RulesUpdate, String> {
    let path = settings_path(&app);
    let mut settings = Settings::load(&path);
    let base = settings.rules_update_url.trim_end_matches('/').to_string();
    if !base.starts_with("https://") {
        return Err("rules URL must start with https://".into());
    }
    let dir = downloaded_rules_dir(&app);
    tauri::async_runtime::spawn_blocking(move || {
        let agent = ureq::Agent::config_builder()
            .timeout_global(Some(std::time::Duration::from_secs(30)))
            .build()
            .new_agent();
        let mut fetched = Vec::new();
        let mut rules = 0;
        for name in RuleSet::builtin_file_names() {
            let url = format!("{base}/{name}");
            let text = agent
                .get(&url)
                .call()
                .map_err(|e| format!("{url}: {e}"))?
                .body_mut()
                .read_to_string()
                .map_err(|e| format!("{url}: {e}"))?;
            let set = RuleSet::parse(&text).map_err(|e| format!("{name}: {e}"))?;
            rules += set.rules.len();
            fetched.push((name, text));
        }
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        for (name, text) in &fetched {
            std::fs::write(dir.join(name), text).map_err(|e| e.to_string())?;
        }
        let updated_at = chrono::Utc::now();
        settings.rules_updated_at = Some(updated_at);
        settings.save(&path).map_err(|e| e.to_string())?;
        Ok(RulesUpdate {
            files: fetched.len(),
            rules,
            updated_at,
        })
    })
    .await
    .map_err(|e| e.to_string())?
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

/// Big profile folders no rule claims (≥ 1 GB), for rule suggestions.
#[tauri::command]
async fn unrecognized(
    app: AppHandle,
) -> Result<Vec<cleaner_core::unrecognized::Unrecognized>, String> {
    let (rules, _, settings) = effective_rules(&app);
    tauri::async_runtime::spawn_blocking(move || {
        let ctx = cleaner_core::paths::PathContext::from_env();
        cleaner_core::unrecognized::find(&rules, &ctx, 1024 * 1024 * 1024, &settings.excluded_paths)
    })
    .await
    .map_err(|e| e.to_string())
}

/// The prefilled GitHub issue URL for one unrecognized folder.
#[tauri::command]
fn suggest_rule_url(entry: cleaner_core::unrecognized::Unrecognized, lang: String) -> String {
    cleaner_core::unrecognized::issue_url("nethero23/ai-disk-cleaner", &entry, &lang)
}

/// AI tools: what each one has configured (MCP servers, skills, plugins,
/// extensions) and what overlaps or is broken. Read-only; keys are masked
/// inside the engine, so nothing secret reaches the UI.
#[tauri::command]
async fn ai_inventory() -> Result<cleaner_core::aiconfig::Inventory, String> {
    tauri::async_runtime::spawn_blocking(|| {
        cleaner_core::aiconfig::inventory(
            &cleaner_core::aiconfig::Catalog::builtin(),
            &cleaner_core::paths::PathContext::from_env(),
        )
    })
    .await
    .map_err(|e| e.to_string())
}

/// Asks npm / PyPI for the latest version of each MCP server package. The
/// request carries the package name only. Off when the setting is off.
#[tauri::command]
async fn ai_version_check(
    app: AppHandle,
    identities: Vec<String>,
) -> Result<Vec<cleaner_core::aiconfig::VersionInfo>, String> {
    use cleaner_core::aiconfig::{latest_from_response, registry_url, VersionInfo};
    if !Settings::load(&settings_path(&app)).mcp_version_check {
        return Ok(Vec::new());
    }
    tauri::async_runtime::spawn_blocking(move || {
        let agent = ureq::Agent::config_builder()
            .timeout_global(Some(std::time::Duration::from_secs(15)))
            .build()
            .new_agent();
        let mut seen = std::collections::HashSet::new();
        let mut out = Vec::new();
        for identity in identities.into_iter().take(200) {
            if !seen.insert(identity.clone()) {
                continue;
            }
            let Some(url) = registry_url(&identity) else {
                continue;
            };
            let mut info = VersionInfo {
                identity: identity.clone(),
                ..Default::default()
            };
            match agent.get(&url).call() {
                Ok(mut resp) => match resp.body_mut().read_to_string() {
                    Ok(body) => info.latest = latest_from_response(&identity, &body),
                    Err(e) => info.error = Some(e.to_string()),
                },
                Err(e) => info.error = Some(e.to_string()),
            }
            out.push(info);
        }
        out
    })
    .await
    .map_err(|e| e.to_string())
}

/// Deep clean: the catalogue with cheap estimates.
#[tauri::command]
fn deep_tasks(lang: String) -> Vec<cleaner_core::system::DeepTask> {
    cleaner_core::system::tasks(&lang)
}

/// Deep clean: elevated analysis (one UAC prompt).
#[tauri::command]
async fn deep_analyze() -> Result<cleaner_core::system::DeepAnalysis, String> {
    tauri::async_runtime::spawn_blocking(|| {
        cleaner_core::system::analyze().map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Deep clean: run the ticked tasks (one UAC prompt).
#[tauri::command]
async fn deep_run(ids: Vec<String>) -> Result<cleaner_core::system::DeepSummary, String> {
    tauri::async_runtime::spawn_blocking(move || {
        cleaner_core::system::run(&ids).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn hibernate_on() -> Result<cleaner_core::system::DeepResult, String> {
    tauri::async_runtime::spawn_blocking(|| {
        cleaner_core::system::hibernate_on().map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Rule ids whose items are chat history that can be exported first.
#[tauri::command]
fn exportable() -> Vec<String> {
    export::EXPORTABLE_RULES
        .iter()
        .map(|s| s.to_string())
        .collect()
}

#[tauri::command]
async fn export_items(
    app: AppHandle,
    items: Vec<ScanItem>,
    dest_dir: PathBuf,
) -> Result<ExportResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let mut progress = progress_emitter(app, "export-progress");
        export::export(&items, &dest_dir, &mut progress).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

static BIG_SCAN_CANCEL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[derive(Clone, Serialize)]
struct BigScanProgress {
    seen: u64,
    bytes: u64,
    dir: String,
}

#[tauri::command]
fn drives() -> Vec<Drive> {
    bigfiles::drives()
}

/// Whole-drive big-file scan; emits `bigfiles-progress` while walking.
#[tauri::command]
async fn scan_big_files(
    app: AppHandle,
    roots: Vec<PathBuf>,
    min_bytes: u64,
    lang: String,
) -> Result<Vec<ScanItem>, String> {
    use std::sync::atomic::Ordering;
    let settings = Settings::load(&settings_path(&app));
    BIG_SCAN_CANCEL.store(false, Ordering::Relaxed);
    tauri::async_runtime::spawn_blocking(move || {
        let mut progress = |seen: u64, bytes: u64, dir: &std::path::Path| {
            let _ = app.emit(
                "bigfiles-progress",
                BigScanProgress {
                    seen,
                    bytes,
                    dir: dir.to_string_lossy().to_string(),
                },
            );
        };
        bigfiles::scan(
            &roots,
            min_bytes,
            &settings.excluded_paths,
            &lang,
            &BIG_SCAN_CANCEL,
            &mut progress,
        )
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn cancel_big_scan() {
    BIG_SCAN_CANCEL.store(true, std::sync::atomic::Ordering::Relaxed);
}

static OVERVIEW_CANCEL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn overview_path(app: &AppHandle) -> PathBuf {
    log_path(app).with_file_name("overview.json")
}

/// Walks one drive and splits its used space by top-level folder. The
/// result is cached next to the settings so the page opens instantly.
#[tauri::command]
async fn drive_overview(
    app: AppHandle,
    root: PathBuf,
) -> Result<cleaner_core::overview::Overview, String> {
    use std::sync::atomic::Ordering;
    OVERVIEW_CANCEL.store(false, Ordering::Relaxed);
    let (total, free) = bigfiles::drives()
        .into_iter()
        .find(|d| d.path == root)
        .map(|d| (d.total_bytes, d.free_bytes))
        .unwrap_or((0, 0));
    let cache = overview_path(&app);
    tauri::async_runtime::spawn_blocking(move || {
        let mut progress = |seen: u64, bytes: u64, dir: &std::path::Path| {
            let _ = app.emit(
                "overview-progress",
                BigScanProgress {
                    seen,
                    bytes,
                    dir: dir.to_string_lossy().to_string(),
                },
            );
        };
        let ov =
            cleaner_core::overview::overview(&root, total, free, &OVERVIEW_CANCEL, &mut progress);
        if !ov.cancelled {
            if let Ok(text) = serde_json::to_string(&ov) {
                let _ = std::fs::write(&cache, text);
            }
        }
        ov
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn cancel_overview() {
    OVERVIEW_CANCEL.store(true, std::sync::atomic::Ordering::Relaxed);
}

/// The last computed overview, if any.
#[tauri::command]
fn last_overview(app: AppHandle) -> Option<cleaner_core::overview::Overview> {
    std::fs::read_to_string(overview_path(&app))
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
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
        // Must be the first plugin: a second launch (autostart entry plus a
        // click on the shortcut) hands its args to the running instance and
        // exits instead of starting a second tray icon and scheduler.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            background::show_window(app);
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .setup(|app| {
            // The main window is built here rather than in tauri.conf.json so
            // the portable build can keep the WebView2 profile (cookies,
            // localStorage, cache) in its own data folder instead of
            // %LOCALAPPDATA%, where it would be shared with an installed copy.
            let mut wb =
                tauri::WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::default())
                    .title("Clearwisp")
                    .inner_size(1100.0, 740.0)
                    .min_inner_size(860.0, 560.0);
            if let Some(dir) = portable_dir() {
                wb = wb.data_directory(dir.join("webview"));
            }
            if let Some(icon) = app.default_window_icon() {
                wb = wb.icon(icon.clone())?;
            }
            wb.build()?;
            background::setup(app.handle())?;
            Ok(())
        })
        .on_window_event(background::on_window_event)
        .invoke_handler(tauri::generate_handler![
            scan,
            preview_item,
            cancel_scan,
            clean_items,
            clean_log,
            is_portable,
            growth_report,
            restore_status,
            restore_items,
            rules,
            get_settings,
            save_settings,
            set_lang,
            update_rules,
            relocatable,
            move_source,
            relocate_item,
            relocations,
            move_back,
            compactable,
            vhdx_files,
            compact_vhdx,
            unrecognized,
            suggest_rule_url,
            ai_inventory,
            ai_version_check,
            deep_tasks,
            deep_analyze,
            deep_run,
            hibernate_on,
            exportable,
            export_items,
            drives,
            scan_big_files,
            cancel_big_scan,
            drive_overview,
            cancel_overview,
            last_overview,
            find_duplicates,
            merge_duplicates
        ])
        .run(tauri::generate_context!())
        .expect("error while running ai-disk-cleaner");
}
