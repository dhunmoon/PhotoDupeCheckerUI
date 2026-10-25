//! App logo: `assets/logo/app-icon.png` (1024×1024 master, transparent background).
//! Replace that file and rebuild to change the window/taskbar icon and the header logo.

use eframe::egui;

const APP_ICON_PNG: &[u8] = include_bytes!("../../assets/logo/app-icon.png");

fn decode(max_side: u32) -> Option<image::RgbaImage> {
    let img = image::load_from_memory(APP_ICON_PNG).ok()?;
    Some(
        img.resize(max_side, max_side, image::imageops::FilterType::Lanczos3)
            .to_rgba8(),
    )
}

/// Window / taskbar icon (256 px).
pub fn window_icon() -> Option<egui::IconData> {
    let img = decode(256)?;
    Some(egui::IconData {
        width: img.width(),
        height: img.height(),
        rgba: img.into_raw(),
    })
}

/// Texture for the in-app header. Rendered at 28 pt, so 112 px stays sharp on 4× displays.
pub fn texture(ctx: &egui::Context) -> Option<egui::TextureHandle> {
    let img = decode(112)?;
    let color = egui::ColorImage::from_rgba_unmultiplied(
        [img.width() as usize, img.height() as usize],
        img.as_raw(),
    );
    Some(ctx.load_texture("app-logo", color, egui::TextureOptions::LINEAR))
}

#[cfg(test)]
mod tests {
    #[test]
    fn bundled_logo_decodes_and_is_square() {
        let img = super::decode(256).expect("assets/logo/app-icon.png must be a valid image");
        assert_eq!(img.width(), img.height(), "logo should be square");
        assert!(super::window_icon().is_some());
    }
}
