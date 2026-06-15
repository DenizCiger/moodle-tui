use crate::plugins::manifest::{PluginManifest, validate_plugin_id};
use crate::storage::{StorageError, config_dir};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use tui_components::storage::json::{named_file, read_json_or_default, write_json_pretty};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstalledPlugin {
    pub manifest: PluginManifest,
    pub directory: PathBuf,
    pub enabled: bool,
    #[serde(default)]
    pub load_error: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginRegistry {
    #[serde(default)]
    pub plugins: Vec<InstalledPlugin>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct PluginStateFile {
    #[serde(default)]
    enabled: HashMap<String, bool>,
}

pub fn plugins_dir() -> Result<PathBuf, StorageError> {
    Ok(config_dir()?.join("plugins"))
}

pub fn registry_file() -> Result<PathBuf, StorageError> {
    Ok(named_file(config_dir()?, "plugins.json"))
}

pub fn load_registry() -> PluginRegistry {
    discover_plugins().unwrap_or_default()
}

pub fn discover_plugins() -> Result<PluginRegistry, StorageError> {
    let state = read_state();
    let root = plugins_dir()?;
    let mut plugins = Vec::new();

    if let Ok(entries) = fs::read_dir(&root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            plugins.push(load_plugin_dir(&path, &state.enabled));
        }
    }

    plugins.sort_by(|left, right| left.manifest.id.cmp(&right.manifest.id));
    Ok(PluginRegistry { plugins })
}

pub fn save_enabled_state(registry: &PluginRegistry) -> Result<(), StorageError> {
    let enabled = registry
        .plugins
        .iter()
        .map(|plugin| (plugin.manifest.id.clone(), plugin.enabled))
        .collect();
    write_json_pretty(registry_file()?, &PluginStateFile { enabled })
}

pub fn set_plugin_enabled(plugin_id: &str, enabled: bool) -> Result<(), StorageError> {
    let mut state = read_state();
    state.enabled.insert(plugin_id.to_owned(), enabled);
    write_json_pretty(registry_file()?, &state)
}

pub fn validate_install_source(path: &Path) -> Result<PluginManifest, String> {
    let manifest_path = path.join("plugin.json");
    let raw = fs::read_to_string(&manifest_path)
        .map_err(|error| format!("failed to read {}: {error}", manifest_path.display()))?;
    let manifest: PluginManifest = serde_json::from_str(&raw)
        .map_err(|error| format!("invalid plugin manifest JSON: {error}"))?;
    manifest.validate()?;
    Ok(manifest)
}

pub fn install_plugin_from_dir(source: &Path) -> Result<InstalledPlugin, String> {
    let destination_root =
        plugins_dir().map_err(|error| format!("failed to locate plugin directory: {error}"))?;
    install_plugin_from_dir_to(source, &destination_root)
}

fn install_plugin_from_dir_to(
    source: &Path,
    destination_root: &Path,
) -> Result<InstalledPlugin, String> {
    let manifest = validate_install_source(source)?;
    let destination = destination_root.join(&manifest.id);
    if destination.exists() {
        fs::remove_dir_all(&destination)
            .map_err(|error| format!("failed to replace existing plugin: {error}"))?;
    }
    fs::create_dir_all(destination_root)
        .map_err(|error| format!("failed to create plugin directory: {error}"))?;
    copy_dir(source, &destination)?;
    Ok(InstalledPlugin {
        manifest,
        directory: destination,
        enabled: true,
        load_error: None,
    })
}

pub fn plugin_secret_key(plugin_id: &str, secret_name: &str) -> Result<String, String> {
    validate_plugin_id(plugin_id)?;
    let valid_secret = !secret_name.is_empty()
        && secret_name.len() <= 64
        && secret_name
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_');
    if !valid_secret {
        return Err(format!("invalid plugin secret name '{secret_name}'"));
    }
    Ok(format!("plugin:{plugin_id}:{secret_name}"))
}

fn read_state() -> PluginStateFile {
    registry_file()
        .map(read_json_or_default)
        .ok()
        .and_then(|value| serde_json::from_value(value).ok())
        .unwrap_or_default()
}

fn load_plugin_dir(path: &Path, enabled: &HashMap<String, bool>) -> InstalledPlugin {
    match validate_install_source(path) {
        Ok(manifest) => {
            let id = manifest.id.clone();
            InstalledPlugin {
                manifest,
                directory: path.to_owned(),
                enabled: enabled.get(&id).copied().unwrap_or(true),
                load_error: None,
            }
        }
        Err(error) => InstalledPlugin {
            manifest: invalid_manifest_for_path(path),
            directory: path.to_owned(),
            enabled: false,
            load_error: Some(error),
        },
    }
}

fn invalid_manifest_for_path(path: &Path) -> PluginManifest {
    let fallback = path
        .file_name()
        .and_then(|value| value.to_str())
        .map(sanitize_fallback_id)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "invalid-plugin".into());
    PluginManifest {
        id: fallback,
        name: "Invalid plugin".into(),
        version: "0.0.0".into(),
        entry: "plugin".into(),
        permissions: Vec::new(),
        settings_schema: None,
        contributes: Default::default(),
    }
}

fn sanitize_fallback_id(raw: &str) -> String {
    let mut out = String::new();
    for ch in raw.chars() {
        if ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_' {
            out.push(ch);
        } else if ch.is_ascii_uppercase() {
            out.push(ch.to_ascii_lowercase());
        }
    }
    out
}

fn copy_dir(source: &Path, destination: &Path) -> Result<(), String> {
    fs::create_dir_all(destination)
        .map_err(|error| format!("failed to create {}: {error}", destination.display()))?;
    for entry in fs::read_dir(source)
        .map_err(|error| format!("failed to read {}: {error}", source.display()))?
    {
        let entry = entry.map_err(|error| format!("failed to read plugin entry: {error}"))?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if source_path.is_dir() {
            copy_dir(&source_path, &destination_path)?;
        } else if source_path.is_file() {
            fs::copy(&source_path, &destination_path).map_err(|error| {
                format!(
                    "failed to copy {} to {}: {error}",
                    source_path.display(),
                    destination_path.display()
                )
            })?;
        }
    }
    Ok(())
}

pub fn installed_plugin_ids(registry: &PluginRegistry) -> HashSet<String> {
    registry
        .plugins
        .iter()
        .map(|plugin| plugin.manifest.id.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn derives_plugin_secret_keys() {
        assert_eq!(
            plugin_secret_key("quiz-ai-extension", "gemini_api_key").unwrap(),
            "plugin:quiz-ai-extension:gemini_api_key"
        );
        assert!(plugin_secret_key("../bad", "gemini_api_key").is_err());
        assert!(plugin_secret_key("ok", "../bad").is_err());
    }

    #[test]
    fn validates_install_source_manifest() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(
            temp.path().join("plugin.json"),
            r#"{
              "id": "quiz-ai-extension",
              "name": "Quiz AI Extension",
              "version": "0.1.0",
              "entry": "plugin.js",
              "permissions": ["quiz_read_current_question"],
              "contributes": {"quiz_actions": [{"id": "study_help", "title": "AI Study Help"}]}
            }"#,
        )
        .unwrap();
        let manifest = validate_install_source(temp.path()).unwrap();
        assert_eq!(manifest.id, "quiz-ai-extension");
    }

    #[test]
    fn installs_plugin_into_config_plugin_dir() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        let plugins = temp.path().join("plugins");
        fs::create_dir_all(&source).unwrap();
        fs::write(
            source.join("plugin.json"),
            r#"{
              "id": "quiz-ai-extension",
              "name": "Quiz AI Extension",
              "version": "0.1.0",
              "entry": "plugin.js"
            }"#,
        )
        .unwrap();
        fs::write(source.join("plugin.js"), "console.log('ok');").unwrap();

        let installed = install_plugin_from_dir_to(&source, &plugins).unwrap();

        assert_eq!(installed.manifest.id, "quiz-ai-extension");
        assert!(plugins.join("quiz-ai-extension/plugin.json").exists());
        assert!(plugins.join("quiz-ai-extension/plugin.js").exists());
    }
}
