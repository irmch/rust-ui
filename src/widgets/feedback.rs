use imgui::{StyleColor, StyleVar, Ui};

use crate::Kit;
use crate::tokens::{Rgba, color, size, space};

#[allow(unused_imports)]
use super::*;

// ---------------------------------------------------------------------------
// Modal, tooltip
// ---------------------------------------------------------------------------

/// Asks for the modal `title` to open on the next frame (call from a click).
pub fn open_modal(ui: &Ui, title: &str) {
    ui.open_popup(title);
}

/// Centred modal dialog: bg-1 panel with a 1 px line, 24 px padding, the
/// title as a 16 px bold heading, `body` below. Draw it every frame; the
/// body runs only while the modal is open. `size` `None` = auto.
pub fn modal(ui: &Ui, kit: &Kit, title: &str, size_: Option<[f32; 2]>, body: impl FnOnce(&Ui)) {
    let _bg = ui.push_style_color(StyleColor::PopupBg, color::BG1);
    let _bd = ui.push_style_color(StyleColor::Border, color::LINE2);
    let _dim = ui.push_style_color(StyleColor::ModalWindowDimBg, color::DIM);
    let _rounding = ui.push_style_var(StyleVar::WindowRounding(size::RADIUS));
    let _border = ui.push_style_var(StyleVar::WindowBorderSize(size::BORDER));
    let _pad = ui.push_style_var(StyleVar::WindowPadding([space::XL, space::XL]));
    // The host may set a global WindowMinSize for its main window; a dialog
    // must not inherit it.
    let _min = ui.push_style_var(StyleVar::WindowMinSize([0.0, 0.0]));
    if let Some(s) = size_ {
        unsafe {
            imgui::sys::igSetNextWindowSize(
                imgui::sys::ImVec2 { x: s[0], y: s[1] },
                imgui::Condition::Appearing as i32,
            );
        }
    }
    ui.modal_popup_config(title)
        .title_bar(false)
        .resizable(false)
        .movable(false)
        .always_auto_resize(size_.is_none())
        .build(|| {
            {
                let _f = ui.push_font(kit.fonts.mono16b);
                ui.text(title.split("##").next().unwrap_or(""));
            }
            ui.dummy([0.0, space::M]);
            body(ui);
        });
}

/// Tooltip with the kit's panel look.
pub fn tooltip(ui: &Ui, kit: &Kit, text: &str) {
    tooltip_with(ui, kit, |ui| ui.text_colored(color::FG, text));
}

/// Tooltip with the kit's panel look and arbitrary content.
pub fn tooltip_with(ui: &Ui, kit: &Kit, body: impl FnOnce(&Ui)) {
    let _ = kit;
    let _bg = ui.push_style_color(StyleColor::PopupBg, color::BG2);
    let _bd = ui.push_style_color(StyleColor::Border, color::LINE2);
    let _rounding = ui.push_style_var(StyleVar::WindowRounding(size::RADIUS));
    let _border = ui.push_style_var(StyleVar::WindowBorderSize(size::BORDER));
    let _pad = ui.push_style_var(StyleVar::WindowPadding([size::PAD_X, space::S]));
    let _min = ui.push_style_var(StyleVar::WindowMinSize([0.0, 0.0]));
    ui.tooltip(|| body(ui));
}

/// [`tooltip`] when the previous item is hovered.
pub fn tooltip_on_hover(ui: &Ui, kit: &Kit, text: &str) {
    if ui.is_item_hovered() {
        tooltip(ui, kit, text);
    }
}

// ---------------------------------------------------------------------------
// Spinner, empty state, avatar
// ---------------------------------------------------------------------------

/// Rotating 270° arc of `size_` px (indeterminate wait).
pub fn spinner(ui: &Ui, size_: f32, col: Rgba) {
    let p = ui.cursor_screen_pos();
    let r = size_ / 2.0 - 1.0;
    let c = [p[0] + size_ / 2.0, p[1] + size_ / 2.0];
    let t = ui.time() as f32 * 4.0;
    let segs = 24;
    let arc = std::f32::consts::PI * 1.5;
    let al = style_alpha(ui);
    let dl = ui.get_window_draw_list();
    for i in 0..segs {
        let a0 = t + i as f32 / segs as f32 * arc;
        let a1 = t + (i + 1) as f32 / segs as f32 * arc;
        let k = (i as f32 / segs as f32).powf(0.7);
        let line = [col[0], col[1], col[2], col[3] * k * al];
        dl.add_line(
            [c[0] + a0.cos() * r, c[1] + a0.sin() * r],
            [c[0] + a1.cos() * r, c[1] + a1.sin() * r],
            line,
        )
        .thickness(2.0)
        .build();
    }
    ui.dummy([size_, size_]);
}

/// Centred placeholder for empty views: optional glyph, title, dim
/// description. Sits a third of the way down the available region.
pub fn empty_state(ui: &Ui, kit: &Kit, icon: Option<&str>, title: &str, description: &str) {
    let avail = ui.content_region_avail();
    let block_h = 4.0 * space::XL;
    if avail[1] > block_h {
        ui.dummy([0.0, (avail[1] - block_h) * 0.33]);
    }
    let center = avail[0] / 2.0;
    let dl = ui.get_window_draw_list();
    if let Some(g) = icon {
        // 16 px bold: the largest font the icon font is merged into.
        let _f = ui.push_font(kit.fonts.mono16b);
        let s = ui.calc_text_size(g);
        let p = ui.cursor_screen_pos();
        dl.add_text([p[0] + center - s[0] / 2.0, p[1]], color::FG3, g);
        ui.dummy([0.0, s[1] + space::M]);
    }
    {
        let _f = ui.push_font(kit.fonts.mono13b);
        let s = ui.calc_text_size(title);
        let p = ui.cursor_screen_pos();
        dl.add_text([p[0] + center - s[0] / 2.0, p[1]], color::FG, title);
        ui.dummy([0.0, s[1] + space::S]);
    }
    if !description.is_empty() {
        let s = ui.calc_text_size(description);
        let p = ui.cursor_screen_pos();
        dl.add_text([p[0] + center - s[0] / 2.0, p[1]], color::FG3, description);
        ui.dummy([0.0, s[1] + space::M]);
    }
}

/// Square avatar of `size_` px showing `text` (initials or a glyph), bg-2
/// with a line border, or accent fill with dark text when `accent`.
pub fn avatar(ui: &Ui, kit: &Kit, text: &str, size_: f32, accent: bool) {
    let p = ui.cursor_screen_pos();
    let al = style_alpha(ui);
    let (bg, border, fg) = if accent {
        (color::ACCENT, color::ACCENT, color::BG0)
    } else {
        (color::BG2, color::LINE2, color::FG2)
    };
    let dl = ui.get_window_draw_list();
    dl.add_rect(p, [p[0] + size_, p[1] + size_], fade(bg, al))
        .rounding(size::RADIUS)
        .filled(true)
        .build();
    dl.add_rect(p, [p[0] + size_, p[1] + size_], fade(border, al))
        .rounding(size::RADIUS)
        .build();
    let _f = ui.push_font(if size_ >= 40.0 {
        kit.fonts.mono16b
    } else {
        kit.fonts.mono13b
    });
    let s = ui.calc_text_size(text);
    dl.add_text(
        [
            p[0] + ((size_ - s[0]) / 2.0).round(),
            p[1] + ((size_ - s[1]) / 2.0).round(),
        ],
        fade(fg, al),
        text,
    );
    ui.dummy([size_, size_]);
}

// ---------------------------------------------------------------------------
// Bars and rules
// ---------------------------------------------------------------------------

/// Thin coloured bar (HP / MP / CP…): `col` fill over the same colour at
/// 18 %, `height` px tall (4 by default when `0.0`), `width` `0.0` = fill.
pub fn stat_bar(ui: &Ui, fraction: f32, col: Rgba, width: f32, height: f32) {
    let w = if width > 0.0 {
        width
    } else {
        ui.content_region_avail()[0]
    };
    let h = if height > 0.0 { height } else { size::TRACK };
    let p = ui.cursor_screen_pos();
    let al = style_alpha(ui);
    let dl = ui.get_window_draw_list();
    dl.add_rect(
        p,
        [p[0] + w, p[1] + h],
        fade([col[0], col[1], col[2], 0.18], al),
    )
    .rounding(h / 2.0)
    .filled(true)
    .build();
    let f = fraction.clamp(0.0, 1.0);
    if f > 0.0 {
        dl.add_rect(p, [p[0] + w * f, p[1] + h], fade(col, al))
            .rounding(h / 2.0)
            .filled(true)
            .build();
    }
    ui.dummy([w, h]);
}

/// Horizontal rule with 8 px above and below (tighter than [`crate::grid::section_gap`]).
pub fn divider(ui: &Ui) {
    ui.dummy([0.0, space::S]);
    let p = ui.cursor_screen_pos();
    let w = ui.content_region_avail()[0];
    ui.get_window_draw_list()
        .add_line(p, [p[0] + w, p[1]], fade(color::LINE, style_alpha(ui)))
        .build();
    ui.dummy([0.0, space::S]);
}
