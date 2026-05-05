pub mod cache;
pub mod config;
pub mod secret;

use std::path::PathBuf;

pub const CONFIG_DIR_ENV: &str = "TUI_MOODLE_CONFIG_DIR";
pub use tui_components::storage::StorageError;

pub fn config_dir() -> Result<PathBuf, StorageError> {
    tui_components::storage::app_config_dir("tui-moodle", Some(CONFIG_DIR_ENV))
}
