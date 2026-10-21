//! Expands the `${VAR}` templates used in rule files into real paths.
//!
//! Rules are written against Windows variable names (`%LOCALAPPDATA%` style,
//! spelled `${LOCALAPPDATA}` here). On other platforms the same names map onto
//! the XDG/HOME equivalents so the engine and its tests run anywhere.

use std::collections::HashMap;
use std::path::PathBuf;

/// The variables a rule may reference and where they resolve.
#[derive(Debug, Clone)]
pub struct PathContext {
    vars: HashMap<String, PathBuf>,
}

impl PathContext {
    /// Builds the context from the real environment.
    pub fn from_env() -> Self {
        let mut vars: HashMap<String, PathBuf> = HashMap::new();
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        vars.insert("USERPROFILE".into(), home.clone());
        vars.insert("HOMEPATH".into(), home.clone());
        vars.insert("HOME".into(), home.clone());
        vars.insert(
            "LOCALAPPDATA".into(),
            dirs::cache_dir().unwrap_or_else(|| home.join(".cache")),
        );
        vars.insert(
            "APPDATA".into(),
            dirs::config_dir().unwrap_or_else(|| home.join(".config")),
        );
        vars.insert("TEMP".into(), std::env::temp_dir());
        if let Some(u) = std::env::var_os("USERNAME").or_else(|| std::env::var_os("USER")) {
            vars.insert("USERNAME".into(), PathBuf::from(u));
        }
        vars.insert(
            "PROGRAMDATA".into(),
            std::env::var_os("PROGRAMDATA")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("/var/lib")),
        );
        vars.insert(
            "SYSTEMROOT".into(),
            std::env::var_os("SYSTEMROOT")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from(r"C:\Windows")),
        );
        // Tool-specific overrides the user may have set. These win over defaults
        // because that is where the tool actually writes.
        for key in [
            "OLLAMA_MODELS",
            "HF_HOME",
            "HF_HUB_CACHE",
            "PIP_CACHE_DIR",
            "UV_CACHE_DIR",
            "CONDA_PKGS_DIRS",
            "CONDA_ROOT",
            "PLAYWRIGHT_BROWSERS_PATH",
            "TORCH_HOME",
            "MODELSCOPE_CACHE",
            "LMSTUDIO_HOME",
        ] {
            if let Some(v) = std::env::var_os(key) {
                if !v.is_empty() {
                    vars.insert(key.into(), PathBuf::from(v));
                }
            }
        }
        Self { vars }
    }

    /// A context with explicit values, for tests and for "scan this other user folder".
    pub fn with_vars(vars: HashMap<String, PathBuf>) -> Self {
        Self { vars }
    }

    pub fn set(&mut self, key: &str, value: PathBuf) {
        self.vars.insert(key.to_string(), value);
    }

    /// Expands a template. Returns `None` when it references a variable that
    /// is not set, which is how optional overrides (`${OLLAMA_MODELS}`) are
    /// skipped without a special case in every rule.
    pub fn expand(&self, template: &str) -> Option<PathBuf> {
        let mut out = String::new();
        let mut rest = template;
        while let Some(start) = rest.find("${") {
            out.push_str(&rest[..start]);
            let after = &rest[start + 2..];
            let end = after.find('}')?;
            let key = &after[..end];
            let value = self.vars.get(key)?;
            out.push_str(&value.to_string_lossy());
            rest = &after[end + 1..];
        }
        out.push_str(rest);
        // Rules use `/`; Windows accepts both, but normalising keeps ids stable.
        let normalised = if cfg!(windows) {
            out.replace('/', "\\")
        } else {
            out
        };
        Some(PathBuf::from(normalised))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> PathContext {
        let mut vars = HashMap::new();
        vars.insert("USERPROFILE".to_string(), PathBuf::from("/home/u"));
        PathContext::with_vars(vars)
    }

    #[test]
    fn expands_known_vars() {
        let p = ctx().expand("${USERPROFILE}/.ollama/models").unwrap();
        assert!(p.ends_with(".ollama/models") || p.ends_with(".ollama\\models"));
    }

    #[test]
    fn unknown_var_means_skip() {
        assert!(ctx().expand("${OLLAMA_MODELS}").is_none());
    }
}
