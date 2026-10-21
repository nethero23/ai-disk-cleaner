//! User settings: what to leave alone and where extra rules come from.
//! Stored as JSON next to the clean log; missing file means defaults.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Settings {
    /// Folders never reported, whatever rule would match them.
    pub excluded_paths: Vec<PathBuf>,
    /// Rule ids switched off by the user.
    pub disabled_rules: Vec<String>,
    /// A folder of the user's own `*.toml` rule files; same ids override.
    pub custom_rules_dir: Option<PathBuf>,
    /// Where "update rules" downloads from (a folder URL holding the rule files).
    pub rules_update_url: String,
    /// Last successful rule download.
    pub rules_updated_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Items smaller than this are not shown.
    pub min_item_bytes: u64,
    /// UI language ("zh" / "en"); also used for tray menu and notifications.
    pub lang: String,
    /// Closing the window hides it to the tray instead of quitting.
    pub close_to_tray: bool,
    /// Start hidden in the tray when Windows starts.
    pub autostart: bool,
    /// Run a background scan every N hours; 0 = never.
    pub scheduled_scan_hours: u32,
    /// Only notify when the scan finds at least this much junk.
    pub notify_min_bytes: u64,
    /// When the last background scan finished.
    pub last_background_scan: Option<chrono::DateTime<chrono::Utc>>,
    /// After a scheduled scan, move Safe items (whose tool is not running)
    /// to the Recycle Bin without asking. Off by default.
    pub auto_clean_safe: bool,
    /// Look for a new release on GitHub when the app starts.
    pub auto_update_check: bool,
    /// Extra drives (roots) to walk for caches by shape, e.g. `D:\\`.
    pub scan_drives: Vec<PathBuf>,
    /// Notify when one tool grows by at least this much within a week; 0 = never.
    pub growth_notify_bytes: u64,
    /// The first-run walkthrough has been shown (or skipped).
    pub onboarded: bool,
}

pub const DEFAULT_RULES_URL: &str =
    "https://raw.githubusercontent.com/nethero23/ai-disk-cleaner/main/crates/cleaner-core/rules";

impl Default for Settings {
    fn default() -> Self {
        Self {
            excluded_paths: Vec::new(),
            disabled_rules: Vec::new(),
            custom_rules_dir: None,
            rules_update_url: DEFAULT_RULES_URL.into(),
            rules_updated_at: None,
            min_item_bytes: 0,
            lang: "zh".into(),
            close_to_tray: false,
            autostart: false,
            scheduled_scan_hours: 0,
            notify_min_bytes: 1024 * 1024 * 1024,
            last_background_scan: None,
            auto_clean_safe: false,
            auto_update_check: true,
            scan_drives: Vec::new(),
            growth_notify_bytes: 2 * 1024 * 1024 * 1024,
            onboarded: false,
        }
    }
}

impl Settings {
    pub fn load(path: &Path) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let text = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        std::fs::write(path, text)
    }

    /// True when `p` is one of the excluded folders or inside one.
    pub fn is_excluded(&self, p: &Path) -> bool {
        self.excluded_paths.iter().any(|ex| p.starts_with(ex))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_and_defaults_missing_fields() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().join("settings.json");
        assert_eq!(Settings::load(&p), Settings::default());
        let s = Settings {
            excluded_paths: vec![PathBuf::from("/x/keep")],
            disabled_rules: vec!["pip.cache".into()],
            ..Default::default()
        };
        s.save(&p).unwrap();
        let back = Settings::load(&p);
        assert_eq!(back, s);
        assert!(back.is_excluded(Path::new("/x/keep/sub/file")));
        assert!(!back.is_excluded(Path::new("/x/keeper")));
        // An older file without newer fields still loads.
        std::fs::write(&p, r#"{"excluded_paths":["/a"]}"#).unwrap();
        assert_eq!(Settings::load(&p).rules_update_url, DEFAULT_RULES_URL);
    }
}
