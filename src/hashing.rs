//! Byte-level hashing used for exact-duplicate detection.
//!
//! Strategy (same idea as fdupes/jdupes/czkawka):
//! 1. Files with a unique size cannot be duplicates (no I/O at all).
//! 2. A cheap *partial* hash of three chunks (start, middle, end) eliminates almost
//!    all remaining non-duplicates while reading at most 3 × [`CHUNK`] bytes per file.
//! 3. Only the survivors get a full streaming hash.
//!
//! xxh3-128 is used because it runs at memory bandwidth (many GB/s), so the disk
//! is always the bottleneck. It is not cryptographic, which is fine for this use.

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use xxhash_rust::xxh3::Xxh3;

/// Size of each chunk read by [`partial_hash`].
pub const CHUNK: u64 = 64 * 1024;

/// If true, the partial hash already covered every byte of the file.
pub fn partial_covers_whole_file(size: u64) -> bool {
    size <= CHUNK * 3
}

/// Hashes the first, middle and last [`CHUNK`] bytes plus the file size.
/// Small files are hashed completely (identical to [`full_hash`]).
pub fn partial_hash(path: &Path, size: u64) -> io::Result<u128> {
    if partial_covers_whole_file(size) {
        return full_hash(path, None);
    }
    let mut f = File::open(path)?;
    let mut h = Xxh3::new();
    let mut buf = vec![0u8; CHUNK as usize];
    for offset in [0, size / 2 - CHUNK / 2, size - CHUNK] {
        f.seek(SeekFrom::Start(offset))?;
        f.read_exact(&mut buf)?;
        h.update(&buf);
    }
    h.update(&size.to_le_bytes());
    Ok(h.digest128())
}

/// Streams the whole file through xxh3-128. Returns `Interrupted` if `cancel` is set.
pub fn full_hash(path: &Path, cancel: Option<&AtomicBool>) -> io::Result<u128> {
    let mut f = File::open(path)?;
    let mut h = Xxh3::new();
    let mut buf = vec![0u8; 1024 * 1024];
    loop {
        if cancel.is_some_and(|c| c.load(Ordering::Relaxed)) {
            return Err(io::Error::new(io::ErrorKind::Interrupted, "cancelled"));
        }
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Ok(h.digest128())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{TempDir, bytes};

    #[test]
    fn partial_differs_when_middle_differs() {
        let dir = TempDir::new("hash_mid");
        let size = (CHUNK * 10) as usize;
        let a = vec![7u8; size];
        let mut b = a.clone();
        b[size / 2] = 8;
        let pa = dir.write("a.bin", &a);
        let pb = dir.write("b.bin", &b);
        assert_ne!(
            partial_hash(&pa, size as u64).unwrap(),
            partial_hash(&pb, size as u64).unwrap()
        );
        assert_ne!(full_hash(&pa, None).unwrap(), full_hash(&pb, None).unwrap());
    }

    #[test]
    fn partial_misses_change_outside_sampled_chunks_but_full_does_not() {
        // This is the documented trade-off of "fast mode".
        let dir = TempDir::new("hash_gap");
        let size = (CHUNK * 10) as usize;
        let a = bytes(size, 1);
        let mut b = a.clone();
        b[(CHUNK * 2) as usize] ^= 0xFF;
        let pa = dir.write("a.bin", &a);
        let pb = dir.write("b.bin", &b);
        assert_eq!(
            partial_hash(&pa, size as u64).unwrap(),
            partial_hash(&pb, size as u64).unwrap()
        );
        assert_ne!(full_hash(&pa, None).unwrap(), full_hash(&pb, None).unwrap());
    }

    #[test]
    fn small_files_partial_equals_full() {
        let dir = TempDir::new("hash_small");
        for len in [0usize, 1, 1000, (CHUNK * 3) as usize] {
            let p = dir.write(&format!("f{len}"), &bytes(len, 3));
            assert!(partial_covers_whole_file(len as u64));
            assert_eq!(
                partial_hash(&p, len as u64).unwrap(),
                full_hash(&p, None).unwrap()
            );
        }
        assert!(!partial_covers_whole_file(CHUNK * 3 + 1));
    }

    #[test]
    fn identical_content_same_hash_regardless_of_name() {
        let dir = TempDir::new("hash_same");
        let data = bytes(500_000, 9);
        let a = dir.write("one.jpg", &data);
        let b = dir.write("deep/other.png", &data);
        assert_eq!(full_hash(&a, None).unwrap(), full_hash(&b, None).unwrap());
        assert_eq!(
            partial_hash(&a, data.len() as u64).unwrap(),
            partial_hash(&b, data.len() as u64).unwrap()
        );
    }

    #[test]
    fn cancel_and_missing_file_are_errors() {
        let dir = TempDir::new("hash_err");
        let p = dir.write("a", b"abc");
        let cancel = AtomicBool::new(true);
        let e = full_hash(&p, Some(&cancel)).unwrap_err();
        assert_eq!(e.kind(), io::ErrorKind::Interrupted);
        assert!(full_hash(&dir.path().join("missing"), None).is_err());
    }
}
