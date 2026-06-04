use std::fs::{self, File};
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
#[cfg(feature = "heic")]
use std::sync::Once;
use std::time::Instant;

use image::codecs::avif::AvifEncoder;
use image::codecs::bmp::BmpEncoder;
use image::codecs::gif::GifEncoder;
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::{CompressionType, PngEncoder, FilterType as PngFilterType};
use image::codecs::tiff::TiffEncoder;
use image::codecs::webp::WebPEncoder;
use image::{DynamicImage, ExtendedColorType, ImageEncoder};

use crate::cancel::is_batch_cancelled;
use crate::heic_decode;
#[cfg(feature = "heic")]
use libheif_rs::{
    Channel, ColorSpace, CompressionFormat, EncoderQuality, HeifContext, Image, LibHeif, RgbChroma,
};
use crate::formats::{EncodeSettings, PngCompression};
use crate::metadata::{
    apply_exif_orientation, flatten_alpha, parse_flatten_color, read_icc_profile, resize_max,
};
use crate::privacy_strip::strip_privacy_metadata;
use crate::naming::output_file_name;
use crate::supported::{extension_for_format, format_from_path};
use crate::types::{ConvertSettings, ConvertStageMs, GvError, ImageFormat, OutputMode, OverwriteMode, QueueItem};

#[cfg(feature = "heic")]
static HEIF_HOOKS: Once = Once::new();

#[cfg(feature = "heic")]
fn ensure_heif_hooks() {
    HEIF_HOOKS.call_once(|| {
        libheif_rs::integration::image::register_all_decoding_hooks();
    });
}

fn check_batch_cancelled() -> Result<(), GvError> {
    if is_batch_cancelled() {
        return Err(GvError::Message("cancelled".into()));
    }
    Ok(())
}

pub fn convert_one(item: &QueueItem, settings: &ConvertSettings) -> Result<PathBuf, GvError> {
    convert_one_inner(item, settings, None)
}

pub fn convert_one_timed(
    item: &QueueItem,
    settings: &ConvertSettings,
) -> Result<(PathBuf, ConvertStageMs), GvError> {
    let mut stages = ConvertStageMs::default();
    let path = convert_one_inner(item, settings, Some(&mut stages))?;
    Ok((path, stages))
}

fn convert_one_inner(
    item: &QueueItem,
    settings: &ConvertSettings,
    mut stages: Option<&mut ConvertStageMs>,
) -> Result<PathBuf, GvError> {
    check_batch_cancelled()?;

    if settings.to_format.is_any() {
        return Err(GvError::InvalidSettings(
            "Target format cannot be Any".into(),
        ));
    }

    let source = PathBuf::from(&item.source_path);
    if settings.skip_same_format {
        if let Some(src_fmt) = format_from_path(&source) {
            if src_fmt == settings.to_format {
                return Err(GvError::Message("skipped_same_format".into()));
            }
        }
    }

    validate_resize_limits(settings)?;

    let decode_start = Instant::now();
    let mut img = load_image(&source)?;
    if let Some(s) = stages.as_deref_mut() {
        s.decode_ms = decode_start.elapsed().as_millis() as u64;
    }
    check_batch_cancelled()?;

    let transform_start = Instant::now();
    img = apply_exif_orientation(img, &source);
    img = resize_max(img, settings.max_width, settings.max_height);
    if let Some(s) = stages.as_deref_mut() {
        s.transform_ms = transform_start.elapsed().as_millis() as u64;
    }
    check_batch_cancelled()?;

    let output = resolve_output_path(item, settings)?;
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }

    let flatten_bg = parse_flatten_color(&settings.flatten_color);
    check_batch_cancelled()?;
    save_image(&img, &source, &output, settings, flatten_bg, stages)?;
    Ok(output)
}

pub fn load_image(path: &Path) -> Result<DynamicImage, GvError> {
    if format_from_path(path) == Some(ImageFormat::Heic) {
        match heic_decode::load_heic(path) {
            Ok(img) => return Ok(img),
            Err(e) => {
                #[cfg(not(feature = "heic"))]
                return Err(e);
                #[cfg(feature = "heic")]
                {
                    let _ = e;
                    ensure_heif_hooks();
                }
            }
        }
    }
    image::open(path).map_err(GvError::from)
}

pub fn save_image(
    img: &DynamicImage,
    source_path: &Path,
    path: &Path,
    settings: &ConvertSettings,
    flatten_bg: [u8; 3],
    stages: Option<&mut ConvertStageMs>,
) -> Result<(), GvError> {
    let encode_start = Instant::now();
    let encode = EncodeSettings::from_preset(settings.preset);
    let icc = if settings.strip_icc {
        None
    } else {
        read_icc_profile(source_path)
    };

    match settings.to_format {
        ImageFormat::Jpeg => {
            let rgb = rgb_for_flatten_target(img, flatten_bg);
            let file = File::create(path)?;
            let mut writer = BufWriter::new(file);
            JpegEncoder::new_with_quality(&mut writer, encode.jpeg_quality).write_image(
                rgb.as_raw(),
                rgb.width(),
                rgb.height(),
                ExtendedColorType::Rgb8,
            )?;
        }
        ImageFormat::Png => {
            let compression = png_image_compression(encode.png_compression);
            let rgba = img.to_rgba8();
            if let Some(profile) = icc.as_deref() {
                save_png_with_icc(&rgba, path, profile, encode.png_compression)?;
            } else {
                let file = File::create(path)?;
                let mut writer = BufWriter::new(file);
                PngEncoder::new_with_quality(
                    &mut writer,
                    compression,
                    PngFilterType::Adaptive,
                )
                .write_image(
                    rgba.as_raw(),
                    rgba.width(),
                    rgba.height(),
                    ExtendedColorType::Rgba8,
                )?;
            }
        }
        ImageFormat::Webp | ImageFormat::Gif | ImageFormat::Tiff | ImageFormat::Avif => {
            let rgba = img.to_rgba8();
            match settings.to_format {
                ImageFormat::Webp => {
                    if encode.webp_quality >= 99.0 {
                        let file = File::create(path)?;
                        let mut writer = BufWriter::new(file);
                        WebPEncoder::new_lossless(&mut writer).write_image(
                            rgba.as_raw(),
                            rgba.width(),
                            rgba.height(),
                            ExtendedColorType::Rgba8,
                        )?;
                    } else {
                        let encoder =
                            webp::Encoder::from_rgba(rgba.as_raw(), rgba.width(), rgba.height());
                        let webp_data = encoder.encode(encode.webp_quality);
                        fs::write(path, &*webp_data)?;
                    }
                }
                ImageFormat::Gif => {
                    let file = File::create(path)?;
                    let mut writer = BufWriter::new(file);
                    let mut encoder = GifEncoder::new(&mut writer);
                    encoder.encode(
                        rgba.as_raw(),
                        rgba.width(),
                        rgba.height(),
                        ExtendedColorType::Rgba8,
                    )?;
                }
                ImageFormat::Tiff => {
                    let file = File::create(path)?;
                    let mut writer = BufWriter::new(file);
                    TiffEncoder::new(&mut writer).write_image(
                        rgba.as_raw(),
                        rgba.width(),
                        rgba.height(),
                        ExtendedColorType::Rgba8,
                    )?;
                }
                ImageFormat::Avif => {
                    let file = File::create(path)?;
                    let mut writer = BufWriter::new(file);
                    AvifEncoder::new_with_speed_quality(
                        &mut writer,
                        encode.avif_speed,
                        encode.avif_quality,
                    )
                    .write_image(
                        rgba.as_raw(),
                        rgba.width(),
                        rgba.height(),
                        ExtendedColorType::Rgba8,
                    )?;
                }
                _ => unreachable!(),
            }
        }
        ImageFormat::Bmp => {
            let rgb = img.to_rgb8();
            let file = File::create(path)?;
            let mut writer = BufWriter::new(file);
            BmpEncoder::new(&mut writer).write_image(
                rgb.as_raw(),
                rgb.width(),
                rgb.height(),
                ExtendedColorType::Rgb8,
            )?;
        }
        ImageFormat::Heic => {
            let flat = flatten_alpha(img, flatten_bg);
            save_heic(&flat, path, encode.heic_quality)?;
        }
        ImageFormat::Any => return Err(GvError::UnsupportedFormat),
    }

    let encode_ms = encode_start.elapsed().as_millis() as u64;
    let post_start = Instant::now();

    let keep_icc = !settings.strip_icc;
    strip_privacy_metadata(path, keep_icc)?;

    if settings.to_format == ImageFormat::Png && settings.optimize_png {
        check_batch_cancelled()?;
        crate::png_optimize::optimize_png_file(path, settings.preset)?;
    }

    if let Some(s) = stages {
        s.encode_ms = encode_ms;
        s.post_ms = post_start.elapsed().as_millis() as u64;
    }
    Ok(())
}

fn rgb_for_flatten_target(img: &DynamicImage, flatten_bg: [u8; 3]) -> image::RgbImage {
    if img.color().has_alpha() {
        flatten_alpha(img, flatten_bg).to_rgb8()
    } else {
        img.to_rgb8()
    }
}

#[cfg(not(feature = "heic"))]
fn save_heic(_img: &DynamicImage, _path: &Path, _quality: f32) -> Result<(), GvError> {
    Err(GvError::Message(heic_decode::HEIC_UNAVAILABLE.into()))
}

#[cfg(feature = "heic")]
fn save_heic(img: &DynamicImage, path: &Path, quality: f32) -> Result<(), GvError> {
    let rgb = img.to_rgb8();
    let (width, height) = rgb.dimensions();

    let mut heif_image =
        Image::new(width, height, ColorSpace::Rgb(RgbChroma::Rgb)).map_err(|e| GvError::Heif(e.to_string()))?;
    heif_image
        .create_plane(Channel::Interleaved, width, height, 8)
        .map_err(|e| GvError::Heif(e.to_string()))?;

    {
        let mut planes = heif_image.planes_mut();
        let plane = planes
            .interleaved
            .as_mut()
            .ok_or_else(|| GvError::Heif("missing interleaved plane".into()))?;
        let stride = plane.stride;
        let data = &mut plane.data;
        for y in 0..height {
            if y % 32 == 0 {
                if is_batch_cancelled() {
                    return Err(GvError::Message("cancelled".into()));
                }
            }
            for x in 0..width {
                let pixel = rgb.get_pixel(x, y);
                let dst = y as usize * stride + x as usize * 3;
                if dst + 2 >= data.len() {
                    return Err(GvError::Heif(format!(
                        "plane buffer too small for pixel ({x},{y}): need {} bytes, got {}",
                        dst + 3,
                        data.len()
                    )));
                }
                data[dst] = pixel[0];
                data[dst + 1] = pixel[1];
                data[dst + 2] = pixel[2];
            }
        }
    }

    let lib_heif = LibHeif::new();
    let mut context = HeifContext::new().map_err(|e| GvError::Heif(e.to_string()))?;
    let mut encoder = lib_heif
        .encoder_for_format(CompressionFormat::Av1)
        .map_err(|e| GvError::Heif(e.to_string()))?;
    let q = quality.clamp(0.0, 100.0) as u8;
    encoder
        .set_quality(EncoderQuality::Lossy(q))
        .map_err(|e| GvError::Heif(e.to_string()))?;
    context
        .encode_image(&heif_image, &mut encoder, None)
        .map_err(|e| GvError::Heif(e.to_string()))?;
    context
        .write_to_file(path.to_string_lossy().as_ref())
        .map_err(|e| GvError::Heif(e.to_string()))?;
    Ok(())
}

pub fn resolve_output_path(item: &QueueItem, settings: &ConvertSettings) -> Result<PathBuf, GvError> {
    let source = PathBuf::from(&item.source_path);
    let ext = extension_for_format(settings.to_format);
    let file_name = output_file_name(item, &source, ext, settings.naming);

    let base = match settings.output_mode {
        OutputMode::SameFolder => same_folder_base(item, &source),
        OutputMode::CustomDir => {
            let dir = settings
                .custom_output_dir
                .as_ref()
                .ok_or_else(|| GvError::InvalidSettings("custom output dir required".into()))?;
            PathBuf::from(dir)
        }
    };

    let base = if settings.preserve_structure {
        let rel = Path::new(&item.relative_path);
        if let Some(parent) = rel.parent() {
            base.join(parent)
        } else {
            base
        }
    } else {
        base
    };

    let candidate = base.join(file_name);
    match settings.overwrite_mode {
        OverwriteMode::AutoRename => Ok(unique_path(candidate)),
        OverwriteMode::Replace => Ok(candidate),
        OverwriteMode::Skip => {
            if candidate.exists() {
                Err(GvError::Message("skipped_exists".into()))
            } else {
                Ok(candidate)
            }
        }
    }
}

/// ZIP extracts live under temp; write beside the original archive instead.
fn same_folder_base(item: &QueueItem, source: &Path) -> PathBuf {
    if let Some(zip_path) = &item.zip_source_path {
        let zip = PathBuf::from(zip_path);
        if let Some(parent) = zip.parent() {
            return parent.to_path_buf();
        }
    }
    source
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn save_png_with_icc(
    rgba: &image::RgbaImage,
    path: &Path,
    icc: &[u8],
    compression: PngCompression,
) -> Result<(), GvError> {
    use std::borrow::Cow;

    let file = File::create(path)?;
    let (width, height) = rgba.dimensions();
    let mut info = png::Info::with_size(width, height);
    info.bit_depth = png::BitDepth::Eight;
    info.color_type = png::ColorType::Rgba;
    info.icc_profile = Some(Cow::Borrowed(icc));
    info.compression = png_raw_compression(compression);
    let encoder = png::Encoder::with_info(file, info)
        .map_err(|e| GvError::Message(format!("PNG encode: {e}")))?;
    let mut writer = encoder
        .write_header()
        .map_err(|e| GvError::Message(format!("PNG encode: {e}")))?;
    writer
        .write_image_data(rgba.as_raw())
        .map_err(|e| GvError::Message(format!("PNG encode: {e}")))?;
    Ok(())
}

fn png_image_compression(level: PngCompression) -> CompressionType {
    match level {
        PngCompression::Default => CompressionType::Default,
        PngCompression::Best => CompressionType::Best,
    }
}

fn png_raw_compression(level: PngCompression) -> png::Compression {
    match level {
        PngCompression::Default => png::Compression::Default,
        PngCompression::Best => png::Compression::Best,
    }
}

fn validate_resize_limits(settings: &ConvertSettings) -> Result<(), GvError> {
    if settings.max_width == Some(0) || settings.max_height == Some(0) {
        return Err(GvError::InvalidSettings(
            "max width and height must be greater than zero".into(),
        ));
    }
    Ok(())
}

fn unique_path_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

pub fn unique_path(path: PathBuf) -> PathBuf {
    let _guard = unique_path_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    if !path.exists() {
        return path;
    }
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_string());
    let stem_path = path.with_extension("");
    let base_stem = stem_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("image")
        .to_string();
    let parent = path.parent().map(PathBuf::from).unwrap_or_default();

    for i in 1..10_000 {
        let new_name = match &ext {
            Some(e) => format!("{base_stem}_{i}.{e}"),
            None => format!("{base_stem}_{i}"),
        };
        let candidate = parent.join(new_name);
        if !candidate.exists() {
            return candidate;
        }
    }

    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    match &ext {
        Some(e) => parent.join(format!("{base_stem}_{stamp}.{e}")),
        None => parent.join(format!("{base_stem}_{stamp}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ConvertSettings, ImageFormat, NamingMode, OutputMode, Preset, QueueItem};

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
            overwrite_mode: crate::types::OverwriteMode::AutoRename,
            optimize_png: true,
            slow_drive_mode: false,
        }
    }

    fn sample_item(_dir: &Path, source: &Path, relative: &str) -> QueueItem {
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
    use image::{ImageBuffer, Rgb};
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

    fn write_jpeg_with_fake_exif_app1(path: &Path) {
        use std::io::Write;

        let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_fn(8, 8, |_, _| Rgb([100, 150, 200]));
        {
            use image::codecs::jpeg::JpegEncoder;
            use image::{ExtendedColorType, ImageEncoder};
            let file = File::create(path).expect("create jpeg");
            let mut writer = BufWriter::new(file);
            JpegEncoder::new_with_quality(&mut writer, 90)
                .write_image(img.as_raw(), 8, 8, ExtendedColorType::Rgb8)
                .expect("encode");
            writer.flush().expect("flush");
        }
        let bytes = fs::read(path).expect("read jpeg");
        if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xD8 {
            let mut injected = vec![0xFF, 0xD8];
            injected.extend_from_slice(&[0xFF, 0xE1, 0x00, 0x10]);
            injected.extend_from_slice(b"Exif\0\0GPSFAKE123");
            injected.extend_from_slice(&bytes[2..]);
            fs::write(path, injected).expect("inject app1");
        }
    }

    fn write_test_png(path: &Path) {
        let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_fn(32, 32, |x, y| {
            if (x + y) % 2 == 0 {
                Rgb([220, 80, 80])
            } else {
                Rgb([40, 120, 220])
            }
        });
        img.save(path).expect("write png");
    }

    #[test]
    fn convert_strips_privacy_metadata_from_output() {
        use crate::privacy_strip::file_has_privacy_exif;

        let dir = temp_dir("gv-pixara-privacy");
        let source = dir.join("gps.jpg");
        write_jpeg_with_fake_exif_app1(&source);

        let item = sample_item(&dir, &source, "gps.jpg");
        let mut settings = base_settings();
        settings.to_format = ImageFormat::Jpeg;

        let output = convert_one(&item, &settings).expect("convert");
        assert!(output.exists());
        assert!(!file_has_privacy_exif(&output));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn convert_png_to_webp_in_same_folder() {
        let dir = temp_dir("gv-pixara-test");
        let source = dir.join("sample.png");
        write_test_png(&source);

        let item = sample_item(&dir, &source, "sample.png");
        let settings = base_settings();

        let output = convert_one(&item, &settings).expect("convert");
        assert_eq!(output.extension().and_then(|e| e.to_str()), Some("webp"));
        assert!(output.exists());
        assert!(fs::metadata(&output).expect("output metadata").len() > 0);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn skip_same_format_when_enabled() {
        let dir = temp_dir("gv-pixara-skip");
        let source = dir.join("sample.png");
        write_test_png(&source);

        let item = sample_item(&dir, &source, "sample.png");
        let mut settings = base_settings();
        settings.to_format = ImageFormat::Png;
        settings.skip_same_format = true;

        let err = convert_one(&item, &settings).expect_err("should skip");
        assert!(matches!(err, GvError::Message(ref m) if m == "skipped_same_format"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn appends_suffix_and_avoids_collisions() {
        let dir = temp_dir("gv-pixara-naming");
        let source = dir.join("sample.png");
        write_test_png(&source);

        let existing = dir.join("sample_converted.webp");
        fs::write(&existing, b"already-here").expect("seed existing output");

        let item = sample_item(&dir, &source, "sample.png");
        let mut settings = base_settings();
        settings.naming = NamingMode::AppendSuffix;

        let output = convert_one(&item, &settings).expect("convert");
        assert_eq!(output.file_name().and_then(|n| n.to_str()), Some("sample_converted_1.webp"));
        assert!(output.exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn preserves_relative_structure_in_custom_output_dir() {
        let dir = temp_dir("gv-pixara-structure");
        let source_dir = dir.join("source").join("nested");
        fs::create_dir_all(&source_dir).expect("create source dir");
        let source = source_dir.join("sample.png");
        write_test_png(&source);

        let custom_output = dir.join("output");
        fs::create_dir_all(&custom_output).expect("create output dir");

        let item = sample_item(&dir, &source, "nested/sample.png");
        let mut settings = base_settings();
        settings.to_format = ImageFormat::Jpeg;
        settings.output_mode = OutputMode::CustomDir;
        settings.custom_output_dir = Some(custom_output.to_string_lossy().into_owned());

        let output = convert_one(&item, &settings).expect("convert");
        let expected = custom_output.join("nested").join("sample.jpg");
        assert_eq!(output, expected);
        assert!(output.exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn zip_same_folder_writes_beside_archive() {
        let dir = temp_dir("gv-pixara-zip-out");
        let zip_path = dir.join("bundle.zip");
        fs::write(&zip_path, b"zip-placeholder").expect("write zip");

        let extract_dir = dir.join("extracted");
        fs::create_dir_all(&extract_dir).expect("mkdir extract");
        let source = extract_dir.join("nested").join("sample.png");
        fs::create_dir_all(source.parent().unwrap()).expect("mkdir nested");
        write_test_png(&source);

        let mut item = sample_item(&dir, &source, "nested/sample.png");
        item.zip_source_path = Some(zip_path.to_string_lossy().into_owned());

        let settings = base_settings();
        let output = convert_one(&item, &settings).expect("convert");
        let expected = dir.join("nested").join("sample.webp");
        assert_eq!(output, expected);
        assert!(output.exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn rejects_zero_resize_limits() {
        let dir = temp_dir("gv-pixara-resize");
        let source = dir.join("sample.png");
        write_test_png(&source);

        let item = sample_item(&dir, &source, "sample.png");
        let mut settings = base_settings();
        settings.max_width = Some(0);

        let err = convert_one(&item, &settings).expect_err("zero resize should fail");
        assert!(matches!(err, GvError::InvalidSettings(_)));

        let _ = fs::remove_dir_all(dir);
    }
}
