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
    /// A model not touched for this many days is suggested as idle.
    pub idle_model_days: u32,
    /// Notify when one tool grows by at least this much within a week; 0 = never.
    pub growth_notify_bytes: u64,
    /// The first-run walkthrough has been shown (or skipped).
    pub onboarded: bool,
    /// On the AI tools page, ask npm / PyPI for the latest version of each
    /// MCP server package. Sends package names only, never config contents.
    pub mcp_version_check: bool,
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
            idle_model_days: 90,
            growth_notify_bytes: 2 * 1024 * 1024 * 1024,
            onboarded: false,
            mcp_version_check: true,
        }
    }
}

impl Settings {
    /// Network permission must not fall back to enabled on a failed read.
    pub fn version_check_allowed(path: &Path) -> bool {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|text| serde_json::from_str::<Self>(&text).ok())
            .is_some_and(|settings| settings.mcp_version_check)
    }

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
    fn version_check_permission_fails_closed_when_settings_cannot_be_read() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().join("settings.json");
        assert!(!Settings::version_check_allowed(&p), "missing file");
        assert!(!Settings::version_check_allowed(tmp.path()), "not a file");
        for invalid in ["", "{", "null", r#"{"mcp_version_check":"false"}"#] {
            std::fs::write(&p, invalid).unwrap();
            assert!(!Settings::version_check_allowed(&p), "invalid settings");
        }
        for enabled in [false, true, false] {
            Settings {
                mcp_version_check: enabled,
                ..Default::default()
            }
            .save(&p)
            .unwrap();
            assert_eq!(Settings::version_check_allowed(&p), enabled);
        }
        // Existing valid settings keep the documented default for older files.
        std::fs::write(&p, "{}").unwrap();
        assert!(Settings::version_check_allowed(&p));
    }

    #[test]
    fn partial_settings_write_stops_starting_version_requests_mid_batch() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().join("settings.json");
        Settings::default().save(&p).unwrap();
        let mut requests = 0;
        let versions = crate::aiconfig::check_versions(
            Settings::version_check_allowed(&p),
            vec!["npm:first".into(), "npm:second".into()],
            std::time::Duration::from_secs(1),
            |_, _| {
                if !Settings::version_check_allowed(&p) {
                    return Err("Version checking is disabled".into());
                }
                requests += 1;
                std::fs::write(&p, "{").unwrap();
                Ok(r#"{"version":"1.0.0"}"#.into())
            },
        );
        assert_eq!(requests, 1, "no request may start during a partial write");
        assert!(versions
            .iter()
            .any(|v| v.error.as_deref() == Some("Version checking is disabled")));
    }

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
        let old = Settings::load(&p);
        assert_eq!(old.rules_update_url, DEFAULT_RULES_URL);
        assert!(old.mcp_version_check);
    }

    #[test]
    fn disabled_version_check_survives_save_and_reload() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().join("settings.json");
        let settings = Settings {
            mcp_version_check: false,
            ..Default::default()
        };
        settings.save(&p).unwrap();
        assert!(!Settings::load(&p).mcp_version_check);
        std::fs::write(&p, r#"{"mcp_version_check":false}"#).unwrap();
        assert!(!Settings::load(&p).mcp_version_check);
    }
}
