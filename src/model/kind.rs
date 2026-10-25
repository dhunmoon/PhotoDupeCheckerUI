//! File categories, derived from the file extension.

use std::path::Path;

/// Broad category of a file. Used for filtering the scan and for icons in the UI.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FileKind {
    Image,
    Video,
    Audio,
    Document,
    Archive,
    Other,
}

const IMAGE_EXT: &[&str] = &[
    "jpg", "jpeg", "jpe", "jfif", "png", "gif", "bmp", "tif", "tiff", "webp", "ico", "tga", "pnm",
    "pbm", "pgm", "ppm", "qoi", "hdr", "exr", "dds", "ff", "avif", "heic", "heif", "raw", "cr2",
    "nef", "arw", "dng", "orf", "rw2", "svg",
];

/// Subset of [`IMAGE_EXT`] the `image` crate can decode. Only these get thumbnails and
/// take part in visual-similarity matching; the rest are still found by the other methods.
const DECODABLE_IMAGE_EXT: &[&str] = &[
    "jpg", "jpeg", "jpe", "jfif", "png", "gif", "bmp", "tif", "tiff", "webp", "ico", "tga", "pnm",
    "pbm", "pgm", "ppm", "qoi", "hdr", "exr", "dds", "ff",
];

const VIDEO_EXT: &[&str] = &[
    "mp4", "m4v", "mov", "avi", "mkv", "webm", "wmv", "flv", "mpg", "mpeg", "3gp", "3g2", "ts",
    "mts", "m2ts", "vob", "ogv", "f4v", "asf",
];

const AUDIO_EXT: &[&str] = &[
    "mp3", "wav", "flac", "aac", "ogg", "oga", "m4a", "wma", "opus", "aiff", "aif", "alac",
];

const DOC_EXT: &[&str] = &[
    "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "odt", "ods", "odp", "txt", "rtf", "md",
    "csv", "epub", "mobi", "pages", "numbers", "key",
];

const ARCHIVE_EXT: &[&str] = &[
    "zip", "rar", "7z", "tar", "gz", "bz2", "xz", "tgz", "zst", "iso", "dmg", "cab",
];

/// Lower-cased extension without the dot, or an empty string if there is none.
pub fn extension_lower(path: &Path) -> String {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default()
}

/// Whether the file can be decoded for previews / perceptual hashing.
pub fn is_decodable_image(path: &Path) -> bool {
    DECODABLE_IMAGE_EXT.contains(&extension_lower(path).as_str())
}

impl FileKind {
    /// All kinds, in the order used by [`FileKind::index`] and the UI.
    pub const ALL: [FileKind; 6] = [
        FileKind::Image,
        FileKind::Video,
        FileKind::Audio,
        FileKind::Document,
        FileKind::Archive,
        FileKind::Other,
    ];

    pub fn from_path(path: &Path) -> Self {
        let ext = extension_lower(path);
        let ext = ext.as_str();
        if IMAGE_EXT.contains(&ext) {
            FileKind::Image
        } else if VIDEO_EXT.contains(&ext) {
            FileKind::Video
        } else if AUDIO_EXT.contains(&ext) {
            FileKind::Audio
        } else if DOC_EXT.contains(&ext) {
            FileKind::Document
        } else if ARCHIVE_EXT.contains(&ext) {
            FileKind::Archive
        } else {
            FileKind::Other
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            FileKind::Image => "Images",
            FileKind::Video => "Videos",
            FileKind::Audio => "Audio",
            FileKind::Document => "Documents",
            FileKind::Archive => "Archives",
            FileKind::Other => "Other",
        }
    }

    /// Position in [`FileKind::ALL`]; used to index `ScanConfig::kinds`.
    pub fn index(self) -> usize {
        FileKind::ALL.iter().position(|k| *k == self).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_by_extension_case_insensitively() {
        assert_eq!(FileKind::from_path(Path::new("a/B.JPG")), FileKind::Image);
        assert_eq!(FileKind::from_path(Path::new("x.HeIc")), FileKind::Image);
        assert_eq!(FileKind::from_path(Path::new("clip.mkv")), FileKind::Video);
        assert_eq!(FileKind::from_path(Path::new("song.flac")), FileKind::Audio);
        assert_eq!(FileKind::from_path(Path::new("cv.pdf")), FileKind::Document);
        assert_eq!(FileKind::from_path(Path::new("b.tar")), FileKind::Archive);
        assert_eq!(FileKind::from_path(Path::new("Makefile")), FileKind::Other);
        assert_eq!(FileKind::from_path(Path::new("weird.xyz")), FileKind::Other);
    }

    #[test]
    fn decodable_is_a_subset_of_images() {
        for ext in DECODABLE_IMAGE_EXT {
            assert!(IMAGE_EXT.contains(ext), "{ext} missing from IMAGE_EXT");
        }
        assert!(is_decodable_image(Path::new("a.PNG")));
        assert!(!is_decodable_image(Path::new("a.heic")));
        assert!(!is_decodable_image(Path::new("a.mp4")));
    }

    #[test]
    fn extension_lower_handles_missing_and_dotfiles() {
        assert_eq!(extension_lower(Path::new("a.TxT")), "txt");
        assert_eq!(extension_lower(Path::new("noext")), "");
        assert_eq!(extension_lower(Path::new(".bashrc")), "");
        assert_eq!(extension_lower(Path::new("archive.tar.GZ")), "gz");
    }

    #[test]
    fn index_matches_all_order() {
        for (i, k) in FileKind::ALL.iter().enumerate() {
            assert_eq!(k.index(), i);
        }
    }
}
