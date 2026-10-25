//! Side-by-side comparison of one group at a time, with the best value of each property
//! starred to help decide which copy to keep.

use eframe::egui::{self, RichText, Stroke, Vec2};

use super::Cmd;
use super::widgets::{file_interactions, group_title, paint_preview, paint_status};
use crate::model::{FolderRole, extension_lower, format_time, human_size};
use crate::ui::DupeApp;
use crate::ui::fonts::{medium, semibold};
use crate::ui::icons;
use crate::ui::theme::{self, Kind, Palette, Size, space};

pub(super) fn show(app: &mut DupeApp, ui: &mut egui::Ui, cmds: &mut Vec<Cmd>) {
    let n = app.results.visible().len();
    navigation(ui, &mut app.compare_pos, n);

    let gi = app.results.visible()[app.compare_pos];
    let preview = (app.thumb_px * 1.6).max(220.0);
    let load_px = if preview > 512.0 { 1024 } else { 512 };
    let DupeApp {
        results, thumbs, ..
    } = app;
    let g = &results.groups[gi];
    let p = Palette::of(ui);

    ui.horizontal(|ui| group_title(ui, g, results.marked_in_group(g)));
    ui.add_space(space::SM);

    let max_px = g.files.iter().map(|f| f.pixels()).max().unwrap_or(0);
    let max_size = g.files.iter().map(|f| f.size).max().unwrap_or(0);
    let newest = g.files.iter().filter_map(|f| f.modified).max();
    let oldest = g.files.iter().filter_map(|f| f.modified).min();

    egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = space::MD;
            for f in &g.files {
                let marked = results.is_marked(&f.path);
                let stroke = if marked {
                    Stroke::new(2.0, p.remove)
                } else {
                    Stroke::new(1.0, p.border)
                };
                theme::card(ui)
                    .stroke(stroke)
                    .inner_margin(egui::Margin::same(12))
                    .show(ui, |ui| {
                        ui.set_width(preview);
                        ui.vertical(|ui| {
                            let (rect, resp) =
                                ui.allocate_exact_size(Vec2::splat(preview), egui::Sense::click());
                            paint_preview(ui, rect, f, thumbs, load_px);
                            paint_status(ui, rect.left_top() + Vec2::splat(8.0), marked);
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

                            ui.add_space(space::SM);
                            ui.add(
                                egui::Label::new(semibold(f.name()).size(14.5).color(p.text))
                                    .wrap(),
                            );
                            ui.add(
                                egui::Label::new(
                                    RichText::new(f.parent_display()).size(12.0).color(p.muted),
                                )
                                .wrap(),
                            );
                            ui.add_space(space::SM);

                            egui::Grid::new(("cmp", &f.path))
                                .num_columns(2)
                                .spacing([space::LG, 6.0])
                                .show(ui, |ui| {
                                    let key = |ui: &mut egui::Ui, s: &str| {
                                        ui.label(RichText::new(s).size(12.0).color(p.muted))
                                    };
                                    let val = |ui: &mut egui::Ui, s: String, best: bool| {
                                        ui.horizontal(|ui| {
                                            ui.spacing_mut().item_spacing.x = 4.0;
                                            ui.label(medium(s).color(p.text));
                                            if best {
                                                ui.label(
                                                    crate::ui::fonts::icon_fill(icons::STAR)
                                                        .size(12.0)
                                                        .color(p.warn),
                                                )
                                                .on_hover_text("Best in group");
                                            }
                                        });
                                    };
                                    key(ui, "Size");
                                    val(
                                        ui,
                                        human_size(f.size),
                                        g.files.len() > 1 && f.size == max_size,
                                    );
                                    ui.end_row();
                                    key(ui, "Dimensions");
                                    match f.dims {
                                        Some((w, h)) => val(
                                            ui,
                                            format!("{w} × {h}"),
                                            f.pixels() == max_px && max_px > 0,
                                        ),
                                        None => val(ui, "—".into(), false),
                                    }
                                    ui.end_row();
                                    key(ui, "Format");
                                    val(ui, extension_lower(&f.path).to_uppercase(), false);
                                    ui.end_row();
                                    key(ui, "Modified");
                                    let tag = if newest == oldest || f.modified.is_none() {
                                        ""
                                    } else if f.modified == newest {
                                        "  (newest)"
                                    } else if f.modified == oldest {
                                        "  (oldest)"
                                    } else {
                                        ""
                                    };
                                    val(ui, format!("{}{tag}", format_time(f.modified)), false);
                                    ui.end_row();
                                    if results.is_source_vs_target() {
                                        key(ui, "Role");
                                        let c = if f.role == FolderRole::Source {
                                            p.source
                                        } else {
                                            p.target
                                        };
                                        theme::badge(ui, f.role.label(), c);
                                        ui.end_row();
                                    }
                                    if let Some(d) = f.distance {
                                        key(ui, "Difference");
                                        val(
                                            ui,
                                            if d == 0 {
                                                "0 (reference)".to_string()
                                            } else {
                                                d.to_string()
                                            },
                                            false,
                                        );
                                        ui.end_row();
                                    }
                                });

                            ui.add_space(space::MD);
                            ui.horizontal(|ui| {
                                let keep = theme::button(
                                    ui,
                                    Kind::Primary,
                                    Size::Sm,
                                    format!("{}  Keep only this", icons::CHECK),
                                );
                                if ui
                                    .add(keep)
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
                                if ui
                                    .add(theme::button(
                                        ui,
                                        Kind::Ghost,
                                        Size::Sm,
                                        format!("{}  Open", icons::ARROW_SQUARE_OUT),
                                    ))
                                    .clicked()
                                {
                                    cmds.push(Cmd::Open(f.path.clone()));
                                }
                                if ui
                                    .add(theme::button(
                                        ui,
                                        Kind::Ghost,
                                        Size::Sm,
                                        format!("{}  Show in folder", icons::FOLDER_OPEN),
                                    ))
                                    .clicked()
                                {
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
    let p = Palette::of(ui);
    ui.horizontal(|ui| {
        let prev = theme::button(
            ui,
            Kind::Secondary,
            Size::Sm,
            format!("{}  Previous", icons::CARET_LEFT),
        );
        if ui.add_enabled(*pos > 0, prev).clicked() {
            *pos -= 1;
        }
        ui.label(medium(format!("{} of {}", *pos + 1, n)).color(p.text));
        let next = theme::button(
            ui,
            Kind::Secondary,
            Size::Sm,
            format!("Next  {}", icons::CARET_RIGHT),
        );
        if ui.add_enabled(*pos + 1 < n, next).clicked() {
            *pos += 1;
        }
        ui.label(
            RichText::new("Use ← / → to move between groups")
                .size(12.0)
                .color(p.muted),
        );
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
    ui.add_space(space::SM);
}
