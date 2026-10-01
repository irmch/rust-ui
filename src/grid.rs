//! The 12-column grid of artboard 02 and the layout recipes next to it.
//!
//! Reference window 1280 × 800: 12 columns of 88 px, 16 px gutters and 24 px
//! margins. The form pane spans 5 columns (504 px) and the content pane 7
//! (712 px). Below 1280 px the form pane drops to 4 columns.

use imgui::{StyleVar, TableColumnFlags, TableColumnSetup, TableFlags, Ui};

use crate::tokens::{size, space};

/// Width that makes an item fill the remaining content region.
pub const FILL: f32 = -f32::MIN_POSITIVE;

/// Side of a two-pane layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pane {
    Left,
    Right,
}

/// Column grid definition.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grid {
    pub cols: u32,
    pub gutter: f32,
    pub margin: f32,
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            cols: 12,
            gutter: space::L,
            margin: space::XL,
        }
    }
}

impl Grid {
    /// Width of one column for the given available width (already inside the
    /// window padding, i.e. `ui.content_region_avail()[0]`).
    pub fn col_width(&self, avail: f32) -> f32 {
        (avail - self.gutter * (self.cols as f32 - 1.0)) / self.cols as f32
    }

    /// Width of `span` columns including the gutters between them.
    pub fn span_width(&self, avail: f32, span: u32) -> f32 {
        let span = span.clamp(1, self.cols) as f32;
        self.col_width(avail) * span + self.gutter * (span - 1.0)
    }

    /// Form-pane span for a window width: 5 at the reference size and above,
    /// 4 on narrower windows.
    pub fn form_span(window_width: f32) -> u32 {
        if window_width >= 1280.0 {
            5
        } else {
            4
        }
    }

    /// Two side-by-side child windows: the left one spans `left_span`
    /// columns, the right one fills the rest. Both take the full remaining
    /// height. Each closure draws into its pane.
    pub fn two_panes(
        &self,
        ui: &Ui,
        left_span: u32,
        left: impl FnOnce(&Ui),
        right: impl FnOnce(&Ui),
    ) {
        let avail = ui.content_region_avail();
        let left_w = self.span_width(avail[0], left_span).round();
        let _pad = ui.push_style_var(StyleVar::WindowPadding([0.0, 0.0]));
        ui.child_window("##pane_left")
            .size([left_w, 0.0])
            .build(|| left(ui));
        ui.same_line_with_spacing(0.0, self.gutter);
        ui.child_window("##pane_right")
            .size([0.0, 0.0])
            .build(|| right(ui));
    }

    /// Like [`Grid::two_panes`] but with one closure called for each pane,
    /// so it can hold `&mut` state shared by both sides.
    pub fn panes(&self, ui: &Ui, left_span: u32, mut pane: impl FnMut(&Ui, Pane)) {
        let avail = ui.content_region_avail();
        let left_w = self.span_width(avail[0], left_span).round();
        let _pad = ui.push_style_var(StyleVar::WindowPadding([0.0, 0.0]));
        ui.child_window("##pane_left")
            .size([left_w, 0.0])
            .build(|| pane(ui, Pane::Left));
        ui.same_line_with_spacing(0.0, self.gutter);
        ui.child_window("##pane_right")
            .size([0.0, 0.0])
            .build(|| pane(ui, Pane::Right));
    }

    /// Lays out one row of cells with the given column spans (they should add
    /// up to `cols`). `cell` is called with the cell index and its width.
    pub fn row(&self, ui: &Ui, id: &str, spans: &[u32], mut cell: impl FnMut(&Ui, usize, f32)) {
        let avail = ui.content_region_avail()[0];
        let mut x = ui.cursor_pos()[0];
        let y = ui.cursor_pos()[1];
        let mut max_y = y;
        for (i, &span) in spans.iter().enumerate() {
            let w = self.span_width(avail, span).round();
            ui.set_cursor_pos([x, y]);
            let _id = ui.push_id_usize(i);
            ui.group(|| {
                let _w = ui.push_item_width(w);
                cell(ui, i, w);
            });
            max_y = max_y.max(ui.cursor_pos()[1]);
            x += w + self.gutter;
        }
        let _ = id;
        ui.set_cursor_pos([ui.cursor_pos()[0], max_y]);
    }
}

/// Which cell of a [`form_row`] is being drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormCell {
    Label,
    Control,
    Value,
}

/// Form row of the launcher: `label 96 | control stretch | value 64`
/// (artboard 03 · section 16). `cell` is called three times with the cell
/// and its width; one closure so it can own mutable state across cells.
pub fn form_row(ui: &Ui, id: &str, mut cell: impl FnMut(&Ui, FormCell, f32)) {
    let flags = TableFlags::SIZING_STRETCH_PROP | TableFlags::NO_PAD_OUTER_X;
    let _pad = ui.push_style_var(StyleVar::CellPadding([space::L / 2.0, 0.0]));
    if let Some(_t) = ui.begin_table_with_flags(id, 3, flags) {
        ui.table_setup_column_with(TableColumnSetup {
            name: "label",
            flags: TableColumnFlags::WIDTH_FIXED,
            init_width_or_weight: size::FORM_LABEL,
            user_id: Default::default(),
        });
        ui.table_setup_column_with(TableColumnSetup {
            name: "control",
            flags: TableColumnFlags::WIDTH_STRETCH,
            init_width_or_weight: 1.0,
            user_id: Default::default(),
        });
        ui.table_setup_column_with(TableColumnSetup {
            name: "value",
            flags: TableColumnFlags::WIDTH_FIXED,
            init_width_or_weight: size::FORM_VALUE,
            user_id: Default::default(),
        });
        ui.table_next_row_with_height(imgui::TableRowFlags::empty(), size::CONTROL);
        for c in [FormCell::Label, FormCell::Control, FormCell::Value] {
            ui.table_next_column();
            let w = ui.content_region_avail()[0];
            cell(ui, c, w);
        }
    }
}

/// Width left for a text input followed by fixed-width buttons on the same
/// line (artboard 03 · section 14): `avail − Σ buttons − gutters`.
pub fn input_group_width(ui: &Ui, button_widths: &[f32]) -> f32 {
    let avail = ui.content_region_avail()[0];
    let spacing = ui.clone_style().item_spacing[0];
    avail - button_widths.iter().sum::<f32>() - spacing * button_widths.len() as f32
}

/// Width a button with `label` takes: text + 2 × 16 padding.
pub fn button_width(ui: &Ui, label: &str) -> f32 {
    ui.calc_text_size(label)[0] + 2.0 * size::BUTTON_PAD_X
}

/// Moves the cursor so that a group of items with `widths` ends at the
/// right edge of the content region.
pub fn right_align(ui: &Ui, widths: &[f32]) {
    let spacing = ui.clone_style().item_spacing[0];
    let group: f32 = widths.iter().sum::<f32>() + spacing * (widths.len().saturating_sub(1)) as f32;
    let avail = ui.content_region_avail()[0];
    let x = ui.cursor_pos()[0] + (avail - group).max(0.0);
    ui.set_cursor_pos([x, ui.cursor_pos()[1]]);
}

/// Moves the cursor so that an item `item_w` wide is centred in the content
/// region.
pub fn center(ui: &Ui, item_w: f32) {
    let avail = ui.content_region_avail()[0];
    let x = ui.cursor_pos()[0] + ((avail - item_w) / 2.0).max(0.0);
    ui.set_cursor_pos([x, ui.cursor_pos()[1]]);
}

/// Vertically centres the next item of height `item_h` inside a row of
/// `row_h` that starts at the current cursor y.
pub fn vcenter(ui: &Ui, item_h: f32, row_h: f32) {
    let p = ui.cursor_pos();
    ui.set_cursor_pos([p[0], p[1] + ((row_h - item_h) / 2.0).round()]);
}

/// Gap between two sections: 24 · separator · 24 (artboard 02 · rhythm).
pub fn section_gap(ui: &Ui) {
    ui.dummy([0.0, space::XL - space::S]);
    ui.separator();
    ui.dummy([0.0, space::XL - space::S]);
}

/// Pushes the cursor down so that an item `item_h` tall sits at the bottom of
/// the current window with `bottom_pad` below it. No-op if there is no room.
pub fn push_to_bottom(ui: &Ui, item_h: f32, bottom_pad: f32) {
    let avail_h = ui.content_region_avail()[1];
    let dy = avail_h - item_h - bottom_pad;
    if dy > 0.0 {
        ui.dummy([0.0, dy - ui.clone_style().item_spacing[1]]);
    }
}

/// Snaps a coordinate to the 4 px rhythm.
pub fn snap4(v: f32) -> f32 {
    (v / 4.0).round() * 4.0
}

/// Snaps a coordinate to the 8 px unit.
pub fn snap8(v: f32) -> f32 {
    (v / 8.0).round() * 8.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_grid_matches_the_canvas() {
        let g = Grid::default();
        let avail = 1280.0 - 2.0 * g.margin;
        assert_eq!(g.col_width(avail), 88.0);
        assert_eq!(g.span_width(avail, 5), 504.0);
        assert_eq!(g.span_width(avail, 7), 712.0);
        assert_eq!(g.span_width(avail, 12), avail);
        assert_eq!(g.span_width(avail, 5) + g.gutter + g.span_width(avail, 7), avail);
    }

    #[test]
    fn form_span_drops_below_reference_width() {
        assert_eq!(Grid::form_span(1280.0), 5);
        assert_eq!(Grid::form_span(1600.0), 5);
        assert_eq!(Grid::form_span(1024.0), 4);
    }

    #[test]
    fn snapping() {
        assert_eq!(snap4(13.0), 12.0);
        assert_eq!(snap8(13.0), 16.0);
    }
}
