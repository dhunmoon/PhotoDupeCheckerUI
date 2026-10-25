//! egui front end.
//!
//! Layout:
//! ```text
//! ┌──────────────────────── top_bar ─────────────────────────┐
//! │ sidebar │ results views (welcome / progress / list / grid │
//! │ (setup  │ / compare)                                      │
//! │  steps) ├──────────────── action_bar ─────────────────────┤
//! └─────────┴─────────────────────────────────────────────────┘
//! ```
//! Each region lives in its own module and gets `&mut DupeApp`. Views never mutate state
//! that would invalidate indices while drawing; instead they push `views::Cmd`s that are
//! applied after the frame is drawn.

mod action_bar;
mod dialogs;
mod fonts;
mod icons;
mod jobs;
mod logo;
pub mod results;
mod sidebar;
mod theme;
mod thumbs;
mod top_bar;
mod views;
mod welcome;

use std::path::{Path, PathBuf};

use eframe::egui;

use crate::actions::{ActionKind, ActionOptions, ActionReport};
use crate::cli::LaunchArgs;
use crate::model::{ScanConfig, human_size};
use crate::scan::ScanResult;
use jobs::{Job, JobOutput};
use results::Results;
use thumbs::ThumbCache;

/// Opens the main window. Blocks until it is closed.
pub fn run(launch: LaunchArgs) -> eframe::Result {
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Dupe Finder")
        .with_app_id("dupe-finder")
        .with_inner_size([1400.0, 900.0])
        .with_min_inner_size([1000.0, 640.0])
        .with_drag_and_drop(true);
    if let Some(icon) = logo::window_icon() {
        viewport = viewport.with_icon(icon);
    }
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "Dupe Finder",
        options,
        Box::new(|cc| Ok(Box::new(DupeApp::new(cc, launch)))),
    )
}

#[derive(PartialEq, Clone, Copy)]
pub(crate) enum View {
    List,
    Grid,
    Compare,
}

/// An action waiting for confirmation in the modal dialog.
pub(crate) struct PendingAction {
    pub kind: ActionKind,
    pub skip_fully_marked: bool,
    pub remove_empty_dirs: bool,
}

/// Summary of the last finished scan (for the empty-result screen and the status line).
pub(crate) struct ScanSummary {
    pub files_scanned: usize,
    pub seconds: f32,
    pub errors: usize,
}

pub struct DupeApp {
    pub(crate) cfg: ScanConfig,
    /// Size filters as shown in the UI; converted to bytes when a scan starts.
    pub(crate) min_size_kb: f64,
    pub(crate) max_size_mb: f64,
    pub(crate) job: Option<Job>,
    pub(crate) results: Results,
    pub(crate) last_scan: Option<ScanSummary>,
    pub(crate) view: View,
    pub(crate) compare_pos: usize,
    pub(crate) thumb_px: f32,
    pub(crate) thumbs: ThumbCache,
    /// One-line status shown in the action bar.
    pub(crate) status: String,
    pub(crate) log: Vec<String>,
    pub(crate) show_log: bool,
    pub(crate) pending: Option<PendingAction>,
    pub(crate) keep_structure: bool,
    pub(crate) ext_input: String,
    pub(crate) text_input: String,
    pub(crate) show_filters: bool,
    pub(crate) logo: Option<egui::TextureHandle>,
}

impl DupeApp {
    pub fn new(cc: &eframe::CreationContext<'_>, launch: LaunchArgs) -> Self {
        theme::install(&cc.egui_ctx);
        let mut app = Self {
            cfg: ScanConfig::default(),
            min_size_kb: 0.001,
            max_size_mb: 0.0,
            job: None,
            results: Results::default(),
            last_scan: None,
            view: View::List,
            compare_pos: 0,
            thumb_px: 160.0,
            thumbs: ThumbCache::new(),
            status: String::new(),
            log: Vec::new(),
            show_log: false,
            pending: None,
            keep_structure: true,
            ext_input: String::new(),
            text_input: String::new(),
            show_filters: false,
            logo: logo::texture(&cc.egui_ctx),
        };
        for f in launch.folders {
            app.cfg.add_folder(f);
        }
        if let Some(m) = launch.method {
            app.cfg.method = m;
        }
        if launch.auto_scan && !app.cfg.folders.is_empty() {
            app.start_scan(&cc.egui_ctx);
        }
        app
    }

    pub(crate) fn busy(&self) -> bool {
        self.job.is_some()
    }

    pub(crate) fn add_folder(&mut self, path: PathBuf) {
        self.cfg.add_folder(path);
    }

    pub(crate) fn start_scan(&mut self, ctx: &egui::Context) {
        self.cfg.min_size = (self.min_size_kb * 1024.0).round() as u64;
        self.cfg.max_size = (self.max_size_mb * 1024.0 * 1024.0).round() as u64;
        if let Err(e) = self.cfg.validate() {
            self.status = e;
            return;
        }
        self.job = Some(Job::spawn_scan(self.cfg.clone(), ctx));
    }

    pub(crate) fn start_action(&mut self, p: PendingAction, ctx: &egui::Context) {
        let (items, _) = self.results.build_items(&p.kind, p.skip_fully_marked);
        if items.is_empty() {
            self.status = "Nothing to do: no eligible files are marked.".into();
            return;
        }
        let opts = ActionOptions {
            remove_empty_dirs: p.remove_empty_dirs,
        };
        self.job = Some(Job::spawn_action(p.kind, items, opts, ctx));
    }

    fn poll_job(&mut self) {
        let Some(out) = self.job.as_ref().and_then(Job::try_finish) else {
            return;
        };
        self.job = None;
        match out {
            JobOutput::Scan(Ok(res)) => self.apply_scan(res),
            JobOutput::Scan(Err(e)) => {
                self.status = e.clone();
                self.log.push(e);
            }
            JobOutput::Action(kind, report) => self.apply_report(&kind, report),
        }
    }

    fn apply_scan(&mut self, res: ScanResult) {
        self.status = format!(
            "Scanned {} files in {:.1}s · {} groups · {} duplicate files · {} reclaimable{}",
            res.files_scanned,
            res.elapsed.as_secs_f32(),
            res.groups.len(),
            res.duplicate_files(),
            human_size(res.reclaimable()),
            if res.errors.is_empty() {
                String::new()
            } else {
                format!(" · {} could not be read (see log)", res.errors.len())
            }
        );
        self.last_scan = Some(ScanSummary {
            files_scanned: res.files_scanned,
            seconds: res.elapsed.as_secs_f32(),
            errors: res.errors.len(),
        });
        self.log.push(self.status.clone());
        self.log
            .extend(res.errors.into_iter().map(|e| format!("  ⚠ {e}")));
        self.results.set(res.groups, self.cfg.clone());
        self.thumbs.clear();
        self.compare_pos = 0;
    }

    fn apply_report(&mut self, kind: &ActionKind, report: ActionReport) {
        let mut msg = format!(
            "{}: {} succeeded, {} failed",
            kind.label(),
            report.succeeded.len(),
            report.failed.len()
        );
        if report.freed > 0 {
            msg += &format!(", {} freed", human_size(report.freed));
        }
        if report.removed_dirs > 0 {
            msg += &format!(", {} empty folder(s) removed", report.removed_dirs);
        }
        self.log.push(msg.clone());
        for (p, e) in &report.failed {
            self.log.push(format!("  ✖ {}: {e}", p.display()));
        }
        if !report.failed.is_empty() {
            self.show_log = true;
        }
        self.status = msg;
        for p in &report.succeeded {
            self.thumbs.forget(p);
        }
        self.results.apply_report(kind, &report);
    }

    fn handle_dropped_files(&mut self, ctx: &egui::Context) {
        let dropped: Vec<PathBuf> = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .map(|f| f.path().to_path_buf())
                .filter(|p| !p.as_os_str().is_empty())
                .collect()
        });
        if self.busy() {
            return;
        }
        for p in dropped {
            let dir = if p.is_dir() {
                p
            } else {
                p.parent().map(Path::to_path_buf).unwrap_or(p)
            };
            self.add_folder(dir);
        }
    }
}

impl eframe::App for DupeApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.poll_job();
        self.thumbs.begin_frame(&ctx);
        self.handle_dropped_files(&ctx);
        self.results.refresh();
        self.compare_pos = self
            .compare_pos
            .min(self.results.visible().len().saturating_sub(1));

        top_bar::show(self, ui);
        if !self.results.is_empty() {
            action_bar::show(self, ui);
        }
        sidebar::show(self, ui);

        let mut cmds = Vec::new();
        let bg = theme::Palette::of(ui).bg;
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(bg)
                    .inner_margin(egui::Margin::same(20)),
            )
            .show(ui, |ui| {
                if self.results.is_empty() {
                    welcome::show(self, ui);
                } else {
                    views::show(self, ui, &mut cmds);
                }
            });
        views::run_cmds(self, cmds, &ctx);

        dialogs::confirm_action(self, &ctx);
        dialogs::log_window(self, &ctx);
        if self.busy() {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
    }
}
