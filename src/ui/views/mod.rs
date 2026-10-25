//! Result views (list / grid / side-by-side) and the toolbar above them.

mod compare;
mod grid;
mod list;
mod widgets;

use std::path::PathBuf;

use eframe::egui::{self, Align, Layout, RichText};

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

const SEARCH_MIN_W: f32 = 200.0;
const SEARCH_MAX_W: f32 = 360.0;

/// List / Grid / Side by side switcher. Returns its width.
fn view_switcher(app: &mut DupeApp, ui: &mut egui::Ui) -> f32 {
    let list = format!("{}  List", icons::ROWS);
    let grid = format!("{}  Grid", icons::SQUARES_FOUR);
    let cmp = format!("{}  Side by side", icons::COLUMNS);
    let left = ui.cursor().left();
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
    ui.min_rect().right() - left
}

/// Type filter, sort order and "Only with marks". Returns the width used.
fn filters(app: &mut DupeApp, ui: &mut egui::Ui) -> f32 {
    let left = ui.cursor().left();
    let mut right = left;
    let mut track = |r: &egui::Response| right = right.max(r.rect.right());
    let f = &mut app.results.filter;
    let kind_text = match f.kind {
        Some(k) => format!("{}  {}", icons::for_kind(k), k.label()),
        None => format!("{}  All types", icons::FUNNEL),
    };
    let r = egui::ComboBox::from_id_salt("kind_filter")
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
    track(&r.response);
    let r = egui::ComboBox::from_id_salt("sort")
        .selected_text(format!("{}  {}", icons::SORT, f.sort.label()))
        .width(180.0)
        .show_ui(ui, |ui| {
            for s in GroupSort::ALL {
                ui.selectable_value(&mut f.sort, s, s.label());
            }
        });
    track(&r.response);
    let r = theme::chip_sized(ui, f.only_marked, "Only with marks", theme::height::MD)
        .on_hover_text("Show only groups that contain marked files");
    if r.clicked() {
        f.only_marked = !f.only_marked;
    }
    track(&r);

    right - left
}

fn toolbar(app: &mut DupeApp, ui: &mut egui::Ui) {
    let p = Palette::of(ui);
    let before = app.results.filter.clone();
    // View switcher, search and filters share one row when they fit; on narrow windows
    // the filters move to a second row. Widths are measured on the previous frame.
    let seg_id = ui.scope_id().with("toolbar_seg_w");
    let filters_id = ui.scope_id().with("toolbar_filters_w");
    let seg_w = ui.data(|d| d.get_temp::<f32>(seg_id)).unwrap_or(300.0);
    let filters_w = ui.data(|d| d.get_temp::<f32>(filters_id)).unwrap_or(560.0);
    let gap = ui.spacing().item_spacing.x;
    let avail = ui.available_width();
    let rest = avail - seg_w - filters_w - space::XS - 3.0 * gap;
    let one_row = rest >= SEARCH_MIN_W;

    let mut measured_seg = seg_w;
    let mut measured_filters = filters_w;
    ui.horizontal(|ui| {
        ui.spacing_mut().interact_size.y = theme::height::MD;
        measured_seg = view_switcher(app, ui);
        ui.add_space(space::XS);
        let search_w = if one_row {
            rest.min(SEARCH_MAX_W)
        } else {
            (ui.available_width()).clamp(SEARCH_MIN_W, SEARCH_MAX_W)
        };
        theme::search_field(ui, &mut app.results.filter.search, "Search paths", search_w);
        if one_row {
            measured_filters = filters(app, ui);
        }
    });
    if !one_row {
        ui.add_space(space::XS);
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().interact_size.y = theme::height::MD;
            measured_filters = filters(app, ui);
        });
    }
    if (measured_seg - seg_w).abs() > 0.5 || (measured_filters - filters_w).abs() > 0.5 {
        ui.data_mut(|d| {
            d.insert_temp(seg_id, measured_seg);
            d.insert_temp(filters_id, measured_filters);
        });
        ui.ctx().request_repaint();
    }
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
            // Thumbnail size lives here so the toolbar is the same width in every view.
            if app.view != View::List {
                ui.spacing_mut().slider_width = 110.0;
                ui.add(egui::Slider::new(&mut app.thumb_px, 80.0..=360.0).show_value(false))
                    .on_hover_text("Thumbnail size");
                ui.label(RichText::new(icons::IMAGE).size(15.0).color(p.muted))
                    .on_hover_text("Thumbnail size");
                theme::vsep(ui);
            }
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

/// Applies the commands collected while drawing the current frame.
pub(crate) fn run_cmds(app: &mut DupeApp, cmds: Vec<Cmd>, ctx: &egui::Context) {
    for c in cmds {
        match c {
            Cmd::Toggle(p) => {
                app.results.toggle(&p);
                app.results.mark_dirty();
            }
            Cmd::KeepOnly(gi, p) => {
                if let Some(g) = app.results.groups.get(gi) {
                    selection::keep_only(g, &mut app.results.marked, &p);
                    app.results.mark_dirty();
                }
            }
            Cmd::Open(p) => {
                if let Err(e) = open::that_detached(&p) {
                    app.log.push(format!("Could not open {}: {e}", p.display()));
                }
            }
            Cmd::OpenFolder(p) => {
                if let Some(parent) = p.parent()
                    && let Err(e) = open::that_detached(parent)
                {
                    app.log
                        .push(format!("Could not open {}: {e}", parent.display()));
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
