//! Stage 1 of every scan: walk the selected folders and build the list of candidate files.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use walkdir::WalkDir;

use super::{CANCELLED, Progress};
use crate::model::{FileEntry, FileKind, ScanConfig};

#[cfg(windows)]
fn is_hidden(e: &walkdir::DirEntry) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
    e.file_name().to_string_lossy().starts_with('.')
        || e.metadata()
            .map(|m| m.file_attributes() & FILE_ATTRIBUTE_HIDDEN != 0)
            .unwrap_or(false)
}

#[cfg(not(windows))]
fn is_hidden(e: &walkdir::DirEntry) -> bool {
    e.file_name().to_string_lossy().starts_with('.')
}

/// Splits the comma separated exclude list into lower-cased, non-empty patterns.
pub(crate) fn parse_excludes(s: &str) -> Vec<String> {
    s.split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Walks every folder in `cfg` and returns the files that pass the filters
/// (type, size, hidden, excludes). Non-fatal errors are appended to `errors`.
///
/// If folders overlap (one is inside another) each file is returned only once, attributed
/// to the deepest folder, so a file is never reported as a duplicate of itself.
pub(crate) fn collect_files(
    cfg: &ScanConfig,
    progress: &Progress,
    errors: &Mutex<Vec<String>>,
) -> Result<Vec<FileEntry>, String> {
    progress.set_stage("Collecting files", 0);
    let excludes = parse_excludes(&cfg.excludes);

    // canonical path -> (depth of scan root, entry)
    let mut by_path: HashMap<PathBuf, (usize, FileEntry)> = HashMap::new();

    for (folder_idx, folder) in cfg.folders.iter().enumerate() {
        let root_depth = folder.path.components().count();
        let walker = WalkDir::new(&folder.path)
            .follow_links(cfg.follow_symlinks)
            .max_depth(if cfg.recursive { usize::MAX } else { 1 })
            .into_iter()
            .filter_entry(|e| {
                (e.depth() == 0 || cfg.include_hidden || !is_hidden(e))
                    && (excludes.is_empty() || {
                        let p = e.path().to_string_lossy().to_lowercase();
                        !excludes.iter().any(|x| p.contains(x.as_str()))
                    })
            });
        for entry in walker {
            if progress.cancelled() {
                return Err(CANCELLED.into());
            }
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    errors.lock().unwrap().push(e.to_string());
                    continue;
                }
            };
            if !entry.file_type().is_file() && !(cfg.follow_symlinks && entry.path().is_file()) {
                continue;
            }
            let path = entry.path().to_path_buf();
            let kind = FileKind::from_path(&path);
            if !cfg.kinds[kind.index()] {
                continue;
            }
            let meta = match entry.metadata() {
                Ok(m) => m,
                Err(e) => {
                    errors
                        .lock()
                        .unwrap()
                        .push(format!("{}: {e}", path.display()));
                    continue;
                }
            };
            let size = meta.len();
            if size < cfg.min_size || (cfg.max_size > 0 && size > cfg.max_size) {
                continue;
            }
            progress.tick();
            let canonical = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
            if let Some((depth, _)) = by_path.get(&canonical)
                && *depth >= root_depth
            {
                continue;
            }
            by_path.insert(
                canonical,
                (
                    root_depth,
                    FileEntry {
                        path,
                        size,
                        modified: meta.modified().ok(),
                        kind,
                        folder: folder_idx,
                        role: folder.role,
                        dims: None,
                        distance: None,
                    },
                ),
            );
        }
    }
    Ok(by_path.into_values().map(|(_, f)| f).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{FolderEntry, FolderRole};
    use crate::test_util::TempDir;

    fn names(files: &[FileEntry]) -> Vec<String> {
        let mut v: Vec<String> = files.iter().map(|f| f.name()).collect();
        v.sort();
        v
    }

    fn cfg_for(dir: &TempDir) -> ScanConfig {
        ScanConfig {
            folders: vec![FolderEntry {
                path: dir.path().to_path_buf(),
                role: FolderRole::Source,
            }],
            ..Default::default()
        }
    }

    fn collect(cfg: &ScanConfig) -> Vec<FileEntry> {
        let errors = Mutex::new(Vec::new());
        collect_files(cfg, &Progress::default(), &errors).unwrap()
    }

    #[test]
    fn excludes_are_trimmed_and_lowercased() {
        assert_eq!(
            parse_excludes(" Node_Modules, ,.GIT "),
            vec!["node_modules", ".git"]
        );
        assert!(parse_excludes("").is_empty());
    }

    #[test]
    fn filters_hidden_recursive_size_kind_and_excludes() {
        let dir = TempDir::new("collect_filters");
        dir.write("a.jpg", b"1234");
        dir.write("sub/b.txt", b"12345678");
        dir.write(".hidden/c.txt", b"1234");
        dir.write(".dot.txt", b"1234");
        dir.write("cache/d.txt", b"1234");
        dir.write("empty.txt", b"");

        let mut cfg = cfg_for(&dir);
        assert_eq!(
            names(&collect(&cfg)),
            ["a.jpg", "b.txt", "d.txt"],
            "empty + hidden skipped"
        );

        cfg.include_hidden = true;
        assert_eq!(
            names(&collect(&cfg)),
            [".dot.txt", "a.jpg", "b.txt", "c.txt", "d.txt"]
        );
        cfg.include_hidden = false;

        cfg.excludes = "CACHE".into();
        assert_eq!(names(&collect(&cfg)), ["a.jpg", "b.txt"]);
        cfg.excludes.clear();

        cfg.recursive = false;
        assert_eq!(names(&collect(&cfg)), ["a.jpg"]);
        cfg.recursive = true;

        cfg.min_size = 5;
        assert_eq!(names(&collect(&cfg)), ["b.txt"]);
        cfg.min_size = 0;
        cfg.max_size = 4;
        assert_eq!(names(&collect(&cfg)), ["a.jpg", "d.txt", "empty.txt"]);
        cfg.max_size = 0;
        cfg.min_size = 1;

        cfg.kinds[FileKind::Document.index()] = false;
        assert_eq!(names(&collect(&cfg)), ["a.jpg"]);
    }

    #[test]
    fn nested_folders_report_each_file_once_with_deepest_role() {
        let dir = TempDir::new("collect_nested");
        dir.write("top.bin", b"x");
        dir.write("inner/deep.bin", b"y");
        let cfg = ScanConfig {
            folders: vec![
                FolderEntry {
                    path: dir.path().to_path_buf(),
                    role: FolderRole::Source,
                },
                FolderEntry {
                    path: dir.path().join("inner"),
                    role: FolderRole::Target,
                },
            ],
            ..Default::default()
        };
        let files = collect(&cfg);
        assert_eq!(files.len(), 2);
        let deep = files.iter().find(|f| f.name() == "deep.bin").unwrap();
        assert_eq!(deep.role, FolderRole::Target);
        assert_eq!(deep.folder, 1);
    }

    #[test]
    fn missing_folder_is_reported_as_error_not_panic() {
        let dir = TempDir::new("collect_missing");
        let cfg = ScanConfig {
            folders: vec![FolderEntry {
                path: dir.path().join("nope"),
                role: FolderRole::Source,
            }],
            ..Default::default()
        };
        let errors = Mutex::new(Vec::new());
        let files = collect_files(&cfg, &Progress::default(), &errors).unwrap();
        assert!(files.is_empty());
        assert_eq!(errors.lock().unwrap().len(), 1);
    }

    #[test]
    fn cancel_stops_collection() {
        let dir = TempDir::new("collect_cancel");
        dir.write("a.txt", b"1");
        let p = Progress::default();
        p.request_cancel();
        let r = collect_files(&cfg_for(&dir), &p, &Mutex::new(Vec::new()));
        assert_eq!(r.unwrap_err(), CANCELLED);
    }
}
