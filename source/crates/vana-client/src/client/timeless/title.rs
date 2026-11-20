//! Timeless XI: the Timeless title screen (Interface = Timeless). Our own slow walkthrough reels play behind the
//! title instead of retail's random movies: an opening at the arch of Lufaise Meadows (the logo's own background), the
//! Hall of the Gods, and a closing shot that pushes into a crystal and cuts to the crystal in the logo. The hero logo
//! sits over them (layout 1), and a nostalgia filter grades the picture.
//!
//! The reel is `assets/timeless/title_reel.json`, or `%APPDATA%\vana-client\timeless-title.json` when that file
//! exists (it is what /titlecam writes). Each shot is one zone at one hour and weather; its moves are camera paths
//! (eye, target, focal, roll keys, as retail's title paths) played one after another with a cut or a crossfade.

use crate::dat::title::{CameraPath, Key, Op, Title};
use glam::Vec3;
use serde::{Deserialize, Serialize};

const BUILT_IN: &str = include_str!("../../../assets/timeless/title_reel.json");

/// The title's music: the Hall of the Gods' (timeless-server data/zones/hall_of_the_gods/zone.yaml).
pub const MUSIC: u32 = 213;

/// The queue names of our shots (FlyoverData's queue holds retail's scheduler names otherwise).
pub const PREFIX: &str = "tl:";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Reel {
    /// "memory", "film", "dusk" or "off".
    #[serde(default = "default_filter")]
    pub filter: String,
    #[serde(default)]
    pub logo: LogoPlace,
    /// The shot every pass starts with.
    #[serde(default)]
    pub opening: Option<String>,
    pub shots: Vec<Shot>,
}

fn default_filter() -> String {
    "memory".into()
}

/// Where the logo sits, as fractions of the screen, and where its crystal is in the logo picture.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct LogoPlace {
    pub center_y: f32,
    pub height: f32,
    /// Seconds into the intro shot before the logo starts to appear, and how long it takes.
    #[serde(default = "default_logo_in")]
    pub fade_in_at: f32,
    #[serde(default = "default_logo_in_secs")]
    pub fade_in: f32,
}

fn default_logo_in() -> f32 {
    5.0
}
fn default_logo_in_secs() -> f32 {
    7.0
}

impl Default for LogoPlace {
    fn default() -> Self {
        Self { center_y: 0.29, height: 0.42, fade_in_at: default_logo_in(), fade_in: default_logo_in_secs() }
    }
}

/// The logo picture's size and its crystal's place in it (title_logo.png).
pub const LOGO_SIZE: [f32; 2] = [1329.0, 846.0];
pub const LOGO_CRYSTAL: [f32; 2] = [0.5031, 0.2813];

impl LogoPlace {
    /// The logo's rect on a screen of `size` (any unit): x, y, w, h.
    pub fn rect(&self, size: [f32; 2]) -> [f32; 4] {
        let h = self.height * size[1];
        let w = h * LOGO_SIZE[0] / LOGO_SIZE[1];
        // Never wider than 90 % of the screen.
        let (w, h) =
            if w > size[0] * 0.9 { (size[0] * 0.9, size[0] * 0.9 * LOGO_SIZE[1] / LOGO_SIZE[0]) } else { (w, h) };
        [(size[0] - w) / 2.0, self.center_y * size[1] - h / 2.0, w, h]
    }

    /// The logo crystal's place on the screen, as fractions (0..1) of it.
    pub fn crystal_uv(&self, size: [f32; 2]) -> [f32; 2] {
        let r = self.rect(size);
        [(r[0] + LOGO_CRYSTAL[0] * r[2]) / size[0].max(1.0), (r[1] + LOGO_CRYSTAL[1] * r[3]) / size[1].max(1.0)]
    }
}

/// The Timeless title's five buttons (Select, Create, Delete, Settings, Exit Game): one row of text buttons low on the
/// screen, just above the help bar, across about half of it: x, y, w, h in the screen's units.
pub fn button_rects(size: [f32; 2], place: &LogoPlace) -> [[f32; 4]; 5] {
    let [sw, sh] = size;
    let logo = place.rect(size);
    let total = (sw * 0.56).min(sh * 1.0);
    let gap = (total * 0.02).max(6.0);
    let w = (total - gap * 4.0) / 5.0;
    let h = (w * 0.3).min(sh * 0.06);
    // Low, above the help bar and the notice over it; never up into the logo.
    let y = (sh * 0.84 - h).max(logo[1] + logo[3] + sh * 0.08);
    let x0 = (sw - total) / 2.0;
    std::array::from_fn(|i| [(x0 + i as f32 * (w + gap)).round(), y.round(), w.round(), h.round()])
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Shot {
    pub id: String,
    pub zone: u32,
    /// Vana'diel hour, e.g. 21.5.
    #[serde(default = "default_hour")]
    pub hour: f32,
    #[serde(default = "default_weather")]
    pub weather: String,
    /// "intro" (the logo fades in over it) or "match" (the closing crystal cut). Anything else: none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logo: Option<String>,
    #[serde(default = "default_fade")]
    pub fade_in: f32,
    #[serde(default = "default_fade")]
    pub fade_out: f32,
    /// A note for whoever edits the file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    pub moves: Vec<Move>,
}

fn default_hour() -> f32 {
    20.0
}
fn default_weather() -> String {
    "suny".into()
}
fn default_fade() -> f32 {
    3.0
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Move {
    pub seconds: f32,
    /// "linear", "in", "out" or "inout" (the default: a gentle start and stop).
    #[serde(default = "default_ease")]
    pub ease: String,
    /// Seconds of crossfade from the move before (0 = a cut). Not used on a shot's first move.
    #[serde(default)]
    pub cross: f32,
    pub keys: Vec<KeyJson>,
}

fn default_ease() -> String {
    "inout".into()
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct KeyJson {
    pub eye: [f32; 3],
    pub target: [f32; 3],
    #[serde(default = "default_focal")]
    pub focal: f32,
    #[serde(default)]
    pub roll: f32,
}

fn default_focal() -> f32 {
    350.0
}

impl Reel {
    pub fn built_in() -> Self {
        serde_json::from_str(BUILT_IN).expect("title_reel.json parses")
    }

    /// The player's reel (what /titlecam writes) when it exists and parses, else ours.
    pub fn load() -> Self {
        match std::fs::read_to_string(user_path()) {
            Ok(s) => match serde_json::from_str::<Reel>(&s) {
                Ok(r) if !r.shots.is_empty() => r,
                Ok(_) => Self::built_in(),
                Err(e) => {
                    log::warn!("timeless-title.json: {e}; playing the built-in reel");
                    Self::built_in()
                }
            },
            Err(_) => Self::built_in(),
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        let s = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        std::fs::write(user_path(), s)
    }

    pub fn shot(&self, id: &str) -> Option<&Shot> {
        self.shots.iter().find(|s| s.id == id)
    }

    /// One pass: the opening, then the rest in file order.
    pub fn playlist(&self) -> Vec<String> {
        let mut q: Vec<String> = Vec::new();
        if let Some(o) = self.opening.as_ref().filter(|o| self.shot(o).is_some()) {
            q.push(format!("{PREFIX}{o}"));
        }
        for s in &self.shots {
            if Some(&s.id) != self.opening.as_ref() && !s.moves.is_empty() {
                q.push(format!("{PREFIX}{}", s.id));
            }
        }
        q
    }
}

pub fn user_path() -> std::path::PathBuf {
    crate::config::config_path().with_file_name("timeless-title.json")
}

fn ease_code(s: &str) -> u32 {
    match s {
        "linear" => 0,
        "out" => 1,
        "in" => 2,
        _ => 4,
    }
}

/// Ticks (1/60 s) of a span in seconds.
fn ticks(secs: f32) -> u16 {
    (secs.max(0.0) * 60.0).round().clamp(0.0, u16::MAX as f32) as u16
}

impl Shot {
    /// The shot's length in seconds (its moves, less their crossfades' overlap).
    pub fn seconds(&self) -> f32 {
        self.moves.iter().map(|m| m.seconds).sum()
    }

    /// The shot as a title movie: its camera paths and the ops that play them (as file 23's schedulers would).
    pub fn movie(&self) -> (Title, Vec<Op>) {
        let mut paths = Vec::new();
        let mut ops = vec![
            Op::Zone(self.zone),
            Op::Time((self.hour.rem_euclid(24.0) * 216000.0) as u32),
            Op::Weather { tag: self.weather.clone(), ticks: 0 },
            Op::Fade { rgba: [0x80, 0x80, 0x80, 0x80], wait: 0, ticks: ticks(self.fade_in) },
        ];
        let n = self.moves.len();
        for (i, m) in self.moves.iter().enumerate() {
            let name = format!("tl{i}");
            paths.push(CameraPath {
                name: name.clone(),
                attach: 0,
                flags: 0,
                ease: ease_code(&m.ease),
                ease_key: None,
                keys: m
                    .keys
                    .iter()
                    .map(|k| Key { eye: Vec3::from(k.eye), focal: k.focal, target: Vec3::from(k.target), roll: k.roll })
                    .collect(),
            });
            if i > 0 && m.cross > 0.0 {
                ops.push(Op::Crossfade { wait: 0, ticks: ticks(m.cross) });
            }
            let len = ticks(m.seconds).max(1);
            if i + 1 == n {
                // The last move: the fade to black starts fade_out before its end, and the movie ends with it.
                let out = ticks(self.fade_out).min(len);
                ops.push(Op::Camera { path: name, wait: len - out, ticks: len });
                ops.push(Op::Fade { rgba: [0, 0, 0, 0x80], wait: out, ticks: out });
            } else {
                ops.push(Op::Camera { path: name, wait: len, ticks: len });
            }
        }
        (Title { schedulers: Vec::new(), paths }, ops)
    }
}

/// What a playing reel shot tells the title: its logo role and length.
#[derive(Clone, Debug)]
pub struct Playing {
    pub logo: Option<String>,
    pub seconds: f32,
    pub fade_out: f32,
    /// The reel's logo place, for the overlay and the match cut.
    pub place: LogoPlace,
}

impl Playing {
    pub fn of(s: &Shot, place: LogoPlace) -> Self {
        Self { logo: s.logo.clone(), seconds: s.seconds(), fade_out: s.fade_out, place }
    }
}

/// The title's logo and match-cut state, eased each frame.
#[derive(Default)]
pub struct TitleLook {
    /// The logo's alpha 0..1.
    pub logo: f32,
    /// The match cut's light at the logo crystal 0..1, and the logo's extra glow after it.
    pub flare: f32,
    pub glow: f32,
}

impl TitleLook {
    /// `playing`: the shot playing and the seconds into it, if one plays. `dt` in seconds.
    pub fn advance(&mut self, playing: Option<(&Playing, f32)>, dt: f32) {
        // The logo is up at once (a quick fade in); the reel's intro no longer holds it back.
        let want_logo = 1.0;
        let mut flare = 0.0;
        if let Some((p, at)) = playing
            && p.logo.as_deref() == Some("match")
        {
            // The light grows over the last moments of the push-in, peaks as the picture goes black, and
            // leaves its glow in the logo's crystal.
            let lead = (p.fade_out + 2.5).min(p.seconds);
            let t = ((at - (p.seconds - lead)) / lead).clamp(0.0, 1.0);
            // Up to full by 87 % of the way, then out by the end, so nothing is left on the black between
            // shots; the logo's afterglow carries it from there.
            let up = (t / 0.87).min(1.0).powf(1.6);
            let out = 1.0 - ((t - 0.87) / 0.13).clamp(0.0, 1.0);
            flare = up * out * out;
            if t > 0.8 {
                self.glow = self.glow.max((t - 0.8) / 0.2);
            }
        }
        let k = (dt * 6.0).min(1.0);
        self.logo += (want_logo - self.logo) * k;
        self.flare = flare;
        // The crystal's afterglow fades over about 4 s.
        self.glow = (self.glow - dt * 0.25).max(0.0);
    }
}

/// The nostalgia filter's settings for a preset: see render/nostalgia.rs for what each does.
pub fn filter_preset(name: &str) -> Option<crate::render::nostalgia::Grade> {
    use crate::render::nostalgia::Grade;
    match name {
        "off" | "none" => None,
        "film" => Some(Grade {
            lift: [0.050, 0.040, 0.030],
            shadow_tint: [0.02, 0.00, -0.02],
            high_tint: [0.05, 0.02, -0.04],
            saturation: 0.62,
            contrast: 1.04,
            halation: 0.32,
            halation_tint: [1.0, 0.62, 0.42],
            vignette: 0.55,
            grain: 0.055,
            soft_edges: 0.5,
            weave: 0.35,
            flicker: 0.025,
        }),
        "dusk" => Some(Grade {
            lift: [0.030, 0.034, 0.055],
            shadow_tint: [-0.01, 0.01, 0.04],
            high_tint: [0.02, 0.0, 0.03],
            saturation: 0.72,
            contrast: 1.02,
            halation: 0.30,
            halation_tint: [0.62, 0.74, 1.0],
            vignette: 0.50,
            grain: 0.030,
            soft_edges: 0.45,
            weave: 0.0,
            flicker: 0.0,
        }),
        // "memory": a soft, slightly faded colour photograph of a place you once stood.
        _ => Some(Grade {
            lift: [0.035, 0.036, 0.042],
            shadow_tint: [-0.01, 0.005, 0.02],
            high_tint: [0.035, 0.018, -0.02],
            saturation: 0.74,
            contrast: 1.0,
            halation: 0.26,
            halation_tint: [1.0, 0.86, 0.70],
            vignette: 0.45,
            grain: 0.035,
            soft_edges: 0.4,
            weave: 0.0,
            flicker: 0.012,
        }),
    }
}

/// /titlecam's state in the world: the reel being edited (loaded on first use), the shot keys go to, the guide.
#[derive(Default)]
pub struct CamState {
    pub reel: Option<Reel>,
    pub current: Option<String>,
    pub guide: bool,
}

/// /titlecam, in the world: records the current camera into timeless-title.json for the title reel.
pub mod cam {
    use super::*;

    pub const HELP: &[&str] = &[
        "/titlecam new <id> [hour] [weather]  start a shot here (this zone; hour defaults to now)",
        "/titlecam key [up|down N] [seconds]  add the camera as a key (optionally raised N yalms)",
        "/titlecam move [seconds] [cross]  start a new move in the shot (cross = crossfade seconds, 0 = cut)",
        "/titlecam undo | list | opening <id> | logo <intro|match|none> | filter <memory|film|dusk|off>",
        "/titlecam guide  show where the logo and its crystal sit (frame the match-cut crystal on the cross)",
        "Log out to the title to watch it. The file is timeless-title.json beside config.json.",
    ];

    /// Runs one /titlecam line; returns the lines to show in the chat.
    pub fn run(
        reel: &mut Reel,
        current: &mut Option<String>,
        args: &[&str],
        here: Option<(u32, f32, KeyJson)>,
        guide: &mut bool,
    ) -> Vec<String> {
        let num = |i: usize| args.get(i).and_then(|s| s.parse::<f32>().ok());
        let word = args.first().map(|s| s.to_ascii_lowercase());
        let mut out = Vec::new();
        let mut changed = false;
        match word.as_deref() {
            None | Some("help") => out.extend(HELP.iter().map(|s| s.to_string())),
            Some("guide") => {
                *guide = !*guide;
                out.push(format!("Title guide {}.", if *guide { "on" } else { "off" }));
            }
            Some("new") => {
                let Some(id) = args.get(1).map(|s| s.to_string()) else {
                    out.push("Give the shot a name: /titlecam new hall_walk".into());
                    return out;
                };
                let Some((zone, hour, _)) = here else {
                    out.push("Not in a zone.".into());
                    return out;
                };
                reel.shots.retain(|s| s.id != id);
                reel.shots.push(Shot {
                    id: id.clone(),
                    zone,
                    hour: num(2).unwrap_or(hour),
                    weather: args.get(3).map_or_else(default_weather, |s| s.to_string()),
                    logo: None,
                    fade_in: default_fade(),
                    fade_out: default_fade(),
                    note: None,
                    moves: vec![Move { seconds: 20.0, ease: default_ease(), cross: 0.0, keys: Vec::new() }],
                });
                *current = Some(id.clone());
                changed = true;
                out.push(format!(
                    "Shot {id}: zone {zone}, {:.1} h. Add keys with /titlecam key.",
                    num(2).unwrap_or(hour)
                ));
            }
            Some("key") => {
                let Some(shot) = current.as_ref().and_then(|c| reel.shots.iter_mut().find(|s| &s.id == c)) else {
                    out.push("Start a shot first: /titlecam new <id>".into());
                    return out;
                };
                let Some((zone, _, mut key)) = here else { return out };
                if zone != shot.zone {
                    out.push(format!("This shot is in zone {}; you are in {zone}.", shot.zone));
                    return out;
                }
                let mut i = 1;
                let mut secs = None;
                while i < args.len() {
                    match args[i].to_ascii_lowercase().as_str() {
                        // Up is -y in the game's world.
                        "up" => {
                            let n = num(i + 1).unwrap_or(2.0);
                            key.eye[1] -= n;
                            key.target[1] -= n;
                            i += 1;
                        }
                        "down" => {
                            let n = num(i + 1).unwrap_or(2.0);
                            key.eye[1] += n;
                            key.target[1] += n;
                            i += 1;
                        }
                        s => secs = s.parse::<f32>().ok().or(secs),
                    }
                    i += 1;
                }
                let n = shot.moves.len();
                let m = shot.moves.last_mut().unwrap();
                m.keys.push(key);
                if let Some(s) = secs {
                    m.seconds = s.max(1.0);
                }
                changed = true;
                out.push(format!(
                    "Key {} of move {} ({:.0} s): eye {:.1}, {:.1}, {:.1}",
                    m.keys.len(),
                    n,
                    m.seconds,
                    key.eye[0],
                    key.eye[1],
                    key.eye[2]
                ));
            }
            Some("move") => {
                let Some(shot) = current.as_ref().and_then(|c| reel.shots.iter_mut().find(|s| &s.id == c)) else {
                    out.push("Start a shot first: /titlecam new <id>".into());
                    return out;
                };
                shot.moves.push(Move {
                    seconds: num(1).unwrap_or(20.0).max(1.0),
                    ease: default_ease(),
                    cross: num(2).unwrap_or(2.0),
                    keys: Vec::new(),
                });
                changed = true;
                out.push(format!("Move {} started in {}.", shot.moves.len(), shot.id));
            }
            Some("undo") => {
                if let Some(shot) = current.as_ref().and_then(|c| reel.shots.iter_mut().find(|s| &s.id == c)) {
                    let n = shot.moves.len();
                    let m = shot.moves.last_mut().unwrap();
                    if m.keys.pop().is_some() {
                        out.push(format!("Removed a key; move {n} has {}.", m.keys.len()));
                    } else if shot.moves.len() > 1 {
                        shot.moves.pop();
                        out.push("Removed the empty move.".into());
                    }
                    changed = true;
                }
            }
            Some("list") => {
                for s in &reel.shots {
                    let keys: usize = s.moves.iter().map(|m| m.keys.len()).sum();
                    out.push(format!(
                        "{}{}: zone {}, {:.1} h, {} moves, {keys} keys, {:.0} s{}",
                        if current.as_ref() == Some(&s.id) { "> " } else { "" },
                        s.id,
                        s.zone,
                        s.hour,
                        s.moves.len(),
                        s.seconds(),
                        s.logo.as_ref().map(|l| format!(", logo {l}")).unwrap_or_default()
                    ));
                }
                out.push(format!("Opening: {}. Filter: {}.", reel.opening.as_deref().unwrap_or("none"), reel.filter));
            }
            Some("opening") => {
                if let Some(id) = args.get(1).filter(|id| reel.shot(id).is_some()) {
                    reel.opening = Some(id.to_string());
                    changed = true;
                    out.push(format!("The title opens on {id}."));
                } else {
                    out.push("No such shot.".into());
                }
            }
            Some("logo") => {
                if let Some(shot) = current.as_ref().and_then(|c| reel.shots.iter_mut().find(|s| &s.id == c)) {
                    shot.logo = args.get(1).map(|s| s.to_ascii_lowercase()).filter(|s| s == "intro" || s == "match");
                    changed = true;
                    out.push(format!("{}: logo {}.", shot.id, shot.logo.as_deref().unwrap_or("none")));
                }
            }
            Some("filter") => {
                if let Some(f) = args.get(1) {
                    reel.filter = f.to_ascii_lowercase();
                    changed = true;
                    out.push(format!("Title filter: {}.", reel.filter));
                }
            }
            Some("edit") => {
                if let Some(id) = args.get(1).filter(|id| reel.shot(id).is_some()) {
                    *current = Some(id.to_string());
                    out.push(format!("Editing {id}."));
                }
            }
            Some(_) => out.extend(HELP.iter().map(|s| s.to_string())),
        }
        if changed
            && !cfg!(test)
            && let Err(e) = reel.save()
        {
            out.push(format!("Could not save timeless-title.json: {e}"));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_reel_plays() {
        let r = Reel::built_in();
        let q = r.playlist();
        assert!(!q.is_empty());
        assert_eq!(q[0], format!("{PREFIX}{}", r.opening.clone().unwrap()));
        for s in &r.shots {
            let (title, ops) = s.movie();
            assert_eq!(title.paths.len(), s.moves.len());
            assert!(s.moves.iter().all(|m| m.keys.len() >= 2), "{}: every move needs two keys", s.id);
            // The ops' waits add up to the shot's length.
            let wait: u32 = ops
                .iter()
                .map(|o| match o {
                    Op::Camera { wait, .. } | Op::Fade { wait, .. } | Op::Crossfade { wait, .. } => *wait as u32,
                    _ => 0,
                })
                .sum();
            assert!((wait as f32 / 60.0 - s.seconds()).abs() < 0.1, "{}", s.id);
        }
        assert!(r.shots.iter().any(|s| s.logo.as_deref() == Some("match")));
    }

    #[test]
    fn buttons_under_the_logo() {
        let p = LogoPlace::default();
        for size in [[1920.0, 1080.0], [1280.0, 1024.0], [800.0, 600.0]] {
            let r = button_rects(size, &p);
            let logo = p.rect(size);
            assert!(r[0][1] > logo[1] + logo[3], "{size:?}");
            assert!(r[4][0] + r[4][2] < size[0] && r[0][0] > 0.0);
            assert!(r[0][1] + r[0][3] <= size[1] * 0.86);
        }
    }

    #[test]
    fn logo_crystal_on_screen() {
        let p = LogoPlace::default();
        let uv = p.crystal_uv([1920.0, 1080.0]);
        assert!((uv[0] - 0.5).abs() < 0.01);
        assert!(uv[1] > 0.1 && uv[1] < 0.3);
    }

    #[test]
    fn titlecam_records() {
        let mut r = Reel::built_in();
        r.shots.clear();
        let mut cur = None;
        let mut g = false;
        let k = KeyJson { eye: [0.0, -2.0, 0.0], target: [0.0, -2.0, 5.0], focal: 350.0, roll: 0.0 };
        let here = Some((251, 20.0, k));
        // Saving may fail in the test's environment; the reel itself is what we check.
        cam::run(&mut r, &mut cur, &["new", "t"], here, &mut g);
        cam::run(&mut r, &mut cur, &["key", "up", "3", "40"], here, &mut g);
        cam::run(&mut r, &mut cur, &["key"], here, &mut g);
        let s = r.shot("t").unwrap();
        assert_eq!(s.moves[0].keys.len(), 2);
        assert_eq!(s.moves[0].keys[0].eye[1], -5.0);
        assert_eq!(s.moves[0].seconds, 40.0);
    }
}
