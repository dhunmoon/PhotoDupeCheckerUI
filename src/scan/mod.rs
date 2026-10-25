//! The scan pipeline. Runs on a background thread; see [`run_scan`].
//!
//! ```text
//! collect files ─┬─ name / size methods ──────────────┐
//!   (collect)    ├─ content:  size → quick → full hash ├─ Source-vs-Target filter
//!                └─ similar:  perceptual hash + BK-tree┘   → image dims → sort → groups
//! ```

mod collect;
mod content;
mod group;
mod progress;
mod similar;

use std::sync::Mutex;
use std::time::{Duration, Instant};

pub use progress::Progress;

use crate::model::{CompareMode, DupGroup, FileEntry, FolderRole, Method, ScanConfig};
use group::{group_by, group_label, name_key};

/// Error message returned when the user cancels a scan.
pub const CANCELLED: &str = "Scan cancelled";

/// Everything the UI needs to show after a successful scan.
pub struct ScanResult {
    /// Sorted by wasted space (largest first); ids are 1-based in that order.
    pub groups: Vec<DupGroup>,
    /// Number of files that passed the filters and were considered.
    pub files_scanned: usize,
    pub elapsed: Duration,
    /// Non-fatal problems (unreadable files, permission errors, …).
    pub errors: Vec<String>,
}

impl ScanResult {
    pub fn duplicate_files(&self) -> usize {
        self.groups.iter().map(|g| g.files.len()).sum()
    }

    pub fn reclaimable(&self) -> u64 {
        self.groups.iter().map(|g| g.wasted()).sum()
    }
}

/// Runs a complete scan. Blocking; call from a worker thread and watch `progress`.
pub fn run_scan(cfg: &ScanConfig, progress: &Progress) -> Result<ScanResult, String> {
    let start = Instant::now();
    cfg.validate()?;

    let errors = Mutex::new(Vec::new());
    let files = collect::collect_files(cfg, progress, &errors)?;
    let files_scanned = files.len();

    let mut groups: Vec<Vec<FileEntry>> = match cfg.method {
        Method::FileName => group_by(files, |f| name_key(f, cfg)),
        Method::NameAndSize => group_by(files, |f| (name_key(f, cfg), f.size)),
        Method::Size => group_by(files, |f| f.size),
        Method::Content => content::group_by_content(files, cfg, progress, &errors)?,
        Method::SimilarImage => similar::group_similar_images(files, cfg, progress, &errors)?,
    };

    if cfg.mode == CompareMode::SourceVsTarget {
        groups.retain(|g| {
            g.iter().any(|f| f.role == FolderRole::Source)
                && g.iter().any(|f| f.role == FolderRole::Target)
        });
    }

    similar::fill_image_dims(&mut groups, progress)?;

    // Similar-image groups are already sorted by distance to the reference image.
    if cfg.method != Method::SimilarImage {
        for g in &mut groups {
            // Source files first, then by path, so the first file is a sensible "keeper".
            g.sort_by(|a, b| {
                (a.role != FolderRole::Source, &a.path)
                    .cmp(&(b.role != FolderRole::Source, &b.path))
            });
        }
    }

    let mut groups: Vec<DupGroup> = groups
        .into_iter()
        .map(|files| DupGroup {
            id: 0,
            label: group_label(cfg, &files),
            files,
        })
        .collect();
    groups.sort_by(|a, b| {
        b.wasted()
            .cmp(&a.wasted())
            .then(b.files.len().cmp(&a.files.len()))
            .then_with(|| a.files[0].path.cmp(&b.files[0].path))
    });
    for (i, g) in groups.iter_mut().enumerate() {
        g.id = i + 1;
    }

    Ok(ScanResult {
        groups,
        files_scanned,
        elapsed: start.elapsed(),
        errors: errors.into_inner().unwrap(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::FolderEntry;
    use crate::test_util::{TempDir, bytes};

    fn setup() -> (TempDir, ScanConfig) {
        let dir = TempDir::new("scan");
        let big = bytes(1_000_000, 1);
        dir.write("src/a.bin", &big);
        dir.write("src/sub/a_copy.bin", &big);
        dir.write("dst/deep/x/a.bin", &big);
        let mut other = big.clone();
        other[500_123] ^= 1;
        dir.write("dst/not_dupe.bin", &other);
        dir.write("dst/t1.txt", b"hello");
        dir.write("dst/t2.txt", b"hello");
        let cfg = ScanConfig {
            folders: vec![
                FolderEntry {
                    path: dir.path().join("src"),
                    role: FolderRole::Source,
                },
                FolderEntry {
                    path: dir.path().join("dst"),
                    role: FolderRole::Target,
                },
            ],
            ..Default::default()
        };
        (dir, cfg)
    }

    #[test]
    fn content_all_together() {
        let (_d, cfg) = setup();
        let r = run_scan(&cfg, &Progress::default()).unwrap();
        assert_eq!(r.files_scanned, 6);
        assert_eq!(r.groups.len(), 2);
        let g = &r.groups[0];
        assert_eq!(g.id, 1);
        assert_eq!(g.files.len(), 3);
        assert_eq!(g.files[0].role, FolderRole::Source, "sources sorted first");
        assert!(g.files.iter().all(|f| !f.path.ends_with("not_dupe.bin")));
        assert_eq!(r.duplicate_files(), 5);
        assert_eq!(r.reclaimable(), 2_000_000 + 5);
    }

    #[test]
    fn source_vs_target_drops_target_only_groups() {
        let (_d, mut cfg) = setup();
        cfg.mode = CompareMode::SourceVsTarget;
        let r = run_scan(&cfg, &Progress::default()).unwrap();
        assert_eq!(r.groups.len(), 1, "txt files are target-only");
    }

    #[test]
    fn metadata_methods() {
        let (_d, mut cfg) = setup();
        cfg.method = Method::FileName;
        let r = run_scan(&cfg, &Progress::default()).unwrap();
        assert_eq!(r.groups.len(), 1);
        assert_eq!(r.groups[0].label, "Name: a.bin");

        cfg.method = Method::Size;
        let r = run_scan(&cfg, &Progress::default()).unwrap();
        assert_eq!(r.groups.len(), 2);
        assert_eq!(
            r.groups[0].files.len(),
            4,
            "same size includes the near-copy"
        );

        cfg.method = Method::NameAndSize;
        let r = run_scan(&cfg, &Progress::default()).unwrap();
        assert_eq!(r.groups.len(), 1);
        assert_eq!(r.groups[0].files.len(), 2);
    }

    #[test]
    fn nested_folders_do_not_self_match() {
        let (d, mut cfg) = setup();
        cfg.folders.push(FolderEntry {
            path: d.path().join("src/sub"),
            role: FolderRole::Source,
        });
        let r = run_scan(&cfg, &Progress::default()).unwrap();
        assert_eq!(r.groups[0].files.len(), 3);
    }

    #[test]
    fn invalid_config_and_cancel() {
        let p = Progress::default();
        assert!(run_scan(&ScanConfig::default(), &p).is_err());
        let (_d, cfg) = setup();
        p.request_cancel();
        assert_eq!(run_scan(&cfg, &p).err().as_deref(), Some(CANCELLED));
    }
}
