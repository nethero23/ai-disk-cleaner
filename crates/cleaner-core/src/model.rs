use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// How risky deleting an item is. Drives the default tick state in the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SafetyLevel {
    /// Pure cache; the owning tool rebuilds it silently. Ticked by default.
    Safe,
    /// Recoverable but slow: a model or package that must be re-downloaded.
    Redownload,
    /// May hold user data (chat history, an env still in use). Never ticked by default.
    Review,
}

/// What kind of thing an item is, for grouping in the report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Models,
    PackageCache,
    PythonEnv,
    AiEditor,
    AiAgent,
    AiDesktopApp,
    Browser,
    Container,
    ProjectArtifacts,
    Gpu,
    System,
}

/// One thing the user can tick and delete.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanItem {
    /// Stable id: `<rule_id>::<path>` so the UI can keep selection across rescans.
    pub id: String,
    pub rule_id: String,
    pub tool: String,
    pub category: Category,
    /// Human-readable name, e.g. "qwen2.5:7b" rather than "sha256-abc...".
    pub name: String,
    pub path: PathBuf,
    pub size_bytes: u64,
    pub file_count: u64,
    /// Best-effort "last used": newest mtime/atime inside, or tool-specific metadata.
    pub last_used: Option<DateTime<Utc>>,
    pub safety: SafetyLevel,
    /// What happens if you delete it, in the user's words ("Ollama 会重新下载 4.7 GB").
    pub consequence: String,
    /// Extra detail the UI can show on hover (version, revision, env packages).
    pub detail: Option<String>,
    /// Name of a running process that owns this data; set means "close it first".
    #[serde(default)]
    pub in_use_by: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScanReport {
    pub items: Vec<ScanItem>,
    pub total_bytes: u64,
    pub safe_bytes: u64,
    pub redownload_bytes: u64,
    pub review_bytes: u64,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    /// Locations a rule pointed at that could not be read (permissions). Shown, not hidden.
    pub unreadable: Vec<PathBuf>,
}

impl ScanReport {
    pub fn push(&mut self, item: ScanItem) {
        self.total_bytes += item.size_bytes;
        match item.safety {
            SafetyLevel::Safe => self.safe_bytes += item.size_bytes,
            SafetyLevel::Redownload => self.redownload_bytes += item.size_bytes,
            SafetyLevel::Review => self.review_bytes += item.size_bytes,
        }
        self.items.push(item);
    }

    /// Largest first, so the UI's first screen shows what matters.
    pub fn sort_by_size(&mut self) {
        self.items.sort_by_key(|i| std::cmp::Reverse(i.size_bytes));
    }
}

/// "4.7 GB" style formatting shared by CLI, log and tests.
pub fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut v = bytes as f64;
    let mut i = 0;
    while v >= 1024.0 && i < UNITS.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if i == 0 {
        format!("{bytes} B")
    } else {
        format!("{v:.1} {}", UNITS[i])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn human_size_rounds_sensibly() {
        assert_eq!(human_size(0), "0 B");
        assert_eq!(human_size(1023), "1023 B");
        assert_eq!(human_size(1536), "1.5 KB");
        assert_eq!(human_size(4_700_000_000), "4.4 GB");
    }
}
