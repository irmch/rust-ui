use imgui::Ui;

use crate::anim;
use crate::fonts::Fonts;
use crate::tokens::{color, size, space};

#[allow(unused_imports)]
use super::*;

// ---------------------------------------------------------------------------
// Selection controls (custom drawn)
// ---------------------------------------------------------------------------

/// Checkbox with 18 px box, bold label and optional hint, 32 px row
/// (artboard 03 · section 07). Checked state is a filled accent box with a
/// dark check mark.
pub fn checkbox(ui: &Ui, f: &Fonts, label: &str, hint: Option<&str>, value: &mut bool) -> bool {
    check_like(ui, f, label, hint, value, false)
}

/// Radio button with the same metrics as [`checkbox`]. Sets `*current = this`
/// when clicked.
pub fn radio<T: PartialEq + Copy>(
    ui: &Ui,
    f: &Fonts,
    label: &str,
    hint: Option<&str>,
    current: &mut T,
    this: T,
) -> bool {
    let mut on = *current == this;
    let was = on;
    let clicked = check_like(ui, f, label, hint, &mut on, true);
    if clicked && !was {
        *current = this;
        return true;
    }
    false
}

pub(super) fn check_like(ui: &Ui, f: &Fonts, label: &str, hint: Option<&str>, value: &mut bool, round: bool) -> bool {
    let _id = ui.push_id(label);
    let row_h = size::CONTROL;
    let b = size::CHECK;
    let label_w = {
        let _f = ui.push_font(f.mono13b);
        ui.calc_text_size(label)[0]
    };
    let hint_w = hint
        .map(|h| {
            let _f = ui.push_font(f.mono12);
            ui.calc_text_size(h)[0] + space::S
        })
        .unwrap_or(0.0);
    let w = b + space::M + label_w + hint_w;
    let p = ui.cursor_screen_pos();
    let clicked = ui.invisible_button("##box", [w, row_h]);
    if clicked && !round {
        *value = !*value;
    }
    if clicked && round {
        *value = true;
    }
    let hovered = ui.is_item_hovered();
    // 0 = off, 1 = on; eases over anim::CONTROL seconds when enabled.
    let t = anim::toggle(ui, anim::key(ui, "##box"), *value, anim::CONTROL);
    let al = style_alpha(ui);
    let dl = ui.get_window_draw_list();
    let by = p[1] + ((row_h - b) / 2.0).round();
    let off_bg = if hovered { color::BG3 } else { color::BG1 };
    let on_bg = if round { color::BG1 } else { color::ACCENT };
    let bg = anim::mix(off_bg, on_bg, t);
    let border = anim::mix(color::LINE2, color::ACCENT, t);
    let rounding = if round { b / 2.0 } else { size::RADIUS };
    dl.add_rect([p[0], by], [p[0] + b, by + b], fade(bg, al))
        .rounding(rounding)
        .filled(true)
        .build();
    dl.add_rect([p[0], by], [p[0] + b, by + b], fade(border, al))
        .rounding(rounding)
        .build();
    if t > 0.0 {
        let c = [p[0] + b / 2.0, by + b / 2.0];
        if round {
            dl.add_circle(c, 4.0 * t, fade(color::ACCENT, al)).filled(true).build();
        } else {
            // the mark grows from the centre of the box
            let q = |x: f32, y: f32| [c[0] + (p[0] + x - c[0]) * t, c[1] + (by + y - c[1]) * t];
            let ink = [color::BG0[0], color::BG0[1], color::BG0[2], t * al];
            dl.add_line(q(4.0, 9.5), q(7.5, 13.0), ink).thickness(1.6).build();
            dl.add_line(q(7.5, 13.0), q(14.0, 5.0), ink).thickness(1.6).build();
        }
    }
    let mut x = p[0] + b + space::M;
    {
        let _f = ui.push_font(f.mono13b);
        let ty = p[1] + ((row_h - ui.text_line_height()) / 2.0).round();
        dl.add_text([x, ty], fade(color::FG, al), label);
        x += label_w + space::S;
    }
    if let Some(h) = hint {
        let _f = ui.push_font(f.mono12);
        let ty = p[1] + ((row_h - ui.text_line_height()) / 2.0).round();
        dl.add_text([x, ty], fade(color::FG3, al), h);
    }
    clicked
}

/// 36 × 20 switch with a bold label (artboard 03 · section 09).
pub fn switch(ui: &Ui, f: &Fonts, label: &str, value: &mut bool) -> bool {
    let _id = ui.push_id(label);
    let [sw, sh] = size::SWITCH;
    let row_h = size::CONTROL;
    let label_w = {
        let _f = ui.push_font(f.mono13b);
        ui.calc_text_size(label)[0]
    };
    let p = ui.cursor_screen_pos();
    let clicked = ui.invisible_button("##sw", [sw + space::M + label_w, row_h]);
    if clicked {
        *value = !*value;
    }
    let t = anim::toggle(ui, anim::key(ui, "##sw"), *value, anim::CONTROL);
    let al = style_alpha(ui);
    let dl = ui.get_window_draw_list();
    let y = p[1] + ((row_h - sh) / 2.0).round();
    let bg = fade(anim::mix(color::BG4, color::ACCENT, t), al);
    let border = fade(anim::mix(color::LINE2, color::ACCENT, t), al);
    let knob = fade(anim::mix(color::FG3, color::BG0, t), al);
    let kx = p[0] + sh / 2.0 + (sw - sh) * t;
    dl.add_rect([p[0], y], [p[0] + sw, y + sh], bg)
        .rounding(sh / 2.0)
        .filled(true)
        .build();
    dl.add_rect([p[0], y], [p[0] + sw, y + sh], border)
        .rounding(sh / 2.0)
        .build();
    dl.add_circle([kx, y + sh / 2.0], sh / 2.0 - 3.0, knob)
        .filled(true)
        .build();
    let _f = ui.push_font(f.mono13b);
    let ty = p[1] + ((row_h - ui.text_line_height()) / 2.0).round();
    dl.add_text([p[0] + sw + space::M, ty], fade(color::FG, al), label);
    clicked
}
