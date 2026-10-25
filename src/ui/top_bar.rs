//! Header: logo, app name, headline numbers and global toggles.

use eframe::egui::{self, Align, Layout, Margin, Vec2};

use super::DupeApp;
use super::fonts::semibold;
use super::icons;
use super::theme::{self, Palette, space};
use crate::model::human_size;

pub(crate) fn show(app: &mut DupeApp, ui: &mut egui::Ui) {
    let p = Palette::of(ui);
    egui::Panel::top("top_bar")
        .exact_size(56.0)
        .show_separator_line(true)
        .frame(theme::panel_frame(ui, p.surface, Margin::symmetric(16, 0)))
        .show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                if let Some(t) = &app.logo {
                    ui.add(egui::Image::new(t).fit_to_exact_size(Vec2::splat(28.0)));
                }
                ui.label(semibold("Dupe Finder").size(16.0).color(p.text));

                if !app.results.is_empty() {
                    ui.add_space(space::XL);
                    theme::vsep(ui);
                    ui.add_space(space::LG);
                    let (groups, files, wasted) = app.results.totals();
                    let marked = app.results.stats();
                    let metrics = [
                        (groups.to_string(), "Duplicate groups", None),
                        (files.to_string(), "Files involved", None),
                        (human_size(wasted), "Can be freed", Some(p.accent)),
                        (
                            marked.files.to_string(),
                            "Marked",
                            (marked.files > 0).then_some(p.remove),
                        ),
                    ];
                    for (i, (value, caption, color)) in metrics.into_iter().enumerate() {
                        if i > 0 {
                            ui.add_space(space::LG);
                        }
                        theme::metric(ui, value, caption, color);
                    }
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let dark = ui.visuals().dark_mode;
                    let (icon, tip) = if dark {
                        (icons::SUN, "Switch to light mode")
                    } else {
                        (icons::MOON, "Switch to dark mode")
                    };
                    if theme::icon_button(ui, icon, tip).clicked() {
                        ui.ctx().set_theme(if dark {
                            egui::Theme::Light
                        } else {
                            egui::Theme::Dark
                        });
                    }
                    let resp = theme::icon_button(ui, icons::TERMINAL, "Activity log");
                    if !app.log.is_empty() {
                        let c = resp.rect.right_top() + egui::vec2(-7.0, 7.0);
                        ui.painter().circle_filled(c, 3.5, p.accent);
                    }
                    if resp.clicked() {
                        app.show_log = !app.show_log;
                    }
                });
            });
        });
}
