//! Shared, thread-safe progress reporting and cancellation for background jobs.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// Written by worker threads, read by the UI every frame.
#[derive(Default)]
pub struct Progress {
    pub stage: Mutex<String>,
    pub done: AtomicU64,
    pub total: AtomicU64,
    pub cancel: AtomicBool,
}

impl Progress {
    /// Starts a new stage. `total == 0` means "unknown" (an indeterminate bar).
    pub fn set_stage(&self, stage: &str, total: u64) {
        *self.stage.lock().unwrap() = stage.to_string();
        self.done.store(0, Ordering::Relaxed);
        self.total.store(total, Ordering::Relaxed);
    }

    pub fn tick(&self) {
        self.done.fetch_add(1, Ordering::Relaxed);
    }

    pub fn request_cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }

    pub fn cancelled(&self) -> bool {
        self.cancel.load(Ordering::Relaxed)
    }

    /// `(stage, done, total)` snapshot for display.
    pub fn snapshot(&self) -> (String, u64, u64) {
        (
            self.stage.lock().unwrap().clone(),
            self.done.load(Ordering::Relaxed),
            self.total.load(Ordering::Relaxed),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_resets_counter() {
        let p = Progress::default();
        p.set_stage("one", 3);
        p.tick();
        p.tick();
        assert_eq!(p.snapshot(), ("one".into(), 2, 3));
        p.set_stage("two", 0);
        assert_eq!(p.snapshot(), ("two".into(), 0, 0));
        assert!(!p.cancelled());
        p.request_cancel();
        assert!(p.cancelled());
    }
}
