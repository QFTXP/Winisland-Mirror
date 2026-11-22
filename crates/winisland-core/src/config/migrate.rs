use super::{AppConfig, CompactWidgetAlignment, MIN_HIDDEN_WIDTH};

pub const CONFIG_VERSION: u32 = 1;

/// Stamps a config that was loaded as `from_version` with the current version.
///
/// Returns `true` when the config changed and therefore has to be written back.
pub fn migrate(config: &mut AppConfig, from_version: u32) -> bool {
    if from_version >= CONFIG_VERSION {
        return false;
    }
    config.config_version = CONFIG_VERSION;
    true
}

/// Rewrites settings whose keys were renamed or removed before versioning existed.
///
/// The rewrite is driven by key presence rather than by `config_version`: a file
/// that lost a key, whether because it predates the key or because someone edited
/// it, still lands on the current meaning. Schema changes made from here on are
/// steps ordered by version, next to this function.
///
/// Returns `true` when the config changed and therefore has to be written back.
pub fn migrate_legacy_keys(table: &toml::Table, config: &mut AppConfig) -> bool {
    let mut changed = false;
    if !table.contains_key("system_controls_overlay")
        && (table.contains_key("replace_native_volume_flyout")
            || table.contains_key("brightness_overlay_enabled"))
    {
        let volume = table
            .get("replace_native_volume_flyout")
            .and_then(toml::Value::as_bool)
            .unwrap_or(true);
        let brightness = table
            .get("brightness_overlay_enabled")
            .and_then(toml::Value::as_bool)
            .unwrap_or(true);
        config.system_controls_overlay = volume || brightness;
    }
    changed |= [
        "animation_fps",
        "expanded_idle_fps",
        "replace_native_volume_flyout",
        "brightness_overlay_enabled",
    ]
    .iter()
    .any(|key| table.contains_key(*key));
    if !table.contains_key("fullscreen_auto_hide") {
        config.fullscreen_auto_hide = config.auto_hide;
        changed = true;
    }
    if !table.contains_key("expanded_scale") {
        config.expanded_scale = config.compact_scale;
        changed = true;
    }
    if let Some(fully_hide) = table.get("fully_hide").and_then(toml::Value::as_bool) {
        if !table.contains_key("hidden_width") && fully_hide {
            config.hidden_width = MIN_HIDDEN_WIDTH;
        }
        changed = true;
    }
    if !table.contains_key("lyrics_mode") {
        config.lyrics_mode = if config
            .lyrics_local_dir
            .as_deref()
            .is_some_and(|dir| !dir.trim().is_empty())
        {
            "lrc"
        } else {
            "online"
        }
        .to_string();
        changed = true;
    }
    if let Some(entries) = table
        .get("compact_widget_layout")
        .and_then(toml::Value::as_array)
    {
        for (entry, raw_entry) in config.compact_widget_layout.iter_mut().zip(entries) {
            if raw_entry
                .as_table()
                .is_some_and(|table| !table.contains_key("alignment"))
            {
                entry.alignment = CompactWidgetAlignment::legacy_slot(entry.slot);
                changed = true;
            }
        }
    }
    if !table.contains_key("compact_resource_metrics") {
        config.compact_resource_metrics = config.resource_metrics.clone();
        changed = true;
    }
    changed
}
