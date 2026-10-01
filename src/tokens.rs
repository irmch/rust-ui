//! Design tokens of the kit. Every number here is taken from the Figma-like
//! canvas (artboards 01 Foundations and 02 Grid) so the Rust side and the
//! design stay in sync.

/// RGBA colour in the 0.0..=1.0 range, the format `imgui::Style::colors` uses.
pub type Rgba = [f32; 4];

/// Builds an opaque colour from a `0xRRGGBB` literal.
pub const fn hex(rgb: u32) -> Rgba {
    [
        ((rgb >> 16) & 0xff) as f32 / 255.0,
        ((rgb >> 8) & 0xff) as f32 / 255.0,
        (rgb & 0xff) as f32 / 255.0,
        1.0,
    ]
}

/// Same as [`hex`] but with an explicit alpha.
pub const fn hexa(rgb: u32, alpha: f32) -> Rgba {
    let c = hex(rgb);
    [c[0], c[1], c[2], alpha]
}

/// Colour tokens. The comment on each one names the `StyleColor` slots it
/// feeds in [`crate::theme::apply`].
pub mod color {
    use super::{Rgba, hex, hexa};

    /// WindowBg · ChildBg
    pub const BG0: Rgba = hex(0x121212);
    /// FrameBg · PopupBg · ScrollbarBg
    pub const BG1: Rgba = hex(0x181818);
    /// TitleBg · MenuBarBg · TableHeaderBg · secondary button
    pub const BG2: Rgba = hex(0x1e1e1e);
    /// FrameBgHovered · HeaderHovered · TabActive
    pub const BG3: Rgba = hex(0x262626);
    /// FrameBgActive · ButtonHovered · slider track
    pub const BG4: Rgba = hex(0x2e2e2e);
    /// Border · Separator · TableBorderLight
    pub const LINE: Rgba = hex(0x2a2a2a);
    /// Frame border · ScrollbarGrab
    pub const LINE2: Rgba = hex(0x3a3a3a);
    /// Header · TextSelectedBg · selected row
    pub const SEL: Rgba = hexa(0xededed, 0.08);
    /// Text
    pub const FG: Rgba = hex(0xececec);
    /// Secondary text · inactive tab
    pub const FG2: Rgba = hex(0xa8a8a8);
    /// TextDisabled · captions · placeholder
    pub const FG3: Rgba = hex(0x8a8a8a);
    /// CheckMark · SliderGrab · primary button
    pub const ACCENT: Rgba = hex(0xededed);
    /// Primary button hovered
    pub const ACCENT_HOVER: Rgba = hex(0xffffff);
    /// Primary button active
    pub const ACCENT_ACTIVE: Rgba = hex(0xcfcfcf);
    /// PlotLines · status ready
    pub const OK: Rgba = hex(0x5cc98a);
    /// Status warning
    pub const WARN: Rgba = hex(0xe3b341);
    /// Status error · danger button text
    pub const ERR: Rgba = hex(0xe8605c);
    /// Status info · links
    pub const INFO: Rgba = hex(0x5aa0e0);

    /// Danger button background / border.
    pub const DANGER_BG: Rgba = hex(0x1e1616);
    pub const DANGER_BORDER: Rgba = hex(0x4a2a2a);
    pub const DANGER_BG_HOVER: Rgba = hex(0x2a1a1a);

    /// Tinted backgrounds and borders used by tags and banners.
    pub const OK_BG: Rgba = hex(0x132018);
    pub const OK_BORDER: Rgba = hex(0x2b5a3d);
    pub const WARN_BG: Rgba = hex(0x201c12);
    pub const WARN_BORDER: Rgba = hex(0x5a4a1e);
    pub const ERR_BG: Rgba = hex(0x201414);
    pub const ERR_BORDER: Rgba = hex(0x5a2a28);
    pub const INFO_BG: Rgba = hex(0x121a22);
    pub const INFO_BORDER: Rgba = hex(0x24435e);

    /// Modal overlay (bg-0 at 70 %).
    pub const DIM: Rgba = hexa(0x121212, 0.70);
    /// Popup shadow (black at 60 %).
    pub const SHADOW: Rgba = [0.0, 0.0, 0.0, 0.6];
    pub const TRANSPARENT: Rgba = [0.0, 0.0, 0.0, 0.0];
}

/// Spacing scale in logical pixels. Everything is a multiple of 4,
/// control sizes are multiples of 8.
pub mod space {
    pub const XS: f32 = 4.0;
    pub const S: f32 = 8.0;
    pub const M: f32 = 12.0;
    /// Column gutter.
    pub const L: f32 = 16.0;
    /// Window padding.
    pub const XL: f32 = 24.0;
    pub const XXL: f32 = 32.0;
    pub const XXXL: f32 = 48.0;
}

/// Fixed heights and sizes.
pub mod size {
    /// Title bar and status strip height.
    pub const BAR: f32 = 48.0;
    /// Menu bar height.
    pub const MENU_BAR: f32 = 32.0;
    /// Default control height (button, input, combo, slider row).
    pub const CONTROL: f32 = 32.0;
    /// Small control height (compact buttons, pagination).
    pub const SMALL: f32 = 24.0;
    /// Full-width call-to-action button.
    pub const CTA: f32 = 48.0;
    /// Menu, tree, list and sidebar row.
    pub const ROW: f32 = 28.0;
    /// Table row.
    pub const TABLE_ROW: f32 = 32.0;
    /// Log line.
    pub const LINE: f32 = 20.0;
    /// Checkbox / radio box.
    pub const CHECK: f32 = 18.0;
    /// Slider knob diameter.
    pub const KNOB: f32 = 16.0;
    /// Slider track thickness.
    pub const TRACK: f32 = 4.0;
    /// Switch size.
    pub const SWITCH: [f32; 2] = [36.0, 20.0];
    /// Tag / badge height.
    pub const TAG: f32 = 20.0;
    pub const SCROLLBAR: f32 = 8.0;
    pub const ICON: f32 = 16.0;
    /// Corner radius of frames, popups and tabs. Windows use 0.
    pub const RADIUS: f32 = 2.0;
    pub const BORDER: f32 = 1.0;
    /// Horizontal padding inside a 32 px control.
    pub const PAD_X: f32 = 12.0;
    /// Vertical padding inside a 32 px control (32 − 20 line) / 2.
    pub const PAD_Y: f32 = 6.0;
    /// Button horizontal padding.
    pub const BUTTON_PAD_X: f32 = 16.0;
    /// Form sub-grid: label column, value column.
    pub const FORM_LABEL: f32 = 96.0;
    pub const FORM_VALUE: f32 = 64.0;
    /// Input group buttons: "Browse" and "Open".
    pub const BTN_BROWSE: f32 = 88.0;
    pub const BTN_OPEN: f32 = 72.0;
}

/// Font sizes of the atlas (see [`crate::fonts`]).
pub mod font {
    pub const CAPTION: f32 = 10.0;
    pub const SMALL: f32 = 12.0;
    pub const BODY: f32 = 13.0;
    pub const TITLE: f32 = 16.0;
    pub const DISPLAY: f32 = 20.0;
    /// Letter spacing of captions, in px at 10 px size (0.14 em).
    pub const CAPTION_TRACKING: f32 = 1.4;
}
