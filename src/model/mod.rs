//! Plain data types shared by the scanner, the actions and the UI.
//!
//! Nothing in here does I/O except [`ScanConfig::add_folder`] (which checks the folder exists).

mod config;
mod entry;
mod format;
mod kind;

pub use config::{CompareMode, FolderEntry, FolderRole, Method, ScanConfig};
pub use entry::{DupGroup, FileEntry};
pub use format::{format_time, human_size};
pub use kind::{FileKind, extension_lower, is_decodable_image};
