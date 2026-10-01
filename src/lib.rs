//! # imgui_kit
//!
//! Design kit for `imgui-rs` matching the "ImGui Rust UI Kit" canvas:
//! dark monospace look, 8 px unit, 12-column grid.
//!
//! ```no_run
//! use imgui_kit::{fonts::{self, FontFiles}, theme, widgets as w, ButtonKind};
//!
//! let mut ctx = imgui::Context::create();
//! theme::apply_to(&mut ctx);
//! // Ship the TTFs with your app, e.g. `include_bytes!("../assets/JetBrainsMono-Regular.ttf")`.
//! let regular = std::fs::read("assets/JetBrainsMono-Regular.ttf").unwrap();
//! let bold = std::fs::read("assets/JetBrainsMono-Bold.ttf").unwrap();
//! let fonts = fonts::load(&mut ctx, FontFiles {
//!     regular: &regular,
//!     bold: &bold,
//!     semibold: None,
//! }, 1.0);
//! let mut auto_restart = true;
//! // every frame:
//! // let ui = ctx.new_frame();
//! // w::checkbox(ui, &fonts, "Auto-restart", None, &mut auto_restart);
//! // if w::button(ui, &fonts, ButtonKind::Primary, "Launch") { /* … */ }
//! ```
//!
//! The reference Launch screen, the widget gallery and the Map tab live in
//! `examples/launch/` (`cargo run --example launch`), not in the library.
//!
//! Modules:
//! - [`tokens`]: colours, spacing, sizes, font sizes (artboard 01).
//! - [`theme`]: `apply` writes every `Style` var and colour; button variants.
//! - [`fonts`]: the six-font JetBrains Mono atlas.
//! - [`grid`]: the 12-column grid, pane split, form rows, alignment helpers (artboard 02).
//! - [`widgets`]: checkboxes, switches, sliders, tags, bars, panels (artboards 03–05).
//! - [`anim`]: toggle / tab / page animations and the switches to turn them off.
//! - [`map`]: tiled map view with pan / zoom, block-wise tile loading and a world-unit canvas.

pub mod anim;
pub mod fonts;
pub mod grid;
pub mod map;
pub mod theme;
pub mod tokens;
pub mod widgets;

pub use fonts::Fonts;
pub use grid::Grid;
pub use theme::ButtonKind;
