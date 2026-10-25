//! Generic grouping helpers and the cheap metadata-only methods (name / size).

use std::collections::HashMap;
use std::hash::Hash;

use crate::model::{FileEntry, Method, ScanConfig, human_size};

/// Buckets `items` by `key` and returns only buckets with two or more members.
pub(crate) fn group_by<T, K: Hash + Eq>(items: Vec<T>, key: impl Fn(&T) -> K) -> Vec<Vec<T>> {
    let mut map: HashMap<K, Vec<T>> = HashMap::new();
    for f in items {
        map.entry(key(&f)).or_default().push(f);
    }
    map.into_values().filter(|g| g.len() > 1).collect()
}

/// The file name as compared by the name-based methods (honours case / extension options).
pub(crate) fn name_key(f: &FileEntry, cfg: &ScanConfig) -> String {
    let name = if cfg.name_ignore_ext {
        f.path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default()
    } else {
        f.name()
    };
    if cfg.name_case_insensitive {
        name.to_lowercase()
    } else {
        name
    }
}

/// Short description of why the files in a group match. `files` must not be empty.
pub(crate) fn group_label(cfg: &ScanConfig, files: &[FileEntry]) -> String {
    let f = &files[0];
    match cfg.method {
        Method::FileName => format!("Name: {}", name_key(f, cfg)),
        Method::NameAndSize => format!("Name: {} · {}", f.name(), human_size(f.size)),
        Method::Size => format!("Size: {}", human_size(f.size)),
        Method::Content => format!("Identical content · {}", human_size(f.size)),
        Method::SimilarImage => "Visually similar".to_string(),
    }
}

/// Classic disjoint-set with path halving; used to merge chains of similar images.
pub(crate) struct UnionFind {
    parent: Vec<usize>,
}

impl UnionFind {
    pub fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
        }
    }

    pub fn find(&mut self, mut x: usize) -> usize {
        while self.parent[x] != x {
            self.parent[x] = self.parent[self.parent[x]];
            x = self.parent[x];
        }
        x
    }

    pub fn union(&mut self, a: usize, b: usize) {
        let (ra, rb) = (self.find(a), self.find(b));
        if ra != rb {
            self.parent[ra] = rb;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_by_drops_singletons() {
        let mut g = group_by(vec![1, 2, 3, 11, 12, 21], |x| x / 10);
        g.iter_mut().for_each(|v| v.sort());
        g.sort();
        assert_eq!(g, vec![vec![1, 2, 3], vec![11, 12]]);
        assert!(group_by(Vec::<u8>::new(), |x| *x).is_empty());
    }

    #[test]
    fn name_key_options() {
        let f = FileEntry::new("/x/Photo.JPG", 1);
        let mut cfg = ScanConfig::default();
        assert_eq!(name_key(&f, &cfg), "photo.jpg");
        cfg.name_ignore_ext = true;
        assert_eq!(name_key(&f, &cfg), "photo");
        cfg.name_case_insensitive = false;
        assert_eq!(name_key(&f, &cfg), "Photo");
    }

    #[test]
    fn labels_per_method() {
        let files = [FileEntry::new("/a/b.txt", 2048)];
        let mut cfg = ScanConfig::default();
        assert_eq!(group_label(&cfg, &files), "Identical content · 2.00 KB");
        cfg.method = Method::FileName;
        assert_eq!(group_label(&cfg, &files), "Name: b.txt");
        cfg.method = Method::Size;
        assert_eq!(group_label(&cfg, &files), "Size: 2.00 KB");
    }

    #[test]
    fn union_find_merges_transitively() {
        let mut uf = UnionFind::new(5);
        uf.union(0, 1);
        uf.union(1, 2);
        uf.union(3, 4);
        assert_eq!(uf.find(0), uf.find(2));
        assert_eq!(uf.find(3), uf.find(4));
        assert_ne!(uf.find(0), uf.find(3));
    }
}
