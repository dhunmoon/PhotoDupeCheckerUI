//! What the main area shows before there are results: a getting-started guide, scan
//! progress, or "no duplicates found".

use eframe::egui::{self, Align, Align2, Layout, Margin, RichText, Sense, Vec2};

use super::DupeApp;
use super::fonts::{medium, semibold};
use super::icons;
use super::theme::{self, Kind, Palette, Size, space};

pub(crate) fn show(app: &mut DupeApp, ui: &mut egui::Ui) {
    ui.with_layout(Layout::top_down(Align::Center), |ui| {
        let h = ui.available_height();
        ui.add_space((h * 0.14).max(space::XL));
        ui.set_max_width(560.0);
        if let Some(job) = &app.job {
            progress(ui, job.progress());
        } else if let Some(s) = &app.last_scan {
            let p = Palette::of(ui);
            icon_tile(ui, icons::SEAL_CHECK, p.keep);
            ui.add_space(space::LG);
            ui.label(semibold("No duplicates found").size(22.0).color(p.text));
            ui.add_space(space::XS);
            ui.label(
                RichText::new(format!(
                    "Checked {} files in {:.1}s using “{}”.",
                    s.files_scanned,
                    s.seconds,
                    app.cfg.method.label()
                ))
                .color(p.muted),
            );
            if s.errors > 0 {
                ui.label(
                    RichText::new(format!(
                        "{} files could not be read. See the activity log.",
                        s.errors
                    ))
                    .color(p.muted),
                );
            }
            ui.add_space(space::SM);
            ui.label(
                RichText::new(
                    "Try another method, such as Similar-looking images, or add more folders.",
                )
                .color(p.muted),
            );
        } else {
            guide(app, ui);
        }
    });
}

/// Large rounded square with a centred icon.
fn icon_tile(ui: &mut egui::Ui, icon: &str, color: egui::Color32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(64.0), Sense::hover());
    ui.painter()
        .rect_filled(rect, 16.0, color.gamma_multiply(0.14));
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        icon,
        egui::FontId::proportional(32.0),
        color,
    );
}

fn progress(ui: &mut egui::Ui, prog: &crate::scan::Progress) {
    let p = Palette::of(ui);
    let (stage, done, total) = prog.snapshot();
    ui.add(egui::Spinner::new().size(40.0).color(p.accent));
    ui.add_space(space::LG);
    ui.label(semibold(stage).size(18.0).color(p.text));
    ui.add_space(space::SM);
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
            .animate(total == 0)
            .desired_width(380.0)
            .desired_height(8.0)
            .fill(p.accent)
            .corner_radius(4),
    );
    ui.label(RichText::new(text).size(12.0).color(p.muted));
    ui.add_space(space::MD);
    if ui
        .add(theme::button(
            ui,
            Kind::Secondary,
            Size::Md,
            format!("{}  Cancel", icons::STOP),
        ))
        .clicked()
    {
        prog.request_cancel();
    }
}

fn guide(app: &mut DupeApp, ui: &mut egui::Ui) {
    let p = Palette::of(ui);
    if let Some(t) = &app.logo {
        ui.add(egui::Image::new(t).fit_to_exact_size(Vec2::splat(72.0)));
    }
    ui.add_space(space::LG);
    ui.label(
        semibold("Find and clean up duplicate files")
            .size(24.0)
            .color(p.text),
    );
    ui.add_space(space::XS);
    ui.label(
        RichText::new("Photos, videos, music and documents. Nothing changes until you confirm.")
            .size(14.0)
            .color(p.muted),
    );
    ui.add_space(space::XL);

    let steps = [
        (
            icons::FOLDERS,
            "Choose folders",
            "Add the folders to check. Use Source → Target to clean a backup against your originals.",
        ),
        (
            icons::FINGERPRINT,
            "Pick how to match",
            "Identical content finds exact copies. Similar-looking images finds resized or re-saved photos.",
        ),
        (
            icons::BROOM,
            "Review and clean up",
            "Smart select picks the copy to keep. Send the rest to the Trash, a folder, or replace them with links.",
        ),
    ];
    theme::card(ui)
        .inner_margin(Margin::same(20))
        .show(ui, |ui| {
            ui.set_width(480.0);
            for (i, (icon, title, text)) in steps.into_iter().enumerate() {
                if i > 0 {
                    ui.add_space(space::LG);
                }
                ui.horizontal_top(|ui| {
                    let (rect, _) = ui.allocate_exact_size(Vec2::splat(36.0), Sense::hover());
                    ui.painter().rect_filled(rect, 9.0, p.accent_soft);
                    ui.painter().text(
                        rect.center(),
                        Align2::CENTER_CENTER,
                        icon,
                        egui::FontId::proportional(18.0),
                        p.accent,
                    );
                    ui.add_space(space::XS);
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 2.0;
                        ui.label(medium(title).color(p.text));
                        ui.add(
                            egui::Label::new(RichText::new(text).size(12.5).color(p.muted)).wrap(),
                        );
                    });
                });
            }
        });

    ui.add_space(space::XL);
    if app.cfg.folders.is_empty() {
        let b = theme::button(
            ui,
            Kind::Primary,
            Size::Lg,
            format!("{}  Add a folder", icons::PLUS),
        )
        .min_size(Vec2::new(220.0, 40.0));
        if ui.add(b).clicked()
            && let Some(dirs) = rfd::FileDialog::new().pick_folders()
        {
            for d in dirs {
                app.add_folder(d);
            }
        }
        ui.add_space(space::XS);
        ui.label(
            RichText::new("or drag and drop folders onto this window")
                .size(12.0)
                .color(p.muted),
        );
    } else {
        let b = theme::button(
            ui,
            Kind::Primary,
            Size::Lg,
            format!("{}  Scan for duplicates", icons::MAGNIFYING_GLASS),
        )
        .min_size(Vec2::new(220.0, 40.0));
        if ui.add_enabled(app.cfg.validate().is_ok(), b).clicked() {
            let ctx = ui.ctx().clone();
            app.start_scan(&ctx);
        }
        let n = app.cfg.folders.len();
        ui.add_space(space::XS);
        ui.label(
            RichText::new(format!("{n} folder{} ready", if n == 1 { "" } else { "s" }))
                .size(12.0)
                .color(p.muted),
        );
    }
}
