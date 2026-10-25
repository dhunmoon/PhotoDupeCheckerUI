//! Bottom bar: smart selection and mark summary on the left, actions on the right.

use eframe::egui::{self, Align, Layout, Margin, RichText, containers::menu::MenuButton};

use super::fonts::semibold;
use super::icons;
use super::theme::{self, Kind, Palette, Size, space};
use super::{DupeApp, PendingAction};
use crate::actions::ActionKind;
use crate::model::human_size;
use crate::selection::{self, KeepRule};

pub(crate) fn show(app: &mut DupeApp, ui: &mut egui::Ui) {
    let p = Palette::of(ui);
    egui::Panel::bottom("action_bar")
        .show_separator_line(true)
        .frame(theme::panel_frame(ui, p.surface, Margin::symmetric(16, 10)))
        .show(ui, |ui| {
            ui.add_enabled_ui(!app.busy(), |ui| {
                ui.horizontal(|ui| {
                    selection_controls(app, ui);
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        action_controls(app, ui)
                    });
                });
            });
            if !app.status.is_empty() {
                ui.add_space(2.0);
                ui.label(RichText::new(&app.status).size(11.5).color(p.muted));
            }
        });
}

fn selection_controls(app: &mut DupeApp, ui: &mut egui::Ui) {
    let p = Palette::of(ui);
    let before = app.results.marked.len();
    let smart = theme::button(
        ui,
        Kind::Secondary,
        Size::Md,
        format!("{}  Smart select  {}", icons::MAGIC_WAND, icons::CARET_DOWN),
    );
    MenuButton::from_button(smart.into_button(ui))
        .ui(ui, |ui| smart_select_menu(app, ui))
        .0
        .on_hover_text("Automatically decide which copy to keep in every group");
    if ui
        .add(theme::button(
            ui,
            Kind::Ghost,
            Size::Md,
            format!("{}  Invert", icons::SELECTION_INVERSE),
        ))
        .on_hover_text("Swap kept and marked files")
        .clicked()
    {
        selection::invert(&app.results.groups, &mut app.results.marked);
    }
    if ui
        .add(theme::button(
            ui,
            Kind::Ghost,
            Size::Md,
            format!("{}  Clear", icons::ERASER),
        ))
        .on_hover_text("Unmark everything")
        .clicked()
    {
        app.results.marked.clear();
    }
    if app.results.filter.only_marked && app.results.marked.len() != before {
        app.results.mark_dirty();
    }

    ui.add_space(space::XS);
    theme::vsep(ui);
    ui.add_space(space::XS);
    let s = app.results.stats();
    if s.files == 0 {
        ui.label(RichText::new("No files marked").color(p.muted))
            .on_hover_text(
                "Use Smart select, or click a file's Keep badge to mark it for removal.",
            );
    } else {
        ui.label(
            semibold(format!(
                "{} file{} marked",
                s.files,
                if s.files == 1 { "" } else { "s" }
            ))
            .color(p.remove),
        );
        ui.label(RichText::new(format!("· {}", human_size(s.bytes))).color(p.muted));
    }
    if s.fully_marked_groups > 0 {
        theme::badge(ui, format!("{}  {} group(s) fully marked", icons::WARNING, s.fully_marked_groups), p.warn)
            .on_hover_text("Every copy is marked in these groups. They are skipped by default so you never lose all copies.");
    }
}

fn smart_select_menu(app: &mut DupeApp, ui: &mut egui::Ui) {
    ui.set_min_width(300.0);
    theme::menu_caption(ui, "Keep one copy per group, mark the rest");
    let svt = app.results.is_source_vs_target();
    for r in KeepRule::ALL {
        let enabled = !r.needs_source_folders() || svt;
        let resp = ui
            .add_enabled_ui(enabled, |ui| theme::menu_item(ui, icons::CHECK, r.label()))
            .inner;
        let resp = if enabled {
            resp
        } else {
            resp.on_disabled_hover_text("Only for Source → Target scans")
        };
        if resp.clicked() {
            selection::apply_keep_rule(&app.results.groups, &mut app.results.marked, r);
            ui.close();
        }
    }
    ui.separator();
    theme::menu_caption(ui, "By folder");
    if theme::menu_item(ui, icons::FOLDER, "Keep the copies inside a folder…").clicked() {
        ui.close();
        if let Some(d) = rfd::FileDialog::new().pick_folder() {
            selection::keep_in_folder(&app.results.groups, &mut app.results.marked, &d);
        }
    }
    if theme::menu_item(ui, icons::FOLDER_OPEN, "Mark everything inside a folder…").clicked() {
        ui.close();
        if let Some(d) = rfd::FileDialog::new().pick_folder() {
            selection::select_in_folder(&app.results.groups, &mut app.results.marked, &d);
        }
    }
    ui.separator();
    theme::menu_caption(ui, "By pattern (adds to current marks)");
    for (input, hint, by_ext) in [
        (&mut app.ext_input, "Extensions, e.g. png, bmp", true),
        (&mut app.text_input, "Path contains, e.g. backup", false),
    ] {
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(input)
                    .hint_text(hint)
                    .desired_width(200.0)
                    .margin(Margin::symmetric(8, 5)),
            );
            if ui
                .add(theme::button(ui, Kind::Secondary, Size::Sm, "Mark"))
                .clicked()
            {
                if by_ext {
                    selection::select_by_extension(
                        &app.results.groups,
                        &mut app.results.marked,
                        input,
                    );
                } else {
                    selection::select_by_text(&app.results.groups, &mut app.results.marked, input);
                }
            }
        });
    }
}

fn action_controls(app: &mut DupeApp, ui: &mut egui::Ui) {
    let n = app.results.marked.len();
    let enabled = n > 0;
    let ask = |kind: ActionKind| PendingAction {
        skip_fully_marked: true,
        remove_empty_dirs: false,
        kind,
    };

    // Subtle red until there is something to trash, then solid red.
    let trash = theme::button(
        ui,
        if enabled {
            Kind::Danger
        } else {
            Kind::DangerSubtle
        },
        Size::Md,
        format!("{}  Move to Trash", icons::TRASH),
    );
    if ui
        .add_enabled(enabled, trash)
        .on_hover_text("Recoverable from the system Trash / Recycle Bin")
        .on_disabled_hover_text("Mark some files first")
        .clicked()
    {
        app.pending = Some(ask(ActionKind::Trash));
    }

    let more = theme::button(
        ui,
        Kind::Secondary,
        Size::Md,
        format!("More actions  {}", icons::CARET_DOWN),
    );
    MenuButton::from_button(more.into_button(ui)).ui(ui, |ui| {
        ui.set_min_width(290.0);
        ui.add_enabled_ui(enabled, |ui| {
            theme::menu_caption(ui, "Relocate");
            if theme::menu_item(ui, icons::FOLDER_OPEN, "Move marked to a folder…").clicked() {
                ui.close();
                if let Some(dest) = rfd::FileDialog::new().pick_folder() {
                    app.pending = Some(ask(ActionKind::MoveTo {
                        dest,
                        keep_structure: app.keep_structure,
                    }));
                }
            }
            if theme::menu_item(ui, icons::COPY, "Copy marked to a folder…").clicked() {
                ui.close();
                if let Some(dest) = rfd::FileDialog::new().pick_folder() {
                    let mut p = ask(ActionKind::CopyTo {
                        dest,
                        keep_structure: app.keep_structure,
                    });
                    p.skip_fully_marked = false;
                    app.pending = Some(p);
                }
            }
            ui.checkbox(&mut app.keep_structure, "Recreate folder structure");
            ui.separator();
            theme::menu_caption(ui, "Save space, keep paths working");
            if theme::menu_item(ui, icons::LINK, "Replace with hard links")
                .on_hover_text("Same drive only")
                .clicked()
            {
                ui.close();
                app.pending = Some(ask(ActionKind::HardLink));
            }
            if theme::menu_item(ui, icons::LINK_SIMPLE, "Replace with symbolic links").clicked() {
                ui.close();
                app.pending = Some(ask(ActionKind::SymLink));
            }
            ui.separator();
            if theme::menu_item(ui, icons::X_CIRCLE, "Delete permanently…")
                .on_hover_text("Skips the Trash. Cannot be undone.")
                .clicked()
            {
                ui.close();
                app.pending = Some(ask(ActionKind::Delete));
            }
            ui.separator();
            if theme::menu_item(ui, icons::CLIPBOARD_TEXT, "Copy marked paths").clicked() {
                ui.ctx()
                    .copy_text(app.results.marked_paths_sorted().join("\n"));
                app.status = format!("Copied {n} paths to the clipboard");
                ui.close();
            }
        });
        if theme::menu_item(ui, icons::FILE_CSV, "Export report (CSV)…").clicked() {
            ui.close();
            export_csv(app);
        }
    });
}

fn export_csv(app: &mut DupeApp) {
    let Some(path) = rfd::FileDialog::new()
        .set_file_name("duplicates.csv")
        .add_filter("CSV", &["csv"])
        .save_file()
    else {
        return;
    };
    let csv = crate::report::to_csv(&app.results.groups, &app.results.marked);
    app.status = match std::fs::write(&path, csv) {
        Ok(()) => format!("Exported report to {}", path.display()),
        Err(e) => format!("Export failed: {e}"),
    };
}
