//! Everything the user can configure before a scan.

use std::path::PathBuf;

/// What a scanned folder is used for when comparing in [`CompareMode::SourceVsTarget`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FolderRole {
    /// The "originals". Files here are normally kept.
    Source,
    /// Folders being cleaned up. Files here are duplicates if they also exist in a Source.
    Target,
}

impl FolderRole {
    pub fn label(self) -> &'static str {
        match self {
            FolderRole::Source => "Source",
            FolderRole::Target => "Target",
        }
    }

    pub fn toggled(self) -> Self {
        match self {
            FolderRole::Source => FolderRole::Target,
            FolderRole::Target => FolderRole::Source,
        }
    }
}

/// A folder selected for scanning.
#[derive(Clone, Debug)]
pub struct FolderEntry {
    pub path: PathBuf,
    pub role: FolderRole,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CompareMode {
    /// Every folder is compared against every other folder (and itself).
    AllTogether,
    /// Only report files in Target folders that already exist in Source folders.
    SourceVsTarget,
}

/// How two files are decided to be duplicates.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Method {
    FileName,
    NameAndSize,
    Size,
    Content,
    SimilarImage,
}

impl Method {
    /// All methods, in the order shown in the UI (most useful first).
    pub const ALL: [Method; 5] = [
        Method::Content,
        Method::SimilarImage,
        Method::FileName,
        Method::NameAndSize,
        Method::Size,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Method::FileName => "Same file name",
            Method::NameAndSize => "Same name + size",
            Method::Size => "Same size",
            Method::Content => "Identical content",
            Method::SimilarImage => "Similar-looking images",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Method::FileName => "Fastest. Matches by file name only; contents may differ.",
            Method::NameAndSize => "Very fast. Matches by name and exact byte size.",
            Method::Size => "Very fast, but many false positives. Good for a first look.",
            Method::Content => {
                "Exact byte-for-byte copies. Recommended. Uses size → quick hash → full hash, \
                 so only likely matches are read completely."
            }
            Method::SimilarImage => {
                "Finds the same picture saved in a different format, size or quality \
                 (perceptual hash). Videos are not supported by this method."
            }
        }
    }

    /// Name used on the command line (`--method <name>`).
    pub fn cli_name(self) -> &'static str {
        match self {
            Method::FileName => "name",
            Method::NameAndSize => "name-size",
            Method::Size => "size",
            Method::Content => "content",
            Method::SimilarImage => "similar",
        }
    }

    pub fn from_cli_name(s: &str) -> Option<Self> {
        Method::ALL.into_iter().find(|m| m.cli_name() == s)
    }
}

/// Full description of a scan. Cloned and moved to the worker thread when a scan starts.
#[derive(Clone, Debug)]
pub struct ScanConfig {
    pub folders: Vec<FolderEntry>,
    pub mode: CompareMode,
    pub method: Method,
    /// Name methods: `photo.jpg` and `photo.png` count as the same name.
    pub name_ignore_ext: bool,
    pub name_case_insensitive: bool,
    /// Skip the full-file hash and trust size + start/middle/end chunks (fast, tiny risk).
    pub content_partial_only: bool,
    /// Max Hamming distance (out of 128 bits) to consider two images similar.
    pub similarity_threshold: u32,
    /// Also match rotated / mirrored versions of an image.
    pub ignore_rotation: bool,
    /// Which [`FileKind`](super::FileKind)s to include, indexed by `FileKind::index`.
    pub kinds: [bool; 6],
    /// Minimum file size in bytes.
    pub min_size: u64,
    /// Maximum file size in bytes; 0 = unlimited.
    pub max_size: u64,
    pub include_hidden: bool,
    pub follow_symlinks: bool,
    pub recursive: bool,
    /// Comma-separated substrings; paths containing any of them are skipped.
    pub excludes: String,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            folders: Vec::new(),
            mode: CompareMode::AllTogether,
            method: Method::Content,
            name_ignore_ext: false,
            name_case_insensitive: true,
            content_partial_only: false,
            similarity_threshold: 10,
            ignore_rotation: false,
            kinds: [true; 6],
            min_size: 1,
            max_size: 0,
            include_hidden: false,
            follow_symlinks: false,
            recursive: true,
            excludes: String::new(),
        }
    }
}

impl ScanConfig {
    /// Adds `path` if it is a directory and not already in the list. The first folder
    /// becomes the Source, later ones Targets. Returns whether it was added.
    pub fn add_folder(&mut self, path: PathBuf) -> bool {
        let path = std::path::absolute(&path).unwrap_or(path);
        if !path.is_dir() || self.folders.iter().any(|f| f.path == path) {
            return false;
        }
        let role = if self.folders.is_empty() {
            FolderRole::Source
        } else {
            FolderRole::Target
        };
        self.folders.push(FolderEntry { path, role });
        true
    }

    /// Returns an error message if the configuration cannot be scanned.
    pub fn validate(&self) -> Result<(), String> {
        if self.folders.is_empty() {
            return Err("Add at least one folder to scan".into());
        }
        if self.mode == CompareMode::SourceVsTarget {
            let has = |r| self.folders.iter().any(|f| f.role == r);
            if !has(FolderRole::Source) || !has(FolderRole::Target) {
                return Err(
                    "Source vs Target mode needs at least one Source and one Target folder".into(),
                );
            }
        }
        if !self.kinds.iter().any(|k| *k) {
            return Err("Select at least one file type to scan".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn method_cli_names_round_trip() {
        for m in Method::ALL {
            assert_eq!(Method::from_cli_name(m.cli_name()), Some(m));
        }
        assert_eq!(Method::from_cli_name("nope"), None);
    }

    #[test]
    fn add_folder_assigns_roles_and_skips_duplicates_and_files() {
        let dir = std::env::temp_dir();
        let mut cfg = ScanConfig::default();
        assert!(cfg.add_folder(dir.clone()));
        assert!(!cfg.add_folder(dir.clone()), "same folder twice");
        assert!(!cfg.add_folder(dir.join("definitely/not/here/xyz")));
        assert_eq!(cfg.folders.len(), 1);
        assert_eq!(cfg.folders[0].role, FolderRole::Source);
        assert!(cfg.add_folder(std::env::current_dir().unwrap()));
        assert_eq!(cfg.folders[1].role, FolderRole::Target);
    }

    #[test]
    fn validate_catches_bad_configs() {
        let mut cfg = ScanConfig::default();
        assert!(cfg.validate().is_err());
        cfg.folders.push(FolderEntry {
            path: "/a".into(),
            role: FolderRole::Source,
        });
        assert!(cfg.validate().is_ok());
        cfg.mode = CompareMode::SourceVsTarget;
        assert!(cfg.validate().is_err(), "no target");
        cfg.folders.push(FolderEntry {
            path: "/b".into(),
            role: FolderRole::Target,
        });
        assert!(cfg.validate().is_ok());
        cfg.kinds = [false; 6];
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn role_toggle() {
        assert_eq!(FolderRole::Source.toggled(), FolderRole::Target);
        assert_eq!(FolderRole::Target.toggled(), FolderRole::Source);
    }
}
