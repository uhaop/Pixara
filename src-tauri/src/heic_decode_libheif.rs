//! HEIC/HEIF decode with libheif (CPU) and optional bundled FFmpeg hardware path (Windows).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use image::{DynamicImage, Rgba, RgbaImage};
use libheif_rs::{ColorSpace, HeifContext, ImageHandle, LibHeif, RgbChroma};

use crate::heic_decode::parse_ffmpeg_video_size;
use crate::system::{bundled_ffmpeg_path, preferred_heic_hwaccel, probe_ffmpeg_hwaccels};
use crate::types::GvError;

static LIBHEIF: OnceLock<LibHeif> = OnceLock::new();

fn libheif() -> &'static LibHeif {
    LIBHEIF.get_or_init(LibHeif::new)
}

trait HeicDecoder {
    fn decode(&self, path: &Path) -> Result<DynamicImage, GvError>;
}

struct LibheifDecoder;

impl HeicDecoder for LibheifDecoder {
    fn decode(&self, path: &Path) -> Result<DynamicImage, GvError> {
        decode_libheif(path)
    }
}

#[cfg(windows)]
#[derive(Clone)]
struct FfmpegHwDecoder {
    ffmpeg: PathBuf,
    hwaccel: String,
}

#[cfg(windows)]
impl HeicDecoder for FfmpegHwDecoder {
    fn decode(&self, path: &Path) -> Result<DynamicImage, GvError> {
        decode_ffmpeg_hw(&self.ffmpeg, &self.hwaccel, path)
    }
}

#[cfg(windows)]
fn primary_hw_decoder() -> Option<FfmpegHwDecoder> {
    static DECODER: OnceLock<Option<FfmpegHwDecoder>> = OnceLock::new();
    DECODER
        .get_or_init(|| {
            let ffmpeg = bundled_ffmpeg_path()?;
            let hwaccels = probe_ffmpeg_hwaccels(&ffmpeg);
            let hwaccel = preferred_heic_hwaccel(&hwaccels)?;
            Some(FfmpegHwDecoder { ffmpeg, hwaccel })
        })
        .clone()
}

pub fn load_heic(path: &Path) -> Result<DynamicImage, GvError> {
    let cpu = LibheifDecoder;

    #[cfg(windows)]
    if let Some(decoder) = primary_hw_decoder() {
        return match decoder.decode(path) {
            Ok(img) => Ok(img),
            Err(hw_err) => match cpu.decode(path) {
                Ok(img) => {
                    eprintln!(
                        "pixara: FFmpeg HEIC hardware decode failed, used CPU fallback: {hw_err}"
                    );
                    Ok(img)
                }
                Err(cpu_err) => Err(GvError::Message(format!(
                    "FFmpeg hardware decode failed ({hw_err}); CPU fallback also failed: {cpu_err}"
                ))),
            },
        };
    }

    cpu.decode(path)
}

fn decode_libheif(path: &Path) -> Result<DynamicImage, GvError> {
    let path_str = path
        .to_str()
        .ok_or_else(|| GvError::Message("HEIC path is not valid UTF-8".into()))?;
    let context =
        HeifContext::read_from_file(path_str).map_err(|e| GvError::Heif(e.to_string()))?;
    let handle = context
        .primary_image_handle()
        .map_err(|e| GvError::Heif(e.to_string()))?;
    decode_handle(&handle)
}

#[cfg(windows)]
fn decode_ffmpeg_hw(ffmpeg: &Path, hwaccel: &str, path: &Path) -> Result<DynamicImage, GvError> {
    let path_str = path
        .to_str()
        .ok_or_else(|| GvError::Message("HEIC path is not valid UTF-8".into()))?;

    let (width, height) = probe_ffmpeg_video_size(ffmpeg, path_str)?;
    let expected_len = width as usize * height as usize * 4;

    let output = Command::new(ffmpeg)
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-hwaccel",
            hwaccel,
            "-i",
            path_str,
            "-frames:v",
            "1",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgba",
            "-",
        ])
        .output()
        .map_err(|e| GvError::Message(format!("ffmpeg spawn failed: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(GvError::Message(format!(
            "ffmpeg hw decode failed ({hwaccel}): {stderr}"
        )));
    }

    if output.stdout.len() != expected_len {
        return Err(GvError::Message(format!(
            "ffmpeg rgba buffer size mismatch: expected {expected_len}, got {}",
            output.stdout.len()
        )));
    }

    let rgba = RgbaImage::from_raw(width, height, output.stdout).ok_or_else(|| {
        GvError::Message(format!("invalid rgba buffer {width}x{height}"))
    })?;
    Ok(DynamicImage::ImageRgba8(rgba))
}

#[cfg(windows)]
fn probe_ffmpeg_video_size(ffmpeg: &Path, path_str: &str) -> Result<(u32, u32), GvError> {
    let output = Command::new(ffmpeg)
        .args(["-hide_banner", "-i", path_str])
        .output()
        .map_err(|e| GvError::Message(format!("ffmpeg probe failed: {e}")))?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    parse_ffmpeg_video_size(&stderr).ok_or_else(|| {
        GvError::Message("ffmpeg probe could not determine video dimensions".into())
    })
}

fn decode_handle(handle: &ImageHandle) -> Result<DynamicImage, GvError> {
    let color_space = handle
        .preferred_decoding_colorspace()
        .map_err(|e| GvError::Heif(e.to_string()))?;
    let decoded = libheif()
        .decode(handle, color_space, None)
        .map_err(|e| GvError::Heif(e.to_string()))?;

    let planes = decoded.planes();
    let plane = planes
        .interleaved
        .ok_or_else(|| GvError::Heif("HEIC image is not interleaved".into()))?;

    let width = handle.width();
    let height = handle.height();
    let bpp = plane.storage_bits_per_pixel;
    let row_bytes = width as usize * (bpp / 8) as usize;
    if row_bytes > plane.stride {
        return Err(GvError::Heif(format!(
            "HEIC row size {row_bytes} exceeds stride {}",
            plane.stride
        )));
    }

    match (color_space, bpp) {
        (ColorSpace::Rgb(RgbChroma::Rgb | RgbChroma::Rgba), 24 | 32) => {
            let mut rgba = RgbaImage::new(width, height);
            copy_interleaved_rows(&plane.data, plane.stride, height, row_bytes, bpp, &mut rgba);
            Ok(DynamicImage::ImageRgba8(rgba))
        }
        (ColorSpace::Rgb(chroma), 48 | 64) if matches!(
            chroma,
            RgbChroma::HdrRgbLe
                | RgbChroma::HdrRgbBe
                | RgbChroma::HdrRgbaLe
                | RgbChroma::HdrRgbaBe
        ) => {
            let mut rgba = RgbaImage::new(width, height);
            copy_interleaved_rows_hdr(&plane.data, plane.stride, height, row_bytes, bpp, &mut rgba);
            Ok(DynamicImage::ImageRgba8(rgba))
        }
        _ => Err(GvError::Heif(format!(
            "unsupported HEIC colorspace {:?} at {} bpp",
            color_space, bpp
        ))),
    }
}

fn copy_interleaved_rows(
    data: &[u8],
    stride: usize,
    height: u32,
    row_bytes: usize,
    bpp: u8,
    out: &mut RgbaImage,
) {
    let channels = (bpp / 8) as usize;
    for y in 0..height as usize {
        let src_row = &data[y * stride..y * stride + row_bytes];
        for x in 0..out.width() as usize {
            let i = x * channels;
            let r = src_row[i];
            let g = src_row[i + 1];
            let b = src_row[i + 2];
            let a = if channels == 4 { src_row[i + 3] } else { 255 };
            out.put_pixel(x as u32, y as u32, Rgba([r, g, b, a]));
        }
    }
}

fn copy_interleaved_rows_hdr(
    data: &[u8],
    stride: usize,
    height: u32,
    row_bytes: usize,
    bpp: u8,
    out: &mut RgbaImage,
) {
    let channels = (bpp / 8) as usize;
    for y in 0..height as usize {
        let src_row = &data[y * stride..y * stride + row_bytes];
        for x in 0..out.width() as usize {
            let i = x * channels;
            let r = (u16::from_le_bytes([src_row[i], src_row[i + 1]]) >> 8) as u8;
            let g = (u16::from_le_bytes([src_row[i + 2], src_row[i + 3]]) >> 8) as u8;
            let b = (u16::from_le_bytes([src_row[i + 4], src_row[i + 5]]) >> 8) as u8;
            let a = if channels == 8 {
                (u16::from_le_bytes([src_row[i + 6], src_row[i + 7]]) >> 8) as u8
            } else {
                255
            };
            out.put_pixel(x as u32, y as u32, Rgba([r, g, b, a]));
        }
    }
}
