use imgui::Ui;

use crate::Kit;
use crate::tokens::{color, space, Rgba};

#[allow(unused_imports)]
use super::*;

// ---------------------------------------------------------------------------
// Text
// ---------------------------------------------------------------------------

/// Current `style.alpha`, the value [`disabled`] and page fades push.
/// imgui applies it only to style-slot colours; anything drawn through the
/// draw list with an explicit colour has to apply it itself via [`fade`].
pub fn style_alpha(ui: &Ui) -> f32 {
    let _ = ui;
    // SAFETY: a Ui exists, so the current context and its style are valid.
    unsafe { (*imgui::sys::igGetStyle()).Alpha }
}

/// `c` with its alpha multiplied by `alpha` (see [`style_alpha`]).
pub fn fade(c: Rgba, alpha: f32) -> Rgba {
    [c[0], c[1], c[2], c[3] * alpha]
}

/// Ascent of `font` in screen pixels, for aligning texts of different sizes
/// on one baseline (`y_small = y_big + ascent(big) - ascent(small)`).
pub(super) fn ascent(ui: &Ui, font: imgui::FontId) -> f32 {
    let a = ui.fonts().get_font(font).map_or(0.0, |f| f.ascent * f.scale);
    (a * ui.io().font_global_scale).round()
}

/// Caption: 10 px semibold uppercase, fg-3 ("GAME PATH").
pub fn caption(ui: &Ui, kit: &Kit, text: &str) {
    let _f = ui.push_font(kit.fonts.mono10);
    ui.text_colored(color::FG3, text.to_uppercase());
}

/// Section header followed by the 12 px gap of the rhythm.
pub fn section(ui: &Ui, kit: &Kit, text: &str) {
    caption(ui, kit, text);
    ui.dummy([0.0, space::M - space::S]);
}

/// Hint text: 12 px regular fg-3, drawn on the same line as the previous item.
pub fn hint_inline(ui: &Ui, kit: &Kit, text: &str) {
    ui.same_line();
    let _f = ui.push_font(kit.fonts.mono12);
    ui.text_colored(color::FG3, text);
}

/// Body text in a given colour with the bold font.
pub fn text_bold(ui: &Ui, kit: &Kit, text: &str, col: Rgba) {
    let _f = ui.push_font(kit.fonts.mono13b);
    ui.text_colored(col, text);
}

/// Secondary text (fg-2).
pub fn text_muted(ui: &Ui, text: &str) {
    ui.text_colored(color::FG2, text);
}

/// Log line: `[0.084]` in fg-3 then the message (artboard 05 · section 02).
pub fn log_line(ui: &Ui, kit: &Kit, seconds: f32, message: &str, col: Option<Rgba>) {
    let _f = ui.push_font(kit.fonts.mono12);
    ui.text_colored(color::FG3, format!("[{seconds:.3}]"));
    ui.same_line();
    ui.text_colored(col.unwrap_or(color::FG), message);
}

/// "✓ Verified · Path of Exile 2" line under the game path input.
pub fn verified_line(ui: &Ui, kit: &Kit, ok: bool, label: &str, detail: &str) {
    let (mark, col) = if ok { ("✓", color::OK) } else { ("✕", color::ERR) };
    text_bold(ui, kit, &format!("{mark} {label}"), col);
    ui.same_line();
    ui.text_colored(color::FG3, format!("· {detail}"));
}
