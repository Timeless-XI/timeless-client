//! Timeless XI: the Timeless interface's hotbars. Three bars of twelve slots on 1..0 - =, Ctrl + them and Alt + them
//! (36 slots); each slot holds a spell, an ability, a weapon skill, an item or a command line. The number row only
//! types into the chat line once it is open: closed, the keys are the hotbar's (and Ctrl / Alt no longer run the
//! game's macro palette). Saved per character beside the client's config.json. Drawn by modern/tlbar.rs.

use super::super::lua::actions::{Action, apply};
use super::super::{World, event, magic, textcmd};
use crate::inventory::ItemInfo;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Instant;

pub const BARS: usize = 3;
pub const SLOTS: usize = 12;

/// The slot keys in order, as egui names them.
pub const KEYS: [egui::Key; SLOTS] = [
    egui::Key::Num1,
    egui::Key::Num2,
    egui::Key::Num3,
    egui::Key::Num4,
    egui::Key::Num5,
    egui::Key::Num6,
    egui::Key::Num7,
    egui::Key::Num8,
    egui::Key::Num9,
    egui::Key::Num0,
    egui::Key::Minus,
    egui::Key::Equals,
];
const KEY_TEXT: [&str; SLOTS] = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "-", "="];
/// Each bar's modifier, as the slot hints show it.
const BAR_TEXT: [&str; BARS] = ["", "C", "A"];
pub const BAR_NAMES: [&str; BARS] = ["Bar 1", "Ctrl", "Alt"];

/// What a slot runs.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Slot {
    /// A spell by id. `to`: the slot's own target (a tag: <t>, <me>, <st>, <p1> ...), else the action's default.
    Spell {
        id: u16,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        to: Option<String>,
    },
    /// A job ability, pet command or weapon skill by its name-table id (weapon skills 0..0x1ff, job abilities 0x200..).
    Ability {
        id: u16,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        to: Option<String>,
    },
    /// An item by id: used on ourselves (or the slot's target), as /item would.
    Item {
        id: u16,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        to: Option<String>,
    },
    /// A command line, run as if typed in the chat line.
    Command { line: String },
    /// Engage the target, or disengage while engaged: one click into a fight.
    Attack,
    /// A Misc Book action (timeless/book.rs MISC) by its key: Wide Scan, Map, Region Info ...
    Misc { key: String },
    /// A gear set saved from what we wore: (equipment slot 0..15, item id) per piece, put on as /equip would.
    Gear { name: String, pieces: Vec<(u8, u16)> },
}

/// Each bar's pages (Shift + 1..3 for bar 1, Shift + the wheel over any bar).
pub const PAGES: usize = 3;
/// Rows as kept: every bar's pages (row = bar * PAGES + page).
const ROWS: usize = BARS * PAGES;
type Rows = [[Option<Slot>; SLOTS]; ROWS];
/// The bars shown now: each bar's page.
type Bars = [[Option<Slot>; SLOTS]; BARS];

fn to_rows(b: &Rows) -> Vec<Vec<Option<Slot>>> {
    b.iter().map(|r| r.to_vec()).collect()
}

/// Rows from the file; before version 4 (no pages) row b is bar b's first page.
fn from_rows(rows: Vec<Vec<Option<Slot>>>, version: u32) -> Rows {
    let mut out: Rows = Default::default();
    for (b, row) in rows.into_iter().enumerate() {
        let at = if version < 4 { b * PAGES } else { b };
        if at >= ROWS {
            break;
        }
        for (k, s) in row.into_iter().take(SLOTS).enumerate() {
            out[at][k] = s;
        }
    }
    out
}

/// How a bar looks: its columns (12, 6, 4 or 3: 12x1, 6x2, 4x3, 3x4) and when it shows (0 always, 1 only while
/// fighting, 2 hidden).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct BarLook {
    pub cols: u8,
    pub show: u8,
    /// The bar's keys (KEY_SETS index); none: the bar's own default (bar 1 the number row, Ctrl, Alt).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keys: Option<u8>,
}

impl Default for BarLook {
    fn default() -> Self {
        BarLook { cols: 12, show: 0, keys: None }
    }
}

/// The key sets a bar can take: its name, the slot hint's prefix, the whole key's prefix. F keys name F1..F12.
pub const KEY_SETS: [(&str, &str, &str); 7] = [
    ("1 - =", "", ""),
    ("Ctrl + 1 - =", "C", "Ctrl+"),
    ("Alt + 1 - =", "A", "Alt+"),
    ("Shift + 1 - =", "S", "Shift+"),
    ("F1 - F12", "", ""),
    ("Ctrl + F1 - F12", "C", "Ctrl+"),
    ("None (click only)", "", ""),
];
/// KEY_SETS' click-only set.
pub const NO_KEYS: u8 = 6;

/// F1..F12, as egui names them.
const FKEYS: [egui::Key; SLOTS] = [
    egui::Key::F1,
    egui::Key::F2,
    egui::Key::F3,
    egui::Key::F4,
    egui::Key::F5,
    egui::Key::F6,
    egui::Key::F7,
    egui::Key::F8,
    egui::Key::F9,
    egui::Key::F10,
    egui::Key::F11,
    egui::Key::F12,
];

fn one() -> f32 {
    1.0
}

/// The save file. Version 2: each bar shared by every job or kept per main job (by job id, "1" = WAR).
/// Version 1 (one set of bars) loads as the shared bars.
#[derive(Default, Serialize, Deserialize)]
struct SaveFile {
    version: u32,
    #[serde(default)]
    shared_on: Vec<bool>,
    /// Version 2: the shared bars; version 1: the only bars.
    #[serde(default)]
    bars: Vec<Vec<Option<Slot>>>,
    #[serde(default)]
    jobs: std::collections::BTreeMap<String, Vec<Vec<Option<Slot>>>>,
    /// Version 3: the bars locked (nothing added, moved or cleared; the bars stay where they are).
    #[serde(default)]
    locked: bool,
    /// Version 4: rows are bar * 3 + page; each bar's page now, its look; the bars' size, opacity, fading.
    #[serde(default)]
    page: Vec<usize>,
    #[serde(default)]
    looks: Vec<BarLook>,
    #[serde(default = "one")]
    scale: f32,
    #[serde(default = "one")]
    opacity: f32,
    #[serde(default)]
    fade: bool,
    /// Two-hours need a second press; the controller's cross hotbar; mouse-over casting on the party list.
    #[serde(default = "yes")]
    confirm_sp: bool,
    #[serde(default = "yes")]
    cross: bool,
    #[serde(default = "yes")]
    mouseover: bool,
}

fn yes() -> bool {
    true
}

/// The picker's tabs.
pub const TABS: [&str; 5] = ["Magic", "Abilities", "Weapon Skills", "Items", "Commands"];

/// The bars' state: every job's slots and the shared ones, the bars shown now (`bars`, for the main job), and what
/// the drawing needs between frames.
#[derive(Default)]
pub struct Hotbar {
    owner: String,
    /// The main job the shown bars are for (0 before the stats come).
    pub job: u8,
    /// Each bar is the same on every job (true) or kept per main job (false, as FFXIV's default).
    pub shared_on: [bool; BARS],
    shared: Rows,
    jobs: std::collections::BTreeMap<u8, Rows>,
    /// Each bar's page shown.
    pub page: [usize; BARS],
    /// Each bar's look; the bars' size (0.75..1.5), opacity (0.3..1), and fading while not fighting.
    pub looks: [BarLook; BARS],
    pub scale: f32,
    pub opacity: f32,
    pub fade: bool,
    /// Two-hours need a second press (`armed`: the slot pressed once, and when); the controller's cross hotbar (`cross_set`:
    /// the trigger held, 0 LT / 1 RT; `cross_prev` last frame's pad word); mouse-over casting (`party_hover`: the party
    /// window slot under the pointer).
    pub confirm_sp: bool,
    pub armed: Option<(usize, usize, Instant)>,
    pub cross: bool,
    pub cross_set: Option<usize>,
    pub cross_prev: u64,
    pub mouseover: bool,
    pub party_hover: Option<usize>,
    /// The queued press's target (a party member under the pointer when it was pressed).
    pub queued_to: Option<String>,
    /// The layout window's import field and its last message, the backups list open, the clear button armed.
    pub import_text: Option<String>,
    pub note: Option<String>,
    pub backups_open: bool,
    pub clear_armed: Option<Instant>,
    /// The layout window (the strip's gear), while open, and when it was opened or touched.
    pub layout: bool,
    pub layout_at: Option<Instant>,
    /// The bars shown now: each the shared one or this job's.
    pub bars: Bars,
    /// When each slot was last pressed (the press's shadow).
    pub pressed: [[Option<Instant>; SLOTS]; BARS],
    /// Whether each slot's recast ran last frame, and when it last came back (the ready flash).
    pub recasting: [[bool; SLOTS]; BARS],
    pub ready_at: [[Option<Instant>; SLOTS]; BARS],
    /// The slot whose picker is open, and its tab.
    pub picking: Option<(usize, usize)>,
    pub tab: usize,
    /// A slot being dragged.
    pub drag: Option<(usize, usize)>,
    /// Something carried from the Actions window or the Items list, put on the slot it is let go over.
    pub carry: Option<Slot>,
    /// The Actions window, while open.
    pub book: Option<super::book::BookState>,
    /// Locked (the padlock left of Actions): no slot added, moved or cleared, the bars not dragged.
    pub locked: bool,
    /// Escape's order: when the Actions window, the picker and the game's windows were last opened or touched
    /// (modern/tlbar.rs keeps them), so Escape closes the newest first.
    pub book_at: Option<Instant>,
    pub picker_at: Option<Instant>,
    pub game_at: Option<Instant>,
    /// The game window that had the keys last frame.
    pub game_seen: Option<&'static str>,
    /// A Misc Book menu action's window: the key, the window it opened (once seen) and when it was asked for, so a
    /// second press closes it.
    pub misc_open: Option<(String, Option<&'static str>, Instant)>,
    /// Escapes still to send the game (a Misc window closing through its windows), one a frame; the one being sent
    /// passes our own Escape handling.
    pub esc_left: u8,
    /// The window those Escapes are for (they stop once another window has the keys; the main menu it returns to
    /// closes too).
    pub esc_target: Option<&'static str>,
    pub esc_now: bool,
    pub esc_for_game: bool,
    /// Wide Scan pressed: scan and show its window, or fold it when its list is up (modern/tlbar.rs).
    pub scan_press: bool,
    /// The save file predates the bars' default places (beside the chat window, bar 1 at the bottom): forget the
    /// bars' saved places once.
    pub reset_place: bool,
    /// The bar panel being dragged last frame (its snap is worked out when it is let go).
    pub moving: Option<String>,
    /// A slot pressed while we were busy (casting, using an item, just after an action): run once we can act.
    pub queued: Option<(usize, usize, Instant)>,
    /// Our own cast or item use under way (its 0x028 start seen), and when we may act again after an action.
    pub busy: Option<Instant>,
    pub lock_until: Option<Instant>,
    /// Slots cleared by dropping them off the bars: where and when (the puff).
    pub puffs: Vec<(egui::Pos2, Instant)>,
    /// The drag copies (Shift held when it started) instead of moving.
    pub drag_copy: bool,
    /// The right-click menu: its slot and where it opened, when, and the command line being edited in it.
    pub menu: Option<(usize, usize, egui::Pos2)>,
    pub menu_at: Option<Instant>,
    pub menu_edit: Option<String>,
    /// The menu's open list: 0 none, 1 the targets, 2 the jobs to copy to.
    pub menu_sub: u8,
    /// What the menu's line is: 0 a command, 1 a new gear set's name, 2 a set's new name.
    pub menu_mode: u8,
}

/// A character's name as it goes in file names.
fn safe(name: &str) -> String {
    name.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-').collect()
}

/// The save file of a character.
fn path(name: &str) -> PathBuf {
    crate::config::config_path().with_file_name(format!("timeless-hotbar-{}.json", safe(name)))
}

/// Seconds since 1970.
fn now_secs() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

/// How many backups each character keeps.
const BACKUPS: usize = 5;

/// A character's backups (`timeless-hotbar-<name>.bak-<seconds>.json` beside the save file), newest first.
pub fn backups(name: &str) -> Vec<(PathBuf, u64)> {
    let dir = crate::config::config_path().with_file_name("");
    let prefix = format!("timeless-hotbar-{}.bak-", safe(name));
    let Ok(rd) = std::fs::read_dir(&dir) else { return Vec::new() };
    let mut out: Vec<(PathBuf, u64)> = rd
        .flatten()
        .filter_map(|e| {
            let n = e.file_name().to_string_lossy().to_string();
            let secs = n.strip_prefix(&prefix)?.strip_suffix(".json")?.parse().ok()?;
            Some((e.path(), secs))
        })
        .collect();
    out.sort_by(|a, b| b.1.cmp(&a.1));
    out
}

/// The export code's prefix.
const CODE: &str = "TLXI1.";

/// URL-safe base64 without padding (the export codes).
fn b64(data: &[u8]) -> String {
    const A: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::with_capacity(data.len() * 4 / 3 + 3);
    for c in data.chunks(3) {
        let n = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
        for k in 0..=c.len() {
            out.push(A[(n >> (18 - 6 * k) & 63) as usize] as char);
        }
    }
    out
}

fn unb64(text: &str) -> Option<Vec<u8>> {
    let val = |c: u8| -> Option<u32> {
        Some(match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'-' | b'+' => 62,
            b'_' | b'/' => 63,
            _ => return None,
        } as u32)
    };
    let bytes: Vec<u8> = text.bytes().filter(|c| !c.is_ascii_whitespace() && *c != b'=').collect();
    let mut out = Vec::with_capacity(bytes.len() * 3 / 4);
    for c in bytes.chunks(4) {
        if c.len() == 1 {
            return None;
        }
        let mut n = 0u32;
        for (k, &b) in c.iter().enumerate() {
            n |= val(b)? << (18 - 6 * k);
        }
        for k in 0..c.len() - 1 {
            out.push((n >> (16 - 8 * k)) as u8);
        }
    }
    Some(out)
}

/// What an export code holds: every bar's pages as this job has them.
#[derive(Serialize, Deserialize)]
struct Shared {
    v: u32,
    rows: Vec<Vec<Option<Slot>>>,
}

impl Hotbar {
    /// Loads the character's bars the first time they are needed (and again after a character change).
    pub fn sync(&mut self, name: &str) {
        if name.is_empty() || self.owner == name {
            return;
        }
        *self = Hotbar {
            owner: name.to_string(),
            scale: 1.0,
            opacity: 1.0,
            confirm_sp: true,
            cross: true,
            mouseover: true,
            ..Default::default()
        };
        let p = path(name);
        let Ok(text) = std::fs::read_to_string(&p) else { return };
        match serde_json::from_str::<SaveFile>(&text) {
            Ok(f) => {
                self.load(f);
                // A backup at most every six hours of play, so a bad change can be undone.
                if backups(name).first().is_none_or(|b| now_secs().saturating_sub(b.1) > 6 * 3600) {
                    self.backup();
                }
            }
            Err(e) => {
                log::error!("{}: {e}; the hotbars start empty (the file is kept as .bad)", p.display());
                let _ = std::fs::rename(&p, p.with_extension("json.bad"));
            }
        }
    }

    fn load(&mut self, f: SaveFile) {
        self.locked = f.locked;
        // Version 5 stacks the bars the other way (bar 1 at the bottom): the old places go once.
        self.reset_place = f.version < 5;
        self.shared = from_rows(f.bars, f.version);
        for (b, n) in f.page.into_iter().take(BARS).enumerate() {
            self.page[b] = n.min(PAGES - 1);
        }
        for (b, l) in f.looks.into_iter().take(BARS).enumerate() {
            let cols = if [12, 6, 4, 3].contains(&l.cols) { l.cols } else { 12 };
            let keys = l.keys.filter(|&k| (k as usize) < KEY_SETS.len());
            self.looks[b] = BarLook { cols, show: l.show.min(2), keys };
        }
        self.confirm_sp = f.confirm_sp;
        self.cross = f.cross;
        self.mouseover = f.mouseover;
        self.scale = f.scale.clamp(0.75, 1.5);
        self.opacity = f.opacity.clamp(0.3, 1.0);
        self.fade = f.fade;
        let version = f.version;
        if f.version < 2 {
            self.shared_on = [true; BARS];
        } else {
            for (b, on) in f.shared_on.into_iter().take(BARS).enumerate() {
                self.shared_on[b] = on;
            }
        }
        self.jobs = f.jobs.into_iter().filter_map(|(k, v)| Some((k.parse().ok()?, from_rows(v, version)))).collect();
        self.refresh();
    }

    fn file(&self) -> SaveFile {
        SaveFile {
            locked: self.locked,
            version: 5,
            page: self.page.to_vec(),
            looks: self.looks.to_vec(),
            scale: self.scale,
            opacity: self.opacity,
            fade: self.fade,
            confirm_sp: self.confirm_sp,
            cross: self.cross,
            mouseover: self.mouseover,
            shared_on: self.shared_on.to_vec(),
            bars: to_rows(&self.shared),
            jobs: self.jobs.iter().map(|(j, b)| (j.to_string(), to_rows(b))).collect(),
        }
    }

    /// A copy of the bars as saved now, kept beside the save file (the newest few).
    pub fn backup(&self) {
        if self.owner.is_empty() {
            return;
        }
        let mut at = now_secs();
        // Two in one second: the next second's name.
        while backups(&self.owner).iter().any(|b| b.1 == at) {
            at += 1;
        }
        let p = crate::config::config_path().with_file_name(format!("timeless-hotbar-{}.bak-{at}.json", safe(&self.owner)));
        match serde_json::to_string_pretty(&self.file()) {
            Ok(text) => {
                if let Err(e) = std::fs::write(&p, text) {
                    log::error!("{}: {e}", p.display());
                }
            }
            Err(e) => log::error!("hotbar backup: {e}"),
        }
        for (old, _) in backups(&self.owner).into_iter().skip(BACKUPS) {
            let _ = std::fs::remove_file(old);
        }
    }

    /// The bars from a backup (what they are now backed up first). False when it can't be read.
    pub fn restore(&mut self, from: &std::path::Path) -> bool {
        let Ok(f) = std::fs::read_to_string(from).map_err(|_| ()).and_then(|t| serde_json::from_str::<SaveFile>(&t).map_err(|_| ()))
        else {
            return false;
        };
        self.backup();
        let (job, layout, layout_at) = (self.job, self.layout, self.layout_at);
        self.load(f);
        self.reset_place = false;
        (self.job, self.layout, self.layout_at) = (job, layout, layout_at);
        self.refresh();
        self.save();
        true
    }

    /// Every bar's pages as this job has them (the shared rows where a bar is shared), as a code to share.
    pub fn export(&self) -> String {
        let mine = self.jobs.get(&self.job).cloned().unwrap_or_default();
        let rows: Vec<Vec<Option<Slot>>> = (0..ROWS)
            .map(|r| if self.shared_on[r / PAGES] { self.shared[r].to_vec() } else { mine[r].to_vec() })
            .collect();
        let json = serde_json::to_vec(&Shared { v: 1, rows }).unwrap_or_default();
        use std::io::Write;
        let mut z = flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::best());
        let _ = z.write_all(&json);
        format!("{CODE}{}", b64(&z.finish().unwrap_or_default()))
    }

    /// A code from `export` onto this job's bars (a shared bar's rows go to the shared bar), backed up first. The
    /// slots it filled, or why not.
    pub fn import(&mut self, code: &str) -> Result<usize, String> {
        if self.locked {
            return Err("Unlock the bars first.".into());
        }
        let body = code.trim().strip_prefix(CODE).ok_or("That isn't a Timeless hotbar code.")?;
        let raw = unb64(body).ok_or("The code is damaged (copy all of it).")?;
        use std::io::Read;
        let mut json = Vec::new();
        flate2::read::DeflateDecoder::new(&raw[..])
            .take(1 << 20)
            .read_to_end(&mut json)
            .map_err(|_| "The code is damaged (copy all of it).")?;
        let shared: Shared = serde_json::from_slice(&json).map_err(|_| "The code is damaged (copy all of it).")?;
        let rows = from_rows(shared.rows, 4);
        self.backup();
        let mut n = 0;
        self.seed(self.job);
        for (r, row) in rows.iter().enumerate() {
            n += row.iter().filter(|s| s.is_some()).count();
            if self.shared_on[r / PAGES] {
                self.shared[r] = row.clone();
            } else if let Some(j) = self.jobs.get_mut(&self.job) {
                j[r] = row.clone();
            }
        }
        self.recasting = Default::default();
        self.refresh();
        self.save();
        Ok(n)
    }

    /// This job's own bars emptied, every page (shared bars stay), backed up first.
    pub fn clear_job(&mut self) {
        if self.locked {
            return;
        }
        self.backup();
        self.seed(self.job);
        if let Some(j) = self.jobs.get_mut(&self.job) {
            for r in 0..ROWS {
                if !self.shared_on[r / PAGES] {
                    j[r] = Default::default();
                }
            }
        }
        self.refresh();
        self.save();
    }

    /// The padlock: locks or unlocks the bars, saved.
    pub fn set_locked(&mut self, on: bool) {
        self.locked = on;
        self.picking = None;
        self.drag = None;
        self.carry = None;
        self.save();
    }

    pub(in crate::client) fn save(&self) {
        if self.owner.is_empty() {
            return;
        }
        let p = path(&self.owner);
        match serde_json::to_string_pretty(&self.file()) {
            Ok(text) => {
                if let Err(e) = std::fs::write(&p, text) {
                    log::error!("{}: {e}", p.display());
                }
            }
            Err(e) => log::error!("hotbar save: {e}"),
        }
    }

    /// The shown bars again: each the shared one or this job's.
    fn refresh(&mut self) {
        let mine = self.jobs.get(&self.job).cloned().unwrap_or_default();
        for b in 0..BARS {
            let row = self.row(b);
            self.bars[b] = if self.shared_on[b] { self.shared[row].clone() } else { mine[row].clone() };
        }
    }

    /// The kept row a bar shows: its page now.
    fn row(&self, bar: usize) -> usize {
        bar * PAGES + self.page[bar].min(PAGES - 1)
    }

    /// A bar's page (saved; the slots' shadows and flashes start over).
    pub fn set_page(&mut self, bar: usize, page: usize) {
        if bar >= BARS || page >= PAGES || self.page[bar] == page {
            return;
        }
        self.page[bar] = page;
        self.recasting[bar] = Default::default();
        self.pressed[bar] = Default::default();
        self.ready_at[bar] = Default::default();
        if self.queued.is_some_and(|(b, _, _)| b == bar) {
            self.queued = None;
        }
        self.refresh();
        self.save();
    }

    /// Our main job now: a change shows that job's bars (the shared ones stay).
    pub fn set_job(&mut self, job: u8) {
        if job == 0 || job == self.job {
            return;
        }
        self.job = job;
        self.seed(job);
        self.refresh();
        self.recasting = Default::default();
        self.pressed = Default::default();
        self.picking = None;
        self.drag = None;
        self.carry = None;
    }

    /// A job seen for the first time starts with Attack in its first slot, as FFXIV's bars do.
    fn seed(&mut self, job: u8) {
        if !self.jobs.contains_key(&job) {
            let mut seeded: Rows = Default::default();
            if !self.shared_on[0] {
                seeded[0][0] = Some(Slot::Attack);
            }
            self.jobs.insert(job, seeded);
        }
    }

    /// Puts an action on the first empty slot (bar 1 left to right, then Ctrl's, then Alt's): where, or None (locked
    /// or every slot full).
    pub fn add_next(&mut self, s: Slot) -> Option<(usize, usize)> {
        if self.locked {
            return None;
        }
        let at = (0..BARS).flat_map(|b| (0..SLOTS).map(move |k| (b, k))).find(|&(b, k)| self.bars[b][k].is_none())?;
        self.set(at.0, at.1, Some(s));
        Some(at)
    }

    /// A slot copied onto another (Shift + drag): the source stays.
    pub fn copy(&mut self, from: (usize, usize), to: (usize, usize)) {
        if from == to || from.0 >= BARS || to.0 >= BARS || from.1 >= SLOTS || to.1 >= SLOTS || self.locked {
            return;
        }
        let Some(s) = self.bars[from.0][from.1].clone() else { return };
        self.set(to.0, to.1, Some(s));
    }

    /// What another job's own bar holds at a place.
    pub fn job_slot(&self, job: u8, bar: usize, slot: usize) -> Option<&Slot> {
        self.jobs.get(&job)?.get(self.row(bar))?.get(slot)?.as_ref()
    }

    /// A slot copied onto another job's own bar at the same place (a shared row is the same on every job already).
    pub fn copy_to_job(&mut self, bar: usize, slot: usize, job: u8) -> bool {
        if self.locked || bar >= BARS || slot >= SLOTS || job == 0 || job == self.job || self.shared_on[bar] {
            return false;
        }
        let s = self.bars[bar][slot].clone();
        self.seed(job);
        let row = self.row(bar);
        if let Some(b) = self.jobs.get_mut(&job) {
            b[row][slot] = s;
        }
        self.save();
        true
    }

    /// A slot's own target (None: the action's default). Spells, abilities and items only.
    pub fn set_target(&mut self, bar: usize, slot: usize, to: Option<String>) {
        let Some(mut s) = self.bars.get(bar).and_then(|b| b.get(slot)).cloned().flatten() else { return };
        match &mut s {
            Slot::Spell { to: t, .. } | Slot::Ability { to: t, .. } | Slot::Item { to: t, .. } => *t = to,
            _ => return,
        }
        self.set(bar, slot, Some(s));
    }

    /// Where a slot is kept: the shared bar or this job's.
    fn store(&mut self, bar: usize) -> &mut [Option<Slot>; SLOTS] {
        let row = self.row(bar);
        if self.shared_on[bar] { &mut self.shared[row] } else { &mut self.jobs.entry(self.job).or_default()[row] }
    }

    pub fn set(&mut self, bar: usize, slot: usize, s: Option<Slot>) {
        if bar < BARS && slot < SLOTS && !self.locked {
            self.store(bar)[slot] = s;
            self.recasting[bar][slot] = false;
            self.refresh();
            self.save();
        }
    }

    pub fn swap(&mut self, a: (usize, usize), b: (usize, usize)) {
        if a == b || a.0 >= BARS || b.0 >= BARS || a.1 >= SLOTS || b.1 >= SLOTS || self.locked {
            return;
        }
        let (x, y) = (self.bars[a.0][a.1].clone(), self.bars[b.0][b.1].clone());
        self.store(a.0)[a.1] = y;
        self.store(b.0)[b.1] = x;
        self.recasting[a.0][a.1] = false;
        self.recasting[b.0][b.1] = false;
        self.refresh();
        self.save();
    }

    /// A bar between shared and this job's. What it showed stays: switching to shared fills an empty shared bar from
    /// this job's; switching back fills this job's empty bar from the shared one.
    pub fn toggle_shared(&mut self, bar: usize) {
        if bar >= BARS || self.locked {
            return;
        }
        let shown = self.bars[bar].clone();
        let empty = |r: &[Option<Slot>; SLOTS]| r.iter().all(Option::is_none);
        self.shared_on[bar] = !self.shared_on[bar];
        let target = self.store(bar);
        if empty(target) {
            *target = shown;
        }
        self.refresh();
        self.save();
    }
}

/// Loads the character's bars and follows the main job (every frame the bars are drawn, and before a key runs one).
pub fn sync(w: &mut World) {
    let name = w.name.clone();
    w.tl_bar.sync(&name);
    let job = w.stats.as_ref().map_or(0, |s| s.main.0);
    w.tl_bar.set_job(job);
}

/// A job's short name ("WAR"), for the bar labels.
pub fn job_name(job: u8) -> &'static str {
    super::super::items::JOBS.get(job as usize).copied().filter(|s| !s.is_empty()).unwrap_or("Job")
}

/// The Timeless interface is on and the world is up.
pub fn active(w: &World) -> bool {
    w.timeless && w.tl_bars_on && w.modern && w.in_scene()
}

/// A bar's key set: its own, else the bar's default (bar 1 the number row, then Ctrl, then Alt).
pub fn key_set(looks: &[BarLook; BARS], bar: usize) -> u8 {
    looks[bar.min(BARS - 1)].keys.unwrap_or(bar.min(BARS - 1) as u8)
}

/// A slot's key hint: "1", "C1", "A=", "S5", "F3", "CF3"; empty for a click-only bar.
pub fn label_of(set: u8, slot: usize) -> String {
    let k = slot.min(SLOTS - 1);
    match set {
        4 => format!("F{}", k + 1),
        5 => format!("CF{}", k + 1),
        NO_KEYS => String::new(),
        s => format!("{}{}", KEY_SETS[s as usize].1, KEY_TEXT[k]),
    }
}

/// The slot hint with the default key sets.
pub fn label(bar: usize, slot: usize) -> String {
    let _ = BAR_TEXT;
    label_of(bar.min(BARS - 1) as u8, slot)
}

/// The key set and slot a key with these modifiers is: none, Ctrl, Alt or Shift with the number row; none or Ctrl
/// with F1..F12 (Ctrl + Alt, AltGr on some layouts, is none).
pub fn key_of(key: egui::Key, m: egui::Modifiers) -> Option<(u8, usize)> {
    let mods = (m.ctrl || m.command, m.alt, m.shift);
    if let Some(slot) = KEYS.iter().position(|k| *k == key) {
        let set = match mods {
            (false, false, false) => 0,
            (true, false, _) => 1,
            (false, true, _) => 2,
            (false, false, true) => 3,
            _ => return None,
        };
        return Some((set, slot));
    }
    let slot = FKEYS.iter().position(|k| *k == key)?;
    match mods {
        (false, false, false) => Some((4, slot)),
        (true, false, false) => Some((5, slot)),
        _ => None,
    }
}

/// The slot a key presses, with these bars' key sets (the first bar with that set).
pub fn key_slot_with(looks: &[BarLook; BARS], key: egui::Key, m: egui::Modifiers) -> Option<(usize, usize)> {
    let (set, slot) = key_of(key, m)?;
    (0..BARS).find(|&b| key_set(looks, b) == set).map(|b| (b, slot))
}

/// The slot a key presses with the default key sets.
#[cfg(test)]
pub fn key_slot(key: egui::Key, m: egui::Modifiers) -> Option<(usize, usize)> {
    key_slot_with(&[BarLook::default(); BARS], key, m)
}

/// Some bar is on F1..F12: those keys are the hotbars' (not the game's targeting) while the bars take keys.
pub fn masks_fkey(w: &World, code: winit::keyboard::KeyCode) -> bool {
    use winit::keyboard::KeyCode as K;
    let f = matches!(
        code,
        K::F1 | K::F2 | K::F3 | K::F4 | K::F5 | K::F6 | K::F7 | K::F8 | K::F9 | K::F10 | K::F11 | K::F12
    );
    f && takes_keys(w) && (0..BARS).any(|b| matches!(key_set(&w.tl_bar.looks, b), 4 | 5))
}

/// Whether the hotbar takes the keys now: Timeless on, the chat line and every text field closed, no event.
pub fn takes_keys(w: &World) -> bool {
    active(w) && !w.typing && !super::super::text_field_open(w) && !event::active(w) && super::super::chat_opens(w)
}

/// A key press reaching the world (client/mod.rs, with the chat line closed): Escape closes the picker; a slot key
/// runs its slot. True when the hotbar took it.
pub fn key(w: &mut World, key: egui::Key, m: egui::Modifiers, numpad: bool) -> bool {
    if key == egui::Key::Escape && active(w) {
        // An Escape we send the game (a Misc window closing) is the game's.
        if std::mem::take(&mut w.tl_bar.esc_for_game) {
            return false;
        }
        return escape(w);
    }
    if numpad || !takes_keys(w) {
        return false;
    }
    // Shift + 1..3: bar 1's page (unless a bar is on Shift + the number row).
    let shift_bar = (0..BARS).any(|b| key_set(&w.tl_bar.looks, b) == 3);
    if m.shift && !(m.ctrl || m.command || m.alt) && !shift_bar {
        let page = KEYS.iter().position(|k| *k == key).filter(|&p| p < PAGES);
        if let Some(page) = page {
            w.tl_bar.set_page(0, page);
            w.sounds.push(crate::audio::Ui::Cursor as u32);
            return true;
        }
        return false;
    }
    let Some((bar, slot)) = key_slot_with(&w.tl_bar.looks, key, m) else { return false };
    press(w, bar, slot);
    true
}

/// Escape with the chat line closed: our newest window (the picker, the Actions window) closes, unless a game window
/// was opened or touched after it; then the game's own Escape takes it (its windows close newest first themselves).
/// True when one of ours closed.
fn escape(w: &mut World) -> bool {
    let h = &w.tl_bar;
    let game = super::super::modern::focus_panel(w).is_some() || w.map.is_some();
    let ours = [(h.picking.is_some(), h.picker_at, 0u8), (h.book.is_some(), h.book_at, 1), (h.menu.is_some(), h.menu_at, 2), (h.layout, h.layout_at, 3)]
        .into_iter()
        .filter(|(open, _, _)| *open)
        .max_by_key(|(_, at, _)| *at);
    let Some((_, at, which)) = ours else { return false };
    if game && h.game_at.is_some_and(|g| at.is_none_or(|a| g > a)) {
        return false;
    }
    match which {
        0 => w.tl_bar.picking = None,
        1 => w.tl_bar.book = None,
        2 => {
            w.tl_bar.menu = None;
            w.tl_bar.menu_edit = None;
        }
        _ => {
            w.tl_bar.layout = false;
            w.tl_bar.import_text = None;
        }
    }
    w.sounds.push(crate::audio::Ui::Cancel as u32);
    true
}

/// Runs a slot (a key or a click) and starts its press shadow.
pub fn press(w: &mut World, bar: usize, slot: usize) {
    sync(w);
    let Some(s) = w.tl_bar.bars.get(bar).and_then(|b| b.get(slot)).cloned().flatten() else { return };
    // A two-hour takes a second press within a moment (the first arms it).
    if w.tl_bar.confirm_sp && two_hour(w, &s) {
        let armed = w.tl_bar.armed.is_some_and(|(b, k, t)| (b, k) == (bar, slot) && t.elapsed().as_secs_f32() < ARM);
        if !armed {
            w.tl_bar.armed = Some((bar, slot, Instant::now()));
            w.sounds.push(crate::audio::Ui::Cursor as u32);
            return;
        }
        w.tl_bar.armed = None;
    }
    // Mouse-over casting: a party member under the pointer is the target.
    let to = mouse_over(w, &s);
    // An action while we are casting or just after one waits its turn (one at a time: the newest press wins).
    if queues(&s) && busy(w) {
        if w.tl_bar.queued.is_none_or(|(b, k, _)| (b, k) != (bar, slot)) {
            w.sounds.push(crate::audio::Ui::Cursor as u32);
        }
        w.tl_bar.queued = Some((bar, slot, Instant::now()));
        w.tl_bar.queued_to = to;
        return;
    }
    w.tl_bar.pressed[bar][slot] = Some(Instant::now());
    if let Slot::Misc { key } = &s
        && misc_toggle(w, key)
    {
        return;
    }
    if queues(&s) {
        w.tl_bar.queued = None;
    }
    run_to(w, &s, to);
}

/// How long a two-hour stays armed for its second press, in seconds.
pub const ARM: f32 = 1.6;

/// A two-hour (special) ability: a job ability on recast timer 0, the one every two-hour shares.
pub fn two_hour(w: &World, s: &Slot) -> bool {
    let Slot::Ability { id, .. } = s else { return false };
    *id >= 0x200
        && ability_index(w, *id)
            .and_then(|i| w.actions.as_ref()?.abilities.get(i as usize).map(|a| a.timer == 0 && a.kind == 1))
            .unwrap_or(false)
}

/// The armed two-hour slot, while it waits for its second press.
pub fn armed(w: &World) -> Option<(usize, usize)> {
    w.tl_bar.armed.filter(|a| a.2.elapsed().as_secs_f32() < ARM).map(|a| (a.0, a.1))
}

/// Mouse-over casting: the party member under the pointer, as a target for a spell, ability or item that may go to
/// them (ourselves as <me>). None: the slot's own target or default.
fn mouse_over(w: &World, s: &Slot) -> Option<String> {
    if !w.tl_bar.mouseover || !takes_target(s) {
        return None;
    }
    let slot = w.tl_bar.party_hover?;
    let m = w.party.slots().slots.get(slot).cloned().flatten()?;
    let targets = match s {
        Slot::Spell { id, .. } => w.actions.as_ref()?.spell(*id)?.targets,
        Slot::Ability { id, .. } => w.actions.as_ref()?.abilities.get(ability_index(w, *id)? as usize)?.targets,
        _ => 0x1f,
    };
    if m.id == w.my_id {
        return (targets & 1 != 0).then(|| "<me>".to_string());
    }
    // 2 player, 4 party, 8 alliance member.
    (targets & 0xe != 0).then(|| m.id.to_string())
}

/// The slots that wait their turn: spells, abilities and weapon skills, items.
fn queues(s: &Slot) -> bool {
    matches!(s, Slot::Spell { .. } | Slot::Ability { .. } | Slot::Item { .. })
}

/// The server keeps us in an action for its animation after it lands (LSB: 2 s for nearly every spell, ability and
/// weapon skill), and refuses another until then.
const AFTER_ACTION: f32 = 2.1;
/// A cast or item use whose end never came is forgotten after this; a queued press after this.
const BUSY_MAX: f32 = 20.0;
const QUEUE_MAX: f32 = 12.0;

/// We are casting, using an item, or just finished an action.
pub fn busy(w: &World) -> bool {
    let now = Instant::now();
    w.tl_bar.busy.is_some_and(|t| now.duration_since(t).as_secs_f32() < BUSY_MAX)
        || w.casts.contains_key(&w.my_id)
        || w.tl_bar.lock_until.is_some_and(|t| now < t)
}

/// Our own action packets (combat.rs, 0x028 with us acting): a cast or item use starting, being interrupted, or an
/// action landing (then the animation's lock).
pub fn on_action(w: &mut World, category: u8, arg: u32) {
    use crate::net::action::category as c;
    let now = Instant::now();
    match category {
        c::MAGIC_START | c::ITEM_START | c::RANGED_START => {
            if arg & 0xffff == 0x6163 {
                w.tl_bar.busy = Some(now);
            } else {
                // Interrupted: free again in a moment.
                w.tl_bar.busy = None;
                w.tl_bar.lock_until = Some(now + std::time::Duration::from_secs_f32(0.6));
            }
        }
        c::MAGIC_FINISH | c::ITEM_FINISH | c::RANGED_FINISH | c::SKILL_FINISH | c::ABILITY_FINISH => {
            w.tl_bar.busy = None;
            w.tl_bar.lock_until = Some(now + std::time::Duration::from_secs_f32(AFTER_ACTION));
        }
        _ => {}
    }
}

/// Every frame the bars are drawn: the queued press runs once we can act (or is forgotten when it waited too long or
/// its slot changed).
pub fn tick(w: &mut World) {
    let Some((b, k, at)) = w.tl_bar.queued else { return };
    if at.elapsed().as_secs_f32() > QUEUE_MAX || w.tl_bar.bars[b][k].as_ref().is_none_or(|s| !queues(s)) {
        w.tl_bar.queued = None;
        return;
    }
    if busy(w) {
        return;
    }
    w.tl_bar.queued = None;
    let to = w.tl_bar.queued_to.take();
    let Some(s) = w.tl_bar.bars[b][k].clone() else { return };
    w.tl_bar.pressed[b][k] = Some(Instant::now());
    run_to(w, &s, to);
}

/// The controller's cross hotbar (client/mod.rs, each frame with the pad word): while LT (macro slot 9) or RT (13) is
/// held in the field with no window up, the d-pad (up, right, down, left) and the face buttons (the slots for the
/// active window, cancel, confirm and the menu: Y, B, A, X on the default layout) press slots 1..8 of the Ctrl bar
/// (LT) or the Alt bar (RT). Returns the pad bits it took (the game doesn't see them: no macro palette, no confirm).
pub fn cross(w: &mut World, held: u64, menu: bool) -> u64 {
    use super::super::gamepad::{bit, slot};
    let trig = bit(slot::MACRO_CTRL) | bit(slot::MACRO_ALT);
    if !active(w) || !w.tl_bar.cross || menu || w.typing || event::active(w) || held & trig == 0 {
        w.tl_bar.cross_set = None;
        w.tl_bar.cross_prev = held;
        return 0;
    }
    let set = if held & bit(slot::MACRO_CTRL) != 0 { 0 } else { 1 };
    w.tl_bar.cross_set = Some(set);
    let new = held & !w.tl_bar.cross_prev;
    w.tl_bar.cross_prev = held;
    let mut taken = trig;
    for (k, s) in CROSS.iter().enumerate() {
        taken |= bit(*s);
        if new & bit(*s) != 0 {
            press(w, 1 + set, k);
        }
    }
    taken & held
}

/// The cross hotbar's buttons in slot order: d-pad up, right, down, left; then the active-window, cancel, confirm and
/// menu slots (Y, B, A, X).
pub const CROSS: [u8; 8] = [6, 4, 7, 5, 2, 3, 0, 1];

/// A Misc Book press that closes what it opened: its menu window still up (Escapes through the game's windows,
/// one a frame), or Wide Scan (tlbar folds or opens its window). True when the press is taken here.
fn misc_toggle(w: &mut World, key: &str) -> bool {
    if key == "widescan" {
        w.tl_bar.scan_press = true;
        return true;
    }
    if !super::book::opens_window(key) {
        return false;
    }
    let focus = super::super::modern::focus_panel(w).or(w.map.is_some().then_some("map"));
    if let Some((k, Some(id), _)) = &w.tl_bar.misc_open
        && k == key
        && focus == Some(*id)
    {
        w.tl_bar.esc_left = 4;
        w.tl_bar.esc_target = Some(*id);
        w.tl_bar.misc_open = None;
        return true;
    }
    w.tl_bar.misc_open = Some((key.to_string(), None, Instant::now()));
    false
}

/// Runs what a slot holds, through the code the game's menus and text commands take (the same checks and refusals).
pub fn run(w: &mut World, s: &Slot) {
    run_to(w, s, None);
}

/// `run`, with a target over the slot's own (mouse-over casting).
pub fn run_to(w: &mut World, s: &Slot, over: Option<String>) {
    match s {
        Slot::Spell { id, to } => apply(w, None, Action::Cast(*id, over.or_else(|| to.clone()))),
        Slot::Ability { id, to } => apply(w, None, Action::Ability(*id, over.or_else(|| to.clone()))),
        Slot::Item { id, to } => {
            if let Some(n) = item_name(w, *id) {
                let t = over.as_deref().or(to.as_deref()).unwrap_or("<me>");
                textcmd::run(w, &format!("/item \"{n}\" {t}"), textcmd::Mode::Internal);
            }
        }
        Slot::Command { line } => {
            let line = line.trim().to_string();
            if !line.is_empty() {
                textcmd::run(w, &line, textcmd::Mode::Typed);
            }
        }
        Slot::Attack => {
            let line = if engaged(w) { "/attackoff" } else { "/attack <t>" };
            textcmd::run(w, line, textcmd::Mode::Internal);
        }
        Slot::Misc { key } => super::book::run_misc(w, key),
        Slot::Gear { pieces, .. } => {
            use super::super::lua::actions::Piece;
            let list = pieces.iter().map(|&(slot, id)| (slot as usize, Piece::Item(id))).collect();
            apply(w, None, Action::Equip(list));
        }
    }
}

/// We are engaged (motion status 1, as the weapon skill check reads it).
pub fn engaged(w: &World) -> bool {
    w.my_motion.status == 1
}

/// The ability table's index of a name-table id (they match for most rows).
pub fn ability_index(w: &World, id: u16) -> Option<u16> {
    let a = w.actions.as_ref()?;
    if a.abilities.get(id as usize).is_some_and(|x| x.id == id) {
        return Some(id);
    }
    a.abilities.iter().position(|x| x.id == id).map(|p| p as u16)
}

pub fn item_name(w: &World, id: u16) -> Option<String> {
    w.item_info.as_deref().and_then(|c| c.name(id)).filter(|n| !n.is_empty())
}

/// How many of an item we hold where /item finds it (the temporary items, the inventory, the wardrobes).
pub fn item_count(w: &World, id: u16) -> u32 {
    const BAGS: [u8; 10] = [3, 0, 8, 10, 11, 12, 13, 14, 15, 16];
    w.inv.slots.iter().filter(|((bag, _), (item, _))| *item == id && BAGS.contains(bag)).map(|(_, (_, n))| *n).sum()
}

/// A slot's name, for its tooltip and the picker.
pub fn name(w: &World, s: &Slot) -> String {
    match s {
        Slot::Spell { id, .. } => magic::spell_name(w, *id),
        Slot::Ability { id, .. } => ability_index(w, *id).map(|i| magic::ability_name(w, i)).unwrap_or_default(),
        Slot::Item { id, .. } => item_name(w, *id).unwrap_or_else(|| format!("Item {id}")),
        Slot::Command { line } => line.clone(),
        Slot::Attack if engaged(w) => "Disengage".into(),
        Slot::Attack => "Attack".into(),
        Slot::Misc { key } => super::book::misc_names().get(key.as_str()).copied().unwrap_or("Action").to_string(),
        Slot::Gear { name, .. } => name.clone(),
    }
}

/// A slot's recast: seconds left and the whole recast (0, 0 when none runs).
pub fn recast(w: &World, s: &Slot) -> (f32, f32) {
    match s {
        Slot::Spell { id, .. } => {
            let left = magic::spell_recast(w, *id);
            if left <= 0.0 {
                return (0.0, 0.0);
            }
            (left, w.spell_total.get(id).copied().unwrap_or(left).max(left))
        }
        Slot::Ability { id, .. } => {
            let Some(timer) = ability_index(w, *id)
                .and_then(|i| w.actions.as_ref()?.abilities.get(i as usize).map(|a| a.timer))
                .filter(|&t| t != 900)
            else {
                return (0.0, 0.0);
            };
            let left = magic::ability_recast(w, timer);
            if left <= 0.0 {
                return (0.0, 0.0);
            }
            let total = w
                .ability_total
                .get(&timer)
                .copied()
                .or_else(|| crate::dat::recasts::timer(timer).map(|s| s as f32))
                .unwrap_or(left);
            (left, total.max(left))
        }
        Slot::Item { .. } | Slot::Command { .. } | Slot::Attack | Slot::Misc { .. } | Slot::Gear { .. } => (0.0, 0.0),
    }
}

/// Whether a slot can be used now, as the game's lists dim theirs (a command always can).
pub fn usable(w: &World, s: &Slot) -> bool {
    why(w, s).is_none()
}

/// The slot's own target choices (a slot with none takes the action's default: the target, or ourselves).
pub const TARGETS: [(&str, &str); 10] = [
    ("<t>", "Current target"),
    ("<me>", "Myself"),
    ("<st>", "Pick each time"),
    ("<stpc>", "Pick a player"),
    ("<bt>", "Battle target"),
    ("<p1>", "Party member 2"),
    ("<p2>", "Party member 3"),
    ("<p3>", "Party member 4"),
    ("<p4>", "Party member 5"),
    ("<p5>", "Party member 6"),
];

/// A slot's own target tag, if it has one.
pub fn target_tag(s: &Slot) -> Option<&str> {
    match s {
        Slot::Spell { to, .. } | Slot::Ability { to, .. } | Slot::Item { to, .. } => to.as_deref(),
        _ => None,
    }
}

/// The slot can be given a target of its own.
pub fn takes_target(s: &Slot) -> bool {
    matches!(s, Slot::Spell { .. } | Slot::Ability { .. } | Slot::Item { .. })
}

/// A target tag's name in the menus ("Myself").
pub fn target_label(tag: Option<&str>) -> &'static str {
    match tag {
        None => "Automatic",
        Some(t) => TARGETS.iter().find(|(k, _)| *k == t).map_or("Custom", |(_, n)| n),
    }
}

/// A slot's whole key ("Ctrl+3", "F5"), by the bar's key set; empty for a click-only bar.
pub fn key_text_of(set: u8, slot: usize) -> String {
    let k = slot.min(SLOTS - 1);
    match set {
        4 => format!("F{}", k + 1),
        5 => format!("Ctrl+F{}", k + 1),
        NO_KEYS => String::new(),
        s => format!("{}{}", KEY_SETS[s as usize].2, KEY_TEXT[k]),
    }
}

/// A slot's whole key with the default key sets.
pub fn key_text(bar: usize, slot: usize) -> String {
    key_text_of(bar.min(BARS - 1) as u8, slot)
}

/// Why a slot can't be used now (the dimmed slot's tag and its tooltip line).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Why {
    /// Not learned, or not for the jobs and levels we have now.
    Learn,
    /// Its recast runs (the hourglass shows it).
    Recast,
    /// Asleep, petrified, stunned, charmed, terrorized.
    Act,
    Silence,
    Amnesia,
    /// Resting, mounted, fishing, sitting, or somewhere actions aren't allowed.
    Status,
    Mp,
    Tp,
    /// A weapon skill out of a fight.
    Engage,
    /// Nobody to use it on.
    Target,
    Range,
    /// None of the item left.
    Empty,
    /// No ammunition for a ranged attack.
    Ammo,
    /// Above our level now (a level sync or cap, or not there yet).
    Level,
    /// None of a gear set's pieces are with us.
    Missing,
}

impl Why {
    /// The short tag across the dimmed icon (none where the dimming says enough).
    pub fn tag(self) -> &'static str {
        match self {
            Why::Learn | Why::Recast | Why::Empty => "",
            Why::Act => "HALTED",
            Why::Silence => "SILENCE",
            Why::Amnesia => "AMNESIA",
            Why::Status => "BUSY",
            Why::Mp => "MP",
            Why::Tp => "TP",
            Why::Engage => "ENGAGE",
            Why::Target => "NO TGT",
            Why::Range => "RANGE",
            Why::Ammo => "AMMO",
            Why::Level => "LV",
            Why::Missing => "MISSING",
        }
    }

    /// The tooltip's line.
    pub fn text(self) -> &'static str {
        match self {
            Why::Learn => "Not available to your current jobs and levels.",
            Why::Recast => "Recasting.",
            Why::Act => "You can't act right now.",
            Why::Silence => "You are silenced.",
            Why::Amnesia => "You have amnesia.",
            Why::Status => "Not while resting, mounted, fishing or sitting, or in this place.",
            Why::Mp => "Not enough MP.",
            Why::Tp => "Not enough TP (weapon skills need 1000).",
            Why::Engage => "Engage a target first.",
            Why::Target => "No target to use it on.",
            Why::Range => "Your target is out of range.",
            Why::Empty => "You have none left.",
            Why::Ammo => "No ammunition equipped.",
            Why::Level => "Above your current level (or level sync).",
            Why::Missing => "None of this set's pieces are in your inventory or wardrobes.",
        }
    }
}

/// Asleep, petrified, stunned, charmed, terrorized, lullabied (as the modern lists dim).
fn halted(w: &World) -> bool {
    w.buffs.iter().any(|&b| matches!(b, 2 | 7 | 10 | 14 | 17 | 19 | 28 | 193))
}

/// Why a slot can't be used now, None when it can. Uses the same checks the game's menus refuse with.
pub fn why(w: &World, s: &Slot) -> Option<Why> {
    let has = |e: u16| w.buffs.contains(&e);
    match s {
        Slot::Spell { id, .. } => {
            if !textcmd::spell_learned(w, *id) {
                return Some(Why::Learn);
            }
            if halted(w) {
                return Some(Why::Act);
            }
            if has(6) || has(29) {
                return Some(Why::Silence);
            }
            match magic::magic_check(w, *id) {
                Some(0x58) => return Some(Why::Mp),
                Some(0x59 | 0xbf) => return Some(Why::Recast),
                Some(_) if magic::status_check(w).is_some() => return Some(Why::Status),
                Some(_) => return Some(if need_level(w, s).is_some() { Why::Level } else { Why::Learn }),
                None => {}
            }
            let targets = w.actions.as_ref().and_then(|a| a.spell(*id)).map_or(0x20, |r| r.targets);
            reach(w, s, targets, range(w, s))
        }
        Slot::Ability { id, .. } => {
            let Some(i) = ability_index(w, *id) else { return Some(Why::Learn) };
            if (0x200..0x400).contains(id) {
                let bit = (id - 0x200) as usize;
                if !w.commands_known.get(64 + bit / 8).is_some_and(|b| b & (1 << (bit % 8)) != 0) {
                    return Some(Why::Learn);
                }
            }
            if halted(w) {
                return Some(Why::Act);
            }
            if has(16) {
                return Some(Why::Amnesia);
            }
            match magic::ability_check(w, i) {
                Some(0x5c) => return Some(Why::Recast),
                Some(0x5f) => return Some(Why::Tp),
                Some(0x52) => return Some(Why::Engage),
                Some(0x58) => return Some(Why::Mp),
                Some(_) if magic::status_check(w).is_some() => return Some(Why::Status),
                Some(_) => return Some(if need_level(w, s).is_some() { Why::Level } else { Why::Learn }),
                None => {}
            }
            let targets = w.actions.as_ref().and_then(|a| a.abilities.get(i as usize)).map_or(0x20, |a| a.targets);
            reach(w, s, targets, range(w, s))
        }
        Slot::Item { id, .. } => {
            if item_count(w, *id) == 0 {
                return Some(Why::Empty);
            }
            if halted(w) {
                return Some(Why::Act);
            }
            if magic::status_check(w).is_some() {
                return Some(Why::Status);
            }
            reach(w, s, 1, None)
        }
        Slot::Attack if !engaged(w) && w.target.is_none() => Some(Why::Target),
        Slot::Gear { pieces, .. } => {
            if halted(w) {
                Some(Why::Act)
            } else if !pieces.is_empty() && gear_missing(w, pieces).len() == pieces.len() {
                Some(Why::Missing)
            } else {
                None
            }
        }
        Slot::Command { .. } | Slot::Misc { .. } if ranged(s) && ammo(w).is_none_or(|(_, n)| n == 0) => Some(Why::Ammo),
        _ => None,
    }
}

/// The level a spell or job ability needs on the main or sub job when that job has it but isn't high enough now
/// (level sync and caps included): the lowest such level. None when one of them can use it now, or neither has it.
pub fn need_level(w: &World, s: &Slot) -> Option<u8> {
    let st = w.stats.as_ref()?;
    let jobs = [st.main, st.sub];
    let need: Vec<(u8, u8)> = match s {
        Slot::Spell { id, .. } => {
            let r = w.actions.as_ref()?.spell(*id)?;
            jobs.iter()
                .filter_map(|&(j, _)| r.levels.get(j as usize).filter(|&&l| l > 0).map(|&l| (j, l.min(255) as u8)))
                .collect()
        }
        Slot::Ability { id, .. } if *id >= 0x200 => {
            let lv = super::book::ability_levels(*id);
            jobs.iter().filter_map(|&(j, _)| lv.iter().find(|(lj, _)| *lj == j).copied()).collect()
        }
        _ => return None,
    };
    let now = |j: u8| jobs.iter().find(|(x, _)| *x == j).map_or(0, |x| x.1);
    if need.is_empty() || need.iter().any(|&(j, l)| l <= now(j)) {
        return None;
    }
    need.iter().map(|x| x.1).min()
}

/// What we wear now, as a gear set's pieces.
pub fn current_gear(w: &World) -> Vec<(u8, u16)> {
    (0..16u8)
        .filter_map(|k| {
            let (b, i) = w.inv.equip.get(k as usize).copied().flatten()?;
            Some((k, w.inv.item(b, i)?))
        })
        .collect()
}

/// A gear set's pieces that are nowhere we can wear them from (the inventory, the wardrobes), by item id.
pub fn gear_missing(w: &World, pieces: &[(u8, u16)]) -> Vec<u16> {
    const WEAR: [u8; 9] = [0, 8, 10, 11, 12, 13, 14, 15, 16];
    let mut need: std::collections::HashMap<u16, u32> = std::collections::HashMap::new();
    for &(_, id) in pieces {
        *need.entry(id).or_default() += 1;
    }
    let mut out = Vec::new();
    for (&id, &n) in &need {
        let have: u32 =
            w.inv.slots.iter().filter(|((bag, _), (item, _))| *item == id && WEAR.contains(bag)).map(|(_, (_, c))| *c).sum();
        for _ in have..n {
            out.push(id);
        }
    }
    out
}

/// The equipment slots' names, by slot (0 Main .. 15 Back).
pub const GEAR_SLOTS: [&str; 16] = [
    "Main", "Sub", "Range", "Ammo", "Head", "Body", "Hands", "Legs", "Feet", "Neck", "Waist", "Ear 1", "Ear 2", "Ring 1",
    "Ring 2", "Back",
];

/// The pet bar's jobs: Beastmaster, Summoner, Puppetmaster (Dragoon's wyvern has no bar).
pub const PET_JOBS: [u8; 3] = [9, 15, 18];

/// The pet bar's actions while our pet is out (and our main job has one): the pet commands, and a Beastmaster's
/// Ready moves. A Summoner's Blood Pacts are in the Actions window's Ability Book.
pub fn pet_actions(w: &World) -> Vec<Slot> {
    let job = w.stats.as_ref().map_or(0, |s| s.main.0);
    if !PET_JOBS.contains(&job) || w.my_pet == 0 {
        return Vec::new();
    }
    let Some(a) = w.actions.as_ref() else { return Vec::new() };
    let mut rows = magic::ability_rows(w, 2);
    if job == 9 {
        rows.extend(magic::ability_rows(w, 0x12));
    }
    let mut seen = std::collections::HashSet::new();
    rows.into_iter()
        .filter_map(|i| a.abilities.get(i as usize))
        .filter(|x| x.id != 0 && x.sub == 0 && seen.insert(x.id))
        .map(|x| Slot::Ability { id: x.id, to: None })
        .collect()
}

/// A slot that shoots: the Misc Book's Ranged Attack, or a /ra (/range, /shoot, /throw) line.
fn ranged(s: &Slot) -> bool {
    match s {
        Slot::Misc { key } => key == "ranged",
        Slot::Command { line } => {
            let verb = line.trim().split_whitespace().next().unwrap_or("").to_ascii_lowercase();
            matches!(verb.as_str(), "/ra" | "/range" | "/shoot" | "/throw")
        }
        _ => false,
    }
}

/// The ammunition we wear (equipment slot 3) and how many of it we hold.
pub fn ammo(w: &World) -> Option<(u16, u32)> {
    let (b, i) = w.inv.equip.get(3).copied().flatten()?;
    let id = w.inv.item(b, i)?;
    Some((id, item_count(w, id)))
}

/// The count shown in a slot's corner, and whether it is running low: an item's, or the ammunition of a ranged slot.
pub fn count(w: &World, s: &Slot) -> Option<(u32, bool)> {
    match s {
        Slot::Item { id, .. } => {
            let n = item_count(w, *id);
            Some((n, n <= 3))
        }
        _ if ranged(s) => {
            let n = ammo(w).map_or(0, |a| a.1);
            Some((n, n <= 10))
        }
        _ => None,
    }
}

/// A spell's or ability's MP cost under our effects (none for the rest, or when it costs nothing).
pub fn mp_cost(w: &World, s: &Slot) -> Option<u16> {
    match s {
        Slot::Spell { id, .. } => magic::spell_mp_known(w, *id).filter(|&m| m > 0),
        Slot::Ability { id, .. } => {
            let a = w.actions.as_ref()?.abilities.get(ability_index(w, *id)? as usize)?;
            (magic::ability_kind(a.kind) == 6).then(|| magic::ability_mp(w, a.cost)).filter(|&m| m > 0)
        }
        _ => None,
    }
}

/// A spell's or ability's reach in yalms: the server's tables (assets/timeless), else a spell's DAT range.
fn range(w: &World, s: &Slot) -> Option<f32> {
    match s {
        Slot::Spell { id, .. } => super::book::range(0, *id).or_else(|| {
            let r = w.actions.as_ref()?.spell(*id)?;
            Some(crate::dat::actions::RANGES[(r.range & 0xf) as usize] as f32).filter(|&x| x > 0.0 && x < 255.0)
        }),
        Slot::Ability { id, .. } => super::book::range(1, *id),
        _ => None,
    }
}

/// Who the slot's action goes to now: Some(Some(id)) someone here, Some(None) someone picked when it is used, None
/// nobody (no target and it can't be ourselves). `targets`: the action's valid-target bits (1 ourselves).
fn aim(w: &World, s: &Slot, targets: u16) -> Option<Option<u32>> {
    let resolve = |tag: &str| textcmd::resolve(w, Some(tag), false, false, textcmd::Mode::Internal).map(|t| t.0);
    match target_tag(s) {
        None if targets == 1 => Some(Some(w.my_id)),
        None => resolve("<t>").or((targets & 1 != 0).then_some(w.my_id)).map(Some),
        Some("<me>") => Some(Some(w.my_id)),
        Some(t @ ("<t>" | "<bt>")) => resolve(t).map(Some),
        Some(t) if t.starts_with("<p") && t.len() == 4 => {
            let n: usize = t[2..3].parse().ok()?;
            w.party.slots().slots.get(n).cloned().flatten().map(|m| Some(m.id))
        }
        Some(_) => Some(None),
    }
}

/// Within `range` of us (as the server measures: straight-line distance less both bodies' sizes).
fn within(w: &World, t: u32, range: f32) -> bool {
    if t == w.my_id {
        return true;
    }
    let Some(e) = w.entities.get(&t) else { return true };
    (e.pos - w.pos).length() - (w.my_plate.hitbox + e.hitbox) <= range
}

fn reach(w: &World, s: &Slot, targets: u16, range: Option<f32>) -> Option<Why> {
    match aim(w, s, targets) {
        None => Some(Why::Target),
        Some(Some(t)) if range.is_some_and(|r| !within(w, t, r)) => Some(Why::Range),
        _ => None,
    }
}

/// The range ring: Some(true) when the slot's action goes to someone else here within its reach (the green edge).
pub fn in_range(w: &World, s: &Slot) -> Option<bool> {
    let targets = match s {
        Slot::Spell { id, .. } => w.actions.as_ref()?.spell(*id)?.targets,
        Slot::Ability { id, .. } => w.actions.as_ref()?.abilities.get(ability_index(w, *id)? as usize)?.targets,
        _ => return None,
    };
    let r = range(w, s)?;
    let t = aim(w, s, targets)??;
    (t != w.my_id).then(|| within(w, t, r))
}

/// Command lines the picker offers before the player's own.
const COMMANDS: [&str; 9] =
    ["/ra <t>", "/heal", "/check <t>", "/assist <t>", "/lockon", "/follow <t>", "/sit", "/map", "/dismount"];

/// The picker's rows for a tab: what the character can put on a slot now.
pub fn choices(w: &World, tab: usize) -> Vec<Slot> {
    let ability = |rows: Vec<u16>| -> Vec<Slot> {
        let Some(a) = w.actions.as_ref() else { return Vec::new() };
        rows.into_iter()
            .filter_map(|i| a.abilities.get(i as usize))
            .filter(|x| x.id != 0 && x.sub == 0)
            .map(|x| Slot::Ability { id: x.id, to: None })
            .collect()
    };
    match tab {
        0 => magic::magic_rows(w, 0, false, 0).into_iter().map(|id| Slot::Spell { id, to: None }).collect(),
        1 => {
            // Job abilities, pet commands and the sub-lists' rows (Phantom Roll, Blood Pact ...); not traits (4),
            // weapon skills (3) or Monstrosity's moves (0x14).
            let mut rows = Vec::new();
            for ty in [1u16, 2].into_iter().chain(6..=0x17).filter(|&t| t != 0x14) {
                rows.extend(magic::ability_rows(w, ty));
            }
            let mut out = ability(rows);
            let mut seen = std::collections::HashSet::new();
            out.retain(|s| seen.insert(s.clone_key()));
            out
        }
        2 => ability(magic::ability_rows(w, 3)),
        3 => {
            let mut seen = std::collections::HashSet::new();
            super::super::items::listed_at(w, true)
                .into_iter()
                .filter(|(_, s)| s.id != 0 && s.id != 0xffff && seen.insert(s.id))
                .map(|(_, s)| Slot::Item { id: s.id, to: None })
                .collect()
        }
        _ => {
            let mut seen = std::collections::HashSet::new();
            w.line_hist
                .recent()
                .iter()
                .map(String::as_str)
                .filter(|l| l.starts_with('/'))
                .chain(COMMANDS)
                .filter(|l| seen.insert(l.to_string()))
                .map(|l| Slot::Command { line: l.to_string() })
                .fold(vec![Slot::Attack], |mut v, s| {
                    v.push(s);
                    v
                })
        }
    }
}

impl Slot {
    /// A key that tells slots apart (for removing repeats).
    fn clone_key(&self) -> String {
        match self {
            Slot::Spell { id, .. } => format!("s{id}"),
            Slot::Ability { id, .. } => format!("a{id}"),
            Slot::Item { id, .. } => format!("i{id}"),
            Slot::Command { line } => format!("c{line}"),
            Slot::Attack => "attack".into(),
            Slot::Misc { key } => format!("m{key}"),
            Slot::Gear { name, .. } => format!("g{name}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_and_modifiers() {
        let none = egui::Modifiers::default();
        let ctrl = egui::Modifiers { ctrl: true, ..Default::default() };
        let alt = egui::Modifiers { alt: true, ..Default::default() };
        let shift = egui::Modifiers { shift: true, ..Default::default() };
        let both = egui::Modifiers { ctrl: true, alt: true, ..Default::default() };
        assert_eq!(key_slot(egui::Key::Num1, none), Some((0, 0)));
        assert_eq!(key_slot(egui::Key::Num0, ctrl), Some((1, 9)));
        assert_eq!(key_slot(egui::Key::Equals, alt), Some((2, 11)));
        assert_eq!(key_slot(egui::Key::Minus, none), Some((0, 10)));
        // Shift types ("!"), Ctrl+Alt is AltGr on some layouts: neither is a slot.
        assert_eq!(key_slot(egui::Key::Num1, shift), None);
        assert_eq!(key_slot(egui::Key::Num1, both), None);
        assert_eq!(key_slot(egui::Key::A, none), None);
        assert_eq!(label(0, 0), "1");
        assert_eq!(label(1, 10), "C-");
        assert_eq!(label(2, 11), "A=");
    }

    #[test]
    fn save_file_round_trip() {
        let mut h = Hotbar { owner: String::new(), job: 1, ..Default::default() };
        h.shared_on = [false, true, false];
        h.set(0, 0, Some(Slot::Spell { id: 1, to: None }));
        h.set(1, 4, Some(Slot::Ability { id: 0x200 + 35, to: None }));
        h.set(2, 11, Some(Slot::Item { id: 4112, to: None }));
        h.set(0, 1, Some(Slot::Command { line: "/ja \"Provoke\" <t>".into() }));
        let text = serde_json::to_string(&h.file()).unwrap();
        assert!(text.contains("\"kind\":\"spell\""));
        let mut back = Hotbar { job: 1, ..Default::default() };
        back.load(serde_json::from_str(&text).unwrap());
        assert_eq!(back.shared_on, [false, true, false]);
        assert_eq!(back.bars[0][0], Some(Slot::Spell { id: 1, to: None }));
        assert_eq!(back.bars[1][4], Some(Slot::Ability { id: 0x223, to: None }));
        assert_eq!(back.bars[2][11], Some(Slot::Item { id: 4112, to: None }));
        assert_eq!(back.bars[0][1], Some(Slot::Command { line: "/ja \"Provoke\" <t>".into() }));
        assert_eq!(back.bars[0][2], None);
        // Version 1 (one set of bars) loads as shared bars.
        let mut old = Hotbar::default();
        old.load(serde_json::from_str(r#"{"version":1,"bars":[[{"kind":"spell","id":5}]]}"#).unwrap());
        assert_eq!(old.shared_on, [true; BARS]);
        assert_eq!(old.bars[0][0], Some(Slot::Spell { id: 5, to: None }));
    }

    #[test]
    fn bars_follow_the_main_job() {
        let mut h = Hotbar { job: 1, ..Default::default() };
        h.shared_on[2] = true;
        h.set(0, 0, Some(Slot::Spell { id: 1, to: None }));
        h.set(2, 0, Some(Slot::Item { id: 9, to: None }));
        h.set_job(3);
        // This job's bar 1 is its own (new: Attack first); the shared Alt bar stays.
        assert_eq!(h.bars[0][0], Some(Slot::Attack));
        assert_eq!(h.bars[2][0], Some(Slot::Item { id: 9, to: None }));
        h.set(0, 0, Some(Slot::Spell { id: 2, to: None }));
        h.set_job(1);
        assert_eq!(h.bars[0][0], Some(Slot::Spell { id: 1, to: None }));
        // Shared: an empty shared bar takes what the job's bar showed; back to the job's, it keeps its own.
        h.toggle_shared(0);
        assert!(h.shared_on[0]);
        assert_eq!(h.bars[0][0], Some(Slot::Spell { id: 1, to: None }));
        h.set_job(3);
        assert_eq!(h.bars[0][0], Some(Slot::Spell { id: 1, to: None }));
        h.toggle_shared(0);
        assert_eq!(h.bars[0][0], Some(Slot::Spell { id: 2, to: None }));
        assert_eq!(job_name(1), "WAR");
    }

    #[test]
    fn new_jobs_start_with_attack() {
        let mut h = Hotbar::default();
        h.set_job(1);
        assert_eq!(h.bars[0][0], Some(Slot::Attack));
        // Cleared, it stays cleared, also through the save file.
        h.set(0, 0, None);
        h.set_job(2);
        h.set_job(1);
        assert_eq!(h.bars[0][0], None);
        let text = serde_json::to_string(&h.file()).unwrap();
        assert!(text.contains("\"kind\":\"attack\""));
        let mut back = Hotbar::default();
        back.load(serde_json::from_str(&text).unwrap());
        back.set_job(1);
        assert_eq!(back.bars[0][0], None);
        back.set_job(2);
        assert_eq!(back.bars[0][0], Some(Slot::Attack));
    }

    #[test]
    fn locked_bars_refuse_changes() {
        let mut h = Hotbar { job: 1, ..Default::default() };
        h.set(0, 0, Some(Slot::Spell { id: 1, to: None }));
        h.locked = true;
        h.set(0, 1, Some(Slot::Spell { id: 2, to: None }));
        h.set(0, 0, None);
        h.swap((0, 0), (1, 0));
        h.toggle_shared(0);
        assert_eq!(h.bars[0][0], Some(Slot::Spell { id: 1, to: None }));
        assert_eq!(h.bars[0][1], None);
        assert!(!h.shared_on[0]);
        let text = serde_json::to_string(&h.file()).unwrap();
        let mut back = Hotbar { job: 1, ..Default::default() };
        back.load(serde_json::from_str(&text).unwrap());
        assert!(back.locked && !back.reset_place);
    }

    #[test]
    fn swap_and_set() {
        let mut h = Hotbar { job: 1, ..Default::default() };
        h.set(0, 0, Some(Slot::Spell { id: 1, to: None }));
        h.swap((0, 0), (2, 5));
        assert_eq!(h.bars[0][0], None);
        assert_eq!(h.bars[2][5], Some(Slot::Spell { id: 1, to: None }));
        h.set(2, 5, None);
        assert_eq!(h.bars[2][5], None);
    }

    #[test]
    fn pages_keep_their_own_slots() {
        let mut h = Hotbar { job: 1, ..Default::default() };
        h.set(0, 0, Some(Slot::Spell { id: 1, to: None }));
        h.set_page(0, 1);
        assert_eq!(h.bars[0][0], None);
        h.set(0, 0, Some(Slot::Spell { id: 2, to: None }));
        h.set_page(0, 0);
        assert_eq!(h.bars[0][0], Some(Slot::Spell { id: 1, to: None }));
        // Through the file, with the page shown and the looks.
        h.set_page(0, 1);
        h.looks[1] = BarLook { cols: 4, show: 1, keys: None };
        h.scale = 1.25;
        let text = serde_json::to_string(&h.file()).unwrap();
        let mut back = Hotbar { job: 1, ..Default::default() };
        back.load(serde_json::from_str(&text).unwrap());
        assert_eq!(back.page[0], 1);
        assert_eq!(back.bars[0][0], Some(Slot::Spell { id: 2, to: None }));
        assert_eq!(back.looks[1], BarLook { cols: 4, show: 1, keys: None });
        assert_eq!(back.scale, 1.25);
        // A version 3 file (no pages): its rows are each bar's first page.
        let mut old = Hotbar { job: 1, ..Default::default() };
        old.load(
            serde_json::from_str(
                r#"{"version":3,"shared_on":[true,true,true],"bars":[[],[{"kind":"spell","id":7}],[{"kind":"item","id":9}]]}"#,
            )
            .unwrap(),
        );
        assert_eq!(old.bars[1][0], Some(Slot::Spell { id: 7, to: None }));
        assert_eq!(old.bars[2][0], Some(Slot::Item { id: 9, to: None }));
        assert_eq!(old.looks[0], BarLook::default());
        assert_eq!(old.opacity, 1.0);
    }

    #[test]
    fn add_copy_and_targets() {
        let mut h = Hotbar { job: 1, ..Default::default() };
        h.set_job(3);
        // Slot 1 holds Attack: the next empty one is slot 2.
        assert_eq!(h.add_next(Slot::Spell { id: 1, to: None }), Some((0, 1)));
        h.copy((0, 1), (2, 3));
        assert_eq!(h.bars[0][1], h.bars[2][3]);
        h.set_target(0, 1, Some("<st>".into()));
        assert_eq!(target_tag(h.bars[0][1].as_ref().unwrap()), Some("<st>"));
        let text = serde_json::to_string(&h.file()).unwrap();
        assert!(text.contains("\"to\":\"<st>\""));
        // Another job's own bar, at the same place.
        assert!(h.copy_to_job(0, 1, 5));
        assert_eq!(h.job_slot(5, 0, 1), h.bars[0][1].as_ref());
        assert_eq!(h.job_slot(5, 0, 0), Some(&Slot::Attack));
        h.shared_on[2] = true;
        assert!(!h.copy_to_job(2, 3, 5));
        // A gear set keeps its pieces.
        h.set(1, 0, Some(Slot::Gear { name: "Nuke".into(), pieces: vec![(4, 12345), (13, 1), (14, 1)] }));
        let text = serde_json::to_string(&h.file()).unwrap();
        let mut back = Hotbar { job: 3, ..Default::default() };
        back.load(serde_json::from_str(&text).unwrap());
        assert_eq!(back.bars[1][0], Some(Slot::Gear { name: "Nuke".into(), pieces: vec![(4, 12345), (13, 1), (14, 1)] }));
        assert_eq!(key_text(1, 2), "Ctrl+3");
        h.locked = true;
        assert_eq!(h.add_next(Slot::Attack), None);
    }

    #[test]
    fn key_sets_rebind_bars() {
        let none = egui::Modifiers::default();
        let shift = egui::Modifiers { shift: true, ..Default::default() };
        let ctrl = egui::Modifiers { ctrl: true, ..Default::default() };
        let mut looks = [BarLook::default(); BARS];
        looks[1].keys = Some(4);
        looks[2].keys = Some(3);
        assert_eq!(key_slot_with(&looks, egui::Key::F3, none), Some((1, 2)));
        assert_eq!(key_slot_with(&looks, egui::Key::Num5, shift), Some((2, 4)));
        // Ctrl + the number row is nobody's now.
        assert_eq!(key_slot_with(&looks, egui::Key::Num5, ctrl), None);
        assert_eq!(key_slot_with(&looks, egui::Key::Num5, none), Some((0, 4)));
        assert_eq!(label_of(4, 11), "F12");
        assert_eq!(label_of(5, 0), "CF1");
        assert_eq!(label_of(3, 10), "S-");
        assert_eq!(key_text_of(5, 1), "Ctrl+F2");
        assert_eq!(key_text_of(NO_KEYS, 1), "");
        looks[0].keys = Some(NO_KEYS);
        assert_eq!(key_slot_with(&looks, egui::Key::Num1, none), None);
    }

    #[test]
    fn export_import_round_trip() {
        for data in [&b""[..], b"a", b"ab", b"abc", b"abcd", &[0xff, 0x00, 0x7f, 0x80, 0x10][..]] {
            assert_eq!(unb64(&b64(data)).as_deref(), Some(data));
        }
        let mut h = Hotbar { job: 1, ..Default::default() };
        h.set(0, 0, Some(Slot::Spell { id: 1, to: Some("<p1>".into()) }));
        h.set_page(2, 2);
        h.set(2, 11, Some(Slot::Gear { name: "TP".into(), pieces: vec![(0, 16535)] }));
        h.shared_on[1] = true;
        h.set(1, 3, Some(Slot::Command { line: "/ma \"Cure\" <stpc>".into() }));
        let code = h.export();
        assert!(code.starts_with("TLXI1."));
        let mut other = Hotbar { job: 4, ..Default::default() };
        other.set_job(4);
        assert_eq!(other.import(&code), Ok(3));
        assert_eq!(other.bars[0][0], Some(Slot::Spell { id: 1, to: Some("<p1>".into()) }));
        other.set_page(2, 2);
        assert_eq!(other.bars[2][11], Some(Slot::Gear { name: "TP".into(), pieces: vec![(0, 16535)] }));
        assert!(other.import("hello").is_err());
        assert!(other.import(&code[..code.len() / 2]).is_err());
        other.locked = true;
        assert!(other.import(&code).is_err());
        other.locked = false;
        other.clear_job();
        other.set_page(2, 2);
        assert_eq!(other.bars[2][11], None);
        assert_eq!(other.bars[0][0], None);
    }
}
