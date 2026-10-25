//! "Similar-looking images" method, plus reading image dimensions for result display.

use std::collections::HashMap;
use std::sync::Mutex;

use rayon::prelude::*;

use super::group::UnionFind;
use super::{CANCELLED, Progress};
use crate::model::{FileEntry, FolderRole, ScanConfig, is_decodable_image};
use crate::phash::{self, BkTree, ImageHashes};

/// Groups decodable images whose perceptual hashes are within
/// `cfg.similarity_threshold` bits of each other (transitively).
///
/// In each group the reference image is the highest resolution one; every file gets its
/// [`FileEntry::distance`] to that reference and the group is sorted closest-first
/// (ties: larger image, then Source folders, then path).
pub(crate) fn group_similar_images(
    files: Vec<FileEntry>,
    cfg: &ScanConfig,
    progress: &Progress,
    errors: &Mutex<Vec<String>>,
) -> Result<Vec<Vec<FileEntry>>, String> {
    let images: Vec<FileEntry> = files
        .into_iter()
        .filter(|f| is_decodable_image(&f.path))
        .collect();
    progress.set_stage("Computing perceptual hashes", images.len() as u64);
    let mut hashed: Vec<(FileEntry, ImageHashes)> = images
        .into_par_iter()
        .filter_map(|mut f| {
            if progress.cancelled() {
                return None;
            }
            let r = phash::hash_image(&f.path, cfg.ignore_rotation);
            progress.tick();
            match r {
                Ok(h) => {
                    f.dims = Some(h.dims);
                    Some((f, h))
                }
                Err(e) => {
                    errors
                        .lock()
                        .unwrap()
                        .push(format!("{}: {e}", f.path.display()));
                    None
                }
            }
        })
        .collect();
    if progress.cancelled() {
        return Err(CANCELLED.into());
    }
    // Deterministic order regardless of thread scheduling.
    hashed.sort_by(|a, b| a.0.path.cmp(&b.0.path));

    progress.set_stage("Grouping similar images", hashed.len() as u64);
    let mut tree = BkTree::default();
    for (i, (_, h)) in hashed.iter().enumerate() {
        tree.insert(h.primary, i);
    }
    let mut uf = UnionFind::new(hashed.len());
    for (i, (_, h)) in hashed.iter().enumerate() {
        if progress.cancelled() {
            return Err(CANCELLED.into());
        }
        for q in h.all() {
            tree.query(q, cfg.similarity_threshold, |j, _| {
                if j != i {
                    uf.union(i, j);
                }
            });
        }
        progress.tick();
    }

    let mut buckets: HashMap<usize, Vec<usize>> = HashMap::new();
    for i in 0..hashed.len() {
        buckets.entry(uf.find(i)).or_default().push(i);
    }

    let mut out = Vec::new();
    for idxs in buckets.into_values().filter(|v| v.len() > 1) {
        let reference = *idxs
            .iter()
            .max_by_key(|&&i| (hashed[i].0.pixels(), hashed[i].0.size))
            .unwrap();
        let ref_hash = hashed[reference].1.primary;
        let mut group: Vec<FileEntry> = idxs
            .iter()
            .map(|&i| {
                let (f, h) = &hashed[i];
                let mut f = f.clone();
                f.distance = Some(
                    h.all()
                        .map(|v| phash::distance(v, ref_hash))
                        .min()
                        .unwrap_or(0),
                );
                f
            })
            .collect();
        // Closest first; on ties prefer bigger images, then Source folders, then path.
        group.sort_by(|a, b| {
            (
                a.distance,
                std::cmp::Reverse(a.pixels()),
                a.role != FolderRole::Source,
                &a.path,
            )
                .cmp(&(
                    b.distance,
                    std::cmp::Reverse(b.pixels()),
                    b.role != FolderRole::Source,
                    &b.path,
                ))
        });
        out.push(group);
    }
    Ok(out)
}

/// Reads pixel dimensions (header only) for every decodable image that doesn't have them.
pub(crate) fn fill_image_dims(
    groups: &mut [Vec<FileEntry>],
    progress: &Progress,
) -> Result<(), String> {
    let mut todo: Vec<&mut FileEntry> = groups
        .iter_mut()
        .flatten()
        .filter(|f| f.dims.is_none() && is_decodable_image(&f.path))
        .collect();
    if todo.is_empty() {
        return Ok(());
    }
    progress.set_stage("Reading image dimensions", todo.len() as u64);
    todo.par_iter_mut().for_each(|f| {
        if !progress.cancelled() {
            f.dims = phash::dimensions(&f.path);
            progress.tick();
        }
    });
    if progress.cancelled() {
        return Err(CANCELLED.into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{TempDir, test_image};
    use image::DynamicImage;
    use image::imageops::FilterType;

    fn entry(p: std::path::PathBuf) -> FileEntry {
        let size = std::fs::metadata(&p).unwrap().len();
        FileEntry::new(p, size)
    }

    #[test]
    fn finds_resized_and_reformatted_copy_but_not_other_image() {
        let dir = TempDir::new("similar");
        let img = DynamicImage::ImageRgb8(test_image(400, 300, 1));
        let a = dir.path().join("orig.png");
        let b = dir.path().join("small.jpg");
        let c = dir.path().join("other.png");
        img.save(&a).unwrap();
        img.resize(160, 120, FilterType::Triangle)
            .to_rgb8()
            .save(&b)
            .unwrap();
        DynamicImage::ImageRgb8(test_image(400, 300, 50))
            .fliph()
            .save(&c)
            .unwrap();
        let txt = dir.write("notes.txt", b"not an image");

        let files = vec![entry(a), entry(b), entry(c), entry(txt)];
        let errors = Mutex::new(Vec::new());
        let groups =
            group_similar_images(files, &ScanConfig::default(), &Progress::default(), &errors)
                .unwrap();
        assert_eq!(groups.len(), 1, "{groups:?}");
        let g = &groups[0];
        assert_eq!(g.len(), 2);
        // Reference (largest) first with distance 0.
        assert_eq!(g[0].name(), "orig.png");
        assert_eq!(g[0].distance, Some(0));
        assert_eq!(g[0].dims, Some((400, 300)));
        assert_eq!(g[1].dims, Some((160, 120)));
        assert!(errors.lock().unwrap().is_empty());
    }

    #[test]
    fn rotation_matching_is_opt_in() {
        let dir = TempDir::new("similar_rot");
        let img = DynamicImage::ImageRgb8(test_image(300, 200, 3));
        let a = dir.path().join("a.png");
        let b = dir.path().join("b.png");
        img.save(&a).unwrap();
        img.rotate90().save(&b).unwrap();
        let files = vec![entry(a), entry(b)];
        let mut cfg = ScanConfig {
            similarity_threshold: 8,
            ..Default::default()
        };
        let run = |cfg: &ScanConfig| {
            group_similar_images(
                files.clone(),
                cfg,
                &Progress::default(),
                &Mutex::new(vec![]),
            )
            .unwrap()
            .len()
        };
        assert_eq!(run(&cfg), 0);
        cfg.ignore_rotation = true;
        assert_eq!(run(&cfg), 1);
    }

    #[test]
    fn corrupt_image_is_logged() {
        let dir = TempDir::new("similar_bad");
        let bad = dir.write("broken.png", b"definitely not a png");
        let errors = Mutex::new(Vec::new());
        let g = group_similar_images(
            vec![entry(bad)],
            &ScanConfig::default(),
            &Progress::default(),
            &errors,
        )
        .unwrap();
        assert!(g.is_empty());
        assert_eq!(errors.lock().unwrap().len(), 1);
    }

    #[test]
    fn fills_dimensions_only_for_images() {
        let dir = TempDir::new("dims");
        let p = dir.path().join("i.png");
        test_image(31, 17, 1).save(&p).unwrap();
        let t = dir.write("t.txt", b"x");
        let mut groups = vec![vec![entry(p), entry(t)]];
        fill_image_dims(&mut groups, &Progress::default()).unwrap();
        assert_eq!(groups[0][0].dims, Some((31, 17)));
        assert_eq!(groups[0][1].dims, None);
    }
}
