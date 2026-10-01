//! The kit context: everything widgets need besides the `Ui`.
//!
//! One `Kit` per imgui context, created after the fonts are loaded and
//! passed to every widget as `&Kit`. Owning the animation store here (rather
//! than in a global) keeps two windows or a test from sharing state and gives
//! future per-app settings a home without touching widget signatures.

use crate::anim::Anim;
use crate::fonts::Fonts;

pub struct Kit {
    /// The six-font JetBrains Mono atlas.
    pub fonts: Fonts,
    /// Tween store and animation settings.
    pub anim: Anim,
}

impl Kit {
    pub fn new(fonts: Fonts) -> Self {
        Self {
            fonts,
            anim: Anim::new(),
        }
    }
}
