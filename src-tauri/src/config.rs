use std::fs;
use std::path::PathBuf;

use tauri::Manager;

use crate::types::{AppConfig, GvError, ImageFormat};

const CONFIG_DIR: &str = "pixara";
const CONFIG_FILE: &str = "config.json";

fn config_path(app: &tauri::AppHandle) -> Result<PathBuf, GvError> {
    let base = app
        .path()
        .app_data_dir()
        .map_err(|e| GvError::Message(e.to_string()))?;
    Ok(base.join(CONFIG_DIR).join(CONFIG_FILE))
}

fn legacy_config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(app_data) = std::env::var("APPDATA") {
        let app_data = PathBuf::from(app_data);
        paths.push(
            app_data
                .join("com.gv.gv-pixara")
                .join("pixara")
                .join(CONFIG_FILE),
        );
        paths.push(
            app_data
                .join("com.gv.gv-pixara")
                .join("gv-pixara")
                .join(CONFIG_FILE),
        );
        paths.push(
            app_data
                .join("com.gv.gv-image")
                .join("gv-image")
                .join(CONFIG_FILE),
        );
    }
    paths
}

fn migrate_legacy_config(app: &tauri::AppHandle) -> Result<Option<AppConfig>, GvError> {
    for legacy in legacy_config_paths() {
        if !legacy.is_file() {
            continue;
        }
        let data = fs::read_to_string(&legacy)?;
        let cfg: AppConfig = serde_json::from_str(&data)?;
        let cfg = sanitize_config(cfg);
        save_config(app, &cfg)?;
        let _ = fs::remove_file(&legacy);
        return Ok(Some(cfg));
    }
    Ok(None)
}

pub fn sanitize_config(mut config: AppConfig) -> AppConfig {
    if config.to_format.is_any() {
        config.to_format = ImageFormat::Webp;
    }
    #[cfg(not(feature = "heic"))]
    {
        if config.from_format == ImageFormat::Heic {
            config.from_format = ImageFormat::Any;
        }
        if config.to_format == ImageFormat::Heic {
            config.to_format = ImageFormat::Webp;
        }
    }
    if config.max_width == Some(0) {
        config.max_width = None;
    }
    if config.max_height == Some(0) {
        config.max_height = None;
    }
    let color = config.flatten_color.trim();
    if color.is_empty() {
        config.flatten_color = "#ffffff".to_string();
    } else if !color.starts_with('#') {
        config.flatten_color = format!("#{color}");
    } else {
        config.flatten_color = color.to_string();
    }
    config
}

pub fn load_config(app: &tauri::AppHandle) -> Result<AppConfig, GvError> {
    let path = config_path(app)?;
    if !path.exists() {
        if let Some(cfg) = migrate_legacy_config(app)? {
            return Ok(cfg);
        }
        return Ok(AppConfig::default());
    }
    let data = fs::read_to_string(&path)?;
    let cfg: AppConfig = serde_json::from_str(&data)?;
    Ok(sanitize_config(cfg))
}

pub fn save_config(app: &tauri::AppHandle, config: &AppConfig) -> Result<(), GvError> {
    let path = config_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let sanitized = sanitize_config(config.clone());
    let data = serde_json::to_string_pretty(&sanitized)?;
    fs::write(path, data)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{NamingMode, OutputMode, Preset};

    #[test]
    fn sanitize_config_rejects_invalid_target_format() {
        let config = AppConfig {
            from_format: ImageFormat::Any,
            to_format: ImageFormat::Any,
            preset: Preset::Web,
            output_mode: OutputMode::SameFolder,
            custom_output_dir: None,
            preserve_structure: true,
            naming: NamingMode::ReplaceExtension,
            max_width: Some(0),
            max_height: Some(0),
            skip_same_format: true,
            strip_icc: false,
            rezip_outputs: false,
            flatten_color: "#ffffff".to_string(),
            overwrite_mode: crate::types::OverwriteMode::AutoRename,
            optimize_png: true,
            slow_drive_mode: false,
            queue_view: crate::types::QueueView::Grid,
        };

        let sanitized = sanitize_config(config);
        assert_eq!(sanitized.to_format, ImageFormat::Webp);
        assert_eq!(sanitized.max_width, None);
        assert_eq!(sanitized.max_height, None);
    }

    #[test]
    #[cfg(not(feature = "heic"))]
    fn sanitize_config_strips_heic_without_feature() {
        let config = AppConfig {
            from_format: ImageFormat::Heic,
            to_format: ImageFormat::Heic,
            preset: Preset::Web,
            output_mode: OutputMode::SameFolder,
            custom_output_dir: None,
            preserve_structure: true,
            naming: NamingMode::ReplaceExtension,
            max_width: None,
            max_height: None,
            skip_same_format: true,
            strip_icc: false,
            rezip_outputs: false,
            flatten_color: "#ffffff".to_string(),
            overwrite_mode: crate::types::OverwriteMode::AutoRename,
            optimize_png: true,
            slow_drive_mode: false,
            queue_view: crate::types::QueueView::List,
        };

        let sanitized = sanitize_config(config);
        assert_eq!(sanitized.from_format, ImageFormat::Any);
        assert_eq!(sanitized.to_format, ImageFormat::Webp);
    }

    #[test]
    fn deserialize_config_without_queue_view_defaults_to_list() {
        let json = r##"{
            "fromFormat": "any",
            "toFormat": "webp",
            "preset": "web",
            "outputMode": "sameFolder",
            "preserveStructure": true,
            "naming": "replaceExtension",
            "skipSameFormat": true,
            "flattenColor": "#ffffff",
            "overwriteMode": "autoRename",
            "optimizePng": true
        }"##;
        let cfg: AppConfig = serde_json::from_str(json).expect("parse");
        assert_eq!(cfg.queue_view, crate::types::QueueView::List);
    }
}
