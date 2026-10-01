use imgui::{StyleColor, StyleVar, Ui, WindowFlags};

use crate::Kit;
use crate::anim;
use crate::grid;
use crate::theme::ButtonKind;
use crate::tokens::{Rgba, color, size, space};

#[allow(unused_imports)]
use super::*;

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
pub fn title_bar(
    ui: &Ui,
    kit: &Kit,
    app: &str,
    crumb: &str,
    tabs: &[&str],
    active: &mut usize,
) -> TitleBarAction {
    let mut action = TitleBarAction::None;
    let _pad = ui.push_style_var(StyleVar::WindowPadding([space::XL, 0.0]));
    let _bg = ui.push_style_color(StyleColor::ChildBg, color::BG2);
    ui.child_window("##titlebar")
        .size([0.0, size::BAR])
        .flags(
            WindowFlags::NO_SCROLLBAR
                | WindowFlags::NO_SCROLL_WITH_MOUSE
                | WindowFlags::ALWAYS_USE_WINDOW_PADDING,
        )
        .build(|| {
            bottom_border(ui);
            // left: app / crumb, the 13 px texts on the 16 px app baseline
            let x0 = ui.cursor_pos()[0];
            let (y16, app_w) = {
                let _f = ui.push_font(kit.fonts.mono16b);
                let y = ((size::BAR - ui.text_line_height()) / 2.0).round();
                ui.set_cursor_pos([x0, y]);
                ui.text(app);
                (y, ui.calc_text_size(app)[0])
            };
            let y13 = y16 + ascent(ui, kit.fonts.mono16b) - ascent(ui, kit.fonts.mono13);
            let x = x0 + app_w + space::S;
            ui.set_cursor_pos([x, y13]);
            ui.text_colored(color::FG3, "/");
            ui.set_cursor_pos([x + ui.calc_text_size("/")[0] + space::S, y13]);
            ui.text_colored(color::FG2, crumb);

            // centre: tabs
            let tab_w: Vec<f32> = tabs
                .iter()
                .map(|t| ui.calc_text_size(t)[0] + 2.0 * size::PAD_X)
                .collect();
            let total: f32 =
                tab_w.iter().sum::<f32>() + space::XS * (tabs.len().saturating_sub(1)) as f32;
            let avail = ui.window_size()[0];
            let tabs_x = ((avail - total) / 2.0).round();
            let tabs_y = (size::BAR - size::CONTROL) / 2.0;
            let slide = kit.anim.settings().tabs;
            if slide {
                // One highlight rect that eases from the old tab to the new one;
                // the tab buttons then draw no background of their own.
                let (mut x, mut hl_x, mut hl_w) =
                    (tabs_x, tabs_x, tab_w.first().copied().unwrap_or(0.0));
                for (i, w) in tab_w.iter().enumerate() {
                    if i == *active {
                        hl_x = x;
                        hl_w = *w;
                    }
                    x += w + space::XS;
                }
                let hx =
                    kit.anim
                        .approach(ui, anim::key(ui, "##tabs_hl_x"), hl_x, anim::TABS, true);
                let hw =
                    kit.anim
                        .approach(ui, anim::key(ui, "##tabs_hl_w"), hl_w, anim::TABS, true);
                let wp = ui.window_pos();
                let a = [wp[0] + hx, wp[1] + tabs_y];
                ui.get_window_draw_list()
                    .add_rect(
                        a,
                        [a[0] + hw, a[1] + size::CONTROL],
                        fade(color::BG3, style_alpha(ui)),
                    )
                    .rounding(size::RADIUS)
                    .filled(true)
                    .build();
            }
            ui.set_cursor_pos([tabs_x, tabs_y]);
            for (i, (t, w)) in tabs.iter().zip(&tab_w).enumerate() {
                if i > 0 {
                    ui.same_line_with_spacing(0.0, space::XS);
                }
                if tab_ex(ui, kit, t, *w, i == *active, !slide) {
                    *active = i;
                }
            }

            // right: window controls
            let ctrl = 3.0 * size::CONTROL + 2.0 * space::XS;
            let cy = (size::BAR - size::CONTROL) / 2.0;
            let mut x = avail - space::L - ctrl;
            for which in [
                TitleBarAction::Minimize,
                TitleBarAction::Maximize,
                TitleBarAction::Close,
            ] {
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
    let al = style_alpha(ui);
    let dl = ui.get_window_draw_list();
    if hovered || active {
        let bg = fade(if active { color::BG2 } else { color::BG3 }, al);
        dl.add_rect(p, [p[0] + s, p[1] + s], bg)
            .rounding(size::RADIUS)
            .filled(true)
            .build();
    }
    let fg = fade(if hovered { color::FG } else { color::FG2 }, al);
    let c = [(p[0] + s / 2.0).round(), (p[1] + s / 2.0).round()];
    let r = 5.0;
    match which {
        TitleBarAction::Minimize => {
            dl.add_line([c[0] - r, c[1] + 0.5], [c[0] + r, c[1] + 0.5], fg)
                .build();
        }
        TitleBarAction::Maximize => {
            dl.add_rect(
                [c[0] - r + 0.5, c[1] - r + 0.5],
                [c[0] + r - 0.5, c[1] + r - 0.5],
                fg,
            )
            .build();
        }
        TitleBarAction::Close => {
            dl.add_line([c[0] - r, c[1] - r], [c[0] + r, c[1] + r], fg)
                .build();
            dl.add_line([c[0] - r, c[1] + r], [c[0] + r, c[1] - r], fg)
                .build();
        }
        TitleBarAction::None => {}
    }
    clicked
}

/// Left-aligned row of tabs with the title bar's sliding highlight, for a
/// second navigation level inside a page (sub-tabs of a detail screen).
/// 32 px tall at the current cursor; returns `true` when the active tab
/// changed this frame. Labels may contain icon-font glyphs.
pub fn tab_strip(ui: &Ui, kit: &Kit, id: &str, tabs: &[&str], active: &mut usize) -> bool {
    let _id = ui.push_id(id);
    let row = grid::Row::start(ui, size::CONTROL);
    let widths: Vec<f32> = tabs
        .iter()
        .map(|t| {
            let _f = ui.push_font(kit.fonts.mono13b);
            ui.calc_text_size(t)[0] + 2.0 * size::PAD_X
        })
        .collect();
    let slide = kit.anim.settings().tabs;
    if slide {
        let (mut x, mut hl_x, mut hl_w) = (row.x, row.x, widths.first().copied().unwrap_or(0.0));
        for (i, w) in widths.iter().enumerate() {
            if i == *active {
                hl_x = x;
                hl_w = *w;
            }
            x += w + space::XS;
        }
        let hx = kit
            .anim
            .approach(ui, anim::key(ui, "##strip_hl_x"), hl_x, anim::TABS, true);
        let hw = kit
            .anim
            .approach(ui, anim::key(ui, "##strip_hl_w"), hl_w, anim::TABS, true);
        let wp = ui.window_pos();
        let sy = ui.scroll_y();
        let a = [wp[0] + hx, wp[1] + row.top - sy];
        ui.get_window_draw_list()
            .add_rect(
                a,
                [a[0] + hw, a[1] + size::CONTROL],
                fade(color::BG3, style_alpha(ui)),
            )
            .rounding(size::RADIUS)
            .filled(true)
            .build();
    }
    let mut changed = false;
    let mut x = row.x;
    for (i, (t, w)) in tabs.iter().zip(&widths).enumerate() {
        ui.set_cursor_pos([x, row.top]);
        if tab_ex(ui, kit, t, *w, i == *active, !slide) && i != *active {
            *active = i;
            changed = true;
        }
        x += w + space::XS;
    }
    row.end(ui);
    changed
}

/// One tab of the title bar: 32 px, bg-3 + bold when active, fg-2 otherwise.
pub fn tab(ui: &Ui, kit: &Kit, label: &str, width: f32, active: bool) -> bool {
    tab_ex(ui, kit, label, width, active, true)
}

/// [`tab`] with `own_bg = false` when the caller draws the active highlight
/// itself (the title bar's sliding one).
fn tab_ex(ui: &Ui, kit: &Kit, label: &str, width: f32, active: bool, own_bg: bool) -> bool {
    let _f = ui.push_font(if active {
        kit.fonts.mono13b
    } else {
        kit.fonts.mono13
    });
    let bg = if active && own_bg {
        color::BG3
    } else {
        color::TRANSPARENT
    };
    let fg = if active { color::FG } else { color::FG2 };
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
pub fn status_strip(
    ui: &Ui,
    kit: &Kit,
    items: &[StatItem<'_>],
    action_widths: &[f32],
    actions: impl FnOnce(&Ui),
) {
    let _pad = ui.push_style_var(StyleVar::WindowPadding([space::XL, 0.0]));
    let _bg = ui.push_style_color(StyleColor::ChildBg, color::BG1);
    ui.child_window("##statusstrip")
        .size([0.0, size::BAR])
        .flags(
            WindowFlags::NO_SCROLLBAR
                | WindowFlags::NO_SCROLL_WITH_MOUSE
                | WindowFlags::ALWAYS_USE_WINDOW_PADDING,
        )
        .build(|| {
            bottom_border(ui);
            let row = grid::Row::start(ui, size::BAR);
            let lh = ui.text_line_height();
            for (i, it) in items.iter().enumerate() {
                if i > 0 {
                    ui.same_line_with_spacing(0.0, space::L);
                    row.place_here(ui, size::SMALL);
                    vdivider(ui, size::SMALL);
                    ui.same_line_with_spacing(0.0, space::L);
                }
                row.place_here(ui, lh);
                stat(ui, kit, it.caption, it.value, it.unit, it.color);
            }
            // No `same_line` here: imgui would keep the stats' line y and
            // `same_line` calls inside `actions` would snap back to it.
            row.place(ui, row.x, size::CONTROL);
            grid::right_align(ui, action_widths);
            actions(ui);
        });
}

/// Toolbar row above a panel: caption on the left, small buttons on the
/// right (the "STATUS   Copy Save" header of the log).
pub fn panel_header(
    ui: &Ui,
    kit: &Kit,
    cap: &str,
    action_widths: &[f32],
    actions: impl FnOnce(&Ui),
) {
    let row = grid::Row::start(ui, size::CONTROL);
    let cap_h = {
        let _f = ui.push_font(kit.fonts.mono10);
        ui.text_line_height()
    };
    row.place(ui, row.x, cap_h);
    caption(ui, kit, cap);
    // No `same_line`: it would make imgui snap the actions' `same_line`
    // calls back to the caption's y.
    row.place(ui, row.x, size::SMALL);
    grid::right_align(ui, action_widths);
    actions(ui);
    row.end(ui);
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

/// [`panel`] for logs: scrolls with the wheel and the 8 px scrollbar, and
/// when the view is at the bottom it stays there as lines are appended
/// (scroll up to read back, scroll down to re-attach).
pub fn log_panel(ui: &Ui, id: &str, size_: [f32; 2], body: impl FnOnce(&Ui)) {
    panel(ui, id, size_, |ui| {
        let at_bottom = ui.scroll_y() >= ui.scroll_max_y() - 1.0;
        body(ui);
        if at_bottom {
            ui.set_scroll_here_y_with_ratio(1.0);
        }
    });
}

/// [`log_panel`] for long logs: only the rows in view are submitted (imgui
/// list clipper), so a log of 100k lines costs the same as one of 30.
/// `row_h` is the height of one row (measure it with the row's font pushed)
/// and `row(ui, i)` draws row `i`. Follows appended rows like [`log_panel`].
pub fn log_list(
    ui: &Ui,
    id: &str,
    size_: [f32; 2],
    rows: usize,
    row_h: f32,
    mut row: impl FnMut(&Ui, usize),
) {
    panel(ui, id, size_, |ui| {
        let at_bottom = ui.scroll_y() >= ui.scroll_max_y() - 1.0;
        let _sp = ui.push_style_var(StyleVar::ItemSpacing([space::S, 0.0]));
        let clipper = imgui::ListClipper::new(rows as i32)
            .items_height(row_h)
            .begin(ui);
        for i in clipper.iter() {
            row(ui, i as usize);
        }
        if at_bottom {
            ui.set_scroll_here_y_with_ratio(1.0);
        }
    });
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
pub fn banner(ui: &Ui, kit: &Kit, kind: TagKind, text: &str, action: Option<&str>) -> bool {
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
                let w = button_small_width(ui, kit, a);
                grid::right_align(ui, &[w]);
                let y =
                    ui.cursor_pos()[1] - ((size::CONTROL - ui.text_line_height()) / 2.0).round();
                ui.set_cursor_pos([ui.cursor_pos()[0], y + (size::CONTROL - size::SMALL) / 2.0]);
                clicked = button_small(ui, kit, ButtonKind::Secondary, a);
            }
        });
    clicked
}
