//! Scan results: individual files and groups of duplicates.

use std::path::PathBuf;
use std::time::SystemTime;

use super::{FileKind, FolderRole};

/// One file found during a scan.
#[derive(Clone, Debug)]
pub struct FileEntry {
    pub path: PathBuf,
    pub size: u64,
    pub modified: Option<SystemTime>,
    pub kind: FileKind,
    /// Index into `ScanConfig::folders` of the scan root this file was found under.
    pub folder: usize,
    pub role: FolderRole,
    /// Pixel dimensions (images only, filled in after grouping).
    pub dims: Option<(u32, u32)>,
    /// For similar-image groups: perceptual distance to the group's reference image.
    pub distance: Option<u32>,
}

impl FileEntry {
    /// Convenience constructor with no image metadata (handy for tests).
    pub fn new(path: impl Into<PathBuf>, size: u64) -> Self {
        let path = path.into();
        Self {
            kind: FileKind::from_path(&path),
            path,
            size,
            modified: None,
            folder: 0,
            role: FolderRole::Source,
            dims: None,
            distance: None,
        }
    }

    pub fn name(&self) -> String {
        self.path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default()
    }

    pub fn parent_display(&self) -> String {
        self.path
            .parent()
            .map(|p| p.display().to_string())
            .unwrap_or_default()
    }

    /// Width × height, or 0 when unknown.
    pub fn pixels(&self) -> u64 {
        self.dims.map(|(w, h)| w as u64 * h as u64).unwrap_or(0)
    }
}

/// A set of files that the chosen method considers duplicates of each other.
#[derive(Clone, Debug)]
pub struct DupGroup {
    /// 1-based, stable for the lifetime of a scan result.
    pub id: usize,
    /// Short human description of why these files match.
    pub label: String,
    pub files: Vec<FileEntry>,
}

impl DupGroup {
    pub fn total_size(&self) -> u64 {
        self.files.iter().map(|f| f.size).sum()
    }

    /// Space that would be freed by keeping only the largest file.
    pub fn wasted(&self) -> u64 {
        let max = self.files.iter().map(|f| f.size).max().unwrap_or(0);
        self.total_size() - max
    }

    pub fn main_kind(&self) -> FileKind {
        self.files
            .first()
            .map(|f| f.kind)
            .unwrap_or(FileKind::Other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_wasted_space_keeps_the_largest() {
        let g = DupGroup {
            id: 1,
            label: String::new(),
            files: vec![
                FileEntry::new("/a/x.jpg", 100),
                FileEntry::new("/b/x.jpg", 300),
                FileEntry::new("/c/x.png", 50),
            ],
        };
        assert_eq!(g.total_size(), 450);
        assert_eq!(g.wasted(), 150);
        assert_eq!(g.main_kind(), FileKind::Image);
    }

    #[test]
    fn empty_group_is_harmless() {
        let g = DupGroup {
            id: 1,
            label: String::new(),
            files: vec![],
        };
        assert_eq!(g.wasted(), 0);
        assert_eq!(g.main_kind(), FileKind::Other);
    }

    #[test]
    fn entry_helpers() {
        let mut f = FileEntry::new("/photos/2020/beach.JPG", 10);
        assert_eq!(f.name(), "beach.JPG");
        assert_eq!(f.parent_display(), "/photos/2020");
        assert_eq!(f.pixels(), 0);
        f.dims = Some((4000, 3000));
        assert_eq!(f.pixels(), 12_000_000);
    }
}
