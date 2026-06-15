use std::path::Path;

use tauri::{AppHandle, Manager};

use crate::types::GvError;

/// Paths to allowlist for WebView previews (`convertFileSrc`).
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct PreviewAllowlist {
    pub directories: Vec<std::path::PathBuf>,
    pub files: Vec<std::path::PathBuf>,
}

pub(crate) fn preview_allowlist_for_path(path: &Path) -> PreviewAllowlist {
    let mut directories = Vec::new();
    let mut files = Vec::new();

    if path.is_file() {
        if let Some(parent) = path.parent() {
            directories.push(parent.to_path_buf());
        }
        files.push(path.to_path_buf());
    } else if path.is_dir() {
        directories.push(path.to_path_buf());
    } else if let Some(parent) = path.parent() {
        // Output paths may not exist yet; allow the target directory for future previews.
        directories.push(parent.to_path_buf());
    }

    PreviewAllowlist { directories, files }
}

/// Allow filesystem paths for WebView previews (`convertFileSrc`) without a global `**` scope.
pub fn allow_paths_for_preview(app: &AppHandle, paths: &[String]) -> Result<(), GvError> {
    for path_str in paths {
        allow_path_for_preview(app, Path::new(path_str))?;
    }
    Ok(())
}

fn allow_path_for_preview(app: &AppHandle, path: &Path) -> Result<(), GvError> {
    let allowlist = preview_allowlist_for_path(path);
    for directory in allowlist.directories {
        allow_directory(app, &directory)?;
    }
    for file in allowlist.files {
        allow_file(app, &file)?;
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
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(prefix: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("{prefix}-{stamp}"));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn nonexistent_output_path_allowlists_parent_directory() {
        let dir = temp_dir("pixara-preview-scope");
        let missing = dir.join("nested").join("future-output.webp");
        assert!(!missing.exists());

        let allowlist = preview_allowlist_for_path(&missing);
        assert!(allowlist.files.is_empty());
        assert_eq!(allowlist.directories, vec![dir.join("nested")]);

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn existing_file_allowlists_parent_and_file() {
        let dir = temp_dir("pixara-preview-scope-file");
        let file = dir.join("photo.png");
        std::fs::write(&file, b"png").expect("write png");

        let allowlist = preview_allowlist_for_path(&file);
        assert_eq!(allowlist.directories, vec![dir.clone()]);
        assert_eq!(allowlist.files, vec![file]);

        let _ = std::fs::remove_dir_all(dir);
    }
}
