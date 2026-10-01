use imgui::{StyleVar, Ui};

use crate::Kit;
use crate::grid;
use crate::theme::ButtonKind;
use crate::tokens::{size, space};

#[allow(unused_imports)]
use super::*;

// ---------------------------------------------------------------------------
// Inputs
// ---------------------------------------------------------------------------

/// Text input 32 px tall with a placeholder. `width` 0 = fill.
pub fn input(
    ui: &Ui,
    kit: &Kit,
    id: &str,
    hint: &str,
    buf: &mut String,
    width: f32,
    bold: bool,
) -> bool {
    Input::new(id)
        .hint(hint)
        .width(width)
        .bold(bold)
        .show(ui, kit, buf)
}

/// Builder form of [`input`]: `Input::new("login").hint("Login").show(ui, kit, &mut s)`.
#[derive(Clone, Copy, Debug)]
pub struct Input<'a> {
    id: &'a str,
    hint: &'a str,
    width: f32,
    bold: bool,
}

impl<'a> Input<'a> {
    pub fn new(id: &'a str) -> Self {
        Self {
            id,
            hint: "",
            width: 0.0,
            bold: false,
        }
    }

    /// Placeholder shown while the buffer is empty.
    pub fn hint(mut self, hint: &'a str) -> Self {
        self.hint = hint;
        self
    }

    /// Width in px; `0.0` (the default) fills the content region.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// 13 px bold instead of regular.
    pub fn bold(mut self, bold: bool) -> Self {
        self.bold = bold;
        self
    }

    /// Draws the field; returns `true` when the text changed.
    pub fn show(self, ui: &Ui, kit: &Kit, buf: &mut String) -> bool {
        let _f = ui.push_font(if self.bold {
            kit.fonts.mono13b
        } else {
            kit.fonts.mono13
        });
        let _w = ui.push_item_width(if self.width > 0.0 {
            self.width
        } else {
            grid::FILL
        });
        ui.input_text(format!("##{}", self.id), buf)
            .hint(self.hint)
            .build()
    }
}

/// Path input followed by "Browse" and "Open" (artboard 03 · section 14).
/// Returns `(browse_clicked, open_clicked)`.
pub fn path_input(ui: &Ui, kit: &Kit, id: &str, buf: &mut String) -> (bool, bool) {
    let w = grid::input_group_width(ui, &[size::BTN_BROWSE, size::BTN_OPEN]);
    input(ui, kit, id, "Path to the game executable", buf, w, true);
    ui.same_line();
    let browse = button_sized(
        ui,
        kit,
        ButtonKind::Secondary,
        "Browse",
        [size::BTN_BROWSE, size::CONTROL],
    );
    ui.same_line();
    let open = button_sized(
        ui,
        kit,
        ButtonKind::Secondary,
        "Open",
        [size::BTN_OPEN, size::CONTROL],
    );
    (browse, open)
}

/// Multi-line text field `rows` lines tall (`width` `0.0` = fill); returns
/// `true` when the text changed.
pub fn textarea(ui: &Ui, kit: &Kit, id: &str, buf: &mut String, rows: u32, width: f32) -> bool {
    let _f = ui.push_font(kit.fonts.mono13);
    let w = if width > 0.0 {
        width
    } else {
        ui.content_region_avail()[0]
    };
    let h = ui.text_line_height() * rows.max(1) as f32 + 2.0 * space::S;
    let _pad = ui.push_style_var(StyleVar::FramePadding([size::PAD_X, space::S]));
    ui.input_text_multiline(format!("##{id}"), buf, [w, h])
        .build()
}
