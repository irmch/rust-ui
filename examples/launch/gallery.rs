//! Widget gallery shown on the tabs other than *Launch*: every control of
//! the kit in every state, so the theme and the layout helpers can be
//! eyeballed in one place. Tabs: Accounts → data & cards, Instances →
//! table & progress, Proxies → inputs & selection, Resources → stats &
//! sliders, Tools → buttons & banners, Settings → grid & text styles.

use imgui::{StyleVar, TableFlags, TableRowFlags, Ui};

use imgui_kit::Kit;
use imgui_kit::grid::{self, Grid};
use crate::map_demo::MapPage;
use imgui_kit::theme::ButtonKind;
use imgui_kit::tokens::{color, size, space};
use imgui_kit::widgets::{self as w, TagKind};

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
    // new components
    pub combo_idx: usize,
    pub count: i32,
    pub notes: String,
    pub toggles: [bool; 2],
    pub seg: usize,
    pub row_selected: Option<usize>,
}

impl Default for Gallery {
    fn default() -> Self {
        Self {
            login: "player_one".into(),
            password: String::new(),
            proxy: String::new(),
            path: r"C:\Games\MyGame".into(),
            region: Region::Eu,
            switches: [true, false, true, false],
            checks: [true, false, false],
            volume: 65.0,
            delay: 250.0,
            raw: 0.4,
            disabled: true,
            events: Vec::new(),
            map: MapPage::default(),
            combo_idx: 0,
            count: 1500,
            notes: "Multi-line notes…".into(),
            toggles: [true, false],
            seg: 0,
            row_selected: Some(1),
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
    pub fn draw(&mut self, ui: &Ui, kit: &Kit, tab: usize) {
        match tab {
            1 => self.accounts(ui, kit),
            2 => self.instances(ui, kit),
            3 => self.proxies(ui, kit),
            4 => self.resources(ui, kit),
            5 => self.tools(ui, kit),
            7 => self.map.draw(ui, kit),
            _ => self.settings(ui, kit),
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
    fn accounts(&mut self, ui: &Ui, kit: &Kit) {
        let grid = Grid::default();

        w::section(ui, kit, "Cards · 3 × span 4");
        let switches = &mut self.switches;
        let mut clicked: Vec<String> = Vec::new();
        grid.row(ui, "cards", &[4, 4, 4], |ui, i, width| {
            let (mail, status, kind, proxy) = ACCOUNTS[i];
            w::card(ui, &format!("##card{i}"), [width, CARD_H], |ui| {
                w::text_bold(ui, kit, mail, color::FG);
                w::tag(ui, kit, kind, status);
                ui.same_line_with_spacing(0.0, space::S);
                w::status_dot(ui, status_color(kind), proxy);
                let on = &mut switches[i % 4];
                w::switch(ui, kit, "Auto-restart", on);
                if w::button_small(ui, kit, ButtonKind::Secondary, "Open") {
                    clicked.push(format!("open {mail}"));
                }
                ui.same_line();
                if w::button_small(ui, kit, ButtonKind::Danger, "Remove") {
                    clicked.push(format!("remove {mail}"));
                }
            });
        });
        for e in clicked {
            self.note(e);
        }

        grid::section_gap(ui);

        w::section(ui, kit, "Tags");
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
            w::tag(ui, kit, kind, text);
        }

        grid::section_gap(ui);

        w::section(ui, kit, "Status dots");
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

        w::section(ui, kit, "List rows · avatar · stat bars");
        for (i, (mail, _, kind, _)) in ACCOUNTS.iter().take(3).enumerate() {
            let selected = self.row_selected == Some(i);
            let clicked = w::list_row(ui, kit, &format!("row{i}"), selected, 0.0, |ui| {
                w::avatar(ui, kit, &mail[..1].to_uppercase(), 28.0, selected);
                ui.same_line_with_spacing(0.0, space::M);
                ui.group(|| {
                    w::text_bold(ui, kit, mail, color::FG);
                    w::stat_bar(ui, 0.3 + 0.2 * i as f32, color::ERR, 160.0, 0.0);
                    w::stat_bar(ui, 0.8 - 0.2 * i as f32, color::INFO, 160.0, 0.0);
                });
                ui.same_line_with_spacing(0.0, space::L);
                w::status_dot(ui, status_color(*kind), "");
            });
            if clicked {
                self.row_selected = Some(i);
            }
        }
        ui.dummy([0.0, space::S]);
        w::caption(ui, kit, "Selectable rows");
        for (i, name) in ["Wolf", "Keltir", "Orc"].iter().enumerate() {
            if w::selectable(ui, kit, name, self.seg == i) {
                self.seg = i;
            }
        }
        ui.dummy([0.0, space::S]);
        w::spinner(ui, 20.0, color::FG2);
        ui.same_line_with_spacing(0.0, space::M);
        w::text_muted(ui, "Loading…");
        ui.dummy([0.0, space::S]);
        w::panel(ui, "##empty", [0.0, 140.0], |ui| {
            w::empty_state(ui, kit, Some("○"), "No bots connected", "Enable monitoring to find clients");
        });

        grid::section_gap(ui);

        w::section(ui, kit, "Log lines");
        w::panel(ui, "##acc_log", [0.0, 7.0 * 16.0 + 2.0 * space::S], |ui| {
            let _sp = ui.push_style_var(StyleVar::ItemSpacing([space::S, 0.0]));
            w::log_line(ui, kit, 0.000, "Default line", None);
            w::log_line(ui, kit, 0.084, "Success line", Some(color::OK));
            w::log_line(ui, kit, 0.168, "Warning line", Some(color::WARN));
            w::log_line(ui, kit, 0.252, "Error line", Some(color::ERR));
            w::log_line(ui, kit, 0.336, "Info line", Some(color::INFO));
            w::log_line(ui, kit, 0.420, "Muted line", Some(color::FG3));
            for e in &self.events {
                w::log_line(ui, kit, 1.0, e, Some(color::FG2));
            }
        });
    }

    // 2 · Instances: table, progress -----------------------------------------
    fn instances(&mut self, ui: &Ui, kit: &Kit) {
        w::section(ui, kit, "Table · 32 px rows");
        let flags = TableFlags::ROW_BG | TableFlags::BORDERS_INNER_H | TableFlags::SIZING_STRETCH_PROP;
        let _pad = ui.push_style_var(StyleVar::CellPadding([size::PAD_X, 0.0]));
        if let Some(_t) = ui.begin_table_with_flags("##instances", 5, flags) {
            for col in ["Slot", "Account", "Status", "Uptime", "CPU"] {
                ui.table_setup_column(col);
            }
            {
                let _f = ui.push_font(kit.fonts.mono10);
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
                w::text_bold(ui, kit, &format!("{:02}", i + 1), color::FG);
                ui.table_next_column();
                grid::vcenter(ui, lh, size::TABLE_ROW);
                ui.text(mail);
                ui.table_next_column();
                grid::vcenter(ui, size::TAG, size::TABLE_ROW);
                w::tag(ui, kit, *kind, status);
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

        w::section(ui, kit, "Progress");
        let t = ui.time() as f32;
        let anim = (t * 0.25).fract();
        for (i, (label, frac)) in [("Idle", 0.0), ("Quarter", 0.25), ("Half", 0.5), ("Done", 1.0), ("Animated", anim)]
            .into_iter()
            .enumerate()
        {
            if i > 0 {
                ui.dummy([0.0, space::S]);
            }
            w::caption(ui, kit, label);
            ui.dummy([0.0, space::XS]);
            w::progress(ui, frac, 0.0);
        }
    }

    // 3 · Proxies: inputs, radio, switches, checkboxes, disabled -------------
    fn proxies(&mut self, ui: &Ui, kit: &Kit) {
        w::section(ui, kit, "Inputs");
        {
            let _sp = ui.push_style_var(StyleVar::ItemSpacing([space::S, space::S]));
            w::input(ui, kit, "login", "Login", &mut self.login, 320.0, false);
            w::input(ui, kit, "password", "Password (empty, shows hint)", &mut self.password, 320.0, false);
            w::input(ui, kit, "proxy", "host:port:user:pass", &mut self.proxy, 0.0, true);
        }
        ui.dummy([0.0, space::S]);
        let (browse, open) = w::path_input(ui, kit, "gal_path", &mut self.path);
        if browse {
            self.note("browse");
        }
        if open {
            self.note("open");
        }
        w::verified_line(ui, kit, true, "Verified", "MyGame");
        w::verified_line(ui, kit, false, "Not found", "check the path");

        grid::section_gap(ui);

        w::section(ui, kit, "Combo · number input · textarea");
        {
            let _sp = ui.push_style_var(StyleVar::ItemSpacing([space::S, space::S]));
            w::caption(ui, kit, "Region");
            w::combo(ui, kit, "##region_combo", &["Europe", "United States", "Asia", "Oceania"], &mut self.combo_idx, 240.0);
            w::caption(ui, kit, "Cooldown");
            w::number_input(ui, kit, "##cooldown", &mut self.count, 0, 600_000, 100, "ms", 200.0);
            w::caption(ui, kit, "Notes");
            w::textarea(ui, kit, "notes", &mut self.notes, 4, 480.0);
        }

        grid::section_gap(ui);

        w::section(ui, kit, "Radio");
        {
            let _sp = ui.push_style_var(StyleVar::ItemSpacing([space::S, 0.0]));
            w::radio(ui, kit, "Europe", Some("lowest ping"), &mut self.region, Region::Eu);
            w::radio(ui, kit, "United States", None, &mut self.region, Region::Us);
            w::radio(ui, kit, "Asia", Some("beta"), &mut self.region, Region::Asia);
        }

        grid::section_gap(ui);

        w::section(ui, kit, "Switches");
        {
            let _sp = ui.push_style_var(StyleVar::ItemSpacing([space::S, 0.0]));
            w::switch(ui, kit, "Rotate proxies", &mut self.switches[0]);
            w::switch(ui, kit, "Sticky sessions", &mut self.switches[1]);
            w::switch(ui, kit, "Check before launch", &mut self.switches[2]);
            w::switch(ui, kit, "Log traffic", &mut self.switches[3]);
        }

        grid::section_gap(ui);

        w::section(ui, kit, "Checkboxes");
        {
            let _sp = ui.push_style_var(StyleVar::ItemSpacing([space::S, 0.0]));
            w::checkbox(ui, kit, "Checked", None, &mut self.checks[0]);
            w::checkbox(ui, kit, "Unchecked with hint", Some("explains the option"), &mut self.checks[1]);
            w::checkbox(ui, kit, "Another one", None, &mut self.checks[2]);
        }

        grid::section_gap(ui);

        w::section(ui, kit, "Disabled");
        w::switch(ui, kit, "Disable the block below", &mut self.disabled);
        ui.dummy([0.0, space::S]);
        let disabled = self.disabled;
        w::disabled(ui, disabled, || {
            let _sp = ui.push_style_var(StyleVar::ItemSpacing([space::S, 0.0]));
            let mut on = true;
            w::checkbox(ui, kit, "Disabled checkbox", Some("40 % alpha"), &mut on);
            w::switch(ui, kit, "Disabled switch", &mut on);
            ui.dummy([0.0, space::S]);
            w::button(ui, kit, ButtonKind::Primary, "Disabled primary");
            ui.same_line();
            w::button(ui, kit, ButtonKind::Secondary, "Disabled secondary");
        });
    }

    // 4 · Resources: stats, dividers, sliders ----------------------------------
    fn resources(&mut self, ui: &Ui, kit: &Kit) {
        w::section(ui, kit, "Stats");
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
            w::stat(ui, kit, cap, val, unit, col);
        }

        grid::section_gap(ui);

        w::section(ui, kit, "Labeled sliders · form row 96 | stretch | 64");
        {
            let _sp = ui.push_style_var(StyleVar::ItemSpacing([space::S, 0.0]));
            w::labeled_slider(ui, kit, "Volume", &mut self.volume, 0.0, 100.0, 1.0, "%");
            w::labeled_slider(ui, kit, "Delay", &mut self.delay, 0.0, 1000.0, 10.0, "ms");
        }

        grid::section_gap(ui);

        w::section(ui, kit, "Raw slider track · 320 px");
        w::slider_track(ui, "raw", &mut self.raw, 0.0, 1.0, 0.05, 320.0);
        ui.same_line_with_spacing(0.0, space::L);
        grid::vcenter(ui, ui.text_line_height(), size::CONTROL);
        w::text_bold(ui, kit, &format!("{:.2}", self.raw), color::FG);

        grid::section_gap(ui);

        w::section(ui, kit, "Progress bound to the sliders");
        w::progress(ui, self.volume / 100.0, 320.0);
        ui.dummy([0.0, space::S]);
        w::progress(ui, self.raw, 320.0);
    }

    // 5 · Tools: buttons, banners ----------------------------------------------
    fn tools(&mut self, ui: &Ui, kit: &Kit) {
        w::section(ui, kit, "Buttons · 32 px");
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
            if w::button(ui, kit, kind, label) {
                self.note(format!("button {label}"));
            }
        }
        ui.dummy([0.0, space::S]);
        for (i, (kind, label)) in kinds.into_iter().enumerate() {
            if i > 0 {
                ui.same_line_with_spacing(0.0, space::S);
            }
            w::button_sized(ui, kit, kind, &format!("{label} 160"), [160.0, size::CONTROL]);
        }

        ui.dummy([0.0, space::M]);
        w::caption(ui, kit, "Small · 24 px");
        ui.dummy([0.0, space::XS]);
        {
            // same labels as the 32 px row: own ID scope, or the pairs share state
            let _id = ui.push_id("small");
            for (i, (kind, label)) in kinds.into_iter().enumerate() {
                if i > 0 {
                    ui.same_line_with_spacing(0.0, space::S);
                }
                w::button_small(ui, kit, kind, label);
            }
        }

        ui.dummy([0.0, space::M]);
        w::caption(ui, kit, "Icon buttons · 32 and 24 px");
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
        w::caption(ui, kit, "Disabled");
        ui.dummy([0.0, space::XS]);
        w::disabled(ui, true, || {
            let _id = ui.push_id("disabled");
            for (i, (kind, label)) in kinds.into_iter().enumerate() {
                if i > 0 {
                    ui.same_line_with_spacing(0.0, space::S);
                }
                w::button(ui, kit, kind, label);
            }
        });

        grid::section_gap(ui);

        w::section(ui, kit, "Toggle buttons · segmented · modal · tooltip");
        w::toggle_button(ui, kit, "Aggressive only", &mut self.toggles[0]);
        ui.same_line_with_spacing(0.0, space::S);
        w::toggle_button(ui, kit, "Polite hunting", &mut self.toggles[1]);
        ui.same_line_with_spacing(0.0, space::L);
        w::segmented(ui, kit, "##view", &["List", "Grid", "Map"], &mut self.seg);
        ui.dummy([0.0, space::S]);
        if w::button(ui, kit, ButtonKind::Danger, "Delete profile…") {
            w::open_modal(ui, "Delete profile?##gallery_modal");
        }
        w::tooltip_on_hover(ui, kit, "Opens a modal dialog");
        let mut confirmed = false;
        w::modal(ui, kit, "Delete profile?##gallery_modal", Some([360.0, 0.0]), |ui| {
            w::text_muted(ui, "This cannot be undone.");
            ui.dummy([0.0, space::L]);
            if w::button(ui, kit, ButtonKind::Secondary, "Cancel") {
                ui.close_current_popup();
            }
            ui.same_line_with_spacing(0.0, space::S);
            if w::button(ui, kit, ButtonKind::Danger, "Delete") {
                confirmed = true;
                ui.close_current_popup();
            }
        });
        if confirmed {
            self.note("profile deleted (modal)");
        }

        grid::section_gap(ui);

        w::section(ui, kit, "Call to action · 48 px");
        if w::cta(ui, kit, ButtonKind::Primary, "Launch 6 windows") {
            self.note("cta primary");
        }
        ui.dummy([0.0, space::S]);
        w::cta(ui, kit, ButtonKind::Secondary, "Secondary action");
        ui.dummy([0.0, space::S]);
        w::cta(ui, kit, ButtonKind::Danger, "Stop everything");

        grid::section_gap(ui);

        w::section(ui, kit, "Banners");
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
            if w::banner(ui, kit, kind, text, action) {
                self.note(format!("banner {text}"));
            }
        }
    }

    // 6 · Settings: grid rows, form rows, text styles, panels ------------------
    fn settings(&mut self, ui: &Ui, kit: &Kit) {
        let grid = Grid::default();

        w::section(ui, kit, "Animation");
        {
            let _sp = ui.push_style_var(StyleVar::ItemSpacing([space::S, 0.0]));
            let mut st = kit.anim.settings();
            let mut changed = false;
            changed |= w::switch(ui, kit, "Controls: checkbox, radio, switch", &mut st.controls);
            changed |= w::switch(ui, kit, "Tabs: sliding highlight", &mut st.tabs);
            changed |= w::switch(ui, kit, "Pages: fade and slide on tab change", &mut st.pages);
            changed |= w::labeled_slider(ui, kit, "Scale", &mut st.scale, 0.25, 4.0, 0.25, "×");
            if changed {
                kit.anim.set(st);
            }
        }

        grid::section_gap(ui);

        w::section(ui, kit, "Accordion · tabs");
        w::accordion_section(ui, kit, "Safety", None, true, |ui| {
            w::checkbox(ui, kit, "Stop on player detection", None, &mut self.checks[0]);
            w::checkbox(ui, kit, "Logout on PK", None, &mut self.checks[1]);
        });
        w::accordion_section(ui, kit, "Advanced settings", Some("◆"), false, |ui| {
            w::text_muted(ui, "Hidden until opened; state is kept per imgui ID.");
        });
        ui.dummy([0.0, space::S]);
        w::tabs(ui, kit, "##demo_tabs", &["List", "Map", "Logs"], |ui, idx| {
            w::text_muted(ui, &format!("Contents of tab {idx} (stateful tabs)"));
        });
        w::divider(ui);

        grid::section_gap(ui);

        w::section(ui, kit, "Grid row · spans 3 / 3 / 6");
        grid.row(ui, "spans", &[3, 3, 6], |ui, i, width| {
            w::panel(ui, &format!("##span{i}"), [width, 56.0], |ui| {
                w::caption(ui, kit, &format!("span {}", [3, 3, 6][i]));
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

        w::section(ui, kit, "Form rows · 96 | stretch | 64");
        {
            let _sp = ui.push_style_var(StyleVar::ItemSpacing([space::S, 0.0]));
            grid::form_row(ui, "##fr_login", |ui, cell, _w| match cell {
                grid::FormCell::Label => {
                    grid::vcenter(ui, ui.text_line_height(), size::CONTROL);
                    w::caption(ui, kit, "Login");
                }
                grid::FormCell::Control => {
                    w::input(ui, kit, "fr_login", "Login", &mut self.login, 0.0, false);
                }
                grid::FormCell::Value => {
                    grid::vcenter(ui, size::TAG, size::CONTROL);
                    w::tag(ui, kit, TagKind::Ok, "ok");
                }
            });
            grid::form_row(ui, "##fr_region", |ui, cell, _w| match cell {
                grid::FormCell::Label => {
                    grid::vcenter(ui, ui.text_line_height(), size::CONTROL);
                    w::caption(ui, kit, "Region");
                }
                grid::FormCell::Control => {
                    w::radio(ui, kit, "EU", None, &mut self.region, Region::Eu);
                    ui.same_line_with_spacing(0.0, space::L);
                    w::radio(ui, kit, "US", None, &mut self.region, Region::Us);
                    ui.same_line_with_spacing(0.0, space::L);
                    w::radio(ui, kit, "Asia", None, &mut self.region, Region::Asia);
                }
                grid::FormCell::Value => {
                    grid::vcenter(ui, ui.text_line_height(), size::CONTROL);
                    w::text_muted(ui, &format!("{:?}", self.region));
                }
            });
        }

        grid::section_gap(ui);

        w::section(ui, kit, "Text styles");
        {
            let _sp = ui.push_style_var(StyleVar::ItemSpacing([space::S, space::XS]));
            w::caption(ui, kit, "Caption · 10 px semibold tracked");
            ui.text("Body · 13 px regular");
            w::text_bold(ui, kit, "Label · 13 px bold", color::FG);
            w::text_muted(ui, "Muted · fg-2");
            ui.text_colored(color::FG3, "Disabled · fg-3");
            {
                let _f = ui.push_font(kit.fonts.mono12);
                ui.text("Small · 12 px for logs and cells");
            }
            {
                let _f = ui.push_font(kit.fonts.mono16b);
                ui.text("Title · 16 px bold");
            }
            {
                let _f = ui.push_font(kit.fonts.mono20b);
                ui.text("Display · 20 px bold · 1280");
            }
            ui.text("Body with an inline hint");
            w::hint_inline(ui, kit, "extra compatibility");
        }

        grid::section_gap(ui);

        w::section(ui, kit, "Panel header + panel");
        w::panel_header(ui, kit, "Events", &[w::button_small_width(ui, kit, "Clear")], |ui| {
            if w::button_small(ui, kit, ButtonKind::Secondary, "Clear") {
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
                w::log_line(ui, kit, i as f32 * 0.1, e, None);
            }
        });
    }
}

fn status_color(kind: TagKind) -> imgui_kit::tokens::Rgba {
    match kind {
        TagKind::Ok => color::OK,
        TagKind::Warn => color::WARN,
        TagKind::Err => color::ERR,
        TagKind::Info => color::INFO,
        TagKind::Solid => color::FG,
        TagKind::Neutral => color::FG3,
    }
}
