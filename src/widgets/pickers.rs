use imgui::{PopupToken, StyleColor, StyleVar, Ui};

use crate::Kit;
use crate::theme::ButtonKind;
use crate::tokens::{color, size, space};

#[allow(unused_imports)]
use super::*;

// ---------------------------------------------------------------------------
// Combo (dropdown)
// ---------------------------------------------------------------------------

/// Opens a dropdown: a 32 px frame showing `preview` with a chevron, and
/// when clicked a popup below it. Returns the popup token while it is open;
/// draw the options with [`combo_item`] inside. `width` `0.0` uses imgui's
/// item width (`set_next_item_width` is honoured).
///
/// ```no_run
/// # use imgui_kit::widgets as w;
/// # fn f(ui: &imgui::Ui, kit: &imgui_kit::Kit, mode: &mut usize) {
/// if let Some(_open) = w::combo_begin(ui, kit, "##mode", ["Loop", "Ping-pong"][*mode], 0.0) {
///     for (i, name) in ["Loop", "Ping-pong"].iter().enumerate() {
///         if w::combo_item(ui, kit, name, *mode == i) { *mode = i; }
///     }
/// }
/// # }
/// ```
pub fn combo_begin<'ui>(ui: &'ui Ui, kit: &Kit, id: &str, preview: &str, width: f32) -> Option<PopupToken<'ui>> {
    let w = if width > 0.0 { width } else { ui.calc_item_width() };
    let p = ui.cursor_screen_pos();
    let h = size::CONTROL;
    let clicked = ui.invisible_button(id, [w, h]);
    let hovered = ui.is_item_hovered();
    let popup_id = format!("{id}##combo_popup");
    // imgui-rs 0.12 has no is_popup_open; igIsPopupOpen is the same query.
    let open = {
        let c = std::ffi::CString::new(popup_id.as_str()).unwrap_or_default();
        unsafe { imgui::sys::igIsPopupOpen(c.as_ptr(), 0) }
    };
    let al = style_alpha(ui);
    {
        let dl = ui.get_window_draw_list();
        let bg = if hovered || open { color::BG2 } else { color::BG1 };
        dl.add_rect(p, [p[0] + w, p[1] + h], fade(bg, al)).rounding(size::RADIUS).filled(true).build();
        dl.add_rect(p, [p[0] + w, p[1] + h], fade(if open { color::ACCENT } else { color::LINE2 }, al))
            .rounding(size::RADIUS)
            .build();
        let lh = ui.text_line_height();
        let ty = p[1] + ((h - lh) / 2.0).round();
        dl.add_text([p[0] + size::PAD_X, ty], fade(color::FG, al), preview);
        chevron(&dl, [p[0] + w - size::PAD_X - CHEVRON, p[1] + ((h - CHEVRON) / 2.0).round()], true, fade(color::FG3, al));
    }
    if clicked {
        ui.open_popup(&popup_id);
    }
    // The list opens right under the frame, as wide as the frame.
    unsafe {
        imgui::sys::igSetNextWindowPos(
            imgui::sys::ImVec2 { x: p[0], y: p[1] + h + 2.0 },
            imgui::Condition::Always as i32,
            imgui::sys::ImVec2 { x: 0.0, y: 0.0 },
        );
        imgui::sys::igSetNextWindowSize(imgui::sys::ImVec2 { x: w, y: 0.0 }, imgui::Condition::Always as i32);
    }
    let _pad = ui.push_style_var(StyleVar::WindowPadding([0.0, space::XS]));
    let _min = ui.push_style_var(StyleVar::WindowMinSize([0.0, 0.0]));
    let _sp = ui.push_style_var(StyleVar::ItemSpacing([0.0, 0.0]));
    let _rounding = ui.push_style_var(StyleVar::PopupRounding(size::RADIUS));
    let _border = ui.push_style_var(StyleVar::PopupBorderSize(size::BORDER));
    let _bg = ui.push_style_color(StyleColor::PopupBg, color::BG1);
    let _bd = ui.push_style_color(StyleColor::Border, color::LINE2);
    let _ = kit;
    ui.begin_popup(&popup_id)
}

/// One option of an open [`combo_begin`] list; closes the list when picked.
pub fn combo_item(ui: &Ui, kit: &Kit, label: &str, selected: bool) -> bool {
    let picked = selectable(ui, kit, label, selected);
    if picked {
        ui.close_current_popup();
    }
    picked
}

/// Complete dropdown over `options`; returns `true` when `index` changed.
pub fn combo(ui: &Ui, kit: &Kit, id: &str, options: &[&str], index: &mut usize, width: f32) -> bool {
    let preview = options.get(*index).copied().unwrap_or("");
    let mut changed = false;
    if let Some(_open) = combo_begin(ui, kit, id, preview, width) {
        for (i, name) in options.iter().enumerate() {
            if combo_item(ui, kit, name, *index == i) && *index != i {
                *index = i;
                changed = true;
            }
        }
    }
    changed
}

// ---------------------------------------------------------------------------
// Segmented control, toggle button
// ---------------------------------------------------------------------------

/// Single-choice row of joined segments, 24 px: the active one is solid
/// accent with dark text. Returns `true` when the choice changed.
pub fn segmented(ui: &Ui, kit: &Kit, id: &str, options: &[&str], index: &mut usize) -> bool {
    let _id = ui.push_id(id);
    let h = size::SMALL;
    let widths: Vec<f32> = {
        let _f = ui.push_font(kit.fonts.mono13b);
        options.iter().map(|o| ui.calc_text_size(o)[0] + 2.0 * size::PAD_X).collect()
    };
    let total: f32 = widths.iter().sum();
    let p = ui.cursor_screen_pos();
    let al = style_alpha(ui);
    let mut changed = false;
    let mut x = p[0];
    for (i, (o, w)) in options.iter().zip(&widths).enumerate() {
        ui.set_cursor_screen_pos([x, p[1]]);
        let clicked = ui.invisible_button(format!("##seg{i}"), [*w, h]);
        let hovered = ui.is_item_hovered();
        if clicked && *index != i {
            *index = i;
            changed = true;
        }
        let on = *index == i;
        let dl = ui.get_window_draw_list();
        if on {
            dl.add_rect([x, p[1]], [x + w, p[1] + h], fade(color::ACCENT, al)).rounding(size::RADIUS).filled(true).build();
        } else if hovered {
            dl.add_rect([x, p[1]], [x + w, p[1] + h], fade(color::BG3, al)).rounding(size::RADIUS).filled(true).build();
        }
        let _f = ui.push_font(if on { kit.fonts.mono13b } else { kit.fonts.mono13 });
        let ts = ui.calc_text_size(o);
        dl.add_text(
            [x + ((w - ts[0]) / 2.0).round(), p[1] + ((h - ts[1]) / 2.0).round()],
            fade(if on { color::BG0 } else { color::FG2 }, al),
            o,
        );
        x += w;
    }
    // container outline on top of everything
    ui.get_window_draw_list()
        .add_rect(p, [p[0] + total, p[1] + h], fade(color::LINE2, al))
        .rounding(size::RADIUS)
        .build();
    ui.set_cursor_screen_pos([p[0], p[1] + h]);
    ui.dummy([0.0, 0.0]);
    changed
}

/// Button that stays pressed while `value` is on (secondary when off,
/// bg-sel fill + accent border when on). Returns `true` when toggled.
pub fn toggle_button(ui: &Ui, kit: &Kit, label: &str, value: &mut bool) -> bool {
    let _id = ui.push_id(label);
    let shown = label.split("##").next().unwrap_or("");
    let p = ui.cursor_screen_pos();
    let w = {
        let _f = ui.push_font(kit.fonts.mono13b);
        ui.calc_text_size(shown)[0] + 2.0 * size::BUTTON_PAD_X
    };
    let h = size::CONTROL;
    let clicked = ui.invisible_button("##toggle", [w, h]);
    let hovered = ui.is_item_hovered();
    if clicked {
        *value = !*value;
    }
    let on = *value;
    let al = style_alpha(ui);
    let (bg, border, fg) = if on {
        (color::SEL, color::ACCENT, color::FG)
    } else if hovered {
        (color::BG3, color::LINE2, color::FG)
    } else {
        (color::BG2, color::LINE2, color::FG2)
    };
    let dl = ui.get_window_draw_list();
    dl.add_rect(p, [p[0] + w, p[1] + h], fade(bg, al)).rounding(size::RADIUS).filled(true).build();
    dl.add_rect(p, [p[0] + w, p[1] + h], fade(border, al)).rounding(size::RADIUS).build();
    let _f = ui.push_font(kit.fonts.mono13b);
    let ts = ui.calc_text_size(shown);
    dl.add_text([p[0] + size::BUTTON_PAD_X, p[1] + ((h - ts[1]) / 2.0).round()], fade(fg, al), shown);
    clicked
}

// ---------------------------------------------------------------------------
// Number input
// ---------------------------------------------------------------------------

/// `- [ value ] +` stepper, 32 px: kit icon buttons around an imgui integer
/// field, optional dim unit after it. `width` is the whole control (`0.0` =
/// 160). Returns `true` when the value changed.
#[allow(clippy::too_many_arguments)]
pub fn number_input(ui: &Ui, kit: &Kit, id: &str, value: &mut i32, min: i32, max: i32, step: i32, unit: &str, width: f32) -> bool {
    let _ = kit;
    let _id = ui.push_id(id);
    let total = if width > 0.0 { width } else { 160.0 };
    let field_w = (total - 2.0 * size::CONTROL - 2.0 * space::XS).max(40.0);
    let mut changed = false;
    let _sp = ui.push_style_var(StyleVar::ItemSpacing([space::XS, 0.0]));
    if icon_button(ui, ButtonKind::Secondary, "minus", "-", false) {
        *value = (*value - step).max(min);
        changed = true;
    }
    ui.same_line();
    {
        let _w = ui.push_item_width(field_w);
        let _pad = ui.push_style_var(StyleVar::FramePadding([size::PAD_X, (size::CONTROL - ui.text_line_height()) / 2.0]));
        if ui.input_scalar("##field", value).build() {
            *value = (*value).clamp(min, max);
            changed = true;
        }
    }
    ui.same_line();
    if icon_button(ui, ButtonKind::Secondary, "plus", "+", false) {
        *value = (*value + step).min(max);
        changed = true;
    }
    if !unit.is_empty() {
        let top = ui.item_rect_min()[1];
        ui.same_line_with_spacing(0.0, space::S);
        let p = ui.cursor_screen_pos();
        let ts = ui.calc_text_size(unit);
        ui.get_window_draw_list()
            .add_text([p[0], top + ((size::CONTROL - ts[1]) / 2.0).round()], fade(color::FG3, style_alpha(ui)), unit);
        ui.dummy([ts[0], size::CONTROL]);
    }
    changed
}
