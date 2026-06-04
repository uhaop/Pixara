use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;

/// How conversion runs today. GPU image encode/decode is not wired yet; detection is for UI/future use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ConvertBackend {
    /// Parallel workers on the CPU (default).
    CpuParallel,
    /// Reserved: hardware-assisted path when implemented.
    #[allow(dead_code)]
    GpuAssisted,
}

/// Active or probed HEIC decode path. Phase 6 will switch this when hardware decode ships.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum HeicDecodeBackend {
    /// libheif / libde265 on CPU (current default).
    Cpu,
    /// Windows Media Foundation HEVC (Phase 6).
    #[allow(dead_code)]
    MediaFoundation,
    /// Bundled FFmpeg with hardware acceleration (Phase 6).
    #[allow(dead_code)]
    FfmpegHw,
    #[allow(dead_code)]
    Unknown,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemCapabilities {
    pub logical_cpus: usize,
    pub convert_workers: usize,
    /// HEIC/HEIF read path is compiled in and available (libheif or future Windows codecs).
    pub heic_read_available: bool,
    /// HEIC/HEIF export is compiled in and available.
    pub heic_write_available: bool,
    /// True when a non-software DXGI adapter was found.
    pub gpu_detected: bool,
    /// Primary GPU adapter description from DXGI, when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpu_adapter_name: Option<String>,
    pub convert_backend: ConvertBackend,
    pub heic_decode_backend: HeicDecodeBackend,
    /// Hardware acceleration methods reported by bundled `ffmpeg.exe -hwaccels`, if present.
    pub ffmpeg_hwaccels: Vec<String>,
    pub backend_note: String,
    /// HEIC / input decode stage — does not affect PNG/JPEG encode.
    pub decode_note: String,
    /// Output encode stage — always CPU today.
    pub encode_note: String,
}

pub fn logical_cpu_count() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}

/// Worker count for batch conversion: leave headroom for the UI and OS.
pub fn convert_worker_count() -> usize {
    let cpus = logical_cpu_count();
    match cpus {
        0 | 1 => 1,
        2 => 1,
        3 | 4 => 2,
        n => (n - 2).clamp(2, 8),
    }
}

const SLOW_DRIVE_WORKER_CAP: usize = 2;

/// Applies optional slow-drive cap from user settings.
pub fn effective_convert_worker_count(slow_drive_mode: bool) -> usize {
    let auto = convert_worker_count();
    if slow_drive_mode {
        auto.min(SLOW_DRIVE_WORKER_CAP)
    } else {
        auto
    }
}

fn app_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
}

pub(crate) fn bundled_ffmpeg_path() -> Option<PathBuf> {
    app_dir()
        .map(|dir| dir.join("ffmpeg.exe"))
        .filter(|p| p.is_file())
}

/// Parses `ffmpeg -hwaccels` stdout; ignores the header line.
pub(crate) fn probe_ffmpeg_hwaccels(ffmpeg: &Path) -> Vec<String> {
    let output = match Command::new(ffmpeg)
        .args(["-hide_banner", "-hwaccels"])
        .output()
    {
        Ok(o) if o.status.success() => o,
        _ => return Vec::new(),
    };
    let text = String::from_utf8_lossy(&output.stdout);
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.eq_ignore_ascii_case("hardware acceleration methods:"))
        .map(str::to_string)
        .collect()
}

#[cfg(windows)]
fn primary_gpu_adapter_name() -> Option<String> {
    use std::ffi::OsString;
    use std::os::windows::prelude::OsStringExt;

    use windows::Win32::Graphics::Dxgi::{
        CreateDXGIFactory1, DXGI_ADAPTER_DESC1, DXGI_ADAPTER_FLAG_SOFTWARE, IDXGIFactory1,
    };

    unsafe {
        let factory: IDXGIFactory1 = CreateDXGIFactory1().ok()?;
        let mut index = 0u32;
        loop {
            let adapter = match factory.EnumAdapters1(index) {
                Ok(a) => a,
                Err(_) => break,
            };
            index += 1;

            let desc: DXGI_ADAPTER_DESC1 = adapter.GetDesc1().ok()?;
            if desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32 != 0 {
                continue;
            }

            let end = desc
                .Description
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(desc.Description.len());
            let name = OsString::from_wide(&desc.Description[..end]);
            let trimmed = name.to_string_lossy().trim().to_string();
            if trimmed.is_empty() {
                continue;
            }
            return Some(trimmed);
        }
        None
    }
}

#[cfg(not(windows))]
fn primary_gpu_adapter_name() -> Option<String> {
    None
}

fn heic_relevant_hwaccels(hwaccels: &[String]) -> Vec<&str> {
    const RELEVANT: &[&str] = &["d3d11va", "dxva2", "qsv", "nvdec", "cuda", "amf", "videotoolbox"];
    hwaccels
        .iter()
        .filter_map(|h| {
            let lower = h.to_ascii_lowercase();
            RELEVANT.iter().find(|&&r| r == lower).copied()
        })
        .collect()
}

const HWACCEL_PRIORITY: &[&str] = &["d3d11va", "dxva2", "qsv", "nvdec", "cuda", "amf"];

/// Best hardware acceleration method for HEIC/HEVC decode, when bundled FFmpeg reports any.
pub(crate) fn preferred_heic_hwaccel(hwaccels: &[String]) -> Option<String> {
    let relevant = heic_relevant_hwaccels(hwaccels);
    HWACCEL_PRIORITY
        .iter()
        .copied()
        .find(|p| relevant.contains(p))
        .map(str::to_string)
}

fn build_stage_notes(
    convert_workers: usize,
    logical_cpus: usize,
    gpu_adapter_name: &Option<String>,
    ffmpeg_hwaccels: &[String],
) -> (String, String, String) {
    let heic_backend = active_heic_decode_backend(ffmpeg_hwaccels);
    let relevant = heic_relevant_hwaccels(ffmpeg_hwaccels);

    let decode_note = if !heic_stack_available() {
        "HEIC is not enabled in this build. Install Microsoft HEIF/HEVC extensions (see setup-windows-heic.ps1) for future support, or use PNG/JPEG/WebP.".to_string()
    } else {
        match heic_backend {
        HeicDecodeBackend::Cpu => {
            let mut note = "HEIC decode: libheif on CPU.".to_string();
            if let Some(gpu) = gpu_adapter_name {
                note.push_str(&format!(" GPU detected ({gpu})."));
            }
            if !relevant.is_empty() {
                note.push_str(&format!(
                    " Bundled FFmpeg reports {} — enable by placing ffmpeg.exe beside the app.",
                    relevant.join(", ")
                ));
            }
            note
        }
        HeicDecodeBackend::FfmpegHw => {
            format!(
                "HEIC decode: FFmpeg hardware ({}) with CPU fallback.",
                relevant.join(", ")
            )
        }
        HeicDecodeBackend::MediaFoundation => {
            "HEIC decode: Windows Media Foundation with CPU fallback.".to_string()
        }
        HeicDecodeBackend::Unknown => "HEIC decode backend unknown.".to_string(),
    }
    };

    let encode_note =
        "PNG, JPEG, and WebP encode run on the CPU only — a detected GPU does not speed up output encoding."
            .to_string();

    let backend_note = format!(
        "Batch convert uses {convert_workers} CPU worker(s) on {logical_cpus} logical processors. {decode_note} {encode_note}"
    );

    (backend_note, decode_note, encode_note)
}

fn heic_stack_available() -> bool {
    cfg!(feature = "heic")
}

/// Which HEIC decode backend is active when bundled FFmpeg + hwaccels are present (Windows).
fn active_heic_decode_backend(ffmpeg_hwaccels: &[String]) -> HeicDecodeBackend {
    if !heic_stack_available() {
        return HeicDecodeBackend::Unknown;
    }
    #[cfg(windows)]
    {
        if bundled_ffmpeg_path().is_some() && preferred_heic_hwaccel(ffmpeg_hwaccels).is_some() {
            return HeicDecodeBackend::FfmpegHw;
        }
    }
    HeicDecodeBackend::Cpu
}

pub fn system_capabilities() -> SystemCapabilities {
    let logical_cpus = logical_cpu_count();
    let convert_workers = convert_worker_count();
    let gpu_adapter_name = primary_gpu_adapter_name();
    let gpu_detected = gpu_adapter_name.is_some();
    let ffmpeg_hwaccels = bundled_ffmpeg_path()
        .as_deref()
        .map(probe_ffmpeg_hwaccels)
        .unwrap_or_default();
    let heic_decode_backend = active_heic_decode_backend(&ffmpeg_hwaccels);

    let (backend_note, decode_note, encode_note) =
        build_stage_notes(convert_workers, logical_cpus, &gpu_adapter_name, &ffmpeg_hwaccels);

    let heic_available = heic_stack_available();
    SystemCapabilities {
        logical_cpus,
        convert_workers,
        heic_read_available: heic_available,
        heic_write_available: heic_available,
        gpu_detected,
        gpu_adapter_name,
        convert_backend: ConvertBackend::CpuParallel,
        heic_decode_backend,
        ffmpeg_hwaccels,
        backend_note,
        decode_note,
        encode_note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worker_count_is_sane() {
        let n = convert_worker_count();
        assert!(n >= 1);
        assert!(n <= 8);
    }

    #[test]
    fn slow_drive_caps_workers() {
        let auto = convert_worker_count();
        let capped = effective_convert_worker_count(true);
        assert!(capped <= SLOW_DRIVE_WORKER_CAP);
        assert!(capped <= auto);
        assert_eq!(
            effective_convert_worker_count(false),
            convert_worker_count()
        );
    }

    #[test]
    fn capabilities_use_cpu_decode_and_encode() {
        let caps = system_capabilities();
        if cfg!(feature = "heic") {
            assert!(caps.heic_read_available);
            assert_eq!(caps.heic_decode_backend, HeicDecodeBackend::Cpu);
        } else {
            assert!(!caps.heic_read_available);
            assert!(!caps.heic_write_available);
            assert_eq!(caps.heic_decode_backend, HeicDecodeBackend::Unknown);
        }
        assert_eq!(caps.convert_backend, ConvertBackend::CpuParallel);
        assert!(caps.encode_note.contains("CPU"));
        assert!(!caps.encode_note.to_lowercase().contains("gpu speeds up"));
    }

    #[test]
    fn heic_hwaccel_filter_is_case_insensitive() {
        let accels = vec!["D3D11VA".into(), "opencl".into(), "qsv".into()];
        let relevant = heic_relevant_hwaccels(&accels);
        assert_eq!(relevant, vec!["d3d11va", "qsv"]);
    }

    #[test]
    fn preferred_hwaccel_respects_priority() {
        let accels = vec!["qsv".into(), "d3d11va".into()];
        assert_eq!(preferred_heic_hwaccel(&accels).as_deref(), Some("d3d11va"));
    }

    #[test]
    fn active_backend_is_ffmpeg_when_bundled_and_hwaccels_present() {
        if !cfg!(feature = "heic") {
            assert_eq!(
                active_heic_decode_backend(&["d3d11va".into()]),
                HeicDecodeBackend::Unknown
            );
            return;
        }
        let backend = active_heic_decode_backend(&["d3d11va".into()]);
        #[cfg(windows)]
        if bundled_ffmpeg_path().is_some() {
            assert_eq!(backend, HeicDecodeBackend::FfmpegHw);
            return;
        }
        assert_eq!(backend, HeicDecodeBackend::Cpu);
    }

    #[test]
    fn stage_notes_never_imply_gpu_png_encode() {
        let (backend, decode, encode) = build_stage_notes(
            4,
            8,
            &Some("NVIDIA GeForce RTX".into()),
            &[],
        );
        if cfg!(feature = "heic") {
            assert!(decode.contains("libheif"));
        } else {
            assert!(decode.contains("not enabled"));
        }
        assert!(encode.contains("CPU only"));
        assert!(backend.contains("CPU worker"));
    }
}
