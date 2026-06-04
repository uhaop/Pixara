use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, SystemTime};

use uuid::Uuid;
use walkdir::WalkDir;
use zip::ZipArchive;

use crate::supported::{format_from_path, is_supported_image, matches_from_filter};
use crate::types::{GvError, ImageFormat, IngestResult, QueueItem};

// ZIP extracts under temp_root() are removed when the UI calls cleanup_temp_batches
// (clear queue), or by the 24h stale sweep on startup — not after convert.

pub const MAX_INGEST_FILES: usize = 5000;
pub const MAX_ZIP_ENTRY_BYTES: u64 = 100 * 1024 * 1024;
pub const MAX_ZIP_TOTAL_BYTES: u64 = 500 * 1024 * 1024;
const STALE_TEMP_MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);

pub fn temp_root() -> PathBuf {
    std::env::temp_dir().join("gv-pixara")
}

pub fn cleanup_temp_batches(batch_ids: impl IntoIterator<Item = impl AsRef<str>>) {
    let temp_root = temp_root();
    for batch_id in batch_ids {
        let batch_dir = temp_root.join(batch_id.as_ref());
        let _ = fs::remove_dir_all(batch_dir);
    }
}

pub fn cleanup_stale_temp_dirs() {
    let temp_root = temp_root();
    let Ok(entries) = fs::read_dir(&temp_root) else {
        return;
    };

    let now = SystemTime::now();
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
        if age >= STALE_TEMP_MAX_AGE {
            let _ = fs::remove_dir_all(entry.path());
        }
    }
}

pub fn ingest_paths(paths: &[String], from_format: ImageFormat) -> Result<IngestResult, GvError> {
    let batch_id = Uuid::new_v4().to_string();
    let mut items: Vec<QueueItem> = Vec::new();
    let mut skipped = 0u32;
    let mut seen: HashSet<PathBuf> = HashSet::new();
    let mut truncated = false;

    for path_str in paths {
        let path = PathBuf::from(path_str);
        if !path.exists() {
            skipped += 1;
            continue;
        }
        if path.is_file() {
            if is_zip_file(&path) {
                collect_from_zip(
                    &path,
                    &batch_id,
                    from_format,
                    &mut items,
                    &mut skipped,
                    &mut seen,
                    &mut truncated,
                )?;
            } else {
                push_file(
                    &path,
                    &path.file_name().unwrap_or_default().to_string_lossy(),
                    &batch_id,
                    from_format,
                    None,
                    &mut items,
                    &mut skipped,
                    &mut seen,
                    &mut truncated,
                );
            }
        } else if path.is_dir() {
            collect_from_dir(
                &path,
                &batch_id,
                from_format,
                &mut items,
                &mut skipped,
                &mut seen,
                &mut truncated,
            );
        } else {
            skipped += 1;
        }
    }

    Ok(IngestResult {
        batch_id,
        items,
        skipped,
        truncated,
    })
}

fn is_zip_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
}

fn safe_zip_entry_path(extract_root: &Path, entry_name: &str) -> Result<PathBuf, GvError> {
    let name = entry_name.replace('\\', "/");
    if name.is_empty() || name.ends_with('/') {
        return Err(GvError::Zip("empty zip entry path".into()));
    }
    if name.contains('\0') {
        return Err(GvError::Zip("invalid zip entry path".into()));
    }

    let rel = Path::new(&name);
    if rel.is_absolute() {
        return Err(GvError::Zip("absolute zip entry path".into()));
    }

    for component in rel.components() {
        match component {
            Component::Normal(_) | Component::CurDir => {}
            _ => return Err(GvError::Zip("unsafe zip entry path".into())),
        }
    }

    let out_path = extract_root.join(rel);
    if !path_starts_with(&out_path, extract_root) {
        return Err(GvError::Zip("zip entry escapes extract directory".into()));
    }

    Ok(out_path)
}

fn path_starts_with(candidate: &Path, base: &Path) -> bool {
    match candidate.strip_prefix(base) {
        Ok(relative) => relative
            .components()
            .all(|component| matches!(component, Component::Normal(_) | Component::CurDir)),
        Err(_) => {
            if let (Ok(base_canon), Ok(candidate_canon)) =
                (base.canonicalize(), candidate.canonicalize())
            {
                candidate_canon.starts_with(&base_canon)
            } else {
                false
            }
        }
    }
}

fn push_file(
    path: &Path,
    relative_path: &str,
    batch_id: &str,
    from_format: ImageFormat,
    zip_source_path: Option<&Path>,
    items: &mut Vec<QueueItem>,
    skipped: &mut u32,
    seen: &mut HashSet<PathBuf>,
    truncated: &mut bool,
) {
    if *truncated || items.len() >= MAX_INGEST_FILES {
        *truncated = true;
        return;
    }
    let canonical = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    if !seen.insert(canonical) {
        return;
    }
    if !is_supported_image(path) || !matches_from_filter(path, from_format) {
        *skipped += 1;
        return;
    }
    let size_bytes = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    let source_format = format_from_path(path).unwrap_or(ImageFormat::Png);
    items.push(QueueItem {
        id: Uuid::new_v4().to_string(),
        batch_id: batch_id.to_string(),
        source_path: path.to_string_lossy().to_string(),
        relative_path: relative_path.replace('\\', "/"),
        source_format,
        size_bytes,
        zip_source_path: zip_source_path.map(|p| p.to_string_lossy().to_string()),
        output_base_name: None,
    });
}

fn collect_from_dir(
    root: &Path,
    batch_id: &str,
    from_format: ImageFormat,
    items: &mut Vec<QueueItem>,
    skipped: &mut u32,
    seen: &mut HashSet<PathBuf>,
    truncated: &mut bool,
) {
    collect_from_dir_with_zip(
        root,
        batch_id,
        from_format,
        None,
        items,
        skipped,
        seen,
        truncated,
    );
}

fn copy_limited<R: Read>(reader: &mut R, writer: &mut File, limit: u64) -> Result<u64, io::Error> {
    let mut buffer = [0u8; 64 * 1024];
    let mut total = 0u64;

    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        total += read as u64;
        if total > limit {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "zip entry exceeds size limit",
            ));
        }
        writer.write_all(&buffer[..read])?;
    }

    Ok(total)
}

fn collect_from_zip(
    zip_path: &Path,
    batch_id: &str,
    from_format: ImageFormat,
    items: &mut Vec<QueueItem>,
    skipped: &mut u32,
    seen: &mut HashSet<PathBuf>,
    truncated: &mut bool,
) -> Result<(), GvError> {
    let extract_root = temp_root()
        .join(batch_id)
        .join(
            zip_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("archive"),
        );
    fs::create_dir_all(&extract_root)?;

    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file).map_err(|e| GvError::Zip(e.to_string()))?;
    let mut extracted_total = 0u64;

    let extract_result = (|| -> Result<(), GvError> {
        for i in 0..archive.len() {
            let mut entry = archive
                .by_index(i)
                .map_err(|e| GvError::Zip(e.to_string()))?;
            if entry.is_dir() {
                continue;
            }

            let entry_size = entry.size();
            if entry_size > MAX_ZIP_ENTRY_BYTES {
                return Err(GvError::Zip(format!(
                    "zip entry exceeds {MAX_ZIP_ENTRY_BYTES} byte limit"
                )));
            }
            if extracted_total.saturating_add(entry_size) > MAX_ZIP_TOTAL_BYTES {
                return Err(GvError::Zip(format!(
                    "zip archive exceeds {MAX_ZIP_TOTAL_BYTES} byte extract limit"
                )));
            }

            let out_path = safe_zip_entry_path(&extract_root, entry.name())?;

            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }

            let mut out_file = File::create(&out_path)?;
            let copied = copy_limited(&mut entry, &mut out_file, MAX_ZIP_ENTRY_BYTES)
                .map_err(|e| GvError::Zip(e.to_string()))?;
            extracted_total = extracted_total.saturating_add(copied);
            if extracted_total > MAX_ZIP_TOTAL_BYTES {
                return Err(GvError::Zip(format!(
                    "zip archive exceeds {MAX_ZIP_TOTAL_BYTES} byte extract limit"
                )));
            }
        }
        Ok(())
    })();

    if let Err(err) = extract_result {
        let _ = fs::remove_dir_all(temp_root().join(batch_id));
        return Err(err);
    }

    collect_from_dir_with_zip(
        &extract_root,
        batch_id,
        from_format,
        Some(zip_path),
        items,
        skipped,
        seen,
        truncated,
    );
    Ok(())
}

fn collect_from_dir_with_zip(
    root: &Path,
    batch_id: &str,
    from_format: ImageFormat,
    zip_source_path: Option<&Path>,
    items: &mut Vec<QueueItem>,
    skipped: &mut u32,
    seen: &mut HashSet<PathBuf>,
    truncated: &mut bool,
) {
    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        if *truncated {
            break;
        }
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let rel = path
            .strip_prefix(root)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| path.file_name().unwrap_or_default().to_string_lossy().to_string());
        push_file(
            path,
            &rel,
            batch_id,
            from_format,
            zip_source_path,
            items,
            skipped,
            seen,
            truncated,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgb};
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    fn temp_dir(prefix: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("{prefix}-{stamp}"));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn write_test_png(path: &Path) {
        let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_fn(16, 16, |x, y| {
            if (x + y) % 2 == 0 {
                Rgb([200, 80, 70])
            } else {
                Rgb([40, 160, 210])
            }
        });
        img.save(path).expect("write png");
    }

    fn write_zip(path: &Path, entries: &[(&str, &[u8])]) {
        let file = File::create(path).expect("create zip");
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default();
        for (name, bytes) in entries {
            zip.start_file(*name, options).expect("start zip entry");
            zip.write_all(bytes).expect("write zip entry");
        }
        zip.finish().expect("finish zip");
    }

    #[test]
    fn zip_extract_remains_until_explicit_cleanup() {
        let dir = temp_dir("gv-pixara-zip-retain");
        let source_img = dir.join("input.png");
        write_test_png(&source_img);
        let bytes = fs::read(&source_img).expect("read png");

        let zip_path = dir.join("images.zip");
        write_zip(&zip_path, &[("photo.png", &bytes)]);

        let result = ingest_paths(&[zip_path.to_string_lossy().into_owned()], ImageFormat::Any)
            .expect("ingest zip");
        let batch_dir = temp_root().join(&result.batch_id);
        assert!(batch_dir.exists(), "extract dir should remain for queue previews");

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn ingest_zip_keeps_relative_paths() {
        let dir = temp_dir("gv-pixara-ingest-zip");
        let source_img = dir.join("input.png");
        write_test_png(&source_img);
        let bytes = fs::read(&source_img).expect("read png");

        let zip_path = dir.join("images.zip");
        write_zip(&zip_path, &[("nested/photo.png", &bytes)]);

        let result = ingest_paths(&[zip_path.to_string_lossy().into_owned()], ImageFormat::Any)
            .expect("ingest zip");
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].relative_path, "nested/photo.png");
        assert!(result.items[0].source_path.contains("gv-pixara"));
        assert_eq!(
            result.items[0].zip_source_path.as_deref(),
            zip_path.to_str()
        );

        cleanup_temp_batches([&result.batch_id]);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn ingest_zip_extension_is_case_insensitive() {
        let dir = temp_dir("gv-pixara-ingest-zip-case");
        let source_img = dir.join("input.png");
        write_test_png(&source_img);
        let bytes = fs::read(&source_img).expect("read png");

        let zip_path = dir.join("images.ZIP");
        write_zip(&zip_path, &[("photo.png", &bytes)]);

        let result = ingest_paths(&[zip_path.to_string_lossy().into_owned()], ImageFormat::Any)
            .expect("ingest uppercase zip");
        assert_eq!(result.items.len(), 1);

        cleanup_temp_batches([&result.batch_id]);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn ingest_rejects_zip_slip_paths() {
        let dir = temp_dir("gv-pixara-ingest-slip");
        let zip_path = dir.join("evil.zip");
        write_zip(&zip_path, &[("../../escape.png", b"bad")]);

        let err = ingest_paths(&[zip_path.to_string_lossy().into_owned()], ImageFormat::Any)
            .expect_err("zip slip should be rejected");
        assert!(matches!(err, GvError::Zip(_)));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn ingest_deduplicates_same_file_paths() {
        let dir = temp_dir("gv-pixara-ingest-dedupe");
        let source_img = dir.join("input.png");
        write_test_png(&source_img);

        let source = source_img.to_string_lossy().into_owned();
        let result =
            ingest_paths(&[source.clone(), source], ImageFormat::Any).expect("ingest dedupe");

        assert_eq!(result.items.len(), 1);
        assert_eq!(result.skipped, 0);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn cleanup_temp_batches_removes_extracted_files() {
        let dir = temp_dir("gv-pixara-cleanup");
        let source_img = dir.join("input.png");
        write_test_png(&source_img);
        let bytes = fs::read(&source_img).expect("read png");

        let zip_path = dir.join("images.zip");
        write_zip(&zip_path, &[("photo.png", &bytes)]);

        let result = ingest_paths(&[zip_path.to_string_lossy().into_owned()], ImageFormat::Any)
            .expect("ingest zip");
        let batch_dir = temp_root().join(&result.batch_id);
        assert!(batch_dir.exists());

        cleanup_temp_batches([&result.batch_id]);
        assert!(!batch_dir.exists());

        let _ = fs::remove_dir_all(dir);
    }
}
