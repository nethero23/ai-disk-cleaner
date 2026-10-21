//! Exporting chat history before it is cleaned.
//!
//! Transcripts from Claude Code, Codex, Gemini CLI and the AI editors are
//! the one kind of "junk" a user may regret losing. Before such Review items
//! go to the Recycle Bin, they can be packed into one zip, so the space is
//! freed but the conversations remain readable elsewhere.

use crate::model::ScanItem;
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Rule ids whose items are conversation history worth keeping a copy of.
pub const EXPORTABLE_RULES: &[&str] = &[
    "claude_code.projects",
    "codex.sessions",
    "gemini_cli.sessions",
    "cursor.workspaces",
    "cursor.global_state",
    "windsurf.workspaces",
    "vscode.workspaces",
];

pub fn is_exportable(rule_id: &str) -> bool {
    EXPORTABLE_RULES.contains(&rule_id)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    pub zip_path: PathBuf,
    pub files: u64,
    pub bytes: u64,
}

/// Progress callback: (bytes done, bytes total).
pub type Progress<'a> = &'a mut dyn FnMut(u64, u64);

fn safe_component(s: &str) -> String {
    let cleaned: String = s
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect();
    let trimmed = cleaned.trim_matches(|c| c == '.' || c == ' ');
    if trimmed.is_empty() {
        "item".into()
    } else {
        trimmed.chars().take(80).collect()
    }
}

/// Packs every item (its whole folder, or the file itself) into one zip at
/// `dest_dir/ai-chat-history-<timestamp>.zip`, laid out as `tool/name/...`.
pub fn export(items: &[ScanItem], dest_dir: &Path, progress: Progress) -> Result<ExportResult> {
    if !dest_dir.is_dir() {
        return Err(Error::Refused(
            dest_dir.to_path_buf(),
            "destination folder does not exist".into(),
        ));
    }
    let zip_path = dest_dir.join(format!(
        "ai-chat-history-{}.zip",
        chrono::Local::now().format("%Y%m%d-%H%M%S")
    ));
    let io = |path: &Path| {
        let path = path.to_path_buf();
        move |source| Error::Io { path, source }
    };
    let total: u64 = items.iter().map(|i| i.size_bytes).sum();
    let file = std::fs::File::create(&zip_path).map_err(io(&zip_path))?;
    let mut zip = zip::ZipWriter::new(std::io::BufWriter::new(file));
    let opts = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .large_file(true);
    let mut files = 0u64;
    let mut bytes = 0u64;
    let mut buf = vec![0u8; 1 << 20];
    let mut used_prefixes = std::collections::HashSet::new();
    for item in items {
        let mut prefix = format!(
            "{}/{}",
            safe_component(&item.tool),
            safe_component(&item.name)
        );
        // Two items can share a display name (e.g. two "src" projects).
        let mut n = 2;
        while !used_prefixes.insert(prefix.clone()) {
            prefix = format!(
                "{}/{} ({n})",
                safe_component(&item.tool),
                safe_component(&item.name)
            );
            n += 1;
        }
        let base = if item.path.is_dir() {
            item.path.clone()
        } else {
            item.path
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_default()
        };
        for e in WalkDir::new(&item.path)
            .follow_links(false)
            .into_iter()
            .flatten()
        {
            if !e.file_type().is_file() {
                continue;
            }
            let rel = e.path().strip_prefix(&base).unwrap_or(e.path());
            let name = format!("{prefix}/{}", rel.to_string_lossy().replace('\\', "/"));
            zip.start_file(name, opts)
                .map_err(|e| zip_err(&zip_path, e))?;
            let mut f = std::fs::File::open(e.path()).map_err(io(e.path()))?;
            loop {
                let n = f.read(&mut buf).map_err(io(e.path()))?;
                if n == 0 {
                    break;
                }
                zip.write_all(&buf[..n]).map_err(io(&zip_path))?;
                bytes += n as u64;
                progress(bytes.min(total), total);
            }
            files += 1;
        }
    }
    zip.finish()
        .map_err(|e| zip_err(&zip_path, e))?
        .flush()
        .map_err(io(&zip_path))?;
    progress(total, total);
    Ok(ExportResult {
        zip_path,
        files,
        bytes,
    })
}

fn zip_err(path: &Path, e: zip::result::ZipError) -> Error {
    Error::Io {
        path: path.to_path_buf(),
        source: std::io::Error::other(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Category;
    use std::fs;

    fn item(name: &str, path: PathBuf, size: u64) -> ScanItem {
        ScanItem {
            id: name.into(),
            rule_id: "claude_code.projects".into(),
            tool: "Claude Code".into(),
            category: Category::AiAgent,
            name: name.into(),
            path,
            size_bytes: size,
            file_count: 0,
            last_used: None,
            safety: crate::SafetyLevel::Review,
            consequence: String::new(),
            detail: None,
            in_use_by: None,
            auto_clean: true,
        }
    }

    #[test]
    fn packs_items_under_tool_and_name() {
        let tmp = tempfile::tempdir().unwrap();
        let a = tmp.path().join("proj-a");
        fs::create_dir_all(a.join("sub")).unwrap();
        fs::write(a.join("one.jsonl"), b"hello").unwrap();
        fs::write(a.join("sub/two.jsonl"), b"world!").unwrap();
        let b = tmp.path().join("proj-b");
        fs::create_dir_all(&b).unwrap();
        fs::write(b.join("x.jsonl"), b"x").unwrap();
        let out = tmp.path().join("out");
        fs::create_dir_all(&out).unwrap();

        let items = [item("src/app", a, 11), item("src/app", b, 1)];
        let r = export(&items, &out, &mut |_, _| {}).unwrap();
        assert_eq!(r.files, 3);
        assert_eq!(r.bytes, 12);

        let f = fs::File::open(&r.zip_path).unwrap();
        let mut z = zip::ZipArchive::new(f).unwrap();
        let mut names: Vec<String> = (0..z.len())
            .map(|i| z.by_index(i).unwrap().name().unwrap().into_owned())
            .collect();
        names.sort();
        assert_eq!(
            names,
            vec![
                "Claude Code/src_app (2)/x.jsonl",
                "Claude Code/src_app/one.jsonl",
                "Claude Code/src_app/sub/two.jsonl",
            ]
        );
        let mut s = String::new();
        z.by_name("Claude Code/src_app/sub/two.jsonl")
            .unwrap()
            .read_to_string(&mut s)
            .unwrap();
        assert_eq!(s, "world!");
    }

    #[test]
    fn exportable_rules_are_review_history() {
        assert!(is_exportable("claude_code.projects"));
        assert!(!is_exportable("pip.cache"));
    }
}
