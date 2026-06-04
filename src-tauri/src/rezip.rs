use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::types::GvError;

pub use crate::naming::rezip_entry_name;

/// Create `{zip_stem}_converted.zip` beside the source archive containing converted files.
pub fn create_converted_zip(
    zip_source: &Path,
    output_files: &[(PathBuf, String)],
) -> Result<PathBuf, GvError> {
    if output_files.is_empty() {
        return Err(GvError::InvalidSettings(
            "no converted files to package into zip".into(),
        ));
    }

    let parent = zip_source
        .parent()
        .ok_or_else(|| GvError::InvalidSettings("zip source has no parent directory".into()))?;
    let stem = zip_source
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("archive");
    let out_zip = unique_zip_path(parent.join(format!("{stem}_converted.zip")));

    let file = File::create(&out_zip)?;
    let mut writer = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let mut entries_added = 0usize;
    for (path, entry_name) in output_files {
        if !path.is_file() {
            continue;
        }
        let normalized = entry_name.replace('\\', "/");
        writer
            .start_file(&normalized, options)
            .map_err(|e| GvError::Zip(e.to_string()))?;
        let mut input = File::open(path)?;
        let mut buffer = Vec::new();
        input.read_to_end(&mut buffer)?;
        writer
            .write_all(&buffer)
            .map_err(|e| GvError::Zip(e.to_string()))?;
        entries_added += 1;
    }

    if entries_added == 0 {
        return Err(GvError::Zip(
            "no converted files were available to package into zip".into(),
        ));
    }

    writer
        .finish()
        .map_err(|e| GvError::Zip(e.to_string()))?;
    Ok(out_zip)
}

fn unique_zip_path(path: PathBuf) -> PathBuf {
    if !path.exists() {
        return path;
    }
    let parent = path.parent().map(PathBuf::from).unwrap_or_default();
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("archive");
    for i in 1..10_000 {
        let candidate = parent.join(format!("{stem}_{i}.zip"));
        if !candidate.exists() {
            return candidate;
        }
    }
    path
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::GvError;
    use std::fs;
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
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn creates_converted_zip_next_to_source() {
        let dir = temp_dir("gv-pixara-rezip");
        let source_zip = dir.join("photos.zip");
        {
            let file = File::create(&source_zip).expect("create zip");
            let mut zip = ZipWriter::new(file);
            let options = SimpleFileOptions::default();
            zip.start_file("a.txt", options).expect("start");
            zip.write_all(b"old").expect("write");
            zip.finish().expect("finish");
        }

        let converted = dir.join("nested").join("photo.webp");
        std::fs::create_dir_all(converted.parent().unwrap()).expect("mkdir");
        std::fs::write(&converted, b"webp-bytes").expect("write webp");

        let out = create_converted_zip(
            &source_zip,
            &[(
                converted.clone(),
                "nested/photo.webp".to_string(),
            )],
        )
        .expect("rezip");

        assert_eq!(out.file_name().and_then(|n| n.to_str()), Some("photos_converted.zip"));
        assert!(out.exists());

        let read_back = File::open(&out).expect("open out zip");
        let mut archive = zip::ZipArchive::new(read_back).expect("read archive");
        assert_eq!(archive.len(), 1);
        let entry = archive.by_index(0).expect("entry");
        assert_eq!(entry.name(), "nested/photo.webp");

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn rejects_empty_rezip_when_no_files_exist() {
        let dir = temp_dir("gv-pixara-rezip-empty");
        let source_zip = dir.join("photos.zip");
        fs::write(&source_zip, b"zip-placeholder").expect("write zip");

        let missing = dir.join("missing.webp");
        let err = create_converted_zip(
            &source_zip,
            &[(missing, "nested/photo.webp".to_string())],
        )
        .expect_err("empty rezip should fail");
        assert!(matches!(err, GvError::Zip(_)));

        let _ = fs::remove_dir_all(dir);
    }
}
