use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use exif::{In, Tag};
use image::imageops::{self, FilterType};
use image::{DynamicImage, GenericImageView, Rgba, RgbaImage};


pub fn apply_exif_orientation(img: DynamicImage, source_path: &Path) -> DynamicImage {
    let orientation = read_exif_orientation(source_path).unwrap_or(1);
    orient_image(img, orientation)
}

fn read_exif_orientation(path: &Path) -> Option<u32> {
    let file = File::open(path).ok()?;
    let mut bufreader = BufReader::new(file);
    let exif = exif::Reader::new().read_from_container(&mut bufreader).ok()?;
    let field = exif.get_field(Tag::Orientation, In::PRIMARY)?;
    field.value.get_uint(0).filter(|v| (1..=8).contains(v))
}

fn orient_image(mut img: DynamicImage, orientation: u32) -> DynamicImage {
    match orientation {
        2 => img = DynamicImage::ImageRgba8(imageops::flip_horizontal(&img.to_rgba8())),
        3 => img = img.rotate180(),
        4 => {
            let flipped = imageops::flip_horizontal(&img.to_rgba8());
            img = DynamicImage::ImageRgba8(imageops::flip_vertical(&flipped));
        }
        5 => {
            img = img.rotate90();
            img = DynamicImage::ImageRgba8(imageops::flip_horizontal(&img.to_rgba8()));
        }
        6 => img = img.rotate90(),
        7 => {
            img = img.rotate270();
            img = DynamicImage::ImageRgba8(imageops::flip_horizontal(&img.to_rgba8()));
        }
        8 => img = img.rotate270(),
        _ => {}
    }
    img
}

pub fn parse_flatten_color(hex: &str) -> [u8; 3] {
    let s = hex.trim().trim_start_matches('#');
    if s.len() == 6 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&s[0..2], 16),
            u8::from_str_radix(&s[2..4], 16),
            u8::from_str_radix(&s[4..6], 16),
        ) {
            return [r, g, b];
        }
    }
    [255, 255, 255]
}

pub fn flatten_alpha(img: &DynamicImage, background: [u8; 3]) -> DynamicImage {
    if !img.color().has_alpha() {
        return img.clone();
    }
    let (w, h) = img.dimensions();
    let mut out = RgbaImage::new(w, h);
    let [br, bg, bb] = background;
    for (x, y, pixel) in img.to_rgba8().enumerate_pixels() {
        let alpha = pixel[3] as f32 / 255.0;
        let r = (pixel[0] as f32 * alpha + br as f32 * (1.0 - alpha)) as u8;
        let g = (pixel[1] as f32 * alpha + bg as f32 * (1.0 - alpha)) as u8;
        let b = (pixel[2] as f32 * alpha + bb as f32 * (1.0 - alpha)) as u8;
        out.put_pixel(x, y, Rgba([r, g, b, 255]));
    }
    DynamicImage::ImageRgba8(out)
}

pub fn read_icc_profile(path: &Path) -> Option<Vec<u8>> {
    use image::ImageDecoder;
    use image::ImageReader;
    let reader = ImageReader::open(path).ok()?;
    let mut decoder = reader.into_decoder().ok()?;
    decoder.icc_profile().ok().flatten()
}

pub fn resize_max(img: DynamicImage, max_w: Option<u32>, max_h: Option<u32>) -> DynamicImage {
    let (w, h) = img.dimensions();
    let max_w = max_w.unwrap_or(w);
    let max_h = max_h.unwrap_or(h);
    if w <= max_w && h <= max_h {
        return img;
    }
    let scale_w = max_w as f64 / w as f64;
    let scale_h = max_h as f64 / h as f64;
    let scale = scale_w.min(scale_h);
    let new_w = ((w as f64 * scale).round() as u32).max(1);
    let new_h = ((h as f64 * scale).round() as u32).max(1);
    resize_with_shrink(img, new_w, new_h)
}

/// Pyramid shrink (Triangle halving) then a final Lanczos pass — libvips-style downscale
/// without a second native stack; faster and lower peak memory on large downscales.
fn resize_with_shrink(img: DynamicImage, new_w: u32, new_h: u32) -> DynamicImage {
    let (mut w, mut h) = img.dimensions();
    let mut current = img;

    while w / 2 >= new_w && h / 2 >= new_h && w > 2 && h > 2 {
        let mid_w = (w / 2).max(1);
        let mid_h = (h / 2).max(1);
        current = current.resize(mid_w, mid_h, FilterType::Triangle);
        w = mid_w;
        h = mid_h;
    }

    if w == new_w && h == new_h {
        return current;
    }
    current.resize(new_w, new_h, FilterType::Lanczos3)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgb};

    #[test]
    fn shrink_resize_reaches_target_dimensions() {
        let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_fn(800, 600, |_, _| Rgb([128, 64, 32]));
        let out = resize_max(DynamicImage::ImageRgb8(img), Some(200), Some(150));
        assert_eq!(out.dimensions(), (200, 150));
    }

    #[test]
    fn shrink_resize_noop_when_within_bounds() {
        let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_fn(100, 80, |_, _| Rgb([1, 2, 3]));
        let out = resize_max(DynamicImage::ImageRgb8(img), Some(200), Some(200));
        assert_eq!(out.dimensions(), (100, 80));
    }
}
