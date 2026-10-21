//! Whole-drive scan for big files. The rule library finds junk it knows
//! about; this finds the 40 GB ISO and the forgotten dataset it does not.
//! Nothing here is deleted automatically: every hit is a Review item and the
//! user picks.

use crate::model::{Category, SafetyLevel, ScanItem};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use walkdir::WalkDir;

pub const DEFAULT_MIN_BYTES: u64 = 500 * 1024 * 1024;
/// Keep the result list usable; the biggest files are what matter.
pub const MAX_RESULTS: usize = 1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileKind {
    Video,
    Archive,
    DiskImage,
    ModelWeights,
    VirtualDisk,
    Installer,
    Database,
    Log,
    Other,
}

impl FileKind {
    pub fn of(path: &Path) -> FileKind {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_ascii_lowercase())
            .unwrap_or_default();
        match ext.as_str() {
            "mp4" | "mkv" | "mov" | "avi" | "wmv" | "flv" | "webm" | "ts" | "m4v" | "mpg"
            | "mpeg" => FileKind::Video,
            "zip" | "7z" | "rar" | "tar" | "gz" | "tgz" | "bz2" | "xz" | "zst" | "cab" => {
                FileKind::Archive
            }
            "iso" | "img" | "dmg" | "wim" | "esd" => FileKind::DiskImage,
            "gguf" | "safetensors" | "ckpt" | "pt" | "pth" | "bin" | "onnx" | "h5" | "npz"
            | "parquet" | "arrow" => FileKind::ModelWeights,
            "vhdx" | "vhd" | "vmdk" | "vdi" | "qcow2" => FileKind::VirtualDisk,
            "exe" | "msi" | "msix" | "appx" | "pkg" | "deb" | "rpm" => FileKind::Installer,
            "db" | "sqlite" | "sqlite3" | "mdf" | "ldf" | "ibd" => FileKind::Database,
            "log" | "etl" | "dmp" | "trace" => FileKind::Log,
            _ => FileKind::Other,
        }
    }

    pub fn label(self, lang: &str) -> &'static str {
        let zh = lang == "zh";
        match self {
            FileKind::Video => {
                if zh {
                    "视频"
                } else {
                    "Video"
                }
            }
            FileKind::Archive => {
                if zh {
                    "压缩包"
                } else {
                    "Archive"
                }
            }
            FileKind::DiskImage => {
                if zh {
                    "镜像"
                } else {
                    "Disk image"
                }
            }
            FileKind::ModelWeights => {
                if zh {
                    "模型/数据"
                } else {
                    "Model / data"
                }
            }
            FileKind::VirtualDisk => {
                if zh {
                    "虚拟磁盘"
                } else {
                    "Virtual disk"
                }
            }
            FileKind::Installer => {
                if zh {
                    "安装包"
                } else {
                    "Installer"
                }
            }
            FileKind::Database => {
                if zh {
                    "数据库"
                } else {
                    "Database"
                }
            }
            FileKind::Log => {
                if zh {
                    "日志/转储"
                } else {
                    "Log / dump"
                }
            }
            FileKind::Other => {
                if zh {
                    "其他"
                } else {
                    "Other"
                }
            }
        }
    }
}

/// A mounted volume the user can pick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Drive {
    pub path: PathBuf,
    pub name: String,
    pub total_bytes: u64,
    pub free_bytes: u64,
}

pub fn drives() -> Vec<Drive> {
    let disks = sysinfo::Disks::new_with_refreshed_list();
    let mut out: Vec<Drive> = disks
        .iter()
        .filter(|d| d.total_space() > 0)
        .map(|d| Drive {
            path: d.mount_point().to_path_buf(),
            name: d.name().to_string_lossy().to_string(),
            total_bytes: d.total_space(),
            free_bytes: d.available_space(),
        })
        .collect();
    out.sort_by(|a, b| a.path.cmp(&b.path));
    out.dedup_by(|a, b| a.path == b.path);
    out
}

/// Folders a whole-drive walk must not enter: the OS, the recycle bin, and
/// the places the rule scanner already covers better.
fn skip_dir(p: &Path) -> bool {
    let Some(name) = p.file_name().and_then(|n| n.to_str()) else {
        return false;
    };
    let n = name.to_ascii_lowercase();
    matches!(
        n.as_str(),
        "$recycle.bin"
            | "system volume information"
            | "windows"
            | "windowsapps"
            | "winsxs"
            | "proc"
            | "sys"
            | "dev"
    ) || (p.components().count() == 2 && n == "windows")
}

/// Page and swap files are managed by Windows and never deletable; they are
/// not listed. The hibernation file is listed as information only, since
/// `powercfg /h off` is the way to get that space back.
fn skip_file(p: &Path) -> bool {
    let Some(name) = p.file_name().and_then(|n| n.to_str()) else {
        return false;
    };
    matches!(
        name.to_ascii_lowercase().as_str(),
        "pagefile.sys" | "swapfile.sys"
    )
}

fn is_hiberfil(p: &Path) -> bool {
    p.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.eq_ignore_ascii_case("hiberfil.sys"))
}

/// Rule ids used for big-file items. The UI treats everything starting with
/// `bigfiles` as a big file; the suffix says which ones need special care.
pub const RULE_FILE: &str = "bigfiles";
pub const RULE_VDISK: &str = "bigfiles.vdisk";
pub const RULE_HIBERFIL: &str = "bigfiles.hiberfil";

fn describe(kind: FileKind, path: &Path, size: u64, lang: &str) -> (&'static str, String) {
    let zh = lang == "zh";
    if is_hiberfil(path) {
        let s = crate::model::human_size(size);
        return (
            RULE_HIBERFIL,
            if zh {
                format!("休眠文件，不能直接删除。以管理员身份运行 powercfg /h off 可释放 {s}（同时关闭休眠和快速启动）")
            } else {
                format!("Hibernation file; cannot be deleted. Run powercfg /h off as administrator to free {s} (also turns off hibernate and fast startup)")
            },
        );
    }
    match kind {
        FileKind::VirtualDisk => (
            RULE_VDISK,
            if zh {
                "虚拟磁盘（WSL / Docker / 虚拟机 / Claude Desktop 等），里面是整个系统的数据，删除等于删掉那个系统。想省空间请用「压缩虚拟磁盘」".into()
            } else {
                "A virtual disk (WSL / Docker / a VM / Claude Desktop): deleting it deletes that whole system. To save space, compact it instead".into()
            },
        ),
        _ => (
            RULE_FILE,
            if zh {
                "单个大文件，删除前请确认不再需要".into()
            } else {
                "A single large file; make sure it is no longer needed".into()
            },
        ),
    }
}

/// Progress callback: (files seen, bytes matched so far, folder being walked).
pub type Progress<'a> = &'a mut dyn FnMut(u64, u64, &Path);

/// Walks `roots` and returns files of at least `min_bytes`, biggest first,
/// as Review items the clean pipeline understands. `cancel` stops the walk
/// early and returns what was found so far.
pub fn scan(
    roots: &[PathBuf],
    min_bytes: u64,
    excluded: &[PathBuf],
    lang: &str,
    cancel: &AtomicBool,
    progress: Progress,
) -> Vec<ScanItem> {
    let mut found: Vec<(u64, PathBuf, Option<DateTime<Utc>>)> = Vec::new();
    let mut seen = 0u64;
    let mut matched_bytes = 0u64;
    let mut last_report = std::time::Instant::now();
    'roots: for root in roots {
        let walker = WalkDir::new(root)
            .follow_links(false)
            .same_file_system(true)
            .into_iter()
            .filter_entry(|e| {
                !(e.file_type().is_dir()
                    && (skip_dir(e.path()) || excluded.iter().any(|x| e.path().starts_with(x))))
            });
        for e in walker.flatten() {
            if cancel.load(Ordering::Relaxed) {
                break 'roots;
            }
            if e.file_type().is_dir() {
                if last_report.elapsed().as_millis() > 150 {
                    progress(seen, matched_bytes, e.path());
                    last_report = std::time::Instant::now();
                }
                continue;
            }
            if !e.file_type().is_file() || skip_file(e.path()) {
                continue;
            }
            seen += 1;
            let Ok(md) = e.metadata() else { continue };
            if md.len() < min_bytes {
                continue;
            }
            matched_bytes += md.len();
            let used = md
                .accessed()
                .ok()
                .filter(|t| *t > std::time::UNIX_EPOCH)
                .or_else(|| md.modified().ok())
                .map(DateTime::<Utc>::from);
            found.push((md.len(), e.path().to_path_buf(), used));
        }
    }
    progress(seen, matched_bytes, Path::new(""));
    found.sort_by_key(|(size, _, _)| std::cmp::Reverse(*size));
    found.truncate(MAX_RESULTS);
    found
        .into_iter()
        .map(|(size, path, used)| {
            let kind = FileKind::of(&path);
            let (rule_id, consequence) = describe(kind, &path, size, lang);
            ScanItem {
                id: format!("bigfile:{}", path.display()),
                rule_id: rule_id.into(),
                tool: kind.label(lang).into(),
                category: Category::System,
                name: path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default(),
                size_bytes: size,
                file_count: 1,
                last_used: used,
                safety: SafetyLevel::Review,
                consequence,
                detail: path.parent().map(|p| p.to_string_lossy().to_string()),
                in_use_by: None,
                auto_clean: false,
                path,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn finds_only_files_over_the_floor_biggest_first() {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("a/b")).unwrap();
        fs::create_dir_all(tmp.path().join("skipme")).unwrap();
        fs::write(tmp.path().join("small.bin"), [0u8; 10]).unwrap();
        fs::write(tmp.path().join("a/movie.mkv"), [0u8; 300]).unwrap();
        fs::write(tmp.path().join("a/b/model.gguf"), [0u8; 500]).unwrap();
        fs::write(tmp.path().join("skipme/big.iso"), [0u8; 900]).unwrap();
        let cancel = AtomicBool::new(false);
        let items = scan(
            &[tmp.path().to_path_buf()],
            100,
            &[tmp.path().join("skipme")],
            "en",
            &cancel,
            &mut |_, _, _| {},
        );
        let names: Vec<_> = items.iter().map(|i| i.name.as_str()).collect();
        assert_eq!(names, vec!["model.gguf", "movie.mkv"]);
        assert_eq!(items[0].tool, "Model / data");
        assert_eq!(items[1].tool, "Video");
        assert_eq!(items[0].rule_id, "bigfiles");
    }

    #[test]
    fn cancel_stops_early() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("x.zip"), [0u8; 50]).unwrap();
        let cancel = AtomicBool::new(true);
        let items = scan(
            &[tmp.path().to_path_buf()],
            1,
            &[],
            "zh",
            &cancel,
            &mut |_, _, _| {},
        );
        assert!(items.is_empty());
    }

    #[test]
    fn virtual_disks_and_hiberfil_get_their_own_rule_ids() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("ext4.vhdx"), vec![0u8; 3000]).unwrap();
        fs::write(tmp.path().join("hiberfil.sys"), vec![0u8; 2000]).unwrap();
        fs::write(tmp.path().join("pagefile.sys"), vec![0u8; 2000]).unwrap();
        fs::write(tmp.path().join("movie.mkv"), vec![0u8; 1000]).unwrap();
        let items = scan(
            &[tmp.path().to_path_buf()],
            500,
            &[],
            "zh",
            &AtomicBool::new(false),
            &mut |_, _, _| {},
        );
        let ids: Vec<(&str, &str)> = items
            .iter()
            .map(|i| (i.name.as_str(), i.rule_id.as_str()))
            .collect();
        assert_eq!(
            ids,
            vec![
                ("ext4.vhdx", RULE_VDISK),
                ("hiberfil.sys", RULE_HIBERFIL),
                ("movie.mkv", RULE_FILE)
            ]
        );
        assert!(items[1].consequence.contains("powercfg /h off"));
        assert!(items[1].consequence.contains("2.0 KB") || items[1].consequence.contains("KB"));
    }

    #[test]
    fn kinds_by_extension() {
        assert_eq!(FileKind::of(Path::new("a/B.MKV")), FileKind::Video);
        assert_eq!(
            FileKind::of(Path::new("a/ext4.vhdx")),
            FileKind::VirtualDisk
        );
        assert_eq!(FileKind::of(Path::new("a/noext")), FileKind::Other);
    }

    #[test]
    fn lists_at_least_one_drive() {
        assert!(!drives().is_empty());
    }
}
