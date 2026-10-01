use imgui::Ui;

use crate::Kit;
use crate::tokens::{color, size, space, Rgba};

#[allow(unused_imports)]
use super::*;

// ---------------------------------------------------------------------------
// Status pieces
// ---------------------------------------------------------------------------

/// Kinds of a tag / badge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TagKind {
    Neutral,
    Solid,
    Ok,
    Warn,
    Err,
    Info,
}

impl TagKind {
    pub(super) fn colors(self) -> (Rgba, Rgba, Rgba) {
        match self {
            TagKind::Neutral => (color::TRANSPARENT, color::LINE2, color::FG2),
            TagKind::Solid => (color::ACCENT, color::ACCENT, color::BG0),
            TagKind::Ok => (color::OK_BG, color::OK_BORDER, color::OK),
            TagKind::Warn => (color::WARN_BG, color::WARN_BORDER, color::WARN),
            TagKind::Err => (color::ERR_BG, color::ERR_BORDER, color::ERR),
            TagKind::Info => (color::INFO_BG, color::INFO_BORDER, color::INFO),
        }
    }
}

/// 20 px uppercase tag ("RUNNING").
pub fn tag(ui: &Ui, kit: &Kit, kind: TagKind, text: &str) {
    let _f = ui.push_font(kit.fonts.mono10);
    let txt = upper(text);
    let tw = ui.calc_text_size(&txt)[0];
    let w = tw + 2.0 * space::S;
    let p = ui.cursor_screen_pos();
    ui.dummy([w, size::TAG]);
    let (bg, border, fg) = kind.colors();
    let al = style_alpha(ui);
    let dl = ui.get_window_draw_list();
    let q = [p[0] + w, p[1] + size::TAG];
    dl.add_rect(p, q, fade(bg, al)).rounding(size::RADIUS).filled(true).build();
    dl.add_rect(p, q, fade(border, al)).rounding(size::RADIUS).build();
    let ty = p[1] + ((size::TAG - ui.text_line_height()) / 2.0).round();
    dl.add_text([p[0] + space::S, ty], fade(fg, al), &txt);
}

/// 8 px status dot followed by text.
pub fn status_dot(ui: &Ui, col: Rgba, text: &str) {
    let p = ui.cursor_screen_pos();
    let lh = ui.text_line_height();
    ui.dummy([8.0, lh]);
    ui.get_window_draw_list()
        .add_circle([p[0] + 4.0, p[1] + lh / 2.0], 4.0, fade(col, style_alpha(ui)))
        .filled(true)
        .build();
    ui.same_line_with_spacing(0.0, 6.0);
    ui.text(text);
}

/// Stat item of the status strip: `CAPTION  Value unit`. `col` colours the value.
pub fn stat(ui: &Ui, kit: &Kit, cap: &str, value: &str, unit: &str, col: Rgba) {
    // The cursor marks the top of the 13 px value; the 10 px caption and the
    // unit are placed explicitly on the same baseline (no `same_line`, which
    // would snap back to whatever line y imgui remembers).
    let [x, y] = ui.cursor_pos();
    let asc_val = ascent(ui, kit.fonts.mono13b);
    let cap_txt = upper(cap);
    let cap_w = {
        let _f = ui.push_font(kit.fonts.mono10);
        ui.set_cursor_pos([x, y + asc_val - ascent(ui, kit.fonts.mono10)]);
        ui.text_colored(color::FG3, &cap_txt);
        ui.calc_text_size(&cap_txt)[0]
    };
    let x = x + cap_w + space::S;
    let val_w = {
        let _f = ui.push_font(kit.fonts.mono13b);
        ui.set_cursor_pos([x, y]);
        ui.text_colored(col, value);
        ui.calc_text_size(value)[0]
    };
    if !unit.is_empty() {
        ui.set_cursor_pos([x + val_w + space::XS, y + asc_val - ascent(ui, kit.fonts.mono13)]);
        ui.text_colored(color::FG3, unit);
    }
}

/// 1 px vertical divider of `h` px on the current line.
pub fn vdivider(ui: &Ui, h: f32) {
    let p = ui.cursor_screen_pos();
    ui.dummy([1.0, h]);
    ui.get_window_draw_list()
        .add_line(p, [p[0], p[1] + h], fade(color::LINE, style_alpha(ui)))
        .build();
}

/// Draws a 1 px line along the bottom edge of the current window.
///
/// Acquires and releases the window draw list itself: only one `DrawListMut`
/// may exist at a time, and the widgets drawn after this call need their own.
pub(super) fn bottom_border(ui: &Ui) {
    let p = ui.window_pos();
    let s = ui.window_size();
    ui.get_window_draw_list()
        .add_line([p[0], p[1] + s[1] - 1.0], [p[0] + s[0], p[1] + s[1] - 1.0], fade(color::LINE, style_alpha(ui)))
        .build();
}
