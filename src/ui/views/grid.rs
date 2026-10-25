//! Thumbnail grid. Only groups intersecting the viewport are drawn (virtual scrolling),
//! so it stays fast with thousands of groups.

use eframe::egui::{self, Align2, FontId, Rect, RichText, Stroke, StrokeKind, Vec2};

use super::Cmd;
use super::widgets::{file_interactions, paint_pill, paint_preview};
use crate::model::{DupGroup, FileEntry, human_size};
use crate::ui::DupeApp;
use crate::ui::results::Results;
use crate::ui::theme::{self, ACCENT, KEEP, REMOVE};
use crate::ui::thumbs::ThumbCache;

const GAP: f32 = 10.0;
const HEADER_H: f32 = 40.0;
const TEXT_H: f32 = 56.0;

pub(super) fn show(app: &mut DupeApp, ui: &mut egui::Ui, cmds: &mut Vec<Cmd>) {
    let thumb = app.thumb_px;
    let card = Vec2::new(thumb + 12.0, thumb + 12.0 + TEXT_H);
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
                        + 14.0
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
    let bar = Rect::from_min_size(rect.min, Vec2::new(rect.width(), HEADER_H - 8.0));
    ui.painter()
        .rect_filled(bar, 8.0, ui.visuals().widgets.inactive.weak_bg_fill);
    ui.scope_builder(
        egui::UiBuilder::new().max_rect(bar.shrink2(Vec2::new(10.0, 0.0))),
        |ui| {
            ui.horizontal_centered(|ui| {
                ui.label(RichText::new(g.main_kind().icon()).size(16.0));
                ui.label(RichText::new(format!("#{} · {} copies", g.id, g.files.len())).strong());
                ui.label(RichText::new(&g.label).weak());
                theme::badge(ui, format!("−{}", human_size(g.wasted())), ACCENT);
                let n = results.marked_in_group(g);
                if n > 0 {
                    ui.label(RichText::new(format!("{n} marked")).small().color(REMOVE));
                }
                if ui.small_button("⚖ Compare").clicked() {
                    cmds.push(Cmd::Compare(pos));
                }
            });
        },
    );
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
    let color = if marked { REMOVE } else { KEEP };
    let resp = ui.interact(
        rect,
        ui.scope_id().with(("card", &f.path)),
        egui::Sense::click(),
    );
    let fill = if resp.hovered() {
        ui.visuals().widgets.hovered.weak_bg_fill
    } else {
        ui.visuals().faint_bg_color
    };
    ui.painter().rect_filled(rect, 10.0, fill);
    let stroke_w = if marked { 2.5 } else { 1.5 };
    ui.painter().rect_stroke(
        rect,
        10.0,
        Stroke::new(stroke_w, color.gamma_multiply(0.85)),
        StrokeKind::Inside,
    );

    let img_rect = Rect::from_min_size(rect.min + Vec2::splat(6.0), Vec2::splat(thumb));
    paint_preview(
        ui,
        img_rect,
        f,
        thumbs,
        if thumb > 200.0 { 512 } else { 256 },
    );
    if marked {
        ui.painter()
            .rect_filled(img_rect, 6.0, REMOVE.gamma_multiply(0.18));
    }
    paint_pill(
        ui,
        img_rect.left_top() + Vec2::splat(6.0),
        if marked { "✖ REMOVE" } else { "✔ KEEP" },
        color,
    );

    let chars = (thumb / 6.8) as usize;
    let text_pos = egui::pos2(rect.min.x + 8.0, img_rect.max.y + 6.0);
    let weak = ui.visuals().weak_text_color();
    let name_color = if marked {
        REMOVE
    } else {
        ui.visuals().strong_text_color()
    };
    let p = ui.painter();
    p.text(
        text_pos,
        Align2::LEFT_TOP,
        theme::truncate_left(&f.name(), chars),
        FontId::proportional(13.0),
        name_color,
    );
    let meta = format!(
        "{}{}",
        human_size(f.size),
        f.dims
            .map(|(w, h)| format!(" · {w}×{h}"))
            .unwrap_or_default()
    );
    p.text(
        text_pos + Vec2::new(0.0, 17.0),
        Align2::LEFT_TOP,
        meta,
        FontId::proportional(11.5),
        weak,
    );
    let folder = theme::truncate_left(&f.parent_display(), chars + 2);
    p.text(
        text_pos + Vec2::new(0.0, 32.0),
        Align2::LEFT_TOP,
        folder,
        FontId::proportional(10.5),
        weak,
    );

    if resp.clicked() {
        cmds.push(Cmd::Toggle(f.path.clone()));
    }
    let resp = resp.on_hover_text(format!(
        "{}\nClick to {}",
        f.path.display(),
        if marked { "keep" } else { "mark for removal" }
    ));
    file_interactions(&resp, g, gi, f, cmds);
}
