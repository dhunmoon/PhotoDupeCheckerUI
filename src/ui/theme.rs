//! Design system: colour palette, type scale, spacing and the reusable widgets built on them.
//!
//! All widgets get their colours from [`Palette::of`], so they look right in both light
//! and dark mode. Use these helpers instead of styling widgets ad hoc so every part of the
//! UI keeps the same sizes and spacing.

use eframe::egui::{
    self, Align2, Button, Color32, CornerRadius, FontFamily, FontId, Frame, Margin, Response,
    RichText, Sense, Shadow, Stroke, TextStyle, Theme, Ui, Vec2,
};

use super::fonts::{self, medium, semibold};
use super::icons;

/// Spacing scale (px).
pub mod space {
    pub const XS: f32 = 4.0;
    pub const SM: f32 = 8.0;
    pub const MD: f32 = 12.0;
    pub const LG: f32 = 16.0;
    pub const XL: f32 = 24.0;
}

/// Control heights (px).
pub mod height {
    pub const SM: f32 = 26.0;
    pub const MD: f32 = 32.0;
    pub const LG: f32 = 40.0;
}

pub const RADIUS: u8 = 6;
pub const RADIUS_LG: u8 = 10;

/// Semantic colours for one theme.
#[derive(Clone, Copy)]
pub struct Palette {
    /// Main content background.
    pub bg: Color32,
    /// Side panels, top/bottom bars, dialogs.
    pub surface: Color32,
    /// Inputs, buttons, cards on a surface.
    pub surface2: Color32,
    /// Hover state of `surface2`.
    pub surface3: Color32,
    pub border: Color32,
    pub text: Color32,
    pub muted: Color32,
    pub accent: Color32,
    /// Text drawn on top of `accent`.
    pub on_accent: Color32,
    /// Very light accent tint for selected rows.
    pub accent_soft: Color32,
    pub keep: Color32,
    pub remove: Color32,
    pub danger: Color32,
    pub warn: Color32,
    pub source: Color32,
    pub target: Color32,
}

pub const DARK: Palette = Palette {
    bg: Color32::from_rgb(17, 19, 24),
    surface: Color32::from_rgb(24, 27, 33),
    surface2: Color32::from_rgb(32, 36, 44),
    surface3: Color32::from_rgb(42, 47, 57),
    border: Color32::from_rgb(44, 49, 59),
    text: Color32::from_rgb(231, 233, 238),
    muted: Color32::from_rgb(140, 147, 161),
    accent: Color32::from_rgb(91, 140, 255),
    on_accent: Color32::WHITE,
    accent_soft: Color32::from_rgb(30, 41, 66),
    keep: Color32::from_rgb(63, 185, 122),
    remove: Color32::from_rgb(240, 106, 106),
    danger: Color32::from_rgb(208, 64, 64),
    warn: Color32::from_rgb(233, 180, 76),
    source: Color32::from_rgb(79, 163, 224),
    target: Color32::from_rgb(185, 138, 230),
};

pub const LIGHT: Palette = Palette {
    bg: Color32::from_rgb(246, 247, 249),
    surface: Color32::WHITE,
    surface2: Color32::from_rgb(242, 244, 247),
    surface3: Color32::from_rgb(231, 234, 240),
    border: Color32::from_rgb(225, 228, 234),
    text: Color32::from_rgb(27, 31, 39),
    muted: Color32::from_rgb(106, 114, 128),
    accent: Color32::from_rgb(47, 103, 232),
    on_accent: Color32::WHITE,
    accent_soft: Color32::from_rgb(235, 241, 254),
    keep: Color32::from_rgb(31, 157, 92),
    remove: Color32::from_rgb(214, 69, 69),
    danger: Color32::from_rgb(200, 52, 52),
    warn: Color32::from_rgb(183, 121, 31),
    source: Color32::from_rgb(43, 123, 192),
    target: Color32::from_rgb(139, 92, 201),
};

impl Palette {
    pub fn of(ui: &Ui) -> Self {
        Self::for_dark(ui.visuals().dark_mode)
    }

    pub fn for_dark(dark: bool) -> Self {
        if dark { DARK } else { LIGHT }
    }
}

/// Installs fonts, type scale, spacing and colours for both light and dark mode.
pub fn install(ctx: &egui::Context) {
    ctx.set_fonts(fonts::definitions());
    ctx.all_styles_mut(|s| {
        let semi = FontFamily::Name("inter-semibold".into());
        s.text_styles
            .insert(TextStyle::Small, FontId::proportional(11.5));
        s.text_styles
            .insert(TextStyle::Body, FontId::proportional(13.5));
        s.text_styles
            .insert(TextStyle::Button, FontId::proportional(13.5));
        s.text_styles
            .insert(TextStyle::Heading, FontId::new(18.0, semi));
        s.text_styles
            .insert(TextStyle::Monospace, FontId::monospace(12.5));
        s.spacing.item_spacing = egui::vec2(space::SM, space::SM);
        s.spacing.button_padding = egui::vec2(space::MD, 5.0);
        s.spacing.interact_size = egui::vec2(40.0, 28.0);
        s.spacing.menu_margin = Margin::same(6);
        s.spacing.window_margin = Margin::same(20);
        s.spacing.icon_width = 16.0;
        s.spacing.icon_width_inner = 9.0;
        s.spacing.icon_spacing = 6.0;
        s.interaction.selectable_labels = false;
    });
    for theme in [Theme::Dark, Theme::Light] {
        let p = Palette::for_dark(theme == Theme::Dark);
        ctx.style_mut_of(theme, |s| apply_palette(&mut s.visuals, &p));
    }
}

fn apply_palette(v: &mut egui::Visuals, p: &Palette) {
    let r = CornerRadius::same(RADIUS);
    v.panel_fill = p.surface;
    v.window_fill = p.surface;
    v.window_stroke = Stroke::new(1.0, p.border);
    v.window_corner_radius = CornerRadius::same(12);
    v.menu_corner_radius = CornerRadius::same(8);
    v.extreme_bg_color = p.bg;
    v.text_edit_bg_color = Some(p.surface2);
    v.faint_bg_color = p.surface2;
    v.code_bg_color = p.surface2;
    v.hyperlink_color = p.accent;
    v.warn_fg_color = p.warn;
    v.error_fg_color = p.remove;
    v.weak_text_color = Some(p.muted);
    v.override_text_color = None;
    v.slider_trailing_fill = true;
    v.striped = false;
    v.collapsing_header_frame = false;
    v.indent_has_left_vline = false;
    let shadow_alpha = if v.dark_mode { 90 } else { 28 };
    v.window_shadow = Shadow {
        offset: [0, 8],
        blur: 28,
        spread: 0,
        color: Color32::from_black_alpha(shadow_alpha),
    };
    v.popup_shadow = Shadow {
        offset: [0, 4],
        blur: 16,
        spread: 0,
        color: Color32::from_black_alpha(shadow_alpha),
    };
    v.selection.bg_fill = if v.dark_mode {
        p.accent.gamma_multiply(0.45)
    } else {
        p.accent_soft
    };
    v.selection.stroke = Stroke::new(
        1.0,
        if v.dark_mode {
            Color32::WHITE
        } else {
            p.accent
        },
    );

    let w = &mut v.widgets;
    w.noninteractive.bg_fill = p.surface;
    w.noninteractive.weak_bg_fill = p.surface;
    w.noninteractive.bg_stroke = Stroke::new(1.0, p.border);
    w.noninteractive.fg_stroke = Stroke::new(1.0, p.text);
    w.noninteractive.corner_radius = r;

    w.inactive.bg_fill = p.surface3;
    w.inactive.weak_bg_fill = p.surface2;
    w.inactive.bg_stroke = Stroke::new(1.0, p.border);
    w.inactive.fg_stroke = Stroke::new(1.0, p.text);
    w.inactive.corner_radius = r;
    w.inactive.expansion = 0.0;

    w.hovered.bg_fill = p.surface3;
    w.hovered.weak_bg_fill = p.surface3;
    w.hovered.bg_stroke = Stroke::new(1.0, p.border);
    w.hovered.fg_stroke = Stroke::new(1.5, p.text);
    w.hovered.corner_radius = r;
    w.hovered.expansion = 0.0;

    w.active.bg_fill = p.surface3;
    w.active.weak_bg_fill = p.surface3;
    w.active.bg_stroke = Stroke::new(1.0, p.accent);
    w.active.fg_stroke = Stroke::new(1.5, p.text);
    w.active.corner_radius = r;
    w.active.expansion = 0.0;

    w.open.weak_bg_fill = p.surface3;
    w.open.bg_fill = p.surface3;
    w.open.bg_stroke = Stroke::new(1.0, p.border);
    w.open.fg_stroke = Stroke::new(1.0, p.text);
    w.open.corner_radius = r;
}

// ------------------------------------------------------------------ buttons

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// The one main action in an area.
    Primary,
    /// Normal bordered button.
    Secondary,
    /// No background until hovered (toolbars, icon buttons).
    Ghost,
    /// Irreversible / destructive.
    Danger,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Size {
    Sm,
    Md,
    Lg,
}

impl Size {
    fn height(self) -> f32 {
        match self {
            Size::Sm => height::SM,
            Size::Md => height::MD,
            Size::Lg => height::LG,
        }
    }
}

/// A styled button. `label` may start with an icon, e.g. `format!("{} Add", icons::PLUS)`.
pub fn button(ui: &Ui, kind: Kind, size: Size, label: impl Into<String>) -> Button<'static> {
    let p = Palette::of(ui);
    let text = match size {
        Size::Sm => medium(label).size(12.5),
        Size::Md => medium(label),
        Size::Lg => semibold(label).size(14.5),
    };
    let b = match kind {
        Kind::Primary => Button::new(text.color(p.on_accent))
            .fill(p.accent)
            .stroke(Stroke::NONE),
        Kind::Danger => Button::new(text.color(Color32::WHITE))
            .fill(p.danger)
            .stroke(Stroke::NONE),
        Kind::Secondary => Button::new(text)
            .fill(p.surface2)
            .stroke(Stroke::new(1.0, p.border)),
        Kind::Ghost => Button::new(text).frame_when_inactive(false),
    };
    b.min_size(Vec2::new(0.0, size.height()))
        .corner_radius(RADIUS)
}

/// Square icon-only ghost button with a tooltip.
pub fn icon_button(ui: &mut Ui, icon: &str, tooltip: &str) -> Response {
    let p = Palette::of(ui);
    ui.add(
        Button::new(RichText::new(icon).size(17.0).color(p.text))
            .frame_when_inactive(false)
            .min_size(Vec2::splat(height::MD))
            .corner_radius(RADIUS),
    )
    .on_hover_text(tooltip)
}

// ------------------------------------------------------------------ layout pieces

/// Panel frame with consistent padding (panels draw their own 1 px separator line).
pub fn panel_frame(_ui: &Ui, fill: Color32, margin: Margin) -> Frame {
    Frame::new().fill(fill).inner_margin(margin)
}

/// Rounded container on top of a surface.
pub fn card(ui: &Ui) -> Frame {
    let p = Palette::of(ui);
    Frame::new()
        .fill(p.surface)
        .stroke(Stroke::new(1.0, p.border))
        .corner_radius(RADIUS_LG)
        .inner_margin(Margin::same(16))
}

/// Section header: numbered step + title, with optional controls on the right.
pub fn section_header(ui: &mut Ui, step: Option<usize>, title: &str, right: impl FnOnce(&mut Ui)) {
    let p = Palette::of(ui);
    ui.horizontal(|ui| {
        ui.set_min_height(height::MD);
        if let Some(n) = step {
            step_circle(ui, n, &p);
        }
        ui.label(semibold(title).size(14.5).color(p.text));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), right);
    });
}

fn step_circle(ui: &mut Ui, n: usize, p: &Palette) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(20.0), Sense::hover());
    ui.painter().circle(
        rect.center(),
        9.5,
        p.accent_soft,
        Stroke::new(1.0, p.accent.gamma_multiply(0.5)),
    );
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        n.to_string(),
        FontId::new(11.0, FontFamily::Name("inter-semibold".into())),
        p.accent,
    );
}

/// Muted helper text below a control.
pub fn hint(ui: &mut Ui, text: impl Into<String>) -> Response {
    ui.add(egui::Label::new(RichText::new(text).size(12.0).color(Palette::of(ui).muted)).wrap())
}

/// Thin horizontal divider with vertical breathing room.
pub fn divider(ui: &mut Ui) {
    ui.add_space(space::MD);
    let p = Palette::of(ui);
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 1.0), Sense::hover());
    ui.painter()
        .hline(rect.x_range(), rect.center().y, Stroke::new(1.0, p.border));
    ui.add_space(space::MD);
}

/// Thin vertical separator for toolbars.
pub fn vsep(ui: &mut Ui) {
    let p = Palette::of(ui);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(1.0, 20.0), Sense::hover());
    ui.painter()
        .vline(rect.center().x, rect.y_range(), Stroke::new(1.0, p.border));
}

// ------------------------------------------------------------------ data display

/// Small pill-shaped, tinted label. Fixed 20 px height so it never stretches in table cells.
/// The response senses clicks, so it can double as a toggle.
pub fn badge(ui: &mut Ui, text: impl Into<String>, color: Color32) -> Response {
    let font = FontId::new(11.5, FontFamily::Name("inter-medium".into()));
    let galley = ui.painter().layout_no_wrap(text.into(), font, color);
    let (rect, resp) =
        ui.allocate_exact_size(Vec2::new(galley.size().x + 16.0, 20.0), Sense::click());
    let tint = if ui.visuals().dark_mode { 0.16 } else { 0.11 };
    ui.painter()
        .rect_filled(rect, 10.0, color.gamma_multiply(tint));
    ui.painter()
        .galley(rect.center() - galley.size() / 2.0, galley, color);
    resp
}

/// The keep/remove status pill for a file. Returns true when clicked.
pub fn mark_toggle(ui: &mut Ui, marked: bool) -> bool {
    let p = Palette::of(ui);
    let (icon, label, color) = if marked {
        (icons::X_CIRCLE, "Remove", p.remove)
    } else {
        (icons::CHECK_CIRCLE, "Keep", p.keep)
    };
    let mut job = egui::text::LayoutJob::default();
    fonts::icon_fill(icon).size(13.0).color(color).append_to(
        &mut job,
        ui.style(),
        egui::FontSelection::Default,
        egui::Align::Center,
    );
    medium(format!("  {label}"))
        .size(12.0)
        .color(color)
        .append_to(
            &mut job,
            ui.style(),
            egui::FontSelection::Default,
            egui::Align::Center,
        );
    let tint = if ui.visuals().dark_mode { 0.16 } else { 0.10 };
    ui.add(
        Button::new(job)
            .fill(color.gamma_multiply(tint))
            .stroke(Stroke::new(1.0, color.gamma_multiply(0.45)))
            .corner_radius(12)
            .min_size(Vec2::new(84.0, 24.0)),
    )
    .on_hover_text(if marked {
        "Marked: the next action will be applied to this file. Click to keep it instead."
    } else {
        "Kept: this file will not be touched. Click to mark it for removal."
    })
    .clicked()
}

/// iOS / Fluent style segmented control. Returns true if the value changed.
///
/// With `fill_width` every segment gets an equal share of the available width.
pub fn segmented<T: PartialEq + Copy>(
    ui: &mut Ui,
    id: &str,
    value: &mut T,
    options: &[(T, &str)],
    fill_width: bool,
) -> bool {
    let p = Palette::of(ui);
    let mut changed = false;
    let track = if ui.visuals().dark_mode {
        p.bg
    } else {
        p.surface2
    };
    let selected_fill = if ui.visuals().dark_mode {
        p.surface3
    } else {
        p.surface
    };
    Frame::new()
        .fill(track)
        .stroke(Stroke::new(1.0, p.border))
        .corner_radius(8)
        .inner_margin(Margin::same(3))
        .show(ui, |ui| {
            ui.push_id(id, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 3.0;
                    let n = options.len() as f32;
                    let seg_w = if fill_width {
                        ((ui.available_width() - 3.0 * (n - 1.0)) / n).floor()
                    } else {
                        0.0
                    };
                    for (v, label) in options {
                        let selected = *value == *v;
                        let text = if selected {
                            semibold(*label).color(p.text)
                        } else {
                            medium(*label).color(p.muted)
                        };
                        let mut b = Button::new(text)
                            .corner_radius(6)
                            .min_size(Vec2::new(seg_w, height::SM));
                        b = if selected {
                            b.fill(selected_fill).stroke(Stroke::new(1.0, p.border))
                        } else {
                            b.frame_when_inactive(false)
                        };
                        if ui.add(b).clicked() && !selected {
                            *value = *v;
                            changed = true;
                        }
                    }
                });
            });
        });
    changed
}

/// A menu entry with a leading icon. All menu items share this height and alignment.
pub fn menu_item(ui: &mut Ui, icon: &str, label: &str) -> Response {
    let p = Palette::of(ui);
    let mut job = egui::text::LayoutJob::default();
    RichText::new(format!("{icon}   "))
        .size(15.0)
        .color(p.muted)
        .append_to(
            &mut job,
            ui.style(),
            egui::FontSelection::Default,
            egui::Align::Center,
        );
    RichText::new(label).color(p.text).append_to(
        &mut job,
        ui.style(),
        egui::FontSelection::Default,
        egui::Align::Center,
    );
    // Menus use a justified layout, so the button already spans the menu width.
    ui.add(
        Button::new(job)
            .frame_when_inactive(false)
            .min_size(Vec2::new(0.0, 30.0)),
    )
}

/// Small muted caption above a group of menu items.
pub fn menu_caption(ui: &mut Ui, text: &str) {
    ui.add_space(2.0);
    ui.label(medium(text).size(11.5).color(Palette::of(ui).muted));
}

/// Toggleable chip (filters). Returns the response; flip your bool on click.
pub fn chip(ui: &mut Ui, selected: bool, label: impl Into<String>) -> Response {
    let p = Palette::of(ui);
    let b = if selected {
        Button::new(medium(label).size(12.5).color(p.accent))
            .fill(p.accent_soft)
            .stroke(Stroke::new(1.0, p.accent.gamma_multiply(0.55)))
    } else {
        Button::new(medium(label).size(12.5).color(p.muted))
            .fill(p.surface2)
            .stroke(Stroke::new(1.0, p.border))
    };
    ui.add(b.corner_radius(14).min_size(Vec2::new(0.0, height::SM)))
}

/// Headline number with a caption underneath (top bar, dialogs).
pub fn metric(ui: &mut Ui, value: impl Into<String>, caption: &str, color: Option<Color32>) {
    let p = Palette::of(ui);
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        ui.label(semibold(value).size(16.0).color(color.unwrap_or(p.text)));
        ui.label(RichText::new(caption).size(11.5).color(p.muted));
    });
}

/// Truncates from the left so the end of a path (the informative part) stays visible.
pub fn truncate_left(s: &str, max: usize) -> String {
    let max = max.max(4);
    let n = s.chars().count();
    if n <= max {
        s.to_string()
    } else {
        let tail: String = s.chars().skip(n - (max - 1)).collect();
        format!("…{tail}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_left_keeps_the_end() {
        assert_eq!(truncate_left("short", 10), "short");
        assert_eq!(truncate_left("/a/very/long/path.jpg", 9), "…path.jpg");
        assert_eq!(truncate_left("ééééé", 4), "…ééé");
        assert_eq!(truncate_left("abcdef", 0), "…def");
    }

    #[test]
    fn palettes_have_readable_contrast() {
        // Rough luminance check: text must differ strongly from its background.
        let lum =
            |c: Color32| 0.2126 * c.r() as f32 + 0.7152 * c.g() as f32 + 0.0722 * c.b() as f32;
        for p in [DARK, LIGHT] {
            assert!((lum(p.text) - lum(p.surface)).abs() > 150.0);
            assert!((lum(p.muted) - lum(p.surface)).abs() > 60.0);
        }
    }
}
