//! "Identical content" method: size → partial hash → full hash.
//! See [`crate::hashing`] for why this is fast.

use std::sync::Mutex;

use rayon::prelude::*;

use super::group::group_by;
use super::{CANCELLED, Progress};
use crate::hashing;
use crate::model::{FileEntry, ScanConfig};

pub(crate) fn group_by_content(
    files: Vec<FileEntry>,
    cfg: &ScanConfig,
    progress: &Progress,
    errors: &Mutex<Vec<String>>,
) -> Result<Vec<Vec<FileEntry>>, String> {
    // Stage 1: size (no I/O).
    let candidates: Vec<FileEntry> = group_by(files, |f| f.size).into_iter().flatten().collect();

    // Stage 2: partial hash (start/middle/end chunks).
    progress.set_stage("Quick hash (start/middle/end)", candidates.len() as u64);
    let hashed = hash_all(candidates, progress, errors, |f| {
        hashing::partial_hash(&f.path, f.size)
    })?;
    let groups = group_by(hashed, |(f, h)| (f.size, *h));

    if cfg.content_partial_only {
        return Ok(strip(groups));
    }

    // Stage 3: full hash only where the partial hash did not already read everything.
    let (exact, need_full): (Vec<_>, Vec<_>) = groups
        .into_iter()
        .partition(|g| hashing::partial_covers_whole_file(g[0].0.size));
    let need_full: Vec<FileEntry> = need_full.into_iter().flatten().map(|(f, _)| f).collect();
    progress.set_stage("Full hash", need_full.len() as u64);
    let hashed = hash_all(need_full, progress, errors, |f| {
        hashing::full_hash(&f.path, Some(&progress.cancel))
    })?;
    let mut out = strip(exact);
    out.extend(strip(group_by(hashed, |(f, h)| (f.size, *h))));
    Ok(out)
}

fn strip(groups: Vec<Vec<(FileEntry, u128)>>) -> Vec<Vec<FileEntry>> {
    groups
        .into_iter()
        .map(|g| g.into_iter().map(|(f, _)| f).collect())
        .collect()
}

/// Hashes files in parallel. Unreadable files are logged to `errors` and dropped.
fn hash_all(
    files: Vec<FileEntry>,
    progress: &Progress,
    errors: &Mutex<Vec<String>>,
    hash: impl Fn(&FileEntry) -> std::io::Result<u128> + Sync,
) -> Result<Vec<(FileEntry, u128)>, String> {
    let out: Vec<_> = files
        .into_par_iter()
        .filter_map(|f| {
            if progress.cancelled() {
                return None;
            }
            let r = hash(&f);
            progress.tick();
            match r {
                Ok(h) => Some((f, h)),
                Err(e) => {
                    if !progress.cancelled() {
                        errors
                            .lock()
                            .unwrap()
                            .push(format!("{}: {e}", f.path.display()));
                    }
                    None
                }
            }
        })
        .collect();
    if progress.cancelled() {
        return Err(CANCELLED.into());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hashing::CHUNK;
    use crate::test_util::{TempDir, bytes};

    fn entry(p: std::path::PathBuf) -> FileEntry {
        let size = std::fs::metadata(&p).unwrap().len();
        FileEntry::new(p, size)
    }

    fn run(files: Vec<FileEntry>, partial_only: bool) -> Vec<Vec<String>> {
        let cfg = ScanConfig {
            content_partial_only: partial_only,
            ..Default::default()
        };
        let mut g: Vec<Vec<String>> =
            group_by_content(files, &cfg, &Progress::default(), &Mutex::new(Vec::new()))
                .unwrap()
                .into_iter()
                .map(|g| {
                    let mut v: Vec<String> = g.iter().map(|f| f.name()).collect();
                    v.sort();
                    v
                })
                .collect();
        g.sort();
        g
    }

    #[test]
    fn exact_and_fast_mode() {
        let dir = TempDir::new("content");
        let big = bytes((CHUNK * 8) as usize, 1);
        let mut big_gap = big.clone();
        big_gap[(CHUNK * 2) as usize] ^= 1; // outside the sampled chunks
        let files = vec![
            entry(dir.write("a", &big)),
            entry(dir.write("b", &big)),
            entry(dir.write("c", &big_gap)),
            entry(dir.write("s1", b"small")),
            entry(dir.write("s2", b"small")),
            entry(dir.write("s3", b"SMALL")),
            entry(dir.write("unique", b"a unique size!")),
        ];
        assert_eq!(
            run(files.clone(), false),
            vec![vec!["a", "b"], vec!["s1", "s2"]]
        );
        // Fast mode cannot see the difference in `c`.
        assert_eq!(
            run(files, true),
            vec![vec!["a", "b", "c"], vec!["s1", "s2"]]
        );
    }

    #[test]
    fn unreadable_file_is_logged_and_skipped() {
        let dir = TempDir::new("content_err");
        let a = entry(dir.write("a", b"same"));
        let b = entry(dir.write("b", b"same"));
        let mut ghost = FileEntry::new(dir.path().join("ghost"), 4);
        ghost.size = 4;
        let errors = Mutex::new(Vec::new());
        let g = group_by_content(
            vec![a, b, ghost],
            &ScanConfig::default(),
            &Progress::default(),
            &errors,
        )
        .unwrap();
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].len(), 2);
        assert_eq!(errors.lock().unwrap().len(), 1);
    }
}
