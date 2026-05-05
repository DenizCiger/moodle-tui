pub mod cache;
pub mod config;
pub mod secret;

use directories::BaseDirs;
use std::path::PathBuf;

pub const CONFIG_DIR_ENV: &str = "TUI_MOODLE_CONFIG_DIR";

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("{0}")]
    Message(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

pub fn config_dir() -> Result<PathBuf, StorageError> {
    if let Ok(custom) = std::env::var(CONFIG_DIR_ENV) {
        if !custom.trim().is_empty() {
            return Ok(PathBuf::from(custom));
        }
    }
    let base_dirs = BaseDirs::new()
        .ok_or_else(|| StorageError::Message("Failed to determine home directory".to_owned()))?;
    Ok(base_dirs.home_dir().join(".config").join("tui-moodle"))
}
