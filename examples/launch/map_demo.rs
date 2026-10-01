//! Map page of the demo: a Lineage 2 style world drawn with [`imgui_kit::map`].
//!
//! The world is cut like L2 geodata: regions of 32768 units (one map image
//! each, named `x_y`), blocks of 8 × 8 cells, cells of 16 units. Geodata and
//! the background come from the same synthetic terrain ([`terrain_height`])
//! so the overlay lines up with the picture; a real application replaces
//! [`geo_cell`] with its parser and [`tile_pixels`] with the map images.

use imgui::{StyleVar, Ui};

use imgui_kit::Kit;
use imgui_kit::map::{self, Layer, MapView, TileGrid};
use imgui_kit::theme::ButtonKind;
use imgui_kit::tokens::{color, space, Rgba};

use imgui_kit::widgets as w;

/// Lineage 2 world layout constants.
pub mod l2 {
    /// Size of one region (one map image) in world units.
    pub const REGION: f32 = 32768.0;
    /// Size of one geodata cell.
    pub const CELL: f32 = 16.0;
    /// Size of one geodata block (8 × 8 cells).
    pub const BLOCK: f32 = 8.0 * CELL;
    /// Size of one geodata raster tile: 256 × 256 cells, one pixel per cell.
    pub const GEO_TILE: f32 = 256.0 * CELL;
    /// World position of the corner of region `0_0`: region `20_18` starts at `(0, 0)`.
    pub const ORIGIN: [f32; 2] = [-20.0 * REGION, -18.0 * REGION];
    /// Regions that exist on the map.
    pub const REGION_MIN: [i32; 2] = [16, 10];
    pub const REGION_MAX: [i32; 2] = [26, 26];

    /// Region `x_y` containing world point `p`.
    pub fn region_of(p: [f32; 2]) -> [i32; 2] {
        [
            ((p[0] - ORIGIN[0]) / REGION).floor() as i32,
            ((p[1] - ORIGIN[1]) / REGION).floor() as i32,
        ]
    }

    /// World rect of region `r`.
    pub fn region_rect(r: [i32; 2]) -> ([f32; 2], [f32; 2]) {
        let a = [ORIGIN[0] + r[0] as f32 * REGION, ORIGIN[1] + r[1] as f32 * REGION];
        (a, [a[0] + REGION, a[1] + REGION])
    }
}

/// One geodata cell: height and which sides can be walked through
/// (bit 3 north, 2 south, 1 west, 0 east, as in L2 NSWE).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GeoCell {
    pub height: f32,
    pub nswe: u8,
}

pub const N: u8 = 8;
pub const S: u8 = 4;
pub const W: u8 = 2;
pub const E: u8 = 1;

/// Synthetic terrain height at a world point, roughly -200 … 1200.
pub fn terrain_height(x: f32, y: f32) -> f32 {
    let rolling = 400.0 + 300.0 * (x / 2300.0).sin() * (y / 1900.0).cos()
        + 150.0 * (x / 700.0 + y / 900.0).sin()
        + 80.0 * ((x - y) / 350.0).sin()
        + 40.0 * (x / 130.0).sin() * (y / 170.0).cos();
    // plateaus with sharp edges, so the geodata gets real cliffs (walls)
    let ridge = (x / 900.0).sin() * (y / 1100.0).cos() + 0.3 * (x / 260.0 + y / 310.0).sin();
    let plateau = if ridge > 0.55 { 180.0 } else if ridge < -0.6 { -140.0 } else { 0.0 };
    rolling + plateau
}

fn hash2(a: i32, b: i32) -> u32 {
    let mut h = (a as u32).wrapping_mul(0x9E37_79B1) ^ (b as u32).wrapping_mul(0x85EB_CA77);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    h
}

/// Whether block `(bx, by)` is a synthetic building (all cells blocked).
pub fn is_building(bx: i32, by: i32) -> bool {
    hash2(bx, by) % 29 == 0
}

/// Synthetic geodata for cell `(cx, cy)` (cell = `floor(world / 16)`).
pub fn geo_cell(cx: i32, cy: i32) -> GeoCell {
    let bx = cx.div_euclid(8);
    let by = cy.div_euclid(8);
    let at = |cx: i32, cy: i32| terrain_height(cx as f32 * l2::CELL + 8.0, cy as f32 * l2::CELL + 8.0);
    let h = at(cx, cy);
    if is_building(bx, by) {
        return GeoCell { height: h + 120.0, nswe: 0 };
    }
    let step = 38.0;
    let mut nswe = 0;
    let pass = |dx: i32, dy: i32| {
        let nb = (cx + dx).div_euclid(8);
        let nby = (cy + dy).div_euclid(8);
        !is_building(nb, nby) && (at(cx + dx, cy + dy) - h).abs() < step
    };
    if pass(0, -1) {
        nswe |= N;
    }
    if pass(0, 1) {
        nswe |= S;
    }
    if pass(-1, 0) {
        nswe |= W;
    }
    if pass(1, 0) {
        nswe |= E;
    }
    GeoCell { height: h, nswe }
}

/// Terrain colour for a height: dark lowlands → light highlands.
pub fn height_color(h: f32) -> [u8; 3] {
    let t = ((h + 200.0) / 1400.0).clamp(0.0, 1.0);
    let lo = [38.0, 52.0, 44.0];
    let hi = [142.0, 136.0, 112.0];
    [
        (lo[0] + (hi[0] - lo[0]) * t) as u8,
        (lo[1] + (hi[1] - lo[1]) * t) as u8,
        (lo[2] + (hi[2] - lo[2]) * t) as u8,
    ]
}

/// Pixels of a 256 × 256 RGBA geodata raster for geo tile `tile`
/// ([`l2::GEO_TILE`] units, one pixel per cell): height-tinted cells, blocked
/// cells in red, transparent elsewhere. Drawn as a second tile layer over the
/// map, so geodata costs a few quads per frame instead of one rect per cell.
pub fn geo_pixels(tile: [i32; 2]) -> Vec<u8> {
    const N_PX: usize = 256;
    let a = [l2::ORIGIN[0] + tile[0] as f32 * l2::GEO_TILE, l2::ORIGIN[1] + tile[1] as f32 * l2::GEO_TILE];
    let mut px = Vec::with_capacity(N_PX * N_PX * 4);
    for py in 0..N_PX {
        for pxl in 0..N_PX {
            let wx = a[0] + (pxl as f32 + 0.5) * l2::CELL;
            let wy = a[1] + (py as f32 + 0.5) * l2::CELL;
            let bx = (wx / l2::BLOCK).floor() as i32;
            let by = (wy / l2::BLOCK).floor() as i32;
            if is_building(bx, by) {
                px.extend_from_slice(&[232, 97, 92, 115]);
            } else {
                let c = height_color(terrain_height(wx, wy));
                px.extend_from_slice(&[c[0], c[1], c[2], 140]);
            }
        }
    }
    px
}

/// Pixels of a synthetic 256 × 256 RGBA map image for region `tile`: one
/// pixel per block, shaded by height, with buildings and block borders.
pub fn tile_pixels(tile: [i32; 2]) -> Vec<u8> {
    const N_PX: usize = 256;
    let (a, _) = l2::region_rect(tile);
    let mut px = Vec::with_capacity(N_PX * N_PX * 4);
    for py in 0..N_PX {
        for pxl in 0..N_PX {
            let wx = a[0] + (pxl as f32 + 0.5) * l2::BLOCK;
            let wy = a[1] + (py as f32 + 0.5) * l2::BLOCK;
            let h = terrain_height(wx, wy);
            let mut c = height_color(h);
            // cheap hill shading from the x slope
            let dh = terrain_height(wx + l2::BLOCK, wy) - h;
            let shade = (1.0 - dh / 400.0).clamp(0.7, 1.3);
            for v in &mut c {
                *v = (*v as f32 * shade).clamp(0.0, 255.0) as u8;
            }
            let bx = (wx / l2::BLOCK).floor() as i32;
            let by = (wy / l2::BLOCK).floor() as i32;
            if is_building(bx, by) {
                c = [92, 84, 78];
            }
            if pxl % 16 == 0 || py % 16 == 0 {
                for v in &mut c {
                    *v = (*v as f32 * 0.85) as u8;
                }
            }
            px.extend_from_slice(&[c[0], c[1], c[2], 255]);
        }
    }
    px
}

/// Something standing on the map.
#[derive(Clone, Debug, PartialEq)]
pub struct Npc {
    pub name: String,
    pub pos: [f32; 2],
    pub hostile: bool,
    pub hp: f32,
}

/// State of the map page.
pub struct MapPage {
    pub view: MapView,
    pub tiles: TileGrid,
    /// Geodata raster layer, [`l2::GEO_TILE`] units per tile.
    pub geo: TileGrid,
    pub player: [f32; 2],
    pub heading: f32,
    pub walking: bool,
    pub show_geo: bool,
    pub show_walls: bool,
    pub show_blocks: bool,
    pub show_npcs: bool,
    /// Sight radius around the player, world units.
    pub sight: f32,
    pub npcs: Vec<Npc>,
    pub selected: Option<usize>,
    pub waypoint: Option<[f32; 2]>,
    trail: Vec<[f32; 2]>,
    t: f32,
    base: [f32; 2],
}

const PLAYER_START: [f32; 2] = [83_000.0, 148_000.0];

impl Default for MapPage {
    fn default() -> Self {
        let names = ["Wolf", "Keltir", "Orc", "Goblin", "Bandit", "Spider", "Guard", "Merchant", "Gatekeeper", "Priest"];
        let npcs = (0..28)
            .map(|i| {
                let h = hash2(i, 77);
                let ang = (h % 3600) as f32 / 3600.0 * std::f32::consts::TAU;
                let dist = 600.0 + (hash2(i, 91) % 3400) as f32;
                let hostile = i % 3 != 0;
                Npc {
                    name: format!("{} {}", names[(i as usize) % names.len()], i + 1),
                    pos: [PLAYER_START[0] + ang.cos() * dist, PLAYER_START[1] + ang.sin() * dist],
                    hostile,
                    hp: 0.3 + (hash2(i, 5) % 70) as f32 / 100.0,
                }
            })
            .collect();
        Self {
            view: MapView {
                zoom: 0.125,
                show_tile_labels: true,
                ..MapView::default()
            },
            tiles: TileGrid::new(l2::ORIGIN, [l2::REGION, l2::REGION]).with_bounds(l2::REGION_MIN, l2::REGION_MAX),
            geo: TileGrid::new(l2::ORIGIN, [l2::GEO_TILE, l2::GEO_TILE]).with_bounds(
                [l2::REGION_MIN[0] * 8, l2::REGION_MIN[1] * 8],
                [l2::REGION_MAX[0] * 8 + 7, l2::REGION_MAX[1] * 8 + 7],
            ),
            player: PLAYER_START,
            heading: 0.0,
            walking: true,
            show_geo: true,
            show_walls: true,
            show_blocks: false,
            show_npcs: true,
            sight: 2000.0,
            npcs,
            selected: None,
            waypoint: None,
            trail: Vec::new(),
            t: 0.0,
            base: PLAYER_START,
        }
    }
}

impl MapPage {
    /// Advances the simulation: the player walks a loop, NPCs drift.
    fn tick(&mut self, dt: f32) {
        if !self.walking {
            return;
        }
        self.t += dt;
        let t = self.t;
        let next = [
            self.base[0] + 3000.0 * (t * 0.25).sin(),
            self.base[1] + 2200.0 * (t * 0.5 + 0.7).sin(),
        ];
        let d = [next[0] - self.player[0], next[1] - self.player[1]];
        if d[0].abs() + d[1].abs() > 0.5 {
            self.heading = d[1].atan2(d[0]);
        }
        self.player = next;
        if self.trail.last().is_none_or(|p| (p[0] - next[0]).abs() + (p[1] - next[1]).abs() > 40.0) {
            self.trail.push(next);
            if self.trail.len() > 240 {
                self.trail.remove(0);
            }
        }
        for (i, n) in self.npcs.iter_mut().enumerate() {
            let k = i as f32;
            n.pos[0] += (t * (0.4 + k * 0.01) + k).cos() * 30.0 * dt * 4.0;
            n.pos[1] += (t * (0.35 + k * 0.013) + k * 2.0).sin() * 30.0 * dt * 4.0;
        }
    }

    fn fit_region(&mut self, view_px: [f32; 2]) {
        let (a, b) = l2::region_rect(l2::region_of(self.player));
        self.view.fit(a, b, view_px);
        self.view.follow = false;
    }

    /// Draws the toolbar, the stats line and the map filling the rest.
    pub fn draw(&mut self, ui: &Ui, kit: &Kit) {
        self.tick(ui.io().delta_time.min(0.1));

        // toolbar ------------------------------------------------------------
        {
            let _sp = ui.push_style_var(StyleVar::ItemSpacing([space::L, space::S]));
            w::switch(ui, kit, "Follow", &mut self.view.follow);
            ui.same_line();
            w::switch(ui, kit, "Walk", &mut self.walking);
            ui.same_line();
            w::switch(ui, kit, "Geodata", &mut self.show_geo);
            ui.same_line();
            w::switch(ui, kit, "Walls", &mut self.show_walls);
            ui.same_line();
            w::switch(ui, kit, "Blocks", &mut self.show_blocks);
            ui.same_line();
            w::switch(ui, kit, "Regions", &mut self.view.show_tile_grid);
            ui.same_line();
            w::switch(ui, kit, "NPCs", &mut self.show_npcs);

            let bw = [
                w::button_small_width(ui, kit, "Center"),
                w::button_small_width(ui, kit, "Fit region"),
                w::button_small_width(ui, kit, "-"),
                w::button_small_width(ui, kit, "+"),
            ];
            ui.same_line();
            imgui_kit::grid::right_align(ui, &bw);
            imgui_kit::grid::vcenter(ui, imgui_kit::tokens::size::SMALL, imgui_kit::tokens::size::CONTROL);
            if w::button_small(ui, kit, ButtonKind::Secondary, "Center") {
                self.view.follow = true;
            }
            ui.same_line();
            if w::button_small(ui, kit, ButtonKind::Secondary, "Fit region") {
                let avail = ui.content_region_avail();
                self.fit_region([avail[0], avail[1] - 40.0]);
            }
            ui.same_line();
            if w::button_small(ui, kit, ButtonKind::Secondary, "-") {
                self.view.zoom_by(0.5);
            }
            ui.same_line();
            if w::button_small(ui, kit, ButtonKind::Secondary, "+") {
                self.view.zoom_by(2.0);
            }
        }

        // stats line ---------------------------------------------------------
        {
            let r = l2::region_of(self.player);
            let (ready, loading, _) = self.tiles.counts();
            let (geo_ready, geo_loading, _) = self.geo.counts();
            let cell = [(self.player[0] / l2::CELL).floor() as i32, (self.player[1] / l2::CELL).floor() as i32];
            let g = geo_cell(cell[0], cell[1]);
            let items = [
                ("Position", format!("{:.0}, {:.0}", self.player[0], self.player[1]), String::new()),
                ("Region", format!("{}_{}", r[0], r[1]), String::new()),
                ("Cell", format!("{}, {}", cell[0], cell[1]), String::new()),
                ("Height", format!("{:.0}", g.height), String::new()),
                ("Zoom", map::zoom_label(self.view.zoom), String::new()),
                ("Tiles", format!("{ready}+{geo_ready}"), format!("ready, {} loading", loading + geo_loading)),
                ("NPCs in sight", format!("{}", self.in_sight()), String::new()),
            ];
            for (i, (cap, val, unit)) in items.iter().enumerate() {
                if i > 0 {
                    ui.same_line_with_spacing(0.0, space::L);
                }
                w::stat(ui, kit, cap, val, unit, color::FG);
            }
        }
        ui.dummy([0.0, space::XS]);

        // map ----------------------------------------------------------------
        let _f = ui.push_font(kit.fonts.mono12);
        let player = self.player;
        let heading = self.heading;
        let sight = self.sight;
        let (show_geo, show_walls, show_blocks, show_npcs) = (self.show_geo, self.show_walls, self.show_blocks, self.show_npcs);
        let npcs = &self.npcs;
        let selected = self.selected;
        let waypoint = self.waypoint;
        let trail = &self.trail;
        // Layers: the map image, then the geodata raster (cells pre-rendered
        // into textures, see geo_pixels). Only the walls stay vector: they
        // need to be 1 px at any zoom.
        let mut layers = [Layer::new(&mut self.tiles), Layer::new(&mut self.geo).visible(show_geo)];
        let resp = self.view.show_layers(ui, "##l2map", [0.0, 0.0], &mut layers, Some(player), |c| {
            // 1. walls: the blocked sides of each visible cell, once cells are readable
            let cell_px = c.px(l2::CELL);
            if show_walls && cell_px >= 8.0 {
                let (min, max) = c.world_rect();
                let (x0, x1) = ((min[0] / l2::CELL).floor() as i32, (max[0] / l2::CELL).floor() as i32);
                let (y0, y1) = ((min[1] / l2::CELL).floor() as i32, (max[1] / l2::CELL).floor() as i32);
                for cy in y0..=y1 {
                    for cx in x0..=x1 {
                        let g = geo_cell(cx, cy);
                        if g.nswe == 0b1111 {
                            continue;
                        }
                        let a = [cx as f32 * l2::CELL, cy as f32 * l2::CELL];
                        let b = [a[0] + l2::CELL, a[1] + l2::CELL];
                        if g.nswe & N == 0 {
                            c.line(a, [b[0], a[1]], color::WARN, 1.0);
                        }
                        if g.nswe & S == 0 {
                            c.line([a[0], b[1]], b, color::WARN, 1.0);
                        }
                        if g.nswe & W == 0 {
                            c.line(a, [a[0], b[1]], color::WARN, 1.0);
                        }
                        if g.nswe & E == 0 {
                            c.line([b[0], a[1]], b, color::WARN, 1.0);
                        }
                    }
                }
            }
            // 3. block grid
            if show_blocks {
                c.grid([0.0, 0.0], l2::BLOCK, 12.0, color::LINE2);
            }
            // 4. sight range
            c.circle_world_filled(player, sight, color::SEL);
            c.circle_world(player, sight, color::FG3, 1.0);
            // 5. trail and waypoint
            c.polyline(trail, color::INFO, 1.5);
            if let Some(wp) = waypoint {
                c.line(player, wp, color::ACCENT, 1.0);
                c.marker(wp, color::ACCENT, 4.0);
                c.label(ui, wp, 8.0, color::FG, "waypoint");
            }
            // 6. NPCs
            if show_npcs {
                let labels = c.px(100.0) >= 10.0;
                for (i, n) in npcs.iter().enumerate() {
                    if !c.is_visible(n.pos, n.pos) {
                        continue;
                    }
                    let col = if n.hostile { color::ERR } else { color::OK };
                    let near = dist(n.pos, player) <= sight;
                    c.marker(n.pos, if near { col } else { dim(col) }, if near { 4.0 } else { 3.0 });
                    if selected == Some(i) {
                        c.circle(n.pos, 9.0, color::ACCENT, 1.5);
                    }
                    if labels && (near || selected == Some(i)) {
                        c.label(ui, n.pos, 8.0, col, &format!("{} {:.0}%", n.name, n.hp * 100.0));
                    }
                }
            }
            // 7. player
            c.arrow(player, heading, 11.0, color::ACCENT);
            c.label(ui, player, 14.0, color::FG, "You");
        });

        // clicks: select the NPC under the cursor, otherwise set a waypoint
        if let Some(p) = resp.clicked {
            let px_per_unit = self.view.zoom;
            let hit = self
                .npcs
                .iter()
                .enumerate()
                .map(|(i, n)| (i, dist(n.pos, p) * px_per_unit))
                .filter(|(_, d)| *d <= 10.0)
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .map(|(i, _)| i);
            match hit {
                Some(i) => self.selected = Some(i),
                None => self.waypoint = Some(p),
            }
        }
    }

    fn in_sight(&self) -> usize {
        self.npcs.iter().filter(|n| dist(n.pos, self.player) <= self.sight).count()
    }
}

fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
}

fn dim(c: Rgba) -> Rgba {
    [c[0], c[1], c[2], 0.45]
}
