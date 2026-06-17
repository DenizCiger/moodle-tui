use crate::storage::{StorageError, config_dir};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tui_components::storage::json::{named_file, read_json_or_default, write_json_pretty};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginSettingsFile {
    #[serde(default)]
    pub plugins: HashMap<String, HashMap<String, String>>,
}

fn settings_file() -> Result<std::path::PathBuf, StorageError> {
    Ok(named_file(config_dir()?, "plugin-settings.json"))
}

pub fn load_all() -> PluginSettingsFile {
    settings_file()
        .map(read_json_or_default)
        .ok()
        .and_then(|value| serde_json::from_value(value).ok())
        .unwrap_or_default()
}

pub fn load_plugin_setting(plugin_id: &str, name: &str) -> Option<String> {
    load_all()
        .plugins
        .get(plugin_id)
        .and_then(|settings| settings.get(name))
        .cloned()
}

pub fn save_plugin_setting(plugin_id: &str, name: &str, value: &str) -> Result<(), StorageError> {
    let mut all = load_all();
    all.plugins
        .entry(plugin_id.to_owned())
        .or_default()
        .insert(name.to_owned(), value.to_owned());
    write_json_pretty(settings_file()?, &all)
}

pub fn clear_plugin_settings(plugin_id: &str) -> Result<(), StorageError> {
    let mut all = load_all();
    all.plugins.remove(plugin_id);
    write_json_pretty(settings_file()?, &all)
}
