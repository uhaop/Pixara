//! Headless batch conversion for performance measurement (no Tauri UI).

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use rayon::prelude::*;
use serde::Serialize;
use walkdir::WalkDir;

use crate::engine;
use crate::supported::{format_from_path, is_supported_image};
use crate::system::convert_worker_count;
use crate::types::{
    ConvertSettings, ConvertStageMs, GvError, ImageFormat, NamingMode, OutputMode, OverwriteMode, Preset,
    QueueItem,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StagePct {
    pub decode: f64,
    pub transform: f64,
    pub encode: f64,
    pub post: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchResult {
    pub files: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub workers: usize,
    pub to_format: String,
    pub preset: String,
    pub optimize_png: bool,
    pub wall_ms: u64,
    pub files_per_sec: f64,
    pub input_bytes: u64,
    pub output_bytes: u64,
    pub stage_ms: ConvertStageMs,
    pub stage_pct: StagePct,
}

pub fn collect_input_files(input_dir: &Path) -> Result<Vec<PathBuf>, GvError> {
    if !input_dir.is_dir() {
        return Err(GvError::Message(format!(
            "input directory not found: {}",
            input_dir.display()
        )));
    }

    let mut files: Vec<PathBuf> = WalkDir::new(input_dir)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .filter(|p| is_supported_image(p))
        .collect();

    files.sort();
    if files.is_empty() {
        return Err(GvError::Message(format!(
            "no supported images under {}",
            input_dir.display()
        )));
    }
    Ok(files)
}

pub fn queue_item_for(source: &Path, input_root: &Path) -> QueueItem {
    let relative = source
        .strip_prefix(input_root)
        .unwrap_or(source)
        .to_string_lossy()
        .replace('\\', "/");

    let source_format = format_from_path(source).unwrap_or(ImageFormat::Png);
    let size_bytes = fs::metadata(source).map(|m| m.len()).unwrap_or(0);

    QueueItem {
        id: uuid::Uuid::new_v4().to_string(),
        batch_id: "bench".into(),
        source_path: source.to_string_lossy().into_owned(),
        relative_path: relative,
        source_format,
        size_bytes,
        zip_source_path: None,
        output_base_name: None,
    }
}

pub fn run_bench(
    input_dir: &Path,
    output_dir: &Path,
    settings: ConvertSettings,
    workers: Option<usize>,
) -> Result<BenchResult, GvError> {
    if settings.to_format.is_any() {
        return Err(GvError::InvalidSettings(
            "target format cannot be Any".into(),
        ));
    }

    fs::create_dir_all(output_dir)?;
    let sources = collect_input_files(input_dir)?;
    let items: Vec<QueueItem> = sources
        .iter()
        .map(|p| queue_item_for(p, input_dir))
        .collect();

    let input_bytes: u64 = items.iter().map(|i| i.size_bytes).sum();
    let worker_count = workers.unwrap_or_else(convert_worker_count).max(1);

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(worker_count)
        .build()
        .map_err(|e| GvError::Message(format!("rayon pool: {e}")))?;

    let started = Instant::now();
    let mut stage_totals = ConvertStageMs::default();
    let results: Vec<Result<(PathBuf, ConvertStageMs), GvError>> = pool.install(|| {
        items
            .par_iter()
            .map(|item| engine::convert_one_timed(item, &settings))
            .collect()
    });
    let wall_ms = started.elapsed().as_millis() as u64;

    let succeeded = results.iter().filter(|r| r.is_ok()).count();
    let failed = results.len() - succeeded;
    for result in results.iter().filter_map(|r| r.as_ref().ok()) {
        stage_totals.merge(&result.1);
    }
    let output_bytes: u64 = results
        .iter()
        .filter_map(|r| r.as_ref().ok())
        .filter_map(|(p, _)| fs::metadata(p).ok())
        .map(|m| m.len())
        .sum();

    let files = items.len();
    let files_per_sec = if wall_ms > 0 {
        files as f64 / (wall_ms as f64 / 1000.0)
    } else {
        0.0
    };

    Ok(BenchResult {
        files,
        succeeded,
        failed,
        workers: worker_count,
        to_format: format!("{:?}", settings.to_format).to_lowercase(),
        preset: format!("{:?}", settings.preset).to_lowercase(),
        optimize_png: settings.optimize_png,
        wall_ms,
        files_per_sec,
        input_bytes,
        output_bytes,
        stage_ms: stage_totals.clone(),
        stage_pct: stage_percentages(&stage_totals),
    })
}

fn stage_percentages(totals: &ConvertStageMs) -> StagePct {
    let total = totals.total_ms().max(1) as f64;
    StagePct {
        decode: totals.decode_ms as f64 / total * 100.0,
        transform: totals.transform_ms as f64 / total * 100.0,
        encode: totals.encode_ms as f64 / total * 100.0,
        post: totals.post_ms as f64 / total * 100.0,
    }
}

pub fn parse_preset(s: &str) -> Result<Preset, GvError> {
    match s.to_ascii_lowercase().as_str() {
        "web" => Ok(Preset::Web),
        "high" => Ok(Preset::High),
        "smallest" => Ok(Preset::Smallest),
        _ => Err(GvError::Message(format!("unknown preset: {s}"))),
    }
}

pub fn parse_format(s: &str) -> Result<ImageFormat, GvError> {
    match s.to_ascii_lowercase().as_str() {
        "png" => Ok(ImageFormat::Png),
        "jpeg" | "jpg" => Ok(ImageFormat::Jpeg),
        "webp" => Ok(ImageFormat::Webp),
        "heic" => Ok(ImageFormat::Heic),
        "gif" => Ok(ImageFormat::Gif),
        "bmp" => Ok(ImageFormat::Bmp),
        "tiff" | "tif" => Ok(ImageFormat::Tiff),
        "avif" => Ok(ImageFormat::Avif),
        _ => Err(GvError::Message(format!("unknown format: {s}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::{temp_dir, write_test_png};

    #[test]
    fn bench_png_smoke() {
        let input = temp_dir("pixara-bench-in");
        let output = temp_dir("pixara-bench-out");
        write_test_png(&input.join("sample.png"));

        let settings = bench_settings(ImageFormat::Png, Preset::Web, &output, false);
        let result = run_bench(&input, &output, settings, Some(1)).expect("bench");
        assert_eq!(result.files, 1);
        assert_eq!(result.succeeded, 1);
        assert_eq!(result.failed, 0);
        assert!(result.output_bytes > 0);
        assert!(result.stage_ms.total_ms() > 0);
        assert!((result.stage_pct.decode + result.stage_pct.transform
            + result.stage_pct.encode + result.stage_pct.post - 100.0)
            .abs()
            < 0.01);

        let _ = fs::remove_dir_all(input);
        let _ = fs::remove_dir_all(output);
    }
}

pub fn bench_settings(
    to_format: ImageFormat,
    preset: Preset,
    output_dir: &Path,
    optimize_png: bool,
) -> ConvertSettings {
    ConvertSettings {
        to_format,
        preset,
        output_mode: OutputMode::CustomDir,
        custom_output_dir: Some(output_dir.to_string_lossy().into_owned()),
        preserve_structure: false,
        naming: NamingMode::ReplaceExtension,
        max_width: None,
        max_height: None,
        skip_same_format: false,
        strip_icc: false,
        rezip_outputs: false,
        flatten_color: "#ffffff".to_string(),
        overwrite_mode: OverwriteMode::Replace,
        optimize_png,
        slow_drive_mode: false,
    }
}
