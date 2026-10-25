//! Drawing helpers shared by the result views.

use eframe::egui::{self, Align2, Color32, FontId, Rect, RichText, Vec2};

use super::Cmd;
use crate::model::{DupGroup, FileEntry, human_size, is_decodable_image};
use crate::ui::fonts::semibold;
use crate::ui::icons;
use crate::ui::theme::{self, Palette};
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
        ui.set_min_width(260.0);
        if theme::menu_item(ui, icons::CHECK_CIRCLE, "Keep only this one").clicked() {
            cmds.push(Cmd::KeepOnly(gi, f.path.clone()));
            ui.close();
        }
        ui.separator();
        if theme::menu_item(ui, icons::ARROW_SQUARE_OUT, "Open").clicked() {
            cmds.push(Cmd::Open(f.path.clone()));
            ui.close();
        }
        if theme::menu_item(ui, icons::FOLDER_OPEN, "Show in folder").clicked() {
            cmds.push(Cmd::OpenFolder(f.path.clone()));
            ui.close();
        }
        ui.separator();
        if theme::menu_item(ui, icons::COPY, "Copy path").clicked() {
            cmds.push(Cmd::CopyText(f.path.display().to_string()));
            ui.close();
        }
        if theme::menu_item(ui, icons::CLIPBOARD_TEXT, "Copy all paths in group").clicked() {
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

/// Group heading used by the grid and side-by-side views.
pub(super) fn group_title(ui: &mut egui::Ui, g: &DupGroup, marked: usize) {
    let p = Palette::of(ui);
    ui.label(
        RichText::new(icons::for_kind(g.main_kind()))
            .size(16.0)
            .color(p.muted),
    );
    ui.label(semibold(format!("Group {}", g.id)).color(p.text));
    ui.label(RichText::new(format!("· {} copies · {}", g.files.len(), g.label)).color(p.muted));
    theme::badge(ui, format!("{} freed", human_size(g.wasted())), p.accent)
        .on_hover_text("Space freed by keeping only one copy");
    if marked > 0 {
        theme::badge(ui, format!("{marked} marked"), p.remove);
    }
}

/// Paints an image thumbnail (or a file-type icon) centred in `rect`.
pub(super) fn paint_preview(
    ui: &mut egui::Ui,
    rect: Rect,
    f: &FileEntry,
    thumbs: &mut ThumbCache,
    load_px: u32,
) {
    let p = Palette::of(ui);
    ui.painter().rect_filled(rect, 6.0, p.bg);
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
                    .color(p.muted)
                    .paint_at(ui, Rect::from_center_size(rect.center(), Vec2::splat(22.0)));
            }
            Thumb::Failed => center_icon(ui, rect, icons::WARNING, 24.0, p.muted),
        }
    } else {
        center_icon(
            ui,
            rect,
            icons::for_kind(f.kind),
            rect.height() * 0.3,
            p.muted,
        );
    }
}

fn center_icon(ui: &egui::Ui, rect: Rect, icon: &str, size: f32, color: Color32) {
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        icon,
        FontId::proportional(size),
        color,
    );
}

/// Painter-only keep/remove pill for custom-drawn cards, anchored at its top-left corner.
pub(super) fn paint_status(ui: &egui::Ui, anchor: egui::Pos2, marked: bool) {
    let rect = Rect::from_min_size(anchor, theme::STATUS_PILL_SOLID);
    theme::paint_status_pill(ui, rect, marked, true, false);
}
