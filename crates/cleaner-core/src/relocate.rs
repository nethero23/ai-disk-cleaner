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

use crate::paths::PathContext;
use crate::scan::resolve_paths;
use crate::{Error, Result, RuleSet};
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
        "modelscope.hub" => Redirect::EnvVar("MODELSCOPE_CACHE".into()),
        "npm.cache" => Redirect::EnvVar("NPM_CONFIG_CACHE".into()),
        "cargo.registry" | "gradle.caches" | "go.modcache" | "nuget.cache" => Redirect::Junction,
        "lmstudio.models" | "jan.models" | "whisper.models" | "docker.vhdx" => Redirect::Junction,
        _ => return None,
    })
}

/// The folder a move of `item_path` actually acts on. A tool told about its
/// new place through an environment variable reads its whole store from
/// there, so moving one Playwright browser or one Ollama model (whose item
/// path is a manifest file) out of the store would point the tool at that
/// single folder and lose everything else: those rules move the store root.
/// Junction rules move just the item; the junction keeps the rest intact.
pub fn move_source(rule_id: &str, item_path: &Path, ctx: &PathContext) -> PathBuf {
    if !matches!(redirect_for(rule_id), Some(Redirect::EnvVar(_))) {
        return item_path.into();
    }
    RuleSet::builtin()
        .rules
        .iter()
        .find(|r| r.id == rule_id)
        .and_then(|r| {
            resolve_paths(r, ctx)
                .into_iter()
                .find(|root| item_path.starts_with(root))
        })
        .unwrap_or_else(|| item_path.into())
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
    /// For env-var redirects: what the variable held before the move, so
    /// move-back can restore it instead of deleting it.
    #[serde(default)]
    pub previous_env: Option<String>,
}

/// One line of the relocation log: enough to move the data back.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelocateRecord {
    pub at: chrono::DateTime<chrono::Utc>,
    pub rule_id: String,
    pub from: PathBuf,
    pub to: PathBuf,
    pub bytes: u64,
    pub redirect: Redirect,
    /// False once moved back (or the record is otherwise stale).
    #[serde(default = "default_true")]
    pub active: bool,
    #[serde(default)]
    pub previous_env: Option<String>,
}

fn default_true() -> bool {
    true
}

pub fn log_relocation(log_path: &Path, rule_id: &str, r: &RelocateResult) {
    let rec = RelocateRecord {
        at: chrono::Utc::now(),
        rule_id: rule_id.into(),
        from: r.from.clone(),
        to: r.to.clone(),
        bytes: r.bytes,
        redirect: r.redirect.clone(),
        active: true,
        previous_env: r.previous_env.clone(),
    };
    append_record(log_path, &rec);
}

fn append_record(log_path: &Path, rec: &RelocateRecord) {
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
    {
        if let Ok(line) = serde_json::to_string(rec) {
            let _ = writeln!(f, "{line}");
        }
    }
}

/// Relocations still in effect: the log is append-only, and a move back
/// appends the same record with `active: false`, so the last word wins.
pub fn read_relocations(log_path: &Path) -> Vec<RelocateRecord> {
    let mut latest: std::collections::HashMap<(PathBuf, PathBuf), RelocateRecord> =
        Default::default();
    for line in std::fs::read_to_string(log_path)
        .unwrap_or_default()
        .lines()
    {
        if let Ok(rec) = serde_json::from_str::<RelocateRecord>(line) {
            latest.insert((rec.from.clone(), rec.to.clone()), rec);
        }
    }
    let mut out: Vec<_> = latest.into_values().filter(|r| r.active).collect();
    out.sort_by_key(|r| std::cmp::Reverse(r.at));
    out
}

/// Reverses a relocation: copies the data back to `from`, verifies, points
/// the tool at the original place again, removes the copy at `to`.
pub fn move_back(log_path: &Path, rec: &RelocateRecord, progress: Progress) -> Result<()> {
    if !rec.to.is_dir() {
        return Err(Error::Refused(
            rec.to.clone(),
            "the moved folder no longer exists".into(),
        ));
    }
    let linked = rec.from.is_symlink() || is_junction(&rec.from);
    if rec.from.exists() && !linked {
        return Err(Error::Refused(
            rec.from.clone(),
            "something already exists at the original location".into(),
        ));
    }
    // Copy next to the original first (same volume, so the final step is a
    // rename). The junction stays in place until the copy is known good, so a
    // locked file or a full disk leaves the tool still working.
    let staging = staging_path(&rec.from);
    let _ = std::fs::remove_dir_all(&staging);
    let (total_bytes, total_files) = count_tree(&rec.to);
    let discard = |e: Error| {
        let _ = std::fs::remove_dir_all(&staging);
        e
    };
    copy_tree(&rec.to, &staging, total_bytes, progress).map_err(discard)?;
    if count_tree(&staging) != (total_bytes, total_files) {
        return Err(discard(Error::Refused(
            rec.from.clone(),
            "copy verification failed; nothing removed".into(),
        )));
    }
    if linked {
        remove_link(&rec.from).map_err(|source| Error::Io {
            path: rec.from.clone(),
            source,
        })?;
    }
    if let Err(source) = std::fs::rename(&staging, &rec.from) {
        // Put things back the way they were: the data is still at `to`.
        if linked {
            let _ = make_junction(&rec.from, &rec.to);
        }
        let _ = std::fs::remove_dir_all(&staging);
        return Err(Error::Io {
            path: rec.from.clone(),
            source,
        });
    }
    if let Redirect::EnvVar(var) = &rec.redirect {
        match &rec.previous_env {
            Some(prev) if !prev.is_empty() => set_user_env(var, Path::new(prev))?,
            _ => unset_user_env(var)?,
        }
    }
    std::fs::remove_dir_all(&rec.to).map_err(|source| Error::Io {
        path: rec.to.clone(),
        source,
    })?;
    append_record(
        log_path,
        &RelocateRecord {
            at: chrono::Utc::now(),
            active: false,
            ..rec.clone()
        },
    );
    Ok(())
}

/// Where move-back copies to before swapping it into place: a sibling of the
/// original, so the rename never crosses volumes.
fn staging_path(from: &Path) -> PathBuf {
    let name = from
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "data".into());
    from.with_file_name(format!("{name}.moving-back"))
}

/// A junction or directory symlink is removed as a directory on Windows and
/// as a file on Unix; neither touches what it points to.
#[cfg(windows)]
fn remove_link(p: &Path) -> std::io::Result<()> {
    std::fs::remove_dir(p)
}

#[cfg(not(windows))]
fn remove_link(p: &Path) -> std::io::Result<()> {
    std::fs::remove_file(p)
}

#[cfg(windows)]
fn is_junction(p: &Path) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
    std::fs::symlink_metadata(p)
        .map(|m| m.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0)
        .unwrap_or(false)
}

#[cfg(not(windows))]
fn is_junction(p: &Path) -> bool {
    p.is_symlink()
}

/// Removes a persistent user-level environment variable.
#[cfg(windows)]
fn unset_user_env(var: &str) -> Result<()> {
    // `setx` cannot delete; PowerShell's SetEnvironmentVariable with $null
    // removes the value and broadcasts the change like setx does.
    let script = format!(
        "[Environment]::SetEnvironmentVariable('{}', $null, 'User')",
        var.replace('\'', "''")
    );
    let status = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .status()
        .map_err(|source| Error::Io {
            path: PathBuf::from(var),
            source,
        })?;
    if !status.success() {
        return Err(Error::Refused(
            PathBuf::from(var),
            "could not remove the environment variable".into(),
        ));
    }
    Ok(())
}

#[cfg(not(windows))]
fn unset_user_env(var: &str) -> Result<()> {
    log::info!("would unset {var}");
    Ok(())
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

    // On any failure the partial copy is ours alone; remove it so a retry
    // does not stop at "destination already exists".
    let discard = |e: Error| {
        let _ = std::fs::remove_dir_all(&dest);
        e
    };
    copy_tree(src, &dest, total_bytes, progress).map_err(discard)?;
    // A live cache (npm writing its logs, a download in progress) changes
    // under the copy; that is not a broken copy, the tool is just running.
    if count_tree(src) != (total_bytes, total_files) {
        return Err(discard(Error::Refused(
            src.into(),
            "its contents changed during the copy; close the tool that uses it and try again. Original left untouched".into(),
        )));
    }
    let (got_bytes, got_files) = count_tree(&dest);
    if got_bytes != total_bytes || got_files != total_files {
        return Err(discard(Error::Refused(
            dest.clone(),
            format!(
                "copy verification failed ({got_files}/{total_files} files, {got_bytes}/{total_bytes} bytes); original left untouched"
            ),
        )));
    }

    let mut previous_env = None;
    let note = match &redirect {
        Redirect::EnvVar(var) => {
            previous_env = get_user_env(var);
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
        previous_env,
    })
}

fn restart_note(rule_id: &str) -> String {
    match rule_id {
        "ollama.models" => "restart Ollama".into(),
        _ => "open a new terminal for the variable to take effect".into(),
    }
}

/// Reads the persistent user-level value of a variable (not the process's
/// own, which may come from the system scope or a shell profile).
#[cfg(windows)]
fn get_user_env(var: &str) -> Option<String> {
    let script = format!(
        "[Environment]::GetEnvironmentVariable('{}', 'User')",
        var.replace('\'', "''")
    );
    let out = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .output()
        .ok()?;
    let v = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (!v.is_empty()).then_some(v)
}

#[cfg(not(windows))]
fn get_user_env(var: &str) -> Option<String> {
    std::env::var(var).ok().filter(|v| !v.is_empty())
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
    fn move_back_restores_the_original_and_retires_the_record() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("c").join("cache");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("a"), vec![1u8; 100]).unwrap();
        let d = tmp.path().join("d");
        fs::create_dir_all(&d).unwrap();
        let log = tmp.path().join("relocate.log");

        let r = relocate("cargo.registry", &src, &d, &mut |_, _| {}).unwrap();
        log_relocation(&log, "cargo.registry", &r);
        let recs = read_relocations(&log);
        assert_eq!(recs.len(), 1);
        assert!(src.is_symlink());

        move_back(&log, &recs[0], &mut |_, _| {}).unwrap();
        assert!(!src.is_symlink());
        assert_eq!(fs::read(src.join("a")).unwrap().len(), 100);
        assert!(!d.join("cache").exists());
        assert!(read_relocations(&log).is_empty());
    }

    #[test]
    fn failed_move_back_leaves_the_link_in_place() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("c").join("cache");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("a"), vec![1u8; 10]).unwrap();
        let d = tmp.path().join("d");
        fs::create_dir_all(&d).unwrap();
        let log = tmp.path().join("relocate.log");
        let r = relocate("cargo.registry", &src, &d, &mut |_, _| {}).unwrap();
        log_relocation(&log, "cargo.registry", &r);
        let rec = read_relocations(&log).remove(0);

        // Make the staging folder un-creatable: a file sits where it would go.
        fs::write(staging_path(&src), b"x").unwrap();
        assert!(move_back(&log, &rec, &mut |_, _| {}).is_err());
        fs::remove_file(staging_path(&src)).unwrap();

        // The link is still there, the data still at its moved place, the
        // record still active.
        assert!(src.is_symlink());
        assert!(d.join("cache").join("a").exists());
        assert_eq!(read_relocations(&log).len(), 1);

        // And a retry now succeeds.
        move_back(&log, &rec, &mut |_, _| {}).unwrap();
        assert!(!src.is_symlink());
        assert!(src.join("a").exists());
        assert!(!staging_path(&src).exists());
    }

    #[test]
    fn records_survive_the_previous_env_field_missing() {
        let line = r#"{"at":"2026-01-01T00:00:00Z","rule_id":"npm.cache","from":"/a","to":"/b","bytes":1,"redirect":{"kind":"env_var","value":"NPM_CONFIG_CACHE"}}"#;
        let rec: RelocateRecord = serde_json::from_str(line).unwrap();
        assert!(rec.active);
        assert_eq!(rec.previous_env, None);
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

    #[test]
    fn env_var_rules_move_the_whole_store() {
        let tmp = tempfile::tempdir().unwrap();
        let store = tmp.path().join("ms-playwright");
        fs::create_dir_all(store.join("chromium-1234")).unwrap();
        fs::create_dir_all(tmp.path().join("models/manifests/x")).unwrap();
        let mut ctx = PathContext::with_vars(Default::default());
        ctx.set("PLAYWRIGHT_BROWSERS_PATH", store.clone());
        ctx.set("OLLAMA_MODELS", tmp.path().join("models"));
        assert_eq!(
            move_source("playwright.browsers", &store.join("chromium-1234"), &ctx),
            store
        );
        assert_eq!(
            move_source(
                "ollama.models",
                &tmp.path().join("models/manifests/x/latest"),
                &ctx
            ),
            tmp.path().join("models")
        );
        // A junction leaves the rest of the store where it is.
        let one = tmp.path().join("lm/models/pub");
        assert_eq!(move_source("lmstudio.models", &one, &ctx), one);
    }
}
