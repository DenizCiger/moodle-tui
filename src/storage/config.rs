use crate::models::{DEFAULT_MOODLE_SERVICE, SavedConfig, normalize_base_url};
use crate::storage::{StorageError, config_dir};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

pub fn config_file() -> Result<PathBuf, StorageError> {
    Ok(config_dir()?.join("config.json"))
}

pub fn load_config() -> Option<SavedConfig> {
    let path = config_file().ok()?;
    let raw = fs::read_to_string(path).ok()?;
    let parsed = serde_json::from_str::<Value>(&raw).ok()?;
    let object = parsed.as_object()?;

    let base_url = object.get("baseUrl").and_then(Value::as_str)?;
    let base_url = normalize_base_url(base_url);
    if base_url.is_empty() {
        return None;
    }

    let username = object.get("username").and_then(Value::as_str)?.trim().to_owned();
    if username.is_empty() {
        return None;
    }

    let service = object
        .get("service")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_MOODLE_SERVICE)
        .to_owned();

    Some(SavedConfig {
        base_url,
        username,
        service,
    })
}

pub fn save_config(config: &SavedConfig) -> Result<(), StorageError> {
    fs::create_dir_all(config_dir()?)?;
    fs::write(config_file()?, serde_json::to_vec_pretty(config)?)?;
    Ok(())
}

pub fn clear_config() -> Result<(), StorageError> {
    if let Ok(path) = config_file() {
        let _ = fs::write(path, b"{}");
    }
    Ok(())
}
