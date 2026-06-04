use std::path::Path;

use crate::supported::extension_for_format;
use crate::types::{ConvertSettings, ImageFormat, NamingMode, Preset, QueueItem};

const INVALID_WIN_CHARS: &[char] = &['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

/// Remove characters illegal in Windows file names (control chars and reserved).
pub fn sanitize_output_stem(stem: &str) -> String {
    let mut out = String::with_capacity(stem.len());
    for ch in stem.chars() {
        if ch.is_control() || INVALID_WIN_CHARS.contains(&ch) {
            out.push('_');
        } else {
            out.push(ch);
        }
    }
    let trimmed: String = out
        .trim_matches(|c: char| c == '.' || c == ' ' || c == '_')
        .trim()
        .to_string();
    if trimmed.is_empty() {
        "image".to_string()
    } else {
        trimmed
    }
}

pub fn default_stem(source: &Path, relative_path: &str) -> String {
    let rel = Path::new(relative_path);
    rel.file_stem()
        .and_then(|s| s.to_str())
        .or_else(|| source.file_stem().and_then(|s| s.to_str()))
        .unwrap_or("image")
        .to_string()
}

pub fn output_stem(item: &QueueItem, source: &Path) -> String {
    if let Some(custom) = &item.output_base_name {
        let stem = custom.trim();
        if !stem.is_empty() {
            return sanitize_output_stem(stem);
        }
    }
    sanitize_output_stem(&default_stem(source, &item.relative_path))
}

pub fn output_file_name(item: &QueueItem, source: &Path, ext: &str, naming: NamingMode) -> String {
    let stem = output_stem(item, source);
    match naming {
        NamingMode::ReplaceExtension => format!("{stem}.{ext}"),
        NamingMode::AppendSuffix => format!("{stem}_converted.{ext}"),
    }
}

pub fn rezip_entry_name(item: &QueueItem, settings: &ConvertSettings) -> String {
    let source = Path::new(&item.source_path);
    let ext = extension_for_format(settings.to_format);
    let file_name = output_file_name(item, source, ext, settings.naming);
    let rel = Path::new(&item.relative_path);
    if let Some(parent) = rel.parent() {
        let prefix = parent.to_string_lossy().replace('\\', "/");
        if prefix.is_empty() {
            file_name
        } else {
            format!("{prefix}/{file_name}")
        }
    } else {
        file_name
    }
}

fn lossy_ratio(preset: Preset) -> f64 {
    match preset {
        Preset::Web => 0.55,
        Preset::High => 0.75,
        Preset::Smallest => 0.35,
    }
}

/// Expansion / shrink factor for a source→target pair (container bytes in → encoded bytes out).
fn pair_factor(source: ImageFormat, to_format: ImageFormat, preset: Preset) -> f64 {
    match (source, to_format) {
        (ImageFormat::Heic, ImageFormat::Png) => match preset {
            Preset::Web => 8.0,
            Preset::High => 10.0,
            Preset::Smallest => 6.5,
        },
        (ImageFormat::Heic, ImageFormat::Jpeg) => lossy_ratio(preset) * 0.85,
        (ImageFormat::Heic, ImageFormat::Webp) => lossy_ratio(preset) * 0.75,
        (ImageFormat::Heic, ImageFormat::Avif) => lossy_ratio(preset) * 0.55,
        (ImageFormat::Heic, ImageFormat::Heic) => lossy_ratio(preset),
        (ImageFormat::Heic, ImageFormat::Gif) => 0.7,
        (ImageFormat::Heic, ImageFormat::Bmp | ImageFormat::Tiff) => 12.0,
        (ImageFormat::Png, ImageFormat::Jpeg | ImageFormat::Webp | ImageFormat::Avif) => {
            lossy_ratio(preset)
        }
        (ImageFormat::Png, ImageFormat::Png) => match preset {
            Preset::Web => 0.95,
            Preset::High => 1.05,
            Preset::Smallest => 0.85,
        },
        (ImageFormat::Jpeg, ImageFormat::Png) => 1.4,
        (ImageFormat::Jpeg, ImageFormat::Jpeg) => lossy_ratio(preset),
        _ => {
            let ratio = lossy_ratio(preset);
            match to_format {
                ImageFormat::Png | ImageFormat::Bmp | ImageFormat::Tiff => 1.1,
                ImageFormat::Jpeg | ImageFormat::Heic | ImageFormat::Avif => ratio,
                ImageFormat::Webp => ratio * 0.9,
                ImageFormat::Gif => 0.8,
                ImageFormat::Any => ratio,
            }
        }
    }
}

pub fn estimate_item_output_bytes(
    source_format: ImageFormat,
    input_bytes: u64,
    preset: Preset,
    to_format: ImageFormat,
) -> u64 {
    let factor = pair_factor(source_format, to_format, preset);
    ((input_bytes as f64) * factor).round().max(1.0) as u64
}

pub fn estimate_batch_output_bytes(
    items: &[QueueItem],
    preset: Preset,
    to_format: ImageFormat,
) -> u64 {
    items
        .iter()
        .map(|item| {
            estimate_item_output_bytes(item.source_format, item.size_bytes, preset, to_format)
        })
        .sum()
}

/// True when ratio-based estimate is unreliable (HEIC → lossless-ish target).
pub fn estimate_low_confidence(items: &[QueueItem], to_format: ImageFormat) -> bool {
    if to_format != ImageFormat::Png {
        return false;
    }
    let heic_count = items
        .iter()
        .filter(|i| i.source_format == ImageFormat::Heic)
        .count();
    !items.is_empty() && heic_count * 2 > items.len()
}

pub fn estimate_warning(items: &[QueueItem], to_format: ImageFormat) -> Option<String> {
    if estimate_low_confidence(items, to_format) {
        Some(
            "HEIC → PNG often produces much larger files than the HEIC originals. Use Estimate again for a sample-based size (decodes up to 3 files)."
                .into(),
        )
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ImageFormat, QueueItem};

    #[test]
    fn heic_to_png_expands_more_than_flat_ratio() {
        let heic_est =
            estimate_item_output_bytes(ImageFormat::Heic, 500_000, Preset::Web, ImageFormat::Png);
        let naive =
            estimate_item_output_bytes(ImageFormat::Png, 500_000, Preset::Web, ImageFormat::Png);
        assert!(heic_est > naive * 3);
    }

    #[test]
    fn sanitize_replaces_invalid_chars() {
        assert_eq!(sanitize_output_stem("a<b>c:d"), "a_b_c_d");
        assert_eq!(sanitize_output_stem("a<b>"), "a_b");
    }

    #[test]
    fn empty_sanitize_becomes_image() {
        assert_eq!(sanitize_output_stem("<<<"), "image");
        assert_eq!(sanitize_output_stem("  hello__  "), "hello");
    }

    #[test]
    fn low_confidence_when_mostly_heic_to_png() {
        let items = vec![
            QueueItem {
                id: "1".into(),
                batch_id: "b".into(),
                source_path: "/a.heic".into(),
                relative_path: "a.heic".into(),
                source_format: ImageFormat::Heic,
                size_bytes: 100,
                zip_source_path: None,
                output_base_name: None,
            },
            QueueItem {
                id: "2".into(),
                batch_id: "b".into(),
                source_path: "/b.heic".into(),
                relative_path: "b.heic".into(),
                source_format: ImageFormat::Heic,
                size_bytes: 100,
                zip_source_path: None,
                output_base_name: None,
            },
        ];
        assert!(estimate_low_confidence(&items, ImageFormat::Png));
        assert!(estimate_warning(&items, ImageFormat::Png).is_some());
    }
}
