//! The reference "Launch" screen (artboard 06) assembled from the kit.
//! Draw it with [`LaunchScreen::draw`] inside your frame loop.

use imgui::{Condition, StyleVar, Ui, WindowFlags};

use crate::fonts::Fonts;
use crate::grid::{self, Grid, Pane};
use crate::theme::ButtonKind;
use crate::tokens::{color, size, space};
use crate::widgets::{self as w, StatItem, TitleBarAction};

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
    pub log: Vec<(f32, String)>,
    pub status: &'static str,
}

impl Default for LaunchScreen {
    fn default() -> Self {
        Self {
            tab: 0,
            game_path: r"C:\Games\Path of Exile 2\PathOfExile.exe".into(),
            path_ok: true,
            auto_restart: true,
            safe_mode: false,
            real_gpu: false,
            gpu_dialog: false,
            underflow_fix: false,
            spoof_hash: true,
            windows: 6.0,
            stagger_ms: 350.0,
            log: vec![
                (0.000, "Game path verified".into()),
                (0.084, "6 accounts ready".into()),
                (0.168, "Per-window proxies assigned".into()),
                (0.252, "Auto-restart enabled".into()),
                (0.336, "Ready to launch".into()),
            ],
            status: "Ready",
        }
    }
}

/// Events the screen reports back to the application.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LaunchEvent {
    None,
    Launch,
    StopAll,
    Browse,
    OpenFolder,
    CopyLog,
    SaveLog,
    Window(TitleBarAction),
}

const TABS: [&str; 7] = ["Launch", "Accounts", "Instances", "Proxies", "Resources", "Tools", "Settings"];

impl LaunchScreen {
    /// Draws the screen as a borderless full-display window.
    pub fn draw(&mut self, ui: &Ui, f: &Fonts, display_size: [f32; 2]) -> LaunchEvent {
        let mut ev = LaunchEvent::None;
        let _pad = ui.push_style_var(StyleVar::WindowPadding([0.0, 0.0]));
        let _rounding = ui.push_style_var(StyleVar::WindowRounding(0.0));
        let _border = ui.push_style_var(StyleVar::WindowBorderSize(0.0));
        ui.window("##poemulti")
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
                ev = self.body(ui, f, display_size[0]);
            });
        ev
    }

    fn body(&mut self, ui: &Ui, f: &Fonts, width: f32) -> LaunchEvent {
        let mut ev = LaunchEvent::None;

        // 1. title bar ---------------------------------------------------
        let act = w::title_bar(ui, f, "PoEMulti", TABS[self.tab], &TABS, &mut self.tab);
        if act != TitleBarAction::None {
            ev = LaunchEvent::Window(act);
        }

        // 2. status strip ------------------------------------------------
        let windows = format!("{}", self.windows as i32);
        let stagger = format!("{}", self.stagger_ms as i32);
        let launch_label = format!("Launch {} windows", self.windows as i32);
        let stop_w = {
            let _f = ui.push_font(f.mono13b);
            grid::button_width(ui, "Stop all")
        };
        let launch_w = {
            let _f = ui.push_font(f.mono13b);
            grid::button_width(ui, &launch_label)
        };
        let items = [
            StatItem { caption: "Status", value: self.status, unit: "", color: color::OK },
            StatItem { caption: "Next slot", value: "01", unit: "", color: color::FG },
            StatItem { caption: "Windows", value: &windows, unit: "", color: color::FG },
            StatItem { caption: "Stagger", value: &stagger, unit: "ms", color: color::FG },
        ];
        w::status_strip(ui, f, &items, &[stop_w, launch_w], |ui| {
            if w::button(ui, f, ButtonKind::Danger, "Stop all") {
                ev = LaunchEvent::StopAll;
            }
            ui.same_line();
            if w::button(ui, f, ButtonKind::Primary, &launch_label) {
                ev = LaunchEvent::Launch;
            }
        });

        // 3. content: 5 / 7 panes on the 12-column grid ---------------------
        let grid = Grid::default();
        let _pad = ui.push_style_var(StyleVar::WindowPadding([space::XL, space::XL]));
        ui.child_window("##content")
            .size([0.0, 0.0])
            .flags(WindowFlags::ALWAYS_USE_WINDOW_PADDING | WindowFlags::NO_SCROLLBAR)
            .build(|| {
                let span = Grid::form_span(width);
                grid.panes(ui, span, |ui, pane| {
                    let e = match pane {
                        Pane::Left => self.form_pane(ui, f),
                        Pane::Right => self.log_pane(ui, f),
                    };
                    if let Some(e) = e {
                        ev = e;
                    }
                });
            });
        ev
    }

    fn form_pane(&mut self, ui: &Ui, f: &Fonts) -> Option<LaunchEvent> {
        let mut ev = None;

        // GAME PATH
        w::section(ui, f, "Game path");
        let (browse, open) = w::path_input(ui, f, "game_path", &mut self.game_path);
        if browse {
            ev = Some(LaunchEvent::Browse);
        }
        if open {
            ev = Some(LaunchEvent::OpenFolder);
        }
        w::verified_line(ui, f, self.path_ok, if self.path_ok { "Verified" } else { "Not found" }, "Path of Exile 2");

        grid::section_gap(ui);

        // OPTIONS
        w::section(ui, f, "Options");
        {
            let _sp = ui.push_style_var(StyleVar::ItemSpacing([space::S, 0.0]));
            w::checkbox(ui, f, "Auto-restart windows that close", None, &mut self.auto_restart);
            w::checkbox(ui, f, "Safe mode", Some("extra compatibility"), &mut self.safe_mode);
            w::checkbox(ui, f, "Don't spoof GPU", Some("use real adapter"), &mut self.real_gpu);
            w::checkbox(ui, f, "Outdated GPU driver dialog", None, &mut self.gpu_dialog);
            w::checkbox(ui, f, "Buffer Underflow Fix", None, &mut self.underflow_fix);
            w::checkbox(ui, f, "Spoof hash for NEW windows", None, &mut self.spoof_hash);
        }

        grid::section_gap(ui);

        // PARAMETERS
        w::section(ui, f, "Parameters");
        {
            let _sp = ui.push_style_var(StyleVar::ItemSpacing([space::S, 0.0]));
            w::labeled_slider(ui, f, "Windows", &mut self.windows, 1.0, 12.0, 1.0, "/ 12");
            w::labeled_slider(ui, f, "Stagger", &mut self.stagger_ms, 0.0, 2000.0, 50.0, "ms");
        }

        // CTA pinned to the bottom of the pane
        grid::push_to_bottom(ui, size::CTA, 0.0);
        let label = format!("Launch {} windows", self.windows as i32);
        if w::cta(ui, f, ButtonKind::Primary, &label) {
            ev = Some(LaunchEvent::Launch);
        }
        ev
    }

    fn log_pane(&mut self, ui: &Ui, f: &Fonts) -> Option<LaunchEvent> {
        let mut ev = None;
        let copy_w = {
            let _f = ui.push_font(f.mono12);
            ui.calc_text_size("Copy")[0] + 20.0
        };
        let save_w = {
            let _f = ui.push_font(f.mono12);
            ui.calc_text_size("Save")[0] + 20.0
        };
        w::panel_header(ui, f, "Status", &[copy_w, save_w], |ui| {
            if w::button_small(ui, f, ButtonKind::Secondary, "Copy") {
                ev = Some(LaunchEvent::CopyLog);
            }
            ui.same_line();
            if w::button_small(ui, f, ButtonKind::Secondary, "Save") {
                ev = Some(LaunchEvent::SaveLog);
            }
        });
        ui.dummy([0.0, space::M - space::S]);
        w::panel(ui, "##log", [0.0, 0.0], |ui| {
            let _sp = ui.push_style_var(StyleVar::ItemSpacing([space::S, 0.0]));
            for (t, m) in &self.log {
                w::log_line(ui, f, *t, m, None);
            }
        });
        ev
    }
}
