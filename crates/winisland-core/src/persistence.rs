use std::fs;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::{
    AppConfig, WIDGET_GRID_SLOTS, clamp_schema_numbers, ensure_settings_widget, migrate,
    migrate_legacy_keys, normalize_compact_widget_layout, normalize_expanded_pages,
    normalize_resource_metrics, set_resource_widget_span,
};

static NEXT_CONFIG_WRITE_ID: AtomicU64 = AtomicU64::new(1);

pub fn load_config_at(path: &Path) -> AppConfig {
    let mut migrated = false;
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            log::info!("Config file not found, using defaults");
            let default = AppConfig::default();
            save_after_load(path, &default);
            return default;
        }
        Err(error) => {
            log::error!("Cannot read config '{}': {error}", path.display());
            return AppConfig::default();
        }
    };
    let mut config = match toml::from_str::<AppConfig>(&content) {
        Ok(config) => config,
        Err(error) => {
            log::error!("Cannot parse config '{}': {error}", path.display());
            match preserve_invalid_config(path) {
                Ok(backup) => {
                    log::warn!(
                        "Invalid config preserved at '{}'; using defaults",
                        backup.display()
                    );
                    let default = AppConfig::default();
                    save_after_load(path, &default);
                    return default;
                }
                Err(backup_error) => {
                    log::error!(
                        "Cannot preserve invalid config '{}': {backup_error}",
                        path.display()
                    );
                    return AppConfig::default();
                }
            }
        }
    };
    if let Ok(table) = toml::from_str::<toml::Table>(&content) {
        migrated |= migrate_legacy_keys(&table, &mut config);
    }
    let loaded_version = config.config_version;
    if migrate(&mut config, loaded_version) {
        migrated = true;
    }
    if clamp_schema_numbers(&mut config) {
        migrated = true;
    }
    let resource_span =
        set_resource_widget_span(config.resource_widget_columns, config.resource_widget_rows);
    if (config.resource_widget_columns, config.resource_widget_rows) != resource_span {
        config.resource_widget_columns = resource_span.0;
        config.resource_widget_rows = resource_span.1;
        migrated = true;
    }
    if config.island_style == "mica" {
        config.island_style = "default".to_string();
        migrated = true;
    }
    if !matches!(config.lyrics_mode.as_str(), "online" | "lrc") {
        config.lyrics_mode = "online".to_string();
        migrated = true;
    }
    if ensure_settings_widget(&mut config.widget_layout) {
        migrated = true;
    }
    if normalize_compact_widget_layout(&mut config.compact_widget_layout) {
        migrated = true;
    }
    if normalize_resource_metrics(&mut config.resource_metrics) {
        migrated = true;
    }
    if normalize_resource_metrics(&mut config.compact_resource_metrics) {
        migrated = true;
    }
    if normalize_expanded_pages(
        &mut config.expanded_page_order,
        &mut config.hidden_expanded_pages,
        &mut config.plugin_pages,
    ) {
        migrated = true;
    }
    let plugin_layout_len = config.plugin_widget_layout.len();
    config.plugin_widget_layout.retain(|entry| {
        entry.slot < WIDGET_GRID_SLOTS
            && entry.plugin_id.len() <= 255
            && entry.widget_key.len() <= 63
            && !entry.plugin_id.is_empty()
            && !entry.widget_key.is_empty()
            && entry
                .plugin_id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
            && entry
                .widget_key
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    });
    if config.plugin_widget_layout.len() != plugin_layout_len {
        migrated = true;
    }
    if migrated {
        save_after_load(path, &config);
    }
    config
}

pub fn save_config_at(path: &Path, config: &AppConfig) -> Result<(), PersistError> {
    let content = toml::to_string_pretty(config)
        .map_err(|error| PersistError::Serialize(error.to_string()))?;
    write_config_atomically(path, content.as_bytes()).map_err(PersistError::Write)?;
    log::info!("Config saved to: {}", path.display());
    Ok(())
}

/// Writes the config after a load that had to repair it.
///
/// The caller is a load path with nothing to report to; the error is logged
/// instead of returned so that a read-only config directory still yields the
/// repaired in-memory config.
fn save_after_load(path: &Path, config: &AppConfig) {
    if let Err(error) = save_config_at(path, config) {
        log::error!("Cannot save config '{}': {error}", path.display());
    }
}

#[derive(Debug)]
pub enum PersistError {
    Serialize(String),
    Write(std::io::Error),
}

impl std::fmt::Display for PersistError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Serialize(message) => write!(f, "cannot serialize config: {message}"),
            Self::Write(error) => write!(f, "cannot write config: {error}"),
        }
    }
}

impl std::error::Error for PersistError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Write(error) => Some(error),
            Self::Serialize(_) => None,
        }
    }
}

fn preserve_invalid_config(path: &Path) -> std::io::Result<PathBuf> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let id = NEXT_CONFIG_WRITE_ID.fetch_add(1, Ordering::Relaxed);
    let backup = path.with_file_name(format!(
        "config.invalid-{timestamp}-{}-{id}.toml",
        std::process::id()
    ));
    fs::copy(path, &backup)?;
    Ok(backup)
}

fn write_config_atomically(path: &Path, content: &[u8]) -> std::io::Result<()> {
    let (temporary, mut file) = create_config_temp_file(path)?;
    let result = (|| {
        file.write_all(content)?;
        file.sync_all()?;
        fs::rename(&temporary, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn create_config_temp_file(path: &Path) -> std::io::Result<(PathBuf, fs::File)> {
    for _ in 0..16 {
        let id = NEXT_CONFIG_WRITE_ID.fetch_add(1, Ordering::Relaxed);
        let temporary =
            path.with_file_name(format!(".config.toml.tmp-{}-{id}", std::process::id()));
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
        {
            Ok(file) => return Ok((temporary, file)),
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    Err(std::io::Error::new(
        ErrorKind::AlreadyExists,
        "could not reserve a unique config temporary file",
    ))
}
