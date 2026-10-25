//! Result views (list / grid / side-by-side) and the toolbar above them.

mod compare;
mod grid;
mod list;
mod widgets;

use std::path::PathBuf;

use eframe::egui::{self, Align, Layout, Margin, RichText, Stroke};

use super::results::GroupSort;
use super::theme::{self, Palette, space};
use super::{DupeApp, View, icons};
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
    if app.results.visible().is_empty() {
        ui.add_space(48.0);
        ui.vertical_centered(|ui| {
            let p = Palette::of(ui);
            ui.label(RichText::new(icons::FUNNEL).size(28.0).color(p.muted));
            ui.label(
                RichText::new("No groups match the current filters.")
                    .size(14.0)
                    .color(p.muted),
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
    let p = Palette::of(ui);
    let before = app.results.filter.clone();
    ui.horizontal_wrapped(|ui| {
        let list = format!("{}  List", icons::ROWS);
        let grid = format!("{}  Grid", icons::SQUARES_FOUR);
        let cmp = format!("{}  Side by side", icons::COLUMNS);
        theme::segmented(
            ui,
            "view",
            &mut app.view,
            &[
                (View::List, &list),
                (View::Grid, &grid),
                (View::Compare, &cmp),
            ],
            false,
        );
        ui.add_space(space::XS);

        let f = &mut app.results.filter;
        let kind_text = match f.kind {
            Some(k) => format!("{}  {}", icons::for_kind(k), k.label()),
            None => format!("{}  All types", icons::FUNNEL),
        };
        egui::ComboBox::from_id_salt("kind_filter")
            .selected_text(kind_text)
            .width(150.0)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut f.kind, None, format!("{}  All types", icons::FUNNEL));
                for k in FileKind::ALL {
                    ui.selectable_value(
                        &mut f.kind,
                        Some(k),
                        format!("{}  {}", icons::for_kind(k), k.label()),
                    );
                }
            });
        egui::ComboBox::from_id_salt("sort")
            .selected_text(format!("{}  {}", icons::SORT, f.sort.label()))
            .width(180.0)
            .show_ui(ui, |ui| {
                for s in GroupSort::ALL {
                    ui.selectable_value(&mut f.sort, s, s.label());
                }
            });
        search_box(ui, &mut f.search, &p);
        if theme::chip(ui, f.only_marked, "Only with marks")
            .on_hover_text("Show only groups that contain marked files")
            .clicked()
        {
            f.only_marked = !f.only_marked;
        }

        if app.view != View::List {
            ui.add_space(space::XS);
            ui.label(RichText::new(icons::IMAGE).size(15.0).color(p.muted))
                .on_hover_text("Thumbnail size");
            ui.spacing_mut().slider_width = 110.0;
            ui.add(egui::Slider::new(&mut app.thumb_px, 80.0..=360.0).show_value(false));
        }
    });
    if app.results.filter != before {
        app.results.mark_dirty();
    }

    ui.add_space(space::XS);
    ui.horizontal(|ui| {
        let visible = app.results.visible();
        let files: usize = visible
            .iter()
            .map(|&i| app.results.groups[i].files.len())
            .sum();
        ui.label(
            RichText::new(format!("{} groups · {} files", visible.len(), files))
                .size(12.0)
                .color(p.muted),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(
                RichText::new(
                    "Click a file to switch · right-click for options · double-click to open",
                )
                .size(12.0)
                .color(p.muted),
            );
        });
    });
    ui.add_space(space::SM);
}

fn search_box(ui: &mut egui::Ui, text: &mut String, p: &Palette) {
    egui::Frame::new()
        .fill(p.surface2)
        .stroke(Stroke::new(1.0, p.border))
        .corner_radius(theme::RADIUS)
        .inner_margin(Margin::symmetric(8, 5))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(icons::MAGNIFYING_GLASS)
                        .size(14.0)
                        .color(p.muted),
                );
                ui.add(
                    egui::TextEdit::singleline(text)
                        .hint_text("Search paths")
                        .frame(egui::Frame::NONE)
                        .margin(Margin::ZERO)
                        .align(egui::Align2::LEFT_CENTER)
                        .min_size(egui::vec2(0.0, 20.0))
                        .desired_width(170.0),
                );
            });
        });
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
