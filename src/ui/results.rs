//! Result state behind the UI: groups, marks, filtering and sorting.
//!
//! This has no egui dependency so it can be unit tested.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::actions::{ActionItem, ActionKind, ActionReport};
use crate::model::{CompareMode, DupGroup, FileEntry, FileKind, Method, ScanConfig};

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub enum GroupSort {
    #[default]
    Wasted,
    Count,
    FileSize,
    Name,
}

impl GroupSort {
    pub const ALL: [GroupSort; 4] = [
        GroupSort::Wasted,
        GroupSort::Count,
        GroupSort::FileSize,
        GroupSort::Name,
    ];

    pub fn label(self) -> &'static str {
        match self {
            GroupSort::Wasted => "Most space wasted",
            GroupSort::Count => "Most copies",
            GroupSort::FileSize => "Largest files",
            GroupSort::Name => "Name (A–Z)",
        }
    }
}

/// What subset of groups is visible, and in which order.
#[derive(Clone, PartialEq, Default, Debug)]
pub struct Filter {
    pub kind: Option<FileKind>,
    /// Case-insensitive substring of any path in the group.
    pub search: String,
    pub only_marked: bool,
    pub sort: GroupSort,
}

/// A row in the flat list view.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Row {
    Group(usize),
    File(usize, usize),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct MarkStats {
    pub files: usize,
    pub bytes: u64,
    /// Groups where every copy is marked (acting on them would lose the file entirely).
    pub fully_marked_groups: usize,
}

#[derive(Default)]
pub struct Results {
    pub groups: Vec<DupGroup>,
    /// Config that produced `groups` (folder roots are needed for actions).
    pub cfg: Option<ScanConfig>,
    /// Files marked for the next action.
    pub marked: HashSet<PathBuf>,
    pub filter: Filter,
    visible: Vec<usize>,
    rows: Vec<Row>,
    dirty: bool,
}

impl Results {
    /// Replaces everything with a fresh scan result.
    pub fn set(&mut self, groups: Vec<DupGroup>, cfg: ScanConfig) {
        self.groups = groups;
        self.cfg = Some(cfg);
        self.marked.clear();
        self.dirty = true;
    }

    pub fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Recomputes the visible groups if anything changed since the last call.
    pub fn refresh(&mut self) {
        if self.dirty {
            self.rebuild();
        }
    }

    /// Indices into `groups` of the groups that pass the filter, in display order.
    pub fn visible(&self) -> &[usize] {
        &self.visible
    }

    /// Flattened group-header + file rows for the list view.
    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    pub fn is_source_vs_target(&self) -> bool {
        self.cfg
            .as_ref()
            .is_some_and(|c| c.mode == CompareMode::SourceVsTarget)
    }

    pub fn method(&self) -> Option<Method> {
        self.cfg.as_ref().map(|c| c.method)
    }

    pub fn is_marked(&self, p: &Path) -> bool {
        self.marked.contains(p)
    }

    pub fn set_marked(&mut self, p: &Path, on: bool) {
        if on {
            self.marked.insert(p.to_path_buf());
        } else {
            self.marked.remove(p);
        }
    }

    pub fn toggle(&mut self, p: &Path) {
        let on = !self.is_marked(p);
        self.set_marked(p, on);
    }

    pub fn marked_in_group(&self, g: &DupGroup) -> usize {
        g.files.iter().filter(|f| self.is_marked(&f.path)).count()
    }

    /// `(groups, files, reclaimable bytes)` over all groups.
    pub fn totals(&self) -> (usize, usize, u64) {
        (
            self.groups.len(),
            self.groups.iter().map(|g| g.files.len()).sum(),
            self.groups.iter().map(|g| g.wasted()).sum(),
        )
    }

    pub fn stats(&self) -> MarkStats {
        let mut s = MarkStats::default();
        for g in &self.groups {
            let mut in_group = 0;
            for f in g.files.iter().filter(|f| self.is_marked(&f.path)) {
                s.files += 1;
                s.bytes += f.size;
                in_group += 1;
            }
            if in_group == g.files.len() {
                s.fully_marked_groups += 1;
            }
        }
        s
    }

    pub fn marked_paths_sorted(&self) -> Vec<String> {
        let mut v: Vec<String> = self
            .marked
            .iter()
            .map(|p| p.display().to_string())
            .collect();
        v.sort();
        v
    }

    /// Turns the marks into action items. Groups where nothing would be kept are skipped
    /// when `skip_full` is set or the action needs a keeper; returns `(items, skipped)`.
    pub fn build_items(&self, kind: &ActionKind, skip_full: bool) -> (Vec<ActionItem>, usize) {
        let Some(cfg) = &self.cfg else {
            return (Vec::new(), 0);
        };
        let mut items = Vec::new();
        let mut skipped = 0;
        for g in &self.groups {
            let sel: Vec<&FileEntry> = g.files.iter().filter(|f| self.is_marked(&f.path)).collect();
            if sel.is_empty() {
                continue;
            }
            let keeper = g
                .files
                .iter()
                .find(|f| !self.is_marked(&f.path))
                .map(|f| f.path.clone());
            if keeper.is_none() && (skip_full || kind.needs_keeper()) {
                skipped += 1;
                continue;
            }
            for f in sel {
                items.push(ActionItem {
                    path: f.path.clone(),
                    size: f.size,
                    modified: f.modified,
                    root: cfg
                        .folders
                        .get(f.folder)
                        .map(|x| x.path.clone())
                        .unwrap_or_default(),
                    keeper: keeper.clone(),
                });
            }
        }
        (items, skipped)
    }

    /// Updates groups and marks after an action finished.
    pub fn apply_report(&mut self, kind: &ActionKind, report: &ActionReport) {
        let done: HashSet<&PathBuf> = report.succeeded.iter().collect();
        for p in &done {
            self.marked.remove(*p);
        }
        // Links/moves/deletes all mean the file no longer wastes space as a duplicate.
        if kind.removes_original() {
            for g in &mut self.groups {
                g.files.retain(|f| !done.contains(&f.path));
            }
            self.groups.retain(|g| g.files.len() > 1);
        }
        self.dirty = true;
    }

    fn rebuild(&mut self) {
        let search = self.filter.search.to_lowercase();
        let f = &self.filter;
        let mut visible: Vec<usize> = self
            .groups
            .iter()
            .enumerate()
            .filter(|(_, g)| f.kind.is_none_or(|k| g.files.iter().any(|x| x.kind == k)))
            .filter(|(_, g)| {
                search.is_empty()
                    || g.files
                        .iter()
                        .any(|x| x.path.to_string_lossy().to_lowercase().contains(&search))
            })
            .filter(|(_, g)| {
                !f.only_marked || g.files.iter().any(|x| self.marked.contains(&x.path))
            })
            .map(|(i, _)| i)
            .collect();
        let g = &self.groups;
        match f.sort {
            GroupSort::Wasted => visible.sort_by_key(|&i| std::cmp::Reverse(g[i].wasted())),
            GroupSort::Count => visible.sort_by_key(|&i| std::cmp::Reverse(g[i].files.len())),
            GroupSort::FileSize => visible.sort_by_key(|&i| {
                std::cmp::Reverse(g[i].files.iter().map(|f| f.size).max().unwrap_or(0))
            }),
            GroupSort::Name => visible.sort_by_cached_key(|&i| g[i].files[0].name().to_lowercase()),
        }
        self.rows.clear();
        for &gi in &visible {
            self.rows.push(Row::Group(gi));
            self.rows
                .extend((0..self.groups[gi].files.len()).map(|fi| Row::File(gi, fi)));
        }
        self.visible = visible;
        self.dirty = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{FolderEntry, FolderRole};

    fn results() -> Results {
        let mk = |id, files: Vec<FileEntry>| DupGroup {
            id,
            label: String::new(),
            files,
        };
        let mut a = FileEntry::new("/r/photos/z.jpg", 100);
        a.folder = 0;
        let mut r = Results::default();
        r.set(
            vec![
                mk(1, vec![a, FileEntry::new("/r/backup/z.jpg", 100)]),
                mk(
                    2,
                    vec![
                        FileEntry::new("/r/a.txt", 10),
                        FileEntry::new("/r/b.txt", 10),
                        FileEntry::new("/r/c.txt", 10),
                    ],
                ),
            ],
            ScanConfig {
                folders: vec![FolderEntry {
                    path: "/r".into(),
                    role: FolderRole::Source,
                }],
                ..Default::default()
            },
        );
        r.refresh();
        r
    }

    #[test]
    fn rows_and_sorting() {
        let mut r = results();
        assert_eq!(r.visible(), &[0, 1], "100 wasted > 20 wasted");
        assert_eq!(r.rows().len(), 2 + 2 + 3, "2 headers + 5 files");
        assert_eq!(r.rows()[0], Row::Group(0));
        assert_eq!(r.rows()[1], Row::File(0, 0));
        r.filter.sort = GroupSort::Count;
        r.mark_dirty();
        r.refresh();
        assert_eq!(r.visible(), &[1, 0]);
        r.filter.sort = GroupSort::Name;
        r.mark_dirty();
        r.refresh();
        assert_eq!(r.visible(), &[1, 0], "a.txt < z.jpg");
        assert_eq!(r.totals(), (2, 5, 120));
    }

    #[test]
    fn filters() {
        let mut r = results();
        r.filter.kind = Some(FileKind::Image);
        r.mark_dirty();
        r.refresh();
        assert_eq!(r.visible(), &[0]);
        r.filter = Filter {
            search: "B.TXT".into(),
            ..Default::default()
        };
        r.mark_dirty();
        r.refresh();
        assert_eq!(r.visible(), &[1]);
        r.filter = Filter {
            only_marked: true,
            ..Default::default()
        };
        r.mark_dirty();
        r.refresh();
        assert!(r.visible().is_empty());
        r.toggle(Path::new("/r/a.txt"));
        r.mark_dirty();
        r.refresh();
        assert_eq!(r.visible(), &[1]);
    }

    #[test]
    fn marking_and_stats() {
        let mut r = results();
        r.toggle(Path::new("/r/backup/z.jpg"));
        r.set_marked(Path::new("/r/a.txt"), true);
        assert!(r.is_marked(Path::new("/r/a.txt")));
        assert_eq!(
            r.stats(),
            MarkStats {
                files: 2,
                bytes: 110,
                fully_marked_groups: 0
            }
        );
        r.toggle(Path::new("/r/photos/z.jpg"));
        assert_eq!(r.stats().fully_marked_groups, 1);
        assert_eq!(r.marked_in_group(&r.groups[0]), 2);
        assert_eq!(r.marked_paths_sorted()[0], "/r/a.txt");
    }

    #[test]
    fn build_items_protects_last_copy() {
        let mut r = results();
        for p in ["/r/photos/z.jpg", "/r/backup/z.jpg", "/r/b.txt"] {
            r.set_marked(Path::new(p), true);
        }
        let (items, skipped) = r.build_items(&ActionKind::Trash, true);
        assert_eq!(skipped, 1);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].keeper.as_deref(), Some(Path::new("/r/a.txt")));
        assert_eq!(items[0].root, PathBuf::from("/r"));

        let (items, skipped) = r.build_items(&ActionKind::Trash, false);
        assert_eq!((items.len(), skipped), (3, 0));
        let (_, skipped) = r.build_items(&ActionKind::HardLink, false);
        assert_eq!(skipped, 1, "links always need a keeper");
    }

    #[test]
    fn apply_report_prunes_groups() {
        let mut r = results();
        r.set_marked(Path::new("/r/backup/z.jpg"), true);
        r.set_marked(Path::new("/r/c.txt"), true);
        let report = ActionReport {
            succeeded: vec!["/r/backup/z.jpg".into(), "/r/c.txt".into()],
            ..Default::default()
        };
        let copy = ActionKind::CopyTo {
            dest: "/q".into(),
            keep_structure: false,
        };
        r.apply_report(&copy, &report);
        assert_eq!(r.totals().1, 5, "copies don't change groups");
        assert!(r.marked.is_empty(), "but succeeded files are unmarked");

        r.apply_report(&ActionKind::Trash, &report);
        r.refresh();
        assert_eq!(r.groups.len(), 1, "image group dropped to one file");
        assert_eq!(r.groups[0].files.len(), 2);
        assert_eq!(r.visible(), &[0]);
    }
}
