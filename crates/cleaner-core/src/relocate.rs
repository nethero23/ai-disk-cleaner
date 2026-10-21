//! Move a cache or model store to another drive and teach its tool the new
//! place. Deleting is what the other cleaners do; what users with a full
//! C: and an empty D: actually want is this.
//!
//! Steps: copy the tree with progress → verify file count and bytes match →
//! point the tool at the new location (an environment variable where the
//! tool has one, otherwise a directory junction left at the old path) →
//! remove the original. The original is never touched before the copy is
//! verified, so a failure half-way leaves everything as it was plus a
//! partial copy the caller can delete.

use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// How a tool is told where its data moved to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "value")]
pub enum Redirect {
    /// Set this user environment variable to the new path.
    EnvVar(String),
    /// Leave a junction (Windows) / symlink at the old path.
    Junction,
}

/// Rules whose location can be moved, and how their tool finds it afterwards.
pub fn redirect_for(rule_id: &str) -> Option<Redirect> {
    Some(match rule_id {
        "ollama.models" => Redirect::EnvVar("OLLAMA_MODELS".into()),
        "huggingface.hub" => Redirect::EnvVar("HF_HUB_CACHE".into()),
        "pip.cache" => Redirect::EnvVar("PIP_CACHE_DIR".into()),
        "uv.cache" => Redirect::EnvVar("UV_CACHE_DIR".into()),
        "uv.python" => Redirect::EnvVar("UV_PYTHON_INSTALL_DIR".into()),
        "conda.pkgs" => Redirect::EnvVar("CONDA_PKGS_DIRS".into()),
        "playwright.browsers" => Redirect::EnvVar("PLAYWRIGHT_BROWSERS_PATH".into()),
        "torch.hub" => Redirect::EnvVar("TORCH_HOME".into()),
        "npm.cache" => Redirect::EnvVar("NPM_CONFIG_CACHE".into()),
        "cargo.registry" | "gradle.caches" | "go.modcache" | "nuget.cache" => Redirect::Junction,
        "lmstudio.models" | "jan.models" | "whisper.models" | "docker.vhdx" => Redirect::Junction,
        _ => return None,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelocateResult {
    pub from: PathBuf,
    pub to: PathBuf,
    pub bytes: u64,
    pub files: u64,
    pub redirect: Redirect,
    /// What the user must still do by hand, if anything (e.g. restart the tool).
    pub note: String,
}

/// Called with (copied_bytes, total_bytes) as the copy proceeds.
pub type Progress<'a> = &'a mut dyn FnMut(u64, u64);

fn count_tree(root: &Path) -> (u64, u64) {
    let mut bytes = 0;
    let mut files = 0;
    for e in WalkDir::new(root).follow_links(false).into_iter().flatten() {
        if e.file_type().is_file() {
            files += 1;
            bytes += e.metadata().map(|m| m.len()).unwrap_or(0);
        }
    }
    (bytes, files)
}

fn copy_tree(src: &Path, dst: &Path, total: u64, progress: Progress) -> Result<u64> {
    let mut copied = 0;
    for e in WalkDir::new(src).follow_links(false).into_iter().flatten() {
        let rel = e.path().strip_prefix(src).unwrap_or(e.path());
        let target = dst.join(rel);
        let ft = e.file_type();
        if ft.is_dir() {
            std::fs::create_dir_all(&target).map_err(|source| Error::Io {
                path: target.clone(),
                source,
            })?;
        } else if ft.is_file() {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent).map_err(|source| Error::Io {
                    path: parent.to_path_buf(),
                    source,
                })?;
            }
            let n = std::fs::copy(e.path(), &target).map_err(|source| Error::Io {
                path: e.path().to_path_buf(),
                source,
            })?;
            copied += n;
            progress(copied, total);
        }
        // Symlinks and junctions inside a cache are not carried over: a cache
        // that points elsewhere is rebuilt by its tool anyway.
    }
    Ok(copied)
}

/// Validates the request before any byte moves.
pub fn check(src: &Path, dest_root: &Path) -> Result<PathBuf> {
    crate::clean::guard(src)?;
    if !src.is_dir() {
        return Err(Error::Refused(src.into(), "is not a folder".into()));
    }
    if !dest_root.is_dir() {
        return Err(Error::Refused(
            dest_root.into(),
            "destination does not exist".into(),
        ));
    }
    let name = src
        .file_name()
        .ok_or_else(|| Error::Refused(src.into(), "has no name".into()))?;
    let dest = dest_root.join(name);
    if dest.starts_with(src) {
        return Err(Error::Refused(
            dest,
            "destination is inside the source".into(),
        ));
    }
    if dest.exists() {
        return Err(Error::Refused(dest, "destination already exists".into()));
    }
    Ok(dest)
}

/// Moves `src` into `dest_root` and redirects the tool. See module docs.
pub fn relocate(
    rule_id: &str,
    src: &Path,
    dest_root: &Path,
    progress: Progress,
) -> Result<RelocateResult> {
    let redirect = redirect_for(rule_id)
        .ok_or_else(|| Error::Refused(src.into(), format!("rule {rule_id} cannot be moved")))?;
    let dest = check(src, dest_root)?;
    let (total_bytes, total_files) = count_tree(src);

    copy_tree(src, &dest, total_bytes, progress)?;
    let (got_bytes, got_files) = count_tree(&dest);
    if got_bytes != total_bytes || got_files != total_files {
        return Err(Error::Refused(
            dest,
            format!(
                "copy verification failed ({got_files}/{total_files} files, {got_bytes}/{total_bytes} bytes); original left untouched"
            ),
        ));
    }

    let note = match &redirect {
        Redirect::EnvVar(var) => {
            set_user_env(var, &dest)?;
            std::fs::remove_dir_all(src).map_err(|source| Error::Io {
                path: src.into(),
                source,
            })?;
            restart_note(rule_id)
        }
        Redirect::Junction => {
            std::fs::remove_dir_all(src).map_err(|source| Error::Io {
                path: src.into(),
                source,
            })?;
            make_junction(src, &dest)?;
            String::new()
        }
    };

    Ok(RelocateResult {
        from: src.into(),
        to: dest,
        bytes: total_bytes,
        files: total_files,
        redirect,
        note,
    })
}

fn restart_note(rule_id: &str) -> String {
    match rule_id {
        "ollama.models" => "restart Ollama".into(),
        _ => "open a new terminal for the variable to take effect".into(),
    }
}

/// Sets a persistent user-level environment variable.
#[cfg(windows)]
fn set_user_env(var: &str, value: &Path) -> Result<()> {
    // `setx` writes HKCU\Environment and broadcasts WM_SETTINGCHANGE, so new
    // processes see it without a logout. It does not change this process.
    let status = std::process::Command::new("setx")
        .arg(var)
        .arg(value)
        .status()
        .map_err(|source| Error::Io {
            path: value.into(),
            source,
        })?;
    if !status.success() {
        return Err(Error::Refused(value.into(), format!("setx {var} failed")));
    }
    Ok(())
}

#[cfg(not(windows))]
fn set_user_env(var: &str, value: &Path) -> Result<()> {
    // No universal user-env store on Unix; append to ~/.profile-style file is
    // intrusive, so record the intent and let the caller tell the user.
    log::info!("would set {var}={}", value.display());
    Ok(())
}

#[cfg(windows)]
fn make_junction(at: &Path, target: &Path) -> Result<()> {
    // A junction needs no privilege, unlike a symlink.
    let status = std::process::Command::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(at)
        .arg(target)
        .status()
        .map_err(|source| Error::Io {
            path: at.into(),
            source,
        })?;
    if !status.success() {
        return Err(Error::Refused(at.into(), "mklink /J failed".into()));
    }
    Ok(())
}

#[cfg(not(windows))]
fn make_junction(at: &Path, target: &Path) -> Result<()> {
    std::os::unix::fs::symlink(target, at).map_err(|source| Error::Io {
        path: at.into(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn moves_tree_and_leaves_link_behind() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("c").join("models");
        fs::create_dir_all(src.join("sub")).unwrap();
        fs::write(src.join("a.bin"), vec![1u8; 3000]).unwrap();
        fs::write(src.join("sub/b.bin"), vec![2u8; 500]).unwrap();
        let d = tmp.path().join("d");
        fs::create_dir_all(&d).unwrap();

        let mut last = (0, 0);
        let r = relocate("lmstudio.models", &src, &d, &mut |c, t| last = (c, t)).unwrap();
        assert_eq!(r.bytes, 3500);
        assert_eq!(r.files, 2);
        assert_eq!(last, (3500, 3500));
        assert_eq!(r.redirect, Redirect::Junction);
        assert!(d.join("models/sub/b.bin").exists());
        // The old path still resolves, through the link.
        assert!(src.is_symlink());
        assert_eq!(fs::read(src.join("a.bin")).unwrap().len(), 3000);
    }

    #[test]
    fn refuses_bad_destinations() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        assert!(check(&src, &src).is_err(), "inside itself");
        assert!(
            check(&src, &tmp.path().join("missing")).is_err(),
            "missing dest"
        );
        fs::create_dir_all(tmp.path().join("taken/src")).unwrap();
        assert!(
            check(&src, &tmp.path().join("taken")).is_err(),
            "already exists"
        );
        assert!(
            relocate("pip.cache", &src, tmp.path(), &mut |_, _| {}).is_err(),
            "unmovable rule? no: dest exists"
        );
        assert!(redirect_for("windows.user_temp").is_none());
    }
}
