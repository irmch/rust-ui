//! Widgets of artboards 03–05 that ImGui does not draw the way the kit wants
//! (filled checkboxes, 4 px slider tracks, captions, tags, stats, bars) plus
//! thin wrappers that apply the right font and colours to native ones.
//!
//! One submodule per artboard group; everything is re-exported here, so
//! `widgets::checkbox` keeps working.

mod bars;
mod buttons;
mod controls;
mod inputs;
mod sliders;
mod status;
mod text;

pub use bars::*;
pub use buttons::*;
pub use controls::*;
pub use inputs::*;
pub use sliders::*;
pub use status::*;
pub use text::*;
