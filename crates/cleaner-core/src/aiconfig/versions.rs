//! Online "is there a newer version" check for MCP server packages. Only the
//! package name leaves the machine (one GET per unique npm / PyPI package);
//! nothing from any config file is sent. HTTP is supplied by the app so the
//! request policy and response handling can be tested without the network.

use super::skills::version_key;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::{Duration, Instant};

/// Result for one identity (`npm:<pkg>` / `pypi:<pkg>`).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct VersionInfo {
    pub identity: String,
    pub latest: Option<String>,
    pub error: Option<String>,
}

fn package_part(s: &str) -> bool {
    s.as_bytes().first().is_some_and(u8::is_ascii_alphanumeric)
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c))
}

/// Public package identities only, never URLs, queries, credentials or paths.
pub fn registry_url(identity: &str) -> Option<String> {
    if let Some(pkg) = identity.strip_prefix("npm:") {
        let valid = if let Some(scoped) = pkg.strip_prefix('@') {
            scoped
                .split_once('/')
                .is_some_and(|(scope, name)| package_part(scope) && package_part(name))
        } else {
            package_part(pkg)
        };
        if !valid || pkg.len() > 214 {
            return None;
        }
        return Some(format!("https://registry.npmjs.org/{pkg}/latest"));
    }
    if let Some(pkg) = identity.strip_prefix("pypi:") {
        if !package_part(pkg) {
            return None;
        }
        return Some(format!("https://pypi.org/pypi/{pkg}/json"));
    }
    None
}

/// Reads the version out of npm's `/latest` or PyPI's package JSON.
pub fn latest_from_response(identity: &str, body: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(body).ok()?;
    let s = if identity.starts_with("pypi:") {
        v.get("info")?.get("version")?.as_str()?
    } else {
        v.get("version")?.as_str()?
    };
    (!s.trim().is_empty()).then(|| s.to_string())
}

/// At most 200 unique packages, with one shared deadline rather than 200
/// per-request timeouts. The fetcher must honor its supplied timeout.
pub fn check_versions(
    enabled: bool,
    identities: Vec<String>,
    budget: Duration,
    mut fetch: impl FnMut(&str, Duration) -> Result<String, String>,
) -> Vec<VersionInfo> {
    if !enabled {
        return Vec::new();
    }
    let started = Instant::now();
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for identity in identities {
        if out.len() == 200 {
            break;
        }
        let Some(url) = registry_url(&identity) else {
            continue;
        };
        if !seen.insert(identity.clone()) {
            continue;
        }
        let remaining = budget.saturating_sub(started.elapsed());
        let result = if remaining.is_zero() {
            Err("Version check time limit reached".to_string())
        } else {
            fetch(&url, remaining.min(Duration::from_secs(5))).and_then(|body| {
                latest_from_response(&identity, &body)
                    .ok_or_else(|| "Registry response has no version".to_string())
            })
        };
        let mut info = VersionInfo {
            identity,
            ..Default::default()
        };
        match result {
            Ok(version) => info.latest = Some(version),
            Err(error) => info.error = Some(error),
        }
        out.push(info);
    }
    out
}

/// Numeric comparison for display only, not a dependency/upgrade resolver.
/// Missing numeric components compare as zero, matching the UI comparator.
pub fn is_behind(pinned: Option<&str>, latest: &str) -> bool {
    let Some(pinned) = pinned.filter(|p| p.starts_with(|c: char| c.is_ascii_digit())) else {
        return false;
    };
    let a = version_key(pinned);
    let b = version_key(latest);
    (0..a.len().max(b.len()))
        .map(|i| b.get(i).unwrap_or(&0).cmp(a.get(i).unwrap_or(&0)))
        .find(|order| !order.is_eq())
        .is_some_and(|order| order.is_gt())
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
        for identity in [
            "script:c:/x.js",
            "npm:../etc",
            "pypi:a b",
            "npm:",
            "pypi:..",
            "npm:@scope",
            "npm:@scope/name/extra",
            "npm:x?token=secret",
            "npm:x#fragment",
            "npm:x%2fsecret",
            "npm:https://example.com",
            "npm:x\\secret",
            "npm:@scope/../secret",
        ] {
            assert_eq!(registry_url(identity), None, "{identity}");
        }
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
        for body in ["not json", "{}", r#"{"version":""}"#, r#"{"version":123}"#] {
            assert_eq!(latest_from_response("npm:x", body), None);
        }
        for (pin, latest, expected) in [
            ("1.0.0", "1.2.0", true),
            ("1.2.0", "1.2.0", false),
            ("1.2", "1.2.0", false),
            ("1.2.0", "1.2", false),
            ("1.10", "1.2", false),
            ("1.2", "v1.10.0", true),
            ("latest", "1.2.0", false),
            ("^1.0", "2.0", false),
        ] {
            assert_eq!(is_behind(Some(pin), latest), expected, "{pin} / {latest}");
        }
        assert!(!is_behind(None, "1.2.0"));
    }

    #[test]
    fn disabled_and_expired_checks_never_fetch() {
        let ids = vec!["npm:test-package".to_string()];
        assert!(
            check_versions(false, ids.clone(), Duration::from_secs(15), |_, _| {
                panic!("disabled checks must not use the network")
            })
            .is_empty()
        );
        let expired = check_versions(true, ids, Duration::ZERO, |_, _| {
            panic!("expired checks must not use the network")
        });
        assert_eq!(expired.len(), 1);
        assert!(expired[0].error.is_some());
    }

    #[test]
    fn deduplicates_valid_packages_and_reports_failures() {
        let mut urls = Vec::new();
        let results = check_versions(
            true,
            [
                "npm:a",
                "npm:a",
                "script:x",
                "pypi:b",
                "npm:bad",
                "npm:offline",
            ]
            .map(str::to_string)
            .to_vec(),
            Duration::from_secs(15),
            |url, timeout| {
                assert!(timeout <= Duration::from_secs(5));
                urls.push(url.to_string());
                if url.ends_with("/a/latest") {
                    Ok(r#"{"version":"1.0"}"#.into())
                } else if url.contains("/b/") {
                    Ok(r#"{"info":{"version":"2.0"}}"#.into())
                } else if url.contains("/bad/") {
                    Ok("not json".into())
                } else {
                    Err("offline".into())
                }
            },
        );
        assert_eq!(urls.len(), 4);
        assert_eq!(results[0].latest.as_deref(), Some("1.0"));
        assert_eq!(results[1].latest.as_deref(), Some("2.0"));
        assert!(results[2].error.is_some());
        assert_eq!(results[3].error.as_deref(), Some("offline"));
    }

    #[test]
    fn limits_unique_requests_not_duplicate_inputs() {
        let mut ids = vec!["npm:a".to_string(); 250];
        ids.extend((0..250).map(|i| format!("npm:pkg-{i}")));
        let mut requests = 0;
        let results = check_versions(true, ids, Duration::from_secs(15), |_, _| {
            requests += 1;
            Err("offline".into())
        });
        assert_eq!(requests, 200);
        assert_eq!(results.len(), 200);
    }
}
