//! HEIC/HEIF decode entry point. Full libheif path is enabled with Cargo feature `heic`.

#[cfg(not(feature = "heic"))]
use std::path::Path;
#[cfg(not(feature = "heic"))]
use image::DynamicImage;
#[cfg(not(feature = "heic"))]
use crate::types::GvError;

#[cfg_attr(feature = "heic", allow(dead_code))]
pub const HEIC_UNAVAILABLE: &str = "HEIC is not available in this build. Install Microsoft HEIF/HEVC extensions (see setup-windows-heic.ps1 in the repository) or use a release that includes HEIC support.";

#[cfg(feature = "heic")]
#[path = "heic_decode_libheif.rs"]
mod heic_decode_libheif;

#[cfg(feature = "heic")]
pub use heic_decode_libheif::load_heic;

#[cfg(not(feature = "heic"))]
pub fn load_heic(_path: &Path) -> Result<DynamicImage, GvError> {
    Err(GvError::Message(HEIC_UNAVAILABLE.into()))
}

#[cfg_attr(not(feature = "heic"), allow(dead_code))]
pub(crate) fn parse_ffmpeg_video_size(stderr: &str) -> Option<(u32, u32)> {
    for line in stderr.lines() {
        if !line.contains("Video:") {
            continue;
        }
        for token in line.split([',', ' ']) {
            if let Some((w, h)) = token.split_once('x') {
                if let (Ok(w), Ok(h)) = (w.parse::<u32>(), h.parse::<u32>()) {
                    if w > 0 && h > 0 && w <= 65_535 && h <= 65_535 {
                        return Some((w, h));
                    }
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ffmpeg_stderr_dimensions() {
        let stderr = concat!(
            "Input #0, mov,mp4,m4a,3gp,3g2,mj2, from 'photo.heic':\n",
            "  Stream #0:0[0x1](und): Video: hevc (Main), yuv420p, 4032x3024, 1 fps\n"
        );
        assert_eq!(parse_ffmpeg_video_size(stderr), Some((4032, 3024)));
    }

    #[test]
    fn parse_ffmpeg_stderr_no_video_returns_none() {
        assert_eq!(parse_ffmpeg_video_size("no video here\n"), None);
    }
}
