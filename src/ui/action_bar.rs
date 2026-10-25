//! Bottom bar: smart selection on the left, what to do with marked files on the right.

use eframe::egui::{self, Align, Layout, Margin, RichText};

use super::theme::{self, REMOVE, WARN};
use super::{DupeApp, PendingAction};
use crate::actions::ActionKind;
use crate::model::human_size;
use crate::selection::{self, KeepRule};

pub(crate) fn show(app: &mut DupeApp, ui: &mut egui::Ui) {
    egui::Panel::bottom("action_bar")
        .frame(egui::Frame::side_top_panel(ui.style()).inner_margin(Margin::symmetric(14, 8)))
        .show(ui, |ui| {
            ui.add_enabled_ui(!app.busy(), |ui| {
                ui.horizontal(|ui| {
                    selection_controls(app, ui);
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        action_controls(app, ui)
                    });
                });
            });
            if !app.status.is_empty() {
                ui.label(RichText::new(&app.status).small().weak());
            }
        });
}

fn selection_controls(app: &mut DupeApp, ui: &mut egui::Ui) {
    let before = app.results.marked.len();
    ui.menu_button(RichText::new("✨ Smart select ▾").strong(), |ui| {
        smart_select_menu(app, ui)
    })
    .response
    .on_hover_text("Automatically decide which copy to keep in every group");
    if ui
        .button("Invert")
        .on_hover_text("Swap kept and marked files")
        .clicked()
    {
        selection::invert(&app.results.groups, &mut app.results.marked);
    }
    if ui
        .button("Clear")
        .on_hover_text("Unmark everything")
        .clicked()
    {
        app.results.marked.clear();
    }
    if app.results.filter.only_marked && app.results.marked.len() != before {
        app.results.mark_dirty();
    }

    ui.separator();
    let s = app.results.stats();
    if s.files == 0 {
        ui.label(RichText::new("Nothing marked yet").weak())
            .on_hover_text(
                "Use Smart select, or click a file's KEEP badge to mark it for removal.",
            );
    } else {
        ui.label(
            RichText::new(format!(
                "{} file{} marked · {}",
                s.files,
                if s.files == 1 { "" } else { "s" },
                human_size(s.bytes)
            ))
            .color(REMOVE)
            .strong(),
        );
    }
    if s.fully_marked_groups > 0 {
        theme::badge(ui, format!("⚠ {} group(s) fully marked", s.fully_marked_groups), WARN).on_hover_text(
            "Every copy is marked in these groups. They are skipped by default so you never lose all copies.",
        );
    }
}

fn smart_select_menu(app: &mut DupeApp, ui: &mut egui::Ui) {
    ui.set_min_width(300.0);
    ui.label(
        RichText::new("In every group, keep one copy and mark the rest:")
            .small()
            .weak(),
    );
    let svt = app.results.is_source_vs_target();
    for r in KeepRule::ALL {
        let enabled = !r.needs_source_folders() || svt;
        let resp = ui.add_enabled(enabled, egui::Button::new(r.label()).frame(false));
        let resp = if enabled {
            resp
        } else {
            resp.on_disabled_hover_text("Only for Source → Target scans")
        };
        if resp.clicked() {
            selection::apply_keep_rule(&app.results.groups, &mut app.results.marked, r);
            ui.close();
        }
    }
    ui.separator();
    ui.label(RichText::new("By folder").small().weak());
    if ui
        .add(egui::Button::new("Keep the copies inside a folder…").frame(false))
        .clicked()
    {
        ui.close();
        if let Some(d) = rfd::FileDialog::new().pick_folder() {
            selection::keep_in_folder(&app.results.groups, &mut app.results.marked, &d);
        }
    }
    if ui
        .add(egui::Button::new("Mark everything inside a folder…").frame(false))
        .clicked()
    {
        ui.close();
        if let Some(d) = rfd::FileDialog::new().pick_folder() {
            selection::select_in_folder(&app.results.groups, &mut app.results.marked, &d);
        }
    }
    ui.separator();
    ui.label(
        RichText::new("By pattern (adds to the current marks)")
            .small()
            .weak(),
    );
    ui.horizontal(|ui| {
        ui.add(
            egui::TextEdit::singleline(&mut app.ext_input)
                .hint_text("extensions: png, bmp")
                .desired_width(170.0),
        );
        if ui.button("Mark").clicked() {
            selection::select_by_extension(
                &app.results.groups,
                &mut app.results.marked,
                &app.ext_input,
            );
        }
    });
    ui.horizontal(|ui| {
        ui.add(
            egui::TextEdit::singleline(&mut app.text_input)
                .hint_text("path contains: backup")
                .desired_width(170.0),
        );
        if ui.button("Mark").clicked() {
            selection::select_by_text(
                &app.results.groups,
                &mut app.results.marked,
                &app.text_input,
            );
        }
    });
}

fn action_controls(app: &mut DupeApp, ui: &mut egui::Ui) {
    let n = app.results.marked.len();
    let enabled = n > 0;
    let ask = |kind: ActionKind| PendingAction {
        skip_fully_marked: true,
        remove_empty_dirs: false,
        kind,
    };

    let trash =
        theme::danger_button(format!("🗑  Move {n} to Trash")).min_size(egui::vec2(0.0, 32.0));
    if ui
        .add_enabled(enabled, trash)
        .on_hover_text("Recoverable from the system Trash / Recycle Bin")
        .on_disabled_hover_text("Mark some files first")
        .clicked()
    {
        app.pending = Some(ask(ActionKind::Trash));
    }

    ui.menu_button("More actions ▾", |ui| {
        ui.set_min_width(260.0);
        ui.add_enabled_ui(enabled, |ui| {
            if ui
                .button("📂  Move marked to a folder…")
                .on_hover_text("Move into a quarantine folder of your choice")
                .clicked()
            {
                ui.close();
                if let Some(dest) = rfd::FileDialog::new().pick_folder() {
                    app.pending = Some(ask(ActionKind::MoveTo {
                        dest,
                        keep_structure: app.keep_structure,
                    }));
                }
            }
            if ui
                .button("📋  Copy marked to a folder…")
                .on_hover_text("Originals stay in place")
                .clicked()
            {
                ui.close();
                if let Some(dest) = rfd::FileDialog::new().pick_folder() {
                    let mut p = ask(ActionKind::CopyTo {
                        dest,
                        keep_structure: app.keep_structure,
                    });
                    p.skip_fully_marked = false;
                    app.pending = Some(p);
                }
            }
            ui.checkbox(
                &mut app.keep_structure,
                "Recreate folder structure when moving/copying",
            );
            ui.separator();
            if ui
                .button("🔗  Replace with hard links")
                .on_hover_text("Frees space but every path keeps working (same drive only)")
                .clicked()
            {
                ui.close();
                app.pending = Some(ask(ActionKind::HardLink));
            }
            if ui
                .button("↪  Replace with shortcuts (symlinks)")
                .on_hover_text("Each marked file becomes a link to the kept copy")
                .clicked()
            {
                ui.close();
                app.pending = Some(ask(ActionKind::SymLink));
            }
            ui.separator();
            if ui
                .button(RichText::new("❌  Delete permanently…").color(REMOVE))
                .on_hover_text("Skips the Trash. Cannot be undone.")
                .clicked()
            {
                ui.close();
                app.pending = Some(ask(ActionKind::Delete));
            }
            ui.separator();
            if ui.button("📄  Copy marked paths to clipboard").clicked() {
                ui.ctx()
                    .copy_text(app.results.marked_paths_sorted().join("\n"));
                app.status = format!("Copied {n} paths to the clipboard");
                ui.close();
            }
        });
        if ui.button("💾  Export report (CSV)…").clicked() {
            ui.close();
            export_csv(app);
        }
    });
}

fn export_csv(app: &mut DupeApp) {
    let Some(path) = rfd::FileDialog::new()
        .set_file_name("duplicates.csv")
        .add_filter("CSV", &["csv"])
        .save_file()
    else {
        return;
    };
    let csv = crate::report::to_csv(&app.results.groups, &app.results.marked);
    app.status = match std::fs::write(&path, csv) {
        Ok(()) => format!("Exported report to {}", path.display()),
        Err(e) => format!("Export failed: {e}"),
    };
}
