//! cleaner-core: the scan engine behind ai-disk-cleaner.
//!
//! Flow: load [`rules`] → expand their path templates ([`paths`]) → walk each
//! location ([`scan`]), handing tool-specific folders to an [`inspect`]or that
//! can name what is inside (which Ollama model, which HF repo, which conda
//! env) → produce a [`model::ScanReport`] the UI shows → delete the items the
//! user ticked, reversibly ([`clean`]).
//!
//! Nothing in this crate writes to disk except [`clean`], and that only
//! moves to the Recycle Bin and appends to a log.

pub mod bigfiles;
pub mod clean;
pub mod dedup;
pub mod export;
pub mod inspect;
pub mod model;
pub mod paths;
pub mod preview;
pub mod process;
pub mod relocate;
pub mod rules;
pub mod scan;
pub mod settings;
pub mod vhdx;

pub use model::{Category, SafetyLevel, ScanItem, ScanReport};
pub use rules::{Rule, RuleSet};
pub use scan::{scan_all, ScanOptions};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("rule file is not valid TOML: {0}")]
    RuleParse(#[from] toml::de::Error),
    #[error("io error at {path}: {source}")]
    Io {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
    #[error("refusing to touch {0}: {1}")]
    Refused(std::path::PathBuf, String),
    #[error("recycle bin: {0}")]
    Trash(#[from] trash::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
