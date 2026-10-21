//! Skills (folders with a SKILL.md), plugins and extensions on disk, each
//! with a content hash so copies can be told from diverged versions.

use super::mcp::{servers_in_manifest, McpServer};
use crate::scan::tree_stats;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Skill {
    pub tool: String,
    /// Folder name, which is how tools address the skill.
    pub name: String,
    pub path: PathBuf,
    pub bytes: u64,
    pub files: u64,
    pub modified: Option<DateTime<Utc>>,
    /// sha256 over relative paths and contents, so identical copies match.
    pub hash: String,
    /// First line of SKILL.md's `description:` front matter, if any.
    pub description: Option<String>,
    /// A junction/symlink into another skills folder: not a copy.
    pub link: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Plugin {
    pub tool: String,
    pub name: String,
    pub version: Option<String>,
    pub path: PathBuf,
    pub bytes: u64,
    /// Older versions of the same plugin still on disk (cache folders).
    pub stale_versions: Vec<PathBuf>,
    pub stale_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Extension {
    pub tool: String,
    pub name: String,
    pub version: Option<String>,
    pub path: PathBuf,
    pub bytes: u64,
    /// Older versions of the same extension still on disk.
    pub stale_versions: Vec<PathBuf>,
    pub stale_bytes: u64,
}

/// Files bigger than this are hashed by size and name only, so a skill that
/// ships a model file does not stall the inventory.
const HASH_FULL_LIMIT: u64 = 4 * 1024 * 1024;

/// Content hash of a folder: relative paths (sorted) plus contents.
pub fn dir_hash(root: &Path) -> String {
    let mut entries: Vec<(String, PathBuf, u64)> = WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .flatten()
        .filter(|e| e.file_type().is_file())
        .map(|e| {
            let rel = e
                .path()
                .strip_prefix(root)
                .unwrap_or(e.path())
                .to_string_lossy()
                .replace('\\', "/");
            let len = e.metadata().map(|m| m.len()).unwrap_or(0);
            (rel, e.into_path(), len)
        })
        .collect();
    entries.sort();
    let mut h = Sha256::new();
    for (rel, path, len) in entries {
        h.update(rel.as_bytes());
        h.update([0]);
        if len <= HASH_FULL_LIMIT {
            if let Ok(bytes) = std::fs::read(&path) {
                h.update(&bytes);
            }
        } else {
            h.update(len.to_le_bytes());
        }
        h.update([0]);
    }
    hex(&h.finalize())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn description_of(skill_md: &Path) -> Option<String> {
    let text = std::fs::read_to_string(skill_md).ok()?;
    let mut lines = text.lines();
    if lines.next()?.trim() != "---" {
        return None;
    }
    for l in lines {
        if l.trim() == "---" {
            break;
        }
        if let Some(rest) = l.strip_prefix("description:") {
            let d = rest.trim().trim_matches('"').trim_matches('\'').to_string();
            if !d.is_empty() {
                return Some(d.chars().take(200).collect());
            }
        }
    }
    None
}

pub(crate) fn sorted_subdirs(dir: &Path) -> Vec<PathBuf> {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut v: Vec<PathBuf> = rd
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    v.sort();
    v
}

/// Every subfolder of `dir` that holds a SKILL.md (any case).
pub fn list_skills(tool: &str, dir: &Path) -> Vec<Skill> {
    let mut out = Vec::new();
    for p in sorted_subdirs(dir) {
        let Some(md) = ["SKILL.md", "skill.md", "Skill.md"]
            .iter()
            .map(|n| p.join(n))
            .find(|f| f.is_file())
        else {
            continue;
        };
        let link = p.is_symlink() || is_junction(&p);
        let st = tree_stats(&p);
        out.push(Skill {
            tool: tool.to_string(),
            name: p
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default(),
            path: p.clone(),
            bytes: st.bytes,
            files: st.files,
            modified: st.newest,
            hash: if link { String::new() } else { dir_hash(&p) },
            description: description_of(&md),
            link,
        });
    }
    out
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
fn is_junction(_p: &Path) -> bool {
    false
}

/// VS Code-style platform suffixes: `ms-vscode.cpptools-1.20.5-win32-x64`.
const PLATFORM_SUFFIXES: &[&str] = &[
    "-win32-x64",
    "-win32-arm64",
    "-win32-ia32",
    "-darwin-x64",
    "-darwin-arm64",
    "-linux-x64",
    "-linux-arm64",
    "-linux-armhf",
    "-alpine-x64",
    "-alpine-arm64",
    "-universal",
    "-web",
];

/// Splits `name@1.2.3` / `name-1.2.3` / `name_1.2.3` into name and version.
/// A platform suffix is dropped first, and a pure build number of three or
/// more digits (`chromium-1140`, `ffmpeg-1010`) also counts as a version.
pub fn split_name_version(dir_name: &str) -> (String, Option<String>) {
    let mut base = dir_name;
    for suf in PLATFORM_SUFFIXES {
        if let Some(b) = base.strip_suffix(suf) {
            base = b;
            break;
        }
    }
    if let Some((n, v)) = base.rsplit_once('@') {
        if !v.is_empty() && v.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            return (n.to_string(), Some(v.to_string()));
        }
    }
    for sep in ['-', '_'] {
        if let Some((n, v)) = base.rsplit_once(sep) {
            if looks_like_version(v) || (v.len() >= 3 && v.chars().all(|c| c.is_ascii_digit())) {
                return (n.to_string(), Some(v.to_string()));
            }
        }
    }
    (dir_name.to_string(), None)
}

fn looks_like_version(v: &str) -> bool {
    let v = v.trim_start_matches('v');
    v.chars().next().is_some_and(|c| c.is_ascii_digit())
        && v.contains('.')
        && v.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '+')
}

/// Orders version strings numerically where possible ("1.10.0" > "1.9.2").
pub fn version_key(v: &str) -> Vec<u64> {
    v.trim_start_matches('v')
        .split(['.', '-', '+'])
        .map(|part| {
            part.chars()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
        })
        .map(|digits| digits.parse().unwrap_or(0))
        .collect()
}

/// (name, (newest folder, its version), older folders).
pub(crate) type VersionGroup = (String, (PathBuf, Option<String>), Vec<PathBuf>);

/// Groups versioned folders by name; returns (name, newest, older...).
pub(crate) fn group_versions(dirs: Vec<PathBuf>) -> Vec<VersionGroup> {
    let mut groups: std::collections::BTreeMap<String, Vec<(PathBuf, Option<String>)>> =
        Default::default();
    for d in dirs {
        let name = d
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let (n, v) = split_name_version(&name);
        groups.entry(n).or_default().push((d, v));
    }
    groups
        .into_iter()
        .map(|(name, mut vs)| {
            // Newest version first; when unversioned, the most recently modified.
            vs.sort_by(|a, b| {
                let ka = a.1.as_deref().map(version_key);
                let kb = b.1.as_deref().map(version_key);
                match (ka, kb) {
                    (Some(x), Some(y)) => y.cmp(&x),
                    _ => {
                        let ma = std::fs::metadata(&a.0).and_then(|m| m.modified()).ok();
                        let mb = std::fs::metadata(&b.0).and_then(|m| m.modified()).ok();
                        mb.cmp(&ma)
                    }
                }
            });
            let newest = vs.remove(0);
            let older = vs.into_iter().map(|v| v.0).collect();
            (name, newest, older)
        })
        .collect()
}

/// Plugins of Claude Code's shape: `<dir>/cache/<marketplace>/<plugin>/<version>/`
/// (or `<dir>/<plugin>/` for simpler layouts). Older version folders are
/// reported as stale so the cleanup rules can offer them.
pub fn list_plugins(tool: &str, dir: &Path) -> Vec<Plugin> {
    let mut out = Vec::new();
    let cache = dir.join("cache");
    if cache.is_dir() {
        for market in sorted_subdirs(&cache) {
            for plugin in sorted_subdirs(&market) {
                let versions = sorted_subdirs(&plugin);
                if versions.is_empty() {
                    continue;
                }
                let name = format!(
                    "{}@{}",
                    plugin
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default(),
                    market
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default()
                );
                let mut sorted: Vec<(PathBuf, String)> = versions
                    .into_iter()
                    .map(|v| {
                        let s = v
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default();
                        (v, s)
                    })
                    .collect();
                sorted.sort_by_key(|(_, v)| std::cmp::Reverse(version_key(v)));
                let (newest, ver) = sorted.remove(0);
                let st = tree_stats(&newest);
                let stale_bytes = sorted.iter().map(|(p, _)| tree_stats(p).bytes).sum();
                out.push(Plugin {
                    tool: tool.to_string(),
                    name,
                    version: Some(ver),
                    path: newest,
                    bytes: st.bytes,
                    stale_versions: sorted.into_iter().map(|(p, _)| p).collect(),
                    stale_bytes,
                });
            }
        }
        return out;
    }
    // Flat layout: one folder per plugin, possibly versioned in the name.
    let dirs: Vec<PathBuf> = sorted_subdirs(dir)
        .into_iter()
        .filter(|p| {
            !matches!(
                p.file_name().and_then(|n| n.to_str()),
                Some("marketplaces") | Some("cache")
            )
        })
        .collect();
    for (name, (newest, ver), older) in group_versions(dirs) {
        let st = tree_stats(&newest);
        let stale_bytes = older.iter().map(|p| tree_stats(p).bytes).sum();
        out.push(Plugin {
            tool: tool.to_string(),
            name,
            version: ver,
            path: newest,
            bytes: st.bytes,
            stale_versions: older,
            stale_bytes,
        });
    }
    out
}

fn manifest_in(dir: &Path) -> Option<(serde_json::Value, PathBuf)> {
    for name in [
        "manifest.json",
        "gemini-extension.json",
        "package.json",
        "plugin.json",
    ] {
        let p = dir.join(name);
        if let Ok(text) = std::fs::read_to_string(&p) {
            if let Ok(v) =
                serde_json::from_str::<serde_json::Value>(&super::mcp::strip_jsonc(&text))
            {
                return Some((v, p));
            }
        }
    }
    None
}

/// Extensions (VS Code-style `publisher.name-1.2.3` folders, Claude Desktop
/// `.mcpb` folders, Gemini extensions). MCP servers declared inside a
/// manifest are returned too, scoped `extension:<name>`.
pub fn list_extensions(tool: &str, dir: &Path) -> (Vec<Extension>, Vec<McpServer>) {
    let mut exts = Vec::new();
    let mut servers = Vec::new();
    for (name, (newest, ver), older) in group_versions(sorted_subdirs(dir)) {
        let st = tree_stats(&newest);
        let stale_bytes = older.iter().map(|p| tree_stats(p).bytes).sum();
        let mut version = ver;
        let mut display = name.clone();
        if let Some((manifest, mpath)) = manifest_in(&newest) {
            if let Some(v) = manifest.get("version").and_then(|v| v.as_str()) {
                version = Some(v.to_string());
            }
            if let Some(n) = manifest
                .get("displayName")
                .or_else(|| manifest.get("display_name"))
                .or_else(|| manifest.get("name"))
                .and_then(|v| v.as_str())
            {
                display = n.to_string();
            }
            servers.extend(servers_in_manifest(
                tool,
                &mpath,
                &format!("extension:{display}"),
                &manifest,
            ));
        }
        exts.push(Extension {
            tool: tool.to_string(),
            name: display,
            version,
            path: newest,
            bytes: st.bytes,
            stale_versions: older,
            stale_bytes,
        });
    }
    (exts, servers)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_skill(root: &Path, name: &str, body: &str) -> PathBuf {
        let p = root.join(name);
        std::fs::create_dir_all(&p).unwrap();
        std::fs::write(p.join("SKILL.md"), body).unwrap();
        p
    }

    #[test]
    fn identical_copies_hash_the_same_and_edits_differ() {
        let tmp = tempfile::tempdir().unwrap();
        let a = tmp.path().join("a");
        let b = tmp.path().join("b");
        let body = "---\nname: web\ndescription: \"Design guidelines\"\n---\n# hi\n";
        mk_skill(&a, "web-design", body);
        mk_skill(&b, "web-design", body);
        let sa = list_skills("x", &a);
        let sb = list_skills("y", &b);
        assert_eq!(sa.len(), 1);
        assert_eq!(sa[0].hash, sb[0].hash);
        assert_eq!(sa[0].description.as_deref(), Some("Design guidelines"));
        std::fs::write(b.join("web-design").join("extra.txt"), "x").unwrap();
        assert_ne!(sa[0].hash, list_skills("y", &b)[0].hash);
        // A folder without SKILL.md is not a skill.
        std::fs::create_dir_all(a.join("notes")).unwrap();
        assert_eq!(list_skills("x", &a).len(), 1);
    }

    #[test]
    fn plugin_cache_versions_sorted_and_stale_reported() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("plugins");
        for v in ["1.9.0", "1.10.0", "1.2.3"] {
            let p = root.join("cache/market/my-plugin").join(v);
            std::fs::create_dir_all(&p).unwrap();
            std::fs::write(p.join("f"), vec![0u8; 100]).unwrap();
        }
        let ps = list_plugins("claude_code", &root);
        assert_eq!(ps.len(), 1);
        assert_eq!(ps[0].name, "my-plugin@market");
        assert_eq!(ps[0].version.as_deref(), Some("1.10.0"));
        assert_eq!(ps[0].stale_versions.len(), 2);
        assert_eq!(ps[0].stale_bytes, 200);
    }

    #[test]
    fn extension_folders_group_by_name() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("extensions");
        for d in [
            "ms-python.python-2024.1.0",
            "ms-python.python-2024.3.1",
            "other.ext-1.0.0",
        ] {
            let p = root.join(d);
            std::fs::create_dir_all(&p).unwrap();
            std::fs::write(p.join("x"), "1").unwrap();
        }
        std::fs::write(
            root.join("other.ext-1.0.0/package.json"),
            r#"{"name": "other.ext", "displayName": "Other", "version": "1.0.0", "mcpServers": {"s": {"command": "npx", "args": ["pkg"]}}}"#,
        )
        .unwrap();
        let (exts, servers) = list_extensions("cursor", &root);
        assert_eq!(exts.len(), 2);
        let py = exts.iter().find(|e| e.name == "ms-python.python").unwrap();
        assert_eq!(py.version.as_deref(), Some("2024.3.1"));
        assert_eq!(py.stale_versions.len(), 1);
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].scope, "extension:Other");
    }

    #[test]
    fn version_parsing() {
        assert_eq!(
            split_name_version("pkg@1.2.3"),
            ("pkg".into(), Some("1.2.3".into()))
        );
        assert_eq!(
            split_name_version("a.b-1.0.0"),
            ("a.b".into(), Some("1.0.0".into()))
        );
        assert_eq!(split_name_version("plain"), ("plain".into(), None));
        assert_eq!(
            split_name_version("ms-vscode.cpptools-1.20.5-win32-x64"),
            ("ms-vscode.cpptools".into(), Some("1.20.5".into()))
        );
        assert_eq!(
            split_name_version("chromium_headless_shell-1140"),
            ("chromium_headless_shell".into(), Some("1140".into()))
        );
        assert_eq!(split_name_version("ext-2"), ("ext-2".into(), None));
        assert!(version_key("1.10.0") > version_key("1.9.2"));
    }
}
