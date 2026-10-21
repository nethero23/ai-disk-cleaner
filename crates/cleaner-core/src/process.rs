//! Which AI tools are running right now. Deleting a model while Ollama is
//! serving it, or a workspace folder while Cursor has it open, either fails
//! or corrupts state, so the scan marks such items and the cleaner refuses
//! them until the tool is closed.

use std::collections::HashSet;

/// Lower-cased executable names of every running process.
pub fn running_names() -> HashSet<String> {
    let mut sys = sysinfo::System::new();
    sys.refresh_processes_specifics(
        sysinfo::ProcessesToUpdate::All,
        true,
        sysinfo::ProcessRefreshKind::nothing(),
    );
    sys.processes()
        .values()
        .map(|p| p.name().to_string_lossy().to_lowercase())
        .collect()
}

/// Returns the first of `wanted` (rule's `processes`) found in `running`.
/// Matching ignores case and a `.exe` suffix on either side, so a rule can
/// say `ollama` and match `ollama.exe` or `ollama app.exe`'s sibling.
pub fn first_running<'a>(wanted: &'a [String], running: &HashSet<String>) -> Option<&'a str> {
    wanted.iter().find_map(|w| {
        let w_norm = normalise(w);
        running
            .iter()
            .any(|r| normalise(r) == w_norm)
            .then_some(w.as_str())
    })
}

fn normalise(name: &str) -> String {
    let n = name.to_lowercase();
    n.strip_suffix(".exe").map(str::to_string).unwrap_or(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_ignore_case_and_exe_suffix() {
        let running: HashSet<String> = ["ollama.exe".to_string(), "code".to_string()]
            .into_iter()
            .collect();
        let wanted = vec!["Ollama".to_string()];
        assert_eq!(first_running(&wanted, &running), Some("Ollama"));
        let wanted = vec!["cursor.exe".to_string(), "Code.exe".to_string()];
        assert_eq!(first_running(&wanted, &running), Some("Code.exe"));
        let wanted = vec!["lm studio".to_string()];
        assert_eq!(first_running(&wanted, &running), None);
    }

    #[test]
    fn this_test_binary_is_running() {
        // Sanity check that sysinfo actually lists processes on this platform.
        assert!(!running_names().is_empty());
    }
}
