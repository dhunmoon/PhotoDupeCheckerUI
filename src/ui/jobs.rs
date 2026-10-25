//! Background jobs (scan / file action) and how their results get back to the UI thread.

use std::sync::Arc;

use crossbeam_channel::Receiver;
use eframe::egui;

use crate::actions::{self, ActionItem, ActionKind, ActionOptions, ActionReport};
use crate::model::ScanConfig;
use crate::scan::{self, Progress, ScanResult};

/// A running background job. The UI polls it once per frame.
pub enum Job {
    Scan(Arc<Progress>, Receiver<Result<ScanResult, String>>),
    Action(Arc<Progress>, Receiver<(ActionKind, ActionReport)>),
}

/// What a finished job produced.
pub enum JobOutput {
    Scan(Result<ScanResult, String>),
    Action(ActionKind, ActionReport),
}

impl Job {
    pub fn progress(&self) -> &Progress {
        match self {
            Job::Scan(p, _) | Job::Action(p, _) => p,
        }
    }

    pub fn is_scan(&self) -> bool {
        matches!(self, Job::Scan(..))
    }

    /// Starts scanning `cfg` on a new thread.
    pub fn spawn_scan(cfg: ScanConfig, ctx: &egui::Context) -> Self {
        let progress = Arc::new(Progress::default());
        let (tx, rx) = crossbeam_channel::bounded(1);
        let p = progress.clone();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(scan::run_scan(&cfg, &p));
            ctx.request_repaint();
        });
        Job::Scan(progress, rx)
    }

    /// Runs `kind` on `items` on a new thread.
    pub fn spawn_action(
        kind: ActionKind,
        items: Vec<ActionItem>,
        opts: ActionOptions,
        ctx: &egui::Context,
    ) -> Self {
        let progress = Arc::new(Progress::default());
        let (tx, rx) = crossbeam_channel::bounded(1);
        let p = progress.clone();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let report = actions::execute(&kind, &items, opts, &p);
            let _ = tx.send((kind, report));
            ctx.request_repaint();
        });
        Job::Action(progress, rx)
    }

    /// Returns the output if the job has finished.
    pub fn try_finish(&self) -> Option<JobOutput> {
        match self {
            Job::Scan(_, rx) => rx.try_recv().ok().map(JobOutput::Scan),
            Job::Action(_, rx) => rx.try_recv().ok().map(|(k, r)| JobOutput::Action(k, r)),
        }
    }
}
