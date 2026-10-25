//! Left-hand setup panel: the three numbered steps, advanced filters and the Scan button.

use eframe::egui::{self, Color32, Margin, RichText, Sense, Vec2};

use super::DupeApp;
use super::theme::{self, ACCENT, SOURCE, TARGET, WARN};
use crate::model::{CompareMode, FileKind, FolderRole, Method, human_size};

pub(crate) fn show(app: &mut DupeApp, ui: &mut egui::Ui) {
    egui::Panel::left("sidebar")
        .resizable(true)
        .default_size(360.0)
        .size_range(300.0..=560.0)
        .frame(egui::Frame::side_top_panel(ui.style()).inner_margin(Margin::same(12)))
        .show(ui, |ui| {
            egui::Panel::bottom("scan_area")
                .frame(egui::Frame::new().inner_margin(Margin::symmetric(0, 10)))
                .show_separator_line(false)
                .show(ui, |ui| scan_area(app, ui));

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.add_enabled_ui(!app.busy(), |ui| {
                        folders_card(app, ui);
                        ui.add_space(10.0);
                        compare_card(app, ui);
                        ui.add_space(10.0);
                        method_card(app, ui);
                        ui.add_space(10.0);
                        filters_card(app, ui);
                    });
                });
        });
}

fn folders_card(app: &mut DupeApp, ui: &mut egui::Ui) {
    theme::card(ui).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            theme::step_title(ui, 1, "Choose folders");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add(theme::primary_button("➕ Add"))
                    .on_hover_text("Add one or more folders")
                    .clicked()
                    && let Some(dirs) = rfd::FileDialog::new().pick_folders()
                {
                    for d in dirs {
                        app.add_folder(d);
                    }
                }
                if !app.cfg.folders.is_empty() && ui.small_button("Clear all").clicked() {
                    app.cfg.folders.clear();
                }
            });
        });

        if app.cfg.folders.is_empty() {
            drop_hint(ui);
            return;
        }
        let svt = app.cfg.mode == CompareMode::SourceVsTarget;
        let mut remove = None;
        for (i, f) in app.cfg.folders.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                if svt {
                    let color = if f.role == FolderRole::Source {
                        SOURCE
                    } else {
                        TARGET
                    };
                    if ui
                        .add(
                            theme::pill_button(f.role.label(), color, true)
                                .min_size(Vec2::new(64.0, 0.0)),
                        )
                        .on_hover_text(
                            "Click to switch between Source (originals) and Target (to clean up)",
                        )
                        .clicked()
                    {
                        f.role = f.role.toggled();
                    }
                } else {
                    ui.label(RichText::new("📁").size(18.0));
                }
                let name = f
                    .path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| f.path.display().to_string());
                let parent = f
                    .path
                    .parent()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default();
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    let w = ui.available_width() - 30.0;
                    ui.add(egui::Label::new(RichText::new(name).strong()).truncate())
                        .on_hover_text(f.path.display().to_string());
                    let chars = (w / 6.5).max(8.0) as usize;
                    ui.label(
                        RichText::new(theme::truncate_left(&parent, chars))
                            .small()
                            .weak(),
                    );
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .small_button("✖")
                        .on_hover_text("Remove from list")
                        .clicked()
                    {
                        remove = Some(i);
                    }
                });
            });
        }
        if let Some(i) = remove {
            app.cfg.folders.remove(i);
        }
        ui.label(
            RichText::new("Sub-folders are included. You can also drag & drop folders here.")
                .small()
                .weak(),
        );
    });
}

fn drop_hint(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 70.0), Sense::hover());
    let stroke = egui::Stroke::new(1.5, ui.visuals().weak_text_color().gamma_multiply(0.6));
    let pts = [
        rect.left_top(),
        rect.right_top(),
        rect.right_bottom(),
        rect.left_bottom(),
        rect.left_top(),
    ];
    for w in pts.windows(2) {
        ui.painter()
            .extend(egui::Shape::dashed_line(&[w[0], w[1]], stroke, 6.0, 4.0));
    }
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "Drop folders here or click ➕ Add",
        egui::FontId::proportional(13.0),
        ui.visuals().weak_text_color(),
    );
}

fn compare_card(app: &mut DupeApp, ui: &mut egui::Ui) {
    theme::card(ui).show(ui, |ui| {
        ui.set_width(ui.available_width());
        theme::step_title(ui, 2, "Compare");
        theme::segmented(
            ui,
            &mut app.cfg.mode,
            &[
                (CompareMode::AllTogether, "Everything together"),
                (CompareMode::SourceVsTarget, "Source → Target"),
            ],
        );
        let help = match app.cfg.mode {
            CompareMode::AllTogether => "Find duplicates anywhere across all the folders above.",
            CompareMode::SourceVsTarget => {
                "Only find files in Target folders that already exist in a Source folder. \
                 Click a folder's tag above to change its role."
            }
        };
        ui.label(RichText::new(help).small().weak());
        if app.cfg.mode == CompareMode::SourceVsTarget
            && let Err(e) = app.cfg.validate()
        {
            ui.label(RichText::new(format!("⚠ {e}")).small().color(WARN));
        }
    });
}

fn method_card(app: &mut DupeApp, ui: &mut egui::Ui) {
    theme::card(ui).show(ui, |ui| {
        ui.set_width(ui.available_width());
        theme::step_title(ui, 3, "Find duplicates by");
        for m in Method::ALL {
            method_option(ui, &mut app.cfg.method, m);
        }
        ui.add_space(4.0);
        method_options(app, ui);
    });
}

/// A full-width selectable row with title + description.
fn method_option(ui: &mut egui::Ui, current: &mut Method, m: Method) {
    let selected = *current == m;
    let fill = if selected {
        ACCENT.gamma_multiply(0.18)
    } else {
        Color32::TRANSPARENT
    };
    let stroke = if selected {
        egui::Stroke::new(1.5, ACCENT)
    } else {
        egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color)
    };
    let resp = egui::Frame::new()
        .fill(fill)
        .stroke(stroke)
        .corner_radius(8)
        .inner_margin(Margin::symmetric(10, 6))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(RichText::new(m.icon()).size(18.0));
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 1.0;
                    let mut title = RichText::new(m.label()).strong();
                    if selected {
                        title = title.color(ACCENT);
                    }
                    ui.label(title);
                    if selected {
                        ui.add(
                            egui::Label::new(RichText::new(m.description()).small().weak()).wrap(),
                        );
                    }
                });
            });
        })
        .response;
    let click = ui.interact(
        resp.rect,
        ui.scope_id().with(("method", m.cli_name())),
        Sense::click(),
    );
    if click.hovered() && !selected {
        ui.painter().rect_filled(
            resp.rect,
            8.0,
            ui.visuals()
                .widgets
                .hovered
                .weak_bg_fill
                .gamma_multiply(0.35),
        );
    }
    if click.on_hover_text(m.description()).clicked() {
        *current = m;
    }
    ui.add_space(2.0);
}

fn method_options(app: &mut DupeApp, ui: &mut egui::Ui) {
    let cfg = &mut app.cfg;
    match cfg.method {
        Method::FileName | Method::NameAndSize => {
            ui.checkbox(&mut cfg.name_case_insensitive, "Ignore upper/lower case");
            if cfg.method == Method::FileName {
                ui.checkbox(&mut cfg.name_ignore_ext, "Ignore file extension")
                    .on_hover_text("photo.jpg and photo.png are treated as the same name");
            }
        }
        Method::Content => {
            ui.checkbox(
                &mut cfg.content_partial_only,
                "⚡ Fast mode (skip full hash)",
            )
            .on_hover_text(
                "Only hash 3 × 64 KB chunks (start, middle, end) plus the size.\n\
                 Much faster for big videos; very small risk of false matches.\n\
                 Files ≤ 192 KB are always fully hashed.",
            );
        }
        Method::SimilarImage => {
            ui.label(RichText::new("How similar must images be?").small());
            ui.horizontal(|ui| {
                ui.label(RichText::new("Strict").small().weak());
                ui.add(egui::Slider::new(&mut cfg.similarity_threshold, 0..=32).show_value(true));
                ui.label(RichText::new("Loose").small().weak());
            });
            let hint = match cfg.similarity_threshold {
                0..=6 => "Near-identical only (re-saves, format changes).",
                7..=12 => "Recommended: resized / re-compressed copies.",
                13..=18 => "Loose: also catches edits; check results carefully.",
                _ => "Very loose: expect unrelated matches.",
            };
            ui.label(RichText::new(hint).small().weak());
            ui.checkbox(
                &mut cfg.ignore_rotation,
                "Also match rotated / mirrored copies",
            );
        }
        Method::Size => {
            ui.label(
                RichText::new(
                    "⚠ Files of the same size are often different. Review before deleting.",
                )
                .small()
                .color(WARN),
            );
        }
    }
}

fn filters_card(app: &mut DupeApp, ui: &mut egui::Ui) {
    theme::card(ui).show(ui, |ui| {
        ui.set_width(ui.available_width());
        let enabled = FileKind::ALL
            .iter()
            .filter(|k| app.cfg.kinds[k.index()])
            .count();
        let summary = format!(
            "{} · {}{}",
            if enabled == FileKind::ALL.len() {
                "All file types".to_string()
            } else {
                format!("{enabled} file types")
            },
            if app.cfg.recursive {
                "with sub-folders"
            } else {
                "top level only"
            },
            if app.cfg.include_hidden {
                " · hidden"
            } else {
                ""
            },
        );
        egui::CollapsingHeader::new(RichText::new("⚙  Filters & options").size(15.0).strong())
            .default_open(false)
            .show(ui, |ui| filters_body(app, ui));
        ui.label(RichText::new(summary).small().weak());
    });
}

fn filters_body(app: &mut DupeApp, ui: &mut egui::Ui) {
    ui.label(RichText::new("File types").strong());
    ui.horizontal_wrapped(|ui| {
        for k in FileKind::ALL {
            let on = &mut app.cfg.kinds[k.index()];
            ui.toggle_value(on, format!("{} {}", k.icon(), k.label()));
        }
    });
    ui.add_space(6.0);
    ui.label(RichText::new("File size").strong());
    egui::Grid::new("size_grid")
        .num_columns(2)
        .spacing([8.0, 4.0])
        .show(ui, |ui| {
            ui.label("Minimum (KB)");
            ui.add(
                egui::DragValue::new(&mut app.min_size_kb)
                    .range(0.0..=f64::MAX)
                    .speed(1.0),
            );
            ui.end_row();
            ui.label("Maximum (MB)");
            ui.add(
                egui::DragValue::new(&mut app.max_size_mb)
                    .range(0.0..=f64::MAX)
                    .speed(10.0),
            )
            .on_hover_text("0 = no limit");
            ui.end_row();
        });
    if app.max_size_mb == 0.0 {
        ui.label(
            RichText::new(format!(
                "Files from {} up, no upper limit",
                human_size((app.min_size_kb * 1024.0) as u64)
            ))
            .small()
            .weak(),
        );
    }
    ui.add_space(6.0);
    ui.label(RichText::new("Folders").strong());
    ui.checkbox(&mut app.cfg.recursive, "Include sub-folders");
    ui.checkbox(
        &mut app.cfg.include_hidden,
        "Include hidden files & folders",
    );
    ui.checkbox(&mut app.cfg.follow_symlinks, "Follow symbolic links");
    ui.label(RichText::new("Skip paths containing (comma separated)").small());
    ui.add(
        egui::TextEdit::singleline(&mut app.cfg.excludes)
            .hint_text("node_modules, .git, cache")
            .desired_width(f32::INFINITY),
    );
}

fn scan_area(app: &mut DupeApp, ui: &mut egui::Ui) {
    ui.separator();
    if let Some(job) = &app.job {
        let p = job.progress();
        let (stage, done, total) = p.snapshot();
        ui.label(
            RichText::new(if job.is_scan() {
                "Scanning…"
            } else {
                "Working…"
            })
            .strong(),
        );
        ui.label(RichText::new(&stage).small().weak());
        let frac = if total > 0 {
            done as f32 / total as f32
        } else {
            0.0
        };
        let text = if total > 0 {
            format!("{done} / {total}")
        } else {
            format!("{done} files")
        };
        ui.add(
            egui::ProgressBar::new(frac)
                .text(text)
                .animate(total == 0)
                .desired_width(f32::INFINITY),
        );
        if ui
            .add_sized([ui.available_width(), 30.0], egui::Button::new("⏹ Cancel"))
            .clicked()
        {
            p.request_cancel();
        }
        return;
    }
    let n = app.cfg.folders.len();
    let ready = app.cfg.validate();
    match &ready {
        Ok(()) => ui.label(
            RichText::new(format!(
                "Ready: {} folder{} · {}",
                n,
                if n == 1 { "" } else { "s" },
                app.cfg.method.label().to_lowercase()
            ))
            .small()
            .weak(),
        ),
        Err(e) => ui.label(RichText::new(e).small().color(WARN)),
    };
    let label = if app.results.is_empty() && app.last_scan.is_none() {
        "🔍  Scan for duplicates"
    } else {
        "🔄  Scan again"
    };
    let btn = theme::primary_button(label).min_size(Vec2::new(ui.available_width(), 40.0));
    if ui.add_enabled(ready.is_ok(), btn).clicked() {
        let ctx = ui.ctx().clone();
        app.start_scan(&ctx);
    }
}
