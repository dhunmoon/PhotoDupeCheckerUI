//! Table view: one header row per group followed by its files.

use eframe::egui::{self, Margin, RichText, Sense, Stroke, Vec2};
use egui_extras::{Column, TableBuilder, TableRow};

use super::Cmd;
use super::widgets::file_interactions;
use crate::model::{FolderRole, Method, format_time, human_size, is_decodable_image};
use crate::ui::DupeApp;
use crate::ui::fonts::{medium, semibold};
use crate::ui::icons;
use crate::ui::results::Row;
use crate::ui::theme::{self, Kind, Palette, Size};
use crate::ui::thumbs::Thumb;

const GROUP_ROW_H: f32 = 40.0;
const FILE_ROW_H: f32 = 40.0;

pub(super) fn show(app: &mut DupeApp, ui: &mut egui::Ui, cmds: &mut Vec<Cmd>) {
    let p = Palette::of(ui);
    let svt = app.results.is_source_vs_target();
    let similar = app.results.method() == Some(Method::SimilarImage);
    let DupeApp {
        results, thumbs, ..
    } = app;
    let rows = results.rows();

    egui::Frame::new()
        .fill(p.surface)
        .stroke(Stroke::new(1.0, p.border))
        .corner_radius(theme::RADIUS_LG)
        .inner_margin(Margin::symmetric(4, 4))
        .show(ui, |ui| {
            let table_rect = ui.max_rect();
            // Narrow windows drop the Dimensions / Modified columns instead of overflowing.
            let wide = ui.available_width() > 980.0;
            let mut table = TableBuilder::new(ui)
                .striped(false)
                .resizable(false)
                .sense(Sense::hover())
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::exact(96.0))
                .column(Column::exact(40.0))
                .column(Column::exact(if wide { 240.0 } else { 160.0 }).clip(true))
                .column(Column::remainder().at_least(90.0).clip(true))
                .column(Column::exact(90.0));
            if wide {
                table = table
                    .column(Column::exact(100.0))
                    .column(Column::exact(130.0));
            }
            if svt {
                table = table.column(Column::exact(84.0));
            }
            if similar {
                table = table.column(Column::exact(56.0));
            }
            table
                .header(30.0, |mut h| {
                    let title = |h: &mut TableRow, t: &str, tip: Option<&str>| {
                        h.col(|ui| {
                            let r = ui.label(medium(t).size(11.5).color(p.muted));
                            if let Some(tip) = tip {
                                r.on_hover_text(tip);
                            }
                        });
                    };
                    for t in ["  Status", "", "Name", "Folder", "Size"] {
                        title(&mut h, t, None);
                    }
                    if wide {
                        title(&mut h, "Dimensions", None);
                        title(&mut h, "Modified", None);
                    }
                    if svt {
                        title(&mut h, "Role", None);
                    }
                    if similar {
                        title(
                            &mut h,
                            "Diff",
                            Some("Visual difference to the group's best image (0 = identical)"),
                        );
                    }
                })
                .body(|body| {
                    let heights = rows.iter().map(|r| match r {
                        Row::Group(_) => GROUP_ROW_H,
                        Row::File(..) => FILE_ROW_H,
                    });
                    body.heterogeneous_rows(heights, |mut row| match rows[row.index()] {
                        Row::Group(gi) => {
                            let g = &results.groups[gi];
                            let n_marked = results.marked_in_group(g);
                            row.set_overline(true);
                            let cell = |row: &mut TableRow, add: &mut dyn FnMut(&mut egui::Ui)| {
                                row.col(|ui| {
                                    paint_row_bg(ui, table_rect, p.surface2);
                                    add(ui);
                                });
                            };
                            cell(&mut row, &mut |ui| {
                                ui.add_space(8.0);
                                ui.label(
                                    RichText::new(icons::for_kind(g.main_kind()))
                                        .size(16.0)
                                        .color(p.muted),
                                );
                                ui.label(semibold(format!("Group {}", g.id)).color(p.text));
                            });
                            cell(&mut row, &mut |_| {});
                            cell(&mut row, &mut |ui| {
                                ui.label(medium(format!("{} copies", g.files.len())).color(p.text));
                                if n_marked > 0 {
                                    theme::badge(ui, format!("{n_marked} marked"), p.remove);
                                }
                            });
                            cell(&mut row, &mut |ui| {
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        let b = theme::button(
                                            ui,
                                            Kind::Ghost,
                                            Size::Sm,
                                            format!("{}  Compare", icons::COLUMNS),
                                        );
                                        if ui
                                            .add(b)
                                            .on_hover_text("View this group side by side")
                                            .clicked()
                                            && let Some(pos) =
                                                results.visible().iter().position(|&v| v == gi)
                                        {
                                            cmds.push(Cmd::Compare(pos));
                                        }
                                        ui.with_layout(
                                            egui::Layout::left_to_right(egui::Align::Center),
                                            |ui| {
                                                ui.label(RichText::new(&g.label).color(p.muted));
                                            },
                                        );
                                    },
                                );
                            });
                            cell(&mut row, &mut |ui| {
                                theme::badge(
                                    ui,
                                    format!("−{}", human_size(g.wasted())),
                                    p.accent_text,
                                )
                                .on_hover_text("Space freed by keeping only one copy");
                            });
                            let extra =
                                2 * usize::from(wide) + usize::from(svt) + usize::from(similar);
                            for _ in 0..extra {
                                cell(&mut row, &mut |_| {});
                            }
                        }
                        Row::File(gi, fi) => {
                            let g = &results.groups[gi];
                            let f = &g.files[fi];
                            let marked = results.is_marked(&f.path);
                            row.col(|ui| {
                                row_hover(ui, table_rect, p.surface3);
                                ui.add_space(8.0);
                                if theme::mark_toggle(ui, marked) {
                                    cmds.push(Cmd::Toggle(f.path.clone()));
                                }
                            });
                            row.col(|ui| {
                                row_hover(ui, table_rect, p.surface3);
                                if is_decodable_image(&f.path) {
                                    match thumbs.get(&f.path, 96) {
                                        Thumb::Ready(t) => {
                                            ui.add(
                                                egui::Image::new(t)
                                                    .fit_to_exact_size(Vec2::splat(32.0))
                                                    .corner_radius(4),
                                            )
                                            .on_hover_ui(|ui| {
                                                ui.add(
                                                    egui::Image::new(t)
                                                        .max_size(Vec2::splat(256.0))
                                                        .corner_radius(6),
                                                );
                                            });
                                        }
                                        Thumb::Loading => {
                                            ui.add(egui::Spinner::new().size(14.0).color(p.muted));
                                        }
                                        Thumb::Failed => {
                                            ui.label(RichText::new(icons::WARNING).color(p.muted));
                                        }
                                    }
                                } else {
                                    ui.label(
                                        RichText::new(icons::for_kind(f.kind))
                                            .size(18.0)
                                            .color(p.muted),
                                    );
                                }
                            });
                            row.col(|ui| {
                                row_hover(ui, table_rect, p.surface3);
                                let mut name = medium(f.name()).color(p.text);
                                if marked {
                                    name = name.color(p.remove).strikethrough();
                                }
                                let r = ui
                                    .add(egui::Label::new(name).sense(Sense::click()))
                                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                                if r.clicked() {
                                    cmds.push(Cmd::Toggle(f.path.clone()));
                                }
                                file_interactions(
                                    &r.on_hover_text(f.path.display().to_string()),
                                    g,
                                    gi,
                                    f,
                                    cmds,
                                );
                            });
                            row.col(|ui| {
                                row_hover(ui, table_rect, p.surface3);
                                ui.label(RichText::new(f.parent_display()).color(p.muted))
                                    .on_hover_text(f.path.display().to_string());
                            });
                            row.col(|ui| {
                                row_hover(ui, table_rect, p.surface3);
                                ui.label(RichText::new(human_size(f.size)).color(p.text));
                            });
                            if wide {
                                row.col(|ui| {
                                    row_hover(ui, table_rect, p.surface3);
                                    ui.label(
                                        RichText::new(
                                            f.dims
                                                .map(|(w, h)| format!("{w} × {h}"))
                                                .unwrap_or_else(|| "—".into()),
                                        )
                                        .color(p.text),
                                    );
                                });
                                row.col(|ui| {
                                    row_hover(ui, table_rect, p.surface3);
                                    ui.label(RichText::new(format_time(f.modified)).color(p.muted));
                                });
                            }
                            if svt {
                                row.col(|ui| {
                                    row_hover(ui, table_rect, p.surface3);
                                    let c = if f.role == FolderRole::Source {
                                        p.source
                                    } else {
                                        p.target
                                    };
                                    theme::badge(ui, f.role.label(), c);
                                });
                            }
                            if similar {
                                row.col(|ui| {
                                    row_hover(ui, table_rect, p.surface3);
                                    ui.label(
                                        RichText::new(
                                            f.distance.map(|d| d.to_string()).unwrap_or_default(),
                                        )
                                        .color(p.text),
                                    );
                                });
                            }
                        }
                    });
                });
        });
}

/// Highlights the whole file row under the pointer. Called at the start of every cell
/// of the row, so the cells together paint one continuous band.
fn row_hover(ui: &egui::Ui, table: egui::Rect, fill: egui::Color32) {
    let bg = ui.max_rect().expand2(0.5 * ui.spacing().item_spacing);
    let hovered = ui.ctx().pointer_hover_pos().is_some_and(|pos| {
        table.contains(pos)
            && bg.y_range().contains(pos.y)
            && ui.ctx().layer_id_at(pos) == Some(ui.layer_id())
    });
    if hovered {
        paint_row_bg(ui, table, fill);
    }
}

/// Fills a cell's background including half the item spacing on each side, so the
/// cells of a row join into one band. The cell's own clip rect is only as wide as its
/// column, so paint with the table's horizontal extent and the scroll area's vertical clip.
fn paint_row_bg(ui: &egui::Ui, table: egui::Rect, fill: egui::Color32) {
    let bg = ui.max_rect().expand2(0.5 * ui.spacing().item_spacing);
    let clip = egui::Rect::from_x_y_ranges(table.x_range(), ui.clip_rect().y_range());
    let mut painter = ui.painter().clone();
    painter.set_clip_rect(clip);
    painter.rect_filled(bg, 0.0, fill);
}
