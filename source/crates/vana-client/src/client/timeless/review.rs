//! Timeless XI: the Scene Review panel on the Select screen (owner, 2026-11-21), a testing tool for the vignettes
//! (client/timeless/vignette.rs). Pick any zone's scene and any of the actions (walk, town stroll, fishing, standing)
//! on demand, play it with the highlighted character, and sign it off: **Approved** scenes drop into their own
//! collapsed list; scenes marked **Needs a new location** (with a note) go to a re-evaluation list. Both are kept in
//! `timeless-vignette-review.json` beside config.json, which is what Claude reads back for the next round.
//!
//! Also: pause, restart and play speed; the hour and the weather live; the filter (the vignettes' stronger one, the
//! title's, or off); the camera's angle off the character; next / previous scene and "next to review"; auto-advance
//! after a verdict; a whole zone sent back at once; and Ctrl+Shift+R to hide or show the panel.

use super::scout::{self, Scene};
use super::vignette::spots;
use egui::{Color32, RichText};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// What a scene is played as.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    Walk,
    Town,
    Fish,
    Stand,
}

impl Action {
    pub const ALL: [Action; 4] = [Action::Walk, Action::Town, Action::Fish, Action::Stand];

    pub fn label(self) -> &'static str {
        match self {
            Action::Walk => "Walk",
            Action::Town => "Town stroll",
            Action::Fish => "Fishing",
            Action::Stand => "Standing",
        }
    }
}

/// Where a scene comes from: the shipped table or a place recorded with /vigspot.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    Table,
    Recorded,
}

/// One scene in the catalogue.
#[derive(Clone, Debug)]
pub struct Item {
    pub zone: u16,
    /// "walk" or "fish": which list it sits in.
    pub fishing: bool,
    pub source: Source,
    pub index: usize,
    pub scene: Scene,
    pub note: String,
}

impl Item {
    /// Stable across table rebuilds: the zone, the list and the spot.
    pub fn id(&self) -> String {
        format!(
            "{}:{}:{:.1},{:.1}",
            self.zone,
            if self.fishing { "fish" } else { "walk" },
            self.scene.at[0],
            self.scene.at[2]
        )
    }

    pub fn label(&self) -> String {
        let what = if self.fishing { "Fishing spot" } else { "Walk" };
        match self.source {
            Source::Table => format!("{what} {}", self.index + 1),
            Source::Recorded => {
                format!(
                    "{what} (yours){}",
                    if self.note.is_empty() { String::new() } else { format!(": {}", self.note) }
                )
            }
        }
    }
}

/// What the Select screen is asked to show in place of the character's own scene.
#[derive(Clone, Debug, PartialEq)]
pub struct Choice {
    pub zone: u16,
    pub action: Action,
    pub scene: Scene,
    pub fishing_list: bool,
}

impl Choice {
    pub fn key(&self) -> String {
        format!("review:{}:{:?}:{:?}:{:?}", self.zone, self.action, self.scene.at, self.scene.dir)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Approved,
    Reevaluate,
}

/// A verdict, as saved for the next round.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entry {
    pub id: String,
    pub zone: u16,
    pub zone_name: String,
    pub list: String,
    pub source: Source,
    pub index: usize,
    pub scene: Scene,
    /// The action it was watched as.
    pub action: Action,
    pub status: Status,
    #[serde(default)]
    pub note: String,
    pub hour: f32,
    pub weather: String,
    #[serde(default)]
    pub character: String,
    /// The filter it was judged under ("current", "title", "off") and its strength (1 = exact).
    #[serde(default)]
    pub filter: String,
    #[serde(default)]
    pub filter_strength: f32,
    /// Unix seconds.
    pub updated: u64,
}

#[derive(Default, Serialize, Deserialize)]
struct File {
    #[serde(default)]
    entries: Vec<Entry>,
    /// Whole zones sent back for new places: zone -> note.
    #[serde(default)]
    zones: BTreeMap<String, String>,
}

pub fn path() -> std::path::PathBuf {
    crate::config::config_path().with_file_name("timeless-vignette-review.json")
}

pub fn zone_name(zone: u16) -> String {
    static NAMES: std::sync::LazyLock<std::collections::HashMap<String, String>> = std::sync::LazyLock::new(|| {
        serde_json::from_str(include_str!("../../../assets/timeless/zone_names.json")).unwrap_or_default()
    });
    NAMES.get(&zone.to_string()).cloned().unwrap_or_else(|| format!("Zone {zone}"))
}

/// The filter over the vignette.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Filter {
    #[default]
    Stronger,
    Title,
    Off,
}

/// The panel's state, live controls and the saved verdicts.
pub struct Review {
    pub open: bool,
    pub hidden: bool,
    /// The scene and action shown in place of the character's own; None: their own.
    pub choice: Option<Choice>,
    /// The catalogue item playing (an index into `items`).
    pub at: Option<usize>,
    pub items: Vec<Item>,
    pub paused: bool,
    pub speed: f32,
    pub restart: bool,
    /// The hour and weather in place of the record's (None: as recorded).
    pub hour: Option<f32>,
    pub weather: Option<String>,
    pub filter: Filter,
    /// The filter's strength: 0 nothing, 1 exactly as it is, 2 heavy.
    pub strength: f32,
    pub auto_next: bool,
    pub note: String,
    /// egui had the keyboard (the note field) last frame: the Select screen's keys wait.
    pub typing: bool,
    /// Told by the stage each frame: what is playing.
    pub playing: String,
    pub camera_deg: f32,
    entries: Vec<Entry>,
    zones: BTreeMap<String, String>,
    status: String,
    loaded: bool,
}

impl Default for Review {
    fn default() -> Self {
        Self {
            open: false,
            hidden: false,
            choice: None,
            at: None,
            items: Vec::new(),
            paused: false,
            speed: 1.0,
            restart: false,
            hour: None,
            weather: None,
            filter: Filter::Stronger,
            strength: 1.0,
            auto_next: true,
            note: String::new(),
            typing: false,
            playing: String::new(),
            camera_deg: 0.0,
            entries: Vec::new(),
            zones: BTreeMap::new(),
            status: String::new(),
            loaded: false,
        }
    }
}

impl Review {
    /// The catalogue (the table's scenes and your /vigspot places, by zone name) and the saved verdicts.
    fn load(&mut self) {
        if self.loaded {
            return;
        }
        self.loaded = true;
        let mut items = Vec::new();
        let mut zones: Vec<u16> = scout::table_zones();
        for s in spots::load() {
            if !zones.contains(&s.zone) {
                zones.push(s.zone);
            }
        }
        zones.sort_by_key(|z| zone_name(*z));
        for z in zones {
            for fishing in [false, true] {
                for (i, scene) in spots::scenes(z, fishing).into_iter().enumerate() {
                    let note = spots::load()
                        .into_iter()
                        .filter(|s| s.zone == z && (s.kind == spots::SpotKind::Fish) == fishing)
                        .nth(i)
                        .map(|s| s.note)
                        .unwrap_or_default();
                    items.push(Item { zone: z, fishing, source: Source::Recorded, index: i, scene, note });
                }
                if let Some(t) = scout::table(z) {
                    let list = if fishing { t.fish } else { t.walk };
                    for (i, scene) in list.into_iter().enumerate() {
                        items.push(Item {
                            zone: z,
                            fishing,
                            source: Source::Table,
                            index: i,
                            scene,
                            note: String::new(),
                        });
                    }
                }
            }
        }
        self.items = items;
        let file: File =
            std::fs::read_to_string(path()).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();
        self.entries = file.entries;
        self.zones = file.zones;
    }

    fn save(&mut self) {
        let file = File { entries: self.entries.clone(), zones: self.zones.clone() };
        self.status = match serde_json::to_string_pretty(&file).map(|s| std::fs::write(path(), s + "\n")) {
            Ok(Ok(())) => "Saved.".into(),
            Ok(Err(e)) => format!("Couldn't save: {e}"),
            Err(e) => format!("Couldn't save: {e}"),
        };
    }

    fn status_of(&self, id: &str) -> Option<&Entry> {
        self.entries.iter().find(|e| e.id == id)
    }

    /// Play catalogue item `i` as `action` (its own list's action when None).
    fn play(&mut self, i: usize, action: Option<Action>) {
        let Some(it) = self.items.get(i) else { return };
        let action = action.unwrap_or(if it.fishing {
            Action::Fish
        } else if super::vignette::is_town(it.zone) {
            Action::Town
        } else {
            Action::Walk
        });
        self.choice = Some(Choice { zone: it.zone, action, scene: it.scene, fishing_list: it.fishing });
        self.at = Some(i);
        self.note = self.status_of(&it.id()).map(|e| e.note.clone()).unwrap_or_default();
    }

    /// The next item after `from` (wrapping); `unreviewed`: only those without a verdict.
    fn next(&self, from: Option<usize>, step: isize, unreviewed: bool) -> Option<usize> {
        let n = self.items.len() as isize;
        if n == 0 {
            return None;
        }
        let start = from.map_or(if step > 0 { -1 } else { 0 }, |i| i as isize);
        (1..=n)
            .map(|k| (start + k * step).rem_euclid(n) as usize)
            .find(|&i| !unreviewed || self.status_of(&self.items[i].id()).is_none())
    }

    fn verdict(&mut self, status: Status, character: &str) {
        let Some(i) = self.at else { return };
        let Some(it) = self.items.get(i).cloned() else { return };
        let id = it.id();
        self.entries.retain(|e| e.id != id);
        self.entries.push(Entry {
            id,
            zone: it.zone,
            zone_name: zone_name(it.zone),
            list: if it.fishing { "fish" } else { "walk" }.into(),
            source: it.source,
            index: it.index,
            scene: it.scene,
            action: self.choice.as_ref().map_or(Action::Walk, |c| c.action),
            status,
            note: self.note.trim().to_string(),
            hour: self.hour.unwrap_or(-1.0),
            weather: self.weather.clone().unwrap_or_default(),
            character: character.to_string(),
            filter: match self.filter {
                Filter::Stronger => "current",
                Filter::Title => "title",
                Filter::Off => "off",
            }
            .into(),
            filter_strength: self.strength,
            updated: now(),
        });
        self.save();
        if self.auto_next
            && let Some(n) = self.next(Some(i), 1, true)
        {
            self.play(n, None);
        }
    }

    fn undo(&mut self, id: &str) {
        self.entries.retain(|e| e.id != id);
        self.save();
    }

    /// Ctrl+Shift+R shows or hides the panel.
    pub fn keys(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::R)) {
            self.hidden = !self.hidden;
        }
    }

    /// The panel. `character`: the highlighted character's name (saved with a verdict).
    pub fn ui(&mut self, ctx: &egui::Context, character: &str) {
        self.keys(ctx);
        self.typing = ctx.egui_wants_keyboard_input();
        if self.hidden {
            return;
        }
        let mut open = self.open;
        let r = egui::Window::new("Scene Review")
            .default_pos([12.0, 12.0])
            .default_width(360.0)
            .collapsible(true)
            .resizable(true)
            .default_open(false)
            .show(ctx, |ui| {
                self.load();
                self.body(ui, character);
            });
        if let Some(r) = r {
            open = r.inner.is_some();
        }
        self.open = open;
    }

    fn body(&mut self, ui: &mut egui::Ui, character: &str) {
        let approved = self.entries.iter().filter(|e| e.status == Status::Approved).count();
        let reevaluate = self.entries.iter().filter(|e| e.status == Status::Reevaluate).count();
        let total = self.items.len();
        ui.label(format!(
            "{approved} approved · {reevaluate} to re-evaluate · {} to review (of {total})",
            total.saturating_sub(approved + reevaluate)
        ));
        ui.separator();

        // What is playing.
        match (&self.choice, self.at.and_then(|i| self.items.get(i))) {
            (Some(c), Some(it)) => {
                ui.label(RichText::new(format!("{} · {}", zone_name(c.zone), it.label())).strong());
                if let Some(e) = self.status_of(&it.id()) {
                    ui.label(match e.status {
                        Status::Approved => RichText::new("✔ Approved").color(Color32::from_rgb(130, 200, 140)),
                        Status::Reevaluate => {
                            RichText::new("✖ Needs a new location").color(Color32::from_rgb(230, 140, 120))
                        }
                    });
                }
            }
            _ => {
                ui.label(RichText::new("The highlighted character's own scene").strong());
            }
        }
        if !self.playing.is_empty() {
            ui.label(RichText::new(&self.playing).small());
        }
        ui.horizontal(|ui| {
            if ui.button("◀ Prev").clicked()
                && let Some(i) = self.next(self.at, -1, false)
            {
                self.play(i, None);
            }
            if ui.button("Next ▶").clicked()
                && let Some(i) = self.next(self.at, 1, false)
            {
                self.play(i, None);
            }
            if ui.button("Next to review ▶▶").on_hover_text("The next scene without a verdict").clicked()
                && let Some(i) = self.next(self.at, 1, true)
            {
                self.play(i, None);
            }
            if ui.button("Own scene").on_hover_text("Back to the character's own vignette").clicked() {
                self.choice = None;
                self.at = None;
            }
        });

        // The action.
        ui.horizontal(|ui| {
            ui.label("Action:");
            for a in Action::ALL {
                let on = self.choice.as_ref().is_some_and(|c| c.action == a);
                if ui.selectable_label(on, a.label()).clicked() {
                    match &mut self.choice {
                        Some(c) => c.action = a,
                        None => {
                            if let Some(i) = self.next(None, 1, false) {
                                self.play(i, Some(a));
                            }
                        }
                    }
                }
            }
        });

        // Playback.
        ui.horizontal(|ui| {
            if ui.button(if self.paused { "▶ Play" } else { "⏸ Pause" }).clicked() {
                self.paused = !self.paused;
            }
            if ui.button("⟲ Restart").on_hover_text("From the opening shot").clicked() {
                self.restart = true;
            }
            ui.label("Speed:");
            for s in [0.5f32, 1.0, 2.0, 4.0] {
                if ui.selectable_label((self.speed - s).abs() < 0.01, format!("{s}×")).clicked() {
                    self.speed = s;
                }
            }
        });

        // Look.
        ui.horizontal(|ui| {
            let mut fixed = self.hour.is_some();
            if ui.checkbox(&mut fixed, "Hour").on_hover_text("Off: the hour of the character's last session").changed()
            {
                self.hour = fixed.then_some(17.5);
            }
            if let Some(h) = &mut self.hour {
                ui.add(egui::Slider::new(h, 0.0..=23.9).step_by(0.25).suffix(" h"));
            }
        });
        ui.horizontal(|ui| {
            ui.label("Weather:");
            let shown = self.weather.clone().unwrap_or_else(|| "as recorded".into());
            egui::ComboBox::from_id_salt("tl_review_weather").selected_text(shown).show_ui(ui, |ui| {
                if ui.selectable_label(self.weather.is_none(), "as recorded").clicked() {
                    self.weather = None;
                }
                for t in crate::dat::weather::WEATHER_TAGS {
                    if ui.selectable_label(self.weather.as_deref() == Some(t), t).clicked() {
                        self.weather = Some(t.to_string());
                    }
                }
            });
        });
        ui.horizontal(|ui| {
            ui.label("Filter:");
            for (f, l, tip) in [
                (
                    Filter::Stronger,
                    "Current",
                    "The Select screen's filter as it ships now (the title's, a little stronger)",
                ),
                (Filter::Title, "Title screen", "The title screen's filter exactly"),
                (Filter::Off, "Off", "No filter at all"),
            ] {
                if ui.selectable_label(self.filter == f, l).on_hover_text(tip).clicked() {
                    self.filter = f;
                }
            }
        });
        ui.add_enabled_ui(self.filter != Filter::Off, |ui| {
            ui.horizontal(|ui| {
                ui.label("Strength:");
                ui.add(egui::Slider::new(&mut self.strength, 0.0..=2.0).step_by(0.05).custom_formatter(
                    |v, _| match v {
                        v if v < 0.001 => "nothing".into(),
                        v if (v - 1.0).abs() < 0.001 => "exact".into(),
                        v => format!("{:.0}%", v * 100.0),
                    },
                ))
                .on_hover_text("0 = nothing, 100% = the filter exactly as chosen, 200% = heavy");
                if ui.small_button("Exact").on_hover_text("Back to 100%").clicked() {
                    self.strength = 1.0;
                }
            });
        });
        ui.label(RichText::new(format!("Camera {:.0}° off the character's facing", self.camera_deg)).small());
        ui.separator();

        // The verdict.
        ui.add(
            egui::TextEdit::singleline(&mut self.note)
                .hint_text("Note (what's wrong, what to look for instead)")
                .desired_width(f32::INFINITY),
        );
        ui.horizontal(|ui| {
            let can = self.at.is_some();
            if ui.add_enabled(can, egui::Button::new("✔ Approve")).clicked() {
                self.verdict(Status::Approved, character);
            }
            if ui
                .add_enabled(can, egui::Button::new("✖ Needs a new location"))
                .on_hover_text("Goes to the re-evaluation list for a new place in this zone")
                .clicked()
            {
                self.verdict(Status::Reevaluate, character);
            }
            ui.checkbox(&mut self.auto_next, "Auto-advance");
        });
        if let Some(c) = &self.choice {
            let z = c.zone.to_string();
            let sent = self.zones.contains_key(&z);
            let label = if sent { "↺ Keep this zone's places" } else { "Send whole zone back for new places" };
            if ui
                .small_button(label)
                .on_hover_text("Every place in this zone gets re-scouted; the note goes with it")
                .clicked()
            {
                if sent {
                    self.zones.remove(&z);
                } else {
                    self.zones.insert(z, self.note.trim().to_string());
                }
                self.save();
            }
        }
        if !self.status.is_empty() {
            ui.label(RichText::new(&self.status).small());
        }
        ui.separator();

        // The lists.
        egui::ScrollArea::vertical().max_height(360.0).show(ui, |ui| {
            let mut play = None;
            egui::CollapsingHeader::new(format!("To review ({})", total.saturating_sub(approved + reevaluate)))
                .default_open(true)
                .show(ui, |ui| {
                    let mut zone = None;
                    let mut i = 0;
                    while i < self.items.len() {
                        let z = self.items[i].zone;
                        if zone == Some(z) {
                            i += 1;
                            continue;
                        }
                        zone = Some(z);
                        let idx: Vec<usize> = (i..self.items.len())
                            .take_while(|&k| self.items[k].zone == z)
                            .filter(|&k| self.status_of(&self.items[k].id()).is_none())
                            .collect();
                        if !idx.is_empty() {
                            let sent =
                                if self.zones.contains_key(&z.to_string()) { "  (whole zone sent back)" } else { "" };
                            egui::CollapsingHeader::new(format!("{} — {}{sent}", zone_name(z), idx.len()))
                                .id_salt(("tl_review_zone", z))
                                .default_open(false)
                                .show(ui, |ui| {
                                    for k in idx {
                                        let on = self.at == Some(k);
                                        if ui.selectable_label(on, self.items[k].label()).clicked() {
                                            play = Some(k);
                                        }
                                    }
                                });
                        }
                        i += 1;
                    }
                });
            for (status, title) in [(Status::Reevaluate, "Needs a new location"), (Status::Approved, "Approved")] {
                let list: Vec<Entry> = self.entries.iter().filter(|e| e.status == status).cloned().collect();
                egui::CollapsingHeader::new(format!("{title} ({})", list.len()))
                    .id_salt(("tl_review_list", title))
                    .default_open(false)
                    .show(ui, |ui| {
                        for e in list {
                            ui.horizontal(|ui| {
                                let k = self.items.iter().position(|it| it.id() == e.id);
                                let name = k.map_or_else(|| e.id.clone(), |k| self.items[k].label());
                                ui.label(format!("{} · {name}", e.zone_name));
                                if let Some(k) = k
                                    && ui.small_button("Play").clicked()
                                {
                                    play = Some(k);
                                }
                                if ui.small_button("Undo").on_hover_text("Back to To review").clicked() {
                                    self.undo(&e.id);
                                }
                            });
                            if !e.note.is_empty() {
                                ui.label(RichText::new(format!("    “{}”", e.note)).small().italics());
                            }
                        }
                    });
            }
            if !self.zones.is_empty() {
                egui::CollapsingHeader::new(format!("Zones sent back ({})", self.zones.len()))
                    .id_salt("tl_review_zones")
                    .default_open(false)
                    .show(ui, |ui| {
                        for (z, note) in self.zones.clone() {
                            let name = z.parse().map(zone_name).unwrap_or(z.clone());
                            ui.label(if note.is_empty() { name } else { format!("{name}: “{note}”") });
                        }
                    });
            }
            if let Some(k) = play {
                self.play(k, None);
            }
        });
        ui.separator();
        ui.label(
            RichText::new(format!("Verdicts: {} · Ctrl+Shift+R hides this panel", path().display())).small().weak(),
        );
    }
}

fn now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(zone: u16, x: f32) -> Item {
        Item {
            zone,
            fishing: false,
            source: Source::Table,
            index: 0,
            scene: Scene { at: [x, 0.0, 1.0], dir: [1.0, 0.0], len: 8.0, side: 1.0, score: 1.0, front: None },
            note: String::new(),
        }
    }

    #[test]
    fn next_skips_reviewed_and_wraps() {
        let mut r =
            Review { loaded: true, items: vec![item(1, 0.0), item(1, 5.0), item(2, 0.0)], ..Default::default() };
        r.entries.push(Entry {
            id: r.items[1].id(),
            zone: 1,
            zone_name: String::new(),
            list: "walk".into(),
            source: Source::Table,
            index: 1,
            scene: r.items[1].scene,
            action: Action::Walk,
            status: Status::Approved,
            note: String::new(),
            hour: 0.0,
            weather: String::new(),
            character: String::new(),
            filter: String::new(),
            filter_strength: 1.0,
            updated: 0,
        });
        assert_eq!(r.next(Some(0), 1, true), Some(2));
        assert_eq!(r.next(Some(0), 1, false), Some(1));
        assert_eq!(r.next(Some(2), 1, true), Some(0));
        assert_eq!(r.next(Some(0), -1, false), Some(2));
        assert!(r.items[0].id().starts_with("1:walk:"));
    }
}
