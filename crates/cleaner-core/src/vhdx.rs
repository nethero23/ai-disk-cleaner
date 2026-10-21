//! Compacting WSL / Docker Desktop virtual disks.
//!
//! A WSL2 `.vhdx` only ever grows: deleting files inside Linux (or
//! `docker system prune`) frees space in the guest, not on the Windows drive.
//! The fix is `wsl --shutdown` followed by `diskpart`'s `compact vdisk`,
//! which needs administrator rights. `Optimize-VHD` would do the same but
//! needs the Hyper-V module, absent on Windows Home, so diskpart it is.

use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// A virtual disk found under a scanned item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vhdx {
    pub path: PathBuf,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactResult {
    pub path: PathBuf,
    pub before: u64,
    pub after: u64,
}

/// Rule ids whose items hold virtual disks worth compacting.
pub const COMPACTABLE_RULES: &[&str] = &["docker.vhdx", "wsl.distros"];

/// Every `.vhdx` under `root` (or `root` itself when it is one), largest first.
pub fn find(root: &Path) -> Vec<Vhdx> {
    let mut out: Vec<Vhdx> = WalkDir::new(root)
        .follow_links(false)
        .max_depth(4)
        .into_iter()
        .flatten()
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .is_some_and(|x| x.eq_ignore_ascii_case("vhdx"))
        })
        .filter_map(|e| {
            Some(Vhdx {
                path: e.path().to_path_buf(),
                size_bytes: e.metadata().ok()?.len(),
            })
        })
        .collect();
    out.sort_by_key(|v| std::cmp::Reverse(v.size_bytes));
    out
}

/// The diskpart script that compacts one disk. Attaching read-only is what
/// lets diskpart compact a disk that has an NTFS-foreign (ext4) payload.
pub fn diskpart_script(vhdx: &Path) -> String {
    format!(
        "select vdisk file=\"{}\"\r\nattach vdisk readonly\r\ncompact vdisk\r\ndetach vdisk\r\nexit\r\n",
        vhdx.display()
    )
}

/// Shuts WSL down (which also stops Docker Desktop's engine), then compacts
/// `vhdx` with an elevated diskpart. Blocks until diskpart exits.
pub fn compact(vhdx: &Path) -> Result<CompactResult> {
    if !vhdx.is_file() {
        return Err(Error::Refused(
            vhdx.to_path_buf(),
            "is not a virtual disk file".into(),
        ));
    }
    let before = std::fs::metadata(vhdx)
        .map_err(|source| Error::Io {
            path: vhdx.to_path_buf(),
            source,
        })?
        .len();
    run_compact(vhdx)?;
    let after = std::fs::metadata(vhdx)
        .map_err(|source| Error::Io {
            path: vhdx.to_path_buf(),
            source,
        })?
        .len();
    Ok(CompactResult {
        path: vhdx.to_path_buf(),
        before,
        after,
    })
}

#[cfg(windows)]
fn run_compact(vhdx: &Path) -> Result<()> {
    use std::process::Command;
    let io = |source| Error::Io {
        path: vhdx.to_path_buf(),
        source,
    };
    // Stop every distro and Docker's VM so the disk is closed. Ignoring the
    // status is deliberate: when WSL is not installed this fails harmlessly.
    let _ = Command::new("wsl").arg("--shutdown").status();
    let script = std::env::temp_dir().join("aidc-compact.txt");
    std::fs::write(&script, diskpart_script(vhdx)).map_err(io)?;
    // diskpart must run elevated; Start-Process -Verb RunAs shows the UAC
    // prompt and -Wait -PassThru hands the exit code back.
    let ps = format!(
        "$p = Start-Process diskpart -ArgumentList '/s \"{}\"' -Verb RunAs -Wait -PassThru; exit $p.ExitCode",
        script.display()
    );
    let status = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &ps])
        .status()
        .map_err(io)?;
    let _ = std::fs::remove_file(&script);
    if !status.success() {
        return Err(Error::Refused(
            vhdx.to_path_buf(),
            format!(
                "diskpart failed ({}); the UAC prompt may have been declined",
                status.code().unwrap_or(-1)
            ),
        ));
    }
    Ok(())
}

#[cfg(not(windows))]
fn run_compact(vhdx: &Path) -> Result<()> {
    Err(Error::Refused(
        vhdx.to_path_buf(),
        "compacting virtual disks only works on Windows".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_disks_largest_first() {
        let tmp = tempfile::tempdir().unwrap();
        let a = tmp.path().join("data/ext4.vhdx");
        let b = tmp.path().join("disk/docker_data.VHDX");
        std::fs::create_dir_all(a.parent().unwrap()).unwrap();
        std::fs::create_dir_all(b.parent().unwrap()).unwrap();
        std::fs::write(&a, [0u8; 10]).unwrap();
        std::fs::write(&b, [0u8; 20]).unwrap();
        std::fs::write(tmp.path().join("notes.txt"), [0u8; 30]).unwrap();
        let found = find(tmp.path());
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].path, b);
        assert_eq!(found[0].size_bytes, 20);
    }

    #[test]
    fn script_targets_the_disk() {
        let s = diskpart_script(Path::new(r"C:\x\ext4.vhdx"));
        assert!(s.starts_with("select vdisk file=\"C:\\x\\ext4.vhdx\"\r\n"));
        assert!(s.contains("compact vdisk"));
        assert!(s.trim_end().ends_with("exit"));
    }
}
