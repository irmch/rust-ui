//! Map view: a tiled background image with layers drawn in world units.
//!
//! Three pieces, each replaceable on its own:
//!
//! - [`MapView`] owns the camera (`center` in world units, `zoom` in screen
//!   pixels per world unit), pans with the left mouse button, zooms with the
//!   wheel around the cursor and can follow a target. [`MapView::show`]
//!   draws one frame and returns a [`MapResponse`] with what the mouse did.
//! - [`TileSource`] supplies the background block by block, exactly like
//!   Lineage 2 map regions or slippy-map tiles: the view asks only for the
//!   tiles that are visible, a tile that is not ready yet is drawn as a
//!   placeholder and asked for again next frame. [`TileGrid`] is the
//!   ready-made source: the view marks tiles *pending*, the application
//!   decodes and uploads them whenever it likes and calls
//!   [`TileGrid::set`]. The crate never touches image files or textures.
//! - [`Canvas`] is what the overlay closure receives: world → screen
//!   conversion plus lines, rects, circles, text, markers, a culled
//!   cell-filler for geodata and grids. Layers never deal with pan or zoom.
//!
//! ```no_run
//! # use imgui_kit::map::{MapView, TileGrid};
//! # fn frame(ui: &imgui::Ui, view: &mut MapView, tiles: &mut TileGrid, player: [f32; 2]) {
//! view.show(ui, "##map", [0.0, 0.0], tiles, Some(player), |c| {
//!     c.cells([0.0, 0.0], 16.0, 6.0, |cx, cy| Some(if (cx + cy) % 2 == 0 { [0.2, 0.3, 0.2, 0.6] } else { [0.1, 0.2, 0.1, 0.6] }));
//!     c.marker(player, [0.36, 0.79, 0.54, 1.0], 6.0);
//! });
//! for tile in tiles.take_pending() {
//!     // decode "assets/map/{x}_{y}.png", upload it, then:
//!     // tiles.set(tile, texture_id);
//! }
//! # }
//! ```

use std::collections::HashMap;

use imgui::{DrawListMut, MouseButton, StyleColor, StyleVar, TextureId, Ui, WindowFlags};

use crate::tokens::{color, size, space, Rgba};

// ---------------------------------------------------------------------------
// Tiles
// ---------------------------------------------------------------------------

/// What a [`TileSource`] knows about one tile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tile {
    /// Draw this texture.
    Ready(TextureId),
    /// Not available yet: a placeholder is drawn and the tile is asked for
    /// again next frame.
    Loading,
    /// There is no image for this tile: plain background.
    Missing,
}

/// Background image supplier. Tile `[ix, iy]` covers the world rect from
/// `origin + [ix, iy] * tile_size` to `origin + [ix + 1, iy + 1] * tile_size`.
pub trait TileSource {
    /// World position of the corner of tile `[0, 0]`.
    fn origin(&self) -> [f32; 2];
    /// Size of one tile in world units.
    fn tile_size(&self) -> [f32; 2];
    /// State of a tile. Called once per visible tile per frame, so this is
    /// where an implementation starts loading what it does not have.
    fn tile(&mut self, tile: [i32; 2]) -> Tile;
    /// Label shown on placeholders and with [`MapView::show_tile_labels`].
    fn label(&self, tile: [i32; 2]) -> String {
        format!("{}_{}", tile[0], tile[1])
    }
}

/// Loading state of a [`TileGrid`] tile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileState {
    /// Requested by the view, not yet handed out by [`TileGrid::take_pending`].
    Pending,
    /// Handed to the application, waiting for [`TileGrid::set`] / [`TileGrid::missing`].
    Loading,
    Ready(TextureId),
    Missing,
}

/// [`TileSource`] backed by a map of states the application fills in.
///
/// Each frame the view marks the visible tiles it lacks as pending; the
/// application takes them with [`TileGrid::take_pending`], loads and uploads
/// them (synchronously or on a thread) and reports back with
/// [`TileGrid::set`] or [`TileGrid::missing`]. Tiles outside `bounds` are
/// missing without ever being requested.
pub struct TileGrid {
    pub origin: [f32; 2],
    pub tile_size: [f32; 2],
    /// Inclusive `[min, max]` tile indices that exist, if known.
    pub bounds: Option<[[i32; 2]; 2]>,
    tiles: HashMap<[i32; 2], TileState>,
}

impl TileGrid {
    pub fn new(origin: [f32; 2], tile_size: [f32; 2]) -> Self {
        Self {
            origin,
            tile_size,
            bounds: None,
            tiles: HashMap::new(),
        }
    }

    /// Limits the grid to the inclusive index range `min ..= max`.
    pub fn with_bounds(mut self, min: [i32; 2], max: [i32; 2]) -> Self {
        self.bounds = Some([min, max]);
        self
    }

    pub fn in_bounds(&self, tile: [i32; 2]) -> bool {
        match self.bounds {
            Some([min, max]) => tile[0] >= min[0] && tile[0] <= max[0] && tile[1] >= min[1] && tile[1] <= max[1],
            None => true,
        }
    }

    /// Tiles the view asked for since the last call; they become `Loading`.
    pub fn take_pending(&mut self) -> Vec<[i32; 2]> {
        self.take_pending_limit(usize::MAX)
    }

    /// Like [`TileGrid::take_pending`] but at most `max` tiles, so a host
    /// that decodes synchronously can spread the work over frames; the rest
    /// stay pending and are returned by later calls.
    pub fn take_pending_limit(&mut self, max: usize) -> Vec<[i32; 2]> {
        let mut out = Vec::new();
        for (k, v) in &mut self.tiles {
            if out.len() >= max {
                break;
            }
            if *v == TileState::Pending {
                *v = TileState::Loading;
                out.push(*k);
            }
        }
        out
    }

    /// Tiles still waiting for [`TileGrid::take_pending`].
    pub fn pending_count(&self) -> usize {
        self.tiles.values().filter(|v| **v == TileState::Pending).count()
    }

    pub fn set(&mut self, tile: [i32; 2], texture: TextureId) {
        self.tiles.insert(tile, TileState::Ready(texture));
    }

    pub fn missing(&mut self, tile: [i32; 2]) {
        self.tiles.insert(tile, TileState::Missing);
    }

    pub fn state(&self, tile: [i32; 2]) -> TileState {
        self.tiles.get(&tile).copied().unwrap_or(TileState::Pending)
    }

    /// Forgets a tile so the view requests it again (after the texture was
    /// freed, for instance).
    pub fn evict(&mut self, tile: [i32; 2]) -> Option<TileState> {
        self.tiles.remove(&tile)
    }

    /// Number of tiles in each state: `(ready, loading + pending, missing)`.
    pub fn counts(&self) -> (usize, usize, usize) {
        let mut c = (0, 0, 0);
        for v in self.tiles.values() {
            match v {
                TileState::Ready(_) => c.0 += 1,
                TileState::Pending | TileState::Loading => c.1 += 1,
                TileState::Missing => c.2 += 1,
            }
        }
        c
    }

    /// Tile index containing world position `p`.
    pub fn tile_at(&self, p: [f32; 2]) -> [i32; 2] {
        [
            ((p[0] - self.origin[0]) / self.tile_size[0]).floor() as i32,
            ((p[1] - self.origin[1]) / self.tile_size[1]).floor() as i32,
        ]
    }
}

impl TileSource for TileGrid {
    fn origin(&self) -> [f32; 2] {
        self.origin
    }

    fn tile_size(&self) -> [f32; 2] {
        self.tile_size
    }

    fn tile(&mut self, tile: [i32; 2]) -> Tile {
        if !self.in_bounds(tile) {
            return Tile::Missing;
        }
        match self.tiles.entry(tile).or_insert(TileState::Pending) {
            TileState::Ready(t) => Tile::Ready(*t),
            TileState::Missing => Tile::Missing,
            TileState::Pending | TileState::Loading => Tile::Loading,
        }
    }
}

// ---------------------------------------------------------------------------
// Canvas
// ---------------------------------------------------------------------------

/// Drawing context handed to the overlay: world units in, pixels out.
pub struct Canvas<'ui> {
    dl: DrawListMut<'ui>,
    /// Screen position of the top-left corner of the view.
    pub origin: [f32; 2],
    /// Size of the view in pixels.
    pub size: [f32; 2],
    /// World position at the centre of the view.
    pub center: [f32; 2],
    /// Screen pixels per world unit.
    pub zoom: f32,
    /// World position under the mouse while it is over the view.
    pub mouse_world: Option<[f32; 2]>,
    /// `style.alpha` at draw time, applied to every colour (page fades,
    /// `disabled()`); the draw list does not do it by itself.
    pub alpha: f32,
}

fn world_to_screen(origin: [f32; 2], size: [f32; 2], center: [f32; 2], zoom: f32, w: [f32; 2]) -> [f32; 2] {
    [
        origin[0] + size[0] * 0.5 + (w[0] - center[0]) * zoom,
        origin[1] + size[1] * 0.5 + (w[1] - center[1]) * zoom,
    ]
}

fn screen_to_world(origin: [f32; 2], size: [f32; 2], center: [f32; 2], zoom: f32, s: [f32; 2]) -> [f32; 2] {
    [
        center[0] + (s[0] - origin[0] - size[0] * 0.5) / zoom,
        center[1] + (s[1] - origin[1] - size[1] * 0.5) / zoom,
    ]
}

impl<'ui> Canvas<'ui> {
    /// `c` with the view's current alpha applied.
    pub fn c(&self, c: Rgba) -> Rgba {
        [c[0], c[1], c[2], c[3] * self.alpha]
    }

    /// World → screen.
    pub fn to_screen(&self, w: [f32; 2]) -> [f32; 2] {
        world_to_screen(self.origin, self.size, self.center, self.zoom, w)
    }

    /// Screen → world.
    pub fn to_world(&self, s: [f32; 2]) -> [f32; 2] {
        screen_to_world(self.origin, self.size, self.center, self.zoom, s)
    }

    /// Length in world units → pixels.
    pub fn px(&self, world_len: f32) -> f32 {
        world_len * self.zoom
    }

    /// Visible world rectangle `(min, max)`.
    pub fn world_rect(&self) -> ([f32; 2], [f32; 2]) {
        (
            self.to_world(self.origin),
            self.to_world([self.origin[0] + self.size[0], self.origin[1] + self.size[1]]),
        )
    }

    /// Whether the world rect `a..b` intersects the view.
    pub fn is_visible(&self, a: [f32; 2], b: [f32; 2]) -> bool {
        let (min, max) = self.world_rect();
        a[0].min(b[0]) <= max[0] && a[0].max(b[0]) >= min[0] && a[1].min(b[1]) <= max[1] && a[1].max(b[1]) >= min[1]
    }

    /// The raw draw list, already clipped to the view.
    pub fn draw_list(&self) -> &DrawListMut<'ui> {
        &self.dl
    }

    pub fn line(&self, a: [f32; 2], b: [f32; 2], col: Rgba, thickness: f32) {
        self.dl.add_line(self.to_screen(a), self.to_screen(b), self.c(col)).thickness(thickness).build();
    }

    pub fn polyline(&self, points: &[[f32; 2]], col: Rgba, thickness: f32) {
        if points.len() < 2 {
            return;
        }
        let pts: Vec<[f32; 2]> = points.iter().map(|p| self.to_screen(*p)).collect();
        self.dl.add_polyline(pts, self.c(col)).thickness(thickness).build();
    }

    pub fn rect(&self, a: [f32; 2], b: [f32; 2], col: Rgba, thickness: f32) {
        self.dl.add_rect(self.to_screen(a), self.to_screen(b), self.c(col)).thickness(thickness).build();
    }

    pub fn rect_filled(&self, a: [f32; 2], b: [f32; 2], col: Rgba) {
        self.dl.add_rect(self.to_screen(a), self.to_screen(b), self.c(col)).filled(true).build();
    }

    /// Circle of `radius_px` pixels around world point `c`.
    pub fn circle(&self, c: [f32; 2], radius_px: f32, col: Rgba, thickness: f32) {
        self.dl.add_circle(self.to_screen(c), radius_px, self.c(col)).thickness(thickness).build();
    }

    pub fn circle_filled(&self, c: [f32; 2], radius_px: f32, col: Rgba) {
        self.dl.add_circle(self.to_screen(c), radius_px, self.c(col)).filled(true).build();
    }

    /// Circle whose radius is in world units (a sight range, an aggro radius).
    pub fn circle_world(&self, c: [f32; 2], radius: f32, col: Rgba, thickness: f32) {
        let r = self.px(radius);
        let segs = (r / 2.0).clamp(12.0, 128.0) as u32;
        self.dl.add_circle(self.to_screen(c), r, self.c(col)).thickness(thickness).num_segments(segs).build();
    }

    pub fn circle_world_filled(&self, c: [f32; 2], radius: f32, col: Rgba) {
        let r = self.px(radius);
        let segs = (r / 2.0).clamp(12.0, 128.0) as u32;
        self.dl.add_circle(self.to_screen(c), r, self.c(col)).filled(true).num_segments(segs).build();
    }

    /// Text with its top-left corner at world point `w`, offset by `off` pixels.
    pub fn text(&self, w: [f32; 2], off: [f32; 2], col: Rgba, s: &str) {
        let p = self.to_screen(w);
        self.dl.add_text([p[0] + off[0], p[1] + off[1]], self.c(col), s);
    }

    /// Text on a dark box, centred `above_px` pixels above world point `w`.
    pub fn label(&self, ui: &Ui, w: [f32; 2], above_px: f32, col: Rgba, s: &str) {
        let p = self.to_screen(w);
        let ts = ui.calc_text_size(s);
        let a = [(p[0] - ts[0] / 2.0 - space::XS).round(), (p[1] - above_px - ts[1] - space::XS).round()];
        let b = [a[0] + ts[0] + 2.0 * space::XS, a[1] + ts[1] + 2.0 * space::XS];
        self.dl.add_rect(a, b, self.c(color::DIM)).rounding(size::RADIUS).filled(true).build();
        self.dl.add_text([a[0] + space::XS, a[1] + space::XS], self.c(col), s);
    }

    /// Filled dot with a dark outline: entities, waypoints.
    pub fn marker(&self, w: [f32; 2], col: Rgba, radius_px: f32) {
        let p = self.to_screen(w);
        self.dl.add_circle(p, radius_px + 1.5, self.c(color::BG0)).filled(true).build();
        self.dl.add_circle(p, radius_px, self.c(col)).filled(true).build();
    }

    /// Arrow head at `w` pointing along `heading` radians (0 = +x, π/2 = +y).
    pub fn arrow(&self, w: [f32; 2], heading: f32, len_px: f32, col: Rgba) {
        let p = self.to_screen(w);
        let (s, c) = heading.sin_cos();
        let tip = [p[0] + c * len_px, p[1] + s * len_px];
        let back = len_px * 0.55;
        let wing = len_px * 0.45;
        let l = [p[0] - c * back - s * wing, p[1] - s * back + c * wing];
        let r = [p[0] - c * back + s * wing, p[1] - s * back - c * wing];
        self.dl.add_triangle(tip, l, r, self.c(color::BG0)).thickness(3.0).build();
        self.dl.add_triangle(tip, l, r, self.c(col)).filled(true).build();
    }

    /// Image covering the world rect `a..b`.
    pub fn image(&self, tex: TextureId, a: [f32; 2], b: [f32; 2]) {
        self.dl.add_image(tex, self.to_screen(a), self.to_screen(b)).col(self.c([1.0, 1.0, 1.0, 1.0])).build();
    }

    /// Fills the visible cells of a grid of `cell` world units anchored at
    /// `origin`; `color(cx, cy)` returns the fill or `None` to skip. Nothing
    /// is drawn when a cell would be under `min_px` pixels (too dense to
    /// read, too many rects to draw); returns whether it drew. Pair two calls
    /// with different cell sizes for level-of-detail.
    pub fn cells(&self, origin: [f32; 2], cell: f32, min_px: f32, mut color: impl FnMut(i32, i32) -> Option<Rgba>) -> bool {
        if self.px(cell) < min_px {
            return false;
        }
        let (min, max) = self.world_rect();
        let cx0 = ((min[0] - origin[0]) / cell).floor() as i32;
        let cy0 = ((min[1] - origin[1]) / cell).floor() as i32;
        let cx1 = ((max[0] - origin[0]) / cell).floor() as i32;
        let cy1 = ((max[1] - origin[1]) / cell).floor() as i32;
        for cy in cy0..=cy1 {
            for cx in cx0..=cx1 {
                if let Some(col) = color(cx, cy) {
                    let a = [origin[0] + cx as f32 * cell, origin[1] + cy as f32 * cell];
                    self.rect_filled(a, [a[0] + cell, a[1] + cell], col);
                }
            }
        }
        true
    }

    /// Grid lines every `step` world units from `origin`, skipped when the
    /// step would be under `min_px` pixels. Returns whether it drew.
    pub fn grid(&self, origin: [f32; 2], step: f32, min_px: f32, col: Rgba) -> bool {
        if self.px(step) < min_px {
            return false;
        }
        let (min, max) = self.world_rect();
        let x0 = ((min[0] - origin[0]) / step).floor() as i32;
        let x1 = ((max[0] - origin[0]) / step).ceil() as i32;
        let y0 = ((min[1] - origin[1]) / step).floor() as i32;
        let y1 = ((max[1] - origin[1]) / step).ceil() as i32;
        let top = self.origin[1];
        let bottom = self.origin[1] + self.size[1];
        let left = self.origin[0];
        let right = self.origin[0] + self.size[0];
        for x in x0..=x1 {
            let sx = self.to_screen([origin[0] + x as f32 * step, 0.0])[0].round() + 0.5;
            self.dl.add_line([sx, top], [sx, bottom], self.c(col)).build();
        }
        for y in y0..=y1 {
            let sy = self.to_screen([0.0, origin[1] + y as f32 * step])[1].round() + 0.5;
            self.dl.add_line([left, sy], [right, sy], self.c(col)).build();
        }
        true
    }

    fn draw_tiles(&self, ui: &Ui, tiles: &mut dyn TileSource, grid: bool, labels: bool) {
        let o = tiles.origin();
        let ts = tiles.tile_size();
        let (min, max) = self.world_rect();
        let x0 = ((min[0] - o[0]) / ts[0]).floor() as i32;
        let x1 = ((max[0] - o[0]) / ts[0]).floor() as i32;
        let y0 = ((min[1] - o[1]) / ts[1]).floor() as i32;
        let y1 = ((max[1] - o[1]) / ts[1]).floor() as i32;
        // Never ask for an absurd number of tiles when zoomed far out.
        if (x1 - x0 + 1) as i64 * (y1 - y0 + 1) as i64 > 4096 {
            return;
        }
        let px = [self.px(ts[0]), self.px(ts[1])];
        for iy in y0..=y1 {
            for ix in x0..=x1 {
                let a = self.to_screen([o[0] + ix as f32 * ts[0], o[1] + iy as f32 * ts[1]]);
                let b = [a[0] + px[0], a[1] + px[1]];
                let state = tiles.tile([ix, iy]);
                match state {
                    Tile::Ready(t) => {
                        self.dl.add_image(t, a, b).col(self.c([1.0, 1.0, 1.0, 1.0])).build();
                    }
                    Tile::Loading => {
                        self.dl.add_rect(a, b, self.c(color::BG1)).filled(true).build();
                    }
                    Tile::Missing => {}
                }
                if grid || state == Tile::Loading {
                    self.dl.add_rect(a, b, self.c(color::LINE2)).build();
                }
                if (labels || state == Tile::Loading) && px[0] >= 48.0 && px[1] >= 24.0 {
                    let s = if state == Tile::Loading {
                        format!("{} …", tiles.label([ix, iy]))
                    } else {
                        tiles.label([ix, iy])
                    };
                    let _ = ui;
                    self.dl.add_text([a[0] + space::XS, a[1] + space::XS], self.c(color::FG3), s);
                }
            }
        }
    }

    fn hud(&self, ui: &Ui) {
        let mut lines = vec![format!("zoom {}", zoom_label(self.zoom))];
        if let Some(m) = self.mouse_world {
            lines.push(format!("x {:.0}  y {:.0}", m[0], m[1]));
        }
        let text = lines.join("   ");
        let ts = ui.calc_text_size(&text);
        let a = [self.origin[0] + space::S, self.origin[1] + self.size[1] - ts[1] - 2.0 * space::XS - space::S];
        let b = [a[0] + ts[0] + 2.0 * space::XS, a[1] + ts[1] + 2.0 * space::XS];
        self.dl.add_rect(a, b, self.c(color::DIM)).rounding(size::RADIUS).filled(true).build();
        self.dl.add_text([a[0] + space::XS, a[1] + space::XS], self.c(color::FG2), text);
    }
}

/// `1:64` style label for a zoom factor.
pub fn zoom_label(zoom: f32) -> String {
    if zoom >= 1.0 {
        format!("{:.1}:1", zoom)
    } else {
        format!("1:{:.0}", 1.0 / zoom)
    }
}

// ---------------------------------------------------------------------------
// View
// ---------------------------------------------------------------------------

/// What happened to the view this frame.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MapResponse {
    pub hovered: bool,
    /// World position under the mouse while hovered.
    pub mouse_world: Option<[f32; 2]>,
    /// World position of a left click that did not pan.
    pub clicked: Option<[f32; 2]>,
    pub panned: bool,
    pub zoomed: bool,
    /// Screen rect of the view.
    pub origin: [f32; 2],
    pub size: [f32; 2],
}

/// Camera + input for a map; draw it with [`MapView::show`].
#[derive(Clone, Debug, PartialEq)]
pub struct MapView {
    /// World position at the centre of the view.
    pub center: [f32; 2],
    /// Screen pixels per world unit.
    pub zoom: f32,
    pub min_zoom: f32,
    pub max_zoom: f32,
    /// Zoom factor per wheel notch.
    pub wheel_step: f32,
    /// Keep `center` on the target passed to [`MapView::show`]. Panning by
    /// hand turns it off; set it again to re-attach.
    pub follow: bool,
    pub show_tile_grid: bool,
    pub show_tile_labels: bool,
    /// Zoom and mouse position box in the bottom-left corner.
    pub show_hud: bool,
}

impl Default for MapView {
    fn default() -> Self {
        Self {
            center: [0.0, 0.0],
            zoom: 1.0,
            min_zoom: 1.0 / 1024.0,
            max_zoom: 16.0,
            wheel_step: 1.25,
            follow: true,
            show_tile_grid: false,
            show_tile_labels: false,
            show_hud: true,
        }
    }
}

impl MapView {
    pub fn center_on(&mut self, p: [f32; 2]) {
        self.center = p;
    }

    /// Centres on the world rect `min..max` and picks the zoom that fits it
    /// into `view_px` pixels, within the zoom limits.
    pub fn fit(&mut self, min: [f32; 2], max: [f32; 2], view_px: [f32; 2]) {
        let w = (max[0] - min[0]).abs().max(1.0);
        let h = (max[1] - min[1]).abs().max(1.0);
        self.center = [(min[0] + max[0]) * 0.5, (min[1] + max[1]) * 0.5];
        self.zoom = (view_px[0] / w).min(view_px[1] / h).clamp(self.min_zoom, self.max_zoom);
    }

    /// Multiplies the zoom by `factor`, keeping the centre.
    pub fn zoom_by(&mut self, factor: f32) {
        self.zoom = (self.zoom * factor).clamp(self.min_zoom, self.max_zoom);
    }

    /// Draws the view as a bordered child of `size_` (`0.0` = fill) and
    /// calls `overlay` on top of the tiles. `follow` is the position to stay
    /// centred on while [`MapView::follow`] is set.
    pub fn show(
        &mut self,
        ui: &Ui,
        id: &str,
        size_: [f32; 2],
        tiles: &mut dyn TileSource,
        follow: Option<[f32; 2]>,
        overlay: impl FnOnce(&Canvas<'_>),
    ) -> MapResponse {
        let mut resp = MapResponse::default();
        let _bs = ui.push_style_var(StyleVar::ChildBorderSize(size::BORDER));
        let _pad = ui.push_style_var(StyleVar::WindowPadding([0.0, 0.0]));
        let _bg = ui.push_style_color(StyleColor::ChildBg, color::BG0);
        let _bd = ui.push_style_color(StyleColor::Border, color::LINE);
        ui.child_window(id)
            .size(size_)
            .border(true)
            .flags(WindowFlags::NO_SCROLLBAR | WindowFlags::NO_SCROLL_WITH_MOUSE | WindowFlags::NO_MOVE)
            .build(|| {
                let origin = ui.cursor_screen_pos();
                let sz = ui.content_region_avail();
                if sz[0] < 1.0 || sz[1] < 1.0 {
                    return;
                }
                if self.follow {
                    if let Some(t) = follow {
                        self.center = t;
                    }
                }
                ui.invisible_button("##canvas", sz);
                let hovered = ui.is_item_hovered();
                let active = ui.is_item_active();
                let mouse = ui.io().mouse_pos;
                let wheel = ui.io().mouse_wheel;
                let delta = ui.io().mouse_delta;

                // pan: drag with the left button
                if active && ui.is_mouse_dragging(MouseButton::Left) && (delta[0] != 0.0 || delta[1] != 0.0) {
                    self.center[0] -= delta[0] / self.zoom;
                    self.center[1] -= delta[1] / self.zoom;
                    self.follow = false;
                    resp.panned = true;
                }
                // zoom: wheel, keeping the world point under the cursor still
                if hovered && wheel != 0.0 {
                    let before = screen_to_world(origin, sz, self.center, self.zoom, mouse);
                    self.zoom = (self.zoom * self.wheel_step.powf(wheel)).clamp(self.min_zoom, self.max_zoom);
                    let after = screen_to_world(origin, sz, self.center, self.zoom, mouse);
                    if !self.follow {
                        self.center[0] += before[0] - after[0];
                        self.center[1] += before[1] - after[1];
                    }
                    resp.zoomed = true;
                }

                // Only one DrawListMut may exist at a time: this is the one.
                let canvas = Canvas {
                    dl: ui.get_window_draw_list(),
                    origin,
                    size: sz,
                    center: self.center,
                    zoom: self.zoom,
                    mouse_world: hovered.then(|| screen_to_world(origin, sz, self.center, self.zoom, mouse)),
                    alpha: crate::widgets::style_alpha(ui),
                };
                resp.hovered = hovered;
                resp.mouse_world = canvas.mouse_world;
                resp.origin = origin;
                resp.size = sz;
                if hovered && ui.is_mouse_released(MouseButton::Left) {
                    let d = ui.mouse_drag_delta_with_threshold(MouseButton::Left, 0.0);
                    if d[0].abs() < 3.0 && d[1].abs() < 3.0 {
                        resp.clicked = canvas.mouse_world;
                    }
                }

                canvas.draw_tiles(ui, tiles, self.show_tile_grid, self.show_tile_labels);
                overlay(&canvas);
                if self.show_hud {
                    canvas.hud(ui);
                }
            });
        resp
    }
}
