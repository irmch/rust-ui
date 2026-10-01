//! Font atlas of the kit: JetBrains Mono in six sizes (artboard 01 · table 07).
//!
//! The TTF bytes are supplied by the application, e.g. with
//! `include_bytes!("../assets/JetBrainsMono-Regular.ttf")`, so the crate ships
//! no binary data.

use imgui::{Context, FontConfig, FontGlyphRanges, FontId, FontSource};

use crate::tokens::font;

/// Font files to load. `semibold` is used for captions and falls back to
/// `bold` when `None`.
pub struct FontFiles<'a> {
    pub regular: &'a [u8],
    pub bold: &'a [u8],
    pub semibold: Option<&'a [u8]>,
    /// Icon font merged into the body, bold, small and title fonts (not into
    /// the 10 / 20 px ones, to keep the atlas small): e.g. Lucide with its
    /// Private Use Area range. Glyphs render inline with `ui.text`.
    pub icons: Option<IconFont<'a>>,
}

/// An icon font to merge into the text fonts, see [`FontFiles::icons`].
pub struct IconFont<'a> {
    pub data: &'a [u8],
    /// Zero-terminated glyph range pairs, e.g. `&[0xE000, 0xE6FF, 0]`.
    pub ranges: &'static [u32],
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

/// Code points baked into the atlas. Besides Basic Latin + Latin-1 (the
/// imgui default) the widgets use `–` `·` (General Punctuation), `□`
/// (Geometric Shapes) and `✓` `✕` (Dingbats).
const GLYPH_RANGES: &[u32] = &[
    0x0020, 0x00FF, // Basic Latin + Latin-1 Supplement (× · are here)
    0x0400, 0x052F, // Cyrillic + Cyrillic Supplement
    0x2010, 0x2027, // General Punctuation: dashes, bullets, ellipsis
    0x2500, 0x25FF, // Box Drawing, Block Elements, Geometric Shapes (□)
    0x2700, 0x27BF, // Dingbats (✓ ✕)
    0,
];

/// Loads the atlas. `scale` is the HiDPI factor (1.0 on a 96 dpi screen);
/// pair it with `io.font_global_scale = 1.0 / scale` or render at that scale.
pub fn load(ctx: &mut Context, files: FontFiles<'_>, scale: f32) -> Fonts {
    load_sized(ctx, files, scale, 1.0)
}

/// [`load`] with every size multiplied by `size_mul`: an app that wants a
/// 14 px body instead of 13 passes `14.0 / 13.0`. Control heights stay
/// the kit's, so keep it near 1.
pub fn load_sized(ctx: &mut Context, files: FontFiles<'_>, scale: f32, size_mul: f32) -> Fonts {
    let scale = scale * size_mul;
    let semibold = files.semibold.unwrap_or(files.bold);
    let icons = files.icons.as_ref();
    let add = |ctx: &mut Context, data: &[u8], px: f32, extra_x: f32, with_icons: bool| -> FontId {
        let mut sources = vec![FontSource::TtfData {
            data,
            size_pixels: px * scale,
            config: Some(FontConfig {
                glyph_extra_spacing: [extra_x * scale, 0.0],
                oversample_h: 2,
                oversample_v: 1,
                pixel_snap_h: true,
                glyph_ranges: FontGlyphRanges::from_slice(GLYPH_RANGES),
                ..FontConfig::default()
            }),
        }];
        if let (true, Some(ic)) = (with_icons, icons) {
            // Every source after the first is merged into the same font.
            sources.push(FontSource::TtfData {
                data: ic.data,
                size_pixels: px * scale,
                config: Some(FontConfig {
                    oversample_h: 2,
                    oversample_v: 2,
                    pixel_snap_h: true,
                    glyph_ranges: FontGlyphRanges::from_slice(ic.ranges),
                    ..FontConfig::default()
                }),
            });
        }
        ctx.fonts().add_font(&sources)
    };
    // The first font added becomes the default one: body 13.
    let mono13 = add(ctx, files.regular, font::BODY, 0.0, true);
    let mono13b = add(ctx, files.bold, font::BODY, 0.0, true);
    let mono12 = add(ctx, files.regular, font::SMALL, 0.0, true);
    let mono10 = add(ctx, semibold, font::CAPTION, font::CAPTION_TRACKING, false);
    let mono16b = add(ctx, files.bold, font::TITLE, 0.0, true);
    let mono20b = add(ctx, files.bold, font::DISPLAY, 0.0, false);
    Fonts {
        mono10,
        mono12,
        mono13,
        mono13b,
        mono16b,
        mono20b,
    }
}
