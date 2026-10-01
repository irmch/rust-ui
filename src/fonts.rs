//! Font atlas of the kit: JetBrains Mono in six sizes (artboard 01 · table 07).
//!
//! The TTF bytes are supplied by the application, e.g. with
//! `include_bytes!("../assets/JetBrainsMono-Regular.ttf")`, so the crate ships
//! no binary data.

use imgui::{Context, FontConfig, FontId, FontSource};

use crate::tokens::font;

/// Font files to load. `semibold` is used for captions and falls back to
/// `bold` when `None`.
pub struct FontFiles<'a> {
    pub regular: &'a [u8],
    pub bold: &'a [u8],
    pub semibold: Option<&'a [u8]>,
}

/// Handles of every font in the atlas.
#[derive(Clone, Copy, Debug)]
pub struct Fonts {
    /// 10 px semibold, tracked +0.14 em: captions, uppercase labels.
    pub mono10: FontId,
    /// 12 px regular: logs, table cells, hints.
    pub mono12: FontId,
    /// 13 px regular: body, inputs, menus.
    pub mono13: FontId,
    /// 13 px bold: labels, buttons, values.
    pub mono13b: FontId,
    /// 16 px bold: titles, dialog headers.
    pub mono16b: FontId,
    /// 20 px bold: app name, display numbers.
    pub mono20b: FontId,
}

/// Loads the atlas. `scale` is the HiDPI factor (1.0 on a 96 dpi screen);
/// pair it with `io.font_global_scale = 1.0 / scale` or render at that scale.
pub fn load(ctx: &mut Context, files: FontFiles<'_>, scale: f32) -> Fonts {
    let semibold = files.semibold.unwrap_or(files.bold);
    let add = |ctx: &mut Context, data: &[u8], px: f32, extra_x: f32| -> FontId {
        ctx.fonts().add_font(&[FontSource::TtfData {
            data,
            size_pixels: px * scale,
            config: Some(FontConfig {
                glyph_extra_spacing: [extra_x * scale, 0.0],
                oversample_h: 2,
                oversample_v: 1,
                pixel_snap_h: true,
                ..FontConfig::default()
            }),
        }])
    };
    // The first font added becomes the default one: body 13.
    let mono13 = add(ctx, files.regular, font::BODY, 0.0);
    let mono13b = add(ctx, files.bold, font::BODY, 0.0);
    let mono12 = add(ctx, files.regular, font::SMALL, 0.0);
    let mono10 = add(ctx, semibold, font::CAPTION, font::CAPTION_TRACKING);
    let mono16b = add(ctx, files.bold, font::TITLE, 0.0);
    let mono20b = add(ctx, files.bold, font::DISPLAY, 0.0);
    Fonts {
        mono10,
        mono12,
        mono13,
        mono13b,
        mono16b,
        mono20b,
    }
}
