//! Small tween store for the kit's animations.
//!
//! Every animated value is keyed by an imgui ID (so it follows the ID stack
//! like any widget state) and eased toward its target a bit each frame using
//! `io.delta_time`. Nothing is allocated per frame; values that stop being
//! queried are dropped after a few seconds.
//!
//! What animates and the switch for each is in [`Settings`]:
//! - `controls`: checkbox, radio and switch toggles (fill, mark, knob);
//! - `tabs`: the active-tab highlight of the title bar slides between tabs;
//! - `pages`: the page under the title bar fades and slides in on tab change.
//!
//! ```no_run
//! imgui_kit::anim::set(imgui_kit::anim::Settings { pages: false, ..Default::default() });
//! ```

use std::cell::RefCell;
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

struct State {
    settings: Settings,
    values: HashMap<u32, Entry>,
    sweep_frame: i32,
    /// Last frame in which some value actually moved.
    moved_frame: i32,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State {
        settings: Settings::default(),
        values: HashMap::new(),
        sweep_frame: 0,
        moved_frame: -1,
    });
}

/// Current settings.
pub fn settings() -> Settings {
    STATE.with(|s| s.borrow().settings)
}

/// Replaces the settings.
pub fn set(settings: Settings) {
    STATE.with(|s| s.borrow_mut().settings = settings);
}

/// Turns every animation on or off at once.
pub fn set_enabled(on: bool) {
    STATE.with(|s| {
        let st = &mut s.borrow_mut().settings;
        st.controls = on;
        st.tabs = on;
        st.pages = on;
    });
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

fn step(ui: &Ui, key: u32, enabled: bool, advance: impl FnOnce(f32, f32) -> f32, target: f32) -> f32 {
    let dt = ui.io().delta_time.min(0.1);
    let frame = ui.frame_count();
    STATE.with(|s| {
        let mut st = s.borrow_mut();
        if !enabled {
            st.values.remove(&key);
            return target;
        }
        if frame - st.sweep_frame > 300 {
            st.sweep_frame = frame;
            st.values.retain(|_, e| frame - e.last_frame < 300);
        }
        let st = &mut *st;
        let e = st.values.entry(key).or_insert(Entry { value: target, last_frame: frame });
        e.last_frame = frame;
        let next = advance(e.value, dt);
        if next != e.value {
            e.value = next;
            st.moved_frame = frame;
        }
        e.value
    })
}

/// Whether any animation moved during this frame (or the previous one, so a
/// value that reached its target still gets one frame drawn at rest). Hosts
/// that render on demand use it to decide whether to schedule another frame.
pub fn animating(ui: &Ui) -> bool {
    let frame = ui.frame_count();
    STATE.with(|s| frame - s.borrow().moved_frame <= 1)
}

/// Linear progress toward `on` over `duration` seconds (scaled by
/// [`Settings::scale`]), returned eased with [`ease_out`]. A value seen for
/// the first time starts at its target, so nothing animates on first draw.
pub fn toggle(ui: &Ui, key: u32, on: bool, duration: f32) -> f32 {
    let st = settings();
    let target = if on { 1.0 } else { 0.0 };
    let d = (duration * st.scale).max(1e-3);
    let v = step(ui, key, st.controls, |v, dt| {
        let s = dt / d;
        // Clamp to the target itself: stepping past it and clamping to
        // 0 / 1 would overshoot by one step every other frame and jitter.
        if v < target {
            (v + s).min(target)
        } else if v > target {
            (v - s).max(target)
        } else {
            v
        }
    }, target);
    ease_out(v)
}

/// Exponential approach of a float toward `target` with time constant
/// `tau` seconds (scaled by [`Settings::scale`]). `enabled = false` snaps.
pub fn approach(ui: &Ui, key: u32, target: f32, tau: f32, enabled: bool) -> f32 {
    let tau = (tau * settings().scale).max(1e-3);
    step(ui, key, enabled, |v, dt| {
        let k = 1.0 - (-dt / tau).exp();
        let n = v + (target - v) * k;
        if (target - n).abs() < 0.05 { target } else { n }
    }, target)
}
