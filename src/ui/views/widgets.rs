//! Drawing helpers shared by the result views.

use eframe::egui::{self, Align2, Color32, FontId, Rect, Vec2};

use super::Cmd;
use crate::model::{DupGroup, FileEntry, is_decodable_image};
use crate::ui::thumbs::{Thumb, ThumbCache};

/// Double-click to open, right-click for the context menu.
pub(super) fn file_interactions(
    r: &egui::Response,
    g: &DupGroup,
    gi: usize,
    f: &FileEntry,
    cmds: &mut Vec<Cmd>,
) {
    if r.double_clicked() {
        cmds.push(Cmd::Open(f.path.clone()));
    }
    r.context_menu(|ui| {
        if ui
            .button("✔ Keep only this one (mark the rest of the group)")
            .clicked()
        {
            cmds.push(Cmd::KeepOnly(gi, f.path.clone()));
            ui.close();
        }
        if ui.button("Open").clicked() {
            cmds.push(Cmd::Open(f.path.clone()));
            ui.close();
        }
        if ui.button("Show in folder").clicked() {
            cmds.push(Cmd::OpenFolder(f.path.clone()));
            ui.close();
        }
        ui.separator();
        if ui.button("Copy path").clicked() {
            cmds.push(Cmd::CopyText(f.path.display().to_string()));
            ui.close();
        }
        if ui.button("Copy all paths in group").clicked() {
            let s: Vec<String> = g
                .files
                .iter()
                .map(|f| f.path.display().to_string())
                .collect();
            cmds.push(Cmd::CopyText(s.join("\n")));
            ui.close();
        }
    });
}

/// Paints an image thumbnail (or a big file-type icon) centred in `rect`.
pub(super) fn paint_preview(
    ui: &mut egui::Ui,
    rect: Rect,
    f: &FileEntry,
    thumbs: &mut ThumbCache,
    load_px: u32,
) {
    ui.painter()
        .rect_filled(rect, 6.0, ui.visuals().extreme_bg_color);
    if is_decodable_image(&f.path) {
        match thumbs.get(&f.path, load_px) {
            Thumb::Ready(t) => {
                let size = t.size_vec2();
                let scale = (rect.width() / size.x).min(rect.height() / size.y).min(1.0);
                let r = Rect::from_center_size(rect.center(), size * scale);
                egui::Image::new(t).corner_radius(4).paint_at(ui, r);
            }
            Thumb::Loading => {
                egui::Spinner::new()
                    .paint_at(ui, Rect::from_center_size(rect.center(), Vec2::splat(24.0)));
            }
            Thumb::Failed => center_text(ui, rect, "⚠ no preview", 13.0),
        }
    } else {
        center_text(ui, rect, f.kind.icon(), rect.height() * 0.35);
    }
}

fn center_text(ui: &egui::Ui, rect: Rect, text: &str, size: f32) {
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        text,
        FontId::proportional(size),
        ui.visuals().weak_text_color(),
    );
}

/// Painter-only version of a pill badge (for custom-drawn cards).
pub(super) fn paint_pill(ui: &egui::Ui, anchor: egui::Pos2, text: &str, color: Color32) {
    let galley =
        ui.painter()
            .layout_no_wrap(text.to_string(), FontId::proportional(11.0), Color32::WHITE);
    let rect = Rect::from_min_size(anchor, galley.size() + Vec2::new(12.0, 4.0));
    ui.painter()
        .rect_filled(rect, 9.0, color.gamma_multiply(0.92));
    ui.painter()
        .galley(rect.min + Vec2::new(6.0, 2.0), galley, Color32::WHITE);
}
