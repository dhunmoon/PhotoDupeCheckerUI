//! Modal confirmation before touching files, and the activity log window.

use eframe::egui::{self, Align, Layout, Margin, RichText};

use super::DupeApp;
use super::fonts::semibold;
use super::icons;
use super::theme::{self, Kind, Palette, Size, space};
use crate::actions::ActionKind;
use crate::model::human_size;

/// Shows the confirmation dialog for `app.pending`, starting the action on confirm.
pub(crate) fn confirm_action(app: &mut DupeApp, ctx: &egui::Context) {
    let Some(pending) = &mut app.pending else {
        return;
    };
    let kind = pending.kind.clone();
    if kind.needs_keeper() {
        pending.skip_fully_marked = true;
    }
    let (items, skipped) = app.results.build_items(&kind, pending.skip_fully_marked);
    let bytes: u64 = items.iter().map(|i| i.size).sum();
    let mut decision: Option<bool> = None;

    egui::Modal::new(egui::Id::unique("confirm_action")).show(ctx, |ui| {
        let p = Palette::of(ui);
        ui.set_width(460.0);
        let (icon, color) = match kind {
            ActionKind::Trash => (icons::TRASH, p.remove),
            ActionKind::Delete => (icons::X_CIRCLE, p.danger),
            ActionKind::MoveTo { .. } => (icons::FOLDER_OPEN, p.accent),
            ActionKind::CopyTo { .. } => (icons::COPY, p.accent),
            ActionKind::HardLink | ActionKind::SymLink => (icons::LINK, p.accent),
        };
        ui.horizontal(|ui| {
            egui::Frame::new()
                .fill(color.gamma_multiply(0.15))
                .corner_radius(10)
                .inner_margin(Margin::same(8))
                .show(ui, |ui| {
                    ui.label(RichText::new(icon).size(20.0).color(color));
                });
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                ui.label(semibold(kind.label()).size(16.0).color(p.text));
                ui.label(
                    RichText::new("Review before continuing")
                        .size(12.0)
                        .color(p.muted),
                );
            });
        });
        ui.add_space(space::LG);

        egui::Frame::new()
            .fill(p.surface2)
            .corner_radius(8)
            .inner_margin(Margin::same(14))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    theme::metric(ui, items.len().to_string(), "files", None);
                    ui.add_space(space::XL);
                    let caption = if kind.removes_original() {
                        "will be freed"
                    } else {
                        "will be copied"
                    };
                    theme::metric(ui, human_size(bytes), caption, None);
                });
            });
        ui.add_space(space::MD);

        if kind.needs_keeper() {
            theme::hint(
                ui,
                "Each marked file is replaced by a link to the first kept file in its group.",
            );
            if matches!(kind, ActionKind::HardLink) {
                theme::hint(
                    ui,
                    "Hard links only work when both files are on the same drive.",
                );
            }
            #[cfg(windows)]
            if matches!(kind, ActionKind::SymLink) {
                theme::hint(
                    ui,
                    "On Windows, symbolic links need Developer Mode or administrator rights.",
                );
            }
        } else if kind.removes_original() {
            ui.checkbox(
                &mut pending.skip_fully_marked,
                "Skip groups where every copy is marked (recommended)",
            );
        }
        if kind.vacates_path() {
            ui.checkbox(
                &mut pending.remove_empty_dirs,
                "Remove folders that become empty",
            )
            .on_hover_text(
                "Only folders inside the scanned folders; the scanned folders themselves are kept.",
            );
        }
        if skipped > 0 {
            ui.label(
                RichText::new(format!(
                    "{}  {skipped} group(s) will be skipped because no copy would be kept.",
                    icons::WARNING
                ))
                .size(12.5)
                .color(p.warn),
            );
        }
        if kind.is_destructive() {
            ui.label(
                semibold(format!("{}  This cannot be undone.", icons::WARNING))
                    .size(12.5)
                    .color(p.remove),
            );
        }
        theme::hint(
            ui,
            "Files that changed since the scan are skipped automatically.",
        );

        ui.add_space(space::LG);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let k = if kind.is_destructive() || kind == ActionKind::Trash {
                Kind::Danger
            } else {
                Kind::Primary
            };
            let confirm = theme::button(ui, k, Size::Md, format!("Confirm ({})", items.len()))
                .min_size(egui::vec2(120.0, 32.0));
            if ui.add_enabled(!items.is_empty(), confirm).clicked() {
                decision = Some(true);
            }
            let cancel = theme::button(ui, Kind::Secondary, Size::Md, "Cancel")
                .min_size(egui::vec2(90.0, 32.0));
            if ui.add(cancel).clicked() || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                decision = Some(false);
            }
        });
    });

    match decision {
        Some(true) => {
            if let Some(p) = app.pending.take() {
                app.start_action(p, ctx);
            }
        }
        Some(false) => app.pending = None,
        None => {}
    }
}

pub(crate) fn log_window(app: &mut DupeApp, ctx: &egui::Context) {
    let mut open = app.show_log;
    egui::Window::new(semibold(format!("{}  Activity log", icons::TERMINAL)))
        .open(&mut open)
        .default_size([680.0, 320.0])
        .show(ctx, |ui| {
            let p = Palette::of(ui);
            ui.horizontal(|ui| {
                if ui
                    .add(theme::button(
                        ui,
                        Kind::Secondary,
                        Size::Sm,
                        format!("{}  Copy all", icons::CLIPBOARD_TEXT),
                    ))
                    .clicked()
                {
                    ui.ctx().copy_text(app.log.join("\n"));
                }
                if ui
                    .add(theme::button(
                        ui,
                        Kind::Ghost,
                        Size::Sm,
                        format!("{}  Clear", icons::ERASER),
                    ))
                    .clicked()
                {
                    app.log.clear();
                }
            });
            ui.add_space(space::XS);
            egui::Frame::new()
                .fill(p.bg)
                .corner_radius(8)
                .inner_margin(Margin::same(10))
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .stick_to_bottom(true)
                        .auto_shrink(false)
                        .show(ui, |ui| {
                            if app.log.is_empty() {
                                ui.label(RichText::new("Nothing logged yet.").color(p.muted));
                            }
                            for l in &app.log {
                                let t = l.trim_start();
                                let color = if t.starts_with('✖') {
                                    p.remove
                                } else if t.starts_with('⚠') {
                                    p.warn
                                } else {
                                    p.text
                                };
                                ui.label(RichText::new(l).monospace().color(color));
                            }
                        });
                });
        });
    app.show_log = open;
}
