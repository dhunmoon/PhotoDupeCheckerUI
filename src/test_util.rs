//! Helpers shared by unit tests (compiled only for `cargo test`).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

/// A unique temporary directory that is deleted when dropped.
pub struct TempDir(PathBuf);

impl TempDir {
    pub fn new(tag: &str) -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("dupe_finder_{tag}_{}_{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }

    /// Writes `data` to `rel` (creating parent folders) and returns the full path.
    pub fn write(&self, rel: &str, data: &[u8]) -> PathBuf {
        let p = self.0.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, data).unwrap();
        p
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Deterministic pseudo-random bytes (so tests don't need a RNG crate).
pub fn bytes(len: usize, seed: u32) -> Vec<u8> {
    let mut x = seed.wrapping_mul(2_654_435_761).wrapping_add(1);
    (0..len)
        .map(|_| {
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;
            x as u8
        })
        .collect()
}

/// A colourful synthetic test picture with enough structure for perceptual hashing.
pub fn test_image(w: u32, h: u32, seed: u32) -> image::RgbImage {
    image::RgbImage::from_fn(w, h, |x, y| {
        let (fx, fy) = (x * 400 / w, y * 300 / h);
        image::Rgb([
            ((fx + seed * 37) % 256) as u8,
            ((fy * 255 / 300 + seed * 91) % 256) as u8,
            (((fx / 40) ^ (fy / 30)).wrapping_mul(seed + 7) % 200) as u8,
        ])
    })
}
