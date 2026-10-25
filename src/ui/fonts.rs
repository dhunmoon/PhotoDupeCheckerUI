//! Bundled fonts: Inter for text (Regular / Medium / SemiBold) and Phosphor for icons.
//!
//! egui has no font-weight property, so heavier weights are registered as separate
//! named families; use [`medium`] / [`semibold`] to get them.

use std::sync::Arc;

use eframe::egui::{FontData, FontDefinitions, FontFamily, FontTweak, RichText};

const INTER_REGULAR: &[u8] = include_bytes!("../../assets/fonts/Inter-Regular.ttf");
const INTER_MEDIUM: &[u8] = include_bytes!("../../assets/fonts/Inter-Medium.ttf");
const INTER_SEMIBOLD: &[u8] = include_bytes!("../../assets/fonts/Inter-SemiBold.ttf");
const PHOSPHOR: &[u8] = include_bytes!("../../assets/fonts/Phosphor.ttf");
const PHOSPHOR_FILL: &[u8] = include_bytes!("../../assets/fonts/Phosphor-Fill.ttf");

pub(crate) const MEDIUM: &str = "inter-medium";
pub(crate) const SEMIBOLD: &str = "inter-semibold";
const ICON_FILL: &str = "phosphor-fill";

/// Builds the font set. egui's own fonts stay as a last fallback for rare symbols.
pub fn definitions() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    let icon_tweak = FontTweak {
        // Phosphor glyphs sit slightly high next to Inter; nudge them onto the text's centre.
        y_offset_factor: 0.08,
        ..Default::default()
    };
    let mut add = |name: &str, bytes: &'static [u8], tweak: Option<FontTweak>| {
        let mut data = FontData::from_static(bytes);
        if let Some(t) = tweak {
            data = data.tweak(t);
        }
        fonts.font_data.insert(name.to_owned(), Arc::new(data));
    };
    add("inter", INTER_REGULAR, None);
    add(MEDIUM, INTER_MEDIUM, None);
    add(SEMIBOLD, INTER_SEMIBOLD, None);
    add("phosphor", PHOSPHOR, Some(icon_tweak.clone()));
    add(ICON_FILL, PHOSPHOR_FILL, Some(icon_tweak));

    let fallbacks = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    let family = |first: &str, icons: &str| {
        let mut v = vec![first.to_owned(), icons.to_owned()];
        v.extend(fallbacks.iter().cloned());
        v
    };
    fonts
        .families
        .insert(FontFamily::Proportional, family("inter", "phosphor"));
    fonts
        .families
        .insert(FontFamily::Name(MEDIUM.into()), family(MEDIUM, "phosphor"));
    fonts.families.insert(
        FontFamily::Name(SEMIBOLD.into()),
        family(SEMIBOLD, "phosphor"),
    );
    fonts.families.insert(
        FontFamily::Name(ICON_FILL.into()),
        family("inter", ICON_FILL),
    );
    if let Some(mono) = fonts.families.get_mut(&FontFamily::Monospace) {
        mono.insert(1.min(mono.len()), "phosphor".into());
    }
    fonts
}

/// Text in Inter Medium.
pub fn medium(text: impl Into<String>) -> RichText {
    RichText::new(text).family(FontFamily::Name(MEDIUM.into()))
}

/// Text in Inter SemiBold (titles, emphasised numbers, primary buttons).
pub fn semibold(text: impl Into<String>) -> RichText {
    RichText::new(text).family(FontFamily::Name(SEMIBOLD.into()))
}

/// A filled Phosphor icon (outline icons come from the normal text font).
pub fn icon_fill(icon: &str) -> RichText {
    RichText::new(icon).family(FontFamily::Name(ICON_FILL.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_families_registered_with_existing_fonts() {
        let f = definitions();
        for fam in [
            FontFamily::Proportional,
            FontFamily::Monospace,
            FontFamily::Name(MEDIUM.into()),
            FontFamily::Name(SEMIBOLD.into()),
            FontFamily::Name(ICON_FILL.into()),
        ] {
            let names = f
                .families
                .get(&fam)
                .unwrap_or_else(|| panic!("{fam:?} missing"));
            for n in names {
                assert!(
                    f.font_data.contains_key(n),
                    "{fam:?} references unknown font {n}"
                );
            }
        }
        assert_eq!(f.families[&FontFamily::Proportional][0], "inter");
    }
}
