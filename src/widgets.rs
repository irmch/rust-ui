//! Widgets of artboards 03–05 that ImGui does not draw the way the kit wants
//! (filled checkboxes, 4 px slider tracks, captions, tags, stats, bars) plus
//! thin wrappers that apply the right font and colours to native ones.

use imgui::{MouseButton, StyleColor, StyleVar, Ui, WindowFlags};

use crate::fonts::Fonts;
use crate::grid;
use crate::theme::ButtonKind;
use crate::tokens::{color, size, space, Rgba};

// ---------------------------------------------------------------------------
// Text
// ---------------------------------------------------------------------------

/// Ascent of `font` in screen pixels, for aligning texts of different sizes
/// on one baseline (`y_small = y_big + ascent(big) - ascent(small)`).
fn ascent(ui: &Ui, font: imgui::FontId) -> f32 {
    let a = ui.fonts().get_font(font).map_or(0.0, |f| f.ascent * f.scale);
    (a * ui.io().font_global_scale).round()
}

/// Caption: 10 px semibold uppercase, fg-3 ("GAME PATH").
pub fn caption(ui: &Ui, f: &Fonts, text: &str) {
    let _f = ui.push_font(f.mono10);
    ui.text_colored(color::FG3, text.to_uppercase());
}

/// Section header followed by the 12 px gap of the rhythm.
pub fn section(ui: &Ui, f: &Fonts, text: &str) {
    caption(ui, f, text);
    ui.dummy([0.0, space::M - space::S]);
}

/// Hint text: 12 px regular fg-3, drawn on the same line as the previous item.
pub fn hint_inline(ui: &Ui, f: &Fonts, text: &str) {
    ui.same_line();
    let _f = ui.push_font(f.mono12);
    ui.text_colored(color::FG3, text);
}

/// Body text in a given colour with the bold font.
pub fn text_bold(ui: &Ui, f: &Fonts, text: &str, col: Rgba) {
    let _f = ui.push_font(f.mono13b);
    ui.text_colored(col, text);
}

/// Secondary text (fg-2).
pub fn text_muted(ui: &Ui, text: &str) {
    ui.text_colored(color::FG2, text);
}

/// Log line: `[0.084]` in fg-3 then the message (artboard 05 · section 02).
pub fn log_line(ui: &Ui, f: &Fonts, seconds: f32, message: &str, col: Option<Rgba>) {
    let _f = ui.push_font(f.mono12);
    ui.text_colored(color::FG3, format!("[{seconds:.3}]"));
    ui.same_line();
    ui.text_colored(col.unwrap_or(color::FG), message);
}

/// "✓ Verified · Path of Exile 2" line under the game path input.
pub fn verified_line(ui: &Ui, f: &Fonts, ok: bool, label: &str, detail: &str) {
    let (mark, col) = if ok { ("✓", color::OK) } else { ("✕", color::ERR) };
    text_bold(ui, f, &format!("{mark} {label}"), col);
    ui.same_line();
    ui.text_colored(color::FG3, format!("· {detail}"));
}

// ---------------------------------------------------------------------------
// Buttons
// ---------------------------------------------------------------------------

/// Button of a variant with the default 32 px height. Width auto.
pub fn button(ui: &Ui, f: &Fonts, kind: ButtonKind, label: &str) -> bool {
    button_sized(ui, f, kind, label, [0.0, size::CONTROL])
}

/// Button of a variant with an explicit size (`0.0` = auto, [`grid::FILL`] = fill).
pub fn button_sized(ui: &Ui, f: &Fonts, kind: ButtonKind, label: &str, sz: [f32; 2]) -> bool {
    let _f = ui.push_font(f.mono13b);
    let _c = kind.push(ui);
    let _pad = ui.push_style_var(StyleVar::FramePadding([size::BUTTON_PAD_X, size::PAD_Y]));
    ui.button_with_size(label, sz)
}

/// Small 24 px button (toolbar "Copy", "Save").
pub fn button_small(ui: &Ui, f: &Fonts, kind: ButtonKind, label: &str) -> bool {
    let _f = ui.push_font(f.mono12);
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
pub fn cta(ui: &Ui, f: &Fonts, kind: ButtonKind, label: &str) -> bool {
    let _f = ui.push_font(f.mono13b);
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

fn check_like(ui: &Ui, f: &Fonts, label: &str, hint: Option<&str>, value: &mut bool, round: bool) -> bool {
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
    let dl = ui.get_window_draw_list();
    let by = p[1] + ((row_h - b) / 2.0).round();
    let (bg, border) = match (*value, round, hovered) {
        (true, false, _) => (color::ACCENT, color::ACCENT),
        (true, true, _) => (color::BG1, color::ACCENT),
        (false, _, true) => (color::BG3, color::LINE2),
        (false, _, false) => (color::BG1, color::LINE2),
    };
    let rounding = if round { b / 2.0 } else { size::RADIUS };
    dl.add_rect([p[0], by], [p[0] + b, by + b], bg)
        .rounding(rounding)
        .filled(true)
        .build();
    dl.add_rect([p[0], by], [p[0] + b, by + b], border)
        .rounding(rounding)
        .build();
    if *value {
        if round {
            dl.add_circle([p[0] + b / 2.0, by + b / 2.0], 4.0, color::ACCENT)
                .filled(true)
                .build();
        } else {
            let q = |x: f32, y: f32| [p[0] + x, by + y];
            dl.add_line(q(4.0, 9.5), q(7.5, 13.0), color::BG0)
                .thickness(1.6)
                .build();
            dl.add_line(q(7.5, 13.0), q(14.0, 5.0), color::BG0)
                .thickness(1.6)
                .build();
        }
    }
    let mut x = p[0] + b + space::M;
    {
        let _f = ui.push_font(f.mono13b);
        let ty = p[1] + ((row_h - ui.text_line_height()) / 2.0).round();
        dl.add_text([x, ty], color::FG, label);
        x += label_w + space::S;
    }
    if let Some(h) = hint {
        let _f = ui.push_font(f.mono12);
        let ty = p[1] + ((row_h - ui.text_line_height()) / 2.0).round();
        dl.add_text([x, ty], color::FG3, h);
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
    let dl = ui.get_window_draw_list();
    let y = p[1] + ((row_h - sh) / 2.0).round();
    let (bg, border, knob, kx) = if *value {
        (color::ACCENT, color::ACCENT, color::BG0, p[0] + sw - sh / 2.0)
    } else {
        (color::BG4, color::LINE2, color::FG3, p[0] + sh / 2.0)
    };
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
    dl.add_text([p[0] + sw + space::M, ty], color::FG, label);
    clicked
}

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
    let dl = ui.get_window_draw_list();
    let th = size::TRACK / 2.0;
    dl.add_rect([x0, cy - th], [x1, cy + th], color::BG4)
        .rounding(th)
        .filled(true)
        .build();
    dl.add_rect([x0, cy - th], [kx, cy + th], color::FG3)
        .rounding(th)
        .filled(true)
        .build();
    let knob = if active || hovered {
        color::ACCENT_HOVER
    } else {
        color::ACCENT
    };
    dl.add_circle([kx, cy], k, color::BG0).filled(true).build();
    dl.add_circle([kx, cy], k - 1.0, knob).filled(true).build();
    changed
}

/// Launcher parameter row: `CAPTION | slider | 6 / 12` (artboard 03 · section 16).
/// `suffix` is drawn in fg-3 after the value ("/ 12", "ms").
#[allow(clippy::too_many_arguments)]
pub fn labeled_slider(
    ui: &Ui,
    f: &Fonts,
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
                let _f = ui.push_font(f.mono10);
                ui.text_line_height()
            };
            grid::vcenter(ui, cap_h, size::CONTROL);
            caption(ui, f, label);
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
                let _f = ui.push_font(f.mono13b);
                ui.calc_text_size(&txt)[0]
            };
            let sw = if suffix.is_empty() {
                0.0
            } else {
                ui.calc_text_size(suffix)[0] + space::XS
            };
            grid::right_align(ui, &[tw + sw]);
            grid::vcenter(ui, ui.text_line_height(), size::CONTROL);
            text_bold(ui, f, &txt, color::FG);
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
    let dl = ui.get_window_draw_list();
    dl.add_rect(p, [p[0] + w, p[1] + 8.0], color::BG4)
        .rounding(4.0)
        .filled(true)
        .build();
    let fw = w * fraction.clamp(0.0, 1.0);
    if fw > 0.0 {
        dl.add_rect(p, [p[0] + fw, p[1] + 8.0], color::ACCENT)
            .rounding(4.0)
            .filled(true)
            .build();
    }
}

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
    fn colors(self) -> (Rgba, Rgba, Rgba) {
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
pub fn tag(ui: &Ui, f: &Fonts, kind: TagKind, text: &str) {
    let _f = ui.push_font(f.mono10);
    let txt = text.to_uppercase();
    let tw = ui.calc_text_size(&txt)[0];
    let w = tw + 2.0 * space::S;
    let p = ui.cursor_screen_pos();
    ui.dummy([w, size::TAG]);
    let (bg, border, fg) = kind.colors();
    let dl = ui.get_window_draw_list();
    let q = [p[0] + w, p[1] + size::TAG];
    dl.add_rect(p, q, bg).rounding(size::RADIUS).filled(true).build();
    dl.add_rect(p, q, border).rounding(size::RADIUS).build();
    let ty = p[1] + ((size::TAG - ui.text_line_height()) / 2.0).round();
    dl.add_text([p[0] + space::S, ty], fg, &txt);
}

/// 8 px status dot followed by text.
pub fn status_dot(ui: &Ui, col: Rgba, text: &str) {
    let p = ui.cursor_screen_pos();
    let lh = ui.text_line_height();
    ui.dummy([8.0, lh]);
    ui.get_window_draw_list()
        .add_circle([p[0] + 4.0, p[1] + lh / 2.0], 4.0, col)
        .filled(true)
        .build();
    ui.same_line_with_spacing(0.0, 6.0);
    ui.text(text);
}

/// Stat item of the status strip: `CAPTION  Value unit`. `col` colours the value.
pub fn stat(ui: &Ui, f: &Fonts, cap: &str, value: &str, unit: &str, col: Rgba) {
    // The cursor marks the top of the 13 px value; the 10 px caption and the
    // unit are placed explicitly on the same baseline (no `same_line`, which
    // would snap back to whatever line y imgui remembers).
    let [x, y] = ui.cursor_pos();
    let asc_val = ascent(ui, f.mono13b);
    let cap_txt = cap.to_uppercase();
    let cap_w = {
        let _f = ui.push_font(f.mono10);
        ui.set_cursor_pos([x, y + asc_val - ascent(ui, f.mono10)]);
        ui.text_colored(color::FG3, &cap_txt);
        ui.calc_text_size(&cap_txt)[0]
    };
    let x = x + cap_w + space::S;
    let val_w = {
        let _f = ui.push_font(f.mono13b);
        ui.set_cursor_pos([x, y]);
        ui.text_colored(col, value);
        ui.calc_text_size(value)[0]
    };
    if !unit.is_empty() {
        ui.set_cursor_pos([x + val_w + space::XS, y + asc_val - ascent(ui, f.mono13)]);
        ui.text_colored(color::FG3, unit);
    }
}

/// 1 px vertical divider of `h` px on the current line.
pub fn vdivider(ui: &Ui, h: f32) {
    let p = ui.cursor_screen_pos();
    ui.dummy([1.0, h]);
    ui.get_window_draw_list()
        .add_line(p, [p[0], p[1] + h], color::LINE)
        .build();
}

/// Draws a 1 px line along the bottom edge of the current window.
///
/// Acquires and releases the window draw list itself: only one `DrawListMut`
/// may exist at a time, and the widgets drawn after this call need their own.
fn bottom_border(ui: &Ui) {
    let p = ui.window_pos();
    let s = ui.window_size();
    ui.get_window_draw_list()
        .add_line([p[0], p[1] + s[1] - 1.0], [p[0] + s[0], p[1] + s[1] - 1.0], color::LINE)
        .build();
}

// ---------------------------------------------------------------------------
// Bars
// ---------------------------------------------------------------------------

/// What the window controls of the title bar reported.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TitleBarAction {
    None,
    Minimize,
    Maximize,
    Close,
}

/// 48 px title bar: app name / crumb on the left, tabs centred, window
/// controls on the right (artboard 04 · section 01). Returns the window
/// control pressed, if any, and updates `active` on tab clicks.
pub fn title_bar(ui: &Ui, f: &Fonts, app: &str, crumb: &str, tabs: &[&str], active: &mut usize) -> TitleBarAction {
    let mut action = TitleBarAction::None;
    let _pad = ui.push_style_var(StyleVar::WindowPadding([space::XL, 0.0]));
    let _bg = ui.push_style_color(StyleColor::ChildBg, color::BG2);
    ui.child_window("##titlebar")
        .size([0.0, size::BAR])
        .flags(WindowFlags::NO_SCROLLBAR | WindowFlags::NO_SCROLL_WITH_MOUSE | WindowFlags::ALWAYS_USE_WINDOW_PADDING)
        .build(|| {
            bottom_border(ui);
            // left: app / crumb, the 13 px texts on the 16 px app baseline
            let x0 = ui.cursor_pos()[0];
            let (y16, app_w) = {
                let _f = ui.push_font(f.mono16b);
                let y = ((size::BAR - ui.text_line_height()) / 2.0).round();
                ui.set_cursor_pos([x0, y]);
                ui.text(app);
                (y, ui.calc_text_size(app)[0])
            };
            let y13 = y16 + ascent(ui, f.mono16b) - ascent(ui, f.mono13);
            let x = x0 + app_w + space::S;
            ui.set_cursor_pos([x, y13]);
            ui.text_colored(color::FG3, "/");
            ui.set_cursor_pos([x + ui.calc_text_size("/")[0] + space::S, y13]);
            ui.text_colored(color::FG2, crumb);

            // centre: tabs
            let tab_w: Vec<f32> = tabs.iter().map(|t| ui.calc_text_size(t)[0] + 2.0 * size::PAD_X).collect();
            let total: f32 = tab_w.iter().sum::<f32>() + space::XS * (tabs.len().saturating_sub(1)) as f32;
            let avail = ui.window_size()[0];
            ui.set_cursor_pos([((avail - total) / 2.0).round(), (size::BAR - size::CONTROL) / 2.0]);
            for (i, (t, w)) in tabs.iter().zip(&tab_w).enumerate() {
                if i > 0 {
                    ui.same_line_with_spacing(0.0, space::XS);
                }
                if tab(ui, f, t, *w, i == *active) {
                    *active = i;
                }
            }

            // right: window controls
            let ctrl = 3.0 * size::CONTROL + 2.0 * space::XS;
            let cy = (size::BAR - size::CONTROL) / 2.0;
            let mut x = avail - space::L - ctrl;
            for which in [TitleBarAction::Minimize, TitleBarAction::Maximize, TitleBarAction::Close] {
                ui.set_cursor_pos([x, cy]);
                if window_control(ui, which) {
                    action = which;
                }
                x += size::CONTROL + space::XS;
            }
        });
    action
}

/// 32 × 32 ghost window control. The – □ × glyphs are drawn with 1 px lines
/// (10 px icon) instead of font glyphs, so they stay crisp and identical.
pub fn window_control(ui: &Ui, which: TitleBarAction) -> bool {
    let id = match which {
        TitleBarAction::Minimize => "##win_min",
        TitleBarAction::Maximize => "##win_max",
        TitleBarAction::Close => "##win_close",
        TitleBarAction::None => "##win_none",
    };
    let s = size::CONTROL;
    let p = ui.cursor_screen_pos();
    let clicked = ui.invisible_button(id, [s, s]);
    let hovered = ui.is_item_hovered();
    let active = ui.is_item_active();
    let dl = ui.get_window_draw_list();
    if hovered || active {
        let bg = if active { color::BG2 } else { color::BG3 };
        dl.add_rect(p, [p[0] + s, p[1] + s], bg).rounding(size::RADIUS).filled(true).build();
    }
    let fg = if hovered { color::FG } else { color::FG2 };
    let c = [(p[0] + s / 2.0).round(), (p[1] + s / 2.0).round()];
    let r = 5.0;
    match which {
        TitleBarAction::Minimize => {
            dl.add_line([c[0] - r, c[1] + 0.5], [c[0] + r, c[1] + 0.5], fg).build();
        }
        TitleBarAction::Maximize => {
            dl.add_rect([c[0] - r + 0.5, c[1] - r + 0.5], [c[0] + r - 0.5, c[1] + r - 0.5], fg).build();
        }
        TitleBarAction::Close => {
            dl.add_line([c[0] - r, c[1] - r], [c[0] + r, c[1] + r], fg).build();
            dl.add_line([c[0] - r, c[1] + r], [c[0] + r, c[1] - r], fg).build();
        }
        TitleBarAction::None => {}
    }
    clicked
}

/// One tab of the title bar: 32 px, bg-3 + bold when active, fg-2 otherwise.
pub fn tab(ui: &Ui, f: &Fonts, label: &str, width: f32, active: bool) -> bool {
    let _f = ui.push_font(if active { f.mono13b } else { f.mono13 });
    let (bg, fg) = if active { (color::BG3, color::FG) } else { (color::TRANSPARENT, color::FG2) };
    let _c = [
        ui.push_style_color(StyleColor::Button, bg),
        ui.push_style_color(StyleColor::ButtonHovered, color::BG2),
        ui.push_style_color(StyleColor::ButtonActive, color::BG3),
        ui.push_style_color(StyleColor::Text, fg),
        ui.push_style_color(StyleColor::Border, color::TRANSPARENT),
    ];
    ui.button_with_size(label, [width, size::CONTROL])
}

/// One entry of the status strip.
pub struct StatItem<'a> {
    pub caption: &'a str,
    pub value: &'a str,
    pub unit: &'a str,
    pub color: Rgba,
}

/// 48 px status strip: stats with dividers on the left, `actions` drawn
/// right-aligned (artboard 04 · section 01). `action_widths` must list the
/// widths of the items `actions` draws so they can be right-aligned.
pub fn status_strip(ui: &Ui, f: &Fonts, items: &[StatItem<'_>], action_widths: &[f32], actions: impl FnOnce(&Ui)) {
    let _pad = ui.push_style_var(StyleVar::WindowPadding([space::XL, 0.0]));
    let _bg = ui.push_style_color(StyleColor::ChildBg, color::BG1);
    ui.child_window("##statusstrip")
        .size([0.0, size::BAR])
        .flags(WindowFlags::NO_SCROLLBAR | WindowFlags::NO_SCROLL_WITH_MOUSE | WindowFlags::ALWAYS_USE_WINDOW_PADDING)
        .build(|| {
            bottom_border(ui);
            let lh = ui.text_line_height();
            for (i, it) in items.iter().enumerate() {
                if i > 0 {
                    ui.same_line_with_spacing(0.0, space::L);
                    grid::vcenter_at(ui, size::SMALL, 0.0, size::BAR);
                    vdivider(ui, size::SMALL);
                    ui.same_line_with_spacing(0.0, space::L);
                }
                grid::vcenter_at(ui, lh, 0.0, size::BAR);
                stat(ui, f, it.caption, it.value, it.unit, it.color);
            }
            // No `same_line` here: imgui would keep the stats' line y and
            // `same_line` calls inside `actions` would snap back to it.
            grid::right_align(ui, action_widths);
            grid::vcenter_at(ui, size::CONTROL, 0.0, size::BAR);
            actions(ui);
        });
}

/// Toolbar row above a panel: caption on the left, small buttons on the
/// right (the "STATUS   Copy Save" header of the log).
pub fn panel_header(ui: &Ui, f: &Fonts, cap: &str, action_widths: &[f32], actions: impl FnOnce(&Ui)) {
    let [x, y] = ui.cursor_pos();
    let cap_h = {
        let _f = ui.push_font(f.mono10);
        ui.text_line_height()
    };
    ui.set_cursor_pos([x, y + ((size::CONTROL - cap_h) / 2.0).round()]);
    caption(ui, f, cap);
    // No `same_line`: it would make imgui snap the actions' `same_line`
    // calls back to the caption's y.
    ui.set_cursor_pos([x, y + (size::CONTROL - size::SMALL) / 2.0]);
    grid::right_align(ui, action_widths);
    actions(ui);
    ui.set_cursor_pos([x, y + size::CONTROL]);
}

/// Bordered panel (bg-0, 1 px line) that scrolls its content; used for the log.
pub fn panel(ui: &Ui, id: &str, size_: [f32; 2], body: impl FnOnce(&Ui)) {
    let _bs = ui.push_style_var(StyleVar::ChildBorderSize(size::BORDER));
    let _pad = ui.push_style_var(StyleVar::WindowPadding([size::PAD_X, space::S]));
    let _bg = ui.push_style_color(StyleColor::ChildBg, color::BG0);
    let _bd = ui.push_style_color(StyleColor::Border, color::LINE);
    ui.child_window(id)
        .size(size_)
        .border(true)
        .flags(WindowFlags::ALWAYS_USE_WINDOW_PADDING)
        .build(|| body(ui));
}

/// Card: bg-1, 1 px line, 16 px padding. `size` as in [`panel`]: `0.0`
/// fills the remaining width / height (imgui child windows cannot size
/// themselves to their content, so give cards in a row an explicit height).
pub fn card(ui: &Ui, id: &str, size_: [f32; 2], body: impl FnOnce(&Ui)) {
    let _bs = ui.push_style_var(StyleVar::ChildBorderSize(size::BORDER));
    let _pad = ui.push_style_var(StyleVar::WindowPadding([space::L, space::L]));
    let _bg = ui.push_style_color(StyleColor::ChildBg, color::BG1);
    let _bd = ui.push_style_color(StyleColor::Border, color::LINE);
    ui.child_window(id)
        .size(size_)
        .border(true)
        .flags(WindowFlags::ALWAYS_USE_WINDOW_PADDING | WindowFlags::NO_SCROLLBAR)
        .build(|| body(ui));
}

/// Inline banner with a coloured border (artboard 04 · section 14).
pub fn banner(ui: &Ui, f: &Fonts, kind: TagKind, text: &str, action: Option<&str>) -> bool {
    let (bg, border, fg) = kind.colors();
    let _bs = ui.push_style_var(StyleVar::ChildBorderSize(size::BORDER));
    let _pad = ui.push_style_var(StyleVar::WindowPadding([size::PAD_X, space::S]));
    let _bg = ui.push_style_color(StyleColor::ChildBg, bg);
    let _bd = ui.push_style_color(StyleColor::Border, border);
    let mut clicked = false;
    ui.child_window(format!("##banner_{text}"))
        .size([0.0, size::CONTROL + 2.0 * space::S])
        .border(true)
        .flags(WindowFlags::ALWAYS_USE_WINDOW_PADDING | WindowFlags::NO_SCROLLBAR)
        .build(|| {
            grid::vcenter(ui, ui.text_line_height(), size::CONTROL);
            ui.text_colored(fg, text);
            if let Some(a) = action {
                ui.same_line();
                let w = ui.calc_text_size(a)[0] + 20.0;
                grid::right_align(ui, &[w]);
                let y = ui.cursor_pos()[1] - ((size::CONTROL - ui.text_line_height()) / 2.0).round();
                ui.set_cursor_pos([ui.cursor_pos()[0], y + (size::CONTROL - size::SMALL) / 2.0]);
                clicked = button_small(ui, f, ButtonKind::Secondary, a);
            }
        });
    clicked
}
