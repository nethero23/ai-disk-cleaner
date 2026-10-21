//! Online "is there a newer version" check for MCP server packages. Only the
//! package name leaves the machine (one GET per unique npm / PyPI package);
//! nothing from any config file is sent. The HTTP call itself lives in the
//! app (ureq); this module is the pure part: which URL, how to read the
//! reply, and whether a pinned version is behind.

use super::skills::version_key;
use serde::{Deserialize, Serialize};

/// Result for one identity (`npm:<pkg>` / `pypi:<pkg>`).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct VersionInfo {
    pub identity: String,
    pub latest: Option<String>,
    pub error: Option<String>,
}

/// The registry endpoint for an identity, or `None` when it is not a public
/// package (scripts, docker images, URLs).
pub fn registry_url(identity: &str) -> Option<String> {
    if let Some(pkg) = identity.strip_prefix("npm:") {
        if pkg.is_empty() || pkg.contains("..") || pkg.contains('/') && !pkg.starts_with('@') {
            return None;
        }
        return Some(format!("https://registry.npmjs.org/{pkg}/latest"));
    }
    if let Some(pkg) = identity.strip_prefix("pypi:") {
        if pkg.is_empty()
            || !pkg
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c))
        {
            return None;
        }
        return Some(format!("https://pypi.org/pypi/{pkg}/json"));
    }
    None
}

/// Reads the version out of a registry reply (`{"version": …}` for npm's
/// `/latest`, `{"info": {"version": …}}` for PyPI).
pub fn latest_from_response(identity: &str, body: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(body).ok()?;
    let s = if identity.starts_with("pypi:") {
        v.get("info")?.get("version")?.as_str()?
    } else {
        v.get("version")?.as_str()?
    };
    Some(s.to_string())
}

/// True when `pinned` is a version and `latest` is numerically newer.
pub fn is_behind(pinned: Option<&str>, latest: &str) -> bool {
    match pinned {
        Some(p) if p.chars().next().is_some_and(|c| c.is_ascii_digit()) => {
            version_key(latest) > version_key(p)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urls_only_for_public_registries() {
        assert_eq!(
            registry_url("npm:@modelcontextprotocol/server-github").as_deref(),
            Some("https://registry.npmjs.org/@modelcontextprotocol/server-github/latest")
        );
        assert_eq!(
            registry_url("pypi:mcp-server-fetch").as_deref(),
            Some("https://pypi.org/pypi/mcp-server-fetch/json")
        );
        assert_eq!(registry_url("script:c:/x.js"), None);
        assert_eq!(registry_url("npm:../etc"), None);
        assert_eq!(registry_url("pypi:a b"), None);
    }

    #[test]
    fn parses_both_registries_and_compares() {
        assert_eq!(
            latest_from_response("npm:x", r#"{"name":"x","version":"2.1.0"}"#).as_deref(),
            Some("2.1.0")
        );
        assert_eq!(
            latest_from_response("pypi:x", r#"{"info":{"version":"0.9"}}"#).as_deref(),
            Some("0.9")
        );
        assert_eq!(latest_from_response("npm:x", "not json"), None);
        assert!(is_behind(Some("1.0.0"), "1.2.0"));
        assert!(!is_behind(Some("1.2.0"), "1.2.0"));
        assert!(!is_behind(Some("latest"), "1.2.0"));
        assert!(!is_behind(None, "1.2.0"));
    }
}
