//! Strict, read-only JSONL loading for recovery history. Missing is empty;
//! unreadable or damaged is an error, never evidence that nothing happened.
use crate::{Error, Result};
use serde::de::DeserializeOwned;
use std::path::Path;

pub(crate) fn read<T: DeserializeOwned>(path: &Path) -> Result<Vec<T>> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => {
            return Err(Error::Io {
                path: path.into(),
                source,
            })
        }
    };
    text.lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(index, line)| {
            serde_json::from_str(line).map_err(|_| Error::Io {
                path: path.into(),
                // Do not echo record contents or unknown-field values in errors.
                source: std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("invalid history record at line {}", index + 1),
                ),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_and_blank_logs_are_empty_but_invalid_utf8_is_an_error() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("history.jsonl");
        assert!(read::<serde_json::Value>(&path).unwrap().is_empty());
        std::fs::write(&path, b"\n  \r\n").unwrap();
        assert!(read::<serde_json::Value>(&path).unwrap().is_empty());
        std::fs::write(&path, [0xff]).unwrap();
        assert!(read::<serde_json::Value>(&path).is_err());
    }

    #[test]
    fn damaged_tail_is_reported_without_exposing_or_modifying_contents() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("history.jsonl");
        let bytes = b"{\"valid\":true}\n\nprivate-invalid-record";
        std::fs::write(&path, bytes).unwrap();
        let error = read::<serde_json::Value>(&path).unwrap_err().to_string();
        assert!(error.contains("line 3"));
        assert!(!error.contains("private-invalid-record"));
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
    }
}
