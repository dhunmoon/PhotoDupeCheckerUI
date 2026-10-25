//! BK-tree: a metric tree over Hamming distance for fast "everything within N bits" queries.
//!
//! Comparing every pair of images is O(n²); a BK-tree prunes most of the comparisons using
//! the triangle inequality, which keeps similar-image grouping fast for large libraries.

use super::distance;

#[derive(Default)]
pub struct BkTree {
    nodes: Vec<BkNode>,
}

struct BkNode {
    hash: u128,
    item: usize,
    /// (distance from this node, index of child node)
    children: Vec<(u32, usize)>,
}

impl BkTree {
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Stores `hash` with an arbitrary caller-defined `item` id.
    pub fn insert(&mut self, hash: u128, item: usize) {
        let new_idx = self.nodes.len();
        if !self.nodes.is_empty() {
            let mut cur = 0;
            loop {
                let d = distance(self.nodes[cur].hash, hash);
                match self.nodes[cur].children.iter().find(|(cd, _)| *cd == d) {
                    Some(&(_, next)) => cur = next,
                    None => {
                        self.nodes[cur].children.push((d, new_idx));
                        break;
                    }
                }
            }
        }
        self.nodes.push(BkNode {
            hash,
            item,
            children: Vec::new(),
        });
    }

    /// Calls `f(item, distance)` for every stored hash within `radius` of `hash`.
    pub fn query(&self, hash: u128, radius: u32, mut f: impl FnMut(usize, u32)) {
        if self.nodes.is_empty() {
            return;
        }
        let mut stack = vec![0usize];
        while let Some(i) = stack.pop() {
            let node = &self.nodes[i];
            let d = distance(node.hash, hash);
            if d <= radius {
                f(node.item, d);
            }
            let lo = d.saturating_sub(radius);
            let hi = d + radius;
            for &(cd, child) in &node.children {
                if cd >= lo && cd <= hi {
                    stack.push(child);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::bytes;

    #[test]
    fn finds_neighbours() {
        let mut t = BkTree::default();
        assert!(t.is_empty());
        t.insert(0, 0);
        t.insert(0b1, 1);
        t.insert(0b111, 2);
        t.insert(u128::MAX, 3);
        assert_eq!(t.len(), 4);
        let mut found = vec![];
        t.query(0, 1, |i, _| found.push(i));
        found.sort();
        assert_eq!(found, vec![0, 1]);
    }

    #[test]
    fn empty_tree_query_is_noop() {
        let t = BkTree::default();
        t.query(42, 128, |_, _| panic!("should not be called"));
    }

    #[test]
    fn matches_brute_force_on_random_hashes() {
        let raw = bytes(16 * 400, 7);
        let hashes: Vec<u128> = raw
            .as_chunks::<16>()
            .0
            .iter()
            .map(|c| u128::from_le_bytes(*c))
            .collect();
        let mut t = BkTree::default();
        for (i, h) in hashes.iter().enumerate() {
            t.insert(*h, i);
        }
        for radius in [0, 40, 60, 64] {
            for q in hashes.iter().take(25) {
                let mut got = vec![];
                t.query(*q, radius, |i, d| {
                    assert_eq!(d, distance(hashes[i], *q));
                    got.push(i)
                });
                got.sort();
                let want: Vec<usize> = (0..hashes.len())
                    .filter(|&i| distance(hashes[i], *q) <= radius)
                    .collect();
                assert_eq!(got, want, "radius {radius}");
            }
        }
    }
}
