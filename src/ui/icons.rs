//! The icon set used across the app: a curated subset of [Phosphor Icons](https://phosphoricons.com)
//! (MIT licensed, see `assets/fonts/Phosphor-LICENSE.txt`).
//!
//! Icons are glyphs in the bundled `Phosphor.ttf`, so they can be mixed into any text
//! (`format!("{} Add", icons::PLUS)`) and follow the text colour. Use
//! [`super::fonts::icon_fill`] for the filled variant.
//!
//! Only add icons from the same font so the whole UI stays visually consistent.
//! Codepoints: <https://github.com/phosphor-icons/web> (or the `egui-phosphor` crate).

// Folders & files
pub const FOLDER: &str = "\u{E25A}";
pub const FOLDER_OPEN: &str = "\u{E256}";
pub const FOLDERS: &str = "\u{E260}";
pub const FILE: &str = "\u{E230}";
pub const FILE_TEXT: &str = "\u{E23A}";
pub const FILE_ZIP: &str = "\u{E958}";
pub const IMAGE: &str = "\u{E2CA}";
pub const FILM_STRIP: &str = "\u{E792}";
pub const MUSIC_NOTES: &str = "\u{E340}";
pub const TRAY_ARROW_DOWN: &str = "\u{E010}";

// Match methods
pub const FINGERPRINT: &str = "\u{E23E}";
pub const IMAGES: &str = "\u{E836}";
pub const TEXT_AA: &str = "\u{E6EE}";
pub const IDENTIFICATION_CARD: &str = "\u{E2C8}";
pub const RULER: &str = "\u{E6B8}";

// Generic actions
pub const PLUS: &str = "\u{E3D4}";
pub const X: &str = "\u{E4F6}";
pub const CHECK: &str = "\u{E182}";
pub const CHECK_CIRCLE: &str = "\u{E184}";
pub const X_CIRCLE: &str = "\u{E4F8}";
pub const MAGNIFYING_GLASS: &str = "\u{E30C}";
pub const ARROW_CLOCKWISE: &str = "\u{E036}";
pub const STOP: &str = "\u{E46C}";
pub const SLIDERS: &str = "\u{E434}";
pub const FUNNEL: &str = "\u{E266}";
pub const SORT: &str = "\u{E446}";
pub const CARET_DOWN: &str = "\u{E136}";
pub const CARET_LEFT: &str = "\u{E138}";
pub const CARET_RIGHT: &str = "\u{E13A}";
pub const ARROW_RIGHT: &str = "\u{E06C}";
pub const ARROW_SQUARE_OUT: &str = "\u{E5DE}";

// Selection & file actions
pub const MAGIC_WAND: &str = "\u{E6B6}";
pub const SELECTION_INVERSE: &str = "\u{E744}";
pub const ERASER: &str = "\u{E21E}";
pub const TRASH: &str = "\u{E4A6}";
pub const COPY: &str = "\u{E1CA}";
pub const LINK: &str = "\u{E2E2}";
pub const LINK_SIMPLE: &str = "\u{E2E6}";
pub const FILE_CSV: &str = "\u{EB1C}";
pub const CLIPBOARD_TEXT: &str = "\u{E198}";
pub const BROOM: &str = "\u{EC54}";

// Views
pub const ROWS: &str = "\u{E5A2}";
pub const SQUARES_FOUR: &str = "\u{E464}";
pub const COLUMNS: &str = "\u{E546}";

// Status & chrome
pub const WARNING: &str = "\u{E4E0}";
pub const INFO: &str = "\u{E2CE}";
pub const STAR: &str = "\u{E46A}";
pub const SEAL_CHECK: &str = "\u{E606}";
pub const TERMINAL: &str = "\u{EAE8}";
pub const SUN: &str = "\u{E472}";
pub const MOON: &str = "\u{E330}";

use crate::model::{FileKind, Method};

pub fn for_kind(k: FileKind) -> &'static str {
    match k {
        FileKind::Image => IMAGE,
        FileKind::Video => FILM_STRIP,
        FileKind::Audio => MUSIC_NOTES,
        FileKind::Document => FILE_TEXT,
        FileKind::Archive => FILE_ZIP,
        FileKind::Other => FILE,
    }
}

pub fn for_method(m: Method) -> &'static str {
    match m {
        Method::Content => FINGERPRINT,
        Method::SimilarImage => IMAGES,
        Method::FileName => TEXT_AA,
        Method::NameAndSize => IDENTIFICATION_CARD,
        Method::Size => RULER,
    }
}
