//! Modal confirmation before touching files, and the log window.

use eframe::egui::{self, Align, Layout, RichText};

use super::DupeApp;
use super::theme::{self, REMOVE, WARN};
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
        ui.set_width(500.0);
        let icon = match kind {
            ActionKind::Trash => "🗑",
            ActionKind::Delete => "❌",
            ActionKind::MoveTo { .. } => "📂",
            ActionKind::CopyTo { .. } => "📋",
            ActionKind::HardLink | ActionKind::SymLink => "🔗",
        };
        ui.label(
            RichText::new(format!("{icon}  {}", kind.label()))
                .size(18.0)
                .strong(),
        );
        ui.add_space(8.0);

        theme::card(ui).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                theme::stat(ui, items.len().to_string(), "files", None);
                ui.add_space(20.0);
                let caption = if kind.removes_original() {
                    "will be freed"
                } else {
                    "will be copied"
                };
                theme::stat(ui, human_size(bytes), caption, None);
            });
        });
        ui.add_space(6.0);

        if kind.needs_keeper() {
            ui.label("Each marked file is replaced by a link to the first kept file in its group.");
            if matches!(kind, ActionKind::HardLink) {
                ui.label(
                    RichText::new("Hard links only work when both files are on the same drive.")
                        .small()
                        .weak(),
                );
            }
            #[cfg(windows)]
            if matches!(kind, ActionKind::SymLink) {
                ui.label(
                    RichText::new(
                        "On Windows, symbolic links need Developer Mode or administrator rights.",
                    )
                    .small()
                    .weak(),
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
                    "⚠ {skipped} group(s) will be skipped because no copy would be kept."
                ))
                .color(WARN),
            );
        }
        if kind.is_destructive() {
            ui.label(
                RichText::new("⚠ This cannot be undone.")
                    .color(REMOVE)
                    .strong(),
            );
        }
        ui.label(
            RichText::new("Files that changed since the scan are skipped automatically.")
                .small()
                .weak(),
        );

        ui.add_space(10.0);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let label = format!("Confirm ({} files)", items.len());
            let btn = if kind.is_destructive() || kind == ActionKind::Trash {
                theme::danger_button(label)
            } else {
                theme::primary_button(label)
            };
            if ui
                .add_enabled(!items.is_empty(), btn.min_size(egui::vec2(140.0, 32.0)))
                .clicked()
            {
                decision = Some(true);
            }
            if ui
                .add(egui::Button::new("Cancel").min_size(egui::vec2(90.0, 32.0)))
                .clicked()
                || ui.input(|i| i.key_pressed(egui::Key::Escape))
            {
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
    egui::Window::new("📜 Log")
        .open(&mut open)
        .default_size([680.0, 320.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Clear").clicked() {
                    app.log.clear();
                }
                if ui.button("Copy all").clicked() {
                    ui.ctx().copy_text(app.log.join("\n"));
                }
            });
            ui.separator();
            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .auto_shrink(false)
                .show(ui, |ui| {
                    if app.log.is_empty() {
                        ui.label(RichText::new("Nothing logged yet.").weak());
                    }
                    for l in &app.log {
                        let mut t = RichText::new(l).monospace();
                        if l.trim_start().starts_with('✖') {
                            t = t.color(REMOVE);
                        } else if l.trim_start().starts_with('⚠') {
                            t = t.color(WARN);
                        }
                        ui.label(t);
                    }
                });
        });
    app.show_log = open;
}
