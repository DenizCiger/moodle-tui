use serde::{Deserialize, Serialize};
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub entry: String,
    #[serde(default)]
    pub permissions: Vec<PluginPermission>,
    #[serde(default)]
    pub settings_schema: Option<serde_json::Value>,
    #[serde(default)]
    pub contributes: PluginContributions,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PluginPermission {
    QuizReadCurrentQuestion,
    QuizWriteAnswers,
    Network,
    NetworkGemini,
    SecretsReadPlugin,
    UiShowPanel,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginContributions {
    #[serde(default)]
    pub quiz_actions: Vec<QuizActionContribution>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QuizActionContribution {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub result_kind: Option<String>,
    #[serde(default)]
    pub default_key: Option<String>,
}

impl PluginManifest {
    pub fn validate(&self) -> Result<(), String> {
        validate_plugin_id(&self.id)?;
        if self.name.trim().is_empty() {
            return Err("plugin name is required".into());
        }
        if self.version.trim().is_empty() {
            return Err("plugin version is required".into());
        }
        validate_relative_entry(&self.entry)?;
        for action in &self.contributes.quiz_actions {
            validate_action_id(&action.id)?;
            if action.title.trim().is_empty() {
                return Err(format!("quiz action '{}' requires a title", action.id));
            }
            if let Some(kind) = &action.result_kind {
                validate_action_id(kind)?;
            }
        }
        Ok(())
    }

    pub fn entry_path(&self, plugin_dir: &Path) -> Result<PathBuf, String> {
        validate_relative_entry(&self.entry)?;
        Ok(plugin_dir.join(&self.entry))
    }
}

pub fn validate_plugin_id(id: &str) -> Result<(), String> {
    let valid = !id.is_empty()
        && id.len() <= 96
        && id
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_');
    if valid {
        Ok(())
    } else {
        Err(format!("invalid plugin id '{id}'"))
    }
}

fn validate_action_id(id: &str) -> Result<(), String> {
    let valid = !id.is_empty()
        && id.len() <= 64
        && id
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_');
    if valid {
        Ok(())
    } else {
        Err(format!("invalid quiz action id '{id}'"))
    }
}

fn validate_relative_entry(entry: &str) -> Result<(), String> {
    let path = Path::new(entry);
    if entry.trim().is_empty()
        || path.is_absolute()
        || entry.contains(['\\', ':', '\0'])
        || entry.starts_with("//")
    {
        return Err("plugin entry must be a relative path".into());
    }
    let safe = path
        .components()
        .all(|component| matches!(component, Component::Normal(_) | Component::CurDir));
    if safe {
        Ok(())
    } else {
        Err("plugin entry cannot escape the plugin directory".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_manifest_ids_and_entries() {
        let manifest = PluginManifest {
            id: "quiz-ai-extension".into(),
            name: "Quiz AI Extension".into(),
            version: "0.1.0".into(),
            entry: "plugin.js".into(),
            permissions: vec![PluginPermission::QuizReadCurrentQuestion],
            settings_schema: None,
            contributes: PluginContributions {
                quiz_actions: vec![QuizActionContribution {
                    id: "study_help".into(),
                    title: "AI Study Help".into(),
                    description: None,
                    result_kind: None,
                    default_key: None,
                }],
            },
        };
        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn rejects_unsafe_manifest_paths() {
        let mut manifest = PluginManifest {
            id: "bad".into(),
            name: "Bad".into(),
            version: "0.1.0".into(),
            entry: "../plugin.js".into(),
            permissions: Vec::new(),
            settings_schema: None,
            contributes: PluginContributions::default(),
        };
        assert!(manifest.validate().is_err());
        manifest.entry = "C:/plugin.js".into();
        assert!(manifest.validate().is_err());
        manifest.entry = r"..\plugin.js".into();
        assert!(manifest.validate().is_err());
    }
}
