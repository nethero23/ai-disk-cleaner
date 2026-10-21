//! Tray icon, scheduled background scans, notifications and autostart.
//! Everything here reads `settings.json` fresh so a change in the settings
//! page takes effect without a restart.

use crate::{effective_rules, settings_path};
use cleaner_core::settings::Settings;
use cleaner_core::{scan_all, ScanOptions};
use std::time::Duration;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, WindowEvent};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_notification::NotificationExt;

pub const TRAY_ID: &str = "main";
const CHECK_EVERY: Duration = Duration::from_secs(60);

struct Labels {
    open: &'static str,
    scan: &'static str,
    quit: &'static str,
    found_title: &'static str,
    found_body: &'static str,
    clean_title: &'static str,
    clean_body: &'static str,
}

fn labels(lang: &str) -> Labels {
    if lang == "en" {
        Labels {
            open: "Open AI Disk Cleaner",
            scan: "Scan now",
            quit: "Quit",
            found_title: "Junk found",
            found_body: "{size} can be cleaned safely. Open AI Disk Cleaner to review.",
            clean_title: "Scan finished",
            clean_body: "Nothing worth cleaning right now.",
        }
    } else {
        Labels {
            open: "打开 AI Disk Cleaner",
            scan: "立即扫描",
            quit: "退出",
            found_title: "发现可清理的垃圾",
            found_body: "有 {size} 可以安全清理，打开 AI Disk Cleaner 查看。",
            clean_title: "扫描完成",
            clean_body: "目前没有值得清理的东西。",
        }
    }
}

pub fn show_window(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

fn build_menu(app: &AppHandle, lang: &str) -> tauri::Result<Menu<tauri::Wry>> {
    let l = labels(lang);
    let open = MenuItem::with_id(app, "open", l.open, true, None::<&str>)?;
    let scan = MenuItem::with_id(app, "scan", l.scan, true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", l.quit, true, None::<&str>)?;
    Menu::with_items(app, &[&open, &scan, &quit])
}

/// Swap the tray menu to another language (called when the UI switches).
pub fn relabel(app: &AppHandle, lang: &str) {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        if let Ok(menu) = build_menu(app, lang) {
            let _ = tray.set_menu(Some(menu));
        }
    }
}

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let settings = Settings::load(&settings_path(app));
    let menu = build_menu(app, &settings.lang)?;
    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("AI Disk Cleaner")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => show_window(app),
            "scan" => {
                let app = app.clone();
                std::thread::spawn(move || run_background_scan(&app, true));
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_window(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;

    // `--minimized` is what the autostart entry passes: start in the tray.
    if std::env::args().any(|a| a == "--minimized") {
        if let Some(w) = app.get_webview_window("main") {
            let _ = w.hide();
        }
    }

    // Keep the autostart registration in line with the setting; the user may
    // have removed it from Task Manager or the setting may be newer.
    apply_autostart(app, settings.autostart);

    let handle = app.clone();
    std::thread::spawn(move || scheduler(handle));
    Ok(())
}

pub fn apply_autostart(app: &AppHandle, on: bool) {
    let launcher = app.autolaunch();
    let _ = if on {
        launcher.enable()
    } else {
        launcher.disable()
    };
}

/// Hide instead of close when the user asked for a tray app.
pub fn on_window_event(window: &tauri::Window, event: &WindowEvent) {
    if let WindowEvent::CloseRequested { api, .. } = event {
        let settings = Settings::load(&settings_path(window.app_handle()));
        if settings.close_to_tray {
            api.prevent_close();
            let _ = window.hide();
        }
    }
}

fn scheduler(app: AppHandle) {
    loop {
        std::thread::sleep(CHECK_EVERY);
        let settings = Settings::load(&settings_path(&app));
        if settings.scheduled_scan_hours == 0 {
            continue;
        }
        let due = match settings.last_background_scan {
            None => true,
            Some(last) => {
                chrono::Utc::now() - last
                    >= chrono::Duration::hours(i64::from(settings.scheduled_scan_hours))
            }
        };
        if due {
            run_background_scan(&app, false);
        }
    }
}

/// Scan with the user's rules, remember when, notify, and hand the report to
/// the window so it shows the result when opened. `forced` (tray "scan now")
/// always notifies; a scheduled scan only when above the threshold.
pub fn run_background_scan(app: &AppHandle, forced: bool) {
    let (rules, _, settings) = effective_rules(app);
    let opts = ScanOptions {
        lang: settings.lang.clone(),
        global_min_bytes: settings.min_item_bytes,
        excluded: settings.excluded_paths.clone(),
        ..ScanOptions::default()
    };
    let report = scan_all(&rules, &opts);

    let mut latest = Settings::load(&settings_path(app));
    latest.last_background_scan = Some(chrono::Utc::now());
    let _ = latest.save(&settings_path(app));

    let l = labels(&settings.lang);
    // Only what is safe to delete counts: the total also holds Review items
    // such as a 30 GB Docker disk, which would make every scan "worth it".
    let size = cleaner_core::model::human_size(report.safe_bytes);
    let worth = report.safe_bytes > 0 && report.safe_bytes >= settings.notify_min_bytes;
    if worth || forced {
        let (title, body) = if worth {
            (l.found_title, l.found_body.replace("{size}", &size))
        } else {
            (l.clean_title, l.clean_body.to_string())
        };
        let _ = app.notification().builder().title(title).body(body).show();
    }
    let _ = app.emit("background-scan", &report);
}
