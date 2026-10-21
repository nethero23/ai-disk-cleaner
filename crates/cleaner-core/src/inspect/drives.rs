//! Signature scan of a whole drive the user picked (D:, E: …).
//!
//! Rules know where tools cache things *by default*, which is almost always
//! the system drive. Anything the user moved elsewhere, or installed on a
//! data drive, has no fixed address. So for extra drives we walk the tree
//! (bounded depth, skipping the OS and known no-go folders) and recognise
//! caches by shape: a `hub` full of `models--*`, a `pkgs` next to
//! `conda-meta`, an `.ollama/models`, a project's `node_modules`…
//!
//! Each hit is reported under a synthetic `drive.*` rule so the UI can group,
//! disable and explain it like any other item.

use crate::inspect::artifacts;
use crate::model::{Category, SafetyLevel, ScanItem};
use crate::rules::{Inspector, Rule};
use crate::scan::{make_item, tree_stats, ScanOptions};
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

/// Deeper than this and we are inside an install, not looking for caches.
const MAX_DEPTH: usize = 8;

/// A recognisable cache shape and how to describe it.
pub struct Sig {
    pub id: &'static str,
    pub tool: &'static str,
    pub category: Category,
    pub name_zh: &'static str,
    pub name_en: &'static str,
    pub safety: SafetyLevel,
    pub consequence_zh: &'static str,
    pub consequence_en: &'static str,
    pub processes: &'static [&'static str],
    pub min_bytes: u64,
}

const MB: u64 = 1024 * 1024;

const SIGS: &[Sig] = &[
    Sig {
        id: "drive.ollama",
        tool: "Ollama",
        category: Category::Models,
        name_zh: "Ollama 模型（其他盘）",
        name_en: "Ollama models (other drive)",
        safety: SafetyLevel::Redownload,
        consequence_zh: "删除后需要重新 ollama pull，共 {size}",
        consequence_en: "Models must be pulled again, {size}",
        processes: &["ollama"],
        min_bytes: 0,
    },
    Sig {
        id: "drive.hf_hub",
        tool: "Hugging Face",
        category: Category::Models,
        name_zh: "Hugging Face hub 缓存（其他盘）",
        name_en: "Hugging Face hub cache (other drive)",
        safety: SafetyLevel::Redownload,
        consequence_zh: "删除后模型会重新下载，共 {size}",
        consequence_en: "Models download again, {size}",
        processes: &[],
        min_bytes: 0,
    },
    Sig {
        id: "drive.comfyui_models",
        tool: "ComfyUI",
        category: Category::Models,
        name_zh: "ComfyUI 模型目录",
        name_en: "ComfyUI models folder",
        safety: SafetyLevel::Review,
        consequence_zh: "包含你放进去的 checkpoint / LoRA，删除后需重新下载，共 {size}",
        consequence_en: "Holds checkpoints / LoRAs you added; re-download after deleting, {size}",
        processes: &[],
        min_bytes: 0,
    },
    Sig {
        id: "drive.sdwebui_models",
        tool: "SD WebUI",
        category: Category::Models,
        name_zh: "Stable Diffusion WebUI 模型目录",
        name_en: "Stable Diffusion WebUI models folder",
        safety: SafetyLevel::Review,
        consequence_zh: "包含你放进去的模型，删除后需重新下载，共 {size}",
        consequence_en: "Holds models you added; re-download after deleting, {size}",
        processes: &[],
        min_bytes: 0,
    },
    Sig {
        id: "drive.conda_pkgs",
        tool: "Conda",
        category: Category::PackageCache,
        name_zh: "Conda 包缓存（其他盘）",
        name_en: "Conda package cache (other drive)",
        safety: SafetyLevel::Safe,
        consequence_zh: "下次安装包时重新下载，共 {size}",
        consequence_en: "Packages download again on next install, {size}",
        processes: &[],
        min_bytes: 0,
    },
    Sig {
        id: "drive.conda_envs",
        tool: "Conda",
        category: Category::PythonEnv,
        name_zh: "Conda 环境目录（其他盘）",
        name_en: "Conda environments (other drive)",
        safety: SafetyLevel::Review,
        consequence_zh: "里面是你创建的环境，删除后需要重建，共 {size}",
        consequence_en: "Your environments live here; recreate after deleting, {size}",
        processes: &[],
        min_bytes: 0,
    },
    Sig {
        id: "drive.pip_cache",
        tool: "pip",
        category: Category::PackageCache,
        name_zh: "pip 缓存（其他盘）",
        name_en: "pip cache (other drive)",
        safety: SafetyLevel::Safe,
        consequence_zh: "下次 pip install 重新下载，共 {size}",
        consequence_en: "Downloads again on next pip install, {size}",
        processes: &[],
        min_bytes: 0,
    },
    Sig {
        id: "drive.uv_cache",
        tool: "uv",
        category: Category::PackageCache,
        name_zh: "uv 缓存（其他盘）",
        name_en: "uv cache (other drive)",
        safety: SafetyLevel::Safe,
        consequence_zh: "下次 uv 安装重新下载，共 {size}",
        consequence_en: "Downloads again on next uv install, {size}",
        processes: &[],
        min_bytes: 0,
    },
    Sig {
        id: "drive.hf_cache",
        tool: "Hugging Face",
        category: Category::Models,
        name_zh: "Hugging Face 缓存（其他盘）",
        name_en: "Hugging Face cache (other drive)",
        safety: SafetyLevel::Redownload,
        consequence_zh: "删除后模型会重新下载，共 {size}",
        consequence_en: "Models download again, {size}",
        processes: &[],
        min_bytes: 0,
    },
    Sig {
        id: "drive.torch_cache",
        tool: "PyTorch",
        category: Category::Models,
        name_zh: "PyTorch hub 缓存（其他盘）",
        name_en: "PyTorch hub cache (other drive)",
        safety: SafetyLevel::Redownload,
        consequence_zh: "删除后权重会重新下载，共 {size}",
        consequence_en: "Weights download again, {size}",
        processes: &[],
        min_bytes: 0,
    },
    Sig {
        id: "drive.playwright",
        tool: "Playwright",
        category: Category::PackageCache,
        name_zh: "Playwright 浏览器（其他盘）",
        name_en: "Playwright browsers (other drive)",
        safety: SafetyLevel::Redownload,
        consequence_zh: "下次 playwright install 重新下载，共 {size}",
        consequence_en: "Downloads again on next playwright install, {size}",
        processes: &[],
        min_bytes: 0,
    },
    Sig {
        id: "drive.npm_cache",
        tool: "npm",
        category: Category::PackageCache,
        name_zh: "npm 缓存（其他盘）",
        name_en: "npm cache (other drive)",
        safety: SafetyLevel::Safe,
        consequence_zh: "下次 npm install 重新下载，共 {size}",
        consequence_en: "Downloads again on next npm install, {size}",
        processes: &[],
        min_bytes: 0,
    },
    Sig {
        id: "drive.pnpm_store",
        tool: "pnpm",
        category: Category::PackageCache,
        name_zh: "pnpm 存储（其他盘）",
        name_en: "pnpm store (other drive)",
        safety: SafetyLevel::Safe,
        consequence_zh: "下次 pnpm install 重新下载，共 {size}",
        consequence_en: "Downloads again on next pnpm install, {size}",
        processes: &[],
        min_bytes: 0,
    },
    Sig {
        id: "drive.gradle_caches",
        tool: "Gradle",
        category: Category::PackageCache,
        name_zh: "Gradle 缓存（其他盘）",
        name_en: "Gradle caches (other drive)",
        safety: SafetyLevel::Safe,
        consequence_zh: "下次构建重新下载依赖，共 {size}",
        consequence_en: "Dependencies download again on next build, {size}",
        processes: &[],
        min_bytes: 0,
    },
    Sig {
        id: "drive.nuget",
        tool: "NuGet",
        category: Category::PackageCache,
        name_zh: "NuGet 包缓存（其他盘）",
        name_en: "NuGet package cache (other drive)",
        safety: SafetyLevel::Safe,
        consequence_zh: "下次还原时重新下载，共 {size}",
        consequence_en: "Packages restore again, {size}",
        processes: &[],
        min_bytes: 0,
    },
    Sig {
        id: "drive.maven",
        tool: "Maven",
        category: Category::PackageCache,
        name_zh: "Maven 仓库缓存（其他盘）",
        name_en: "Maven repository (other drive)",
        safety: SafetyLevel::Safe,
        consequence_zh: "下次构建重新下载依赖，共 {size}",
        consequence_en: "Dependencies download again on next build, {size}",
        processes: &[],
        min_bytes: 0,
    },
    Sig {
        id: "drive.cargo_registry",
        tool: "Cargo",
        category: Category::PackageCache,
        name_zh: "Cargo registry 缓存（其他盘）",
        name_en: "Cargo registry (other drive)",
        safety: SafetyLevel::Safe,
        consequence_zh: "下次构建重新下载 crate，共 {size}",
        consequence_en: "Crates download again on next build, {size}",
        processes: &[],
        min_bytes: 0,
    },
    Sig {
        id: "drive.model_files",
        tool: "本地模型",
        category: Category::Models,
        name_zh: "模型文件目录 (.gguf / .safetensors)",
        name_en: "Model files folder (.gguf / .safetensors)",
        safety: SafetyLevel::Review,
        consequence_zh: "看起来是手动下载的模型，删了要重新下载，共 {size}",
        consequence_en: "Looks like manually downloaded models; re-download after deleting, {size}",
        processes: &[],
        min_bytes: 100 * MB,
    },
];

fn sig(id: &str) -> &'static Sig {
    SIGS.iter()
        .find(|s| s.id == id)
        .expect("known signature id")
}

/// A rule object for a signature, so items carry the usual fields.
pub fn rule_for(s: &Sig) -> Rule {
    Rule {
        id: s.id.into(),
        tool: s.tool.into(),
        category: s.category,
        name_zh: s.name_zh.into(),
        name_en: s.name_en.into(),
        paths: vec![],
        all_paths: false,
        safety: s.safety,
        inspector: Inspector::Plain,
        consequence_zh: s.consequence_zh.into(),
        consequence_en: s.consequence_en.into(),
        min_bytes: s.min_bytes,
        windows_only: false,
        require: vec![],
        processes: s.processes.iter().map(|p| p.to_string()).collect(),
    }
}

/// All synthetic rules, so callers can list/disable them like real ones.
pub fn rules() -> Vec<Rule> {
    SIGS.iter().map(rule_for).collect()
}

fn has_child(dir: &Path, name: &str) -> bool {
    dir.join(name).exists()
}

fn has_child_starting(dir: &Path, prefix: &str) -> bool {
    std::fs::read_dir(dir)
        .map(|rd| {
            rd.flatten()
                .any(|e| e.file_name().to_string_lossy().starts_with(prefix))
        })
        .unwrap_or(false)
}

/// True when the folder holds model weights within two levels. Samples at
/// most a few hundred entries so a huge folder does not stall the walk.
fn holds_model_files(dir: &Path) -> bool {
    let mut seen = 0usize;
    for e in WalkDir::new(dir)
        .follow_links(false)
        .max_depth(2)
        .into_iter()
        .flatten()
    {
        seen += 1;
        if seen > 400 {
            break;
        }
        if !e.file_type().is_file() {
            continue;
        }
        let name = e.file_name().to_string_lossy().to_ascii_lowercase();
        if name.ends_with(".gguf") || name.ends_with(".safetensors") || name.ends_with(".ckpt") {
            return true;
        }
    }
    false
}

/// What a folder is, if it is a known cache: the signature and the path to
/// report (usually the folder itself, sometimes a child).
pub fn classify(path: &Path) -> Vec<(&'static Sig, PathBuf)> {
    let Some(name) = path.file_name().map(|n| n.to_string_lossy().to_string()) else {
        return vec![];
    };
    let parent = path.parent().unwrap_or(path);
    let lower = name.to_ascii_lowercase();
    let mut out = Vec::new();
    match lower.as_str() {
        ".ollama" if has_child(path, "models") => {
            out.push((sig("drive.ollama"), path.join("models")))
        }
        "hub" if has_child_starting(path, "models--") => {
            out.push((sig("drive.hf_hub"), path.to_path_buf()))
        }
        "models" => {
            let pname = parent
                .file_name()
                .map(|n| n.to_string_lossy().to_ascii_lowercase())
                .unwrap_or_default();
            if has_child(parent, "main.py") && pname.contains("comfy") {
                out.push((sig("drive.comfyui_models"), path.to_path_buf()));
            } else if has_child(parent, "webui.py")
                || has_child(parent, "launch.py") && has_child(parent, "webui-user.bat")
            {
                out.push((sig("drive.sdwebui_models"), path.to_path_buf()));
            } else if holds_model_files(path) {
                out.push((sig("drive.model_files"), path.to_path_buf()));
            }
        }
        "pkgs" if has_child(parent, "conda-meta") => {
            out.push((sig("drive.conda_pkgs"), path.to_path_buf()))
        }
        "envs" if has_child(parent, "conda-meta") => {
            out.push((sig("drive.conda_envs"), path.to_path_buf()))
        }
        ".cache" => {
            for (child, id) in [
                ("pip", "drive.pip_cache"),
                ("uv", "drive.uv_cache"),
                ("huggingface", "drive.hf_cache"),
                ("torch", "drive.torch_cache"),
                ("ms-playwright", "drive.playwright"),
            ] {
                if has_child(path, child) {
                    out.push((sig(id), path.join(child)));
                }
            }
        }
        "npm-cache" => out.push((sig("drive.npm_cache"), path.to_path_buf())),
        "_cacache" => out.push((sig("drive.npm_cache"), path.to_path_buf())),
        ".pnpm-store" => out.push((sig("drive.pnpm_store"), path.to_path_buf())),
        ".gradle" if has_child(path, "caches") => {
            out.push((sig("drive.gradle_caches"), path.join("caches")))
        }
        ".nuget" if has_child(path, "packages") => {
            out.push((sig("drive.nuget"), path.join("packages")))
        }
        ".m2" if has_child(path, "repository") => {
            out.push((sig("drive.maven"), path.join("repository")))
        }
        ".cargo" if has_child(path, "registry") => {
            out.push((sig("drive.cargo_registry"), path.join("registry")))
        }
        _ => {}
    }
    out
}

fn should_skip(entry: &DirEntry) -> bool {
    let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
    matches!(
        name.as_str(),
        "$recycle.bin"
            | "system volume information"
            | "windows"
            | "program files"
            | "program files (x86)"
            | "programdata"
            | "recovery"
            | "windows.old"
    ) || artifacts::should_skip(entry)
}

/// Walks `root` and returns every recognised cache and project artifact.
/// `artifacts_rule` is the `projects.artifacts` rule when enabled.
pub fn scan_drive(root: &Path, artifacts_rule: Option<&Rule>, opts: &ScanOptions) -> Vec<ScanItem> {
    let mut items = Vec::new();
    let excluded = |p: &Path| opts.excluded.iter().any(|ex| p.starts_with(ex));
    let mut it = WalkDir::new(root)
        .follow_links(false)
        .max_depth(MAX_DEPTH)
        .into_iter();
    while let Some(next) = it.next() {
        let Ok(entry) = next else { continue };
        if entry.depth() == 0 || !entry.file_type().is_dir() {
            continue;
        }
        if excluded(entry.path()) {
            it.skip_current_dir();
            continue;
        }
        let hits = classify(entry.path());
        if !hits.is_empty() {
            for (s, path) in hits {
                let rule = rule_for(s);
                let stats = tree_stats(&path);
                if stats.bytes < rule.min_bytes.max(opts.global_min_bytes) || excluded(&path) {
                    continue;
                }
                let name = if opts.lang == "en" {
                    rule.name_en.clone()
                } else {
                    rule.name_zh.clone()
                };
                items.push(make_item(&rule, &path, name, stats, opts, None));
            }
            it.skip_current_dir();
            continue;
        }
        if let Some(rule) = artifacts_rule {
            if artifacts::is_artifact_dir(&entry) {
                let item = artifacts::item_for(rule, entry.path(), opts);
                if item.size_bytes >= rule.min_bytes.max(opts.global_min_bytes) {
                    items.push(item);
                }
                it.skip_current_dir();
                continue;
            }
        }
        if should_skip(&entry) {
            it.skip_current_dir();
        }
    }
    items
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::PathContext;
    use std::collections::HashMap;
    use std::fs;

    fn opts() -> ScanOptions {
        ScanOptions {
            ctx: PathContext::with_vars(HashMap::new()),
            lang: "zh".into(),
            global_min_bytes: 0,
            running: Default::default(),
            excluded: Vec::new(),
            extra_roots: Vec::new(),
        }
    }

    #[test]
    fn recognises_caches_by_shape_on_a_data_drive() {
        let tmp = tempfile::tempdir().unwrap();
        let d = tmp.path();
        fs::create_dir_all(d.join("tools/.ollama/models/blobs")).unwrap();
        fs::write(
            d.join("tools/.ollama/models/blobs/sha256-1"),
            vec![0u8; 300],
        )
        .unwrap();
        fs::create_dir_all(d.join("hf/hub/models--org--name/blobs")).unwrap();
        fs::write(d.join("hf/hub/models--org--name/blobs/x"), vec![0u8; 200]).unwrap();
        fs::create_dir_all(d.join("miniconda/pkgs/numpy")).unwrap();
        fs::create_dir_all(d.join("miniconda/conda-meta")).unwrap();
        fs::write(d.join("miniconda/pkgs/numpy/a"), vec![0u8; 100]).unwrap();
        // A plain "models" folder with nothing model-like inside is ignored.
        fs::create_dir_all(d.join("misc/models")).unwrap();
        fs::write(d.join("misc/models/readme.txt"), b"hi").unwrap();
        // A "hub" without models--* is not Hugging Face.
        fs::create_dir_all(d.join("other/hub/stuff")).unwrap();
        fs::write(d.join("other/hub/stuff/f"), vec![0u8; 50]).unwrap();
        // Recycle bin is never entered.
        fs::create_dir_all(d.join("$RECYCLE.BIN/x/.ollama/models")).unwrap();
        fs::write(d.join("$RECYCLE.BIN/x/.ollama/models/f"), vec![0u8; 50]).unwrap();

        let items = scan_drive(d, None, &opts());
        let mut ids: Vec<&str> = items.iter().map(|i| i.rule_id.as_str()).collect();
        ids.sort();
        assert_eq!(
            ids,
            vec!["drive.conda_pkgs", "drive.hf_hub", "drive.ollama"]
        );
        let ollama = items.iter().find(|i| i.rule_id == "drive.ollama").unwrap();
        assert_eq!(ollama.size_bytes, 300);
        assert!(ollama.path.ends_with("models"));
    }

    #[test]
    fn project_artifacts_are_found_on_the_drive_too() {
        let tmp = tempfile::tempdir().unwrap();
        let d = tmp.path();
        fs::create_dir_all(d.join("code/app/node_modules/x")).unwrap();
        fs::write(d.join("code/app/package.json"), "{}").unwrap();
        fs::write(d.join("code/app/node_modules/x/a.js"), vec![0u8; 500]).unwrap();
        let rule = rule_for(sig("drive.ollama"));
        let rule = Rule {
            id: "projects.artifacts".into(),
            min_bytes: 0,
            ..rule
        };
        let items = scan_drive(d, Some(&rule), &opts());
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].rule_id, "projects.artifacts");
        assert!(items[0].path.ends_with("node_modules"));
        // Without the rule (disabled), nothing.
        assert!(scan_drive(d, None, &opts()).is_empty());
    }
}
