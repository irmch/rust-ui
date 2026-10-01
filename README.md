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

Fonts are not bundled: download JetBrains Mono and point `FontFiles` at the TTFs.

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
