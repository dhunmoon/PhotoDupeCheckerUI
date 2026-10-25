//! Perceptual hashing for "same picture, different file" detection.
//!
//! # Research: why not hash the first N pixels or just the border?
//!
//! * Pixels only exist after decoding, and decoding (not hashing) is the expensive part.
//!   Reading "the first N pixels" of a JPEG/PNG still requires entropy-decoding the
//!   stream up to that point, and those pixels change completely when the image is
//!   resized, re-compressed or converted to another format.
//! * Borders are frequently uniform (sky, white margins, black bars) and are the first
//!   thing cropping removes, so they produce both false positives and misses.
//!
//! # What we do instead
//!
//! * JPEGs are decoded directly at reduced scale (1/2, 1/4 or 1/8) using the DCT
//!   scaling of `jpeg-decoder` ([`load::load_small`]). This skips most of the IDCT work
//!   and is several times faster than a full decode for multi-megapixel photos.
//! * The small image is converted to a 64×64 grayscale square and hashed with a
//!   128-bit difference hash (horizontal + vertical dHash). The hash survives format
//!   changes, resizing, mild re-compression and small colour changes.
//! * Similar hashes are found with a [`BkTree`], so we never compare every pair of images.
//!
//! For *exact* duplicates the cheap trick that does work is sampling bytes, not pixels:
//! see [`crate::hashing`].

mod bktree;
mod load;

use std::path::Path;

use image::imageops::{self, FilterType};
use image::{DynamicImage, GrayImage};

pub use bktree::BkTree;
pub use load::{LoadedImage, dimensions, load_small};

/// Side of the grayscale square that the dHash is computed from.
const SQ: u32 = 64;

/// Perceptual hashes of one image.
pub struct ImageHashes {
    pub primary: u128,
    /// Hashes of the 7 other rotations/flips (only computed when requested).
    pub variants: Vec<u128>,
    /// Original pixel dimensions.
    pub dims: (u32, u32),
}

impl ImageHashes {
    /// The primary hash followed by all variants.
    pub fn all(&self) -> impl Iterator<Item = u128> + '_ {
        std::iter::once(self.primary).chain(self.variants.iter().copied())
    }
}

/// Number of differing bits between two hashes (0 = identical, 128 = opposite).
pub fn distance(a: u128, b: u128) -> u32 {
    (a ^ b).count_ones()
}

/// Decodes `path` at reduced size and computes its perceptual hash(es).
/// With `with_variants`, also hashes the 3 rotations and 4 mirror images.
pub fn hash_image(path: &Path, with_variants: bool) -> Result<ImageHashes, String> {
    let loaded = load_small(path, 128)?;
    let sq = square_gray(&loaded.image);
    let primary = dhash128(&sq);
    let variants = if with_variants {
        let r90 = imageops::rotate90(&sq);
        let r180 = imageops::rotate180(&sq);
        let r270 = imageops::rotate270(&sq);
        vec![
            dhash128(&r90),
            dhash128(&r180),
            dhash128(&r270),
            dhash128(&imageops::flip_horizontal(&sq)),
            dhash128(&imageops::flip_vertical(&sq)),
            dhash128(&imageops::flip_horizontal(&r90)),
            dhash128(&imageops::flip_horizontal(&r270)),
        ]
    } else {
        Vec::new()
    };
    Ok(ImageHashes {
        primary,
        variants,
        dims: loaded.orig_dims,
    })
}

fn square_gray(img: &DynamicImage) -> GrayImage {
    // Thumbnail first (fast box-ish filter) then exact resize: keeps it cheap for big images.
    let small = if img.width() > 256 || img.height() > 256 {
        img.thumbnail(256, 256)
    } else {
        img.clone()
    };
    imageops::resize(&small.to_luma8(), SQ, SQ, FilterType::Triangle)
}

/// 128-bit difference hash: 64 bits horizontal gradient + 64 bits vertical gradient.
fn dhash128(sq: &GrayImage) -> u128 {
    let h = imageops::resize(sq, 9, 8, FilterType::Triangle);
    let v = imageops::resize(sq, 8, 9, FilterType::Triangle);
    let mut out: u128 = 0;
    let mut bit = 0;
    for y in 0..8 {
        for x in 0..8 {
            if h.get_pixel(x, y)[0] > h.get_pixel(x + 1, y)[0] {
                out |= 1 << bit;
            }
            bit += 1;
        }
    }
    for y in 0..8 {
        for x in 0..8 {
            if v.get_pixel(x, y)[0] > v.get_pixel(x, y + 1)[0] {
                out |= 1 << bit;
            }
            bit += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{TempDir, test_image};

    #[test]
    fn distance_counts_bits() {
        assert_eq!(distance(0, 0), 0);
        assert_eq!(distance(0, u128::MAX), 128);
        assert_eq!(distance(0b1010, 0b0110), 2);
    }

    #[test]
    fn resized_reencoded_and_rotated_copies_hash_close() {
        let dir = TempDir::new("phash");
        let img = test_image(400, 300, 1);
        let a = dir.path().join("a.png");
        let b = dir.path().join("b.jpg");
        let c = dir.path().join("c.png");
        img.save(&a).unwrap();
        DynamicImage::ImageRgb8(img.clone())
            .resize(200, 150, FilterType::Triangle)
            .to_rgb8()
            .save(&b)
            .unwrap();
        DynamicImage::ImageRgb8(img).rotate90().save(&c).unwrap();
        let ha = hash_image(&a, false).unwrap();
        let hb = hash_image(&b, false).unwrap();
        let hc = hash_image(&c, true).unwrap();
        assert!(ha.variants.is_empty());
        assert_eq!(hc.variants.len(), 7);
        assert_eq!(ha.dims, (400, 300));
        assert_eq!(hb.dims, (200, 150));
        assert!(distance(ha.primary, hb.primary) <= 10);
        assert!(hc.all().any(|v| distance(v, ha.primary) <= 10));
    }

    #[test]
    fn different_pictures_are_far_apart() {
        let dir = TempDir::new("phash_far");
        let a = dir.path().join("a.png");
        let b = dir.path().join("b.png");
        test_image(300, 300, 1).save(&a).unwrap();
        DynamicImage::ImageRgb8(test_image(300, 300, 50))
            .fliph()
            .save(&b)
            .unwrap();
        let (ha, hb) = (
            hash_image(&a, false).unwrap(),
            hash_image(&b, false).unwrap(),
        );
        assert!(distance(ha.primary, hb.primary) > 20);
    }

    #[test]
    fn all_iterates_primary_then_variants() {
        let h = ImageHashes {
            primary: 1,
            variants: vec![2, 3],
            dims: (1, 1),
        };
        assert_eq!(h.all().collect::<Vec<_>>(), vec![1, 2, 3]);
    }
}
