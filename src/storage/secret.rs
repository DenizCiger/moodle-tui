use crate::models::SavedConfig;
use crate::storage::{StorageError, config_dir};
pub use tui_components::storage::secret::SecretStorageDiagnostic;
use tui_components::storage::secret::SecretStore;

const SECRET_SERVICE: &str = "tui-moodle";

pub fn account_key(config: &SavedConfig) -> String {
    format!("{}|{}|{}", config.base_url, config.username, config.service)
}

fn store() -> Result<SecretStore, StorageError> {
    Ok(SecretStore::new(
        SECRET_SERVICE,
        "tui-moodle",
        "TUI_MOODLE",
        config_dir()?,
    ))
}

pub fn get_secure_storage_diagnostic() -> SecretStorageDiagnostic {
    tui_components::storage::secret::get_secure_storage_diagnostic()
}

pub fn save_password(config: &SavedConfig, password: &str) -> Result<(), StorageError> {
    store()?.save(&account_key(config), password)
}

pub fn load_password(config: &SavedConfig) -> Result<Option<String>, StorageError> {
    store()?.load(&account_key(config))
}

pub fn clear_password(config: &SavedConfig) -> Result<(), StorageError> {
    store()?.clear(&account_key(config))
}

pub fn save_plugin_secret(
    plugin_id: &str,
    secret_name: &str,
    value: &str,
) -> Result<(), StorageError> {
    let key = crate::plugins::registry::plugin_secret_key(plugin_id, secret_name)
        .map_err(StorageError::Message)?;
    store()?.save(&key, value)
}

pub fn load_plugin_secret(
    plugin_id: &str,
    secret_name: &str,
) -> Result<Option<String>, StorageError> {
    let key = crate::plugins::registry::plugin_secret_key(plugin_id, secret_name)
        .map_err(StorageError::Message)?;
    store()?.load(&key)
}

pub fn clear_plugin_secret(plugin_id: &str, secret_name: &str) -> Result<(), StorageError> {
    let key = crate::plugins::registry::plugin_secret_key(plugin_id, secret_name)
        .map_err(StorageError::Message)?;
    store()?.clear(&key)
}
