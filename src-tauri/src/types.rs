use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageFormat {
    Any,
    Png,
    Jpeg,
    Webp,
    Heic,
    Gif,
    Bmp,
    Tiff,
    Avif,
}

impl ImageFormat {
    pub fn is_any(self) -> bool {
        matches!(self, ImageFormat::Any)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Preset {
    Web,
    High,
    Smallest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OutputMode {
    SameFolder,
    CustomDir,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NamingMode {
    ReplaceExtension,
    AppendSuffix,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OverwriteMode {
    AutoRename,
    Replace,
    Skip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum QueueView {
    #[default]
    List,
    Grid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueItem {
    pub id: String,
    pub batch_id: String,
    pub source_path: String,
    pub relative_path: String,
    pub source_format: ImageFormat,
    pub size_bytes: u64,
    /// Set when this item was ingested from a ZIP archive (original path on disk).
    #[serde(default)]
    pub zip_source_path: Option<String>,
    /// Optional output basename (no extension); sanitized on convert.
    #[serde(default)]
    pub output_base_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConvertSettings {
    pub to_format: ImageFormat,
    pub preset: Preset,
    pub output_mode: OutputMode,
    pub custom_output_dir: Option<String>,
    pub preserve_structure: bool,
    pub naming: NamingMode,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
    pub skip_same_format: bool,
    #[serde(default)]
    pub strip_icc: bool,
    #[serde(default)]
    pub rezip_outputs: bool,
    #[serde(default = "default_flatten_color")]
    pub flatten_color: String,
    #[serde(default)]
    pub overwrite_mode: OverwriteMode,
    #[serde(default = "default_optimize_png")]
    pub optimize_png: bool,
    #[serde(default)]
    pub slow_drive_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchEstimate {
    pub input_bytes: u64,
    pub estimated_output_bytes: u64,
    pub preview_paths: Vec<String>,
    #[serde(default)]
    pub low_confidence: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
    #[serde(default)]
    pub sampled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IngestResult {
    pub batch_id: String,
    pub items: Vec<QueueItem>,
    pub skipped: u32,
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConvertProgress {
    pub current: u32,
    pub total: u32,
    pub item_id: String,
    pub source_path: String,
    pub status: ProgressStatus,
    pub message: String,
    pub bytes_after: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worker_id: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProgressStatus {
    Converting,
    Skipped,
    Done,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConvertSummary {
    pub succeeded: u32,
    pub failed: u32,
    pub skipped: u32,
    pub errors: Vec<ConvertErrorEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConvertErrorEntry {
    pub item_id: String,
    pub source_path: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub from_format: ImageFormat,
    pub to_format: ImageFormat,
    pub preset: Preset,
    pub output_mode: OutputMode,
    pub custom_output_dir: Option<String>,
    pub preserve_structure: bool,
    pub naming: NamingMode,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
    pub skip_same_format: bool,
    #[serde(default)]
    pub strip_icc: bool,
    #[serde(default)]
    pub rezip_outputs: bool,
    #[serde(default = "default_flatten_color")]
    pub flatten_color: String,
    #[serde(default)]
    pub overwrite_mode: OverwriteMode,
    #[serde(default = "default_optimize_png")]
    pub optimize_png: bool,
    #[serde(default)]
    pub slow_drive_mode: bool,
    #[serde(default)]
    pub queue_view: QueueView,
}

impl Default for OverwriteMode {
    fn default() -> Self {
        OverwriteMode::AutoRename
    }
}

fn default_flatten_color() -> String {
    "#ffffff".to_string()
}

fn default_optimize_png() -> bool {
    true
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            from_format: ImageFormat::Any,
            to_format: ImageFormat::Webp,
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
            flatten_color: default_flatten_color(),
            overwrite_mode: OverwriteMode::AutoRename,
            optimize_png: default_optimize_png(),
            slow_drive_mode: false,
            queue_view: QueueView::default(),
        }
    }
}

impl AppConfig {
    pub fn to_convert_settings(&self) -> ConvertSettings {
        ConvertSettings {
            to_format: self.to_format,
            preset: self.preset,
            output_mode: self.output_mode,
            custom_output_dir: self.custom_output_dir.clone(),
            preserve_structure: self.preserve_structure,
            naming: self.naming,
            max_width: self.max_width,
            max_height: self.max_height,
            skip_same_format: self.skip_same_format,
            strip_icc: self.strip_icc,
            rezip_outputs: self.rezip_outputs,
            flatten_color: self.flatten_color.clone(),
            overwrite_mode: self.overwrite_mode,
            optimize_png: self.optimize_png,
            slow_drive_mode: self.slow_drive_mode,
        }
    }
}

/// Per-file conversion stage timings (milliseconds), aggregated in pixara-bench.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConvertStageMs {
    pub decode_ms: u64,
    pub transform_ms: u64,
    pub encode_ms: u64,
    pub post_ms: u64,
}

impl ConvertStageMs {
    pub fn total_ms(&self) -> u64 {
        self.decode_ms + self.transform_ms + self.encode_ms + self.post_ms
    }

    pub fn merge(&mut self, other: &ConvertStageMs) {
        self.decode_ms += other.decode_ms;
        self.transform_ms += other.transform_ms;
        self.encode_ms += other.encode_ms;
        self.post_ms += other.post_ms;
    }
}

#[derive(Debug, Error)]
pub enum GvError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),
    #[error("Zip error: {0}")]
    Zip(String),
    #[error("HEIF error: {0}")]
    Heif(String),
    #[error("Unsupported format")]
    UnsupportedFormat,
    #[error("Invalid settings: {0}")]
    InvalidSettings(String),
    #[error("{0}")]
    Message(String),
}
