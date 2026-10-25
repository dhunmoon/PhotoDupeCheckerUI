//! "Smart select" rules.
//!
//! The selection is a `HashSet<PathBuf>` of *marked* files: the ones the next action
//! (trash, move, link, …) will be applied to. Unmarked files are kept.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::model::{DupGroup, FileEntry, FolderRole, extension_lower};

/// Which file to keep in each group; every other file in the group gets marked.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum KeepRule {
    Newest,
    Oldest,
    Largest,
    Smallest,
    HighestResolution,
    ShortestPath,
    LongestPath,
    ShortestName,
    SourceFolders,
    First,
}

impl KeepRule {
    pub const ALL: [KeepRule; 10] = [
        KeepRule::SourceFolders,
        KeepRule::HighestResolution,
        KeepRule::Largest,
        KeepRule::Smallest,
        KeepRule::Newest,
        KeepRule::Oldest,
        KeepRule::ShortestPath,
        KeepRule::LongestPath,
        KeepRule::ShortestName,
        KeepRule::First,
    ];

    /// Whether this rule only makes sense for a Source-vs-Target scan.
    pub fn needs_source_folders(self) -> bool {
        self == KeepRule::SourceFolders
    }

    pub fn label(self) -> &'static str {
        match self {
            KeepRule::Newest => "Keep newest (modified date)",
            KeepRule::Oldest => "Keep oldest (modified date)",
            KeepRule::Largest => "Keep largest file",
            KeepRule::Smallest => "Keep smallest file",
            KeepRule::HighestResolution => "Keep highest resolution (images)",
            KeepRule::ShortestPath => "Keep shortest path",
            KeepRule::LongestPath => "Keep longest path",
            KeepRule::ShortestName => "Keep shortest file name",
            KeepRule::SourceFolders => "Keep files in Source folders",
            KeepRule::First => "Keep first file in each group",
        }
    }
}

fn path_len(f: &FileEntry) -> usize {
    f.path.as_os_str().len()
}

/// Marks every file in each group except the one chosen by `rule`.
///
/// [`KeepRule::SourceFolders`] keeps *all* Source files and marks all Target files, and
/// leaves groups without a Source file untouched.
pub fn apply_keep_rule(groups: &[DupGroup], selected: &mut HashSet<PathBuf>, rule: KeepRule) {
    for g in groups {
        if g.files.is_empty() {
            continue;
        }
        if rule == KeepRule::SourceFolders {
            if !g.files.iter().any(|f| f.role == FolderRole::Source) {
                continue;
            }
            for f in &g.files {
                set(selected, &f.path, f.role == FolderRole::Target);
            }
            continue;
        }
        // `min_by_key` returns the *first* best element, so on ties the earliest file wins
        // (groups list Source files first). `max_by_key` would return the last one.
        use std::cmp::Reverse;
        let it = g.files.iter().enumerate();
        let keep = match rule {
            KeepRule::Newest => it.min_by_key(|(_, f)| Reverse(f.modified)).map(|(i, _)| i),
            KeepRule::Oldest => it.min_by_key(|(_, f)| f.modified).map(|(i, _)| i),
            KeepRule::Largest => it.min_by_key(|(_, f)| Reverse(f.size)).map(|(i, _)| i),
            KeepRule::Smallest => it.min_by_key(|(_, f)| f.size).map(|(i, _)| i),
            KeepRule::HighestResolution => it
                .min_by_key(|(_, f)| Reverse((f.pixels(), f.size)))
                .map(|(i, _)| i),
            KeepRule::ShortestPath => it.min_by_key(|(_, f)| path_len(f)).map(|(i, _)| i),
            KeepRule::LongestPath => it.min_by_key(|(_, f)| Reverse(path_len(f))).map(|(i, _)| i),
            KeepRule::ShortestName => it.min_by_key(|(_, f)| f.name().len()).map(|(i, _)| i),
            KeepRule::First | KeepRule::SourceFolders => Some(0),
        }
        .unwrap_or(0);
        for (i, f) in g.files.iter().enumerate() {
            set(selected, &f.path, i != keep);
        }
    }
}

/// Keep files inside `folder`; select the others, but only in groups that have at least
/// one file inside `folder` (so nothing is lost).
pub fn keep_in_folder(groups: &[DupGroup], selected: &mut HashSet<PathBuf>, folder: &Path) {
    for g in groups {
        if g.files.iter().any(|f| f.path.starts_with(folder)) {
            for f in &g.files {
                set(selected, &f.path, !f.path.starts_with(folder));
            }
        }
    }
}

/// Marks every file inside `folder`.
pub fn select_in_folder(groups: &[DupGroup], selected: &mut HashSet<PathBuf>, folder: &Path) {
    for f in groups.iter().flat_map(|g| &g.files) {
        if f.path.starts_with(folder) {
            selected.insert(f.path.clone());
        }
    }
}

/// Marks files with one of the given extensions. `exts` is a comma separated list,
/// e.g. `"png, .BMP"` (dots and case are ignored).
pub fn select_by_extension(groups: &[DupGroup], selected: &mut HashSet<PathBuf>, exts: &str) {
    let exts: Vec<String> = exts
        .split(',')
        .map(|e| e.trim().trim_start_matches('.').to_ascii_lowercase())
        .filter(|e| !e.is_empty())
        .collect();
    for f in groups.iter().flat_map(|g| &g.files) {
        if exts.contains(&extension_lower(&f.path)) {
            selected.insert(f.path.clone());
        }
    }
}

/// Marks every file whose full path contains `text` (case-insensitive).
pub fn select_by_text(groups: &[DupGroup], selected: &mut HashSet<PathBuf>, text: &str) {
    let text = text.to_lowercase();
    if text.is_empty() {
        return;
    }
    for f in groups.iter().flat_map(|g| &g.files) {
        if f.path.to_string_lossy().to_lowercase().contains(&text) {
            selected.insert(f.path.clone());
        }
    }
}

/// Flips the mark of every file in every group.
pub fn invert(groups: &[DupGroup], selected: &mut HashSet<PathBuf>) {
    for f in groups.iter().flat_map(|g| &g.files) {
        let s = selected.contains(&f.path);
        set(selected, &f.path, !s);
    }
}

/// Marks every file in `group` except `keep`.
pub fn keep_only(group: &DupGroup, selected: &mut HashSet<PathBuf>, keep: &Path) {
    for f in &group.files {
        set(selected, &f.path, f.path != keep);
    }
}

fn set(selected: &mut HashSet<PathBuf>, p: &Path, on: bool) {
    if on {
        selected.insert(p.to_path_buf());
    } else {
        selected.remove(p);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, SystemTime};

    fn file(
        path: &str,
        size: u64,
        age_days: u64,
        dims: Option<(u32, u32)>,
        role: FolderRole,
    ) -> FileEntry {
        let mut f = FileEntry::new(path, size);
        f.modified =
            Some(SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000_000 - age_days * 86_400));
        f.dims = dims;
        f.role = role;
        f
    }

    fn groups() -> Vec<DupGroup> {
        use FolderRole::*;
        vec![
            DupGroup {
                id: 1,
                label: String::new(),
                files: vec![
                    file("/src/photos/a.jpg", 300, 10, Some((100, 100)), Source),
                    file(
                        "/dst/backup/long/path/a_copy.png",
                        900,
                        1,
                        Some((400, 300)),
                        Target,
                    ),
                    file("/dst/b.jpg", 100, 30, Some((50, 50)), Target),
                ],
            },
            DupGroup {
                id: 2,
                label: String::new(),
                files: vec![
                    file("/dst/x.txt", 5, 2, None, Target),
                    file("/dst/old/x.txt", 5, 3, None, Target),
                ],
            },
        ]
    }

    fn marked(sel: &HashSet<PathBuf>) -> Vec<String> {
        let mut v: Vec<String> = sel.iter().map(|p| p.display().to_string()).collect();
        v.sort();
        v
    }

    fn kept(g: &[DupGroup], rule: KeepRule) -> Vec<String> {
        let mut sel = HashSet::new();
        apply_keep_rule(g, &mut sel, rule);
        g.iter()
            .flat_map(|g| &g.files)
            .filter(|f| !sel.contains(&f.path))
            .map(|f| f.path.display().to_string())
            .collect()
    }

    #[test]
    fn keep_rules_keep_exactly_one_per_group() {
        let g = groups();
        assert_eq!(
            kept(&g, KeepRule::Newest),
            ["/dst/backup/long/path/a_copy.png", "/dst/x.txt"]
        );
        assert_eq!(kept(&g, KeepRule::Oldest), ["/dst/b.jpg", "/dst/old/x.txt"]);
        assert_eq!(
            kept(&g, KeepRule::Largest)[0],
            "/dst/backup/long/path/a_copy.png"
        );
        assert_eq!(kept(&g, KeepRule::Smallest)[0], "/dst/b.jpg");
        assert_eq!(
            kept(&g, KeepRule::HighestResolution)[0],
            "/dst/backup/long/path/a_copy.png"
        );
        assert_eq!(
            kept(&g, KeepRule::ShortestPath),
            ["/dst/b.jpg", "/dst/x.txt"]
        );
        assert_eq!(
            kept(&g, KeepRule::LongestPath),
            ["/dst/backup/long/path/a_copy.png", "/dst/old/x.txt"]
        );
        assert_eq!(kept(&g, KeepRule::ShortestName)[0], "/src/photos/a.jpg");
        assert_eq!(
            kept(&g, KeepRule::First),
            ["/src/photos/a.jpg", "/dst/x.txt"]
        );
    }

    #[test]
    fn ties_keep_the_first_file() {
        // Exact duplicates: same size, same resolution, same date. Groups list the Source
        // copy first, so every "max" rule must keep index 0, not the last backup copy.
        let files: Vec<FileEntry> = ["/src/a.jpg", "/dst/1/a.jpg", "/dst/2/a.jpg"]
            .iter()
            .map(|p| file(p, 500, 5, Some((10, 10)), FolderRole::Source))
            .collect();
        let g = vec![DupGroup {
            id: 1,
            label: String::new(),
            files,
        }];
        for rule in [
            KeepRule::Newest,
            KeepRule::Oldest,
            KeepRule::Largest,
            KeepRule::Smallest,
            KeepRule::HighestResolution,
        ] {
            assert_eq!(kept(&g, rule), ["/src/a.jpg"], "{rule:?}");
        }
    }

    #[test]
    fn source_rule_marks_targets_only_where_a_source_exists() {
        let g = groups();
        let mut sel = HashSet::new();
        apply_keep_rule(&g, &mut sel, KeepRule::SourceFolders);
        assert_eq!(
            marked(&sel),
            ["/dst/b.jpg", "/dst/backup/long/path/a_copy.png"]
        );
        assert!(KeepRule::SourceFolders.needs_source_folders());
        assert!(!KeepRule::Newest.needs_source_folders());
    }

    #[test]
    fn rules_replace_previous_marks_within_groups() {
        let g = groups();
        let mut sel = HashSet::new();
        invert(&g, &mut sel);
        assert_eq!(sel.len(), 5);
        apply_keep_rule(&g, &mut sel, KeepRule::First);
        assert_eq!(sel.len(), 3);
        invert(&g, &mut sel);
        assert_eq!(marked(&sel), ["/dst/x.txt", "/src/photos/a.jpg"]);
    }

    #[test]
    fn folder_rules() {
        let g = groups();
        let mut sel = HashSet::new();
        keep_in_folder(&g, &mut sel, Path::new("/src"));
        assert_eq!(
            marked(&sel),
            ["/dst/b.jpg", "/dst/backup/long/path/a_copy.png"],
            "group 2 untouched"
        );

        let mut sel = HashSet::new();
        select_in_folder(&g, &mut sel, Path::new("/dst/old"));
        assert_eq!(marked(&sel), ["/dst/old/x.txt"]);
        // Path::starts_with is component-wise: "/dst/b" must not match "/dst/backup".
        let mut sel = HashSet::new();
        select_in_folder(&g, &mut sel, Path::new("/dst/b"));
        assert!(sel.is_empty());
    }

    #[test]
    fn extension_and_text_rules() {
        let g = groups();
        let mut sel = HashSet::new();
        select_by_extension(&g, &mut sel, " .PNG, txt ,");
        assert_eq!(
            marked(&sel),
            [
                "/dst/backup/long/path/a_copy.png",
                "/dst/old/x.txt",
                "/dst/x.txt"
            ]
        );

        let mut sel = HashSet::new();
        select_by_text(&g, &mut sel, "BACKUP");
        assert_eq!(marked(&sel), ["/dst/backup/long/path/a_copy.png"]);
        select_by_text(&g, &mut sel, "");
        assert_eq!(sel.len(), 1);
    }

    #[test]
    fn keep_only_one() {
        let g = groups();
        let mut sel = HashSet::new();
        keep_only(&g[0], &mut sel, Path::new("/dst/b.jpg"));
        assert_eq!(
            marked(&sel),
            ["/dst/backup/long/path/a_copy.png", "/src/photos/a.jpg"]
        );
    }
}
