//! Thumbnail grid. Only groups intersecting the viewport are drawn (virtual scrolling),
//! so it stays fast with thousands of groups.

use eframe::egui::{self, Align2, FontId, Rect, Stroke, StrokeKind, Vec2};

use super::Cmd;
use super::widgets::{file_interactions, group_title, paint_preview, paint_status};
use crate::model::{DupGroup, FileEntry, human_size};
use crate::ui::DupeApp;
use crate::ui::icons;
use crate::ui::results::Results;
use crate::ui::theme::{self, Kind, Palette, Size};
use crate::ui::thumbs::ThumbCache;

const GAP: f32 = 12.0;
const HEADER_H: f32 = 44.0;
const TEXT_H: f32 = 58.0;
const PAD: f32 = 8.0;

pub(super) fn show(app: &mut DupeApp, ui: &mut egui::Ui, cmds: &mut Vec<Cmd>) {
    let thumb = app.thumb_px;
    let card = Vec2::new(thumb + 2.0 * PAD, thumb + PAD + TEXT_H);
    let DupeApp {
        results, thumbs, ..
    } = app;
    let visible = results.visible();
    egui::ScrollArea::vertical()
        .auto_shrink(false)
        .show_viewport(ui, |ui, viewport| {
            let width = ui.available_width().max(card.x);
            let cols = ((width + GAP) / (card.x + GAP)).floor().max(1.0) as usize;
            let heights: Vec<f32> = visible
                .iter()
                .map(|&gi| {
                    HEADER_H
                        + results.groups[gi].files.len().div_ceil(cols) as f32 * (card.y + GAP)
                        + 16.0
                })
                .collect();
            ui.set_height(heights.iter().sum());
            let origin = ui.min_rect().min;
            let mut y = 0.0;
            for (pos, (&gi, &h)) in visible.iter().zip(&heights).enumerate() {
                let top = y;
                y += h;
                if y < viewport.min.y || top > viewport.max.y {
                    continue;
                }
                let g = &results.groups[gi];
                let rect = Rect::from_min_size(origin + Vec2::new(0.0, top), Vec2::new(width, h));
                ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
                    group_header(ui, rect, g, pos, results, cmds);
                    let grid_top = rect.min.y + HEADER_H;
                    for (fi, f) in g.files.iter().enumerate() {
                        let (r, c) = (fi / cols, fi % cols);
                        let min = egui::pos2(
                            rect.min.x + c as f32 * (card.x + GAP),
                            grid_top + r as f32 * (card.y + GAP),
                        );
                        let crect = Rect::from_min_size(min, card);
                        file_card(
                            ui,
                            crect,
                            thumb,
                            g,
                            gi,
                            f,
                            results.is_marked(&f.path),
                            thumbs,
                            cmds,
                        );
                    }
                });
            }
        });
}

fn group_header(
    ui: &mut egui::Ui,
    rect: Rect,
    g: &DupGroup,
    pos: usize,
    results: &Results,
    cmds: &mut Vec<Cmd>,
) {
    let bar = Rect::from_min_size(rect.min, Vec2::new(rect.width(), HEADER_H - 10.0));
    ui.scope_builder(egui::UiBuilder::new().max_rect(bar), |ui| {
        // Right-aligned first so the Compare button stays visible when the title is long.
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let b = theme::button(
                ui,
                Kind::Ghost,
                Size::Sm,
                format!("{}  Compare", icons::COLUMNS),
            );
            if ui.add(b).clicked() {
                cmds.push(Cmd::Compare(pos));
            }
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                group_title(ui, g, results.marked_in_group(g));
            });
        });
    });
}

#[allow(clippy::too_many_arguments)]
fn file_card(
    ui: &mut egui::Ui,
    rect: Rect,
    thumb: f32,
    g: &DupGroup,
    gi: usize,
    f: &FileEntry,
    marked: bool,
    thumbs: &mut ThumbCache,
    cmds: &mut Vec<Cmd>,
) {
    let p = Palette::of(ui);
    let color = if marked { p.remove } else { p.keep };
    let resp = ui.interact(
        rect,
        ui.scope_id().with(("card", &f.path)),
        egui::Sense::click(),
    );
    let fill = if resp.hovered() {
        p.surface2
    } else {
        p.surface
    };
    ui.painter().rect_filled(rect, 10.0, fill);
    let stroke = if marked {
        Stroke::new(2.0, p.remove)
    } else {
        Stroke::new(1.0, p.border)
    };
    ui.painter()
        .rect_stroke(rect, 10.0, stroke, StrokeKind::Inside);

    let img_rect = Rect::from_min_size(rect.min + Vec2::splat(PAD), Vec2::splat(thumb));
    paint_preview(
        ui,
        img_rect,
        f,
        thumbs,
        if thumb > 200.0 { 512 } else { 256 },
    );
    if marked {
        ui.painter()
            .rect_filled(img_rect, 6.0, color.gamma_multiply(0.15));
    }
    paint_status(ui, img_rect.left_top() + Vec2::splat(6.0), marked);

    let chars = (thumb / 6.8) as usize;
    let text_pos = egui::pos2(rect.min.x + PAD + 2.0, img_rect.max.y + 8.0);
    let name_color = if marked { p.remove } else { p.text };
    let painter = ui.painter();
    let medium = FontId::new(13.0, egui::FontFamily::Name("inter-medium".into()));
    painter.text(
        text_pos,
        Align2::LEFT_TOP,
        theme::truncate_left(&f.name(), chars),
        medium,
        name_color,
    );
    let meta = format!(
        "{}{}",
        human_size(f.size),
        f.dims
            .map(|(w, h)| format!("  ·  {w} × {h}"))
            .unwrap_or_default()
    );
    painter.text(
        text_pos + Vec2::new(0.0, 18.0),
        Align2::LEFT_TOP,
        meta,
        FontId::proportional(11.5),
        p.muted,
    );
    let folder = theme::truncate_left(&f.parent_display(), chars + 2);
    painter.text(
        text_pos + Vec2::new(0.0, 33.0),
        Align2::LEFT_TOP,
        folder,
        FontId::proportional(11.0),
        p.muted,
    );

    if resp.clicked() {
        cmds.push(Cmd::Toggle(f.path.clone()));
    }
    let resp = resp
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(format!(
            "{}\nClick to {}",
            f.path.display(),
            if marked { "keep" } else { "mark for removal" }
        ));
    file_interactions(&resp, g, gi, f, cmds);
}
