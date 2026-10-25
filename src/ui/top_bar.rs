//! Title, headline numbers and global toggles.

use eframe::egui::{self, Align, Layout, Margin, RichText};

use super::DupeApp;
use super::theme::{self, ACCENT, REMOVE};
use crate::model::human_size;

pub(crate) fn show(app: &mut DupeApp, ui: &mut egui::Ui) {
    egui::Panel::top("top_bar")
        .frame(egui::Frame::side_top_panel(ui.style()).inner_margin(Margin::symmetric(14, 8)))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("🔍").size(26.0).color(ACCENT));
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    ui.label(RichText::new("Dupe Finder").size(19.0).strong());
                    ui.label(
                        RichText::new("Find and clean up duplicate files")
                            .small()
                            .weak(),
                    );
                });

                if !app.results.is_empty() {
                    ui.add_space(24.0);
                    ui.separator();
                    let (groups, files, wasted) = app.results.totals();
                    let marked = app.results.stats();
                    theme::stat(ui, groups.to_string(), "duplicate groups", None);
                    ui.add_space(12.0);
                    theme::stat(ui, files.to_string(), "files involved", None);
                    ui.add_space(12.0);
                    theme::stat(ui, human_size(wasted), "can be freed", Some(ACCENT));
                    ui.add_space(12.0);
                    theme::stat(
                        ui,
                        marked.files.to_string(),
                        "marked for removal",
                        (marked.files > 0).then_some(REMOVE),
                    );
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    egui::widgets::global_theme_preference_buttons(ui);
                    let n_log = app.log.len();
                    let label = if n_log > 0 {
                        format!("📜 Log ({n_log})")
                    } else {
                        "📜 Log".into()
                    };
                    ui.toggle_value(&mut app.show_log, label)
                        .on_hover_text("Scan messages, unreadable files and action results");
                });
            });
        });
}
