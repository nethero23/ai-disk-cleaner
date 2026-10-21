//! Conda environments: `envs/<name>` with `conda-meta/history` recording
//! installs. The newest `conda-meta` entry is a better "last used" than file
//! mtimes, which are all set at install time.

use crate::model::ScanItem;
use crate::rules::Rule;
use crate::scan::{make_item, tree_stats, ScanOptions};
use std::path::Path;

/// Names of notable packages to surface in the detail, so "torch + cuda" is visible.
const NOTABLE: &[&str] = &[
    "torch",
    "pytorch",
    "tensorflow",
    "cuda",
    "transformers",
    "diffusers",
    "jax",
    "vllm",
    "xformers",
];

pub fn notable_packages(env: &Path) -> Vec<String> {
    let mut found = Vec::new();
    if let Ok(rd) = std::fs::read_dir(env.join("conda-meta")) {
        for e in rd.flatten() {
            let n = e.file_name().to_string_lossy().to_string();
            if !n.ends_with(".json") {
                continue;
            }
            let base = n.trim_end_matches(".json");
            // "pytorch-2.4.0-py3.12_cuda12.1_cudnn9.1.0_0" → pkg name is up to the first '-' followed by a digit
            let pkg = split_pkg_name(base);
            if NOTABLE.iter().any(|k| pkg.starts_with(k)) && !found.contains(&pkg) {
                found.push(pkg);
            }
        }
    }
    found.sort();
    found
}

fn split_pkg_name(base: &str) -> String {
    let mut parts = base.split('-').peekable();
    let mut name = String::new();
    while let Some(p) = parts.next() {
        if !name.is_empty() && p.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            break;
        }
        if !name.is_empty() {
            name.push('-');
        }
        name.push_str(p);
        if parts.peek().is_none() {
            break;
        }
    }
    name
}

pub fn envs(rule: &Rule, root: &Path, opts: &ScanOptions) -> Vec<ScanItem> {
    let mut items = Vec::new();
    let Ok(rd) = std::fs::read_dir(root) else {
        return items;
    };
    for entry in rd.flatten() {
        let p = entry.path();
        if !p.is_dir() || p.is_symlink() || !p.join("conda-meta").is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let stats = tree_stats(&p);
        let pkgs = notable_packages(&p);
        let detail = if pkgs.is_empty() {
            None
        } else if opts.lang == "en" {
            Some(format!("Contains {}", pkgs.join(", ")))
        } else {
            Some(format!("包含 {}", pkgs.join("、")))
        };
        items.push(make_item(rule, &p, name, stats, opts, detail));
    }
    items
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_names_come_out_of_conda_meta_filenames() {
        assert_eq!(split_pkg_name("pytorch-2.4.0-py3.12_cuda12.1_0"), "pytorch");
        assert_eq!(
            split_pkg_name("pytorch-cuda-12.1-ha16c6d3_5"),
            "pytorch-cuda"
        );
        assert_eq!(split_pkg_name("numpy-1.26.4-py312h0"), "numpy");
    }
}
