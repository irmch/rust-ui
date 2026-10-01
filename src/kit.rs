//! The kit context: everything widgets need besides the `Ui`.
//!
//! One `Kit` per imgui context, created after the fonts are loaded and
//! passed to every widget as `&Kit`. Owning the animation store and the
//! per-widget UI state here (rather than in a global) keeps two windows or
//! a test from sharing state and gives future per-app settings a home
//! without touching widget signatures.

use std::cell::RefCell;
use std::collections::HashMap;

use imgui::Ui;

use crate::anim::{self, Anim};
use crate::fonts::Fonts;

pub struct Kit {
    /// The six-font JetBrains Mono atlas.
    pub fonts: Fonts,
    /// Tween store and animation settings.
    pub anim: Anim,
    /// Which accordion is open, which tab is active: state widgets keep
    /// between frames, keyed by the imgui ID of the widget.
    pub state: UiState,
}

impl Kit {
    pub fn new(fonts: Fonts) -> Self {
        Self {
            fonts,
            anim: Anim::new(),
            state: UiState::default(),
        }
    }
}

/// Per-widget state keyed by imgui ID (so the same title in two windows
/// does not collide). Interior mutability: widgets take `&Kit`.
#[derive(Default)]
pub struct UiState {
    open: RefCell<HashMap<u32, bool>>,
    tab: RefCell<HashMap<u32, usize>>,
    /// Keyboard navigation of the combo list that is open right now.
    pub(crate) combo: RefCell<ComboNav>,
}

/// Which row of the open combo list the keyboard points at. One combo is
/// open at a time, so a single record is enough; `key` tells whose it is.
#[derive(Default)]
pub(crate) struct ComboNav {
    pub key: u32,
    /// Highlighted row, `-1` = none yet (the selected row takes it).
    pub index: i32,
    /// Rows drawn this frame so far; reset by `combo_begin`.
    pub count: usize,
    /// Rows drawn last frame, to clamp `index`.
    pub last_count: usize,
    /// The highlight moved this frame: scroll it into view.
    pub moved: bool,
    /// Enter was pressed this frame: the highlighted row is picked.
    pub enter: bool,
}

impl UiState {
    /// Open flag of `name` under the current ID stack; `default` on first sight.
    pub fn is_open(&self, ui: &Ui, name: &str, default: bool) -> bool {
        let k = anim::key(ui, name);
        *self.open.borrow_mut().entry(k).or_insert(default)
    }

    pub fn set_open(&self, ui: &Ui, name: &str, open: bool) {
        let k = anim::key(ui, name);
        self.open.borrow_mut().insert(k, open);
    }

    /// Active tab index of `name` under the current ID stack, 0 on first sight.
    pub fn active_tab(&self, ui: &Ui, name: &str) -> usize {
        let k = anim::key(ui, name);
        self.tab.borrow().get(&k).copied().unwrap_or(0)
    }

    pub fn set_active_tab(&self, ui: &Ui, name: &str, idx: usize) {
        let k = anim::key(ui, name);
        self.tab.borrow_mut().insert(k, idx);
    }
}
