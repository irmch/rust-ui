//! # imgui_kit
//!
//! Design kit for `imgui-rs` matching the "ImGui Rust UI Kit" canvas:
//! dark monospace look, 8 px unit, 12-column grid.
//!
//! ```no_run
//! use imgui_kit::{fonts::{self, FontFiles}, theme, demo::LaunchScreen};
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
//! let mut screen = LaunchScreen::default();
//! // every frame:
//! // let ui = ctx.new_frame();
//! // let event = screen.draw(ui, &fonts, [1280.0, 800.0]);
//! ```
//!
//! Modules:
//! - [`tokens`]: colours, spacing, sizes, font sizes (artboard 01).
//! - [`theme`]: `apply` writes every `Style` var and colour; button variants.
//! - [`fonts`]: the six-font JetBrains Mono atlas.
//! - [`grid`]: the 12-column grid, pane split, form rows, alignment helpers (artboard 02).
//! - [`widgets`]: checkboxes, switches, sliders, tags, bars, panels (artboards 03–05).
//! - [`demo`]: the reference Launch screen (artboard 06).
//! - [`gallery`]: every widget in every state, shown on the other tabs of the demo.
//! - [`anim`]: toggle / tab / page animations and the switches to turn them off.

pub mod anim;
pub mod demo;
pub mod fonts;
pub mod gallery;
pub mod grid;
pub mod theme;
pub mod tokens;
pub mod widgets;

pub use fonts::Fonts;
pub use grid::Grid;
pub use theme::ButtonKind;
