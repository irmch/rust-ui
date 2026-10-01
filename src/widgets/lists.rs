use imgui::Ui;

use crate::Kit;
use crate::grid;
use crate::tokens::{color, size, space};

#[allow(unused_imports)]
use super::*;

// ---------------------------------------------------------------------------
// Rows and lists
// ---------------------------------------------------------------------------

/// Clickable row around `body`: hover fill, and when `selected` a 1 px
/// line border with a 2 px accent strip on the left. `height` `0.0` takes
/// the body's height. Returns `true` on click.
pub fn list_row(
    ui: &Ui,
    kit: &Kit,
    id: &str,
    selected: bool,
    height: f32,
    body: impl FnOnce(&Ui),
) -> bool {
    let _ = kit;
    let _id = ui.push_id(id);
    let p = ui.cursor_screen_pos();
    let w = ui.content_region_avail()[0];
    ui.group(|| {
        ui.dummy([0.0, space::S]);
        ui.indent_by(size::PAD_X);
        body(ui);
        ui.unindent_by(size::PAD_X);
        ui.dummy([0.0, space::S]);
    });
    let body_h = ui.cursor_screen_pos()[1] - p[1];
    let h = if height > 0.0 { height } else { body_h };
    // Hit-test the whole row, then put the cursor back below it.
    ui.set_cursor_screen_pos(p);
    let clicked = ui.invisible_button("##row", [w, h]);
    let hovered = ui.is_item_hovered();
    ui.set_cursor_screen_pos([p[0], p[1] + h]);
    ui.dummy([0.0, 0.0]);
    let al = style_alpha(ui);
    let dl = ui.get_window_draw_list();
    let q = [p[0] + w, p[1] + h];
    if selected {
        dl.add_rect(p, q, fade(color::LINE2, al))
            .rounding(size::RADIUS)
            .build();
        dl.add_rect(
            [p[0], p[1] + space::S],
            [p[0] + 2.0, q[1] - space::S],
            fade(color::ACCENT, al),
        )
        .filled(true)
        .build();
    } else if hovered {
        dl.add_rect(p, q, fade(color::SEL, al))
            .rounding(size::RADIUS)
            .filled(true)
            .build();
    }
    clicked
}

/// One-line selectable text row (28 px): hover bg-2, selected bg-sel with
/// bold text. Returns `true` on click.
pub fn selectable(ui: &Ui, kit: &Kit, label: &str, selected: bool) -> bool {
    selectable_row(ui, kit, label, selected, false)
}

/// [`selectable`] with an extra keyboard-highlight state (bg-2 fill, as on
/// hover), used by combo lists.
pub(crate) fn selectable_row(
    ui: &Ui,
    kit: &Kit,
    label: &str,
    selected: bool,
    highlighted: bool,
) -> bool {
    let _id = ui.push_id(label);
    let shown = label.split("##").next().unwrap_or("");
    let p = ui.cursor_screen_pos();
    let w = ui.content_region_avail()[0];
    let h = size::ROW;
    let clicked = ui.invisible_button("##sel", [w, h]);
    let hovered = ui.is_item_hovered();
    let al = style_alpha(ui);
    let dl = ui.get_window_draw_list();
    if selected {
        dl.add_rect(p, [p[0] + w, p[1] + h], fade(color::SEL, al))
            .rounding(size::RADIUS)
            .filled(true)
            .build();
    } else if hovered || highlighted {
        dl.add_rect(p, [p[0] + w, p[1] + h], fade(color::BG2, al))
            .rounding(size::RADIUS)
            .filled(true)
            .build();
    }
    let _f = ui.push_font(if selected {
        kit.fonts.mono13b
    } else {
        kit.fonts.mono13
    });
    let ty = p[1] + ((h - ui.text_line_height()) / 2.0).round();
    dl.add_text([p[0] + size::PAD_X, ty], fade(color::FG, al), shown);
    clicked
}

// ---------------------------------------------------------------------------
// Accordion
// ---------------------------------------------------------------------------

/// Header row of a collapsible section: 32 px, chevron + uppercase caption,
/// open state kept in [`crate::kit::UiState`]. Returns whether the section is
/// open; the caller draws the body when it does (drop-in for imgui's
/// `collapsing_header`). `icon` is an optional glyph drawn before the title.
pub fn accordion(ui: &Ui, kit: &Kit, title: &str, icon: Option<&str>, default_open: bool) -> bool {
    // Read and write the state under the same ID stack.
    let _id = ui.push_id(title);
    let mut open = kit.state.is_open(ui, title, default_open);
    let w = ui.content_region_avail()[0];
    let p = ui.cursor_screen_pos();
    let h = size::CONTROL;
    let clicked = ui.invisible_button("##acc", [w, h]);
    let hovered = ui.is_item_hovered();
    if clicked {
        open = !open;
        kit.state.set_open(ui, title, open);
    }
    let al = style_alpha(ui);
    {
        let dl = ui.get_window_draw_list();
        let bg = if hovered { color::BG2 } else { color::BG1 };
        dl.add_rect(p, [p[0] + w, p[1] + h], fade(bg, al))
            .rounding(size::RADIUS)
            .filled(true)
            .build();
        dl.add_rect(p, [p[0] + w, p[1] + h], fade(color::LINE, al))
            .rounding(size::RADIUS)
            .build();
    }
    ui.set_cursor_screen_pos(p);
    let row = grid::Row::start(ui, h);
    let mut x = row.x + size::PAD_X;
    let lh = ui.text_line_height();
    chevron(
        &ui.get_window_draw_list(),
        [x, p[1] + ((h - CHEVRON) / 2.0).round()],
        open,
        fade(color::FG3, al),
    );
    x += CHEVRON + space::S;
    if let Some(g) = icon {
        row.place(ui, x, lh);
        ui.text_colored(color::FG3, g);
        x += ui.calc_text_size(g)[0] + space::S;
    }
    let cap_h = {
        let _f = ui.push_font(kit.fonts.mono10);
        ui.text_line_height()
    };
    row.place(ui, x, cap_h);
    {
        let _f = ui.push_font(kit.fonts.mono10);
        ui.text_colored(
            if hovered { color::FG } else { color::FG2 },
            upper(title).as_ref(),
        );
    }
    row.end(ui);
    ui.dummy([0.0, space::XS]);
    open
}

/// [`accordion`] with the body drawn indented when open.
pub fn accordion_section(
    ui: &Ui,
    kit: &Kit,
    title: &str,
    icon: Option<&str>,
    default_open: bool,
    body: impl FnOnce(&Ui),
) {
    if accordion(ui, kit, title, icon, default_open) {
        ui.indent_by(size::PAD_X);
        body(ui);
        ui.unindent_by(size::PAD_X);
        ui.dummy([0.0, space::S]);
    }
}

// ---------------------------------------------------------------------------
// Stateful tabs
// ---------------------------------------------------------------------------

/// [`tab_strip`] whose active index lives in [`crate::kit::UiState`]: the
/// caller only supplies labels and a body drawn for the active tab. Returns
/// the active index.
pub fn tabs(ui: &Ui, kit: &Kit, id: &str, labels: &[&str], body: impl FnOnce(&Ui, usize)) -> usize {
    let mut active = kit
        .state
        .active_tab(ui, id)
        .min(labels.len().saturating_sub(1));
    if tab_strip(ui, kit, id, labels, &mut active) {
        kit.state.set_active_tab(ui, id, active);
    }
    ui.dummy([0.0, space::S]);
    body(ui, active);
    active
}
