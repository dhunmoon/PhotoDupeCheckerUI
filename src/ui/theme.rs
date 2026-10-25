//! Design system: colour palette, type scale, spacing and the reusable widgets built on them.
//!
//! All widgets get their colours from [`Palette::of`], so they look right in both light
//! and dark mode. Use these helpers instead of styling widgets ad hoc so every part of the
//! UI keeps the same sizes and spacing.

use eframe::egui::{
    self, Align2, Button, Color32, CornerRadius, FontFamily, FontId, Frame, Margin, Rect, Response,
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
///
/// The dark theme is the "Blue & Slate" design system; the CSS-style token each field
/// implements is noted on the field. The light theme mirrors it with slate light tones.
#[derive(Clone, Copy)]
pub struct Palette {
    /// Behind the main results view (`--bg-app`).
    pub bg: Color32,
    /// Sidebar, top/bottom bars, dialogs, cards and the list container (`--bg-sidebar`).
    pub surface: Color32,
    /// Inputs, secondary buttons, group headers, unselected option cards (`--bg-surface`).
    pub surface2: Color32,
    /// Hover state of `surface2` and of file rows (`--bg-surface-hover`).
    pub surface3: Color32,
    /// Every 1 px border and divider (`--border-subtle`).
    pub border: Color32,
    /// Focused inputs and selected items (`--border-focus`).
    pub border_focus: Color32,
    /// Primary text (`--text-primary`).
    pub text: Color32,
    /// Secondary text: captions, labels, icons (`--text-secondary`).
    pub muted: Color32,
    /// Least important text: placeholders, disabled hints (`--text-muted`).
    pub faint: Color32,
    /// Primary button fill, progress, radio dots (`--accent-primary`).
    pub accent: Color32,
    /// Primary button hover (`--accent-hover`).
    pub accent_hover: Color32,
    /// Text drawn on top of `accent`.
    pub on_accent: Color32,
    /// Background of selected filters / option cards (`--accent-subtle`).
    pub accent_soft: Color32,
    /// Accent used for *text and icons* on dark surfaces. `--accent-primary` is too dark
    /// to read as small text on `--bg-sidebar`, so this is a lighter blue of the same hue.
    pub accent_text: Color32,
    /// "Keep" status (`--status-success`).
    pub keep: Color32,
    /// "Keep" pill fill (`--status-success-bg`).
    pub keep_bg: Color32,
    /// "Remove" status and marked files (`--status-danger`).
    pub remove: Color32,
    /// "Remove" pill fill, subtle danger buttons (`--status-danger-bg`).
    pub remove_bg: Color32,
    /// Solid destructive button fill.
    pub danger: Color32,
    /// Solid destructive button hover.
    pub danger_hover: Color32,
    /// Warnings and "best in group" stars (`--status-warning`).
    pub warn: Color32,
    /// Informational accents (`--status-info`).
    pub info: Color32,
    /// Source folder role.
    pub source: Color32,
    /// Target folder role.
    pub target: Color32,
}

const fn hex(rgb: u32) -> Color32 {
    Color32::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

/// `rgba(r, g, b, alpha)` like in CSS.
const fn rgba(rgb: u32, alpha: f32) -> Color32 {
    Color32::from_rgba_unmultiplied_const(
        (rgb >> 16) as u8,
        (rgb >> 8) as u8,
        rgb as u8,
        (alpha * 255.0) as u8,
    )
}

pub const DARK: Palette = Palette {
    bg: hex(0x0B0F17),
    surface: hex(0x0F172A),
    surface2: hex(0x1E293B),
    surface3: hex(0x243247),
    border: hex(0x2D3B4F),
    border_focus: hex(0x3B82F6),
    text: hex(0xF8FAFC),
    muted: hex(0x94A3B8),
    faint: hex(0x64748B),
    accent: hex(0x2563EB),
    accent_hover: hex(0x1D4ED8),
    on_accent: Color32::WHITE,
    accent_soft: rgba(0x2563EB, 0.15),
    accent_text: hex(0x60A5FA),
    keep: hex(0x10B981),
    keep_bg: rgba(0x10B981, 0.12),
    remove: hex(0xEF4444),
    remove_bg: rgba(0xEF4444, 0.15),
    danger: hex(0xEF4444),
    danger_hover: hex(0xDC2626),
    warn: hex(0xF59E0B),
    info: hex(0x38BDF8),
    source: hex(0x38BDF8),
    target: hex(0xA78BFA),
};

pub const LIGHT: Palette = Palette {
    bg: hex(0xF1F5F9),
    surface: Color32::WHITE,
    surface2: hex(0xF8FAFC),
    surface3: hex(0xEEF2F7),
    border: hex(0xE2E8F0),
    border_focus: hex(0x3B82F6),
    text: hex(0x0F172A),
    muted: hex(0x475569),
    faint: hex(0x94A3B8),
    accent: hex(0x2563EB),
    accent_hover: hex(0x1D4ED8),
    on_accent: Color32::WHITE,
    accent_soft: rgba(0x2563EB, 0.10),
    accent_text: hex(0x1D4ED8),
    keep: hex(0x059669),
    keep_bg: rgba(0x10B981, 0.12),
    remove: hex(0xDC2626),
    remove_bg: rgba(0xEF4444, 0.10),
    danger: hex(0xDC2626),
    danger_hover: hex(0xB91C1C),
    warn: hex(0xD97706),
    info: hex(0x0284C7),
    source: hex(0x0284C7),
    target: hex(0x7C3AED),
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
    v.weak_text_color = Some(p.faint);
    v.faint_bg_color = p.surface2;
    v.code_bg_color = p.surface2;
    v.hyperlink_color = p.accent_text;
    v.warn_fg_color = p.warn;
    v.error_fg_color = p.remove;
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
    v.selection.bg_fill = p.accent.gamma_multiply(0.35);
    v.selection.stroke = Stroke::new(1.0, p.border_focus);

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
    w.active.bg_stroke = Stroke::new(1.0, p.border_focus);
    w.active.fg_stroke = Stroke::new(1.5, p.text);
    w.active.corner_radius = r;
    w.active.expansion = 0.0;

    w.open.weak_bg_fill = p.surface3;
    w.open.bg_fill = p.surface3;
    w.open.bg_stroke = Stroke::new(1.0, p.border_focus);
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
    /// Irreversible / destructive, solid red.
    Danger,
    /// Destructive but currently unavailable / low emphasis: red text on a faint red tint.
    DangerSubtle,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Size {
    Sm,
    Md,
    Lg,
}

impl Size {
    pub fn height(self) -> f32 {
        match self {
            Size::Sm => height::SM,
            Size::Md => height::MD,
            Size::Lg => height::LG,
        }
    }
}

/// A styled button. `label` may start with an icon, e.g. `format!("{} Add", icons::PLUS)`.
///
/// Add it like any widget (`ui.add`, `ui.put`, `ui.add_enabled`). Every kind has a 1 px
/// border so buttons of different kinds line up exactly, and a distinct hover colour.
pub fn button(ui: &Ui, kind: Kind, size: Size, label: impl Into<String>) -> ThemedButton {
    let p = Palette::of(ui);
    let text = match size {
        Size::Sm => medium(label).size(12.5),
        Size::Md => medium(label),
        Size::Lg => semibold(label).size(14.5),
    };
    let text = match kind {
        Kind::Primary => text.color(p.on_accent),
        Kind::Danger => text.color(Color32::WHITE),
        Kind::DangerSubtle => text.color(p.remove),
        Kind::Secondary | Kind::Ghost => text.color(p.text),
    };
    let mut b = Button::new(text)
        .min_size(Vec2::new(0.0, size.height()))
        .corner_radius(RADIUS);
    if kind == Kind::Ghost {
        b = b.frame_when_inactive(false);
    }
    ThemedButton { button: b, kind }
}

/// A [`Button`] plus the colours of its [`Kind`]; created by [`button`].
#[must_use = "add it with `ui.add`"]
pub struct ThemedButton {
    button: Button<'static>,
    kind: Kind,
}

impl ThemedButton {
    pub fn min_size(mut self, size: Vec2) -> Self {
        self.button = self.button.min_size(size);
        self
    }

    /// Keeps the label on one line, eliding it with "…" when it doesn't fit.
    pub fn truncate(mut self) -> Self {
        self.button = self.button.truncate();
        self
    }

    /// The plain [`Button`], for APIs such as `MenuButton::from_button`.
    ///
    /// Secondary and Ghost buttons look identical this way because they use the global
    /// widget colours. Primary / Danger buttons get a fixed fill without a hover state.
    pub fn into_button(self, ui: &Ui) -> Button<'static> {
        match kind_colors(self.kind, &Palette::of(ui)) {
            Some(c) => self.button.fill(c.fill).stroke(Stroke::new(1.0, c.stroke)),
            None => self.button,
        }
    }
}

/// Fill / stroke for the coloured button kinds. `None` = use the global widget visuals.
struct KindColors {
    fill: Color32,
    hover: Color32,
    stroke: Color32,
    hover_stroke: Color32,
}

fn kind_colors(kind: Kind, p: &Palette) -> Option<KindColors> {
    match kind {
        Kind::Primary => Some(KindColors {
            fill: p.accent,
            hover: p.accent_hover,
            stroke: p.accent,
            hover_stroke: p.accent_hover,
        }),
        Kind::Danger => Some(KindColors {
            fill: p.danger,
            hover: p.danger_hover,
            stroke: p.danger,
            hover_stroke: p.danger_hover,
        }),
        Kind::DangerSubtle => Some(KindColors {
            fill: p.remove_bg,
            hover: p.remove.gamma_multiply(0.25),
            stroke: p.remove.gamma_multiply(0.6),
            hover_stroke: p.remove,
        }),
        Kind::Secondary | Kind::Ghost => None,
    }
}

impl egui::Widget for ThemedButton {
    fn ui(self, ui: &mut Ui) -> Response {
        let Some(c) = kind_colors(self.kind, &Palette::of(ui)) else {
            return ui.add(self.button);
        };
        // Temporarily recolour the widget visuals so the button keeps egui's own
        // hover / press handling, then restore the style for the following widgets.
        let saved = ui.style().clone();
        let w = &mut ui.visuals_mut().widgets;
        for (state, fill, stroke) in [
            (&mut w.inactive, c.fill, c.stroke),
            (&mut w.hovered, c.hover, c.hover_stroke),
            (&mut w.active, c.hover, c.hover_stroke),
        ] {
            state.weak_bg_fill = fill;
            state.bg_fill = fill;
            state.bg_stroke = Stroke::new(1.0, stroke);
        }
        let resp = ui.add(self.button);
        ui.set_style(saved);
        resp
    }
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
        Stroke::new(1.0, p.border_focus.gamma_multiply(0.6)),
    );
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        n.to_string(),
        FontId::new(11.0, FontFamily::Name("inter-semibold".into())),
        p.accent_text,
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
    let tint = if ui.visuals().dark_mode { 0.15 } else { 0.11 };
    ui.painter().rect(
        rect,
        10.0,
        color.gamma_multiply(tint),
        Stroke::new(1.0, color.gamma_multiply(0.35)),
        egui::StrokeKind::Inside,
    );
    ui.painter()
        .galley(rect.center() - galley.size() / 2.0, galley, color);
    resp
}

/// Size of the interactive keep/remove pill used in tables and cards.
pub const STATUS_PILL: Vec2 = Vec2::new(84.0, 24.0);
/// Size of the solid keep/remove pill painted on top of thumbnails.
pub const STATUS_PILL_SOLID: Vec2 = Vec2::new(80.0, 22.0);

/// The keep/remove status pill for a file. Returns true when clicked.
pub fn mark_toggle(ui: &mut Ui, marked: bool) -> bool {
    let (rect, resp) = ui.allocate_exact_size(STATUS_PILL, Sense::click());
    if ui.is_rect_visible(rect) {
        paint_status_pill(ui, rect, marked, false, resp.hovered());
    }
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(if marked {
            "Marked: the next action will be applied to this file. Click to keep it instead."
        } else {
            "Kept: this file will not be touched. Click to mark it for removal."
        })
        .clicked()
}

/// Paints a keep/remove pill into `rect`.
///
/// The check / cross glyph is drawn with shapes rather than an icon font so it is always
/// centred exactly on the text, whatever the font metrics. `solid` is the high-contrast
/// variant used on top of images.
pub fn paint_status_pill(ui: &Ui, rect: Rect, marked: bool, solid: bool, hovered: bool) {
    let p = Palette::of(ui);
    let (label, color) = if marked {
        ("Remove", p.remove)
    } else {
        ("Keep", p.keep)
    };
    let (fill, stroke, fg, disc, mark) = if solid {
        (color, Stroke::NONE, Color32::WHITE, Color32::WHITE, color)
    } else {
        let (bg, hover_bg) = if marked {
            (p.remove_bg, p.remove.gamma_multiply(0.25))
        } else {
            (p.keep_bg, p.keep.gamma_multiply(0.22))
        };
        (
            if hovered { hover_bg } else { bg },
            Stroke::new(1.0, color.gamma_multiply(if hovered { 0.8 } else { 0.45 })),
            color,
            color,
            Color32::WHITE,
        )
    };
    let painter = ui.painter();
    painter.rect(
        rect,
        rect.height() / 2.0,
        fill,
        stroke,
        egui::StrokeKind::Inside,
    );

    let font = FontId::new(12.0, FontFamily::Name(fonts::MEDIUM.into()));
    let galley = painter.layout_no_wrap(label.to_owned(), font, fg);
    let d = (rect.height() * 0.5).round();
    let gap = 5.0;
    let content_w = d + gap + galley.size().x;
    let x0 = (rect.center().x - content_w / 2.0).round();
    let cy = rect.center().y;
    let c = egui::pos2(x0 + d / 2.0, cy);
    painter.circle_filled(c, d / 2.0, disc);
    let s = d / 12.0;
    let line = Stroke::new(1.5 * s.max(0.9), mark);
    if marked {
        let k = 2.2 * s;
        painter.line_segment([c + Vec2::new(-k, -k), c + Vec2::new(k, k)], line);
        painter.line_segment([c + Vec2::new(-k, k), c + Vec2::new(k, -k)], line);
    } else {
        painter.add(egui::Shape::line(
            vec![
                c + Vec2::new(-2.6, 0.1) * s,
                c + Vec2::new(-0.8, 1.9) * s,
                c + Vec2::new(2.7, -1.8) * s,
            ],
            line,
        ));
    }
    let text_pos = egui::pos2(x0 + d + gap, (cy - galley.size().y / 2.0).round());
    painter.galley(text_pos, galley, fg);
}

/// Allocates a full-width row of height `h` and splits it into `n` equal cells, so
/// buttons placed with `ui.put(cell, ..)` are exactly the same width whatever their text.
pub fn equal_cells(ui: &mut Ui, n: usize, h: f32) -> Vec<Rect> {
    let n = n.max(1);
    let gap = ui.spacing().item_spacing.x;
    let (row, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), h), Sense::hover());
    let w = (row.width() - gap * (n - 1) as f32) / n as f32;
    (0..n)
        .map(|i| {
            let x = row.left() + i as f32 * (w + gap);
            Rect::from_min_size(egui::pos2(x.round(), row.top()), Vec2::new(w.floor(), h))
        })
        .collect()
}

/// Single-line search input with a leading magnifier, exactly `height::MD` tall so it
/// lines up with buttons and combo boxes in the same toolbar.
pub fn search_field(ui: &mut Ui, text: &mut String, hint: &str, width: f32) -> Response {
    let p = Palette::of(ui);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height::MD), Sense::hover());
    let id = ui.make_persistent_id(("search_field", hint));
    let focused = ui.memory(|m| m.has_focus(id));
    let border = if focused { p.border_focus } else { p.border };
    ui.painter().rect(
        rect,
        RADIUS,
        p.surface2,
        Stroke::new(1.0, border),
        egui::StrokeKind::Inside,
    );
    let icon_c = egui::pos2(rect.left() + 18.0, rect.center().y);
    ui.painter().text(
        icon_c,
        Align2::CENTER_CENTER,
        icons::MAGNIFYING_GLASS,
        FontId::proportional(14.0),
        p.muted,
    );
    let edit_rect = Rect::from_min_max(
        egui::pos2(rect.left() + 32.0, rect.top() + 1.0),
        egui::pos2(rect.right() - 8.0, rect.bottom() - 1.0),
    );
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(edit_rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    child.add(
        egui::TextEdit::singleline(text)
            .id(id)
            .hint_text(hint)
            .frame(Frame::NONE)
            .margin(Margin::ZERO)
            .align(Align2::LEFT_CENTER)
            .min_size(Vec2::new(0.0, edit_rect.height()))
            .desired_width(edit_rect.width()),
    )
}

/// Segment height so the whole control (3 px padding + 1 px border each side) is
/// exactly `height::MD`.
const SEGMENT_H: f32 = height::MD - 8.0;

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
        p.surface
    } else {
        p.surface2
    };
    Frame::new()
        .fill(track)
        .stroke(Stroke::new(1.0, p.border))
        .corner_radius(8)
        .inner_margin(Margin::same(3))
        .show(ui, |ui| {
            // Set before `horizontal`, which uses `interact_size.y` as its row height.
            ui.spacing_mut().interact_size.y = SEGMENT_H;
            ui.spacing_mut().button_padding.y = 2.0;
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
                            .min_size(Vec2::new(seg_w, SEGMENT_H));
                        b = if selected {
                            b.fill(p.accent_soft)
                                .stroke(Stroke::new(1.0, p.border_focus))
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
    chip_sized(ui, selected, label, height::SM)
}

/// [`chip`] with an explicit height, e.g. `height::MD` inside a toolbar.
pub fn chip_sized(ui: &mut Ui, selected: bool, label: impl Into<String>, h: f32) -> Response {
    let p = Palette::of(ui);
    // Unselected chips use the global widget colours so they get the normal hover state.
    let b = if selected {
        Button::new(medium(label).size(12.5).color(p.accent_text))
            .fill(p.accent_soft)
            .stroke(Stroke::new(1.0, p.border_focus))
    } else {
        Button::new(medium(label).size(12.5).color(p.muted))
    };
    let radius = (h / 2.0) as u8;
    ui.add(b.corner_radius(radius).min_size(Vec2::new(0.0, h)))
}

/// Headline number with a caption underneath (top bar, dialogs).
///
/// Allocated as one fixed-height block so a row of metrics shares the same top line and
/// baseline and is centred like any other widget in a centred row.
pub fn metric(ui: &mut Ui, value: impl Into<String>, caption: &str, color: Option<Color32>) {
    let p = Palette::of(ui);
    let painter = ui.painter();
    let value = painter.layout_no_wrap(
        value.into(),
        FontId::new(16.0, FontFamily::Name(fonts::SEMIBOLD.into())),
        color.unwrap_or(p.text),
    );
    let caption = painter.layout_no_wrap(caption.to_owned(), FontId::proportional(11.5), p.muted);
    let gap = 1.0;
    let size = Vec2::new(
        value.size().x.max(caption.size().x).ceil(),
        (value.size().y + gap + caption.size().y).ceil(),
    );
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let caption_y = rect.top() + value.size().y + gap;
    ui.painter().galley(rect.left_top(), value, p.text);
    ui.painter()
        .galley(egui::pos2(rect.left(), caption_y.round()), caption, p.muted);
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
