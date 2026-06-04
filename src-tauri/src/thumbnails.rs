use std::collections::HashMap;
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::sync::{Condvar, Mutex, OnceLock};

use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, ExtendedColorType, ImageEncoder};

use crate::convert_guard;
use crate::engine::load_image;
use crate::ingest::temp_root;
use crate::metadata::apply_exif_orientation;
use crate::types::GvError;

const THUMB_MAX: u32 = 128;
const MAX_CONCURRENT_DECODES: usize = 4;
const STALE_THUMB_MAX_AGE: std::time::Duration = std::time::Duration::from_secs(24 * 60 * 60);

pub fn thumbnail_path(item_id: &str) -> PathBuf {
    temp_root().join("thumbs").join(format!("{item_id}.jpg"))
}

fn thumb_locks() -> &'static Mutex<HashMap<String, ()>> {
    static LOCKS: OnceLock<Mutex<HashMap<String, ()>>> = OnceLock::new();
    LOCKS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn decode_slots() -> &'static DecodeSlots {
    static SLOTS: OnceLock<DecodeSlots> = OnceLock::new();
    SLOTS.get_or_init(|| DecodeSlots::new(MAX_CONCURRENT_DECODES))
}

struct DecodeSlots {
    available: Mutex<usize>,
    cvar: Condvar,
}

impl DecodeSlots {
    fn new(max: usize) -> Self {
        Self {
            available: Mutex::new(max),
            cvar: Condvar::new(),
        }
    }

    fn acquire(&self) -> Result<DecodePermit<'_>, GvError> {
        let mut slots = self
            .available
            .lock()
            .map_err(|_| GvError::Message("thumbnail decode unavailable".into()))?;
        while *slots == 0 {
            slots = self
                .cvar
                .wait(slots)
                .map_err(|_| GvError::Message("thumbnail decode unavailable".into()))?;
        }
        *slots -= 1;
        Ok(DecodePermit { slots: self })
    }

    fn release(&self) {
        if let Ok(mut slots) = self.available.lock() {
            *slots += 1;
            self.cvar.notify_one();
        }
    }
}

struct DecodePermit<'a> {
    slots: &'a DecodeSlots,
}

impl Drop for DecodePermit<'_> {
    fn drop(&mut self) {
        self.slots.release();
    }
}

/// Decode pixels to a JPEG thumbnail. Image decoding only — no script execution or shell invocation.
pub fn get_or_create_thumbnail(item_id: &str, source_path: &str) -> Result<String, GvError> {
    let out = thumbnail_path(item_id);
    if out.is_file() {
        return Ok(out.to_string_lossy().into_owned());
    }

    if convert_guard::is_convert_in_progress() {
        return Err(GvError::Message("conversion_in_progress".into()));
    }

    let _permit = decode_slots().acquire()?;

    if out.is_file() {
        return Ok(out.to_string_lossy().into_owned());
    }

    if convert_guard::is_convert_in_progress() {
        return Err(GvError::Message("conversion_in_progress".into()));
    }

    let _slot = {
        let mut locks = thumb_locks()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if out.is_file() {
            return Ok(out.to_string_lossy().into_owned());
        }
        locks.insert(item_id.to_string(), ());
        ()
    };

    struct ThumbGuard(String);
    impl Drop for ThumbGuard {
        fn drop(&mut self) {
            if let Ok(mut locks) = thumb_locks().lock() {
                locks.remove(&self.0);
            }
        }
    }
    let _guard = ThumbGuard(item_id.to_string());

    if out.is_file() {
        return Ok(out.to_string_lossy().into_owned());
    }

    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
    }

    let source = Path::new(source_path);
    let mut img = load_image(source)?;
    img = apply_exif_orientation(img, source);
    let thumb = img.thumbnail(THUMB_MAX, THUMB_MAX);
    save_thumb_jpeg(&thumb, &out)?;
    Ok(out.to_string_lossy().into_owned())
}

pub fn cleanup_thumbnails(item_ids: &[String]) {
    for item_id in item_ids {
        let path = thumbnail_path(item_id);
        let _ = fs::remove_file(path);
    }
}

pub fn cleanup_stale_thumbnails() {
    let thumbs_dir = temp_root().join("thumbs");
    let Ok(entries) = fs::read_dir(&thumbs_dir) else {
        return;
    };
    let now = std::time::SystemTime::now();
    for entry in entries.flatten() {
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        let Ok(modified) = meta.modified() else {
            continue;
        };
        let Ok(age) = now.duration_since(modified) else {
            continue;
        };
        if age >= STALE_THUMB_MAX_AGE {
            let _ = fs::remove_file(entry.path());
        }
    }
}

fn save_thumb_jpeg(img: &DynamicImage, path: &Path) -> Result<(), GvError> {
    let rgb = img.to_rgb8();
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    JpegEncoder::new_with_quality(&mut writer, 82).write_image(
        rgb.as_raw(),
        rgb.width(),
        rgb.height(),
        ExtendedColorType::Rgb8,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::convert_guard::ConvertInProgressGuard;

    #[test]
    fn cleanup_thumbnails_removes_cached_files() {
        let item_id = "thumb-cleanup-test";
        let path = thumbnail_path(item_id);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("mkdir thumbs");
        }
        fs::write(&path, b"cached").expect("write thumb");
        assert!(path.is_file());

        cleanup_thumbnails(&[item_id.to_string()]);
        assert!(!path.exists());
    }

    #[test]
    fn defers_decode_while_convert_active() {
        let guard = ConvertInProgressGuard::try_acquire().expect("acquire convert lock");
        let result = get_or_create_thumbnail("convert-block-test", "C:\\missing\\file.png");
        drop(guard);
        assert!(matches!(result, Err(GvError::Message(m)) if m == "conversion_in_progress"));
    }

    #[test]
    fn serves_cached_thumb_during_convert() {
        let item_id = "convert-cached-test";
        let path = thumbnail_path(item_id);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("mkdir thumbs");
        }
        fs::write(&path, b"cached").expect("write thumb");

        let guard = ConvertInProgressGuard::try_acquire().expect("acquire convert lock");
        let result = get_or_create_thumbnail(item_id, "C:\\missing\\file.png");
        drop(guard);

        assert_eq!(result.unwrap(), path.to_string_lossy());
        cleanup_thumbnails(&[item_id.to_string()]);
    }
}
