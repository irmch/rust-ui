use imgui::Ui;

use crate::fonts::Fonts;
use crate::grid;
use crate::theme::ButtonKind;
use crate::tokens::size;

#[allow(unused_imports)]
use super::*;

// ---------------------------------------------------------------------------
// Inputs
// ---------------------------------------------------------------------------

/// Text input 32 px tall with a placeholder. `width` 0 = fill.
pub fn input(ui: &Ui, f: &Fonts, id: &str, hint: &str, buf: &mut String, width: f32, bold: bool) -> bool {
    let _f = ui.push_font(if bold { f.mono13b } else { f.mono13 });
    let _w = ui.push_item_width(if width > 0.0 { width } else { grid::FILL });
    ui.input_text(format!("##{id}"), buf).hint(hint).build()
}

/// Path input followed by "Browse" and "Open" (artboard 03 · section 14).
/// Returns `(browse_clicked, open_clicked)`.
pub fn path_input(ui: &Ui, f: &Fonts, id: &str, buf: &mut String) -> (bool, bool) {
    let w = grid::input_group_width(ui, &[size::BTN_BROWSE, size::BTN_OPEN]);
    input(ui, f, id, "Path to the game executable", buf, w, true);
    ui.same_line();
    let browse = button_sized(ui, f, ButtonKind::Secondary, "Browse", [size::BTN_BROWSE, size::CONTROL]);
    ui.same_line();
    let open = button_sized(ui, f, ButtonKind::Secondary, "Open", [size::BTN_OPEN, size::CONTROL]);
    (browse, open)
}
