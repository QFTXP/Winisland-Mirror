use std::fs;

use winisland_core::config::AppConfig;
use winisland_core::persistence::PersistError;

pub fn get_config_path() -> std::path::PathBuf {
    let mut path = crate::platform::shell().config_dir();
    if !path.exists() {
        let _ = fs::create_dir_all(&path);
    }
    path.push("config.toml");
    path
}

pub fn load_config() -> AppConfig {
    winisland_core::persistence::load_config_at(&get_config_path())
}

pub fn save_config(config: &AppConfig) -> Result<(), PersistError> {
    winisland_core::persistence::save_config_at(&get_config_path(), config)
}

/// Saves for callers that have no way to report a failure to the user.
pub fn save_config_logged(config: &AppConfig) {
    if let Err(error) = save_config(config) {
        log::error!("Cannot save config: {error}");
    }
}
