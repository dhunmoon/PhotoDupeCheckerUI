//! Colours, global style and small reusable widgets that give the app a consistent look.

use eframe::egui::{
    self, Align2, Button, Color32, CornerRadius, FontId, Frame, Margin, Response, RichText, Sense,
    Stroke, TextStyle, Theme, Ui, Vec2,
};

pub const ACCENT: Color32 = Color32::from_rgb(92, 134, 255);
pub const KEEP: Color32 = Color32::from_rgb(64, 190, 110);
pub const REMOVE: Color32 = Color32::from_rgb(236, 88, 88);
pub const WARN: Color32 = Color32::from_rgb(240, 180, 60);
pub const SOURCE: Color32 = Color32::from_rgb(80, 160, 230);
pub const TARGET: Color32 = Color32::from_rgb(196, 128, 220);
pub const DANGER_FILL: Color32 = Color32::from_rgb(170, 48, 48);

/// Installs fonts sizes, spacing and colours for both light and dark mode.
pub fn install(ctx: &egui::Context) {
    ctx.all_styles_mut(|s| {
        s.spacing.item_spacing = egui::vec2(8.0, 6.0);
        s.spacing.button_padding = egui::vec2(10.0, 4.0);
        s.spacing.interact_size.y = 24.0;
        s.spacing.menu_margin = Margin::same(8);
        s.text_styles
            .insert(TextStyle::Heading, FontId::proportional(20.0));
        s.text_styles
            .insert(TextStyle::Body, FontId::proportional(14.0));
        s.text_styles
            .insert(TextStyle::Button, FontId::proportional(14.0));
        s.text_styles
            .insert(TextStyle::Small, FontId::proportional(11.5));
        s.text_styles
            .insert(TextStyle::Monospace, FontId::monospace(12.5));
        let v = &mut s.visuals;
        let r = CornerRadius::same(6);
        for w in [
            &mut v.widgets.noninteractive,
            &mut v.widgets.inactive,
            &mut v.widgets.hovered,
            &mut v.widgets.active,
            &mut v.widgets.open,
        ] {
            w.corner_radius = r;
        }
        v.window_corner_radius = CornerRadius::same(12);
        v.menu_corner_radius = CornerRadius::same(8);
        v.hyperlink_color = ACCENT;
        v.slider_trailing_fill = true;
        v.selection.bg_fill = ACCENT.gamma_multiply(0.55);
        v.selection.stroke = Stroke::new(1.0, Color32::WHITE);
    });
    ctx.style_mut_of(Theme::Dark, |s| {
        let v = &mut s.visuals;
        v.panel_fill = Color32::from_rgb(24, 26, 31);
        v.window_fill = Color32::from_rgb(32, 35, 42);
        v.extreme_bg_color = Color32::from_rgb(16, 18, 22);
        v.faint_bg_color = Color32::from_rgb(33, 36, 44);
        v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(50, 54, 64));
        v.widgets.inactive.weak_bg_fill = Color32::from_rgb(44, 48, 58);
        v.widgets.inactive.bg_fill = Color32::from_rgb(44, 48, 58);
        v.widgets.hovered.weak_bg_fill = Color32::from_rgb(56, 62, 76);
        v.widgets.hovered.bg_fill = Color32::from_rgb(56, 62, 76);
    });
    ctx.style_mut_of(Theme::Light, |s| {
        let v = &mut s.visuals;
        v.panel_fill = Color32::from_rgb(244, 245, 249);
        v.window_fill = Color32::WHITE;
        v.extreme_bg_color = Color32::from_rgb(232, 234, 240);
        v.faint_bg_color = Color32::WHITE;
        v.selection.stroke = Stroke::new(1.0, Color32::from_rgb(20, 30, 60));
        v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(218, 221, 229));
    });
}

/// Rounded "card" container used for the setup steps and dialogs.
pub fn card(ui: &Ui) -> Frame {
    Frame::new()
        .fill(ui.visuals().faint_bg_color)
        .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
        .corner_radius(10)
        .inner_margin(Margin::same(12))
}

/// Numbered step title: ①  Title
pub fn step_title(ui: &mut Ui, n: usize, title: &str) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(22.0), Sense::hover());
        ui.painter().circle_filled(rect.center(), 11.0, ACCENT);
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            n.to_string(),
            FontId::proportional(12.5),
            Color32::WHITE,
        );
        ui.label(RichText::new(title).size(16.0).strong());
    });
    ui.add_space(2.0);
}

/// Filled call-to-action button.
pub fn primary_button(text: impl Into<String>) -> Button<'static> {
    Button::new(RichText::new(text.into()).strong().color(Color32::WHITE))
        .fill(ACCENT)
        .corner_radius(8)
}

/// Filled red button for irreversible actions.
pub fn danger_button(text: impl Into<String>) -> Button<'static> {
    Button::new(RichText::new(text.into()).strong().color(Color32::WHITE))
        .fill(DANGER_FILL)
        .corner_radius(8)
}

/// Small pill-shaped, tinted label.
pub fn badge(ui: &mut Ui, text: impl Into<String>, color: Color32) -> Response {
    Frame::new()
        .fill(color.gamma_multiply(0.16))
        .corner_radius(10)
        .inner_margin(Margin::symmetric(8, 1))
        .show(ui, |ui| {
            ui.label(RichText::new(text.into()).color(color).small().strong())
        })
        .response
}

/// Clickable pill (e.g. the KEEP / REMOVE toggle on each file).
pub fn pill_button(text: impl Into<String>, color: Color32, filled: bool) -> Button<'static> {
    let fill = if filled {
        color.gamma_multiply(0.22)
    } else {
        Color32::TRANSPARENT
    };
    Button::new(RichText::new(text.into()).color(color).small().strong())
        .fill(fill)
        .stroke(Stroke::new(1.0, color.gamma_multiply(0.7)))
        .corner_radius(10)
}

/// The keep/remove pill for a file. Returns true when clicked.
pub fn mark_toggle(ui: &mut Ui, marked: bool) -> bool {
    let (text, color) = if marked {
        ("✖ REMOVE", REMOVE)
    } else {
        ("✔ KEEP", KEEP)
    };
    ui.add(pill_button(text, color, true).min_size(Vec2::new(78.0, 0.0)))
        .on_hover_text(if marked {
            "Marked: the next action will be applied to this file. Click to keep it instead."
        } else {
            "Kept: this file will not be touched. Click to mark it for removal."
        })
        .clicked()
}

/// A row of mutually exclusive buttons. Returns true if the value changed.
pub fn segmented<T: PartialEq + Copy>(ui: &mut Ui, value: &mut T, options: &[(T, &str)]) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        for (v, label) in options {
            let selected = *value == *v;
            let text = if selected {
                RichText::new(*label).strong().color(Color32::WHITE)
            } else {
                RichText::new(*label)
            };
            let mut b = Button::new(text).corner_radius(6);
            if selected {
                b = b.fill(ACCENT);
            }
            if ui.add(b).clicked() && !selected {
                *value = *v;
                changed = true;
            }
        }
    });
    changed
}

/// Headline number with a caption underneath (used in the top bar).
pub fn stat(ui: &mut Ui, value: impl Into<String>, caption: &str, color: Option<Color32>) {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        let mut t = RichText::new(value.into()).size(16.0).strong();
        if let Some(c) = color {
            t = t.color(c);
        }
        ui.label(t);
        ui.label(RichText::new(caption).small().weak());
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
}
