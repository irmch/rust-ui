use imgui::{StyleVar, Ui};

use crate::Kit;
use crate::grid;
use crate::theme::ButtonKind;
use crate::tokens::size;

#[allow(unused_imports)]
use super::*;

// ---------------------------------------------------------------------------
// Buttons
// ---------------------------------------------------------------------------

/// Button of a variant with the default 32 px height. Width auto.
pub fn button(ui: &Ui, kit: &Kit, kind: ButtonKind, label: &str) -> bool {
    button_sized(ui, kit, kind, label, [0.0, size::CONTROL])
}

/// Button of a variant with an explicit size (`0.0` = auto, [`grid::FILL`] = fill).
pub fn button_sized(ui: &Ui, kit: &Kit, kind: ButtonKind, label: &str, sz: [f32; 2]) -> bool {
    let _f = ui.push_font(kit.fonts.mono13b);
    let _c = kind.push(ui);
    let _pad = ui.push_style_var(StyleVar::FramePadding([size::BUTTON_PAD_X, size::PAD_Y]));
    ui.button_with_size(label, sz)
}

/// Width [`button_small`] will take for `label`, for right-aligning.
pub fn button_small_width(ui: &Ui, kit: &Kit, label: &str) -> f32 {
    let _f = ui.push_font(kit.fonts.mono12);
    ui.calc_text_size(label)[0] + 20.0
}

/// Small 24 px button (toolbar "Copy", "Save").
pub fn button_small(ui: &Ui, kit: &Kit, kind: ButtonKind, label: &str) -> bool {
    let _f = ui.push_font(kit.fonts.mono12);
    let _c = kind.push(ui);
    let _pad = ui.push_style_var(StyleVar::FramePadding([10.0, 3.0]));
    ui.button_with_size(label, [0.0, size::SMALL])
}

/// Square icon button, 32 × 32 (or 24 × 24 with `small`). `glyph` is the
/// icon glyph of your icon font or a plain character like "×".
pub fn icon_button(ui: &Ui, kind: ButtonKind, id: &str, glyph: &str, small: bool) -> bool {
    let s = if small { size::SMALL } else { size::CONTROL };
    let _c = kind.push(ui);
    let _pad = ui.push_style_var(StyleVar::FramePadding([0.0, 0.0]));
    ui.button_with_size(format!("{glyph}##{id}"), [s, s])
}

/// Full-width 48 px call-to-action, uppercase ("LAUNCH 6 WINDOWS").
pub fn cta(ui: &Ui, kit: &Kit, kind: ButtonKind, label: &str) -> bool {
    let _f = ui.push_font(kit.fonts.mono13b);
    let _c = kind.push(ui);
    ui.button_with_size(label.to_uppercase(), [grid::FILL, size::CTA])
}

/// Disabled wrapper: 40 % alpha and no interaction.
pub fn disabled<R>(ui: &Ui, disabled: bool, f: impl FnOnce() -> R) -> R {
    if disabled {
        let _d = ui.begin_disabled(true);
        let _a = ui.push_style_var(StyleVar::Alpha(0.4));
        f()
    } else {
        f()
    }
}
