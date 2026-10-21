//! The same model weights often sit in two or three places: Hugging Face's
//! cache, LM Studio's folder, a ComfyUI install. This finds large files with
//! identical content across the scanned model stores and can replace the
//! copies with hard links, which keeps every tool working and frees the
//! space of all but one copy.
//!
//! Identity = same size AND same SHA-256 of three 4 MB samples (head, middle,
//! tail). Full hashes of hundreds of GB would take longer than the scan is
//! worth; sampled hashes with size make an accidental match vanishingly
//! unlikely for model weights, and a merge never loses data in any case: a
//! hard link is just another name for the same bytes.

use crate::model::{Category, ScanItem};
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub const MIN_FILE_BYTES: u64 = 100 * 1024 * 1024;
const SAMPLE: u64 = 4 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DupFile {
    pub path: PathBuf,
    /// Which scanned item (tool) the file belongs to.
    pub tool: String,
    pub item_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateGroup {
    pub size_bytes: u64,
    pub files: Vec<DupFile>,
    /// Bytes freed by keeping one copy and linking the rest.
    pub reclaimable_bytes: u64,
    /// Hard links need one volume; false means "can only report".
    pub same_volume: bool,
}

/// Candidate extensions; the Ollama / HF blob stores have none, so files in a
/// folder literally named `blobs` count too.
fn is_weight_file(p: &Path) -> bool {
    let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
    matches!(
        ext,
        "gguf" | "safetensors" | "bin" | "pt" | "pth" | "ckpt" | "onnx"
    ) || p
        .parent()
        .and_then(|d| d.file_name())
        .is_some_and(|n| n == "blobs")
}

pub fn sampled_hash(p: &Path, size: u64) -> std::io::Result<[u8; 32]> {
    let mut f = std::fs::File::open(p)?;
    let mut h = Sha256::new();
    h.update(size.to_le_bytes());
    let mut buf = vec![0u8; SAMPLE as usize];
    let offsets = if size <= 3 * SAMPLE {
        vec![0]
    } else {
        vec![0, size / 2 - SAMPLE / 2, size - SAMPLE]
    };
    for off in offsets {
        f.seek(SeekFrom::Start(off))?;
        let want = if size <= 3 * SAMPLE {
            size as usize
        } else {
            SAMPLE as usize
        };
        let mut read = 0;
        while read < want {
            let n = f.read(&mut buf[read..want])?;
            if n == 0 {
                break;
            }
            read += n;
        }
        h.update(&buf[..read]);
    }
    Ok(h.finalize().into())
}

fn volume_of(p: &Path) -> String {
    // Windows: drive letter or UNC share; Unix: everything is "one volume" as
    // far as this check goes (a cross-device link fails loudly at merge time).
    match p.components().next() {
        Some(std::path::Component::Prefix(pr)) => pr.as_os_str().to_string_lossy().to_lowercase(),
        _ => "/".into(),
    }
}

/// An Ollama item's path is its manifest; the weights live in the sibling
/// `blobs` folder under the models root.
fn scan_root(p: &Path) -> PathBuf {
    if p.is_dir() {
        return p.to_path_buf();
    }
    p.ancestors()
        .find(|a| a.file_name().is_some_and(|n| n == "manifests"))
        .and_then(|m| m.parent())
        .map(|root| root.join("blobs"))
        .unwrap_or_else(|| p.to_path_buf())
}

/// Finds duplicate weight files across the model items of a scan.
pub fn find_duplicates(items: &[ScanItem]) -> Vec<DuplicateGroup> {
    // First by size (cheap), then hash only sizes that collide.
    let mut by_size: HashMap<u64, Vec<DupFile>> = HashMap::new();
    // Every Ollama model (and every orphan blob) maps to the same `blobs`
    // folder; walk it once and count each file once, or each blob turns up
    // as a duplicate of itself.
    let mut roots = std::collections::HashSet::new();
    let mut files = std::collections::HashSet::new();
    for item in items.iter().filter(|i| i.category == Category::Models) {
        let root = scan_root(&item.path);
        if !roots.insert(root.clone()) {
            continue;
        }
        for e in WalkDir::new(&root)
            .follow_links(false)
            .into_iter()
            .flatten()
        {
            if !e.file_type().is_file() || !is_weight_file(e.path()) {
                continue;
            }
            let Ok(md) = e.metadata() else { continue };
            if md.len() < MIN_FILE_BYTES || !files.insert(e.path().to_path_buf()) {
                continue;
            }
            by_size.entry(md.len()).or_default().push(DupFile {
                path: e.path().to_path_buf(),
                tool: item.tool.clone(),
                item_id: item.id.clone(),
            });
        }
    }
    let mut groups = Vec::new();
    for (size, files) in by_size.into_iter().filter(|(_, f)| f.len() > 1) {
        let mut by_hash: HashMap<[u8; 32], Vec<DupFile>> = HashMap::new();
        for f in files {
            if let Ok(h) = sampled_hash(&f.path, size) {
                by_hash.entry(h).or_default().push(f);
            }
        }
        for (_, files) in by_hash.into_iter().filter(|(_, f)| f.len() > 1) {
            // Already hard-linked copies are not duplicates: same inode / file id.
            let files = dedupe_already_linked(files);
            if files.len() < 2 {
                continue;
            }
            let vol = volume_of(&files[0].path);
            let same_volume = files.iter().all(|f| volume_of(&f.path) == vol);
            groups.push(DuplicateGroup {
                size_bytes: size,
                reclaimable_bytes: size * (files.len() as u64 - 1),
                files,
                same_volume,
            });
        }
    }
    groups.sort_by_key(|g| std::cmp::Reverse(g.reclaimable_bytes));
    groups
}

#[cfg(unix)]
fn file_identity(p: &Path) -> Option<(u64, u64)> {
    use std::os::unix::fs::MetadataExt;
    let md = std::fs::metadata(p).ok()?;
    Some((md.dev(), md.ino()))
}

#[cfg(windows)]
fn file_identity(_p: &Path) -> Option<(u64, u64)> {
    // Stable std exposes no file index on Windows; treat copies as distinct
    // and let a merge of already-linked files be a harmless no-op.
    None
}

fn dedupe_already_linked(files: Vec<DupFile>) -> Vec<DupFile> {
    let mut seen = std::collections::HashSet::new();
    files
        .into_iter()
        .filter(|f| match file_identity(&f.path) {
            Some(id) => seen.insert(id),
            None => true,
        })
        .collect()
}

/// Replaces every file in the group except `keep` with a hard link to it.
/// Each copy is renamed aside first and removed only after the link exists,
/// so a failure leaves the original in place.
pub fn merge_group(group: &DuplicateGroup, keep: &Path) -> Result<u64> {
    if !group.files.iter().any(|f| f.path == keep) {
        return Err(Error::Refused(keep.into(), "is not in this group".into()));
    }
    if !group.same_volume {
        return Err(Error::Refused(
            keep.into(),
            "copies are on different drives".into(),
        ));
    }
    let mut freed = 0;
    for f in group.files.iter().filter(|f| f.path != keep) {
        let aside = f.path.with_extension("aidc-dup-tmp");
        std::fs::rename(&f.path, &aside).map_err(|source| Error::Io {
            path: f.path.clone(),
            source,
        })?;
        if let Err(source) = std::fs::hard_link(keep, &f.path) {
            let _ = std::fs::rename(&aside, &f.path);
            return Err(Error::Io {
                path: f.path.clone(),
                source,
            });
        }
        std::fs::remove_file(&aside).map_err(|source| Error::Io {
            path: aside.clone(),
            source,
        })?;
        freed += group.size_bytes;
    }
    Ok(freed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn sampled_hash_distinguishes_content_not_name() {
        let tmp = tempfile::tempdir().unwrap();
        let a = tmp.path().join("a.gguf");
        let b = tmp.path().join("b.gguf");
        let c = tmp.path().join("c.gguf");
        fs::write(&a, vec![7u8; 10_000]).unwrap();
        fs::write(&b, vec![7u8; 10_000]).unwrap();
        fs::write(&c, vec![8u8; 10_000]).unwrap();
        assert_eq!(
            sampled_hash(&a, 10_000).unwrap(),
            sampled_hash(&b, 10_000).unwrap()
        );
        assert_ne!(
            sampled_hash(&a, 10_000).unwrap(),
            sampled_hash(&c, 10_000).unwrap()
        );
    }

    #[test]
    fn merge_replaces_copies_with_hard_links() {
        let tmp = tempfile::tempdir().unwrap();
        let a = tmp.path().join("hf/model.safetensors");
        let b = tmp.path().join("lm/model.safetensors");
        fs::create_dir_all(a.parent().unwrap()).unwrap();
        fs::create_dir_all(b.parent().unwrap()).unwrap();
        fs::write(&a, vec![1u8; 2048]).unwrap();
        fs::write(&b, vec![1u8; 2048]).unwrap();
        let g = DuplicateGroup {
            size_bytes: 2048,
            reclaimable_bytes: 2048,
            same_volume: true,
            files: vec![
                DupFile {
                    path: a.clone(),
                    tool: "HF".into(),
                    item_id: "x".into(),
                },
                DupFile {
                    path: b.clone(),
                    tool: "LM".into(),
                    item_id: "y".into(),
                },
            ],
        };
        assert_eq!(merge_group(&g, &a).unwrap(), 2048);
        assert_eq!(file_identity(&a), file_identity(&b));
        assert_eq!(fs::read(&b).unwrap().len(), 2048);
        assert!(!b.with_extension("aidc-dup-tmp").exists());
    }

    #[test]
    fn shared_blob_store_is_not_its_own_duplicate() {
        // Two Ollama models point into one blobs folder.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("models");
        fs::create_dir_all(root.join("blobs")).unwrap();
        fs::create_dir_all(root.join("manifests/lib/a")).unwrap();
        fs::create_dir_all(root.join("manifests/lib/b")).unwrap();
        let blob = fs::File::create(root.join("blobs/sha256-x")).unwrap();
        blob.set_len(MIN_FILE_BYTES + 1).unwrap();
        let item = |m: &str| ScanItem {
            id: m.into(),
            rule_id: "ollama.models".into(),
            tool: "Ollama".into(),
            category: Category::Models,
            name: m.into(),
            path: root.join("manifests/lib").join(m).join("latest"),
            size_bytes: 0,
            file_count: 0,
            last_used: None,
            safety: crate::SafetyLevel::Redownload,
            consequence: String::new(),
            detail: None,
            in_use_by: None,
        };
        assert!(find_duplicates(&[item("a"), item("b")]).is_empty());
    }

    #[test]
    fn weight_file_detection() {
        assert!(is_weight_file(Path::new("x/model.gguf")));
        assert!(is_weight_file(Path::new("x/blobs/sha256-abc")));
        assert!(!is_weight_file(Path::new("x/config.json")));
    }
}
