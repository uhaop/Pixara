use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

use crate::batch_convert::convert_items_parallel;
use crate::cancel::{request_batch_cancel, reset_batch_cancel};
use crate::config::{load_config, save_config};
use crate::convert_guard::ConvertInProgressGuard;
use crate::engine;
use crate::system::{self, SystemCapabilities};
use crate::ingest;
use crate::naming::{
    estimate_batch_output_bytes, estimate_low_confidence, estimate_warning,
};
use crate::types::OutputMode;
use crate::preview_scope;
use crate::rezip;
use crate::thumbnails;
use crate::types::{
    AppConfig, BatchEstimate, ConvertErrorEntry, ConvertSettings, ConvertSummary, GvError,
    ImageFormat, IngestResult, QueueItem,
};

struct RezipEntry {
    item_id: String,
    path: PathBuf,
    entry_name: String,
}

fn gv_to_string(err: GvError) -> String {
    err.to_string()
}

#[tauri::command]
pub fn get_system_capabilities_cmd() -> SystemCapabilities {
    system::system_capabilities()
}

#[tauri::command]
pub fn load_config_cmd(app: AppHandle) -> Result<AppConfig, String> {
    load_config(&app).map_err(gv_to_string)
}

#[tauri::command]
pub fn save_config_cmd(app: AppHandle, config: AppConfig) -> Result<(), String> {
    save_config(&app, &config).map_err(gv_to_string)
}

#[tauri::command]
pub fn ingest_paths_cmd(
    app: AppHandle,
    paths: Vec<String>,
    from_format: ImageFormat,
) -> Result<IngestResult, String> {
    preview_scope::allow_paths_for_preview(&app, &paths).map_err(gv_to_string)?;
    ingest::ingest_paths(&paths, from_format).map_err(gv_to_string)
}

#[tauri::command]
pub fn cleanup_temp_batches_cmd(batch_ids: Vec<String>, item_ids: Vec<String>) -> Result<(), String> {
    ingest::cleanup_temp_batches(batch_ids);
    thumbnails::cleanup_thumbnails(&item_ids);
    Ok(())
}

#[tauri::command]
pub fn cancel_convert_batch() {
    request_batch_cancel();
}

#[tauri::command]
pub async fn get_thumbnail_cmd(item_id: String, source_path: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        thumbnails::get_or_create_thumbnail(&item_id, &source_path).map_err(gv_to_string)
    })
    .await
    .map_err(|e| format!("thumbnail task failed: {e}"))?
}

#[tauri::command]
pub fn estimate_batch_cmd(
    items: Vec<QueueItem>,
    settings: ConvertSettings,
    accurate_sample: Option<bool>,
) -> Result<BatchEstimate, String> {
    let accurate_sample = accurate_sample.unwrap_or(false);
    let input_bytes: u64 = items.iter().map(|i| i.size_bytes).sum();
    let low_confidence = estimate_low_confidence(&items, settings.to_format);
    let warning = estimate_warning(&items, settings.to_format);

    let (estimated_output_bytes, sampled) = if accurate_sample && !items.is_empty() {
        match sample_based_estimate(&items, &settings) {
            Some(bytes) => (bytes, true),
            None => (
                estimate_batch_output_bytes(&items, settings.preset, settings.to_format),
                false,
            ),
        }
    } else {
        (
            estimate_batch_output_bytes(&items, settings.preset, settings.to_format),
            false,
        )
    };

    let preview_paths = items
        .iter()
        .take(3)
        .filter_map(|item| {
            engine::resolve_output_path(item, &settings)
                .ok()
                .map(|p| p.to_string_lossy().into_owned())
        })
        .collect();
    Ok(BatchEstimate {
        input_bytes,
        estimated_output_bytes,
        preview_paths,
        low_confidence: low_confidence && !sampled,
        warning: if sampled { None } else { warning },
        sampled,
    })
}

fn sample_based_estimate(items: &[QueueItem], settings: &ConvertSettings) -> Option<u64> {
    const MAX_SAMPLES: usize = 3;
    let temp = tempfile::tempdir().ok()?;
    let mut sample_settings = settings.clone();
    sample_settings.optimize_png = false;
    sample_settings.output_mode = OutputMode::CustomDir;
    sample_settings.custom_output_dir = Some(temp.path().to_string_lossy().into_owned());

    let mut sample_in = 0u64;
    let mut sample_out = 0u64;
    let mut sample_count = 0usize;

    for item in items.iter().take(MAX_SAMPLES) {
        sample_in = sample_in.saturating_add(item.size_bytes);
        match engine::convert_one(item, &sample_settings) {
            Ok(out) => {
                if let Ok(meta) = fs::metadata(&out) {
                    sample_out = sample_out.saturating_add(meta.len());
                    sample_count += 1;
                }
            }
            Err(_) => continue,
        }
    }

    if sample_count == 0 || sample_in == 0 {
        return None;
    }

    let ratio = sample_out as f64 / sample_in as f64;
    let total_in: u64 = items.iter().map(|i| i.size_bytes).sum();
    Some((total_in as f64 * ratio).round().max(1.0) as u64)
}

fn run_convert_batch(
    app: AppHandle,
    items: Vec<QueueItem>,
    settings: ConvertSettings,
) -> Result<ConvertSummary, String> {
    reset_batch_cancel();

    preview_scope::allow_queue_preview_paths(
        &app,
        items.iter().map(|item| item.source_path.as_str()),
        settings.custom_output_dir.as_deref(),
    )
    .map_err(gv_to_string)?;

    let output_preview_paths: Vec<String> = items
        .iter()
        .filter_map(|item| {
            engine::resolve_output_path(item, &settings)
                .ok()
                .map(|p| p.to_string_lossy().into_owned())
        })
        .collect();
    preview_scope::allow_paths_for_preview(&app, &output_preview_paths).map_err(gv_to_string)?;

    let outcomes = convert_items_parallel(&app, items, &settings);

    let mut succeeded = 0u32;
    let mut failed = 0u32;
    let mut skipped = 0u32;
    let mut errors = Vec::new();

    let mut rezip_groups: HashMap<String, Vec<RezipEntry>> = HashMap::new();

    for (item, result) in &outcomes {
        match result {
            Ok(out_path) => {
                succeeded += 1;
                if settings.rezip_outputs {
                    if let Some(zip_path) = &item.zip_source_path {
                        let entry = rezip::rezip_entry_name(item, &settings);
                        rezip_groups.entry(zip_path.clone()).or_default().push(RezipEntry {
                            item_id: item.id.clone(),
                            path: out_path.clone(),
                            entry_name: entry,
                        });
                    }
                }
            }
            Err(GvError::Message(m))
                if m == "skipped_same_format" || m == "skipped_exists" || m == "cancelled" =>
            {
                skipped += 1
            }
            Err(e) => {
                failed += 1;
                errors.push(ConvertErrorEntry {
                    item_id: item.id.clone(),
                    source_path: item.source_path.clone(),
                    message: e.to_string(),
                });
            }
        }
    }

    if settings.rezip_outputs {
        for (zip_path, files) in rezip_groups {
            let zip_entries: Vec<(PathBuf, String)> = files
                .iter()
                .map(|f| (f.path.clone(), f.entry_name.clone()))
                .collect();
            if let Err(e) = rezip::create_converted_zip(std::path::Path::new(&zip_path), &zip_entries)
            {
                let message = format!("Re-zip failed: {e}");
                for file in &files {
                    errors.push(ConvertErrorEntry {
                        item_id: file.item_id.clone(),
                        source_path: zip_path.clone(),
                        message: message.clone(),
                    });
                }
            }
        }
    }

    Ok(ConvertSummary {
        succeeded,
        failed,
        skipped,
        errors,
    })
}

#[tauri::command]
pub async fn convert_batch(
    app: AppHandle,
    items: Vec<QueueItem>,
    settings: ConvertSettings,
) -> Result<ConvertSummary, String> {
    let _guard = ConvertInProgressGuard::try_acquire().map_err(|e| e.to_string())?;

    tauri::async_runtime::spawn_blocking(move || run_convert_batch(app, items, settings))
        .await
        .map_err(|e| format!("convert task failed: {e}"))?
}

#[tauri::command]
pub fn open_folder(app: AppHandle, path: String) -> Result<(), String> {
    let folder = std::path::PathBuf::from(&path);
    if !folder.is_dir() {
        return Err("Output folder does not exist".into());
    }

    preview_scope::allow_output_dir(&app, &path).map_err(gv_to_string)?;

    app.opener()
        .open_path(path, None::<&str>)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn browse_folder(app: AppHandle) -> Result<Option<String>, String> {
    let picked = app.dialog().file().blocking_pick_folder();
    if let Some(path) = &picked {
        preview_scope::allow_output_dir(&app, &path.to_string()).map_err(gv_to_string)?;
    }
    Ok(picked.map(|p| p.to_string()))
}

#[tauri::command]
pub async fn browse_zip(app: AppHandle) -> Result<Vec<String>, String> {
    let picked = app
        .dialog()
        .file()
        .add_filter("ZIP archive", &["zip"])
        .blocking_pick_file();
    let paths: Vec<String> = picked.map(|p| vec![p.to_string()]).unwrap_or_default();
    preview_scope::allow_paths_for_preview(&app, &paths).map_err(gv_to_string)?;
    Ok(paths)
}

#[tauri::command]
pub async fn browse_files(app: AppHandle) -> Result<Vec<String>, String> {
    let picked = app
        .dialog()
        .file()
        .add_filter(
            "Images",
            &[
                "png", "jpg", "jpeg", "webp", "heic", "heif", "gif", "bmp", "tif", "tiff", "avif",
            ],
        )
        .blocking_pick_files();
    let paths: Vec<String> = picked
        .unwrap_or_default()
        .into_iter()
        .map(|p| p.to_string())
        .collect();
    preview_scope::allow_paths_for_preview(&app, &paths).map_err(gv_to_string)?;
    Ok(paths)
}

#[tauri::command]
pub async fn pick_output_dir(app: AppHandle) -> Result<Option<String>, String> {
    let picked = app.dialog().file().blocking_pick_folder();
    if let Some(path) = &picked {
        preview_scope::allow_output_dir(&app, &path.to_string()).map_err(gv_to_string)?;
    }
    Ok(picked.map(|p| p.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::convert_progress::progress_status_for_error;
    use crate::engine;
    use crate::types::{
        ConvertSettings, GvError, ImageFormat, NamingMode, OutputMode, OverwriteMode, Preset,
        ProgressStatus, QueueItem,
    };
    use image::{ImageBuffer, Rgb};
    use std::fs;
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(prefix: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("{prefix}-{stamp}"));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn write_test_png(path: &Path) {
        let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_fn(16, 16, |x, y| {
            if (x + y) % 2 == 0 {
                Rgb([200, 80, 70])
            } else {
                Rgb([40, 160, 210])
            }
        });
        img.save(path).expect("write png");
    }

    fn sample_item(source: &Path, relative: &str) -> QueueItem {
        QueueItem {
            id: "test-item".into(),
            batch_id: "test-batch".into(),
            source_path: source.to_string_lossy().into_owned(),
            relative_path: relative.into(),
            source_format: ImageFormat::Png,
            size_bytes: fs::metadata(source).expect("metadata").len(),
            zip_source_path: None,
            output_base_name: None,
        }
    }

    fn base_settings() -> ConvertSettings {
        ConvertSettings {
            to_format: ImageFormat::Webp,
            preset: Preset::Web,
            output_mode: OutputMode::SameFolder,
            custom_output_dir: None,
            preserve_structure: true,
            naming: NamingMode::ReplaceExtension,
            max_width: None,
            max_height: None,
            skip_same_format: false,
            strip_icc: false,
            rezip_outputs: false,
            flatten_color: "#ffffff".to_string(),
            overwrite_mode: OverwriteMode::AutoRename,
            optimize_png: true,
            slow_drive_mode: false,
        }
    }

    #[test]
    fn cancelled_progress_status_is_skipped() {
        let (status, message) = progress_status_for_error("cancelled");
        assert_eq!(status, ProgressStatus::Skipped);
        assert_eq!(message, "Cancelled");
    }

    #[test]
    fn overwrite_skip_avoids_existing_output() {
        let dir = temp_dir("gv-pixara-overwrite-skip");
        let source = dir.join("sample.png");
        write_test_png(&source);
        fs::write(dir.join("sample.webp"), b"existing").expect("seed");

        let item = sample_item(&source, "sample.png");
        let mut settings = base_settings();
        settings.overwrite_mode = OverwriteMode::Skip;

        let err = engine::convert_one(&item, &settings).expect_err("skip");
        assert!(matches!(err, GvError::Message(ref m) if m == "skipped_exists"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn overwrite_replace_replaces_existing_output() {
        let dir = temp_dir("gv-pixara-overwrite-replace");
        let source = dir.join("sample.png");
        write_test_png(&source);
        fs::write(dir.join("sample.webp"), b"existing").expect("seed");

        let item = sample_item(&source, "sample.png");
        let mut settings = base_settings();
        settings.overwrite_mode = OverwriteMode::Replace;

        let output = engine::convert_one(&item, &settings).expect("replace");
        assert!(output.exists());
        assert!(fs::metadata(&output).expect("meta").len() > 8);

        let _ = fs::remove_dir_all(dir);
    }
}
