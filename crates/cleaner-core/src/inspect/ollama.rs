//! Ollama keeps `models/manifests/<registry>/<namespace>/<name>/<tag>` (a JSON
//! manifest) and `models/blobs/sha256-<digest>` (the layers). Several tags can
//! share layers, so a model's "size" is the sum of its layers, and deleting a
//! model must only drop blobs no other manifest references.

use super::read_small;
use crate::model::ScanItem;
use crate::rules::Rule;
use crate::scan::{make_item, ScanOptions, TreeStats};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Deserialize)]
struct Manifest {
    #[serde(default)]
    config: Option<Layer>,
    #[serde(default)]
    layers: Vec<Layer>,
}

#[derive(Deserialize)]
struct Layer {
    digest: String,
    #[serde(default)]
    size: u64,
}

#[derive(Debug, Clone)]
pub struct OllamaModel {
    /// `qwen2.5:7b`, with `registry.ollama.ai/library/` stripped.
    pub name: String,
    pub manifest: PathBuf,
    pub blobs: Vec<PathBuf>,
    pub size_bytes: u64,
    pub last_used: Option<DateTime<Utc>>,
}

fn blob_path(root: &Path, digest: &str) -> PathBuf {
    root.join("blobs").join(digest.replace(':', "-"))
}

/// Lists the models under an Ollama models root.
pub fn list(root: &Path) -> Vec<OllamaModel> {
    let manifests = root.join("manifests");
    let mut out = Vec::new();
    for entry in WalkDir::new(&manifests)
        .follow_links(false)
        .into_iter()
        .flatten()
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let Some(text) = read_small(entry.path()) else {
            continue;
        };
        let Ok(m) = serde_json::from_str::<Manifest>(&text) else {
            continue;
        };
        let rel = entry
            .path()
            .strip_prefix(&manifests)
            .unwrap_or(entry.path());
        let parts: Vec<String> = rel
            .components()
            .map(|c| c.as_os_str().to_string_lossy().to_string())
            .collect();
        // registry / namespace / name / tag  →  "name:tag" (namespace kept if not "library")
        let name = match parts.as_slice() {
            [_, ns, name, tag] if ns == "library" => format!("{name}:{tag}"),
            [_, ns, name, tag] => format!("{ns}/{name}:{tag}"),
            other => other.join("/"),
        };
        let mut blobs = Vec::new();
        let mut size = 0;
        for l in m.layers.iter().chain(m.config.iter()) {
            let p = blob_path(root, &l.digest);
            size += if l.size > 0 {
                l.size
            } else {
                std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0)
            };
            blobs.push(p);
        }
        let last_used = entry
            .metadata()
            .ok()
            .and_then(|m| m.accessed().ok().or_else(|| m.modified().ok()))
            .map(Into::into);
        out.push(OllamaModel {
            name,
            manifest: entry.path().to_path_buf(),
            blobs,
            size_bytes: size,
            last_used,
        });
    }
    out
}

/// Blobs in `blobs/` that no manifest references (aborted pulls, removed tags).
pub fn orphan_blobs(root: &Path, models: &[OllamaModel]) -> Vec<(PathBuf, u64)> {
    let referenced: std::collections::HashSet<&PathBuf> =
        models.iter().flat_map(|m| m.blobs.iter()).collect();
    let mut out = Vec::new();
    if let Ok(rd) = std::fs::read_dir(root.join("blobs")) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_file() && !referenced.contains(&p) {
                let len = e.metadata().map(|m| m.len()).unwrap_or(0);
                out.push((p, len));
            }
        }
    }
    out
}

pub fn models(rule: &Rule, root: &Path, opts: &ScanOptions) -> Vec<ScanItem> {
    let list = list(root);
    // How many manifests share each blob, so the detail can warn about shared layers.
    let mut refs: HashMap<&PathBuf, usize> = HashMap::new();
    for m in &list {
        for b in &m.blobs {
            *refs.entry(b).or_default() += 1;
        }
    }
    let mut items = Vec::new();
    for m in &list {
        let shared = m
            .blobs
            .iter()
            .filter(|b| refs.get(b).copied().unwrap_or(0) > 1)
            .count();
        let detail = if shared > 0 {
            Some(if opts.lang == "en" {
                format!(
                    "{shared} layer(s) shared with other models; only unshared layers are deleted"
                )
            } else {
                format!("有 {shared} 层与其他模型共用，删除时只移除独占的层")
            })
        } else {
            None
        };
        let stats = TreeStats {
            bytes: m.size_bytes,
            files: (m.blobs.len() + 1) as u64,
            newest: m.last_used,
        };
        // The manifest is the item's path: deleting it is what makes Ollama forget
        // the model; blobs are cleaned by the orphan pass afterwards.
        items.push(make_item(
            rule,
            &m.manifest,
            m.name.clone(),
            stats,
            opts,
            detail,
        ));
    }
    for (p, len) in orphan_blobs(root, &list) {
        let name = if opts.lang == "en" {
            "Unreferenced blob (aborted download)".to_string()
        } else {
            "无主数据块（下载中断残留）".to_string()
        };
        let stats = TreeStats {
            bytes: len,
            files: 1,
            newest: None,
        };
        let mut item = make_item(rule, &p, name, stats, opts, None);
        item.safety = crate::model::SafetyLevel::Safe;
        item.consequence = if opts.lang == "en" {
            "No model uses this file".into()
        } else {
            "没有任何模型在用这个文件".into()
        };
        items.push(item);
    }
    items
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn fake_root() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let m = root.join("manifests/registry.ollama.ai/library/qwen2.5");
        fs::create_dir_all(&m).unwrap();
        fs::create_dir_all(root.join("blobs")).unwrap();
        fs::write(root.join("blobs/sha256-aaa"), vec![0u8; 100]).unwrap();
        fs::write(root.join("blobs/sha256-bbb"), vec![0u8; 50]).unwrap();
        fs::write(root.join("blobs/sha256-orphan"), vec![0u8; 7]).unwrap();
        fs::write(
            m.join("7b"),
            r#"{"config":{"digest":"sha256:bbb","size":50},"layers":[{"digest":"sha256:aaa","size":100}]}"#,
        )
        .unwrap();
        fs::write(
            m.join("7b-q8"),
            r#"{"layers":[{"digest":"sha256:aaa","size":0}]}"#,
        )
        .unwrap();
        tmp
    }

    #[test]
    fn names_models_and_sums_layers() {
        let tmp = fake_root();
        let mut list = list(tmp.path());
        list.sort_by(|a, b| a.name.cmp(&b.name));
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].name, "qwen2.5:7b");
        assert_eq!(list[0].size_bytes, 150);
        assert_eq!(list[1].name, "qwen2.5:7b-q8");
        // size 0 in manifest falls back to the blob's real length
        assert_eq!(list[1].size_bytes, 100);
    }

    #[test]
    fn finds_orphan_blobs() {
        let tmp = fake_root();
        let list = list(tmp.path());
        let orphans = orphan_blobs(tmp.path(), &list);
        assert_eq!(orphans.len(), 1);
        assert!(orphans[0].0.ends_with("sha256-orphan"));
        assert_eq!(orphans[0].1, 7);
    }
}
