pub mod bench;
mod batch_convert;
mod heic_decode;
mod cancel;
mod commands;
mod config;
mod convert_guard;
mod convert_progress;
mod engine;
mod system;
mod formats;
mod ingest;
mod metadata;
mod naming;
mod png_optimize;
mod preview_scope;
mod privacy_strip;
mod rezip;
mod supported;
#[cfg(test)]
mod test_fixtures;
mod thumbnails;
mod types;

pub use types::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    ingest::cleanup_stale_temp_dirs();
    thumbnails::cleanup_stale_thumbnails();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_system_capabilities_cmd,
            commands::load_config_cmd,
            commands::save_config_cmd,
            commands::ingest_paths_cmd,
            commands::cleanup_temp_batches_cmd,
            commands::convert_batch,
            commands::cancel_convert_batch,
            commands::get_thumbnail_cmd,
            commands::estimate_batch_cmd,
            commands::open_folder,
            commands::browse_files,
            commands::browse_folder,
            commands::browse_zip,
            commands::pick_output_dir,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
