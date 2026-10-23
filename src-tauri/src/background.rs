//! Tray icon, scheduled background scans, notifications and autostart.
//! Everything here reads `settings.json` fresh so a change in the settings
//! page takes effect without a restart.

use crate::{effective_rules, growth_path, log_path, settings_path};
use cleaner_core::clean;
use cleaner_core::settings::Settings;
use cleaner_core::SafetyLevel;
use cleaner_core::{scan_all, ScanOptions};
use serde::Serialize;
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
    auto_title: &'static str,
    auto_body: &'static str,
    growth_title: &'static str,
    growth_body: &'static str,
}

/// Tray menu and notification text. Same language codes as the UI (src/i18n.ts);
/// anything unknown falls back to Chinese, the app's default.
fn labels(lang: &str) -> Labels {
    let [open, scan, quit, found_title, found_body, clean_title, clean_body, auto_title, auto_body, growth_title, growth_body] = match lang {
        "en" => [
            "Open Clearwisp",
            "Scan now",
            "Quit",
            "Junk found",
            "{size} can be cleaned safely. Open Clearwisp to review.",
            "Scan finished",
            "Nothing worth cleaning right now.",
            "Cleaned automatically",
            "{n} items ({size}) moved to the Recycle Bin.",
            "Something is growing",
            "{tool} grew by {size} this week. Open Clearwisp to see what.",
        ],
        "zh-TW" => [
            "開啟 Clearwisp",
            "立即掃描",
            "結束",
            "發現可清理的垃圾",
            "有 {size} 可以安全清理，開啟 Clearwisp 查看。",
            "掃描完成",
            "目前沒有值得清理的東西。",
            "已自動清理",
            "{n} 項（{size}）已移入資源回收筒。",
            "有東西在長",
            "{tool} 這週漲了 {size}，開啟 Clearwisp 看看是什麼。",
        ],
        "ja" => [
            "Clearwisp を開く",
            "今すぐスキャン",
            "終了",
            "クリーンできるごみが見つかりました",
            "{size} を安全にクリーンできます。Clearwisp を開いて確認してください。",
            "スキャン完了",
            "今のところクリーンする価値のあるものはありません。",
            "自動でクリーンしました",
            "{n} 件（{size}）をごみ箱へ移動しました。",
            "増えているものがあります",
            "{tool} が今週 {size} 増えました。Clearwisp で中身を確認してください。",
        ],
        "ko" => [
            "Clearwisp 열기",
            "지금 검사",
            "종료",
            "정리할 항목을 찾았습니다",
            "{size}를 안전하게 정리할 수 있습니다. Clearwisp를 열어 확인하세요.",
            "검사 완료",
            "지금은 정리할 만한 것이 없습니다.",
            "자동으로 정리했습니다",
            "{n}개 항목({size})을 휴지통으로 옮겼습니다.",
            "커지고 있는 것이 있습니다",
            "{tool}이(가) 이번 주 {size} 늘었습니다. Clearwisp에서 확인하세요.",
        ],
        "de" => [
            "Clearwisp öffnen",
            "Jetzt scannen",
            "Beenden",
            "Datenmüll gefunden",
            "{size} können sicher bereinigt werden. Öffne Clearwisp zum Prüfen.",
            "Scan abgeschlossen",
            "Gerade gibt es nichts, was sich zu bereinigen lohnt.",
            "Automatisch bereinigt",
            "{n} Einträge ({size}) in den Papierkorb verschoben.",
            "Etwas wächst",
            "{tool} ist diese Woche um {size} gewachsen. Öffne Clearwisp, um zu sehen, was.",
        ],
        "fr" => [
            "Ouvrir Clearwisp",
            "Analyser maintenant",
            "Quitter",
            "Fichiers inutiles trouvés",
            "{size} peuvent être nettoyés sans risque. Ouvrez Clearwisp pour vérifier.",
            "Analyse terminée",
            "Rien qui vaille la peine d'être nettoyé pour l'instant.",
            "Nettoyé automatiquement",
            "{n} éléments ({size}) déplacés vers la Corbeille.",
            "Quelque chose grossit",
            "{tool} a grossi de {size} cette semaine. Ouvrez Clearwisp pour voir quoi.",
        ],
        "es" => [
            "Abrir Clearwisp",
            "Analizar ahora",
            "Salir",
            "Se encontró basura",
            "Se pueden limpiar {size} de forma segura. Abre Clearwisp para revisarlo.",
            "Análisis terminado",
            "Ahora mismo no hay nada que valga la pena limpiar.",
            "Limpieza automática",
            "{n} elementos ({size}) movidos a la Papelera de reciclaje.",
            "Algo está creciendo",
            "{tool} creció {size} esta semana. Abre Clearwisp para ver qué es.",
        ],
        "pt" => [
            "Abrir o Clearwisp",
            "Verificar agora",
            "Sair",
            "Lixo encontrado",
            "{size} podem ser limpos com segurança. Abra o Clearwisp para revisar.",
            "Verificação concluída",
            "Nada que valha a pena limpar agora.",
            "Limpo automaticamente",
            "{n} itens ({size}) movidos para a Lixeira.",
            "Algo está crescendo",
            "{tool} cresceu {size} esta semana. Abra o Clearwisp para ver o quê.",
        ],
        "ru" => [
            "Открыть Clearwisp",
            "Сканировать сейчас",
            "Выход",
            "Найден мусор",
            "Можно безопасно очистить {size}. Откройте Clearwisp, чтобы посмотреть.",
            "Сканирование завершено",
            "Сейчас нечего очищать.",
            "Очищено автоматически",
            "Элементы ({n}, {size}) перемещены в корзину.",
            "Что-то растёт",
            "{tool} вырос на {size} за эту неделю. Откройте Clearwisp, чтобы узнать, что именно.",
        ],
        "ar" => [
            "فتح Clearwisp",
            "افحص الآن",
            "خروج",
            "تم العثور على ملفات غير ضرورية",
            "يمكن تنظيف {size} بأمان. افتح Clearwisp للمراجعة.",
            "اكتمل الفحص",
            "لا يوجد ما يستحق التنظيف الآن.",
            "تم التنظيف تلقائيًا",
            "نُقل {n} عنصر ({size}) إلى سلة المحذوفات.",
            "هناك ما يكبر",
            "كبرت {tool} بمقدار {size} هذا الأسبوع. افتح Clearwisp لترى ما هو.",
        ],
        "hi" => [
            "Clearwisp खोलें",
            "अभी स्कैन करें",
            "बाहर निकलें",
            "कचरा मिला",
            "{size} सुरक्षित रूप से साफ़ किया जा सकता है। देखने के लिए Clearwisp खोलें।",
            "स्कैन पूरा हुआ",
            "अभी साफ़ करने लायक कुछ नहीं है।",
            "अपने-आप साफ़ किया गया",
            "{n} आइटम ({size}) रीसायकल बिन में भेजे गए।",
            "कुछ बढ़ रहा है",
            "{tool} इस हफ़्ते {size} बढ़ा। क्या है, देखने के लिए Clearwisp खोलें।",
        ],
        _ => [
            "打开 Clearwisp",
            "立即扫描",
            "退出",
            "发现可清理的垃圾",
            "有 {size} 可以安全清理，打开 Clearwisp 查看。",
            "扫描完成",
            "目前没有值得清理的东西。",
            "已自动清理",
            "{n} 项（{size}）已移入回收站。",
            "有东西在长",
            "{tool} 这周涨了 {size}，打开 Clearwisp 看看是什么。",
        ],
    };
    Labels {
        open,
        scan,
        quit,
        found_title,
        found_body,
        clean_title,
        clean_body,
        auto_title,
        auto_body,
        growth_title,
        growth_body,
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
        .tooltip("Clearwisp")
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

    // Installed copies synchronize their setting on startup. A portable copy
    // has separate settings and must not overwrite the installed registration.
    sync_startup_autostart(crate::portable_dir().is_some(), settings.autostart, |on| {
        apply_autostart(app, on)
    });

    let handle = app.clone();
    std::thread::spawn(move || scheduler(handle));
    Ok(())
}

fn sync_startup_autostart(portable: bool, on: bool, apply: impl FnOnce(bool)) {
    if !portable {
        apply(on);
    }
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
#[derive(Serialize, Clone)]
struct BackgroundClean {
    cleaned: usize,
    freed_bytes: u64,
}

/// One notification for the fastest-growing tool, when it crossed the threshold.
fn notify_growth(app: &AppHandle, settings: &Settings) {
    if settings.growth_notify_bytes == 0 {
        return;
    }
    let snaps = cleaner_core::growth::read(&growth_path(app));
    let Some(top) = cleaner_core::growth::growth(&snaps, 7).into_iter().next() else {
        return;
    };
    if top.delta_bytes < settings.growth_notify_bytes as i64 {
        return;
    }
    let l = labels(&settings.lang);
    let body = l.growth_body.replace("{tool}", &top.tool).replace(
        "{size}",
        &cleaner_core::model::human_size(top.delta_bytes as u64),
    );
    let _ = app
        .notification()
        .builder()
        .title(l.growth_title)
        .body(body)
        .show();
}

pub fn run_background_scan(app: &AppHandle, forced: bool) {
    let (rules, _, settings) = effective_rules(app);
    let opts = ScanOptions {
        // The core only speaks zh and en; the UI may have stored e.g. "ja".
        lang: if settings.lang.starts_with("zh") {
            "zh"
        } else {
            "en"
        }
        .into(),
        global_min_bytes: settings.min_item_bytes,
        excluded: settings.excluded_paths.clone(),
        extra_roots: settings.scan_drives.clone(),
        ..ScanOptions::default()
    };
    let mut report = scan_all(&rules, &opts);
    cleaner_core::growth::record(&growth_path(app), &report, &settings.scan_drives);
    notify_growth(app, &settings);

    // Unattended clean: only Safe items whose owning tool is not running,
    // only for scheduled scans (a tray "scan now" is a request to look).
    if settings.auto_clean_safe && !forced {
        let (safe, rest): (Vec<_>, Vec<_>) = report
            .items
            .drain(..)
            .partition(|i| i.safety == SafetyLevel::Safe && i.in_use_by.is_none() && i.auto_clean);
        let mut kept = rest;
        if !safe.is_empty() {
            let summary = clean::clean(&safe, &log_path(app));
            let failed: Vec<_> = summary
                .records
                .iter()
                .filter(|r| !r.ok)
                .map(|r| r.path.clone())
                .collect();
            let cleaned = summary.records.iter().filter(|r| r.ok).count();
            let l = labels(&settings.lang);
            if cleaned > 0 {
                let body = l.auto_body.replace("{n}", &cleaned.to_string()).replace(
                    "{size}",
                    &cleaner_core::model::human_size(summary.freed_bytes),
                );
                let _ = app
                    .notification()
                    .builder()
                    .title(l.auto_title)
                    .body(body)
                    .show();
            }
            let _ = app.emit(
                "background-clean",
                BackgroundClean {
                    cleaned,
                    freed_bytes: summary.freed_bytes,
                },
            );
            kept.extend(safe.into_iter().filter(|i| failed.contains(&i.path)));
        }
        // Rebuild from scratch: `drain` left the totals behind, so pushing
        // onto the old report would count every remaining item twice.
        let mut left = cleaner_core::ScanReport {
            started_at: report.started_at,
            finished_at: report.finished_at,
            unreadable: std::mem::take(&mut report.unreadable),
            timings: std::mem::take(&mut report.timings),
            cancelled: report.cancelled,
            ..Default::default()
        };
        for item in kept {
            left.push(item);
        }
        left.sort_by_size();
        report = left;
    }

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

#[cfg(test)]
mod startup_tests {
    use super::sync_startup_autostart;

    #[test]
    fn portable_launch_never_changes_autostart_registration() {
        for on in [false, true] {
            sync_startup_autostart(true, on, |_| {
                panic!("portable launch must not change an installed app's registration")
            });
        }
    }

    #[test]
    fn installed_launch_keeps_the_existing_sync_policy() {
        for on in [false, true] {
            let mut applied = None;
            sync_startup_autostart(false, on, |value| applied = Some(value));
            assert_eq!(applied, Some(on));
        }
    }
}
