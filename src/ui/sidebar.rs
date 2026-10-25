//! Left-hand setup panel: folders, compare mode, match method, filters, and the Scan button.

use eframe::egui::{self, Align2, Margin, RichText, Sense, Stroke, Vec2};

use super::DupeApp;
use super::fonts::{medium, semibold};
use super::icons;
use super::theme::{self, Kind, Palette, Size, height, space};
use crate::model::{CompareMode, FileKind, FolderRole, Method, human_size};

pub(crate) fn show(app: &mut DupeApp, ui: &mut egui::Ui) {
    let p = Palette::of(ui);
    egui::Panel::left("sidebar")
        .resizable(true)
        .default_size(348.0)
        .size_range(310.0..=520.0)
        .show_separator_line(true)
        .frame(theme::panel_frame(ui, p.surface, Margin::ZERO))
        .show(ui, |ui| {
            egui::Panel::bottom("scan_area")
                .show_separator_line(true)
                .frame(theme::panel_frame(ui, p.surface, Margin::symmetric(20, 16)))
                .show(ui, |ui| scan_area(app, ui));

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    egui::Frame::new()
                        .inner_margin(Margin {
                            left: 20,
                            right: 20,
                            top: 16,
                            bottom: 16,
                        })
                        .show(ui, |ui| {
                            ui.add_enabled_ui(!app.busy(), |ui| {
                                folders_section(app, ui);
                                theme::divider(ui);
                                compare_section(app, ui);
                                theme::divider(ui);
                                method_section(app, ui);
                                theme::divider(ui);
                                filters_section(app, ui);
                            });
                        });
                });
        });
}

// ------------------------------------------------------------------ 1. folders

fn folders_section(app: &mut DupeApp, ui: &mut egui::Ui) {
    let mut add = false;
    let mut clear = false;
    let has_folders = !app.cfg.folders.is_empty();
    theme::section_header(ui, Some(1), "Folders", |ui| {
        add = ui
            .add(theme::button(
                ui,
                Kind::Primary,
                Size::Sm,
                format!("{}  Add", icons::PLUS),
            ))
            .on_hover_text("Add one or more folders")
            .clicked();
        if has_folders {
            clear = ui
                .add(theme::button(ui, Kind::Secondary, Size::Sm, "Clear"))
                .on_hover_text("Remove all folders from the list")
                .clicked();
        }
    });
    if add && let Some(dirs) = rfd::FileDialog::new().pick_folders() {
        for d in dirs {
            app.add_folder(d);
        }
    }
    if clear {
        app.cfg.folders.clear();
    }
    ui.add_space(space::XS);

    if app.cfg.folders.is_empty() {
        drop_zone(ui);
        return;
    }
    let svt = app.cfg.mode == CompareMode::SourceVsTarget;
    let mut remove = None;
    for (i, f) in app.cfg.folders.iter_mut().enumerate() {
        if folder_row(ui, f, svt) {
            remove = Some(i);
        }
    }
    if let Some(i) = remove {
        app.cfg.folders.remove(i);
    }
    theme::hint(
        ui,
        if svt {
            "Click a tag to switch between Source and Target."
        } else {
            "Sub-folders are included. Drag & drop also works."
        },
    );
}

/// One folder line. Returns true if the remove button was clicked.
fn folder_row(ui: &mut egui::Ui, f: &mut crate::model::FolderEntry, svt: bool) -> bool {
    let p = Palette::of(ui);
    let mut removed = false;
    let name = f
        .path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| f.path.display().to_string());
    let parent = f
        .path
        .parent()
        .map(|x| x.display().to_string())
        .unwrap_or_default();
    egui::Frame::new()
        .fill(p.surface2)
        .corner_radius(8)
        .inner_margin(Margin::symmetric(10, 6))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(RichText::new(icons::FOLDER).size(18.0).color(p.muted));
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 1.0;
                    ui.set_max_width(ui.available_width() - if svt { 110.0 } else { 40.0 });
                    ui.add(egui::Label::new(medium(name).color(p.text)).truncate())
                        .on_hover_text(f.path.display().to_string());
                    let chars = (ui.available_width() / 6.2).max(8.0) as usize;
                    ui.label(
                        RichText::new(theme::truncate_left(&parent, chars))
                            .size(11.5)
                            .color(p.muted),
                    );
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(
                            egui::Button::new(RichText::new(icons::X).size(14.0).color(p.muted))
                                .frame_when_inactive(false)
                                .min_size(Vec2::splat(height::SM)),
                        )
                        .on_hover_text("Remove from list")
                        .clicked()
                    {
                        removed = true;
                    }
                    if svt {
                        let color = if f.role == FolderRole::Source {
                            p.source
                        } else {
                            p.target
                        };
                        if theme::badge(ui, f.role.label(), color)
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .on_hover_text("Click to switch role")
                            .clicked()
                        {
                            f.role = f.role.toggled();
                        }
                    }
                });
            });
        });
    ui.add_space(2.0);
    removed
}

fn drop_zone(ui: &mut egui::Ui) {
    let p = Palette::of(ui);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 96.0), Sense::hover());
    ui.painter().rect_filled(rect, 10.0, p.surface2);
    let stroke = Stroke::new(1.0, p.muted.gamma_multiply(0.6));
    let r = rect.shrink(0.5);
    let pts = [
        r.left_top(),
        r.right_top(),
        r.right_bottom(),
        r.left_bottom(),
        r.left_top(),
    ];
    for w in pts.windows(2) {
        ui.painter()
            .extend(egui::Shape::dashed_line(&[w[0], w[1]], stroke, 5.0, 4.0));
    }
    let painter = ui.painter();
    painter.text(
        rect.center() - Vec2::new(0.0, 16.0),
        Align2::CENTER_CENTER,
        icons::TRAY_ARROW_DOWN,
        egui::FontId::proportional(22.0),
        p.muted,
    );
    painter.text(
        rect.center() + Vec2::new(0.0, 10.0),
        Align2::CENTER_CENTER,
        "Drop folders here",
        egui::FontId::new(13.0, egui::FontFamily::Name("inter-medium".into())),
        p.text,
    );
    painter.text(
        rect.center() + Vec2::new(0.0, 28.0),
        Align2::CENTER_CENTER,
        "or click Add",
        egui::FontId::proportional(11.5),
        p.muted,
    );
}

// ------------------------------------------------------------------ 2. compare

fn compare_section(app: &mut DupeApp, ui: &mut egui::Ui) {
    let p = Palette::of(ui);
    theme::section_header(ui, Some(2), "Compare", |_| {});
    ui.add_space(space::XS);
    let svt_label = format!("Source {} Target", icons::ARROW_RIGHT);
    theme::segmented(
        ui,
        "compare_mode",
        &mut app.cfg.mode,
        &[
            (CompareMode::AllTogether, "All folders together"),
            (CompareMode::SourceVsTarget, svt_label.as_str()),
        ],
        true,
    );
    ui.add_space(2.0);
    theme::hint(
        ui,
        match app.cfg.mode {
            CompareMode::AllTogether => "Find duplicates anywhere across all the folders above.",
            CompareMode::SourceVsTarget => {
                "Only find files in Target folders that already exist in a Source folder (e.g. a backup vs. your originals)."
            }
        },
    );
    if app.cfg.mode == CompareMode::SourceVsTarget
        && let Err(e) = app.cfg.validate()
    {
        ui.label(
            RichText::new(format!("{}  {e}", icons::WARNING))
                .size(12.0)
                .color(p.warn),
        );
    }
}

// ------------------------------------------------------------------ 3. method

fn method_section(app: &mut DupeApp, ui: &mut egui::Ui) {
    theme::section_header(ui, Some(3), "Match by", |_| {});
    ui.add_space(space::XS);
    for m in Method::ALL {
        method_option(ui, &mut app.cfg.method, m);
    }
    ui.add_space(space::SM);
    method_options(app, ui);
}

/// Full-width selectable row: icon tile, title, description (when selected), radio dot.
fn method_option(ui: &mut egui::Ui, current: &mut Method, m: Method) {
    let p = Palette::of(ui);
    let selected = *current == m;
    let id = ui.make_persistent_id(("method", m.cli_name()));
    let hovered = ui.ctx().read_response(id).is_some_and(|r| r.hovered());
    let (fill, stroke) = match (selected, hovered) {
        (true, _) => (p.accent_soft, Stroke::new(1.5, p.accent)),
        (false, true) => (p.surface2, Stroke::new(1.0, p.border)),
        (false, false) => (p.surface, Stroke::new(1.0, p.border)),
    };
    let resp = egui::Frame::new()
        .fill(fill)
        .stroke(stroke)
        .corner_radius(8)
        .inner_margin(Margin::symmetric(10, 8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                let (tile, _) = ui.allocate_exact_size(Vec2::splat(30.0), Sense::hover());
                let (tile_bg, icon_c) = if selected {
                    (p.accent, p.on_accent)
                } else {
                    (p.surface2, p.muted)
                };
                ui.painter().rect_filled(tile, 7.0, tile_bg);
                ui.painter().text(
                    tile.center(),
                    Align2::CENTER_CENTER,
                    icons::for_method(m),
                    egui::FontId::proportional(16.0),
                    icon_c,
                );
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    ui.set_max_width(ui.available_width() - 24.0);
                    let title = if selected {
                        semibold(m.label()).color(p.text)
                    } else {
                        medium(m.label()).color(p.text)
                    };
                    ui.add_space(if selected { 0.0 } else { 6.0 });
                    ui.label(title);
                    if selected {
                        ui.add(
                            egui::Label::new(
                                RichText::new(m.description()).size(12.0).color(p.muted),
                            )
                            .wrap(),
                        );
                    }
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let (r, _) = ui.allocate_exact_size(Vec2::splat(16.0), Sense::hover());
                    if selected {
                        ui.painter().circle_filled(r.center(), 7.5, p.accent);
                        ui.painter().circle_filled(r.center(), 3.0, p.on_accent);
                    } else {
                        ui.painter().circle_stroke(
                            r.center(),
                            7.0,
                            Stroke::new(1.5, p.muted.gamma_multiply(0.7)),
                        );
                    }
                });
            });
        })
        .response;
    let click = ui
        .interact(resp.rect, id, Sense::click())
        .on_hover_cursor(egui::CursorIcon::PointingHand);
    if click.clicked() {
        *current = m;
    }
    ui.add_space(2.0);
}

fn method_options(app: &mut DupeApp, ui: &mut egui::Ui) {
    let p = Palette::of(ui);
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
            ui.checkbox(&mut cfg.content_partial_only, "Fast mode (skip full hash)")
                .on_hover_text(
                    "Only hash 3 × 64 KB chunks (start, middle, end) plus the size.\n\
                 Much faster for big videos; very small risk of false matches.\n\
                 Files ≤ 192 KB are always fully hashed.",
                );
        }
        Method::SimilarImage => {
            ui.horizontal(|ui| {
                ui.label(medium("Similarity").color(p.text));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(semibold(cfg.similarity_threshold.to_string()).color(p.accent));
                });
            });
            ui.spacing_mut().slider_width = ui.available_width() - 4.0;
            ui.add(egui::Slider::new(&mut cfg.similarity_threshold, 0..=32).show_value(false));
            ui.horizontal(|ui| {
                ui.label(RichText::new("Strict").size(11.5).color(p.muted));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new("Loose").size(11.5).color(p.muted));
                });
            });
            theme::hint(
                ui,
                match cfg.similarity_threshold {
                    0..=6 => "Near-identical only (re-saves, format changes).",
                    7..=12 => "Recommended: resized and re-compressed copies.",
                    13..=18 => "Loose: also catches edits. Review results carefully.",
                    _ => "Very loose: expect unrelated matches.",
                },
            );
            ui.add_space(space::XS);
            ui.checkbox(
                &mut cfg.ignore_rotation,
                "Also match rotated / mirrored copies",
            );
        }
        Method::Size => {
            ui.label(
                RichText::new(format!(
                    "{}  Files of the same size are often different. Review before deleting.",
                    icons::WARNING
                ))
                .size(12.0)
                .color(p.warn),
            );
        }
    }
}

// ------------------------------------------------------------------ 4. filters

fn filters_section(app: &mut DupeApp, ui: &mut egui::Ui) {
    let p = Palette::of(ui);
    let enabled = FileKind::ALL
        .iter()
        .filter(|k| app.cfg.kinds[k.index()])
        .count();
    let summary = format!(
        "{} · {}{}",
        if enabled == FileKind::ALL.len() {
            "All file types".to_string()
        } else {
            format!("{enabled} of 6 file types")
        },
        if app.cfg.recursive {
            "including sub-folders"
        } else {
            "top level only"
        },
        if app.cfg.include_hidden {
            " · hidden files"
        } else {
            ""
        },
    );
    let caret = if app.show_filters {
        icons::CARET_DOWN
    } else {
        icons::CARET_RIGHT
    };
    let header = ui.horizontal(|ui| {
        ui.set_min_height(height::MD);
        ui.label(RichText::new(icons::SLIDERS).size(16.0).color(p.muted));
        ui.label(semibold("Filters & options").size(14.5).color(p.text));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(RichText::new(caret).size(14.0).color(p.muted));
        });
    });
    if ui
        .interact(
            header.response.rect,
            ui.scope_id().with("filters_hdr"),
            Sense::click(),
        )
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .clicked()
    {
        app.show_filters = !app.show_filters;
    }
    if !app.show_filters {
        theme::hint(ui, summary);
        return;
    }
    ui.add_space(space::XS);

    ui.label(medium("File types").size(12.5).color(p.muted));
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(6.0, 6.0);
        for k in FileKind::ALL {
            let on = app.cfg.kinds[k.index()];
            if theme::chip(ui, on, format!("{}  {}", icons::for_kind(k), k.label())).clicked() {
                app.cfg.kinds[k.index()] = !on;
            }
        }
    });
    ui.add_space(space::MD);

    ui.label(medium("File size").size(12.5).color(p.muted));
    egui::Grid::new("size_grid")
        .num_columns(2)
        .spacing([space::MD, 6.0])
        .show(ui, |ui| {
            ui.label("Minimum");
            ui.add(
                egui::DragValue::new(&mut app.min_size_kb)
                    .range(0.0..=f64::MAX)
                    .speed(1.0)
                    .suffix(" KB"),
            );
            ui.end_row();
            ui.label("Maximum");
            ui.add(
                egui::DragValue::new(&mut app.max_size_mb)
                    .range(0.0..=f64::MAX)
                    .speed(10.0)
                    .suffix(" MB"),
            )
            .on_hover_text("0 = no limit");
            ui.end_row();
        });
    if app.max_size_mb == 0.0 {
        theme::hint(
            ui,
            format!(
                "From {} up, no upper limit.",
                human_size((app.min_size_kb * 1024.0) as u64)
            ),
        );
    }
    ui.add_space(space::MD);

    ui.label(medium("Folders").size(12.5).color(p.muted));
    ui.checkbox(&mut app.cfg.recursive, "Include sub-folders");
    ui.checkbox(
        &mut app.cfg.include_hidden,
        "Include hidden files & folders",
    );
    ui.checkbox(&mut app.cfg.follow_symlinks, "Follow symbolic links");
    ui.add_space(space::XS);
    ui.label(
        RichText::new("Skip paths containing")
            .size(12.5)
            .color(p.muted),
    );
    ui.add(
        egui::TextEdit::singleline(&mut app.cfg.excludes)
            .hint_text("node_modules, .git, cache")
            .desired_width(f32::INFINITY)
            .margin(Margin::symmetric(8, 6)),
    );
}

// ------------------------------------------------------------------ scan button

fn scan_area(app: &mut DupeApp, ui: &mut egui::Ui) {
    let p = Palette::of(ui);
    if let Some(job) = &app.job {
        let prog = job.progress();
        let (stage, done, total) = prog.snapshot();
        ui.horizontal(|ui| {
            ui.add(egui::Spinner::new().size(14.0));
            ui.label(
                semibold(if job.is_scan() {
                    "Scanning…"
                } else {
                    "Working…"
                })
                .color(p.text),
            );
        });
        ui.label(RichText::new(&stage).size(12.0).color(p.muted));
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
                .text(RichText::new(text).size(11.5))
                .animate(total == 0)
                .desired_width(f32::INFINITY)
                .desired_height(8.0)
                .fill(p.accent)
                .corner_radius(4),
        );
        ui.add_space(space::XS);
        let w = ui.available_width();
        if ui
            .add(
                theme::button(
                    ui,
                    Kind::Secondary,
                    Size::Md,
                    format!("{}  Cancel", icons::STOP),
                )
                .min_size(Vec2::new(w, height::MD)),
            )
            .clicked()
        {
            prog.request_cancel();
        }
        return;
    }
    let n = app.cfg.folders.len();
    let ready = app.cfg.validate();
    match &ready {
        Ok(()) => theme::hint(
            ui,
            format!(
                "{n} folder{} · {}",
                if n == 1 { "" } else { "s" },
                app.cfg.method.label()
            ),
        ),
        Err(e) => ui.label(
            RichText::new(format!("{}  {e}", icons::INFO))
                .size(12.0)
                .color(p.muted),
        ),
    };
    let label = if app.last_scan.is_none() {
        format!("{}  Scan for duplicates", icons::MAGNIFYING_GLASS)
    } else {
        format!("{}  Scan again", icons::ARROW_CLOCKWISE)
    };
    let w = ui.available_width();
    if ui
        .add_enabled(
            ready.is_ok(),
            theme::button(ui, Kind::Primary, Size::Lg, label).min_size(Vec2::new(w, height::LG)),
        )
        .clicked()
    {
        let ctx = ui.ctx().clone();
        app.start_scan(&ctx);
    }
}
