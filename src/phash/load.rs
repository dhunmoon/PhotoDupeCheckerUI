//! Fast image decoding at reduced size (used for hashing and thumbnails).

use std::path::Path;

use image::{DynamicImage, GrayImage, ImageBuffer, Luma, Rgb, RgbImage};

use crate::model::extension_lower;

pub struct LoadedImage {
    pub image: DynamicImage,
    /// Original (full-resolution) dimensions of the file.
    pub orig_dims: (u32, u32),
}

/// Loads an image as small as possible while keeping at least `min_side` pixels on its
/// shortest side (when the source is that large). EXIF orientation is applied.
///
/// JPEGs use DCT-domain downscaling (1/2, 1/4, 1/8), which is several times faster than
/// a full decode; other formats fall back to a normal decode.
pub fn load_small(path: &Path, min_side: u32) -> Result<LoadedImage, String> {
    let ext = extension_lower(path);
    if matches!(ext.as_str(), "jpg" | "jpeg" | "jpe" | "jfif")
        && let Ok(img) = load_jpeg_scaled(path, min_side)
    {
        return Ok(img);
    }
    load_generic(path)
}

fn load_jpeg_scaled(path: &Path, min_side: u32) -> Result<LoadedImage, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut dec = jpeg_decoder::Decoder::new(std::io::BufReader::new(file));
    dec.read_info().map_err(|e| e.to_string())?;
    let info = dec.info().ok_or("no jpeg info")?;
    let orig_dims = (info.width as u32, info.height as u32);
    let req = min_side.clamp(1, u16::MAX as u32) as u16;
    dec.scale(req, req).map_err(|e| e.to_string())?;
    let pixels = dec.decode().map_err(|e| e.to_string())?;
    let info = dec.info().ok_or("no jpeg info")?;
    let (w, h) = (info.width as u32, info.height as u32);
    let mut image = match info.pixel_format {
        jpeg_decoder::PixelFormat::RGB24 => {
            DynamicImage::ImageRgb8(RgbImage::from_raw(w, h, pixels).ok_or("bad buffer")?)
        }
        jpeg_decoder::PixelFormat::L8 => {
            DynamicImage::ImageLuma8(GrayImage::from_raw(w, h, pixels).ok_or("bad buffer")?)
        }
        jpeg_decoder::PixelFormat::L16 => {
            let px: Vec<u16> = pixels
                .chunks_exact(2)
                .map(|c| u16::from_be_bytes([c[0], c[1]]))
                .collect();
            DynamicImage::ImageLuma16(
                ImageBuffer::<Luma<u16>, _>::from_raw(w, h, px).ok_or("bad buffer")?,
            )
        }
        jpeg_decoder::PixelFormat::CMYK32 => {
            let rgb: Vec<u8> = pixels.chunks_exact(4).flat_map(cmyk_to_rgb).collect();
            DynamicImage::ImageRgb8(
                ImageBuffer::<Rgb<u8>, _>::from_raw(w, h, rgb).ok_or("bad buffer")?,
            )
        }
    };
    if let Some(o) = dec
        .exif_data()
        .and_then(image::metadata::Orientation::from_exif_chunk)
    {
        image.apply_orientation(o);
    }
    Ok(LoadedImage { image, orig_dims })
}

/// jpeg-decoder returns Adobe-style inverted CMYK, so channel × K approximates RGB.
fn cmyk_to_rgb(c: &[u8]) -> [u8; 3] {
    let k = c[3] as u32;
    [
        (c[0] as u32 * k / 255) as u8,
        (c[1] as u32 * k / 255) as u8,
        (c[2] as u32 * k / 255) as u8,
    ]
}

fn load_generic(path: &Path) -> Result<LoadedImage, String> {
    use image::ImageDecoder;
    let reader = image::ImageReader::open(path)
        .map_err(|e| e.to_string())?
        .with_guessed_format()
        .map_err(|e| e.to_string())?;
    let mut decoder = reader.into_decoder().map_err(|e| e.to_string())?;
    let orientation = decoder.orientation().ok();
    let mut image = DynamicImage::from_decoder(decoder).map_err(|e| e.to_string())?;
    let orig_dims = (image.width(), image.height());
    if let Some(o) = orientation {
        image.apply_orientation(o);
    }
    Ok(LoadedImage { image, orig_dims })
}

/// Reads only the file header to get the pixel dimensions.
pub fn dimensions(path: &Path) -> Option<(u32, u32)> {
    image::ImageReader::open(path)
        .ok()?
        .with_guessed_format()
        .ok()?
        .into_dimensions()
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{TempDir, test_image};

    #[test]
    fn jpeg_is_downscaled_but_reports_original_size() {
        let dir = TempDir::new("load_jpeg");
        let p = dir.path().join("big.jpg");
        test_image(1600, 1200, 1).save(&p).unwrap();
        let l = load_small(&p, 128).unwrap();
        assert_eq!(l.orig_dims, (1600, 1200));
        assert!(l.image.width() < 1600 && l.image.height() >= 128);
        assert_eq!(dimensions(&p), Some((1600, 1200)));
    }

    #[test]
    fn png_and_wrong_extension_still_load() {
        let dir = TempDir::new("load_png");
        let p = dir.path().join("a.png");
        test_image(50, 40, 2).save(&p).unwrap();
        // A PNG renamed to .jpg: JPEG path fails, generic decoder sniffs the real format.
        let renamed = dir.path().join("actually_png.jpg");
        std::fs::copy(&p, &renamed).unwrap();
        for path in [&p, &renamed] {
            let l = load_small(path, 128).unwrap();
            assert_eq!(l.orig_dims, (50, 40));
            assert_eq!((l.image.width(), l.image.height()), (50, 40));
        }
    }

    #[test]
    fn garbage_fails_cleanly() {
        let dir = TempDir::new("load_bad");
        let p = dir.write("x.jpg", b"nope");
        assert!(load_small(&p, 64).is_err());
        assert_eq!(dimensions(&p), None);
    }

    #[test]
    fn cmyk_conversion() {
        assert_eq!(cmyk_to_rgb(&[255, 255, 255, 255]), [255, 255, 255]);
        assert_eq!(cmyk_to_rgb(&[255, 0, 128, 0]), [0, 0, 0]);
    }
}
