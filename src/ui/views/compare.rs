//! Side-by-side comparison of one group at a time, with the best value of each property
//! highlighted (★) to help decide which copy to keep.

use eframe::egui::{self, RichText, Stroke, Vec2};

use super::Cmd;
use super::widgets::{file_interactions, paint_pill, paint_preview};
use crate::model::{FolderRole, extension_lower, format_time, human_size};
use crate::ui::DupeApp;
use crate::ui::theme::{self, ACCENT, KEEP, REMOVE, SOURCE, TARGET};

pub(super) fn show(app: &mut DupeApp, ui: &mut egui::Ui, cmds: &mut Vec<Cmd>) {
    let n = app.results.visible().len();
    navigation(ui, &mut app.compare_pos, n);

    let gi = app.results.visible()[app.compare_pos];
    let preview = (app.thumb_px * 1.8).max(220.0);
    let load_px = if preview > 512.0 { 1024 } else { 512 };
    let DupeApp {
        results, thumbs, ..
    } = app;
    let g = &results.groups[gi];

    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!(
                "{} #{} · {} copies",
                g.main_kind().icon(),
                g.id,
                g.files.len()
            ))
            .size(16.0)
            .strong(),
        );
        ui.label(RichText::new(&g.label).weak());
        theme::badge(ui, format!("−{}", human_size(g.wasted())), ACCENT);
    });
    ui.add_space(6.0);

    let max_px = g.files.iter().map(|f| f.pixels()).max().unwrap_or(0);
    let max_size = g.files.iter().map(|f| f.size).max().unwrap_or(0);
    let newest = g.files.iter().filter_map(|f| f.modified).max();
    let oldest = g.files.iter().filter_map(|f| f.modified).min();
    let star = |b: bool| if b { "  ★" } else { "" };

    egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
        ui.horizontal_top(|ui| {
            for f in &g.files {
                let marked = results.is_marked(&f.path);
                let color = if marked { REMOVE } else { KEEP };
                theme::card(ui)
                    .stroke(Stroke::new(2.0, color.gamma_multiply(0.85)))
                    .show(ui, |ui| {
                        ui.set_width(preview);
                        ui.vertical(|ui| {
                            let (rect, resp) =
                                ui.allocate_exact_size(Vec2::splat(preview), egui::Sense::click());
                            paint_preview(ui, rect, f, thumbs, load_px);
                            paint_pill(
                                ui,
                                rect.left_top() + Vec2::splat(8.0),
                                if marked { "✖ REMOVE" } else { "✔ KEEP" },
                                color,
                            );
                            if resp.clicked() {
                                cmds.push(Cmd::Toggle(f.path.clone()));
                            }
                            file_interactions(
                                &resp.on_hover_text("Click to switch keep / remove"),
                                g,
                                gi,
                                f,
                                cmds,
                            );

                            ui.add_space(4.0);
                            ui.add(
                                egui::Label::new(RichText::new(f.name()).strong().size(15.0))
                                    .wrap(),
                            );
                            ui.add(
                                egui::Label::new(RichText::new(f.parent_display()).small().weak())
                                    .wrap(),
                            );
                            ui.add_space(4.0);
                            egui::Grid::new(("cmp", &f.path))
                                .num_columns(2)
                                .spacing([12.0, 4.0])
                                .show(ui, |ui| {
                                    let k = |ui: &mut egui::Ui, s: &str| {
                                        ui.label(RichText::new(s).small().weak())
                                    };
                                    k(ui, "Size");
                                    ui.label(format!(
                                        "{}{}",
                                        human_size(f.size),
                                        star(f.size == max_size)
                                    ));
                                    ui.end_row();
                                    k(ui, "Dimensions");
                                    ui.label(match f.dims {
                                        Some((w, h)) => format!(
                                            "{w}×{h}{}",
                                            star(f.pixels() == max_px && max_px > 0)
                                        ),
                                        None => "–".into(),
                                    });
                                    ui.end_row();
                                    k(ui, "Format");
                                    ui.label(extension_lower(&f.path).to_uppercase());
                                    ui.end_row();
                                    k(ui, "Modified");
                                    let tag = if f.modified.is_some()
                                        && f.modified == newest
                                        && newest != oldest
                                    {
                                        "  (newest)"
                                    } else if f.modified.is_some()
                                        && f.modified == oldest
                                        && newest != oldest
                                    {
                                        "  (oldest)"
                                    } else {
                                        ""
                                    };
                                    ui.label(format!("{}{tag}", format_time(f.modified)));
                                    ui.end_row();
                                    if results.is_source_vs_target() {
                                        k(ui, "Role");
                                        let c = if f.role == FolderRole::Source {
                                            SOURCE
                                        } else {
                                            TARGET
                                        };
                                        theme::badge(ui, f.role.label(), c);
                                        ui.end_row();
                                    }
                                    if let Some(d) = f.distance {
                                        k(ui, "Difference");
                                        ui.label(if d == 0 {
                                            "0 (reference)".to_string()
                                        } else {
                                            d.to_string()
                                        });
                                        ui.end_row();
                                    }
                                });
                            ui.add_space(6.0);
                            ui.horizontal(|ui| {
                                if ui
                                    .add(theme::primary_button("✔ Keep only this"))
                                    .on_hover_text("Mark every other copy in this group")
                                    .clicked()
                                {
                                    cmds.push(Cmd::KeepOnly(gi, f.path.clone()));
                                }
                                if theme::mark_toggle(ui, marked) {
                                    cmds.push(Cmd::Toggle(f.path.clone()));
                                }
                            });
                            ui.horizontal(|ui| {
                                if ui.small_button("Open").clicked() {
                                    cmds.push(Cmd::Open(f.path.clone()));
                                }
                                if ui.small_button("Show in folder").clicked() {
                                    cmds.push(Cmd::OpenFolder(f.path.clone()));
                                }
                            });
                        });
                    });
            }
        });
    });
}

fn navigation(ui: &mut egui::Ui, pos: &mut usize, n: usize) {
    ui.horizontal(|ui| {
        if ui
            .add_enabled(*pos > 0, egui::Button::new("◀ Previous"))
            .clicked()
        {
            *pos -= 1;
        }
        ui.label(RichText::new(format!("Group {} of {}", *pos + 1, n)).strong());
        if ui
            .add_enabled(*pos + 1 < n, egui::Button::new("Next ▶"))
            .clicked()
        {
            *pos += 1;
        }
        ui.label(RichText::new("Tip: ← / → keys also work").small().weak());
    });
    let no_text_focus = !ui.ctx().egui_wants_keyboard_input();
    ui.input(|i| {
        if no_text_focus && i.key_pressed(egui::Key::ArrowRight) && *pos + 1 < n {
            *pos += 1;
        }
        if no_text_focus && i.key_pressed(egui::Key::ArrowLeft) && *pos > 0 {
            *pos -= 1;
        }
    });
    ui.separator();
}
