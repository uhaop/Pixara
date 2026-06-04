use crate::types::Preset;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PngCompression {
    Default,
    Best,
}

#[derive(Debug, Clone, Copy)]
pub struct EncodeSettings {
    pub jpeg_quality: u8,
    pub webp_quality: f32,
    pub avif_quality: u8,
    pub avif_speed: u8,
    pub heic_quality: f32,
    pub png_compression: PngCompression,
}

impl EncodeSettings {
    pub fn from_preset(preset: Preset) -> Self {
        match preset {
            Preset::Web => Self {
                jpeg_quality: 85,
                webp_quality: 82.0,
                avif_quality: 55,
                avif_speed: 6,
                heic_quality: 50.0,
                png_compression: PngCompression::Default,
            },
            Preset::High => Self {
                jpeg_quality: 95,
                webp_quality: 92.0,
                avif_quality: 75,
                avif_speed: 4,
                heic_quality: 75.0,
                png_compression: PngCompression::Best,
            },
            Preset::Smallest => Self {
                jpeg_quality: 72,
                webp_quality: 65.0,
                avif_quality: 40,
                avif_speed: 8,
                heic_quality: 40.0,
                png_compression: PngCompression::Best,
            },
        }
    }
}
