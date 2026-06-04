use std::fs;
use std::io::{Cursor, Read};
use std::path::Path;

use exif::{In, Tag};
use image::codecs::tiff::TiffEncoder;
use image::{ExtendedColorType, ImageEncoder};

use crate::supported::format_from_path;
use crate::types::{GvError, ImageFormat};

/// Remove GPS, EXIF, XMP, IPTC, and other identity metadata from a written image file.
/// ICC (`iCCP` / JPEG APP2) is kept only when `keep_icc` is true (PNG-focused).
pub fn strip_privacy_metadata(path: &Path, keep_icc: bool) -> Result<(), GvError> {
    let format = format_from_path(path).ok_or_else(|| {
        GvError::Message(format!(
            "cannot strip metadata: unknown format for {}",
            path.display()
        ))
    })?;
    let bytes = fs::read(path)?;
    let stripped = match format {
        ImageFormat::Jpeg => strip_jpeg_metadata(&bytes, keep_icc)?,
        ImageFormat::Png => strip_png_metadata(&bytes, keep_icc)?,
        ImageFormat::Webp => strip_webp_metadata(&bytes)?,
        ImageFormat::Gif => strip_gif_metadata(&bytes)?,
        ImageFormat::Bmp => bytes.clone(),
        ImageFormat::Tiff => strip_tiff_metadata(&bytes)?,
        ImageFormat::Avif | ImageFormat::Heic => strip_isobmff_metadata(&bytes)?,
        ImageFormat::Any => bytes.clone(),
    };
    if stripped != bytes {
        fs::write(path, stripped)?;
    }

    if file_has_privacy_exif(path) {
        return Err(GvError::Message(format!(
            "privacy metadata remains in output after strip: {}",
            path.display()
        )));
    }
    Ok(())
}

/// Returns true if readable EXIF contains GPS or common identity tags.
#[cfg_attr(not(test), allow(dead_code))]
pub fn file_has_privacy_exif(path: &Path) -> bool {
    let Ok(file) = fs::File::open(path) else {
        return false;
    };
    let mut bufreader = std::io::BufReader::new(file);
    let Ok(exif) = exif::Reader::new().read_from_container(&mut bufreader) else {
        return false;
    };

    let gps_tags = [
        Tag::GPSLatitude,
        Tag::GPSLongitude,
        Tag::GPSAltitude,
        Tag::GPSLatitudeRef,
        Tag::GPSLongitudeRef,
    ];
    for tag in gps_tags {
        if exif.get_field(tag, In::PRIMARY).is_some() {
            return true;
        }
    }

    let identity_tags = [
        Tag::Make,
        Tag::Model,
        Tag::BodySerialNumber,
        Tag::LensModel,
        Tag::DateTime,
        Tag::DateTimeOriginal,
        Tag::Artist,
        Tag::Copyright,
        Tag::UserComment,
    ];
    for tag in identity_tags {
        if exif.get_field(tag, In::PRIMARY).is_some() {
            return true;
        }
    }

    false
}

fn strip_jpeg_metadata(data: &[u8], keep_icc: bool) -> Result<Vec<u8>, GvError> {
    if data.len() < 4 || data[0] != 0xFF || data[1] != 0xD8 {
        return Err(GvError::Message("invalid JPEG".into()));
    }

    let mut out = Vec::with_capacity(data.len());
    out.extend_from_slice(&data[0..2]);
    let mut i = 2usize;

    // Only parse marker segments before Start Of Scan; entropy-coded data may contain 0xFF bytes.
    while i + 1 < data.len() {
        if data[i] != 0xFF {
            return Err(GvError::Message("corrupt JPEG marker".into()));
        }

        let marker = data[i + 1];
        if marker == 0xD9 {
            out.extend_from_slice(&data[i..]);
            break;
        }
        if (0xD0..=0xD7).contains(&marker) || marker == 0x01 {
            out.extend_from_slice(&data[i..i + 2]);
            i += 2;
            continue;
        }

        if i + 3 >= data.len() {
            return Err(GvError::Message("truncated JPEG segment".into()));
        }
        let len = u16::from_be_bytes([data[i + 2], data[i + 3]]) as usize;
        if len < 2 || i + 2 + len > data.len() {
            return Err(GvError::Message("invalid JPEG segment length".into()));
        }
        let end = i + 2 + len;

        if marker == 0xDA {
            out.extend_from_slice(&data[i..]);
            break;
        }

        let is_app = (0xE0..=0xEF).contains(&marker);
        if is_app {
            let keep = marker == 0xE0 || (keep_icc && marker == 0xE2);
            if keep {
                out.extend_from_slice(&data[i..end]);
            }
        } else {
            out.extend_from_slice(&data[i..end]);
        }
        i = end;
    }

    Ok(out)
}

fn strip_png_metadata(data: &[u8], keep_icc: bool) -> Result<Vec<u8>, GvError> {
    const SIG: &[u8] = &[137, 80, 78, 71, 13, 10, 26, 10];
    if data.len() < SIG.len() || &data[..SIG.len()] != SIG {
        return Err(GvError::Message("invalid PNG".into()));
    }

    let mut out = Vec::with_capacity(data.len());
    out.extend_from_slice(SIG);

    let mut cursor = Cursor::new(&data[SIG.len()..]);
    loop {
        let mut len_buf = [0u8; 4];
        if cursor.read_exact(&mut len_buf).is_err() {
            break;
        }
        let length = u32::from_be_bytes(len_buf) as usize;

        let mut type_buf = [0u8; 4];
        cursor
            .read_exact(&mut type_buf)
            .map_err(|e| GvError::Message(format!("PNG chunk type: {e}")))?;

        let mut chunk_data = vec![0u8; length];
        if length > 0 {
            cursor
                .read_exact(&mut chunk_data)
                .map_err(|e| GvError::Message(format!("PNG chunk data: {e}")))?;
        }

        let mut crc_buf = [0u8; 4];
        cursor
            .read_exact(&mut crc_buf)
            .map_err(|e| GvError::Message(format!("PNG chunk crc: {e}")))?;

        let chunk_type = &type_buf;
        let keep = matches!(
            chunk_type,
            b"IHDR" | b"PLTE" | b"IDAT" | b"IEND" | b"sRGB" | b"gAMA" | b"cICP" | b"sBIT" | b"bKGD"
        ) || (keep_icc && chunk_type == b"iCCP");

        if keep {
            out.extend_from_slice(&len_buf);
            out.extend_from_slice(&type_buf);
            out.extend_from_slice(&chunk_data);
            out.extend_from_slice(&crc_buf);
        }

        if chunk_type == b"IEND" {
            break;
        }
    }

    Ok(out)
}

fn strip_webp_metadata(data: &[u8]) -> Result<Vec<u8>, GvError> {
    if data.len() < 12 || &data[0..4] != b"RIFF" || &data[8..12] != b"WEBP" {
        return Ok(data.to_vec());
    }

    let mut out = Vec::with_capacity(data.len());
    out.extend_from_slice(&data[0..12]);

    let mut pos = 12usize;
    while pos + 8 <= data.len() {
        let fourcc = &data[pos..pos + 4];
        let size = u32::from_le_bytes([
            data[pos + 4],
            data[pos + 5],
            data[pos + 6],
            data[pos + 7],
        ]) as usize;
        let padded = size + (size % 2);
        let end = pos + 8 + padded;
        if end > data.len() {
            break;
        }

        let drop = fourcc == b"EXIF" || fourcc == b"XMP ";
        if !drop {
            out.extend_from_slice(&data[pos..end]);
        }
        pos = end;
    }

    if out.len() >= 8 {
        let chunk_payload = out.len() - 8;
        out[4..8].copy_from_slice(&(chunk_payload as u32).to_le_bytes());
    }

    Ok(out)
}

/// Drop GIF comment and application extension blocks that can carry arbitrary text.
fn strip_gif_metadata(data: &[u8]) -> Result<Vec<u8>, GvError> {
    if data.len() < 6 || &data[0..3] != b"GIF" {
        return Ok(data.to_vec());
    }

    let mut out = Vec::with_capacity(data.len());
    out.extend_from_slice(&data[0..6]);
    let mut pos = 6usize;

    if pos >= data.len() {
        return Ok(out);
    }
    let flags = data[pos];
    pos += 1;
    out.push(flags);
    if pos + 2 > data.len() {
        return Err(GvError::Message("truncated GIF logical screen".into()));
    }
    out.extend_from_slice(&data[pos..pos + 2]);
    pos += 2;

    if flags & 0x80 != 0 {
        let table_size = 3usize << (flags & 0x07);
        if pos + table_size > data.len() {
            return Err(GvError::Message("truncated GIF color table".into()));
        }
        out.extend_from_slice(&data[pos..pos + table_size]);
        pos += table_size;
    }

    while pos < data.len() {
        if data[pos] == 0x3B {
            out.push(0x3B);
            break;
        }
        if data[pos] != 0x21 {
            out.extend_from_slice(&data[pos..]);
            break;
        }
        if pos + 2 >= data.len() {
            return Err(GvError::Message("truncated GIF extension".into()));
        }
        let label = data[pos + 1];
        let drop = label == 0xFE || label == 0xFF;
        if !drop {
            out.push(data[pos]);
            out.push(data[pos + 1]);
            pos += 2;
            if pos >= data.len() {
                return Err(GvError::Message("truncated GIF extension".into()));
            }
            let mut sub_len = data[pos] as usize;
            out.push(data[pos]);
            pos += 1;
            while sub_len > 0 {
                if pos + sub_len > data.len() {
                    return Err(GvError::Message("truncated GIF extension data".into()));
                }
                out.extend_from_slice(&data[pos..pos + sub_len]);
                pos += sub_len;
                if pos >= data.len() {
                    break;
                }
                sub_len = data[pos] as usize;
                out.push(data[pos]);
                pos += 1;
            }
            continue;
        }
        pos += 2;
        if pos >= data.len() {
            break;
        }
        let mut sub_len = data[pos] as usize;
        pos += 1;
        while sub_len > 0 {
            if pos + sub_len > data.len() {
                return Err(GvError::Message("truncated GIF extension data".into()));
            }
            pos += sub_len;
            if pos >= data.len() {
                break;
            }
            sub_len = data[pos] as usize;
            pos += 1;
        }
    }

    Ok(out)
}

/// Re-encode TIFF from decoded pixels so IFD tags (GPS, etc.) are not carried over.
fn strip_tiff_metadata(data: &[u8]) -> Result<Vec<u8>, GvError> {
    let img = image::load_from_memory(data).map_err(GvError::from)?;
    let rgba = img.to_rgba8();
    let mut buf = Cursor::new(Vec::new());
    TiffEncoder::new(&mut buf).write_image(
        rgba.as_raw(),
        rgba.width(),
        rgba.height(),
        ExtendedColorType::Rgba8,
    )?;
    Ok(buf.into_inner())
}

/// Remove top-level metadata boxes from ISO-BMFF containers (HEIC, AVIF).
fn strip_isobmff_metadata(data: &[u8]) -> Result<Vec<u8>, GvError> {
    if data.len() < 12 {
        return Ok(data.to_vec());
    }
    let mut out = Vec::with_capacity(data.len());
    let mut pos = 0usize;
    while pos + 8 <= data.len() {
        let Some((box_size, box_type, header_len)) = read_bmff_box_header(data, pos) else {
            break;
        };
        if box_size < header_len || pos.saturating_add(box_size) > data.len() {
            break;
        }
        if box_type == *b"meta" || box_type == *b"uuid" || box_type == *b"udta" {
            pos += box_size;
            continue;
        }
        out.extend_from_slice(&data[pos..pos + box_size]);
        pos += box_size;
    }
    if out.is_empty() {
        return Ok(data.to_vec());
    }
    Ok(out)
}

fn read_bmff_box_header(data: &[u8], pos: usize) -> Option<(usize, [u8; 4], usize)> {
    if pos + 8 > data.len() {
        return None;
    }
    let mut size = u32::from_be_bytes(data[pos..pos + 4].try_into().ok()?) as usize;
    let box_type: [u8; 4] = data[pos + 4..pos + 8].try_into().ok()?;
    let mut header_len = 8usize;
    if size == 0 {
        size = data.len().saturating_sub(pos);
    } else if size == 1 {
        if pos + 16 > data.len() {
            return None;
        }
        size = u64::from_be_bytes(data[pos + 8..pos + 16].try_into().ok()?) as usize;
        header_len = 16;
    }
    if size < header_len {
        return None;
    }
    Some((size, box_type, header_len))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(prefix: &str) -> std::path::PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("{prefix}-{stamp}"));
        std::fs::create_dir_all(&dir).expect("mkdir");
        dir
    }

    #[test]
    fn strip_jpeg_on_encoder_output_succeeds() {
        let dir = temp_dir("gv-strip-jpeg");
        let path = dir.join("test.jpg");
        let img: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> =
            image::ImageBuffer::from_fn(4, 4, |_, _| image::Rgb([10, 20, 30]));
        img.save(&path).expect("save jpeg");

        let bytes = fs::read(&path).expect("read");
        let stripped = strip_jpeg_metadata(&bytes, false).expect("strip");
        assert!(stripped.starts_with(&[0xFF, 0xD8]));
        assert!(!stripped.windows(2).any(|w| w == [0xFF, 0xE1]));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn strip_png_on_encoder_output_succeeds() {
        let dir = temp_dir("gv-strip-png");
        let path = dir.join("test.png");
        let img: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> =
            image::ImageBuffer::from_fn(2, 2, |_, _| image::Rgb([1, 2, 3]));
        img.save(&path).expect("save png");

        let bytes = fs::read(&path).expect("read");
        let stripped = strip_png_metadata(&bytes, true).expect("strip");
        assert!(stripped.starts_with(&[137, 80, 78, 71, 13, 10, 26, 10]));
        image::load_from_memory(&stripped).expect("valid png after strip");

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn strip_isobmff_removes_meta_box() {
        fn push_box(file: &mut Vec<u8>, fourcc: &[u8; 4], payload: &[u8]) {
            let size = (8 + payload.len()) as u32;
            file.extend_from_slice(&size.to_be_bytes());
            file.extend_from_slice(fourcc);
            file.extend_from_slice(payload);
        }

        let mut file = Vec::new();
        push_box(&mut file, b"ftyp", b"avif");
        push_box(&mut file, b"meta", &[0u8; 8]);
        push_box(&mut file, b"mdat", b"pixels");

        let stripped = strip_isobmff_metadata(&file).expect("strip");
        assert!(stripped.len() < file.len());
        assert!(stripped.windows(4).any(|w| w == b"ftyp"));
        assert!(stripped.windows(4).any(|w| w == b"mdat"));
        assert!(!stripped.windows(4).any(|w| w == b"meta"));
    }

    #[test]
    fn strip_webp_removes_exif_chunk() {
        let mut webp = Vec::new();
        webp.extend_from_slice(b"RIFF");
        webp.extend_from_slice(&0u32.to_le_bytes()); // patched later
        webp.extend_from_slice(b"WEBP");
        // EXIF chunk
        webp.extend_from_slice(b"EXIF");
        webp.extend_from_slice(&4u32.to_le_bytes());
        webp.extend_from_slice(b"meta");
        // VP8 placeholder chunk
        webp.extend_from_slice(b"VP8 ");
        webp.extend_from_slice(&2u32.to_le_bytes());
        webp.extend_from_slice(&[0u8, 1]);

        let payload_len = webp.len() - 8;
        webp[4..8].copy_from_slice(&(payload_len as u32).to_le_bytes());

        let stripped = strip_webp_metadata(&webp).expect("strip");
        let body = String::from_utf8_lossy(&stripped);
        assert!(!body.contains("EXIF"));
        assert!(body.contains("VP8 "));
    }

    #[test]
    fn converted_webp_has_no_readable_exif() {
        use image::codecs::png::PngEncoder;
        use image::{ExtendedColorType, ImageEncoder};

        let dir = temp_dir("gv-privacy");
        let source = dir.join("in.png");
        let output = dir.join("out.webp");

        let file = std::fs::File::create(&source).expect("create");
        let mut writer = std::io::BufWriter::new(file);
        PngEncoder::new(&mut writer)
            .write_image(&[255, 0, 0, 255], 1, 1, ExtendedColorType::Rgba8)
            .expect("png");
        writer.flush().expect("flush");

        let img = image::open(&source).expect("open");
        let rgba = img.to_rgba8();
        let encoder = webp::Encoder::from_rgba(rgba.as_raw(), 1, 1);
        std::fs::write(&output, &*encoder.encode(80.0)).expect("webp");

        strip_privacy_metadata(&output, false).expect("strip");
        assert!(!file_has_privacy_exif(&output));

        let _ = std::fs::remove_dir_all(dir);
    }
}
