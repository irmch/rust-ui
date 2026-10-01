//! Widget gallery shown on the tabs other than *Launch*: every control of
//! the kit in every state, so the theme and the layout helpers can be
//! eyeballed in one place. Tabs: Accounts → data & cards, Instances →
//! table & progress, Proxies → inputs & selection, Resources → stats &
//! sliders, Tools → buttons & banners, Settings → grid & text styles.

use imgui::{StyleVar, TableFlags, TableRowFlags, Ui};

use crate::anim;
use crate::fonts::Fonts;
use crate::grid::{self, Grid};
use crate::map_demo::MapPage;
use crate::theme::ButtonKind;
use crate::tokens::{color, size, space};
use crate::widgets::{self as w, TagKind};

/// Radio group value of the gallery.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Region {
    Eu,
    Us,
    Asia,
}

/// State of the gallery widgets.
pub struct Gallery {
    pub login: String,
    pub password: String,
    pub proxy: String,
    pub path: String,
    pub region: Region,
    pub switches: [bool; 4],
    pub checks: [bool; 3],
    pub volume: f32,
    pub delay: f32,
    pub raw: f32,
    pub disabled: bool,
    pub events: Vec<String>,
    /// The Map tab.
    pub map: MapPage,
}

impl Default for Gallery {
    fn default() -> Self {
        Self {
            login: "player_one".into(),
            password: String::new(),
            proxy: String::new(),
            path: r"C:\Games\Path of Exile 2".into(),
            region: Region::Eu,
            switches: [true, false, true, false],
            checks: [true, false, false],
            volume: 65.0,
            delay: 250.0,
            raw: 0.4,
            disabled: true,
            events: Vec::new(),
            map: MapPage::default(),
        }
    }
}

/// Height of the account cards: 16 pad + 13 + 8 + 20 + 8 + 20 + 8 + 24 + 16 pad.
const CARD_H: f32 = 136.0;

const ACCOUNTS: [(&str, &str, TagKind, &str); 6] = [
    ("alpha@mail.com", "Running", TagKind::Ok, "proxy 1"),
    ("bravo@mail.com", "Running", TagKind::Ok, "proxy 2"),
    ("charlie@mail.com", "Idle", TagKind::Neutral, "proxy 3"),
    ("delta@mail.com", "Banned", TagKind::Err, "no proxy"),
    ("echo@mail.com", "Captcha", TagKind::Warn, "proxy 5"),
    ("foxtrot@mail.com", "Queued", TagKind::Info, "proxy 6"),
];

impl Gallery {
    /// Draws the gallery page for `tab` (1 ..= 7) into the current window.
    pub fn draw(&mut self, ui: &Ui, f: &Fonts, tab: usize) {
        match tab {
            1 => self.accounts(ui, f),
            2 => self.instances(ui, f),
            3 => self.proxies(ui, f),
            4 => self.resources(ui, f),
            5 => self.tools(ui, f),
            7 => self.map.draw(ui, f),
            _ => self.settings(ui, f),
        }
    }

    /// Whether page `tab` changes by itself (needs frames without input):
    /// the animated progress bar on Instances, the walking player on Map.
    pub fn is_animating(&self, tab: usize) -> bool {
        match tab {
            2 => true,
            7 => self.map.walking,
            _ => false,
        }
    }

    fn note(&mut self, s: impl Into<String>) {
        self.events.push(s.into());
        if self.events.len() > 6 {
            self.events.remove(0);
        }
    }

    // 1 · Accounts: cards, tags, status dots, log lines --------------------
    fn accounts(&mut self, ui: &Ui, f: &Fonts) {
        let grid = Grid::default();

        w::section(ui, f, "Cards · 3 × span 4");
        let switches = &mut self.switches;
        let events = &mut self.events;
        grid.row(ui, "cards", &[4, 4, 4], |ui, i, width| {
            let (mail, status, kind, proxy) = ACCOUNTS[i];
            w::card(ui, &format!("##card{i}"), [width, CARD_H], |ui| {
                w::text_bold(ui, f, mail, color::FG);
                w::tag(ui, f, kind, status);
                ui.same_line_with_spacing(0.0, space::S);
                w::status_dot(ui, status_color(kind), proxy);
                let on = &mut switches[i % 4];
                w::switch(ui, f, "Auto-restart", on);
                if w::button_small(ui, f, ButtonKind::Secondary, "Open") {
                    events.push(format!("open {mail}"));
                }
                ui.same_line();
                if w::button_small(ui, f, ButtonKind::Danger, "Remove") {
                    events.push(format!("remove {mail}"));
                }
            });
        });

        grid::section_gap(ui);

        w::section(ui, f, "Tags");
        for (i, (kind, text)) in [
            (TagKind::Neutral, "Idle"),
            (TagKind::Solid, "Selected"),
            (TagKind::Ok, "Running"),
            (TagKind::Warn, "Captcha"),
            (TagKind::Err, "Banned"),
            (TagKind::Info, "Queued"),
        ]
        .into_iter()
        .enumerate()
        {
            if i > 0 {
                ui.same_line_with_spacing(0.0, space::S);
            }
            w::tag(ui, f, kind, text);
        }

        grid::section_gap(ui);

        w::section(ui, f, "Status dots");
        for (i, (col, text)) in [
            (color::OK, "Online"),
            (color::WARN, "Degraded"),
            (color::ERR, "Offline"),
            (color::INFO, "Connecting"),
            (color::FG3, "Unknown"),
        ]
        .into_iter()
        .enumerate()
        {
            if i > 0 {
                ui.same_line_with_spacing(0.0, space::L);
            }
            w::status_dot(ui, col, text);
        }

        grid::section_gap(ui);

        w::section(ui, f, "Log lines");
        w::panel(ui, "##acc_log", [0.0, 7.0 * 16.0 + 2.0 * space::S], |ui| {
            let _sp = ui.push_style_var(StyleVar::ItemSpacing([space::S, 0.0]));
            w::log_line(ui, f, 0.000, "Default line", None);
            w::log_line(ui, f, 0.084, "Success line", Some(color::OK));
            w::log_line(ui, f, 0.168, "Warning line", Some(color::WARN));
            w::log_line(ui, f, 0.252, "Error line", Some(color::ERR));
            w::log_line(ui, f, 0.336, "Info line", Some(color::INFO));
            w::log_line(ui, f, 0.420, "Muted line", Some(color::FG3));
            for e in &self.events {
                w::log_line(ui, f, 1.0, e, Some(color::FG2));
            }
        });
    }

    // 2 · Instances: table, progress -----------------------------------------
    fn instances(&mut self, ui: &Ui, f: &Fonts) {
        w::section(ui, f, "Table · 32 px rows");
        let flags = TableFlags::ROW_BG | TableFlags::BORDERS_INNER_H | TableFlags::SIZING_STRETCH_PROP;
        let _pad = ui.push_style_var(StyleVar::CellPadding([size::PAD_X, 0.0]));
        if let Some(_t) = ui.begin_table_with_flags("##instances", 5, flags) {
            for col in ["Slot", "Account", "Status", "Uptime", "CPU"] {
                ui.table_setup_column(col);
            }
            {
                let _f = ui.push_font(f.mono10);
                ui.table_next_row_with_height(TableRowFlags::HEADERS, size::TABLE_ROW);
                for col in ["SLOT", "ACCOUNT", "STATUS", "UPTIME", "CPU"] {
                    ui.table_next_column();
                    grid::vcenter(ui, ui.text_line_height(), size::TABLE_ROW);
                    ui.text_colored(color::FG3, col);
                }
            }
            for (i, (mail, status, kind, _)) in ACCOUNTS.iter().enumerate() {
                ui.table_next_row_with_height(TableRowFlags::empty(), size::TABLE_ROW);
                let lh = ui.text_line_height();
                ui.table_next_column();
                grid::vcenter(ui, lh, size::TABLE_ROW);
                w::text_bold(ui, f, &format!("{:02}", i + 1), color::FG);
                ui.table_next_column();
                grid::vcenter(ui, lh, size::TABLE_ROW);
                ui.text(mail);
                ui.table_next_column();
                grid::vcenter(ui, size::TAG, size::TABLE_ROW);
                w::tag(ui, f, *kind, status);
                ui.table_next_column();
                grid::vcenter(ui, lh, size::TABLE_ROW);
                w::text_muted(ui, &format!("{}h {:02}m", i * 3 + 1, i * 17 % 60));
                ui.table_next_column();
                grid::vcenter(ui, 8.0, size::TABLE_ROW);
                w::progress(ui, (i as f32 + 1.0) / 7.0, 0.0);
            }
        }
        drop(_pad);

        grid::section_gap(ui);

        w::section(ui, f, "Progress");
        let t = ui.time() as f32;
        let anim = (t * 0.25).fract();
        for (i, (label, frac)) in [("Idle", 0.0), ("Quarter", 0.25), ("Half", 0.5), ("Done", 1.0), ("Animated", anim)]
            .into_iter()
            .enumerate()
        {
            if i > 0 {
                ui.dummy([0.0, space::S]);
            }
            w::caption(ui, f, label);
            ui.dummy([0.0, space::XS]);
            w::progress(ui, frac, 0.0);
        }
    }

    // 3 · Proxies: inputs, radio, switches, checkboxes, disabled -------------
    fn proxies(&mut self, ui: &Ui, f: &Fonts) {
        w::section(ui, f, "Inputs");
        {
            let _sp = ui.push_style_var(StyleVar::ItemSpacing([space::S, space::S]));
            w::input(ui, f, "login", "Login", &mut self.login, 320.0, false);
            w::input(ui, f, "password", "Password (empty, shows hint)", &mut self.password, 320.0, false);
            w::input(ui, f, "proxy", "host:port:user:pass", &mut self.proxy, 0.0, true);
        }
        ui.dummy([0.0, space::S]);
        let (browse, open) = w::path_input(ui, f, "gal_path", &mut self.path);
        if browse {
            self.note("browse");
        }
        if open {
            self.note("open");
        }
        w::verified_line(ui, f, true, "Verified", "Path of Exile 2");
        w::verified_line(ui, f, false, "Not found", "check the path");

        grid::section_gap(ui);

        w::section(ui, f, "Radio");
        {
            let _sp = ui.push_style_var(StyleVar::ItemSpacing([space::S, 0.0]));
            w::radio(ui, f, "Europe", Some("lowest ping"), &mut self.region, Region::Eu);
            w::radio(ui, f, "United States", None, &mut self.region, Region::Us);
            w::radio(ui, f, "Asia", Some("beta"), &mut self.region, Region::Asia);
        }

        grid::section_gap(ui);

        w::section(ui, f, "Switches");
        {
            let _sp = ui.push_style_var(StyleVar::ItemSpacing([space::S, 0.0]));
            w::switch(ui, f, "Rotate proxies", &mut self.switches[0]);
            w::switch(ui, f, "Sticky sessions", &mut self.switches[1]);
            w::switch(ui, f, "Check before launch", &mut self.switches[2]);
            w::switch(ui, f, "Log traffic", &mut self.switches[3]);
        }

        grid::section_gap(ui);

        w::section(ui, f, "Checkboxes");
        {
            let _sp = ui.push_style_var(StyleVar::ItemSpacing([space::S, 0.0]));
            w::checkbox(ui, f, "Checked", None, &mut self.checks[0]);
            w::checkbox(ui, f, "Unchecked with hint", Some("explains the option"), &mut self.checks[1]);
            w::checkbox(ui, f, "Another one", None, &mut self.checks[2]);
        }

        grid::section_gap(ui);

        w::section(ui, f, "Disabled");
        w::switch(ui, f, "Disable the block below", &mut self.disabled);
        ui.dummy([0.0, space::S]);
        let disabled = self.disabled;
        w::disabled(ui, disabled, || {
            let _sp = ui.push_style_var(StyleVar::ItemSpacing([space::S, 0.0]));
            let mut on = true;
            w::checkbox(ui, f, "Disabled checkbox", Some("40 % alpha"), &mut on);
            w::switch(ui, f, "Disabled switch", &mut on);
            ui.dummy([0.0, space::S]);
            w::button(ui, f, ButtonKind::Primary, "Disabled primary");
            ui.same_line();
            w::button(ui, f, ButtonKind::Secondary, "Disabled secondary");
        });
    }

    // 4 · Resources: stats, dividers, sliders ----------------------------------
    fn resources(&mut self, ui: &Ui, f: &Fonts) {
        w::section(ui, f, "Stats");
        for (i, (cap, val, unit, col)) in [
            ("Status", "Ready", "", color::OK),
            ("CPU", "42", "%", color::FG),
            ("RAM", "6.4", "GB", color::FG),
            ("Errors", "3", "", color::ERR),
            ("Warnings", "12", "", color::WARN),
        ]
        .into_iter()
        .enumerate()
        {
            if i > 0 {
                ui.same_line_with_spacing(0.0, space::L);
                w::vdivider(ui, ui.text_line_height());
                ui.same_line_with_spacing(0.0, space::L);
            }
            w::stat(ui, f, cap, val, unit, col);
        }

        grid::section_gap(ui);

        w::section(ui, f, "Labeled sliders · form row 96 | stretch | 64");
        {
            let _sp = ui.push_style_var(StyleVar::ItemSpacing([space::S, 0.0]));
            w::labeled_slider(ui, f, "Volume", &mut self.volume, 0.0, 100.0, 1.0, "%");
            w::labeled_slider(ui, f, "Delay", &mut self.delay, 0.0, 1000.0, 10.0, "ms");
        }

        grid::section_gap(ui);

        w::section(ui, f, "Raw slider track · 320 px");
        w::slider_track(ui, "raw", &mut self.raw, 0.0, 1.0, 0.05, 320.0);
        ui.same_line_with_spacing(0.0, space::L);
        grid::vcenter(ui, ui.text_line_height(), size::CONTROL);
        w::text_bold(ui, f, &format!("{:.2}", self.raw), color::FG);

        grid::section_gap(ui);

        w::section(ui, f, "Progress bound to the sliders");
        w::progress(ui, self.volume / 100.0, 320.0);
        ui.dummy([0.0, space::S]);
        w::progress(ui, self.raw, 320.0);
    }

    // 5 · Tools: buttons, banners ----------------------------------------------
    fn tools(&mut self, ui: &Ui, f: &Fonts) {
        w::section(ui, f, "Buttons · 32 px");
        let kinds = [
            (ButtonKind::Primary, "Primary"),
            (ButtonKind::Secondary, "Secondary"),
            (ButtonKind::Danger, "Danger"),
            (ButtonKind::Ghost, "Ghost"),
        ];
        for (i, (kind, label)) in kinds.into_iter().enumerate() {
            if i > 0 {
                ui.same_line_with_spacing(0.0, space::S);
            }
            if w::button(ui, f, kind, label) {
                self.note(format!("button {label}"));
            }
        }
        ui.dummy([0.0, space::S]);
        for (i, (kind, label)) in kinds.into_iter().enumerate() {
            if i > 0 {
                ui.same_line_with_spacing(0.0, space::S);
            }
            w::button_sized(ui, f, kind, &format!("{label} 160"), [160.0, size::CONTROL]);
        }

        ui.dummy([0.0, space::M]);
        w::caption(ui, f, "Small · 24 px");
        ui.dummy([0.0, space::XS]);
        for (i, (kind, label)) in kinds.into_iter().enumerate() {
            if i > 0 {
                ui.same_line_with_spacing(0.0, space::S);
            }
            w::button_small(ui, f, kind, label);
        }

        ui.dummy([0.0, space::M]);
        w::caption(ui, f, "Icon buttons · 32 and 24 px");
        ui.dummy([0.0, space::XS]);
        for (i, (kind, glyph)) in [
            (ButtonKind::Primary, "+"),
            (ButtonKind::Secondary, "×"),
            (ButtonKind::Danger, "–"),
            (ButtonKind::Ghost, "□"),
        ]
        .into_iter()
        .enumerate()
        {
            if i > 0 {
                ui.same_line_with_spacing(0.0, space::S);
            }
            w::icon_button(ui, kind, &format!("ib{i}"), glyph, false);
            ui.same_line_with_spacing(0.0, space::XS);
            w::icon_button(ui, kind, &format!("ibs{i}"), glyph, true);
        }

        ui.dummy([0.0, space::M]);
        w::caption(ui, f, "Disabled");
        ui.dummy([0.0, space::XS]);
        w::disabled(ui, true, || {
            for (i, (kind, label)) in kinds.into_iter().enumerate() {
                if i > 0 {
                    ui.same_line_with_spacing(0.0, space::S);
                }
                w::button(ui, f, kind, label);
            }
        });

        grid::section_gap(ui);

        w::section(ui, f, "Call to action · 48 px");
        if w::cta(ui, f, ButtonKind::Primary, "Launch 6 windows") {
            self.note("cta primary");
        }
        ui.dummy([0.0, space::S]);
        w::cta(ui, f, ButtonKind::Secondary, "Secondary action");
        ui.dummy([0.0, space::S]);
        w::cta(ui, f, ButtonKind::Danger, "Stop everything");

        grid::section_gap(ui);

        w::section(ui, f, "Banners");
        for (i, (kind, text, action)) in [
            (TagKind::Info, "6 accounts ready to launch", Some("Launch")),
            (TagKind::Ok, "All proxies are reachable", None),
            (TagKind::Warn, "2 accounts need a captcha", Some("Solve")),
            (TagKind::Err, "Game path not found", Some("Browse")),
            (TagKind::Neutral, "Idle banner without colour", None),
        ]
        .into_iter()
        .enumerate()
        {
            if i > 0 {
                ui.dummy([0.0, space::S]);
            }
            if w::banner(ui, f, kind, text, action) {
                self.note(format!("banner {text}"));
            }
        }
    }

    // 6 · Settings: grid rows, form rows, text styles, panels ------------------
    fn settings(&mut self, ui: &Ui, f: &Fonts) {
        let grid = Grid::default();

        w::section(ui, f, "Animation");
        {
            let _sp = ui.push_style_var(StyleVar::ItemSpacing([space::S, 0.0]));
            let mut st = anim::settings();
            let mut changed = false;
            changed |= w::switch(ui, f, "Controls: checkbox, radio, switch", &mut st.controls);
            changed |= w::switch(ui, f, "Tabs: sliding highlight", &mut st.tabs);
            changed |= w::switch(ui, f, "Pages: fade and slide on tab change", &mut st.pages);
            changed |= w::labeled_slider(ui, f, "Scale", &mut st.scale, 0.25, 4.0, 0.25, "×");
            if changed {
                anim::set(st);
            }
        }

        grid::section_gap(ui);

        w::section(ui, f, "Grid row · spans 3 / 3 / 6");
        grid.row(ui, "spans", &[3, 3, 6], |ui, i, width| {
            w::panel(ui, &format!("##span{i}"), [width, 56.0], |ui| {
                w::caption(ui, f, &format!("span {}", [3, 3, 6][i]));
                ui.text(format!("{width:.0} px"));
            });
        });
        ui.dummy([0.0, space::S]);
        grid.row(ui, "spans2", &[2, 2, 2, 2, 2, 2], |ui, i, width| {
            w::panel(ui, &format!("##span2_{i}"), [width, 40.0], |ui| {
                w::text_muted(ui, &format!("{width:.0}"));
            });
        });

        grid::section_gap(ui);

        w::section(ui, f, "Form rows · 96 | stretch | 64");
        {
            let _sp = ui.push_style_var(StyleVar::ItemSpacing([space::S, 0.0]));
            grid::form_row(ui, "##fr_login", |ui, cell, _w| match cell {
                grid::FormCell::Label => {
                    grid::vcenter(ui, ui.text_line_height(), size::CONTROL);
                    w::caption(ui, f, "Login");
                }
                grid::FormCell::Control => {
                    w::input(ui, f, "fr_login", "Login", &mut self.login, 0.0, false);
                }
                grid::FormCell::Value => {
                    grid::vcenter(ui, size::TAG, size::CONTROL);
                    w::tag(ui, f, TagKind::Ok, "ok");
                }
            });
            grid::form_row(ui, "##fr_region", |ui, cell, _w| match cell {
                grid::FormCell::Label => {
                    grid::vcenter(ui, ui.text_line_height(), size::CONTROL);
                    w::caption(ui, f, "Region");
                }
                grid::FormCell::Control => {
                    w::radio(ui, f, "EU", None, &mut self.region, Region::Eu);
                    ui.same_line_with_spacing(0.0, space::L);
                    w::radio(ui, f, "US", None, &mut self.region, Region::Us);
                    ui.same_line_with_spacing(0.0, space::L);
                    w::radio(ui, f, "Asia", None, &mut self.region, Region::Asia);
                }
                grid::FormCell::Value => {
                    grid::vcenter(ui, ui.text_line_height(), size::CONTROL);
                    w::text_muted(ui, &format!("{:?}", self.region));
                }
            });
        }

        grid::section_gap(ui);

        w::section(ui, f, "Text styles");
        {
            let _sp = ui.push_style_var(StyleVar::ItemSpacing([space::S, space::XS]));
            w::caption(ui, f, "Caption · 10 px semibold tracked");
            ui.text("Body · 13 px regular");
            w::text_bold(ui, f, "Label · 13 px bold", color::FG);
            w::text_muted(ui, "Muted · fg-2");
            ui.text_colored(color::FG3, "Disabled · fg-3");
            {
                let _f = ui.push_font(f.mono12);
                ui.text("Small · 12 px for logs and cells");
            }
            {
                let _f = ui.push_font(f.mono16b);
                ui.text("Title · 16 px bold");
            }
            {
                let _f = ui.push_font(f.mono20b);
                ui.text("Display · 20 px bold · 1280");
            }
            ui.text("Body with an inline hint");
            w::hint_inline(ui, f, "extra compatibility");
        }

        grid::section_gap(ui);

        w::section(ui, f, "Panel header + panel");
        w::panel_header(ui, f, "Events", &[grid::button_width(ui, "Clear") - 8.0], |ui| {
            if w::button_small(ui, f, ButtonKind::Secondary, "Clear") {
                self.events.clear();
            }
        });
        ui.dummy([0.0, space::M - space::S]);
        w::panel(ui, "##events", [0.0, 6.0 * 16.0 + 2.0 * space::S], |ui| {
            let _sp = ui.push_style_var(StyleVar::ItemSpacing([space::S, 0.0]));
            if self.events.is_empty() {
                w::text_muted(ui, "Click buttons on the other tabs to log events here.");
            }
            for (i, e) in self.events.iter().enumerate() {
                w::log_line(ui, f, i as f32 * 0.1, e, None);
            }
        });
    }
}

fn status_color(kind: TagKind) -> crate::tokens::Rgba {
    match kind {
        TagKind::Ok => color::OK,
        TagKind::Warn => color::WARN,
        TagKind::Err => color::ERR,
        TagKind::Info => color::INFO,
        TagKind::Solid => color::FG,
        TagKind::Neutral => color::FG3,
    }
}
