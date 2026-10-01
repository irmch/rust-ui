# imgui_kit

UI kit for [imgui-rs](https://github.com/imgui-rs/imgui-rs) in the PoEMulti look:
dark only, JetBrains Mono, 8 px unit, 12-column grid.

The design source is the canvas **ImGui Rust UI Kit**
(https://claude.ai/artifact/M8RLaJ3kY9NAeN9jEJ7hXH). Each module maps to an artboard:

| Artboard | Module | What it holds |
|---|---|---|
| 01 Foundations | `tokens`, `theme`, `fonts` | 16 colour tokens → `StyleColor`, spacing, sizes, 6-font atlas |
| 02 Grid & layout | `grid` | 12 × 88 + 11 × 16 + 2 × 24 = 1280, pane split 5 / 7, form row `96 \| stretch \| 64`, alignment helpers, all `Style` vars in `theme::apply` |
| 03 Controls | `widgets` | button variants, checkbox, radio, switch, 4 px slider, progress, inputs, path group |
| 04 Navigation & overlays | `widgets` | title bar with centred tabs, status strip, tags, banner, panels, cards |
| 05 Data & feedback | `widgets` | log line, stat, status dot |
| 06 Reference screen | `demo::LaunchScreen` | the Launch screen built from the above |
| all of the above | `gallery::Gallery` | every widget in every state on the Accounts … Settings tabs of the demo |
| — | `anim` | toggle, tab and page animations; `anim::set` / `anim::set_enabled` switch them off |
| — | `map`, `map_demo` | tiled map view: pan / zoom camera, block-wise tile loading, world-unit canvas for geodata and entities; the Map tab |

## Usage

```rust
use imgui_kit::{demo::LaunchScreen, fonts::{self, FontFiles}, theme};

let mut ctx = imgui::Context::create();
theme::apply_to(&mut ctx);
let fonts = fonts::load(&mut ctx, FontFiles {
    regular: include_bytes!("assets/JetBrainsMono-Regular.ttf"),
    bold: include_bytes!("assets/JetBrainsMono-Bold.ttf"),
    semibold: Some(include_bytes!("assets/JetBrainsMono-SemiBold.ttf")),
}, hidpi_scale);

let mut screen = LaunchScreen::default();
// per frame, with any imgui-rs backend (glow + winit, wgpu, …):
let ui = ctx.new_frame();
match screen.draw(ui, &fonts, [w, h]) {
    imgui_kit::demo::LaunchEvent::Launch => { /* … */ }
    _ => {}
}
```

JetBrains Mono (Regular, Bold, SemiBold; SIL OFL, see `assets/OFL.txt`) ships in
`assets/` for the demo. In your own app, embed or load the TTFs yourself and point
`FontFiles` at them.

## Grid in one formula

```
col   = (avail − gutter · (n − 1)) / n        // avail = window width − 2 · 24
span  = col · s + gutter · (s − 1)
left  = span(5) = 504 px   right = span(7) = 712 px   at 1280 wide
```

`Grid::panes` opens the two child windows, `grid::form_row` the three-column table,
`grid::right_align` / `grid::center` / `grid::vcenter` place groups, `grid::section_gap`
draws the 24 · separator · 24 rhythm, `grid::push_to_bottom` pins the CTA.

## Building

```
cargo build            # needs a C++ compiler for imgui-sys
cargo test
```

## Running the demo

The crate is a library, so there is nothing to `cargo run` by itself. The
`examples/launch.rs` host opens the reference Launch screen in a real window
(winit + glutin + imgui-glow-renderer, pulled in as dev-dependencies only):

```
cargo run --example launch
```

The OS window is undecorated; the kit's own title bar is the drag handle and
its `– □ ×` buttons minimize / maximize / close the window. Screen events
(`Launch`, `StopAll`, `Browse`, …) are printed to stdout.

The other tabs (Accounts, Instances, Proxies, Resources, Tools, Settings) show
the widget gallery: cards, tags, status dots, a table, progress bars, inputs,
radios, switches, checkboxes, the disabled state, stats, sliders, every button
variant, banners, grid and form rows, text styles, panel header and panel.

Checkbox, radio and switch toggles, the active-tab highlight and the page
change are animated (140 / 50 / 220 ms). The *Animation* section at the top of
the Settings tab turns each one off and scales the durations; in code use
`anim::set(anim::Settings { .. })` or `anim::set_enabled(false)`.

## Map view

`map::MapView` draws a map in three layers you control separately:

- **Camera.** `center` in world units, `zoom` in px per unit. Left-drag pans,
  the wheel zooms around the cursor, `follow` keeps the centre on a target
  (panning by hand turns it off). `fit` frames a world rect.
- **Tiles.** A `TileSource` hands out the background block by block, like
  L2 map regions: the view asks only for visible tiles and draws a
  placeholder for one that is not ready yet. `TileGrid` is the ready-made
  source: each frame call `take_pending()`, decode and upload the images any
  way you like, then `set(tile, texture_id)`. The crate itself never reads
  files or touches the GPU.
- **Canvas.** The overlay closure gets world → screen conversion plus lines,
  rects, circles (pixel or world radius), text, labels, markers, arrows, a
  culled `cells(..)` filler for geodata with level-of-detail and `grid(..)`.

The Map tab (`map_demo`) is an L2-style world: regions of 32768 units named
`x_y`, blocks of 8 × 8 cells of 16 units, synthetic `geo_cell(cx, cy)`
(height + NSWE) drawn as cells and walls, a walking player with sight range
and trail, NPCs with labels, click to select an NPC or set a waypoint. Drop
real region images into `assets/map/22_22.png` and the example picks them up
instead of the generated ones; replace `geo_cell` with your parser.
