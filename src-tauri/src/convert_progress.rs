use std::path::Path;

use tauri::{AppHandle, Emitter};

use crate::types::{ConvertProgress, ProgressStatus, QueueItem};

pub fn progress_status_for_error(message: &str) -> (ProgressStatus, String) {
    match message {
        "skipped_same_format" => (ProgressStatus::Skipped, "Same format".into()),
        "skipped_exists" => (ProgressStatus::Skipped, "File exists".into()),
        "cancelled" => (ProgressStatus::Skipped, "Cancelled".into()),
        other => (ProgressStatus::Error, other.to_string()),
    }
}

pub fn source_display_name(source_path: &str) -> String {
    Path::new(source_path)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| source_path.to_string())
}

pub fn emit_progress(
    app: &AppHandle,
    completed: u32,
    total: u32,
    item: &QueueItem,
    status: ProgressStatus,
    message: String,
    bytes_after: Option<u64>,
    worker_id: Option<u32>,
) {
    let progress = ConvertProgress {
        current: completed,
        total,
        item_id: item.id.clone(),
        source_path: item.source_path.clone(),
        status,
        message,
        bytes_after,
        worker_id,
    };
    let _ = app.emit("convert-progress", &progress);
}
