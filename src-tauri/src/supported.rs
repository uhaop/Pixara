use std::path::Path;

use crate::types::ImageFormat;

pub fn extension_for_format(format: ImageFormat) -> &'static str {
    match format {
        ImageFormat::Any => "bin",
        ImageFormat::Png => "png",
        ImageFormat::Jpeg => "jpg",
        ImageFormat::Webp => "webp",
        ImageFormat::Heic => "heic",
        ImageFormat::Gif => "gif",
        ImageFormat::Bmp => "bmp",
        ImageFormat::Tiff => "tiff",
        ImageFormat::Avif => "avif",
    }
}

pub fn format_from_extension(ext: &str) -> Option<ImageFormat> {
    match ext.to_ascii_lowercase().as_str() {
        "png" => Some(ImageFormat::Png),
        "jpg" | "jpeg" | "jpe" => Some(ImageFormat::Jpeg),
        "webp" => Some(ImageFormat::Webp),
        "heic" | "heif" => Some(ImageFormat::Heic),
        "gif" => Some(ImageFormat::Gif),
        "bmp" | "dib" => Some(ImageFormat::Bmp),
        "tif" | "tiff" => Some(ImageFormat::Tiff),
        "avif" | "avifs" => Some(ImageFormat::Avif),
        _ => None,
    }
}

pub fn format_from_path(path: &Path) -> Option<ImageFormat> {
    path.extension()
        .and_then(|e| e.to_str())
        .and_then(format_from_extension)
}

pub fn is_supported_image(path: &Path) -> bool {
    format_from_path(path).is_some()
}

pub fn matches_from_filter(path: &Path, from_format: ImageFormat) -> bool {
    if from_format.is_any() {
        return is_supported_image(path);
    }
    format_from_path(path) == Some(from_format)
}
