//! Table view: one header row per group followed by its files.

use eframe::egui::{self, RichText, Vec2};
use egui_extras::{Column, TableBuilder};

use super::Cmd;
use super::widgets::file_interactions;
use crate::model::{FolderRole, Method, format_time, human_size, is_decodable_image};
use crate::ui::DupeApp;
use crate::ui::results::Row;
use crate::ui::theme::{self, ACCENT, REMOVE, SOURCE, TARGET};
use crate::ui::thumbs::Thumb;

const GROUP_ROW_H: f32 = 38.0;
const FILE_ROW_H: f32 = 36.0;

pub(super) fn show(app: &mut DupeApp, ui: &mut egui::Ui, cmds: &mut Vec<Cmd>) {
    let svt = app.results.is_source_vs_target();
    let similar = app.results.method() == Some(Method::SimilarImage);
    let DupeApp {
        results, thumbs, ..
    } = app;
    let rows = results.rows();
    let header_bg = ui.visuals().widgets.inactive.weak_bg_fill;

    let mut table = TableBuilder::new(ui)
        .striped(false)
        .resizable(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::exact(96.0))
        .column(Column::exact(40.0))
        .column(Column::initial(260.0).at_least(100.0).clip(true))
        .column(Column::initial(360.0).at_least(100.0).clip(true))
        .column(Column::initial(90.0).at_least(60.0))
        .column(Column::initial(100.0).at_least(60.0))
        .column(Column::initial(130.0).at_least(70.0));
    if svt {
        table = table.column(Column::initial(80.0));
    }
    if similar {
        table = table.column(Column::initial(60.0));
    }
    table
        .column(Column::remainder())
        .header(24.0, |mut h| {
            for title in [
                "Status",
                "",
                "Name",
                "Folder",
                "Size",
                "Dimensions",
                "Modified",
            ] {
                h.col(|ui| {
                    ui.label(RichText::new(title).small().strong());
                });
            }
            if svt {
                h.col(|ui| {
                    ui.label(RichText::new("Role").small().strong());
                });
            }
            if similar {
                h.col(|ui| {
                    ui.label(RichText::new("Diff").small().strong())
                        .on_hover_text(
                            "Visual difference to the group's best image (0 = identical)",
                        );
                });
            }
            h.col(|_| {});
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
                    let cell = |row: &mut egui_extras::TableRow,
                                add: &mut dyn FnMut(&mut egui::Ui)| {
                        row.col(|ui| {
                            let bg = ui.max_rect().expand2(0.5 * ui.spacing().item_spacing);
                            ui.painter().rect_filled(bg, 0.0, header_bg);
                            add(ui);
                        });
                    };
                    cell(&mut row, &mut |ui| {
                        ui.label(RichText::new(g.main_kind().icon()).size(18.0));
                        ui.label(RichText::new(format!("#{}", g.id)).strong());
                    });
                    cell(&mut row, &mut |_| {});
                    cell(&mut row, &mut |ui| {
                        ui.label(RichText::new(format!("{} copies", g.files.len())).strong());
                    });
                    cell(&mut row, &mut |ui| {
                        ui.label(RichText::new(&g.label).weak());
                    });
                    cell(&mut row, &mut |ui| {
                        theme::badge(ui, format!("−{}", human_size(g.wasted())), ACCENT)
                            .on_hover_text("Space freed by keeping only one copy");
                    });
                    cell(&mut row, &mut |ui| {
                        if n_marked > 0 {
                            ui.label(
                                RichText::new(format!("{n_marked} marked"))
                                    .small()
                                    .color(REMOVE),
                            );
                        }
                    });
                    cell(&mut row, &mut |ui| {
                        if ui
                            .small_button("⚖ Compare")
                            .on_hover_text("View this group side by side")
                            .clicked()
                            && let Some(pos) = results.visible().iter().position(|&v| v == gi)
                        {
                            cmds.push(Cmd::Compare(pos));
                        }
                    });
                    let extra = 1 + usize::from(svt) + usize::from(similar);
                    for _ in 0..extra {
                        cell(&mut row, &mut |_| {});
                    }
                }
                Row::File(gi, fi) => {
                    let g = &results.groups[gi];
                    let f = &g.files[fi];
                    let marked = results.is_marked(&f.path);
                    row.col(|ui| {
                        ui.add_space(4.0);
                        if theme::mark_toggle(ui, marked) {
                            cmds.push(Cmd::Toggle(f.path.clone()));
                        }
                    });
                    row.col(|ui| {
                        if is_decodable_image(&f.path) {
                            match thumbs.get(&f.path, 96) {
                                Thumb::Ready(t) => {
                                    ui.add(
                                        egui::Image::new(t)
                                            .fit_to_exact_size(Vec2::splat(30.0))
                                            .corner_radius(3),
                                    )
                                    .on_hover_ui(|ui| {
                                        ui.add(egui::Image::new(t).max_size(Vec2::splat(256.0)));
                                    });
                                }
                                Thumb::Loading => {
                                    ui.spinner();
                                }
                                Thumb::Failed => {
                                    ui.label("⚠");
                                }
                            }
                        } else {
                            ui.label(RichText::new(f.kind.icon()).size(18.0));
                        }
                    });
                    row.col(|ui| {
                        let mut name = RichText::new(f.name());
                        if marked {
                            name = name.color(REMOVE).strikethrough();
                        }
                        let r = ui.add(egui::Label::new(name).sense(egui::Sense::click()));
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
                        ui.label(RichText::new(f.parent_display()).weak())
                            .on_hover_text(f.path.display().to_string());
                    });
                    row.col(|ui| {
                        ui.label(human_size(f.size));
                    });
                    row.col(|ui| {
                        ui.label(
                            f.dims
                                .map(|(w, h)| format!("{w}×{h}"))
                                .unwrap_or_else(|| "–".into()),
                        );
                    });
                    row.col(|ui| {
                        ui.label(format_time(f.modified));
                    });
                    if svt {
                        row.col(|ui| {
                            let c = if f.role == FolderRole::Source {
                                SOURCE
                            } else {
                                TARGET
                            };
                            theme::badge(ui, f.role.label(), c);
                        });
                    }
                    if similar {
                        row.col(|ui| {
                            ui.label(f.distance.map(|d| d.to_string()).unwrap_or_default());
                        });
                    }
                    row.col(|_| {});
                }
            });
        });
}
