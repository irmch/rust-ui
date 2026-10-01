use imgui::{MouseButton, Ui};

use crate::Kit;
use crate::grid;
use crate::tokens::{color, size, space};

#[allow(unused_imports)]
use super::*;

// ---------------------------------------------------------------------------
// Sliders
// ---------------------------------------------------------------------------

/// Custom slider: 4 px track, 16 px knob, fills `width` and is 32 px tall.
/// Returns `true` while the value changes. Drag with the mouse; the value is
/// rounded to `step`.
pub fn slider_track(ui: &Ui, id: &str, value: &mut f32, min: f32, max: f32, step: f32, width: f32) -> bool {
    let _id = ui.push_id(id);
    let p = ui.cursor_screen_pos();
    let w = if width > 0.0 { width } else { ui.content_region_avail()[0] };
    let h = size::CONTROL;
    ui.invisible_button("##track", [w, h]);
    let active = ui.is_item_active();
    let hovered = ui.is_item_hovered();
    let mut changed = false;
    let range = (max - min).max(f32::EPSILON);
    let k = size::KNOB / 2.0;
    let x0 = p[0] + k;
    let x1 = p[0] + w - k;
    if active && ui.is_mouse_down(MouseButton::Left) {
        let mx = ui.io().mouse_pos[0].clamp(x0, x1);
        let mut v = min + (mx - x0) / (x1 - x0) * range;
        if step > 0.0 {
            v = ((v - min) / step).round() * step + min;
        }
        let v = v.clamp(min, max);
        if (v - *value).abs() > f32::EPSILON {
            *value = v;
            changed = true;
        }
    }
    let t = ((*value - min) / range).clamp(0.0, 1.0);
    let kx = x0 + (x1 - x0) * t;
    let cy = p[1] + h / 2.0;
    let al = style_alpha(ui);
    let dl = ui.get_window_draw_list();
    let th = size::TRACK / 2.0;
    dl.add_rect([x0, cy - th], [x1, cy + th], fade(color::BG4, al))
        .rounding(th)
        .filled(true)
        .build();
    dl.add_rect([x0, cy - th], [kx, cy + th], fade(color::FG3, al))
        .rounding(th)
        .filled(true)
        .build();
    let knob = if active || hovered {
        color::ACCENT_HOVER
    } else {
        color::ACCENT
    };
    dl.add_circle([kx, cy], k, fade(color::BG0, al)).filled(true).build();
    dl.add_circle([kx, cy], k - 1.0, fade(knob, al)).filled(true).build();
    changed
}

/// Launcher parameter row: `CAPTION | slider | 6 / 12` (artboard 03 · section 16).
/// `suffix` is drawn in fg-3 after the value ("/ 12", "ms").
/// Builder form of [`labeled_slider`]:
/// `Slider::new("Windows", 1.0, 12.0).step(1.0).suffix("/ 12").show(ui, kit, &mut v)`.
#[derive(Clone, Copy, Debug)]
pub struct Slider<'a> {
    label: &'a str,
    min: f32,
    max: f32,
    step: f32,
    suffix: &'a str,
}

impl<'a> Slider<'a> {
    pub fn new(label: &'a str, min: f32, max: f32) -> Self {
        Self { label, min, max, step: 0.0, suffix: "" }
    }

    /// Rounds the value to multiples of `step` (`0.0` = continuous).
    pub fn step(mut self, step: f32) -> Self {
        self.step = step;
        self
    }

    /// Drawn in fg-3 after the value ("/ 12", "ms").
    pub fn suffix(mut self, suffix: &'a str) -> Self {
        self.suffix = suffix;
        self
    }

    /// Draws the row; returns `true` while the value changes.
    pub fn show(self, ui: &Ui, kit: &Kit, value: &mut f32) -> bool {
        labeled_slider(ui, kit, self.label, value, self.min, self.max, self.step, self.suffix)
    }
}

#[allow(clippy::too_many_arguments)]
pub fn labeled_slider(
    ui: &Ui,
    kit: &Kit,
    label: &str,
    value: &mut f32,
    min: f32,
    max: f32,
    step: f32,
    suffix: &str,
) -> bool {
    let mut changed = false;
    let id = format!("##row_{label}");
    grid::form_row(ui, &id, |ui, cell, w| match cell {
        grid::FormCell::Label => {
            let cap_h = {
                let _f = ui.push_font(kit.fonts.mono10);
                ui.text_line_height()
            };
            grid::vcenter(ui, cap_h, size::CONTROL);
            caption(ui, kit, label);
        }
        grid::FormCell::Control => {
            changed = slider_track(ui, label, value, min, max, step, w);
        }
        grid::FormCell::Value => {
            let txt = if step >= 1.0 {
                format!("{}", value.round() as i64)
            } else {
                format!("{value:.2}")
            };
            let tw = {
                let _f = ui.push_font(kit.fonts.mono13b);
                ui.calc_text_size(&txt)[0]
            };
            let sw = if suffix.is_empty() {
                0.0
            } else {
                ui.calc_text_size(suffix)[0] + space::XS
            };
            grid::right_align(ui, &[tw + sw]);
            grid::vcenter(ui, ui.text_line_height(), size::CONTROL);
            text_bold(ui, kit, &txt, color::FG);
            if !suffix.is_empty() {
                ui.same_line_with_spacing(0.0, space::XS);
                ui.text_colored(color::FG3, suffix);
            }
        }
    });
    changed
}

/// Determinate progress bar: 8 px track, accent fill.
pub fn progress(ui: &Ui, fraction: f32, width: f32) {
    let p = ui.cursor_screen_pos();
    let w = if width > 0.0 { width } else { ui.content_region_avail()[0] };
    ui.dummy([w, 8.0]);
    let al = style_alpha(ui);
    let dl = ui.get_window_draw_list();
    dl.add_rect(p, [p[0] + w, p[1] + 8.0], fade(color::BG4, al))
        .rounding(4.0)
        .filled(true)
        .build();
    let fw = w * fraction.clamp(0.0, 1.0);
    if fw > 0.0 {
        dl.add_rect(p, [p[0] + fw, p[1] + 8.0], fade(color::ACCENT, al))
            .rounding(4.0)
            .filled(true)
            .build();
    }
}
