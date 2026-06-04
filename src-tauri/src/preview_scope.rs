use std::path::Path;

use tauri::{AppHandle, Manager};

use crate::types::GvError;

/// Allow filesystem paths for WebView previews (`convertFileSrc`) without a global `**` scope.
pub fn allow_paths_for_preview(app: &AppHandle, paths: &[String]) -> Result<(), GvError> {
    for path_str in paths {
        allow_path_for_preview(app, Path::new(path_str))?;
    }
    Ok(())
}

fn allow_path_for_preview(app: &AppHandle, path: &Path) -> Result<(), GvError> {
    if path.is_file() {
        if let Some(parent) = path.parent() {
            allow_directory(app, parent)?;
        }
        allow_file(app, path)?;
    } else if path.is_dir() {
        allow_directory(app, path)?;
    }
    Ok(())
}

fn allow_directory(app: &AppHandle, path: &Path) -> Result<(), GvError> {
    app.asset_protocol_scope()
        .allow_directory(path, true)
        .map_err(|e| GvError::Message(format!("preview scope: {e}")))
}

fn allow_file(app: &AppHandle, path: &Path) -> Result<(), GvError> {
    app.asset_protocol_scope()
        .allow_file(path)
        .map_err(|e| GvError::Message(format!("preview scope: {e}")))
}

/// Allow preview access for every queued source file and optional custom output directory.
pub fn allow_queue_preview_paths(
    app: &AppHandle,
    source_paths: impl IntoIterator<Item = impl AsRef<str>>,
    custom_output_dir: Option<&str>,
) -> Result<(), GvError> {
    for path_str in source_paths {
        allow_path_for_preview(app, Path::new(path_str.as_ref()))?;
    }
    if let Some(dir) = custom_output_dir.filter(|d| !d.trim().is_empty()) {
        allow_directory(app, Path::new(dir))?;
    }
    Ok(())
}

/// Allow a single output directory chosen in settings.
pub fn allow_output_dir(app: &AppHandle, dir: &str) -> Result<(), GvError> {
    if dir.trim().is_empty() {
        return Ok(());
    }
    allow_directory(app, Path::new(dir))
}

#[cfg(test)]
mod tests {
    #[test]
    fn preview_scope_helpers_compile() {
        assert!(std::env::temp_dir().is_dir());
    }
}
