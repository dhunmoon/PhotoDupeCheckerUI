//! Background thumbnail decoding with a bounded GPU texture cache.
//!
//! Thumbnails are decoded on a small dedicated thread pool (so they never compete with a
//! running scan for all cores), uploaded a few per frame, and evicted least-recently-used
//! once more than `capacity` are cached.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crossbeam_channel::{Receiver, Sender};
use eframe::egui;

use crate::phash;

/// State of one thumbnail.
pub enum Thumb {
    Loading,
    Ready(egui::TextureHandle),
    Failed,
}

struct Entry {
    thumb: Thumb,
    last_used: u64,
}

type Key = (PathBuf, u32);

pub struct ThumbCache {
    entries: HashMap<Key, Entry>,
    tx: Sender<(Key, Option<egui::ColorImage>)>,
    rx: Receiver<(Key, Option<egui::ColorImage>)>,
    pool: rayon::ThreadPool,
    frame: u64,
    capacity: usize,
}

impl Default for ThumbCache {
    fn default() -> Self {
        Self::new()
    }
}

impl ThumbCache {
    pub fn new() -> Self {
        let (tx, rx) = crossbeam_channel::unbounded();
        let threads = std::thread::available_parallelism()
            .map(|n| (n.get() / 2).clamp(1, 6))
            .unwrap_or(2);
        Self {
            entries: HashMap::new(),
            tx,
            rx,
            pool: rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .thread_name(|i| format!("thumb-{i}"))
                .build()
                .expect("thumbnail thread pool"),
            frame: 0,
            capacity: 800,
        }
    }

    /// Drops every cached texture (e.g. after a new scan).
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Drops cached textures for a file that was moved / deleted.
    pub fn forget(&mut self, path: &Path) {
        self.entries.retain(|(p, _), _| p != path);
    }

    /// Call once per frame: uploads finished thumbnails and evicts old ones.
    pub fn begin_frame(&mut self, ctx: &egui::Context) {
        self.frame += 1;
        let mut uploaded = 0;
        while uploaded < 24 {
            let Ok((key, img)) = self.rx.try_recv() else {
                break;
            };
            let Some(entry) = self.entries.get_mut(&key) else {
                continue;
            };
            entry.thumb = match img {
                Some(img) => {
                    uploaded += 1;
                    Thumb::Ready(ctx.load_texture(
                        key.0.to_string_lossy(),
                        img,
                        egui::TextureOptions::LINEAR,
                    ))
                }
                None => Thumb::Failed,
            };
        }
        if !self.rx.is_empty()
            || self
                .entries
                .values()
                .any(|e| matches!(e.thumb, Thumb::Loading))
        {
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        }
        if self.entries.len() > self.capacity {
            let mut by_age: Vec<(u64, Key)> = self
                .entries
                .iter()
                .filter(|(_, e)| e.last_used + 2 < self.frame)
                .map(|(k, e)| (e.last_used, k.clone()))
                .collect();
            by_age.sort_by_key(|(t, _)| *t);
            let excess = self.entries.len() - self.capacity;
            for (_, k) in by_age.into_iter().take(excess) {
                self.entries.remove(&k);
            }
        }
    }

    /// Returns the thumbnail for `path`, starting a background decode on first request.
    /// `size` is the max side in pixels of the decoded thumbnail.
    pub fn get(&mut self, path: &Path, size: u32) -> &Thumb {
        let key = (path.to_path_buf(), size);
        let frame = self.frame;
        if !self.entries.contains_key(&key) {
            let tx = self.tx.clone();
            let k = key.clone();
            self.pool.spawn(move || {
                let img = phash::load_small(&k.0, size).ok().map(|l| {
                    let t = l.image.thumbnail(size, size).to_rgba8();
                    egui::ColorImage::from_rgba_unmultiplied(
                        [t.width() as usize, t.height() as usize],
                        t.as_raw(),
                    )
                });
                let _ = tx.send((k, img));
            });
            self.entries.insert(
                key.clone(),
                Entry {
                    thumb: Thumb::Loading,
                    last_used: frame,
                },
            );
        }
        let e = self.entries.get_mut(&key).unwrap();
        e.last_used = frame;
        &e.thumb
    }
}
