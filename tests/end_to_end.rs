//! End-to-end: scan real files, auto-select, run an action, re-scan.
//! Only uses the public API, exactly like the GUI does.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use dupe_finder::actions::{self, ActionItem, ActionKind, ActionOptions};
use dupe_finder::model::{CompareMode, DupGroup, FolderEntry, FolderRole, Method, ScanConfig};
use dupe_finder::scan::{Progress, run_scan};
use dupe_finder::selection::{self, KeepRule};

struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let p = std::env::temp_dir().join(format!("dupe_finder_it_{tag}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        Self(p)
    }

    fn write(&self, rel: &str, data: &[u8]) -> PathBuf {
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

fn picture(w: u32, h: u32, seed: u32) -> image::RgbImage {
    image::RgbImage::from_fn(w, h, |x, y| {
        let (fx, fy) = (x * 400 / w, y * 300 / h);
        image::Rgb([
            ((fx + seed * 37) % 256) as u8,
            ((fy * 255 / 300 + seed * 91) % 256) as u8,
            (((fx / 40) ^ (fy / 30)).wrapping_mul(seed + 7) % 200) as u8,
        ])
    })
}

/// Same logic the GUI uses: each marked file gets the first unmarked file as keeper.
fn items_for(cfg: &ScanConfig, groups: &[DupGroup], marked: &HashSet<PathBuf>) -> Vec<ActionItem> {
    groups
        .iter()
        .flat_map(|g| {
            let keeper = g
                .files
                .iter()
                .find(|f| !marked.contains(&f.path))
                .map(|f| f.path.clone());
            g.files
                .iter()
                .filter(|f| marked.contains(&f.path))
                .map(move |f| ActionItem {
                    path: f.path.clone(),
                    size: f.size,
                    modified: f.modified,
                    root: cfg.folders[f.folder].path.clone(),
                    keeper: keeper.clone(),
                })
        })
        .collect()
}

#[test]
fn clean_backup_against_originals_recursively() {
    let d = TempDir::new("backup");
    let video = vec![42u8; 600_000];
    d.write("originals/2021/holiday/clip.mp4", &video);
    d.write("originals/notes.txt", b"keep me");
    d.write("backup/old/2021/clip.mp4", &video);
    d.write("backup/old/2021/deeper/clip (1).mp4", &video);
    d.write("backup/old/notes.txt", b"keep me");
    d.write("backup/unique.txt", b"only here");

    let cfg = ScanConfig {
        folders: vec![
            FolderEntry {
                path: d.0.join("originals"),
                role: FolderRole::Source,
            },
            FolderEntry {
                path: d.0.join("backup"),
                role: FolderRole::Target,
            },
        ],
        mode: CompareMode::SourceVsTarget,
        ..Default::default()
    };
    let r = run_scan(&cfg, &Progress::default()).unwrap();
    assert_eq!(r.groups.len(), 2);

    let mut marked = HashSet::new();
    selection::apply_keep_rule(&r.groups, &mut marked, KeepRule::SourceFolders);
    assert_eq!(marked.len(), 3, "all backup copies marked, originals kept");
    assert!(marked.iter().all(|p| p.starts_with(d.0.join("backup"))));

    let items = items_for(&cfg, &r.groups, &marked);
    let report = actions::execute(
        &ActionKind::Delete,
        &items,
        ActionOptions {
            remove_empty_dirs: true,
        },
        &Progress::default(),
    );
    assert!(report.failed.is_empty(), "{:?}", report.failed);
    assert_eq!(report.freed, 600_000 * 2 + 7);
    assert!(
        !d.0.join("backup/old").exists(),
        "emptied folders cleaned up"
    );
    assert!(d.0.join("backup/unique.txt").exists());
    assert!(d.0.join("originals/2021/holiday/clip.mp4").exists());

    let again = run_scan(&cfg, &Progress::default()).unwrap();
    assert!(again.groups.is_empty());
}

#[test]
fn same_photo_in_other_format_and_size_is_found_and_best_kept() {
    let d = TempDir::new("photos");
    let img = image::DynamicImage::ImageRgb8(picture(800, 600, 4));
    img.save(d.0.join("IMG_0001.jpg")).unwrap();
    std::fs::create_dir_all(d.0.join("export")).unwrap();
    img.resize(400, 300, image::imageops::FilterType::Triangle)
        .save(d.0.join("export/IMG_0001_small.png"))
        .unwrap();
    image::DynamicImage::ImageRgb8(picture(800, 600, 77))
        .fliph()
        .save(d.0.join("other.jpg"))
        .unwrap();

    let cfg = ScanConfig {
        folders: vec![FolderEntry {
            path: d.0.clone(),
            role: FolderRole::Source,
        }],
        method: Method::SimilarImage,
        ..Default::default()
    };
    let r = run_scan(&cfg, &Progress::default()).unwrap();
    assert_eq!(r.groups.len(), 1, "{:?}", r.groups);
    assert_eq!(r.groups[0].files.len(), 2);

    let mut marked = HashSet::new();
    selection::apply_keep_rule(&r.groups, &mut marked, KeepRule::HighestResolution);
    let marked: Vec<&Path> = marked.iter().map(|p| p.as_path()).collect();
    assert_eq!(
        marked,
        vec![d.0.join("export/IMG_0001_small.png").as_path()]
    );
}

#[test]
fn quarantine_move_then_links() {
    let d = TempDir::new("quarantine");
    let data = b"duplicate payload".repeat(1000);
    d.write("lib/a.bin", &data);
    d.write("lib/sub/b.bin", &data);
    d.write("lib/sub/c.bin", &data);
    let cfg = ScanConfig {
        folders: vec![FolderEntry {
            path: d.0.join("lib"),
            role: FolderRole::Source,
        }],
        ..Default::default()
    };
    let r = run_scan(&cfg, &Progress::default()).unwrap();
    assert_eq!(r.groups[0].files.len(), 3);

    // Move one copy to a quarantine folder, keeping its relative location.
    let mut marked = HashSet::new();
    marked.insert(d.0.join("lib/sub/b.bin"));
    let q = d.0.join("quarantine");
    let kind = ActionKind::MoveTo {
        dest: q.clone(),
        keep_structure: true,
    };
    let rep = actions::execute(
        &kind,
        &items_for(&cfg, &r.groups, &marked),
        ActionOptions::default(),
        &Progress::default(),
    );
    assert!(rep.failed.is_empty(), "{:?}", rep.failed);
    assert!(q.join("lib/sub/b.bin").is_file());

    // Replace the remaining duplicate with a hard link: data stays reachable.
    let r = run_scan(&cfg, &Progress::default()).unwrap();
    let mut marked = HashSet::new();
    selection::apply_keep_rule(&r.groups, &mut marked, KeepRule::ShortestPath);
    let rep = actions::execute(
        &ActionKind::HardLink,
        &items_for(&cfg, &r.groups, &marked),
        ActionOptions::default(),
        &Progress::default(),
    );
    assert!(rep.failed.is_empty(), "{:?}", rep.failed);
    assert_eq!(std::fs::read(d.0.join("lib/sub/c.bin")).unwrap(), data);
}
