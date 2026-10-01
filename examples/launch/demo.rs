//! The reference "Launch" screen (artboard 06) assembled from the kit.
//! Draw it with [`LaunchScreen::draw`] inside your frame loop.

use std::collections::VecDeque;

use imgui::{Condition, Key, StyleVar, Ui, WindowFlags};

use crate::gallery::Gallery;
use crate::settings::Settings;
use imgui_kit::Kit;
use imgui_kit::anim;
use imgui_kit::grid::{self, Grid, Pane};
use imgui_kit::theme::ButtonKind;
use imgui_kit::tokens::{color, size, space};
use imgui_kit::widgets::{self as w, StatItem, TitleBarAction};

/// State of the demo screen.
pub struct LaunchScreen {
    pub tab: usize,
    pub game_path: String,
    pub path_ok: bool,
    pub auto_restart: bool,
    pub safe_mode: bool,
    pub real_gpu: bool,
    pub gpu_dialog: bool,
    pub underflow_fix: bool,
    pub spoof_hash: bool,
    pub windows: f32,
    pub stagger_ms: f32,
    /// Status log, newest last, at most [`LOG_CAP`] lines.
    pub log: VecDeque<(f32, String)>,
    pub status: &'static str,
    /// State of the widget gallery on the other tabs.
    pub gallery: Gallery,
    /// Tab currently shown by the page transition, its progress and direction.
    page_tab: usize,
    page_t: f32,
    page_dir: f32,
}

impl Default for LaunchScreen {
    fn default() -> Self {
        Self {
            tab: 0,
            game_path: r"C:\Games\MyGame\Game.exe".into(),
            path_ok: true,
            auto_restart: true,
            safe_mode: false,
            real_gpu: false,
            gpu_dialog: false,
            underflow_fix: false,
            spoof_hash: true,
            windows: 6.0,
            stagger_ms: 350.0,
            log: VecDeque::from(vec![
                (0.000, "Game path verified".into()),
                (0.084, "6 accounts ready".into()),
                (0.168, "Per-window proxies assigned".into()),
                (0.252, "Auto-restart enabled".into()),
                (0.336, "Ready to launch".into()),
            ]),
            status: "Ready",
            gallery: Gallery::default(),
            page_tab: 0,
            page_t: 1.0,
            page_dir: 1.0,
        }
    }
}

/// Lines kept in the status log; older ones are dropped.
pub const LOG_CAP: usize = 2000;

/// Events the screen reports back to the application. A frame can produce
/// several (a title-bar action and a button on the same frame), so
/// [`LaunchScreen::draw`] returns all of them in order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LaunchEvent {
    Launch,
    StopAll,
    Browse,
    OpenFolder,
    CopyLog,
    SaveLog,
    Window(TitleBarAction),
}

const TABS: [&str; 8] = [
    "Launch",
    "Accounts",
    "Instances",
    "Proxies",
    "Resources",
    "Tools",
    "Settings",
    "Map",
];

impl LaunchScreen {
    /// Draws the screen as a borderless full-display window and returns the
    /// events it produced this frame (usually none).
    pub fn draw(&mut self, ui: &Ui, kit: &Kit, display_size: [f32; 2]) -> Vec<LaunchEvent> {
        let mut events = Vec::new();
        self.hotkeys(ui, &mut events);
        let _pad = ui.push_style_var(StyleVar::WindowPadding([0.0, 0.0]));
        let _rounding = ui.push_style_var(StyleVar::WindowRounding(0.0));
        let _border = ui.push_style_var(StyleVar::WindowBorderSize(0.0));
        ui.window("##launcher")
            .position([0.0, 0.0], Condition::Always)
            .size(display_size, Condition::Always)
            .flags(
                WindowFlags::NO_DECORATION
                    | WindowFlags::NO_MOVE
                    | WindowFlags::NO_SAVED_SETTINGS
                    | WindowFlags::NO_BRING_TO_FRONT_ON_FOCUS
                    | WindowFlags::NO_NAV_FOCUS,
            )
            .build(|| {
                self.body(ui, kit, display_size[0], &mut events);
            });
        for ev in &events {
            self.react(*ev);
        }
        events
    }

    fn body(&mut self, ui: &Ui, kit: &Kit, width: f32, events: &mut Vec<LaunchEvent>) {
        // 1. title bar ---------------------------------------------------
        let act = w::title_bar(ui, kit, "Launcher", TABS[self.tab], &TABS, &mut self.tab);
        if act != TitleBarAction::None {
            events.push(LaunchEvent::Window(act));
        }

        // page transition: restart on tab change, advance while enabled
        if self.tab != self.page_tab {
            self.page_dir = if self.tab > self.page_tab { 1.0 } else { -1.0 };
            self.page_tab = self.tab;
            self.page_t = 0.0;
        }
        let st = kit.anim.settings();
        self.page_t = if st.pages {
            let d = (anim::PAGE * st.scale).max(1e-3);
            (self.page_t + ui.io().delta_time.min(0.1) / d).min(1.0)
        } else {
            1.0
        };
        let page = anim::ease_out(self.page_t);

        // 2. status strip ------------------------------------------------
        let windows = format!("{}", self.windows as i32);
        let stagger = format!("{}", self.stagger_ms as i32);
        let launch_label = format!("Launch {} windows", self.windows as i32);
        let stop_w = {
            let _f = ui.push_font(kit.fonts.mono13b);
            grid::button_width(ui, "Stop all")
        };
        let launch_w = {
            let _f = ui.push_font(kit.fonts.mono13b);
            grid::button_width(ui, &launch_label)
        };
        let items = [
            StatItem {
                caption: "Status",
                value: self.status,
                unit: "",
                color: color::OK,
            },
            StatItem {
                caption: "Next slot",
                value: "01",
                unit: "",
                color: color::FG,
            },
            StatItem {
                caption: "Windows",
                value: &windows,
                unit: "",
                color: color::FG,
            },
            StatItem {
                caption: "Stagger",
                value: &stagger,
                unit: "ms",
                color: color::FG,
            },
        ];
        w::status_strip(ui, kit, &items, &[stop_w, launch_w], |ui| {
            if w::button(ui, kit, ButtonKind::Danger, "Stop all") {
                events.push(LaunchEvent::StopAll);
            }
            ui.same_line();
            if w::button(ui, kit, ButtonKind::Primary, &launch_label) {
                events.push(LaunchEvent::Launch);
            }
        });

        // 3. content ---------------------------------------------------------
        let _pad = ui.push_style_var(StyleVar::WindowPadding([space::XL, space::XL]));
        // fade + 24 px slide in the direction of the tab change
        let _alpha = ui.push_style_var(StyleVar::Alpha(page));
        let content_w = ui.content_region_avail()[0];
        let [cx, cy] = ui.cursor_pos();
        ui.set_cursor_pos([cx + (1.0 - page) * 24.0 * self.page_dir, cy]);
        if self.tab == 0 {
            // Launch: 5 / 7 panes on the 12-column grid
            let grid = Grid::default();
            ui.child_window("##content")
                .size([content_w, 0.0])
                .flags(WindowFlags::ALWAYS_USE_WINDOW_PADDING | WindowFlags::NO_SCROLLBAR)
                .build(|| {
                    let span = Grid::form_span(width);
                    grid.panes(ui, span, |ui, pane| match pane {
                        Pane::Left => self.form_pane(ui, kit, events),
                        Pane::Right => self.log_pane(ui, kit, events),
                    });
                });
        } else {
            // Other tabs: the scrolling widget gallery
            let tab = self.tab;
            // The map fills its page and takes the wheel itself.
            let flags = if tab == 7 {
                WindowFlags::ALWAYS_USE_WINDOW_PADDING
                    | WindowFlags::NO_SCROLLBAR
                    | WindowFlags::NO_SCROLL_WITH_MOUSE
            } else {
                WindowFlags::ALWAYS_USE_WINDOW_PADDING
            };
            ui.child_window("##gallery")
                .size([content_w, 0.0])
                .flags(flags)
                .build(|| self.gallery.draw(ui, kit, tab));
        }
    }

    fn form_pane(&mut self, ui: &Ui, kit: &Kit, events: &mut Vec<LaunchEvent>) {
        // GAME PATH
        w::section(ui, kit, "Game path");
        let (browse, open) = w::path_input(ui, kit, "game_path", &mut self.game_path);
        if browse {
            events.push(LaunchEvent::Browse);
        }
        if open {
            events.push(LaunchEvent::OpenFolder);
        }
        w::verified_line(
            ui,
            kit,
            self.path_ok,
            if self.path_ok {
                "Verified"
            } else {
                "Not found"
            },
            "MyGame",
        );

        grid::section_gap(ui);

        // OPTIONS
        w::section(ui, kit, "Options");
        {
            let _sp = ui.push_style_var(StyleVar::ItemSpacing([space::S, 0.0]));
            w::checkbox(
                ui,
                kit,
                "Auto-restart windows that close",
                None,
                &mut self.auto_restart,
            );
            w::checkbox(
                ui,
                kit,
                "Safe mode",
                Some("extra compatibility"),
                &mut self.safe_mode,
            );
            w::checkbox(
                ui,
                kit,
                "Don't spoof GPU",
                Some("use real adapter"),
                &mut self.real_gpu,
            );
            w::checkbox(
                ui,
                kit,
                "Outdated GPU driver dialog",
                None,
                &mut self.gpu_dialog,
            );
            w::checkbox(
                ui,
                kit,
                "Buffer Underflow Fix",
                None,
                &mut self.underflow_fix,
            );
            w::checkbox(
                ui,
                kit,
                "Spoof hash for NEW windows",
                None,
                &mut self.spoof_hash,
            );
        }

        grid::section_gap(ui);

        // PARAMETERS
        w::section(ui, kit, "Parameters");
        {
            let _sp = ui.push_style_var(StyleVar::ItemSpacing([space::S, 0.0]));
            w::labeled_slider(
                ui,
                kit,
                "Windows",
                &mut self.windows,
                1.0,
                12.0,
                1.0,
                "/ 12",
            );
            w::labeled_slider(
                ui,
                kit,
                "Stagger",
                &mut self.stagger_ms,
                0.0,
                2000.0,
                50.0,
                "ms",
            );
        }

        // CTA pinned to the bottom of the pane
        grid::push_to_bottom(ui, size::CTA, 0.0);
        let label = format!("Launch {} windows", self.windows as i32);
        if w::cta(ui, kit, ButtonKind::Primary, &label) {
            events.push(LaunchEvent::Launch);
        }
    }

    fn log_pane(&mut self, ui: &Ui, kit: &Kit, events: &mut Vec<LaunchEvent>) {
        let copy_w = w::button_small_width(ui, kit, "Copy");
        let save_w = w::button_small_width(ui, kit, "Save");
        w::panel_header(ui, kit, "Status", &[copy_w, save_w], |ui| {
            if w::button_small(ui, kit, ButtonKind::Secondary, "Copy") {
                ui.set_clipboard_text(log_text(&self.log));
                events.push(LaunchEvent::CopyLog);
            }
            ui.same_line();
            if w::button_small(ui, kit, ButtonKind::Secondary, "Save") {
                events.push(LaunchEvent::SaveLog);
            }
        });
        ui.dummy([0.0, space::M - space::S]);
        // Clipped: only the rows in view are submitted, whatever the log size.
        let row_h = {
            let _f = ui.push_font(kit.fonts.mono12);
            ui.text_line_height()
        };
        let log = &self.log;
        w::log_list(ui, "##log", [0.0, 0.0], log.len(), row_h, |ui, i| {
            let (t, m) = &log[i];
            w::log_line(ui, kit, *t, m, None);
        });
    }

    /// Ctrl+1…8 pick a tab, Ctrl+Tab / Ctrl+Shift+Tab cycle them,
    /// Ctrl+Enter launches, Ctrl+Backspace stops everything.
    fn hotkeys(&mut self, ui: &Ui, events: &mut Vec<LaunchEvent>) {
        let io = ui.io();
        if !io.key_ctrl {
            return;
        }
        const DIGITS: [Key; 8] = [
            Key::Alpha1,
            Key::Alpha2,
            Key::Alpha3,
            Key::Alpha4,
            Key::Alpha5,
            Key::Alpha6,
            Key::Alpha7,
            Key::Alpha8,
        ];
        for (i, k) in DIGITS.iter().enumerate() {
            if ui.is_key_pressed_no_repeat(*k) {
                self.tab = i;
            }
        }
        if ui.is_key_pressed_no_repeat(Key::Tab) {
            let n = TABS.len();
            self.tab = if io.key_shift {
                (self.tab + n - 1) % n
            } else {
                (self.tab + 1) % n
            };
        }
        if ui.is_key_pressed_no_repeat(Key::Enter) {
            events.push(LaunchEvent::Launch);
        }
        if ui.is_key_pressed_no_repeat(Key::Backspace) {
            events.push(LaunchEvent::StopAll);
        }
    }

    /// The log as text, one `[t] message` per line.
    pub fn log_text(&self) -> String {
        log_text(&self.log)
    }

    /// Restores what [`LaunchScreen::persist`] saved.
    pub fn restore(&mut self, kit: &Kit, s: &Settings) {
        self.tab = s.get_or("tab", self.tab).min(TABS.len() - 1);
        self.page_tab = self.tab;
        if let Some(p) = s.get::<String>("game_path") {
            self.game_path = p;
        }
        self.auto_restart = s.get_or("auto_restart", self.auto_restart);
        self.safe_mode = s.get_or("safe_mode", self.safe_mode);
        self.real_gpu = s.get_or("real_gpu", self.real_gpu);
        self.gpu_dialog = s.get_or("gpu_dialog", self.gpu_dialog);
        self.underflow_fix = s.get_or("underflow_fix", self.underflow_fix);
        self.spoof_hash = s.get_or("spoof_hash", self.spoof_hash);
        self.windows = s.get_or("windows", self.windows).clamp(1.0, 12.0);
        self.stagger_ms = s.get_or("stagger_ms", self.stagger_ms).clamp(0.0, 2000.0);
        let map = &mut self.gallery.map;
        map.view.zoom = s
            .get_or("map.zoom", map.view.zoom)
            .clamp(map.view.min_zoom, map.view.max_zoom);
        map.view.follow = s.get_or("map.follow", map.view.follow);
        if let (Some(x), Some(y)) = (s.get("map.center_x"), s.get("map.center_y")) {
            map.view.center = [x, y];
        }
        map.view.show_tile_grid = s.get_or("map.regions", map.view.show_tile_grid);
        map.show_geo = s.get_or("map.geo", map.show_geo);
        map.show_walls = s.get_or("map.walls", map.show_walls);
        map.show_blocks = s.get_or("map.blocks", map.show_blocks);
        map.show_npcs = s.get_or("map.npcs", map.show_npcs);
        map.walking = s.get_or("map.walk", map.walking);
        let mut a = kit.anim.settings();
        a.controls = s.get_or("anim.controls", a.controls);
        a.tabs = s.get_or("anim.tabs", a.tabs);
        a.pages = s.get_or("anim.pages", a.pages);
        a.scale = s.get_or("anim.scale", a.scale).clamp(0.25, 4.0);
        kit.anim.set(a);
    }

    /// Everything worth keeping between runs.
    pub fn persist(&self, kit: &Kit) -> Settings {
        let mut s = Settings::default();
        s.set("tab", self.tab);
        s.set("game_path", &self.game_path);
        s.set("auto_restart", self.auto_restart);
        s.set("safe_mode", self.safe_mode);
        s.set("real_gpu", self.real_gpu);
        s.set("gpu_dialog", self.gpu_dialog);
        s.set("underflow_fix", self.underflow_fix);
        s.set("spoof_hash", self.spoof_hash);
        s.set("windows", self.windows);
        s.set("stagger_ms", self.stagger_ms);
        let map = &self.gallery.map;
        s.set("map.zoom", map.view.zoom);
        s.set("map.follow", map.view.follow);
        s.set("map.center_x", map.view.center[0]);
        s.set("map.center_y", map.view.center[1]);
        s.set("map.regions", map.view.show_tile_grid);
        s.set("map.geo", map.show_geo);
        s.set("map.walls", map.show_walls);
        s.set("map.blocks", map.show_blocks);
        s.set("map.npcs", map.show_npcs);
        s.set("map.walk", map.walking);
        let a = kit.anim.settings();
        s.set("anim.controls", a.controls);
        s.set("anim.tabs", a.tabs);
        s.set("anim.pages", a.pages);
        s.set("anim.scale", a.scale);
        s
    }

    /// Whether the screen changes without input right now: a page
    /// transition in flight or a self-animating page. Together with
    /// [`imgui_kit::anim::animating`] this tells a host that renders on demand
    /// when it may stop scheduling frames.
    pub fn is_animating(&self) -> bool {
        self.page_t < 1.0 || self.gallery.is_animating(self.tab)
    }

    /// Appends a line to the status log, stamped after the last one; the
    /// oldest line goes once [`LOG_CAP`] is reached.
    pub fn log(&mut self, message: impl Into<String>) {
        let t = self.log.back().map_or(0.0, |(t, _)| t + 0.084);
        self.log.push_back((t, message.into()));
        while self.log.len() > LOG_CAP {
            self.log.pop_front();
        }
    }

    /// Reacts to the screen's own events so the demo feels alive: `Launch`
    /// logs one line per window, `StopAll` logs the stop, `CopyLog` /
    /// `SaveLog` acknowledge. The host still receives the event.
    fn react(&mut self, ev: LaunchEvent) {
        match ev {
            LaunchEvent::Launch => {
                let n = self.windows as i32;
                self.log(format!(
                    "Launching {n} windows, stagger {} ms",
                    self.stagger_ms as i32
                ));
                for i in 1..=n {
                    self.log(format!("Window {i}/{n} started on proxy {i}"));
                }
                self.status = "Running";
            }
            LaunchEvent::StopAll => {
                self.log("Stop requested, closing all windows");
                self.status = "Ready";
            }
            LaunchEvent::CopyLog => self.log("Log copied to clipboard"),
            LaunchEvent::SaveLog => {
                let path = crate::assets::data_dir().join("launcher.log");
                match std::fs::write(&path, self.log_text()) {
                    Ok(()) => self.log(format!("Log saved to {}", path.display())),
                    Err(e) => self.log(format!("Log not saved: {e}")),
                }
            }
            _ => {}
        }
    }
}

/// [`LaunchScreen::log_text`] on a bare log, usable while `self` is borrowed.
fn log_text(log: &VecDeque<(f32, String)>) -> String {
    let mut s = String::new();
    for (t, m) in log {
        s.push_str(&format!("[{t:.3}] {m}\n"));
    }
    s
}
