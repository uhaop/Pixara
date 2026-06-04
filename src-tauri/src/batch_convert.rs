use std::cell::Cell;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

use rayon::prelude::*;
use tauri::AppHandle;

use crate::cancel::is_batch_cancelled;
use crate::convert_progress::{emit_progress, progress_status_for_error, source_display_name};
use crate::engine;
use crate::system::effective_convert_worker_count;
use crate::types::{
    ConvertSettings, GvError, ProgressStatus, QueueItem,
};

thread_local! {
    static WORKER_ID: Cell<u32> = const { Cell::new(0) };
}

static NEXT_WORKER_SLOT: AtomicUsize = AtomicUsize::new(1);

fn current_worker_id() -> u32 {
    WORKER_ID.with(|id| {
        if id.get() == 0 {
            let slot = NEXT_WORKER_SLOT.fetch_add(1, Ordering::Relaxed) as u32;
            id.set(slot);
        }
        id.get()
    })
}

pub fn convert_items_parallel(
    app: &AppHandle,
    items: Vec<QueueItem>,
    settings: &ConvertSettings,
) -> Vec<(QueueItem, Result<PathBuf, GvError>)> {
    let total = items.len() as u32;
    let workers = effective_convert_worker_count(settings.slow_drive_mode);
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(workers)
        .start_handler(|_| {
            let _ = current_worker_id();
        })
        .build()
        .expect("rayon thread pool");

    let finished = AtomicU32::new(0);

    pool.install(|| {
        items
            .into_par_iter()
            .map(|item| {
                let worker_id = current_worker_id();

                if is_batch_cancelled() {
                    let completed = finished.load(Ordering::SeqCst);
                    let (status, message) = progress_status_for_error("cancelled");
                    emit_progress(
                        app,
                        completed,
                        total,
                        &item,
                        status,
                        message,
                        None,
                        Some(worker_id),
                    );
                    return (item, Err(GvError::Message("cancelled".into())));
                }

                let completed_before = finished.load(Ordering::SeqCst);
                emit_progress(
                    app,
                    completed_before,
                    total,
                    &item,
                    ProgressStatus::Converting,
                    source_display_name(&item.source_path),
                    None,
                    Some(worker_id),
                );

                let result = engine::convert_one(&item, settings);
                let completed_after = finished.fetch_add(1, Ordering::SeqCst) + 1;

                let (status, message, bytes_after) = match &result {
                    Ok(out) => (
                        ProgressStatus::Done,
                        out.to_string_lossy().to_string(),
                        std::fs::metadata(out).ok().map(|meta| meta.len()),
                    ),
                    Err(GvError::Message(m)) => {
                        let (status, msg) = progress_status_for_error(m);
                        (status, msg, None)
                    }
                    Err(e) => (ProgressStatus::Error, e.to_string(), None),
                };
                emit_progress(
                    app,
                    completed_after,
                    total,
                    &item,
                    status,
                    message,
                    bytes_after,
                    Some(worker_id),
                );
                (item, result)
            })
            .collect()
    })
}
