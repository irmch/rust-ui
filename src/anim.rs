//! Small tween store for the kit's animations, owned by [`crate::Kit`].
//!
//! Every animated value is keyed by an imgui ID (so it follows the ID stack
//! like any widget state) and eased toward its target a bit each frame using
//! `io.delta_time`. Nothing is allocated per frame; values that stop being
//! queried are dropped after a few seconds. The store lives in the
//! application's `Kit`, not in a global, so two contexts or a test never
//! share state.
//!
//! What animates and the switch for each is in [`Settings`]:
//! - `controls`: checkbox, radio and switch toggles (fill, mark, knob);
//! - `tabs`: the active-tab highlight of the title bar slides between tabs;
//! - `pages`: the page under the title bar fades and slides in on tab change.
//!
//! ```no_run
//! # let fonts: imgui_kit::Fonts = unimplemented!();
//! let kit = imgui_kit::Kit::new(fonts);
//! kit.anim.set(imgui_kit::anim::Settings { pages: false, ..Default::default() });
//! ```

use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use imgui::Ui;

/// Which animations run, and how long they take relative to the defaults.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    /// Checkbox / radio / switch toggles.
    pub controls: bool,
    /// Sliding active-tab highlight in the title bar.
    pub tabs: bool,
    /// Fade + slide of the page on tab change.
    pub pages: bool,
    /// Duration multiplier: `1.0` = defaults below, `2.0` = twice as slow.
    pub scale: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            controls: true,
            tabs: true,
            pages: true,
            scale: 1.0,
        }
    }
}

/// Default duration of a control toggle, seconds.
pub const CONTROL: f32 = 0.14;
/// Default time constant of the tab highlight slide, seconds.
pub const TABS: f32 = 0.05;
/// Default duration of a page transition, seconds.
pub const PAGE: f32 = 0.22;

struct Entry {
    value: f32,
    last_frame: i32,
}

#[derive(Default)]
struct State {
    values: HashMap<u32, Entry>,
    sweep_frame: i32,
    /// Last frame in which some value actually moved.
    moved_frame: i32,
}

/// The tween store: settings plus every in-flight value. Interior
/// mutability so widgets can animate through a shared `&Kit`.
pub struct Anim {
    settings: Cell<Settings>,
    state: RefCell<State>,
}

impl Default for Anim {
    fn default() -> Self {
        Self::new()
    }
}

impl Anim {
    pub fn new() -> Self {
        Self {
            settings: Cell::new(Settings::default()),
            state: RefCell::new(State {
                moved_frame: -1,
                ..State::default()
            }),
        }
    }

    /// Current settings.
    pub fn settings(&self) -> Settings {
        self.settings.get()
    }

    /// Replaces the settings.
    pub fn set(&self, settings: Settings) {
        self.settings.set(settings);
    }

    /// Turns every animation on or off at once.
    pub fn set_enabled(&self, on: bool) {
        let mut st = self.settings.get();
        st.controls = on;
        st.tabs = on;
        st.pages = on;
        self.settings.set(st);
    }

    /// Whether any animation moved during this frame (or the previous one,
    /// so a value that reached its target still gets one frame drawn at
    /// rest). Hosts that render on demand use it to schedule frames.
    pub fn animating(&self, ui: &Ui) -> bool {
        ui.frame_count() - self.state.borrow().moved_frame <= 1
    }

    /// Linear progress toward `on` over `duration` seconds (scaled by
    /// [`Settings::scale`]), returned eased with [`ease_out`]. A value seen
    /// for the first time starts at its target, so nothing animates on first
    /// draw.
    pub fn toggle(&self, ui: &Ui, key: u32, on: bool, duration: f32) -> f32 {
        let st = self.settings();
        let target = if on { 1.0 } else { 0.0 };
        let d = (duration * st.scale).max(1e-3);
        let dt = ui.io().delta_time;
        let v = self.state.borrow_mut().step(
            key,
            st.controls,
            dt,
            ui.frame_count(),
            target,
            |v, dt| toggle_step(v, target, dt / d),
        );
        ease_out(v)
    }

    /// Exponential approach of a float toward `target` with time constant
    /// `tau` seconds (scaled by [`Settings::scale`]). `enabled = false` snaps.
    pub fn approach(&self, ui: &Ui, key: u32, target: f32, tau: f32, enabled: bool) -> f32 {
        let tau = (tau * self.settings().scale).max(1e-3);
        let dt = ui.io().delta_time;
        self.state
            .borrow_mut()
            .step(key, enabled, dt, ui.frame_count(), target, |v, dt| {
                approach_step(v, target, dt, tau)
            })
    }
}

/// One linear step of size `s` toward `target`, never past it (stepping past
/// and clamping to 0 / 1 would overshoot by one step every other frame).
fn toggle_step(v: f32, target: f32, s: f32) -> f32 {
    if v < target {
        (v + s).min(target)
    } else if v > target {
        (v - s).max(target)
    } else {
        v
    }
}

/// One exponential step toward `target`; snaps when within 0.05.
fn approach_step(v: f32, target: f32, dt: f32, tau: f32) -> f32 {
    let k = 1.0 - (-dt / tau).exp();
    let n = v + (target - v) * k;
    if (target - n).abs() < 0.05 { target } else { n }
}

impl State {
    fn step(
        &mut self,
        key: u32,
        enabled: bool,
        dt: f32,
        frame: i32,
        target: f32,
        advance: impl FnOnce(f32, f32) -> f32,
    ) -> f32 {
        let dt = dt.min(0.1);
        if !enabled {
            self.values.remove(&key);
            return target;
        }
        if frame - self.sweep_frame > 300 {
            self.sweep_frame = frame;
            self.values.retain(|_, e| frame - e.last_frame < 300);
        }
        let e = self.values.entry(key).or_insert(Entry {
            value: target,
            last_frame: frame,
        });
        e.last_frame = frame;
        let next = advance(e.value, dt);
        if next != e.value {
            e.value = next;
            self.moved_frame = frame;
        }
        e.value
    }
}

/// ID of `name` under the current imgui ID stack.
pub fn key(ui: &Ui, name: &str) -> u32 {
    let _ = ui;
    let s = name.as_bytes();
    unsafe {
        let p = s.as_ptr() as *const std::os::raw::c_char;
        imgui::sys::igGetID_StrStr(p, p.add(s.len()))
    }
}

/// Ease-out cubic on `t` in `0..=1`.
pub fn ease_out(t: f32) -> f32 {
    let u = 1.0 - t.clamp(0.0, 1.0);
    1.0 - u * u * u
}

/// Smoothstep on `t` in `0..=1`.
pub fn ease_in_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Linear interpolation of two colours.
pub fn mix(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    let t = t.clamp(0.0, 1.0);
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
        a[3] + (b[3] - a[3]) * t,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_settles_exactly_on_target_and_stays() {
        // the checkbox jitter bug: at the target the value must not move
        let mut st = State::default();
        let mut v = 0.0;
        for frame in 1..200 {
            v = st.step(7, true, 1.0 / 60.0, frame, 1.0, |v, dt| {
                toggle_step(v, 1.0, dt / 0.14)
            });
        }
        assert_eq!(v, 1.0);
        let moved_at = st.moved_frame;
        for frame in 200..260 {
            assert_eq!(
                st.step(7, true, 1.0 / 60.0, frame, 1.0, |v, dt| toggle_step(
                    v,
                    1.0,
                    dt / 0.14
                )),
                1.0
            );
        }
        assert_eq!(
            st.moved_frame, moved_at,
            "value kept 'moving' after reaching its target"
        );
    }

    #[test]
    fn toggle_takes_about_duration_seconds() {
        let mut st = State::default();
        // seen off first (a value seen for the first time starts at its target)
        st.step(1, true, 0.01, 0, 0.0, |v, _| v);
        let mut frames = 0;
        let mut v = 0.0;
        while v < 1.0 && frames < 1000 {
            frames += 1;
            v = st.step(1, true, 0.01, frames, 1.0, |v, dt| {
                toggle_step(v, 1.0, dt / 0.14)
            });
        }
        assert!(
            (13..=15).contains(&frames),
            "0.14 s at 100 fps, got {frames} frames"
        );
    }

    #[test]
    fn first_sight_starts_at_target_and_disabled_snaps() {
        let mut st = State::default();
        assert_eq!(st.step(1, true, 0.016, 1, 1.0, |v, _| v), 1.0);
        assert_eq!(st.step(2, false, 0.016, 1, 0.3, |_, _| 0.9), 0.3);
        assert!(!st.values.contains_key(&2));
    }

    #[test]
    fn approach_reaches_and_snaps() {
        let mut v = 0.0;
        for _ in 0..100 {
            v = approach_step(v, 100.0, 0.016, 0.05);
        }
        assert_eq!(v, 100.0);
    }

    #[test]
    fn stale_entries_are_swept() {
        let mut st = State::default();
        st.step(1, true, 0.016, 1, 0.0, |v, _| v);
        st.step(2, true, 0.016, 700, 0.0, |v, _| v);
        assert!(!st.values.contains_key(&1));
        assert!(st.values.contains_key(&2));
    }
}
