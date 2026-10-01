# imgui_kit

UI kit for [imgui-rs](https://github.com/imgui-rs/imgui-rs) in a dark launcher look:
dark only, JetBrains Mono, 8 px unit, 12-column grid.

The design source is the canvas **ImGui Rust UI Kit**
(https://claude.ai/artifact/M8RLaJ3kY9NAeN9jEJ7hXH). Each module maps to an artboard:

| Artboard | Module | What it holds |
|---|---|---|
| 01 Foundations | `tokens`, `theme`, `fonts` | 16 colour tokens → `StyleColor`, spacing, sizes, 6-font atlas |
| 02 Grid & layout | `grid` | 12 × 88 + 11 × 16 + 2 × 24 = 1280, pane split 5 / 7, form row `96 \| stretch \| 64`, alignment helpers, all `Style` vars in `theme::apply` |
| 03 Controls | `widgets::{buttons, controls, sliders, inputs}` | button variants, checkbox, radio, switch, 4 px slider, progress, inputs, path group |
| 04 Navigation & overlays | `widgets::{bars}` | title bar with sliding tabs, status strip, panels, cards, banners, log panels |
| 05 Data & feedback | `widgets::{text, status}` | captions, log line, tags, stat, status dot |
| — | `kit` | `Kit { fonts, anim }`: the context every widget takes as `&Kit` |
| — | `anim` | toggle, tab and page animations with per-`Kit` settings |
| — | `map` | tiled map view: camera, tile layers loaded block-wise, world-unit canvas |
| 06 Reference screen | `examples/launch/` | the Launch screen, the widget gallery and the L2 map page |

The library ships no demo code: `cargo run --example launch` builds the demo
from `examples/launch/{main,demo,gallery,map_demo}.rs`.

## Usage

```rust
use imgui_kit::{fonts::{self, FontFiles}, theme, widgets as w, ButtonKind, Kit};

let mut ctx = imgui::Context::create();
theme::apply_to(&mut ctx);
let fonts = fonts::load(&mut ctx, FontFiles {
    regular: include_bytes!("assets/JetBrainsMono-Regular.ttf"),
    bold: include_bytes!("assets/JetBrainsMono-Bold.ttf"),
    semibold: Some(include_bytes!("assets/JetBrainsMono-SemiBold.ttf")),
}, hidpi_scale);
let kit = Kit::new(fonts);

// per frame, with any imgui-rs backend (glow + winit, wgpu, …):
let ui = ctx.new_frame();
w::checkbox(ui, &kit, "Auto-restart", None, &mut auto_restart);
if w::button(ui, &kit, ButtonKind::Primary, "Launch") { /* … */ }
```

Every widget takes `(ui, &kit, …)`; the longer ones also come as builders,
`widgets::Input::new("login").hint("Login").show(ui, &kit, &mut s)` and
`widgets::Slider::new("Windows", 1.0, 12.0).step(1.0).suffix("/ 12").show(..)`. `Kit` owns the font atlas and the
animation store, so nothing in the crate is global: two contexts or a test
never share state.

JetBrains Mono (Regular, Bold, SemiBold; SIL OFL, see `assets/OFL.txt`) ships in
`assets/` for the demo. In your own app, embed or load the TTFs yourself and point
`FontFiles` at them.

## Grid in one formula

```
col   = (avail − gutter · (n − 1)) / n        // avail = window width − 2 · 24
span  = col · s + gutter · (s − 1)
left  = span(5) = 504 px   right = span(7) = 712 px   at 1280 wide
```

`Grid::panes` opens the two child windows, `Grid::row` lays out spans,
`grid::form_row` the three-column table, `grid::Row` places items of mixed
heights on one line, `grid::right_align` / `center` / `vcenter` place groups, `grid::section_gap` draws the
24 · separator · 24 rhythm, `grid::push_to_bottom` pins the CTA.

## Building

```
cargo build            # needs a C++ compiler for imgui-sys
cargo test             # grid maths, animation tweens, map camera and tile grid
```

## Running the demo

```
cargo run --example launch
```

The host (`examples/launch/main.rs`, winit + glutin + imgui-glow-renderer) is
a reference for what an application has to do:

- **Render on demand.** Frames are scheduled only after input, while an
  animation runs (`kit.anim.animating(ui)`, `LaunchScreen::is_animating()`),
  while a text field has focus and while map tiles are loading; at 15 fps
  when the window is not focused, never when minimized. Idle the process
  sleeps in the event loop at 0 % CPU.
- **Events.** `LaunchScreen::draw` returns every event of the frame as a
  `Vec<LaunchEvent>`; the host matches on them.
- **Tiles.** A worker thread decodes pending map tiles (`tiles.rs`); the main
  thread uploads a few per frame and frees textures of tiles that left the
  view. Assets are found next to the executable or in the checkout
  (`assets.rs`).
- **Precise pacing.** winit's `WaitUntil` is a 15.6 ms timer on Windows, so
  the host sleeps itself (after `timeBeginPeriod(1)`) for real 60 fps while
  something animates.
- **DPI.** A scale-factor change rebuilds the font atlas and the renderer.
- **Settings.** Tab, options, map camera and animation settings are saved to
  `launcher.settings` next to the executable on exit and restored on start
  (`settings.rs`, plain `key=value`).
- **Clipboard.** `arboard` is wired in as imgui's clipboard backend: Ctrl+C /
  Ctrl+V in text fields and the log's Copy button work; Save writes
  `launcher.log` next to the executable.
- **Hotkeys.** Ctrl+1…8 pick a tab, Ctrl+Tab / Ctrl+Shift+Tab cycle,
  Ctrl+Enter launches, Ctrl+Backspace stops everything.
- The OS window is undecorated: the kit's title bar is the drag handle and
  its `– □ ×` controls minimize / maximize / close.

The tabs other than Launch show the widget gallery: cards, tags, status
dots, a table, progress bars, inputs, radios, switches, checkboxes, the
disabled state, stats, sliders, every button variant, banners, grid and form
rows, text styles, panel header and a clipped, bounded log. `IMGUI_KIT_FPS=1`
prints UI build time, frames and draw-list size once a second.

## Animations

Checkbox, radio and switch toggles, the active-tab highlight and the page
change are animated (140 / 50 / 220 ms). The *Animation* section at the top
of the Settings tab turns each one off and scales the durations; in code use
`kit.anim.set(anim::Settings { .. })` or `kit.anim.set_enabled(false)`.
Custom-drawn widgets honour `style.alpha`, so `widgets::disabled` and page
fades dim them like native items.

## Map view

`map::MapView` draws a map from three parts you control separately:

- **Camera.** `center` in world units, `zoom` in px per unit. Left-drag pans,
  the wheel zooms around the cursor, `follow` keeps the centre on a target
  (panning by hand turns it off), `fit` frames a world rect.
- **Tile layers.** `show` takes one `TileSource`, `show_layers` a list of
  `Layer { tiles, alpha, visible }` drawn in order: the map image first, then
  overlays such as a rasterised geodata grid or fog of war. A source hands
  out the background block by block, like L2 map regions: the view asks only
  for visible tiles and draws a placeholder for one that is not ready.
  `TileGrid` is the ready-made source: each frame call `take_pending_limit(n)`,
  decode and upload any way you like (synchronously or on a thread), then
  `set(tile, texture_id)`; `evict_unseen(max_age)` returns textures of tiles
  that scrolled away so you can free them. The crate never reads files or
  touches the GPU.
- **Canvas.** The overlay closure gets world → screen conversion plus lines,
  rects, circles (pixel or world radius), text, labels, markers, arrows,
  images, a culled `cells(..)` filler and `grid(..)`.

The Map tab (`examples/launch/map_demo.rs`) is an L2-style world: regions of
32768 units named `x_y`, blocks of 8 × 8 cells of 16 units, synthetic
`geo_cell(cx, cy)` (height + NSWE). Geodata is pre-rendered into 256 × 256
tiles of one pixel per cell (`geo_pixels`) and drawn as a second layer, so it
costs a few quads at any zoom; only the walls are vector lines. A walking
player with sight range and trail, NPCs with labels, click to select an NPC
or set a waypoint. Drop real region images into `assets/map/22_22.png` and
the example picks them up; replace `geo_cell` / `geo_pixels` with your parser.
