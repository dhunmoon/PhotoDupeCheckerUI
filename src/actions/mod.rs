//! What to do with the files the user marked: trash, delete, move, copy or replace with links.
//!
//! Every action re-checks each file before touching it (`verify_unchanged`) so a file that
//! was modified after the scan (and may no longer be a duplicate) is never lost.

mod fs_ops;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::scan::Progress;

#[derive(Clone, Debug, PartialEq)]
pub enum ActionKind {
    Trash,
    Delete,
    MoveTo { dest: PathBuf, keep_structure: bool },
    CopyTo { dest: PathBuf, keep_structure: bool },
    HardLink,
    SymLink,
}

impl ActionKind {
    pub fn label(&self) -> String {
        match self {
            ActionKind::Trash => "Move to Trash / Recycle Bin".into(),
            ActionKind::Delete => "Delete permanently".into(),
            ActionKind::MoveTo { dest, .. } => format!("Move to {}", dest.display()),
            ActionKind::CopyTo { dest, .. } => format!("Copy to {}", dest.display()),
            ActionKind::HardLink => "Replace with hard links to the kept file".into(),
            ActionKind::SymLink => "Replace with symbolic links to the kept file".into(),
        }
    }

    /// Link actions need an unmarked file in the group to point at.
    pub fn needs_keeper(&self) -> bool {
        matches!(self, ActionKind::HardLink | ActionKind::SymLink)
    }

    /// Whether the file stops being a separate copy afterwards (and frees space).
    pub fn removes_original(&self) -> bool {
        !matches!(self, ActionKind::CopyTo { .. })
    }

    /// Whether nothing remains at the original path afterwards (so folders may become empty).
    pub fn vacates_path(&self) -> bool {
        matches!(
            self,
            ActionKind::Trash | ActionKind::Delete | ActionKind::MoveTo { .. }
        )
    }

    /// Cannot be undone from inside or outside the app.
    pub fn is_destructive(&self) -> bool {
        matches!(
            self,
            ActionKind::Delete | ActionKind::HardLink | ActionKind::SymLink
        )
    }
}

/// Extra behaviour chosen in the confirmation dialog.
#[derive(Clone, Copy, Debug, Default)]
pub struct ActionOptions {
    /// After removing/moving files, delete folders that became empty (up to the scan root).
    pub remove_empty_dirs: bool,
}

/// One file to act on, with the metadata captured at scan time.
#[derive(Clone, Debug)]
pub struct ActionItem {
    pub path: PathBuf,
    pub size: u64,
    pub modified: Option<SystemTime>,
    /// Scan root the file was found under (used to keep folder structure when moving).
    pub root: PathBuf,
    /// The file that stays in place (for link actions).
    pub keeper: Option<PathBuf>,
}

#[derive(Debug, Default)]
pub struct ActionReport {
    pub succeeded: Vec<PathBuf>,
    pub failed: Vec<(PathBuf, String)>,
    /// Bytes no longer used by duplicate copies.
    pub freed: u64,
    pub removed_dirs: usize,
}

/// Runs `kind` on every item, collecting per-file success/failure. Never panics on I/O errors.
pub fn execute(
    kind: &ActionKind,
    items: &[ActionItem],
    opts: ActionOptions,
    progress: &Progress,
) -> ActionReport {
    progress.set_stage(&kind.label(), items.len() as u64);
    let mut report = ActionReport::default();
    let mut touched_dirs: BTreeSet<(PathBuf, PathBuf)> = BTreeSet::new();
    for item in items {
        if progress.cancelled() {
            report.failed.push((item.path.clone(), "cancelled".into()));
            continue;
        }
        let r = verify_unchanged(item).and_then(|_| run_one(kind, item));
        progress.tick();
        match r {
            Ok(()) => {
                if kind.removes_original() {
                    report.freed += item.size;
                }
                if let Some(parent) = item.path.parent() {
                    touched_dirs.insert((parent.to_path_buf(), item.root.clone()));
                }
                report.succeeded.push(item.path.clone());
            }
            Err(e) => report.failed.push((item.path.clone(), e)),
        }
    }
    if opts.remove_empty_dirs && kind.vacates_path() {
        // Deepest first, so children are removed before their parents are checked.
        for (dir, root) in touched_dirs.iter().rev() {
            report.removed_dirs += fs_ops::remove_empty_parents(dir, root);
        }
    }
    report
}

/// Refuses to act on a file that changed since the scan (it may no longer be a duplicate).
fn verify_unchanged(item: &ActionItem) -> Result<(), String> {
    let meta = std::fs::symlink_metadata(&item.path).map_err(|e| e.to_string())?;
    if !meta.is_file() {
        return Err("not a regular file anymore".into());
    }
    if meta.len() != item.size || (item.modified.is_some() && meta.modified().ok() != item.modified)
    {
        return Err("file changed since the scan; skipped (re-scan to update)".into());
    }
    if let Some(k) = &item.keeper {
        if !k.is_file() {
            return Err(format!("kept file {} is missing", k.display()));
        }
        if fs_ops::same_file(k, &item.path) {
            return Err("kept file and duplicate are the same file".into());
        }
    }
    Ok(())
}

fn run_one(kind: &ActionKind, item: &ActionItem) -> Result<(), String> {
    let p: &Path = &item.path;
    match kind {
        ActionKind::Trash => trash::delete(p).map_err(|e| e.to_string()),
        ActionKind::Delete => std::fs::remove_file(p).map_err(|e| e.to_string()),
        ActionKind::MoveTo {
            dest,
            keep_structure,
        } => {
            let target = fs_ops::destination(dest, p, &item.root, *keep_structure)?;
            fs_ops::move_file(p, &target)
        }
        ActionKind::CopyTo {
            dest,
            keep_structure,
        } => {
            let target = fs_ops::destination(dest, p, &item.root, *keep_structure)?;
            std::fs::copy(p, &target)
                .map(|_| ())
                .map_err(|e| e.to_string())
        }
        ActionKind::HardLink => {
            let keeper = item.keeper.as_ref().ok_or("no kept file in group")?;
            fs_ops::replace_with(p, |tmp| std::fs::hard_link(keeper, tmp))
        }
        ActionKind::SymLink => {
            let keeper = item.keeper.as_ref().ok_or("no kept file in group")?;
            let keeper = std::fs::canonicalize(keeper).map_err(|e| e.to_string())?;
            fs_ops::replace_with(p, |tmp| fs_ops::make_symlink(&keeper, tmp))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::TempDir;

    fn item(p: &Path, root: &Path, keeper: Option<PathBuf>) -> ActionItem {
        let m = std::fs::metadata(p).unwrap();
        ActionItem {
            path: p.to_path_buf(),
            size: m.len(),
            modified: m.modified().ok(),
            root: root.to_path_buf(),
            keeper,
        }
    }

    fn run(kind: ActionKind, items: &[ActionItem]) -> ActionReport {
        execute(&kind, items, ActionOptions::default(), &Progress::default())
    }

    #[test]
    fn move_keeps_structure() {
        let d = TempDir::new("act_move");
        let root = d.path().join("photos");
        let a = d.write("photos/2020/trip/a.jpg", b"data");
        let q = d.path().join("quarantine");
        let r = run(
            ActionKind::MoveTo {
                dest: q.clone(),
                keep_structure: true,
            },
            &[item(&a, &root, None)],
        );
        assert!(r.failed.is_empty(), "{:?}", r.failed);
        assert_eq!(r.freed, 4);
        assert!(q.join("photos/2020/trip/a.jpg").is_file());
        assert!(!a.exists());
        assert!(
            root.join("2020/trip").is_dir(),
            "empty dirs kept by default"
        );
    }

    #[test]
    fn move_flat_never_overwrites() {
        let d = TempDir::new("act_move_flat");
        let root = d.path().join("r");
        let a = d.write("r/x/a.jpg", b"1");
        let b = d.write("r/y/a.jpg", b"2");
        let q = d.path().join("q");
        let kind = ActionKind::MoveTo {
            dest: q.clone(),
            keep_structure: false,
        };
        let r = run(kind, &[item(&a, &root, None), item(&b, &root, None)]);
        assert_eq!(r.succeeded.len(), 2);
        assert_eq!(std::fs::read(q.join("a.jpg")).unwrap(), b"1");
        assert_eq!(std::fs::read(q.join("a (1).jpg")).unwrap(), b"2");
    }

    #[test]
    fn copy_leaves_original_and_frees_nothing() {
        let d = TempDir::new("act_copy");
        let root = d.path().join("r");
        let a = d.write("r/a.txt", b"hello");
        let q = d.path().join("q");
        let r = run(
            ActionKind::CopyTo {
                dest: q.clone(),
                keep_structure: false,
            },
            &[item(&a, &root, None)],
        );
        assert_eq!(r.succeeded.len(), 1);
        assert_eq!(r.freed, 0);
        assert!(a.exists() && q.join("a.txt").exists());
    }

    #[test]
    fn delete_and_remove_empty_dirs_up_to_root() {
        let d = TempDir::new("act_delete");
        let root = d.path().join("r");
        let a = d.write("r/old/backup/a.txt", b"1");
        let b = d.write("r/old/b.txt", b"2");
        let keep = d.write("r/keep/c.txt", b"3");
        let opts = ActionOptions {
            remove_empty_dirs: true,
        };
        let r = execute(
            &ActionKind::Delete,
            &[item(&a, &root, None), item(&b, &root, None)],
            opts,
            &Progress::default(),
        );
        assert_eq!(r.succeeded.len(), 2);
        assert_eq!(r.removed_dirs, 2);
        assert!(!root.join("old").exists());
        assert!(keep.exists());
        assert!(root.is_dir(), "scan root is never removed");
    }

    #[test]
    fn links_need_and_point_to_keeper() {
        let d = TempDir::new("act_link");
        let root = d.path().to_path_buf();
        let keep = d.write("keep.jpg", b"data");
        let h = d.write("h.jpg", b"data");
        let s = d.write("s.jpg", b"data");

        let r = run(ActionKind::HardLink, &[item(&h, &root, Some(keep.clone()))]);
        assert!(r.failed.is_empty(), "{:?}", r.failed);
        assert_eq!(std::fs::read(&h).unwrap(), b"data");
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            assert_eq!(
                std::fs::metadata(&h).unwrap().ino(),
                std::fs::metadata(&keep).unwrap().ino()
            );
        }

        let r = run(ActionKind::SymLink, &[item(&s, &root, Some(keep.clone()))]);
        assert!(r.failed.is_empty(), "{:?}", r.failed);
        assert!(
            std::fs::symlink_metadata(&s)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(std::fs::read(&s).unwrap(), b"data");

        // Missing keeper / no keeper.
        let x = d.write("x.jpg", b"data");
        let r = run(
            ActionKind::HardLink,
            &[item(&x, &root, Some(d.path().join("gone")))],
        );
        assert!(r.failed[0].1.contains("missing"));
        let r = run(ActionKind::HardLink, &[item(&x, &root, None)]);
        assert!(r.failed[0].1.contains("no kept file"));
        // Keeper is the same file.
        let r = run(ActionKind::HardLink, &[item(&x, &root, Some(x.clone()))]);
        assert!(r.failed[0].1.contains("same file"));
        assert!(x.exists());
    }

    #[test]
    fn changed_or_missing_files_are_skipped() {
        let d = TempDir::new("act_stale");
        let root = d.path().to_path_buf();
        let b = d.write("b.jpg", b"data");
        let stale = item(&b, &root, None);
        std::fs::write(&b, b"changed!").unwrap();
        let r = run(ActionKind::Delete, &[stale]);
        assert_eq!(r.failed.len(), 1);
        assert!(r.failed[0].1.contains("changed"));
        assert!(b.exists());

        let gone = item(&b, &root, None);
        std::fs::remove_file(&b).unwrap();
        let r = run(ActionKind::Delete, &[gone]);
        assert_eq!(r.failed.len(), 1);
    }

    #[test]
    fn cancelled_progress_skips_everything() {
        let d = TempDir::new("act_cancel");
        let a = d.write("a", b"1");
        let p = Progress::default();
        p.request_cancel();
        let r = execute(
            &ActionKind::Delete,
            &[item(&a, d.path(), None)],
            ActionOptions::default(),
            &p,
        );
        assert_eq!(r.failed.len(), 1);
        assert!(a.exists());
    }

    #[test]
    fn kind_properties() {
        let mv = ActionKind::MoveTo {
            dest: "/q".into(),
            keep_structure: true,
        };
        let cp = ActionKind::CopyTo {
            dest: "/q".into(),
            keep_structure: true,
        };
        assert!(mv.removes_original() && mv.vacates_path() && !mv.is_destructive());
        assert!(!cp.removes_original() && !cp.vacates_path());
        assert!(ActionKind::Delete.is_destructive());
        assert!(ActionKind::HardLink.needs_keeper() && !ActionKind::HardLink.vacates_path());
        assert!(!ActionKind::Trash.is_destructive());
        assert!(mv.label().contains("/q"));
    }
}
