//! Shared helpers and fixtures for conversion / privacy tests.

use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use image::codecs::jpeg::JpegEncoder;
use image::{ExtendedColorType, ImageBuffer, ImageEncoder, Rgb};

use crate::engine::{convert_one, load_image, save_image};
use crate::metadata::parse_flatten_color;
use crate::privacy_strip::file_has_privacy_exif;
use crate::supported::{extension_for_format, format_from_path};
use crate::types::{
    ConvertSettings, ImageFormat, NamingMode, OutputMode, OverwriteMode, Preset,
    QueueItem,
};

pub fn temp_dir(prefix: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("{prefix}-{stamp}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

pub fn write_test_png(path: &Path) {
    let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_fn(32, 32, |x, y| {
        if (x + y) % 2 == 0 {
            Rgb([220, 80, 80])
        } else {
            Rgb([40, 120, 220])
        }
    });
    img.save(path).expect("write png");
}

pub fn write_test_gif(path: &Path) {
    use image::codecs::gif::GifEncoder;
    let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_fn(8, 8, |_, _| Rgb([90, 120, 180]));
    let file = File::create(path).expect("create gif");
    let mut writer = BufWriter::new(file);
    GifEncoder::new(&mut writer)
        .encode(img.as_raw(), 8, 8, ExtendedColorType::Rgb8)
        .expect("encode gif");
}

pub fn write_test_bmp(path: &Path) {
    let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_fn(8, 8, |_, _| Rgb([100, 150, 200]));
    img.save(path).expect("write bmp");
}

pub fn write_test_tiff(path: &Path) {
    let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_fn(8, 8, |_, _| Rgb([100, 150, 200]));
    img.save(path).expect("write tiff");
}

pub fn write_test_webp(path: &Path) {
    let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_fn(8, 8, |_, _| Rgb([100, 150, 200]));
    let rgba = image::DynamicImage::ImageRgb8(img).to_rgba8();
    let encoder = webp::Encoder::from_rgba(rgba.as_raw(), 8, 8);
    fs::write(path, &*encoder.encode(80.0)).expect("write webp");
}

pub fn write_test_avif(path: &Path) {
    use image::codecs::avif::AvifEncoder;
    let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_fn(8, 8, |_, _| Rgb([100, 150, 200]));
    let rgba = image::DynamicImage::ImageRgb8(img).to_rgba8();
    let file = File::create(path).expect("create avif");
    let mut writer = BufWriter::new(file);
    AvifEncoder::new_with_speed_quality(&mut writer, 6, 75)
        .write_image(rgba.as_raw(), 8, 8, ExtendedColorType::Rgba8)
        .expect("encode avif");
}

pub fn write_jpeg_with_fake_exif_app1(path: &Path) {
    let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_fn(8, 8, |_, _| Rgb([100, 150, 200]));
    {
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

pub fn base_settings(to_format: ImageFormat) -> ConvertSettings {
    ConvertSettings {
        to_format,
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

pub fn sample_item(source: &Path, relative: &str) -> QueueItem {
    QueueItem {
        id: uuid::Uuid::new_v4().to_string(),
        batch_id: "test-batch".into(),
        source_path: source.to_string_lossy().into_owned(),
        relative_path: relative.into(),
        source_format: ImageFormat::Png,
        size_bytes: fs::metadata(source).expect("metadata").len(),
        zip_source_path: None,
        output_base_name: None,
    }
}

pub fn convert_to_format(source: &Path, to_format: ImageFormat) -> PathBuf {
    let item = sample_item(source, source.file_name().unwrap().to_string_lossy().as_ref());
    convert_one(&item, &base_settings(to_format)).unwrap_or_else(|e| {
        panic!(
            "convert {:?} -> {:?} failed: {e}",
            source,
            extension_for_format(to_format)
        )
    })
}

pub fn try_convert_to_format(source: &Path, to_format: ImageFormat) -> Result<PathBuf, crate::types::GvError> {
    let item = sample_item(source, source.file_name().unwrap().to_string_lossy().as_ref());
    convert_one(&item, &base_settings(to_format))
}

pub fn assert_output_has_no_privacy_exif(output: &Path) {
    assert!(
        !file_has_privacy_exif(output),
        "privacy metadata found in {}",
        output.display()
    );
}

pub fn assert_roundtrip_loadable(path: &Path) {
    if format_from_path(path) == Some(ImageFormat::Avif) {
        assert!(path.is_file(), "missing avif output at {}", path.display());
        assert!(
            fs::metadata(path).map(|m| m.len()).unwrap_or(0) > 0,
            "empty avif output at {}",
            path.display()
        );
        return;
    }
    load_image(path).unwrap_or_else(|e| panic!("load {} failed: {e}", path.display()));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_target_formats() -> Vec<ImageFormat> {
        vec![
            ImageFormat::Png,
            ImageFormat::Jpeg,
            ImageFormat::Webp,
            ImageFormat::Gif,
            ImageFormat::Bmp,
            ImageFormat::Tiff,
            ImageFormat::Avif,
            ImageFormat::Heic,
        ]
    }

    #[test]
    fn convert_png_source_to_every_target_format() {
        let dir = temp_dir("gv-all-targets-png");
        let source = dir.join("sample.png");
        write_test_png(&source);

        for to_format in all_target_formats() {
            let output = match try_convert_to_format(&source, to_format) {
                Ok(path) => path,
                Err(e) if to_format == ImageFormat::Heic => {
                    eprintln!("skipping HEIC output test: {e}");
                    continue;
                }
                Err(e) => panic!("convert to {to_format:?} failed: {e}"),
            };
            assert!(output.exists(), "missing output for {to_format:?}");
            assert_eq!(
                output.extension().and_then(|e| e.to_str()),
                Some(extension_for_format(to_format))
            );
            assert_roundtrip_loadable(&output);
        }

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn convert_gif_source_to_webp_and_jpeg() {
        let dir = temp_dir("gv-gif-src");
        let source = dir.join("anim.gif");
        write_test_gif(&source);

        for to_format in [ImageFormat::Webp, ImageFormat::Jpeg] {
            let output = convert_to_format(&source, to_format);
            assert!(output.exists());
            assert_roundtrip_loadable(&output);
        }

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn convert_bmp_source_to_png() {
        let dir = temp_dir("gv-bmp-src");
        let source = dir.join("sample.bmp");
        write_test_bmp(&source);
        let output = convert_to_format(&source, ImageFormat::Png);
        assert!(output.exists());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn convert_tiff_source_to_jpeg() {
        let dir = temp_dir("gv-tiff-src");
        let source = dir.join("sample.tiff");
        write_test_tiff(&source);
        let output = convert_to_format(&source, ImageFormat::Jpeg);
        assert!(output.exists());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn convert_webp_source_to_png() {
        let dir = temp_dir("gv-webp-src");
        let source = dir.join("sample.webp");
        write_test_webp(&source);
        let output = convert_to_format(&source, ImageFormat::Png);
        assert!(output.exists());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn convert_avif_source_to_png() {
        let dir = temp_dir("gv-avif-src");
        let source = dir.join("sample.avif");
        write_test_avif(&source);
        if load_image(&source).is_err() {
            let _ = fs::remove_dir_all(dir);
            return;
        }
        let output = convert_to_format(&source, ImageFormat::Png);
        assert!(output.exists());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn gps_jpeg_converted_to_every_format_has_no_privacy_exif() {
        let dir = temp_dir("gv-privacy-all-formats");
        let source = dir.join("gps.jpg");
        write_jpeg_with_fake_exif_app1(&source);

        for to_format in all_target_formats() {
            let output = match try_convert_to_format(&source, to_format) {
                Ok(path) => path,
                Err(e) if to_format == ImageFormat::Heic => {
                    eprintln!("skipping HEIC privacy test: {e}");
                    continue;
                }
                Err(e) => panic!("convert to {to_format:?} failed: {e}"),
            };
            assert_output_has_no_privacy_exif(&output);
        }

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn save_image_directly_strips_privacy_for_jpeg() {
        let dir = temp_dir("gv-save-strip");
        let source = dir.join("gps.jpg");
        let output = dir.join("out.jpg");
        write_jpeg_with_fake_exif_app1(&source);

        let img = load_image(&source).expect("load");
        let settings = base_settings(ImageFormat::Jpeg);
        save_image(
            &img,
            &source,
            &output,
            &settings,
            parse_flatten_color(&settings.flatten_color),
            None,
        )
        .expect("save");
        assert_output_has_no_privacy_exif(&output);

        let _ = fs::remove_dir_all(dir);
    }

    /// Run with `cargo test write_fixtures_to_disk -- --ignored` to materialize files on disk.
    #[test]
    #[ignore]
    fn write_fixtures_to_disk() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        fs::create_dir_all(&root).expect("mkdir fixtures");

        write_test_png(&root.join("sample.png"));
        write_test_gif(&root.join("sample.gif"));
        write_test_bmp(&root.join("sample.bmp"));
        write_test_tiff(&root.join("sample.tiff"));
        write_test_webp(&root.join("sample.webp"));
        write_test_avif(&root.join("sample.avif"));
        write_jpeg_with_fake_exif_app1(&root.join("gps.jpg"));

        for format in all_target_formats() {
            let source = root.join("sample.png");
            let output = match try_convert_to_format(&source, format) {
                Ok(path) => path,
                Err(e) if format == ImageFormat::Heic => {
                    eprintln!("skipping HEIC fixture write: {e}");
                    continue;
                }
                Err(e) => panic!("fixture convert to {format:?} failed: {e}"),
            };
            let ext = extension_for_format(format);
            let dest = root.join(format!("converted.{ext}"));
            fs::copy(&output, &dest).expect("copy converted fixture");
        }
    }
}
