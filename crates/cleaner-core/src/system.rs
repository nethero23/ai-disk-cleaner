//! Windows deep clean: the things Disk Cleanup used to do and Dism++ did
//! better, all needing administrator rights.
//!
//! Everything runs through one elevated PowerShell script per call, so the
//! user sees a single UAC prompt however many tasks are ticked. The script
//! writes a JSON result file that we parse back. Nothing here ever runs
//! `Dism /ResetBase` (it makes updates un-uninstallable) and nothing
//! deletes the newest restore point.

use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// One deep-clean task the user can tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepTask {
    pub id: String,
    pub title: String,
    pub body: String,
    /// What it will cost the user, in plain words.
    pub consequence: String,
    /// Bytes we expect to get back, when we can tell without admin rights.
    pub estimate_bytes: Option<u64>,
    /// True when the estimate needs the elevated analysis pass.
    pub needs_analysis: bool,
    /// False when the task does not apply (e.g. hibernation already off).
    pub applicable: bool,
}

/// Numbers the elevated analysis pass brings back.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeepAnalysis {
    pub winsxs_actual_bytes: Option<u64>,
    pub winsxs_reclaimable_bytes: Option<u64>,
    pub winsxs_cleanup_recommended: Option<bool>,
    pub shadow_used_bytes: Option<u64>,
    pub shadow_count: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepResult {
    pub id: String,
    pub ok: bool,
    pub message: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeepSummary {
    pub results: Vec<DeepResult>,
    /// Free space on the system drive after minus before (can be 0 when a
    /// task frees space only after a reboot, as WinSxS cleanup often does).
    pub freed_bytes: u64,
}

pub const WINSXS: &str = "winsxs";
pub const RESTORE_POINTS: &str = "restore_points";
pub const HIBERNATION: &str = "hibernation";
pub const DELIVERY_OPT: &str = "delivery_opt";
pub const WU_CACHE: &str = "wu_cache";
pub const WINDOWS_OLD: &str = "windows_old";

pub const TASK_IDS: &[&str] = &[
    WINSXS,
    RESTORE_POINTS,
    HIBERNATION,
    DELIVERY_OPT,
    WU_CACHE,
    WINDOWS_OLD,
];

fn system_root() -> PathBuf {
    std::env::var_os("SYSTEMROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Windows"))
}

fn system_drive() -> PathBuf {
    std::env::var_os("SYSTEMDRIVE")
        .map(|d| PathBuf::from(format!("{}\\", d.to_string_lossy())))
        .unwrap_or_else(|| PathBuf::from(r"C:\"))
}

pub(crate) fn is_windows_old(path: &Path) -> bool {
    let normalize = |value: &Path| {
        let mut normalized = value
            .to_string_lossy()
            .replace('/', "\\")
            .trim_start_matches(r"\\?\")
            .trim_end_matches('\\')
            .to_ascii_lowercase();
        // ${SYSTEMDRIVE} ends in a separator and rule templates add another.
        while normalized.contains(r"\\") {
            normalized = normalized.replace(r"\\", r"\");
        }
        normalized
    };
    normalize(path) == normalize(&system_drive().join("Windows.old"))
}

fn dir_bytes(p: &Path) -> Option<u64> {
    if !p.is_dir() {
        return None;
    }
    Some(crate::scan::tree_stats(p).bytes)
}

/// The catalogue with cheap (non-admin) estimates filled in.
pub fn tasks(lang: &str) -> Vec<DeepTask> {
    let en = lang == "en";
    let root = system_root();
    let hiber = system_drive().join("hiberfil.sys");
    let hiber_bytes = std::fs::metadata(&hiber).ok().map(|m| m.len());
    let do_cache = dir_bytes(
        &root
            .join("SoftwareDistribution")
            .join("DeliveryOptimization"),
    );
    let wu_cache = dir_bytes(&root.join("SoftwareDistribution").join("Download"));
    let t = |id: &str,
             zh: (&str, &str, &str),
             en_: (&str, &str, &str),
             est: Option<u64>,
             needs_analysis: bool,
             applicable: bool| {
        let (title, body, consequence) = if en { en_ } else { zh };
        DeepTask {
            id: id.into(),
            title: title.into(),
            body: body.into(),
            consequence: consequence.into(),
            estimate_bytes: est,
            needs_analysis,
            applicable,
        }
    };
    vec![
        t(
            WINDOWS_OLD,
            (
                "Windows.old（上一个系统版本）",
                "由 Windows 系统维护工具移除当前系统的升级回退文件；单独从扫描列表进入时，不会处理其他已勾选项目。",
                "永久删除，不进入回收站，不能在清理历史中还原，也无法回退到升级前的 Windows。请先确认旧系统中没有需要保留的个人文件。",
            ),
            (
                "Windows.old (previous Windows)",
                "Uses Windows servicing to remove this system's upgrade rollback files. Entering from scan results leaves other selected items untouched.",
                "Permanent: not sent to the Recycle Bin or restorable from history. You cannot roll back Windows. First check for personal files you need in the old installation.",
            ),
            None,
            false,
            system_drive().join("Windows.old").is_dir(),
        ),
        t(
            WINSXS,
            (
                "WinSxS 组件存储清理",
                "删除已被新版本取代的系统组件备份（DISM /StartComponentCleanup）。和「磁盘清理」里的「Windows 更新清理」一样，是微软支持的做法。",
                "清理后无法卸载已安装的旧更新；部分空间在下次重启后才释放。不会执行 /ResetBase。",
            ),
            (
                "WinSxS component store cleanup",
                "Removes superseded system component backups (DISM /StartComponentCleanup). The same thing Disk Cleanup's \"Windows Update Cleanup\" does, supported by Microsoft.",
                "Older updates can no longer be uninstalled; some space is freed only after the next reboot. Never runs /ResetBase.",
            ),
            None,
            true,
            true,
        ),
        t(
            RESTORE_POINTS,
            (
                "旧系统还原点",
                "删除除最新一个以外的所有系统还原点（卷影副本）。",
                "只能回滚到最新的还原点。",
            ),
            (
                "Old system restore points",
                "Deletes every system restore point (shadow copy) except the newest one.",
                "You can only roll back to the newest restore point.",
            ),
            None,
            true,
            true,
        ),
        t(
            HIBERNATION,
            (
                "关闭休眠（删除 hiberfil.sys）",
                "休眠文件和内存一样大，关闭休眠后 Windows 会删除它。台式机或从不用休眠的笔记本可以关。",
                "不能再休眠，「快速启动」也会关闭（开机慢几秒）。随时可以在这里重新打开。",
            ),
            (
                "Turn hibernation off (deletes hiberfil.sys)",
                "The hibernation file is as big as your RAM; turning hibernation off makes Windows delete it. Fine for desktops and laptops that never hibernate.",
                "No more hibernation, and Fast Startup is turned off too (boot a few seconds slower). Can be turned back on here any time.",
            ),
            hiber_bytes,
            false,
            hiber_bytes.is_some(),
        ),
        t(
            DELIVERY_OPT,
            (
                "传递优化缓存",
                "Windows 更新为了分享给局域网其他电脑而保留的下载缓存。",
                "没有影响，下次更新会重新下载需要的部分。",
            ),
            (
                "Delivery Optimization cache",
                "Downloads Windows Update keeps so it can share them with other PCs on your network.",
                "No effect; the next update re-downloads what it needs.",
            ),
            do_cache,
            false,
            do_cache.is_some(),
        ),
        t(
            WU_CACHE,
            (
                "Windows 更新下载缓存",
                "已安装更新的安装包残留（SoftwareDistribution\\Download）。会先停止、后重启 Windows Update 服务。",
                "正在下载的更新要重新开始。",
            ),
            (
                "Windows Update download cache",
                "Leftover installer packages of updates already installed (SoftwareDistribution\\Download). Stops and restarts the Windows Update service around the delete.",
                "An update that is mid-download starts over.",
            ),
            wu_cache,
            false,
            wu_cache.is_some(),
        ),
    ]
}

/// The script for the elevated analysis pass. Writes JSON to `out`.
pub fn analyze_script(out: &Path) -> String {
    format!(
        r#"$ErrorActionPreference = 'Continue'
$r = @{{}}
try {{ $r.dism = (dism /online /cleanup-image /analyzecomponentstore /english) -join "`n" }} catch {{ $r.dism = "" }}
try {{ $r.shadow_used = (Get-CimInstance Win32_ShadowStorage | Measure-Object -Property UsedSpace -Sum).Sum }} catch {{ $r.shadow_used = $null }}
try {{ $r.shadow_count = @(Get-CimInstance Win32_ShadowCopy).Count }} catch {{ $r.shadow_count = $null }}
$r | ConvertTo-Json -Compress | Out-File -Encoding utf8 -FilePath "{}"
"#,
        out.display()
    )
}

/// One elevated script that runs every ticked task in order and records an
/// exit code per task. Unknown ids are ignored.
pub fn run_script(ids: &[String], out: &Path) -> String {
    let mut s = String::from("$ErrorActionPreference = 'Continue'\n$r = @{}\n");
    for id in ids {
        let body = match id.as_str() {
            WINSXS => "dism /online /cleanup-image /startcomponentcleanup /english | Out-Null; $r.winsxs = $LASTEXITCODE",
            RESTORE_POINTS => "Get-CimInstance Win32_ShadowCopy | Sort-Object InstallDate | Select-Object -SkipLast 1 | ForEach-Object { Remove-CimInstance $_ }; $r.restore_points = 0",
            HIBERNATION => "powercfg /hibernate off | Out-Null; $r.hibernation = $LASTEXITCODE",
            DELIVERY_OPT => "Delete-DeliveryOptimizationCache -Force; $r.delivery_opt = 0",
            WU_CACHE => "Stop-Service wuauserv -Force; Remove-Item \"$env:SystemRoot\\SoftwareDistribution\\Download\\*\" -Recurse -Force -ErrorAction SilentlyContinue; Start-Service wuauserv; $r.wu_cache = 0",
            WINDOWS_OLD => r#"
if ($env:SystemDrive -notmatch '^[A-Za-z]:$' -or [IO.Path]::GetPathRoot($env:SystemRoot).TrimEnd('\') -ine $env:SystemDrive) {
    throw 'Cannot verify the current Windows system drive; nothing was cleaned.'
}
$old = Join-Path ($env:SystemDrive + '\') 'Windows.old'
if (Test-Path -LiteralPath $old -ErrorAction Stop) {
    $entry = Get-Item -LiteralPath $old -Force -ErrorAction Stop
    if ($entry.Attributes -band [IO.FileAttributes]::ReparsePoint) {
        throw 'Windows.old is a reparse point; nothing was cleaned.'
    }
    if (-not ($entry.Attributes -band [IO.FileAttributes]::Directory)) {
        throw 'Windows.old is not a directory; nothing was cleaned.'
    }
    $process = Start-Process -FilePath (Join-Path $env:SystemRoot 'System32\dism.exe') -ArgumentList @('/Online', '/Remove-OSUninstall', '/NoRestart', '/English') -WindowStyle Hidden -Wait -PassThru -ErrorAction Stop
    if ($process.ExitCode -notin @(0, 3010)) {
        throw "Windows system cleanup failed (DISM exit code $($process.ExitCode)). Use Windows Storage > Temporary files > Previous Windows installation(s)."
    }
    if (Test-Path -LiteralPath $old -ErrorAction Stop) {
        throw 'Windows.old still exists; cleanup is not confirmed. Windows may require a manual restart or Storage > Temporary files > Previous Windows installation(s). No restart was requested.'
    }
}
$r.windows_old = 0
"#,
            _ => continue,
        };
        s.push_str(&format!(
            "try {{ {body} }} catch {{ $r.{id} = -1; $r.{id}_error = \"$_\" }}\n"
        ));
    }
    s.push_str(&format!(
        "$r | ConvertTo-Json -Compress | Out-File -Encoding utf8 -FilePath \"{}\"\n",
        out.display()
    ));
    s
}

/// Turns hibernation back on (the only task with an "undo").
pub fn hibernate_on_script(out: &Path) -> String {
    format!(
        "$r = @{{}}\npowercfg /hibernate on | Out-Null; $r.hibernation = $LASTEXITCODE\n$r | ConvertTo-Json -Compress | Out-File -Encoding utf8 -FilePath \"{}\"\n",
        out.display()
    )
}

/// "8.97 GB" / "512 MB" / "0 bytes" → bytes.
fn parse_size(s: &str) -> Option<u64> {
    let s = s.trim();
    let (num, unit) = s.split_once(' ')?;
    let v: f64 = num.trim().replace(',', "").parse().ok()?;
    let mult = match unit.trim().to_ascii_lowercase().as_str() {
        "bytes" | "byte" | "b" => 1.0,
        "kb" => 1024.0,
        "mb" => 1024.0 * 1024.0,
        "gb" => 1024.0 * 1024.0 * 1024.0,
        "tb" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => return None,
    };
    Some((v * mult) as u64)
}

/// Pulls the numbers out of `Dism /AnalyzeComponentStore /English` output.
pub fn parse_dism_analysis(text: &str) -> DeepAnalysis {
    let mut a = DeepAnalysis::default();
    for line in text.lines() {
        let Some((k, v)) = line.split_once(':') else {
            continue;
        };
        let k = k.trim();
        let v = v.trim();
        if k == "Actual Size of Component Store" {
            a.winsxs_actual_bytes = parse_size(v);
        } else if k == "Backups and Disabled Features" {
            a.winsxs_reclaimable_bytes = parse_size(v);
        } else if k == "Component Store Cleanup Recommended" {
            a.winsxs_cleanup_recommended = Some(v.eq_ignore_ascii_case("yes"));
        }
    }
    a
}

/// Parses the JSON the analysis script wrote.
pub fn parse_analysis(json: &str) -> DeepAnalysis {
    let v: serde_json::Value =
        serde_json::from_str(json.trim_start_matches('\u{feff}')).unwrap_or_default();
    let mut a = parse_dism_analysis(v.get("dism").and_then(|d| d.as_str()).unwrap_or(""));
    a.shadow_used_bytes = v
        .get("shadow_used")
        .and_then(|x| x.as_f64())
        .map(|x| x as u64);
    a.shadow_count = v.get("shadow_count").and_then(|x| x.as_u64());
    a
}

/// Parses the JSON the run script wrote into one result per requested id.
pub fn parse_run(ids: &[String], json: &str) -> Vec<DeepResult> {
    let v: HashMap<String, serde_json::Value> =
        serde_json::from_str(json.trim_start_matches('\u{feff}')).unwrap_or_default();
    ids.iter()
        .map(|id| {
            let code = v.get(id).and_then(|c| c.as_i64());
            let err = v
                .get(&format!("{id}_error"))
                .and_then(|e| e.as_str())
                .unwrap_or("")
                .to_string();
            match code {
                Some(0) => DeepResult {
                    id: id.clone(),
                    ok: true,
                    message: String::new(),
                },
                Some(c) => DeepResult {
                    id: id.clone(),
                    ok: false,
                    message: if err.is_empty() {
                        format!("exit code {c}")
                    } else {
                        err
                    },
                },
                None => DeepResult {
                    id: id.clone(),
                    ok: false,
                    message: "did not run (the UAC prompt may have been declined)".into(),
                },
            }
        })
        .collect()
}

/// Runs `script` as administrator (one UAC prompt) and returns what it
/// wrote to `out`. Windows only.
#[cfg(windows)]
pub fn run_elevated(script: &str, out: &Path) -> Result<String> {
    use std::process::Command;
    let ps1 = out.with_extension("ps1");
    let io = |source| Error::Io {
        path: ps1.clone(),
        source,
    };
    let _ = std::fs::remove_file(out);
    // Windows PowerShell 5.1 needs a BOM for non-ASCII user/temp paths.
    let bytes = [b"\xef\xbb\xbf".as_slice(), script.as_bytes()].concat();
    std::fs::write(&ps1, bytes).map_err(io)?;
    let launcher = format!(
        "$p = Start-Process powershell -ArgumentList '-NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File \"{}\"' -Verb RunAs -Wait -PassThru; exit $p.ExitCode",
        ps1.display()
    );
    let status = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &launcher])
        .status()
        .map_err(io)?;
    let _ = std::fs::remove_file(&ps1);
    let text = std::fs::read_to_string(out).unwrap_or_default();
    let _ = std::fs::remove_file(out);
    if text.is_empty() && !status.success() {
        return Err(Error::Refused(
            ps1,
            "elevation failed or the UAC prompt was declined".into(),
        ));
    }
    Ok(text)
}

#[cfg(not(windows))]
pub fn run_elevated(_script: &str, _out: &Path) -> Result<String> {
    Err(Error::Refused(
        PathBuf::from("deep clean"),
        "only works on Windows".into(),
    ))
}

fn out_file() -> PathBuf {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let sequence = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "aidc-deepclean-{}-{}-{sequence}.json",
        std::process::id(),
        chrono::Utc::now().timestamp_micros()
    ))
}

/// Elevated analysis: WinSxS numbers and shadow-copy usage.
pub fn analyze() -> Result<DeepAnalysis> {
    let out = out_file();
    let json = run_elevated(&analyze_script(&out), &out)?;
    Ok(parse_analysis(&json))
}

fn system_free() -> u64 {
    crate::bigfiles::drives()
        .into_iter()
        .find(|d| d.system)
        .map(|d| d.free_bytes)
        .unwrap_or(0)
}

/// Runs the ticked tasks under one UAC prompt.
pub fn run(ids: &[String]) -> Result<DeepSummary> {
    let ids: Vec<String> = ids
        .iter()
        .filter(|i| TASK_IDS.contains(&i.as_str()))
        .cloned()
        .collect();
    if ids.is_empty() {
        return Ok(DeepSummary::default());
    }
    let before = system_free();
    let out = out_file();
    let json = run_elevated(&run_script(&ids, &out), &out)?;
    let after = system_free();
    Ok(DeepSummary {
        results: parse_run(&ids, &json),
        freed_bytes: after.saturating_sub(before),
    })
}

/// Turns hibernation back on.
pub fn hibernate_on() -> Result<DeepResult> {
    let out = out_file();
    let json = run_elevated(&hibernate_on_script(&out), &out)?;
    Ok(parse_run(&[HIBERNATION.to_string()], &json).remove(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_old_uses_only_the_native_online_os_cleanup() {
        let script = run_script(&["windows_old".into()], Path::new(r"C:\fixture\out.json"));
        assert!(script.contains("/Remove-OSUninstall"));
        assert!(script.contains("dism.exe"));
        assert!(script.contains("/NoRestart"));
        assert!(script.contains("Windows.old still exists"));
        assert!(script.contains("ReparsePoint"));
        assert!(!script.contains("/sagerun:"));
        assert!(!script.contains("/Initiate-OSUninstall"));
        assert!(!script.contains("Remove-Item "));
        assert!(!script.contains("takeown"));
        assert!(!script.contains("Win32_ShadowCopy"));
        assert!(!script.contains("startcomponentcleanup"));
        assert!(!script.contains("/AUTOCLEAN"));
    }

    #[test]
    fn system_cleanup_output_paths_do_not_collide() {
        assert_ne!(out_file(), out_file());
    }

    #[test]
    fn windows_old_path_matches_only_the_system_drive_root() {
        let old = system_drive().join("Windows.old");
        assert!(is_windows_old(&old));
        assert!(!is_windows_old(&old.join("Users")));
        assert!(!is_windows_old(
            &system_drive().join("fixture").join("Windows.old")
        ));
        let scanned = crate::paths::PathContext::from_env()
            .expand("${SYSTEMDRIVE}/Windows.old")
            .unwrap();
        assert!(is_windows_old(&scanned), "{}", scanned.display());
        assert!(!is_windows_old(Path::new(r"\\C:\Windows.old")));
    }

    #[test]
    fn windows_old_has_a_separate_irreversible_task_in_both_languages() {
        for lang in ["zh", "en"] {
            let task = tasks(lang)
                .into_iter()
                .find(|task| task.id == "windows_old");
            let task = task.expect("Windows.old must not use ordinary recycling");
            assert!(task.title.contains("Windows.old"));
            assert!(task.consequence.contains(if lang == "zh" {
                "回收站"
            } else {
                "Recycle Bin"
            }));
        }
    }

    #[test]
    fn dism_output_is_parsed() {
        let text = "Deployment Image Servicing and Management tool\n\n[==========================100.0%==========================]\n\nComponent Store (WinSxS) information:\n\nWindows Explorer Reported Size of Component Store : 9.15 GB\n\nActual Size of Component Store : 8.97 GB\n\n    Shared with Windows : 5.41 GB\n    Backups and Disabled Features : 3.19 GB\n    Cache and Temporary Data :  0 bytes\n\nDate of Last Cleanup : 2024-01-01 10:00:00\n\nNumber of Reclaimable Packages : 3\nComponent Store Cleanup Recommended : Yes\n";
        let a = parse_dism_analysis(text);
        assert_eq!(
            a.winsxs_actual_bytes,
            Some((8.97 * 1024.0 * 1024.0 * 1024.0) as u64)
        );
        assert_eq!(
            a.winsxs_reclaimable_bytes,
            Some((3.19 * 1024.0 * 1024.0 * 1024.0) as u64)
        );
        assert_eq!(a.winsxs_cleanup_recommended, Some(true));
        assert_eq!(parse_size("0 bytes"), Some(0));
        assert_eq!(parse_size("1,024 MB"), Some(1024 * 1024 * 1024));
    }

    #[test]
    fn analysis_json_round_trips_with_bom() {
        let json = "\u{feff}{\"dism\":\"Actual Size of Component Store : 2 GB\\nComponent Store Cleanup Recommended : No\",\"shadow_used\":12345.0,\"shadow_count\":3}";
        let a = parse_analysis(json);
        assert_eq!(a.winsxs_actual_bytes, Some(2 * 1024 * 1024 * 1024));
        assert_eq!(a.winsxs_cleanup_recommended, Some(false));
        assert_eq!(a.shadow_used_bytes, Some(12345));
        assert_eq!(a.shadow_count, Some(3));
    }

    #[test]
    fn run_script_only_contains_requested_known_tasks() {
        let ids = vec![
            HIBERNATION.to_string(),
            "bogus".to_string(),
            WINSXS.to_string(),
        ];
        let s = run_script(&ids, Path::new(r"C:\t\out.json"));
        assert!(s.contains("powercfg /hibernate off"));
        assert!(s.contains("startcomponentcleanup"));
        assert!(!s.contains("resetbase"));
        assert!(!s.contains("bogus"));
        assert!(!s.contains("Win32_ShadowCopy"));
        assert!(s.ends_with("out.json\"\n"));
    }

    #[test]
    fn run_results_map_exit_codes() {
        let ids = vec![
            WINSXS.to_string(),
            HIBERNATION.to_string(),
            WU_CACHE.to_string(),
        ];
        let r = parse_run(
            &ids,
            "{\"winsxs\":0,\"hibernation\":-1,\"hibernation_error\":\"nope\"}",
        );
        assert!(r[0].ok);
        assert!(!r[1].ok);
        assert_eq!(r[1].message, "nope");
        assert!(!r[2].ok);
        assert!(r[2].message.contains("did not run"));
        // Declined UAC: no file at all.
        assert!(parse_run(&ids, "").iter().all(|x| !x.ok));
    }

    #[test]
    fn catalogue_has_every_task_in_both_languages() {
        for lang in ["zh", "en"] {
            let t = tasks(lang);
            assert_eq!(t.len(), TASK_IDS.len());
            assert!(t
                .iter()
                .all(|x| !x.title.is_empty() && !x.consequence.is_empty()));
        }
    }
}
