//! Low-level, safety-focused file system operations used by the actions.

use std::path::{Path, PathBuf};

pub(crate) fn same_file(a: &Path, b: &Path) -> bool {
    match (std::fs::canonicalize(a), std::fs::canonicalize(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}

#[cfg(unix)]
pub(crate) fn make_symlink(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
pub(crate) fn make_symlink(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_file(target, link)
}

/// Creates the replacement next to `p` under a temp name, then atomically renames it over
/// `p`, so the original is never lost if creating the replacement fails.
pub(crate) fn replace_with(
    p: &Path,
    create: impl FnOnce(&Path) -> std::io::Result<()>,
) -> Result<(), String> {
    let name = p.file_name().ok_or("bad file name")?.to_string_lossy();
    let tmp = p.with_file_name(format!(".{name}.dupe_link_tmp"));
    let _ = std::fs::remove_file(&tmp);
    create(&tmp).map_err(|e| format!("could not create link: {e}"))?;
    if let Err(e) = std::fs::rename(&tmp, p) {
        let _ = std::fs::remove_file(&tmp);
        return Err(e.to_string());
    }
    Ok(())
}

/// Where a moved/copied file should go. With `keep_structure` the path relative to its
/// scan `root` is recreated under `dest/<root name>/`. Never overwrites: see [`unique_path`].
pub(crate) fn destination(
    dest: &Path,
    file: &Path,
    root: &Path,
    keep_structure: bool,
) -> Result<PathBuf, String> {
    let file_name = file.file_name().ok_or("bad file name")?;
    let dir = if keep_structure {
        let root_name = root
            .file_name()
            .map(|n| n.to_os_string())
            .unwrap_or_else(|| "root".into());
        let rel_parent = file
            .parent()
            .and_then(|p| p.strip_prefix(root).ok())
            .unwrap_or(Path::new(""));
        dest.join(root_name).join(rel_parent)
    } else {
        dest.to_path_buf()
    };
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(unique_path(&dir.join(file_name)))
}

/// `p` if it is free, otherwise `name (1).ext`, `name (2).ext`, …
pub(crate) fn unique_path(p: &Path) -> PathBuf {
    if !p.exists() {
        return p.to_path_buf();
    }
    let stem = p.file_stem().unwrap_or_default().to_string_lossy();
    let ext = p
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    (1..)
        .map(|i| p.with_file_name(format!("{stem} ({i}){ext}")))
        .find(|c| !c.exists())
        .unwrap()
}

/// Rename, falling back to copy + verify + delete when moving across drives.
pub(crate) fn move_file(from: &Path, to: &Path) -> Result<(), String> {
    if std::fs::rename(from, to).is_ok() {
        return Ok(());
    }
    let copied = std::fs::copy(from, to).map_err(|e| e.to_string())?;
    let orig = std::fs::metadata(from).map_err(|e| e.to_string())?.len();
    if copied != orig {
        let _ = std::fs::remove_file(to);
        return Err("copy verification failed".into());
    }
    std::fs::remove_file(from).map_err(|e| e.to_string())
}

/// Removes `dir` and its ancestors while they are empty, stopping at (and never removing)
/// `stop_at`. Returns how many folders were removed. Does nothing if `dir` is not inside
/// `stop_at`.
pub(crate) fn remove_empty_parents(dir: &Path, stop_at: &Path) -> usize {
    let mut removed = 0;
    let mut cur = dir;
    while cur != stop_at && cur.starts_with(stop_at) {
        // remove_dir only succeeds on empty directories, which is exactly what we want.
        if std::fs::remove_dir(cur).is_err() {
            break;
        }
        removed += 1;
        match cur.parent() {
            Some(p) => cur = p,
            None => break,
        }
    }
    removed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::TempDir;

    #[test]
    fn unique_path_appends_counter() {
        let d = TempDir::new("unique");
        let p = d.path().join("a.jpg");
        assert_eq!(unique_path(&p), p);
        d.write("a.jpg", b"1");
        assert_eq!(unique_path(&p), d.path().join("a (1).jpg"));
        d.write("a (1).jpg", b"1");
        assert_eq!(unique_path(&p), d.path().join("a (2).jpg"));
        d.write("noext", b"1");
        assert_eq!(
            unique_path(&d.path().join("noext")),
            d.path().join("noext (1)")
        );
    }

    #[test]
    fn destination_with_and_without_structure() {
        let d = TempDir::new("dest");
        let root = Path::new("/photos");
        let file = Path::new("/photos/2020/trip/a.jpg");
        let q = d.path().join("q");
        assert_eq!(
            destination(&q, file, root, true).unwrap(),
            q.join("photos/2020/trip/a.jpg")
        );
        assert!(q.join("photos/2020/trip").is_dir());
        assert_eq!(destination(&q, file, root, false).unwrap(), q.join("a.jpg"));
    }

    #[test]
    fn replace_with_keeps_original_on_failure() {
        let d = TempDir::new("replace");
        let p = d.write("a.txt", b"original");
        let r = replace_with(&p, |_| Err(std::io::Error::other("boom")));
        assert!(r.unwrap_err().contains("boom"));
        assert_eq!(std::fs::read(&p).unwrap(), b"original");
        replace_with(&p, |tmp| std::fs::write(tmp, b"new")).unwrap();
        assert_eq!(std::fs::read(&p).unwrap(), b"new");
        assert_eq!(
            std::fs::read_dir(d.path()).unwrap().count(),
            1,
            "no temp file left"
        );
    }

    #[test]
    fn remove_empty_parents_stops_at_root_and_non_empty() {
        let d = TempDir::new("empty_dirs");
        std::fs::create_dir_all(d.path().join("a/b/c")).unwrap();
        d.write("a/keep.txt", b"x");
        assert_eq!(remove_empty_parents(&d.path().join("a/b/c"), d.path()), 2);
        assert!(d.path().join("a").is_dir());
        assert!(!d.path().join("a/b").exists());

        std::fs::create_dir_all(d.path().join("e")).unwrap();
        assert_eq!(
            remove_empty_parents(&d.path().join("e"), &d.path().join("e")),
            0
        );
        assert_eq!(
            remove_empty_parents(&d.path().join("e"), Path::new("/elsewhere")),
            0
        );
        assert!(d.path().join("e").is_dir());
    }

    #[test]
    fn move_and_same_file() {
        let d = TempDir::new("move");
        let a = d.write("a", b"x");
        let b = d.path().join("b");
        move_file(&a, &b).unwrap();
        assert!(!a.exists() && b.exists());
        assert!(same_file(&b, &d.path().join(".").join("b")));
        assert!(!same_file(&b, &a));
    }
}
