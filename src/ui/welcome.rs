//! What the main area shows before there are results: a guide, scan progress, or
//! "no duplicates found".

use eframe::egui::{self, Align, Layout, RichText, Vec2};

use super::DupeApp;
use super::theme::{self, ACCENT, KEEP};

pub(crate) fn show(app: &mut DupeApp, ui: &mut egui::Ui) {
    ui.with_layout(Layout::top_down(Align::Center), |ui| {
        let h = ui.available_height();
        ui.add_space((h * 0.18).max(20.0));
        ui.set_max_width(560.0);
        if let Some(job) = &app.job {
            progress(ui, job.progress());
        } else if let Some(s) = &app.last_scan {
            ui.label(RichText::new("🎉").size(48.0));
            ui.label(
                RichText::new("No duplicates found")
                    .size(24.0)
                    .strong()
                    .color(KEEP),
            );
            ui.add_space(6.0);
            ui.label(
                RichText::new(format!(
                    "Checked {} files in {:.1}s using “{}”.",
                    s.files_scanned,
                    s.seconds,
                    app.cfg.method.label()
                ))
                .weak(),
            );
            if s.errors > 0 {
                ui.label(
                    RichText::new(format!(
                        "{} files could not be read — see the log.",
                        s.errors
                    ))
                    .weak(),
                );
            }
            ui.add_space(6.0);
            ui.label(
                RichText::new(
                    "Try another method (e.g. Similar-looking images) or add more folders.",
                )
                .weak(),
            );
        } else {
            guide(app, ui);
        }
    });
}

fn progress(ui: &mut egui::Ui, p: &crate::scan::Progress) {
    let (stage, done, total) = p.snapshot();
    ui.add(egui::Spinner::new().size(48.0));
    ui.add_space(10.0);
    ui.label(RichText::new(stage).size(20.0).strong());
    let frac = if total > 0 {
        done as f32 / total as f32
    } else {
        0.0
    };
    let text = if total > 0 {
        format!("{done} of {total}")
    } else {
        format!("{done} files found")
    };
    ui.add(
        egui::ProgressBar::new(frac)
            .text(text)
            .animate(total == 0)
            .desired_width(420.0),
    );
    ui.add_space(8.0);
    if ui.button("⏹ Cancel").clicked() {
        p.request_cancel();
    }
}

fn guide(app: &mut DupeApp, ui: &mut egui::Ui) {
    ui.label(RichText::new("🔍").size(52.0).color(ACCENT));
    ui.label(
        RichText::new("Find and clean up duplicate files")
            .size(26.0)
            .strong(),
    );
    ui.label(
        RichText::new("Photos, videos, music, documents — nothing is changed until you confirm.")
            .weak(),
    );
    ui.add_space(18.0);

    theme::card(ui).show(ui, |ui| {
        ui.set_width(460.0);
        for (n, title, text) in [
            (1, "Choose folders", "Add the folders to check. Use Source → Target to clean a backup against your originals."),
            (2, "Pick a method", "“Identical content” for exact copies, “Similar-looking images” for resized or re-saved photos."),
            (3, "Review & clean up", "Smart select picks which copy to keep. Marked files can go to the Trash, a folder, or become links."),
        ] {
            ui.horizontal_top(|ui| {
                theme::step_title(ui, n, "");
                ui.vertical(|ui| {
                    ui.label(RichText::new(title).strong());
                    ui.add(egui::Label::new(RichText::new(text).small().weak()).wrap());
                });
            });
            ui.add_space(4.0);
        }
    });

    ui.add_space(18.0);
    if app.cfg.folders.is_empty() {
        if ui
            .add(
                theme::primary_button("➕  Add a folder to start").min_size(Vec2::new(240.0, 42.0)),
            )
            .clicked()
            && let Some(dirs) = rfd::FileDialog::new().pick_folders()
        {
            for d in dirs {
                app.add_folder(d);
            }
        }
        ui.label(
            RichText::new("or drag & drop folders onto this window")
                .small()
                .weak(),
        );
    } else {
        let can = app.cfg.validate().is_ok();
        if ui
            .add_enabled(
                can,
                theme::primary_button("🔍  Scan for duplicates").min_size(Vec2::new(240.0, 42.0)),
            )
            .clicked()
        {
            let ctx = ui.ctx().clone();
            app.start_scan(&ctx);
        }
        let n = app.cfg.folders.len();
        ui.label(
            RichText::new(format!("{n} folder{} ready", if n == 1 { "" } else { "s" }))
                .small()
                .weak(),
        );
    }
}
