//! Result views (list / grid / side-by-side) and the toolbar above them.

mod compare;
mod grid;
mod list;
mod widgets;

use std::path::PathBuf;

use eframe::egui::{self, Align, Layout, RichText};

use super::results::GroupSort;
use super::theme::{self, KEEP, REMOVE};
use super::{DupeApp, View};
use crate::model::FileKind;
use crate::selection;

/// Requests raised while drawing; applied after the frame (keeps borrows simple).
pub(crate) enum Cmd {
    Toggle(PathBuf),
    KeepOnly(usize, PathBuf),
    Open(PathBuf),
    OpenFolder(PathBuf),
    CopyText(String),
    /// Switch to the Compare view at this position in the visible list.
    Compare(usize),
}

pub(crate) fn show(app: &mut DupeApp, ui: &mut egui::Ui, cmds: &mut Vec<Cmd>) {
    toolbar(app, ui);
    ui.add_space(4.0);
    if app.results.visible().is_empty() {
        ui.add_space(40.0);
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new("No groups match the current filter.")
                    .size(16.0)
                    .weak(),
            );
        });
        return;
    }
    match app.view {
        View::List => list::show(app, ui, cmds),
        View::Grid => grid::show(app, ui, cmds),
        View::Compare => compare::show(app, ui, cmds),
    }
}

fn toolbar(app: &mut DupeApp, ui: &mut egui::Ui) {
    let before = app.results.filter.clone();
    ui.horizontal(|ui| {
        theme::segmented(
            ui,
            &mut app.view,
            &[
                (View::List, "☰ List"),
                (View::Grid, "▦ Grid"),
                (View::Compare, "⚖ Side by side"),
            ],
        );
        ui.separator();
        let f = &mut app.results.filter;
        egui::ComboBox::from_id_salt("kind_filter")
            .selected_text(
                f.kind
                    .map(|k| format!("{} {}", k.icon(), k.label()))
                    .unwrap_or("All types".into()),
            )
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut f.kind, None, "All types");
                for k in FileKind::ALL {
                    ui.selectable_value(
                        &mut f.kind,
                        Some(k),
                        format!("{} {}", k.icon(), k.label()),
                    );
                }
            });
        egui::ComboBox::from_id_salt("sort")
            .selected_text(format!("Sort: {}", f.sort.label()))
            .width(190.0)
            .show_ui(ui, |ui| {
                for s in GroupSort::ALL {
                    ui.selectable_value(&mut f.sort, s, s.label());
                }
            });
        ui.add(
            egui::TextEdit::singleline(&mut f.search)
                .hint_text("🔎 Filter by path…")
                .desired_width(170.0),
        );
        ui.toggle_value(&mut f.only_marked, "Only with marks")
            .on_hover_text("Show only groups that contain marked files");

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if app.view != View::List {
                ui.add(egui::Slider::new(&mut app.thumb_px, 80.0..=360.0).show_value(false));
                ui.label(RichText::new("🔍 Size").small());
            }
        });
    });
    if app.results.filter != before {
        app.results.mark_dirty();
    }

    ui.horizontal(|ui| {
        let visible = app.results.visible();
        let files: usize = visible
            .iter()
            .map(|&i| app.results.groups[i].files.len())
            .sum();
        ui.label(
            RichText::new(format!(
                "Showing {} groups · {} files",
                visible.len(),
                files
            ))
            .small()
            .weak(),
        );
        ui.separator();
        theme::badge(ui, "✔ KEEP", KEEP);
        ui.label(RichText::new("stays untouched").small().weak());
        theme::badge(ui, "✖ REMOVE", REMOVE);
        ui.label(RichText::new("the action applies to it").small().weak());
        ui.separator();
        ui.label(
            RichText::new("Click to switch · right-click for options · double-click to open")
                .small()
                .weak(),
        );
    });
    ui.separator();
}

pub(crate) fn run_cmds(app: &mut DupeApp, cmds: Vec<Cmd>, ctx: &egui::Context) {
    for c in cmds {
        match c {
            Cmd::Toggle(p) => app.results.toggle(&p),
            Cmd::KeepOnly(gi, p) => {
                if let Some(g) = app.results.groups.get(gi) {
                    selection::keep_only(g, &mut app.results.marked, &p);
                }
            }
            Cmd::Open(p) => {
                if let Err(e) = open::that_detached(&p) {
                    app.log.push(format!("Could not open {}: {e}", p.display()));
                }
            }
            Cmd::OpenFolder(p) => {
                if let Some(parent) = p.parent() {
                    let _ = open::that_detached(parent);
                }
            }
            Cmd::CopyText(t) => ctx.copy_text(t),
            Cmd::Compare(pos) => {
                app.compare_pos = pos;
                app.view = View::Compare;
            }
        }
    }
}
