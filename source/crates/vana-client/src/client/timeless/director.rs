//! Timeless XI: Director Mode, the filming toolkit (Config-free: the Director window, modern/tldirector.rs). It
//! drives this client's view only; nothing goes to the server, so other players see nothing of it.
//!
//! - Camera: the game's, a free flying camera (WASD / Q E / mouse), an orbit around a subject, or a follow camera
//!   that rides beside one; field of view, Dutch roll, aim lock and handheld shake on top.
//! - Shots: camera keys played back as a smooth spline with per-key travel and hold times, saved by name, and
//!   exported to the Timeless title reel (timeless-title.json).
//! - Scene: the hour (held, or a time-lapse), the moon phase, the weather and its blend, all of the zone's scenery.
//! - Look: the nostalgia filter's grade (render/nostalgia.rs) over the world, with presets and every slider.
//! - Cast: hide yourself, other players, NPCs, monsters or Companions.
//! - Capture: a full-size photo, and video through the client's recorder (client/record.rs, needs ffmpeg).
//!
//! Everything goes back to normal when the window closes.

use super::super::World;
use crate::render::Camera;
use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Instant;

/// The weathers by number (dat/weather.rs WEATHER_TAGS), as players know them.
pub const WEATHERS: [&str; 20] = [
    "Clear",
    "Sunny",
    "Cloudy",
    "Fog",
    "Hot spell",
    "Heat wave",
    "Rain",
    "Squall",
    "Dust storm",
    "Sand storm",
    "Wind",
    "Gales",
    "Snow",
    "Blizzard",
    "Thunder",
    "Thunderstorm",
    "Auroras",
    "Stellar glare",
    "Gloom",
    "Darkness",
];

/// The camera modes.
pub const GAME: u8 = 0;
pub const FLY: u8 = 1;
pub const ORBIT: u8 = 2;
pub const FOLLOW: u8 = 3;
pub const MODES: [&str; 4] = ["Game camera", "Free fly", "Orbit", "Follow"];

/// What a camera looks at or circles: nothing (free), our character, our target.
pub const FREE: u8 = 0;
pub const ME: u8 = 1;
pub const TARGET: u8 = 2;
pub const AIMS: [&str; 3] = ["Free look", "My character", "My target"];

/// Path easing: one gentle start and stop over the whole path, an even pace, or a stop at every key.
pub const EASE_SMOOTH: u8 = 0;
pub const EASE_LINEAR: u8 = 1;
pub const EASE_EACH: u8 = 2;
pub const EASES: [&str; 3] = ["Smooth start and stop", "Even pace", "Ease into every key"];

/// Letterbox ratios (0 = none).
pub const BARS: [(&str, f32); 6] = [
    ("None", 0.0),
    ("2.39 : 1 Scope", 2.39),
    ("2.00 : 1 Univisium", 2.0),
    ("1.85 : 1 Flat", 1.85),
    ("4 : 3 Classic", 4.0 / 3.0),
    ("9 : 16 Vertical", 9.0 / 16.0),
];

/// The farthest a key, a bookmark or the free camera may be from the origin (zones are a few thousand across).
const REACH: f32 = 30000.0;

/// One camera key: where the camera is, where it looks (yaw / pitch, radians, the game's axes: Y is down), its
/// vertical field of view and roll (degrees), the seconds it takes to travel on to the next key, and how long it
/// holds still here first.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Key {
    pub eye: [f32; 3],
    pub yaw: f32,
    pub pitch: f32,
    pub fov: f32,
    #[serde(default)]
    pub roll: f32,
    #[serde(default = "default_secs")]
    pub secs: f32,
    #[serde(default)]
    pub hold: f32,
}

fn default_secs() -> f32 {
    4.0
}

/// The look's sliders (render/nostalgia.rs Grade, kept serialisable) and whether it is on.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Look {
    pub on: bool,
    pub preset: u8,
    pub lift: [f32; 3],
    pub shadow_tint: [f32; 3],
    pub high_tint: [f32; 3],
    pub saturation: f32,
    pub contrast: f32,
    pub halation: f32,
    pub halation_tint: [f32; 3],
    pub vignette: f32,
    pub grain: f32,
    pub soft_edges: f32,
    pub weave: f32,
    pub flicker: f32,
}

impl Default for Look {
    fn default() -> Self {
        let mut l = Look::neutral();
        l.preset = 0;
        l
    }
}

/// The look presets the window offers: (name, settings). "Natural" is the picture untouched.
pub fn look_presets() -> Vec<(&'static str, Look)> {
    let from = |name: &str| {
        let g = super::title::filter_preset(name).expect("a title preset");
        Look::from_grade(&g)
    };
    let mut golden = Look::neutral();
    (golden.lift, golden.shadow_tint, golden.high_tint) =
        ([0.02, 0.012, 0.0], [0.0, -0.01, -0.03], [0.07, 0.035, -0.03]);
    (golden.saturation, golden.contrast, golden.halation, golden.halation_tint) = (1.08, 1.05, 0.38, [1.0, 0.72, 0.42]);
    (golden.vignette, golden.grain) = (0.35, 0.015);
    let mut noir = Look::neutral();
    (noir.saturation, noir.contrast, noir.lift, noir.halation, noir.halation_tint) =
        (0.0, 1.22, [0.02, 0.02, 0.02], 0.18, [1.0, 1.0, 1.0]);
    (noir.vignette, noir.grain, noir.flicker, noir.weave) = (0.7, 0.06, 0.02, 0.2);
    let mut crisp = Look::neutral();
    (crisp.saturation, crisp.contrast, crisp.vignette, crisp.shadow_tint) = (1.15, 1.1, 0.15, [-0.005, 0.0, 0.015]);
    let mut dream = Look::neutral();
    (dream.lift, dream.saturation, dream.contrast, dream.halation, dream.halation_tint) =
        ([0.05, 0.045, 0.06], 0.85, 0.94, 0.55, [0.85, 0.85, 1.0]);
    (dream.soft_edges, dream.vignette, dream.high_tint) = (0.9, 0.3, [0.02, 0.01, 0.04]);
    let mut moody = Look::neutral();
    (moody.lift, moody.shadow_tint, moody.high_tint) = ([0.0, 0.01, 0.02], [-0.02, 0.0, 0.04], [0.02, 0.0, -0.02]);
    (moody.saturation, moody.contrast, moody.vignette, moody.grain) = (0.8, 1.12, 0.6, 0.02);
    vec![
        ("Natural", Look::neutral()),
        ("Memory", from("memory")),
        ("Film", from("film")),
        ("Dusk", from("dusk")),
        ("Golden hour", golden),
        ("Moody", moody),
        ("Crisp", crisp),
        ("Dream", dream),
        ("Noir", noir),
    ]
}

impl Look {
    /// The picture as it comes: no tint, no lift, saturation and contrast 1.
    pub fn neutral() -> Self {
        Look {
            on: true,
            preset: 0,
            lift: [0.0; 3],
            shadow_tint: [0.0; 3],
            high_tint: [0.0; 3],
            saturation: 1.0,
            contrast: 1.0,
            halation: 0.0,
            halation_tint: [1.0, 0.86, 0.7],
            vignette: 0.0,
            grain: 0.0,
            soft_edges: 0.0,
            weave: 0.0,
            flicker: 0.0,
        }
    }

    pub fn from_grade(g: &crate::render::nostalgia::Grade) -> Self {
        Look {
            on: true,
            preset: 0,
            lift: g.lift,
            shadow_tint: g.shadow_tint,
            high_tint: g.high_tint,
            saturation: g.saturation,
            contrast: g.contrast,
            halation: g.halation,
            halation_tint: g.halation_tint,
            vignette: g.vignette,
            grain: g.grain,
            soft_edges: g.soft_edges,
            weave: g.weave,
            flicker: g.flicker,
        }
    }

    pub fn grade(&self) -> crate::render::nostalgia::Grade {
        crate::render::nostalgia::Grade {
            lift: self.lift,
            shadow_tint: self.shadow_tint,
            high_tint: self.high_tint,
            saturation: self.saturation,
            contrast: self.contrast,
            halation: self.halation,
            halation_tint: self.halation_tint,
            vignette: self.vignette,
            grain: self.grain,
            soft_edges: self.soft_edges,
            weave: self.weave,
            flicker: self.flicker,
        }
    }

    /// True when the grade would leave the picture as it is (no pass needed).
    pub fn is_neutral(&self) -> bool {
        let n = Look::neutral();
        let near = |a: f32, b: f32| (a - b).abs() < 1e-4;
        let near3 = |a: [f32; 3], b: [f32; 3]| (0..3).all(|k| near(a[k], b[k]));
        near3(self.lift, n.lift)
            && near3(self.shadow_tint, n.shadow_tint)
            && near3(self.high_tint, n.high_tint)
            && near(self.saturation, 1.0)
            && near(self.contrast, 1.0)
            && near(self.halation, 0.0)
            && near(self.vignette, 0.0)
            && near(self.grain, 0.0)
            && near(self.soft_edges, 0.0)
            && near(self.weave, 0.0)
            && near(self.flicker, 0.0)
    }
}

/// The scene: the hour and moon, the weather, the scenery, the look, the letterbox and who is shown. Saved with a
/// shot, so a take can be set up again exactly.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Scene {
    pub time_on: bool,
    /// Vana'diel hour, 0..24.
    pub hour: f32,
    /// Time-lapse: Vana'diel hours a real second (0 = held still).
    pub lapse: f32,
    pub moon_on: bool,
    /// Moon phase 0..11 (the client's twelve steps).
    pub moon: u8,
    pub weather_on: bool,
    pub weather: u16,
    /// Seconds a weather change blends over.
    pub blend: f32,
    /// Draw all of the zone's scenery (the camera lists are made for a camera near the ground).
    pub all_scenery: bool,
    pub look: Look,
    /// BARS index.
    pub bars: u8,
    pub hide_hud: bool,
    pub hide_me: bool,
    pub hide_players: bool,
    pub keep_party: bool,
    pub hide_npcs: bool,
    pub hide_mobs: bool,
    pub hide_companions: bool,
}

impl Default for Scene {
    fn default() -> Self {
        Scene {
            time_on: false,
            hour: 18.5,
            lapse: 0.0,
            moon_on: false,
            moon: 0,
            weather_on: false,
            weather: 1,
            blend: 4.0,
            all_scenery: false,
            look: Look::default(),
            bars: 0,
            hide_hud: false,
            hide_me: false,
            hide_players: false,
            keep_party: true,
            hide_npcs: false,
            hide_mobs: false,
            hide_companions: false,
        }
    }
}

/// A saved shot: its keys and how they play, and the scene it was set up in.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShotFile {
    pub name: String,
    #[serde(default)]
    pub zone: Option<u16>,
    pub keys: Vec<Key>,
    #[serde(default)]
    pub ease: u8,
    #[serde(default)]
    pub aim: u8,
    #[serde(default)]
    pub looped: bool,
    #[serde(default)]
    pub scene: Scene,
}

/// What is kept between sessions (timeless-director.json beside config.json).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Prefs {
    pub speed: f32,
    pub smoothing: f32,
    pub look_smoothing: f32,
    pub level_flight: bool,
    pub hold_character: bool,
    pub bookmarks: Vec<Option<(Option<u16>, Key)>>,
    pub countdown: u8,
    pub fps: u32,
    pub auto_hide: bool,
    pub guides_thirds: bool,
    pub guides_safe: bool,
    pub guides_center: bool,
    pub scene: Scene,
}

impl Default for Prefs {
    fn default() -> Self {
        Prefs {
            speed: 6.0,
            smoothing: 0.6,
            look_smoothing: 0.35,
            level_flight: false,
            hold_character: true,
            bookmarks: vec![None; 9],
            countdown: 3,
            fps: 60,
            auto_hide: true,
            guides_thirds: false,
            guides_safe: false,
            guides_center: false,
            scene: Scene::default(),
        }
    }
}

/// The keys the flying camera reads this frame, -1..1 a direction.
#[derive(Clone, Copy, Debug, Default)]
pub struct FlyInput {
    pub forward: f32,
    pub right: f32,
    pub up: f32,
    pub fast: bool,
    pub slow: bool,
    /// The arrow keys: turn (right +) and tilt (up +).
    pub turn: f32,
    pub tilt: f32,
}

/// What a countdown starts when it reaches zero.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cue {
    Play,
    Record,
    RecordPath,
}

/// The free camera's body: where it is and where it looks, eased toward what the input asks.
#[derive(Clone, Copy, Debug)]
pub struct Rig {
    pub eye: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    /// Vertical field of view and roll, degrees.
    pub fov: f32,
    pub roll: f32,
    vel: Vec3,
    /// The look the mouse asked for; yaw / pitch ease toward it.
    want: (f32, f32),
}

impl Default for Rig {
    fn default() -> Self {
        Rig { eye: Vec3::ZERO, yaw: 0.0, pitch: 0.0, fov: 45.0, roll: 0.0, vel: Vec3::ZERO, want: (0.0, 0.0) }
    }
}

/// The view's direction for a yaw and pitch (Y down: a positive pitch looks up).
pub fn dir_of(yaw: f32, pitch: f32) -> Vec3 {
    Vec3::new(pitch.cos() * yaw.sin(), -pitch.sin(), pitch.cos() * yaw.cos())
}

/// Yaw and pitch of a direction.
pub fn angles_of(d: Vec3) -> (f32, f32) {
    let d = d.try_normalize().unwrap_or(Vec3::Z);
    (d.x.atan2(d.z), (-d.y).clamp(-1.0, 1.0).asin())
}

/// The up vector for a view direction and a roll (degrees): (0, -1, 0) turned about the view.
pub fn up_of(dir: Vec3, roll_deg: f32) -> Vec3 {
    let up = Vec3::NEG_Y;
    if roll_deg.abs() < 1e-3 {
        return up;
    }
    glam::Quat::from_axis_angle(dir.try_normalize().unwrap_or(Vec3::Z), roll_deg.to_radians()) * up
}

/// The 35 mm-equivalent focal length (24 mm frame height) of a vertical field of view in degrees.
pub fn focal_mm(fov_deg: f32) -> f32 {
    12.0 / (fov_deg.to_radians() * 0.5).tan()
}

impl Rig {
    pub fn key(&self) -> Key {
        Key {
            eye: self.eye.to_array(),
            yaw: self.yaw,
            pitch: self.pitch,
            fov: self.fov,
            roll: self.roll,
            secs: 4.0,
            hold: 0.0,
        }
    }

    pub fn set(&mut self, k: &Key) {
        self.eye = Vec3::from(k.eye);
        (self.yaw, self.pitch, self.fov, self.roll) = (k.yaw, k.pitch, k.fov, k.roll);
        self.want = (k.yaw, k.pitch);
        self.vel = Vec3::ZERO;
    }

    pub fn from_camera(c: &Camera) -> Self {
        let (yaw, pitch) = angles_of(c.target - c.eye);
        Rig { eye: c.eye, yaw, pitch, fov: c.fov_y.to_degrees(), roll: 0.0, vel: Vec3::ZERO, want: (yaw, pitch) }
    }
}

/// The orbit: around a subject (`ME` / `TARGET`, or the point it was set on), at a yaw and pitch (radians), a
/// distance and a height above the subject's feet, turning by itself `spin` degrees a second.
#[derive(Clone, Copy, Debug)]
pub struct Orbit {
    pub subject: u8,
    pub point: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub dist: f32,
    pub height: f32,
    pub spin: f32,
}

impl Default for Orbit {
    fn default() -> Self {
        Orbit { subject: ME, point: Vec3::ZERO, yaw: 0.6, pitch: 0.2, dist: 5.0, height: 1.2, spin: 0.0 }
    }
}

/// The follow camera: beside a subject, in the subject's own frame (`side` right, `up`, `back` behind it), looking
/// at it `ahead` yalms in front of it, `ease` how quickly it catches up (per second).
#[derive(Clone, Copy, Debug)]
pub struct Follow {
    pub subject: u8,
    pub side: f32,
    pub up: f32,
    pub back: f32,
    pub ahead: f32,
    pub ease: f32,
}

impl Default for Follow {
    fn default() -> Self {
        Follow { subject: ME, side: 2.5, up: 1.6, back: 3.5, ahead: 1.5, ease: 2.5 }
    }
}

/// Handheld shake: degrees of sway and how quickly it wanders.
#[derive(Clone, Copy, Debug, Default)]
pub struct Shake {
    pub amount: f32,
    pub speed: f32,
}

/// Director Mode's state in the world.
pub struct Director {
    pub open: bool,
    /// Ctrl+Shift+H: the window tucked away (everything else stays as set); during a take or a playing shot, the
    /// window brought back over it.
    pub panel_hidden: bool,
    pub peek: bool,
    pub tab: u8,
    pub mode: u8,
    pub rig: Rig,
    pub orbit: Orbit,
    pub follow: Follow,
    pub shake: Shake,
    /// The free camera's and the path's aim lock.
    pub aim: u8,
    pub prefs: Prefs,
    /// The path being edited, how it plays, and the key selected in the list.
    pub keys: Vec<Key>,
    pub ease: u8,
    pub looped: bool,
    pub play_speed: f32,
    pub selected: Option<usize>,
    pub show_path: bool,
    /// Playing: seconds into the path (unscaled).
    pub playing: Option<f32>,
    /// The timeline being scrubbed: the camera shows the path at that time (the window stays up).
    pub scrub: Option<f32>,
    /// A countdown before a cue: seconds left.
    pub countdown: Option<(f32, Cue)>,
    /// Recording a video: its file and when it began; whether it ends with the path.
    pub recording: Option<(PathBuf, Instant, bool)>,
    /// A photo asked for: its file and frames to wait (the window and the HUD gone first), the HUD state to restore.
    pub photo: Option<(PathBuf, u8, bool)>,
    /// The time-lapse's running hour (fractional).
    pub lapse_hour: f32,
    /// The weather tag applied to the effects, while the director's weather is in force.
    applied_weather: Option<u16>,
    /// The zone the rig was set in (a new zone puts it back behind the game camera).
    zone: Option<u16>,
    /// The HUD hidden by the director (so closing the window shows it again).
    hud_ours: bool,
    /// The keyboard is in one of our text fields (no flying while typing a name).
    pub typing: bool,
    /// The name box for saving, the saved shots list and the last message.
    pub shot_name: String,
    pub library: Vec<String>,
    pub note: Option<(String, Instant)>,
    pub ffmpeg: std::sync::Arc<std::sync::atomic::AtomicU8>,
    /// The grade's full-screen pass (render/nostalgia.rs), made on first use.
    pub pass: Option<crate::render::nostalgia::Nostalgia>,
    started: Instant,
    loaded: bool,
}

/// ffmpeg's state for video: not checked yet, checking, found, missing.
pub const FF_UNKNOWN: u8 = 0;
pub const FF_CHECKING: u8 = 1;
pub const FF_OK: u8 = 2;
pub const FF_MISSING: u8 = 3;

impl Default for Director {
    fn default() -> Self {
        Director {
            open: false,
            panel_hidden: false,
            peek: false,
            tab: 0,
            mode: GAME,
            rig: Rig::default(),
            orbit: Orbit::default(),
            follow: Follow::default(),
            shake: Shake { amount: 0.0, speed: 0.6 },
            aim: FREE,
            prefs: Prefs::default(),
            keys: Vec::new(),
            ease: EASE_SMOOTH,
            looped: false,
            play_speed: 1.0,
            selected: None,
            show_path: true,
            playing: None,
            scrub: None,
            countdown: None,
            recording: None,
            photo: None,
            lapse_hour: 18.5,
            applied_weather: None,
            zone: None,
            hud_ours: false,
            typing: false,
            shot_name: String::new(),
            library: Vec::new(),
            note: None,
            ffmpeg: Default::default(),
            pass: None,
            started: Instant::now(),
            loaded: false,
        }
    }
}

fn prefs_path() -> PathBuf {
    crate::config::config_path().with_file_name("timeless-director.json")
}

/// The saved shots' folder.
pub fn shots_dir() -> PathBuf {
    crate::config::config_path().with_file_name("timeless-director")
}

/// Where videos and photos go: Videos / Pictures \ Timeless XI.
pub fn video_dir() -> PathBuf {
    dirs::video_dir().unwrap_or_else(crate::config::data_dir).join("Timeless XI")
}
pub fn photo_dir() -> PathBuf {
    dirs::picture_dir().unwrap_or_else(crate::config::data_dir).join("Timeless XI")
}

fn stamp() -> String {
    let t = jiff::Zoned::now();
    format!("{:04}-{:02}-{:02} {:02}.{:02}.{:02}", t.year(), t.month(), t.day(), t.hour(), t.minute(), t.second())
}

/// A shot name as a file name: letters, digits, spaces, dashes and underscores.
pub fn file_name(name: &str) -> String {
    let s: String = name.chars().filter(|c| c.is_alphanumeric() || matches!(c, ' ' | '-' | '_')).collect();
    let s = s.trim().to_string();
    if s.is_empty() { "shot".into() } else { s }
}

impl Director {
    pub fn say(&mut self, s: impl Into<String>) {
        self.note = Some((s.into(), Instant::now()));
    }

    fn load(&mut self) {
        if std::mem::replace(&mut self.loaded, true) {
            return;
        }
        if let Some(p) = std::fs::read_to_string(prefs_path()).ok().and_then(|s| serde_json::from_str::<Prefs>(&s).ok())
        {
            self.prefs = p;
        }
        self.prefs.bookmarks.resize(9, None);
        self.lapse_hour = self.prefs.scene.hour;
        self.refresh_library();
    }

    pub fn save_prefs(&self) {
        if let Ok(s) = serde_json::to_string_pretty(&self.prefs) {
            let _ = std::fs::write(prefs_path(), s);
        }
    }

    pub fn refresh_library(&mut self) {
        let mut names: Vec<String> = std::fs::read_dir(shots_dir())
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|e| {
                let p = e.path();
                if p.extension()? != "json" {
                    return None;
                }
                Some(p.file_stem()?.to_string_lossy().into_owned())
            })
            .collect();
        names.sort_by_key(|n| n.to_lowercase());
        self.library = names;
    }

    /// The path's length in seconds: every key's hold, and the travel between keys (the last key's travel only when
    /// looping back to the first).
    pub fn path_seconds(&self) -> f32 {
        path_seconds(&self.keys, self.looped)
    }

    /// Whether the window should be drawn this frame.
    pub fn panel_shown(&self) -> bool {
        self.open
            && !self.panel_hidden
            && self.photo.is_none()
            && (self.peek || !self.auto_hidden())
            && self.countdown.is_none()
    }

    /// Tucked away by itself for a take or a playing shot.
    pub fn auto_hidden(&self) -> bool {
        self.prefs.auto_hide && (self.playing.is_some() || self.recording.is_some())
    }

    /// Guides (thirds, safe area, centre) are for framing: never in a take or a photo.
    pub fn guides_shown(&self) -> bool {
        self.open
            && self.playing.is_none()
            && self.recording.is_none()
            && self.photo.is_none()
            && self.countdown.is_none()
    }
}

/// Opens or closes the window. Closing puts the world back: the game camera, the server's time and weather, every
/// actor shown, the HUD as it was; a recording ends.
pub fn toggle(w: &mut World) {
    if w.director.open {
        close(w);
    } else {
        open(w);
    }
}

pub fn open(w: &mut World) {
    let d = &mut w.director;
    d.load();
    d.open = true;
    d.panel_hidden = false;
    if d.ffmpeg.load(std::sync::atomic::Ordering::Relaxed) == FF_UNKNOWN {
        check_ffmpeg(d.ffmpeg.clone());
    }
    // The free camera starts where the game camera is, with its lens.
    if let Some(c) = w.last_camera {
        w.director.rig = Rig::from_camera(&c);
    }
    w.director.zone = w.zone;
}

pub fn close(w: &mut World) {
    stop_recording(w);
    let d = &mut w.director;
    d.save_prefs();
    d.open = false;
    d.playing = None;
    d.countdown = None;
    d.mode = GAME;
    if std::mem::take(&mut d.hud_ours) {
        w.ui_hidden = false;
    }
    restore_weather(w);
}

/// `ffmpeg -version` on a thread: is video possible on this PC?
fn check_ffmpeg(state: std::sync::Arc<std::sync::atomic::AtomicU8>) {
    use std::sync::atomic::Ordering;
    state.store(FF_CHECKING, Ordering::Relaxed);
    std::thread::spawn(move || {
        let mut cmd = std::process::Command::new("ffmpeg");
        cmd.arg("-version").stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x0800_0000);
        }
        let ok = cmd.status().is_ok_and(|s| s.success());
        state.store(if ok { FF_OK } else { FF_MISSING }, Ordering::Relaxed);
    });
}

/// The Director's keys (Ctrl+Shift with a letter), wherever the chat line is closed. True when taken.
pub fn key(w: &mut World, key: egui::Key, m: egui::Modifiers) -> bool {
    if !(m.ctrl || m.command) || !m.shift || m.alt {
        return false;
    }
    use egui::Key as K;
    if key == K::D {
        toggle(w);
        return true;
    }
    if !w.director.open {
        return false;
    }
    match key {
        K::H if w.director.auto_hidden() => w.director.peek = !w.director.peek,
        K::H => w.director.panel_hidden = !w.director.panel_hidden,
        K::F => {
            let to = if w.director.mode == FLY { GAME } else { FLY };
            set_mode(w, to);
        }
        K::K => add_key(w),
        K::P => play_toggle(w),
        K::R => record_toggle(w),
        K::S => photo(w),
        _ => return false,
    }
    true
}

/// A new camera mode; the free camera starts where the camera is now.
pub fn set_mode(w: &mut World, mode: u8) {
    if mode == w.director.mode {
        return;
    }
    if mode == FLY
        && let Some(c) = current_camera(w)
    {
        let (fov, roll) = (w.director.rig.fov, w.director.rig.roll);
        w.director.rig = Rig::from_camera(&c);
        (w.director.rig.fov, w.director.rig.roll) = (fov, roll);
    }
    if mode == ORBIT && w.director.orbit.subject == FREE {
        orbit_here(w);
    }
    w.director.playing = None;
    w.director.mode = mode;
}

/// A fixed-point orbit around what the camera looks at now, starting from where it is.
pub fn orbit_here(w: &mut World) {
    let Some(c) = current_camera(w) else { return };
    let o = &mut w.director.orbit;
    let (yaw, pitch) = angles_of(c.target - c.eye);
    (o.point, o.yaw, o.pitch, o.height) = (c.target, yaw, -pitch, 0.0);
    o.dist = c.eye.distance(c.target).max(1.0);
}

/// The camera as drawn last frame (the director's own when it was in charge).
fn current_camera(w: &World) -> Option<Camera> {
    w.last_camera
}

/// The subject's place: feet, and the way it faces.
fn subject(w: &World, who: u8) -> Option<(Vec3, Vec3)> {
    let id = match who {
        ME => w.my_id,
        TARGET => w.target?,
        _ => return None,
    };
    let my_dir = w.net.me.lock().map(|m| m.dir).unwrap_or(0);
    if let Some(p) = super::super::actor_place(w, id, my_dir) {
        return Some((p.pos, p.forward));
    }
    if id == w.my_id {
        let p = super::super::drawn_pos(w);
        return Some((p, Vec3::new(w.yaw.sin(), 0.0, w.yaw.cos())));
    }
    let e = w.entities.get(&id)?;
    Some((e.pos, Vec3::Z))
}

/// A subject's height (its model's top), for aiming at the chest and framing.
fn subject_height(w: &World, who: u8) -> f32 {
    let id = match who {
        ME => Some(w.my_id),
        TARGET => w.target,
        _ => None,
    };
    id.and_then(|id| w.actors.get(&id)).map(|a| -a.top).filter(|h| *h > 0.2).unwrap_or(1.6).min(12.0)
}

/// Where an aim lock looks: the subject's chest.
fn aim_point(w: &World, who: u8) -> Option<Vec3> {
    let (feet, _) = subject(w, who)?;
    Some(feet + Vec3::NEG_Y * subject_height(w, who) * 0.62)
}

/// The key the camera is at now (whatever mode drives it).
pub fn key_now(w: &World) -> Option<Key> {
    let c = camera(w).or(w.last_camera)?;
    let (yaw, pitch) = angles_of(c.target - c.eye);
    let roll = if w.director.mode == GAME { 0.0 } else { w.director.rig.roll };
    Some(Key { eye: c.eye.to_array(), yaw, pitch, fov: c.fov_y.to_degrees(), roll, secs: 4.0, hold: 0.0 })
}

/// Adds the camera's place as a key after the selected one (or at the end).
pub fn add_key(w: &mut World) {
    let Some(k) = key_now(w) else { return };
    let d = &mut w.director;
    let at = d.selected.map_or(d.keys.len(), |s| (s + 1).min(d.keys.len()));
    let secs = d.keys.get(at.wrapping_sub(1)).map_or(4.0, |p| p.secs);
    d.keys.insert(at, Key { secs, ..k });
    d.selected = Some(at);
    let n = d.keys.len();
    d.say(format!("Key {} added ({n} in the shot).", at + 1));
}

/// Puts the camera on a key (the free camera takes it).
pub fn go_to(w: &mut World, k: Key) {
    w.director.playing = None;
    if w.director.mode != FLY {
        w.director.mode = FLY;
    }
    w.director.rig.set(&k);
}

pub fn play_toggle(w: &mut World) {
    let d = &mut w.director;
    if d.playing.is_some() || d.countdown.is_some_and(|c| c.1 == Cue::Play) {
        d.playing = None;
        d.countdown = None;
        return;
    }
    if d.keys.len() < 2 {
        d.say("A shot needs at least two keys to play.");
        return;
    }
    let secs = d.prefs.countdown as f32;
    if secs > 0.0 { d.countdown = Some((secs, Cue::Play)) } else { start_play(w) }
}

fn start_play(w: &mut World) {
    let d = &mut w.director;
    if d.keys.len() < 2 {
        return;
    }
    d.playing = Some(0.0);
}

pub fn record_toggle(w: &mut World) {
    if w.director.recording.is_some() {
        stop_recording(w);
        return;
    }
    if w.director.countdown.is_some() {
        w.director.countdown = None;
        return;
    }
    record_start(w, false);
}

/// Starts a take: after the countdown, the recording (and with `with_path`, the shot playing, the recording ending
/// with it).
pub fn record_start(w: &mut World, with_path: bool) {
    use std::sync::atomic::Ordering;
    let d = &mut w.director;
    if d.ffmpeg.load(Ordering::Relaxed) == FF_MISSING {
        d.say(
            "Video needs ffmpeg: install it (winget install ffmpeg) and restart Timeless XI. Photos work without it.",
        );
        return;
    }
    if with_path && d.keys.len() < 2 {
        d.say("A shot needs at least two keys to film.");
        return;
    }
    let cue = if with_path { Cue::RecordPath } else { Cue::Record };
    let secs = d.prefs.countdown as f32;
    if secs > 0.0 {
        d.countdown = Some((secs, cue));
    } else {
        fire(w, cue);
    }
}

fn fire(w: &mut World, cue: Cue) {
    match cue {
        Cue::Play => start_play(w),
        Cue::Record | Cue::RecordPath => {
            let name = if w.director.shot_name.trim().is_empty() {
                format!("Timeless XI {}", stamp())
            } else {
                format!("{} {}", file_name(&w.director.shot_name), stamp())
            };
            let path = video_dir().join(format!("{name}.mp4"));
            let fps = w.director.prefs.fps.clamp(24, 120);
            w.lua.record_req = Some((OWNER.into(), Some((path.clone(), fps))));
            w.director.recording = Some((path, Instant::now(), cue == Cue::RecordPath));
            if cue == Cue::RecordPath {
                start_play(w);
            }
        }
    }
}

/// The recorder's owner name for the Director's takes (client/record.rs keeps one recording per owner).
pub const OWNER: &str = "Director";

pub fn stop_recording(w: &mut World) {
    if let Some((path, at, _)) = w.director.recording.take() {
        w.lua.record_req = Some((OWNER.into(), None));
        let secs = at.elapsed().as_secs_f32();
        w.director.say(format!("Saved {:.0} s of video to {}", secs, path.display()));
    }
}

/// A full-size photo (the window and the HUD out of it), to Pictures\Timeless XI.
pub fn photo(w: &mut World) {
    let name = if w.director.shot_name.trim().is_empty() {
        "Timeless XI".to_string()
    } else {
        file_name(&w.director.shot_name)
    };
    let path = photo_dir().join(format!("{name} {}.png", stamp()));
    let hud = w.ui_hidden;
    w.ui_hidden = true;
    w.director.photo = Some((path, 2, hud));
}

/// The photo due this frame, if one is (App::shot_requests): taken once the window and the HUD have been gone for
/// two frames.
pub fn photo_due(w: &mut World) -> Option<PathBuf> {
    let (path, wait, hud) = w.director.photo.take()?;
    match wait {
        // Taken last frame (the window and the HUD were kept out of that frame too): everything comes back.
        TAKEN => {
            w.ui_hidden = hud;
            let shown = path.display().to_string();
            w.director.say(format!("Photo saved to {shown}"));
            None
        }
        0 => {
            w.director.photo = Some((path.clone(), TAKEN, hud));
            Some(path)
        }
        n => {
            w.director.photo = Some((path, n - 1, hud));
            None
        }
    }
}

/// `Director::photo`'s wait once the photo is taken (the frame after puts the window and the HUD back).
const TAKEN: u8 = u8::MAX;

/// Saves the path and the scene as a shot by the name in the box.
pub fn save_shot(w: &mut World) {
    let d = &mut w.director;
    let name = file_name(&d.shot_name);
    let f = ShotFile {
        name: name.clone(),
        zone: w.zone,
        keys: d.keys.clone(),
        ease: d.ease,
        aim: d.aim,
        looped: d.looped,
        scene: d.prefs.scene.clone(),
    };
    let dir = shots_dir();
    let _ = std::fs::create_dir_all(&dir);
    let out = serde_json::to_string_pretty(&f)
        .map_err(std::io::Error::other)
        .and_then(|s| std::fs::write(dir.join(format!("{name}.json")), s));
    match out {
        Ok(()) => d.say(format!("Saved shot \"{name}\".")),
        Err(e) => d.say(format!("Couldn't save \"{name}\": {e}")),
    }
    d.shot_name = name;
    d.refresh_library();
}

pub fn load_shot(w: &mut World, name: &str) {
    let path = shots_dir().join(format!("{name}.json"));
    let f = std::fs::read_to_string(&path).ok().and_then(|s| serde_json::from_str::<ShotFile>(&s).ok());
    let zone = w.zone;
    let d = &mut w.director;
    let Some(f) = f else {
        d.say(format!("Couldn't read \"{name}\"."));
        return;
    };
    d.keys = f.keys;
    (d.ease, d.aim, d.looped) = (f.ease, f.aim, f.looped);
    d.prefs.scene = f.scene;
    d.lapse_hour = d.prefs.scene.hour;
    d.shot_name = f.name.clone();
    d.selected = None;
    d.playing = None;
    let elsewhere = f.zone.is_some() && f.zone != zone;
    d.say(if elsewhere {
        format!("Loaded \"{}\". It was set in another zone: its keys are where they were there.", f.name)
    } else {
        format!("Loaded \"{}\" ({} keys).", f.name, d.keys.len())
    });
}

pub fn delete_shot(w: &mut World, name: &str) {
    let _ = std::fs::remove_file(shots_dir().join(format!("{name}.json")));
    w.director.say(format!("Deleted \"{name}\"."));
    w.director.refresh_library();
}

/// The path as a shot of the Timeless title reel (timeless-title.json, which the title screen plays when it exists):
/// replaced when a shot of that name is there, else added.
pub fn export_reel(w: &mut World) {
    use super::title::{KeyJson, Move, Reel, Shot};
    let d = &w.director;
    let Some(zone) = w.zone else { return };
    if d.keys.len() < 2 {
        w.director.say("A shot needs at least two keys for the title reel.");
        return;
    }
    let id = file_name(&d.shot_name).to_lowercase().replace(' ', "_");
    let s = &d.prefs.scene;
    let hour = if s.time_on { s.hour } else { clock_hour(w) };
    let weather = crate::dat::weather::weather_tag(if s.weather_on { s.weather } else { w.weather }).to_string();
    let keys: Vec<KeyJson> = d
        .keys
        .iter()
        .map(|k| {
            let eye = Vec3::from(k.eye);
            KeyJson {
                eye: k.eye,
                target: (eye + dir_of(k.yaw, k.pitch) * 20.0).to_array(),
                focal: crate::dat::title::focal_of(k.fov.to_radians()),
                roll: k.roll.to_radians(),
            }
        })
        .collect();
    let ease = match d.ease {
        EASE_LINEAR => "linear",
        _ => "inout",
    };
    let shot = Shot {
        id: id.clone(),
        zone: zone as u32,
        hour,
        weather,
        logo: None,
        fade_in: 3.0,
        fade_out: 3.0,
        all_scenery: s.all_scenery,
        note: Some("Made in Director Mode.".into()),
        moves: vec![Move { seconds: d.path_seconds().max(1.0), ease: ease.into(), cross: 0.0, keys }],
    };
    let mut reel = Reel::load();
    match reel.shots.iter_mut().find(|x| x.id == id) {
        Some(x) => *x = shot,
        None => reel.shots.push(shot),
    }
    let msg = match reel.save() {
        Ok(()) => format!("Added \"{id}\" to your title reel. It plays on the title screen next time."),
        Err(e) => format!("Couldn't write the title reel: {e}"),
    };
    w.director.say(msg);
}

/// The server clock's hour now.
fn clock_hour(w: &World) -> f32 {
    w.clock.map_or(12.0, |(t, at)| crate::dat::weather::vana_minute(t, at.elapsed().as_secs_f32()) / 60.0)
}

/// Saves the camera's place in a bookmark slot.
pub fn set_bookmark(w: &mut World, slot: usize) {
    let Some(k) = key_now(w) else { return };
    let zone = w.zone;
    let d = &mut w.director;
    if let Some(b) = d.prefs.bookmarks.get_mut(slot) {
        *b = Some((zone, k));
    }
    d.save_prefs();
    d.say(format!("Bookmark {} set.", slot + 1));
}

// ---------------------------------------------------------------------------------------------------------------
// The frame.

/// Once a frame, before the camera is read: the free camera flown, the path played, the countdown, the time-lapse,
/// the weather, the HUD.
pub fn step(w: &mut World, dt: f32, input: FlyInput) {
    if !w.director.open {
        return;
    }
    let dt = dt.clamp(0.0, 0.25);
    // A new zone: the free camera back where the game camera is, the path stopped.
    if w.director.zone != w.zone {
        w.director.zone = w.zone;
        w.director.playing = None;
        if let Some(c) = w.last_camera.filter(|_| w.director.mode == GAME) {
            let (fov, roll) = (w.director.rig.fov, w.director.rig.roll);
            w.director.rig = Rig::from_camera(&c);
            (w.director.rig.fov, w.director.rig.roll) = (fov, roll);
        } else if w.director.mode != GAME {
            w.director.mode = GAME;
            w.director.say("New zone: back to the game camera.");
        }
        w.director.applied_weather = None;
    }
    // The countdown.
    if let Some((left, cue)) = w.director.countdown {
        let left = left - dt;
        if left <= 0.0 {
            w.director.countdown = None;
            fire(w, cue);
        } else {
            w.director.countdown = Some((left, cue));
        }
    }
    let typing = w.typing || w.director.typing;
    let d = &mut w.director;
    // The free camera.
    if d.mode == FLY && d.playing.is_none() && !typing {
        let base = d.prefs.speed.clamp(0.05, 400.0);
        let speed = base * if input.fast { 4.0 } else { 1.0 } * if input.slow { 0.25 } else { 1.0 };
        let f = if d.prefs.level_flight { dir_of(d.rig.yaw, 0.0) } else { dir_of(d.rig.yaw, d.rig.pitch) };
        let right = dir_of(d.rig.yaw + std::f32::consts::FRAC_PI_2, 0.0);
        let want = (f * input.forward + right * input.right + Vec3::NEG_Y * input.up) * speed;
        // Smoothing 0 stops at once; 0.95 glides.
        let k = 1.0 - (-dt * (1.0 - d.prefs.smoothing.clamp(0.0, 0.97)) * 18.0).exp();
        d.rig.vel = d.rig.vel.lerp(want, k);
        d.rig.eye += d.rig.vel * dt;
        d.rig.eye = d.rig.eye.clamp(Vec3::splat(-REACH), Vec3::splat(REACH));
        // The arrows turn and tilt (90 degrees a second).
        d.rig.want.0 += input.turn * dt * 1.6;
        d.rig.want.1 = (d.rig.want.1 + input.tilt * dt * 1.2).clamp(-1.55, 1.55);
    } else {
        d.rig.vel = Vec3::ZERO;
    }
    if d.mode == FLY && d.playing.is_none() {
        let k = 1.0 - (-dt * (1.0 - d.prefs.look_smoothing.clamp(0.0, 0.95)) * 30.0).exp();
        d.rig.yaw += (d.rig.want.0 - d.rig.yaw) * k;
        d.rig.pitch += (d.rig.want.1 - d.rig.pitch) * k;
    }
    // The orbit's own turn.
    if d.mode == ORBIT && d.orbit.spin != 0.0 {
        d.orbit.yaw = (d.orbit.yaw + d.orbit.spin.to_radians() * dt).rem_euclid(std::f32::consts::TAU);
    }
    // The follow camera eases toward its place.
    if d.mode == FOLLOW {
        let fo = d.follow;
        if let Some((feet, fwd)) = subject(w, fo.subject) {
            let h = subject_height(w, fo.subject);
            let fwd = Vec3::new(fwd.x, 0.0, fwd.z).try_normalize().unwrap_or(Vec3::Z);
            let right = Vec3::new(fwd.z, 0.0, -fwd.x);
            let goal = feet + right * fo.side + Vec3::NEG_Y * fo.up - fwd * fo.back;
            let look = feet + Vec3::NEG_Y * h * 0.62 + fwd * fo.ahead;
            let d = &mut w.director;
            let k = 1.0 - (-dt * fo.ease.clamp(0.2, 20.0)).exp();
            if d.rig.eye.distance(goal) > 60.0 {
                d.rig.eye = goal;
            }
            d.rig.eye = d.rig.eye.lerp(goal, k);
            let (yaw, pitch) = angles_of(look - d.rig.eye);
            d.rig.yaw = yaw;
            d.rig.pitch = pitch;
        }
    }
    // The path.
    let d = &mut w.director;
    if let Some(t) = d.playing {
        let total = d.path_seconds();
        let t = t + dt * d.play_speed.clamp(0.05, 8.0);
        if t >= total {
            if d.looped && total > 0.0 {
                d.playing = Some(t % total);
            } else {
                d.playing = None;
                // The camera stays on the last key.
                if let Some(k) = d.keys.last().copied() {
                    d.mode = FLY;
                    d.rig.set(&k);
                }
                if d.recording.as_ref().is_some_and(|r| r.2) {
                    stop_recording(w);
                }
            }
        } else {
            d.playing = Some(t);
        }
    }
    // The time-lapse.
    let d = &mut w.director;
    if d.playing.is_none() && d.recording.is_none() {
        d.peek = false;
    }
    let s = &d.prefs.scene;
    if s.time_on && s.lapse != 0.0 {
        d.lapse_hour = (d.lapse_hour + s.lapse * dt).rem_euclid(24.0);
    } else {
        d.lapse_hour = s.hour;
    }
    // The HUD.
    let hide = d.prefs.scene.hide_hud;
    if d.photo.is_none() {
        if hide && !w.ui_hidden {
            w.ui_hidden = true;
            w.director.hud_ours = true;
        } else if !hide && w.director.hud_ours {
            w.ui_hidden = false;
            w.director.hud_ours = false;
        }
    }
    // The weather's effects (rain, snow, the sky's particles) follow the director's weather.
    let want = weather(w);
    if want != w.director.applied_weather {
        let blend = w.director.prefs.scene.blend.max(0.0);
        match want {
            Some(n) => w.effects.set_weather(crate::dat::weather::weather_tag(n), blend),
            None => {
                let n = super::super::event::weather(w);
                w.effects.set_weather(crate::dat::weather::weather_tag(n), blend);
            }
        }
        w.director.applied_weather = want;
    }
    // A take the recorder isn't making (ffmpeg missing or failed, the window resized): let it go and say so.
    if let Some((_, at, _)) = &w.director.recording
        && at.elapsed().as_secs_f32() > 1.5
        && w.lua.recording.borrow().as_ref().is_none_or(|r| r.0 != OWNER)
    {
        w.director.recording = None;
        w.director.say("The recording stopped: is ffmpeg installed? Resizing the window also ends a recording.");
    }
    if w.director.note.as_ref().is_some_and(|n| n.1.elapsed().as_secs() > 8) {
        w.director.note = None;
    }
}

fn restore_weather(w: &mut World) {
    if w.director.applied_weather.take().is_some() {
        let n = super::super::event::weather(w);
        w.effects.set_weather(crate::dat::weather::weather_tag(n), 2.0);
    }
}

/// The mouse drag while the director's camera is in charge (the game camera keeps its own). True when taken.
pub fn drag(w: &mut World, dx: f64, dy: f64, middle: bool) -> bool {
    let d = &mut w.director;
    if !d.open || d.playing.is_some() {
        return false;
    }
    let (dx, dy) = (dx as f32, dy as f32);
    match d.mode {
        FLY if middle => {
            let right = dir_of(d.rig.yaw + std::f32::consts::FRAC_PI_2, 0.0);
            let k = d.prefs.speed * 0.004;
            d.rig.eye += right * dx * k + Vec3::Y * dy * k;
        }
        FLY => {
            d.rig.want.0 += dx * 0.0035 * (d.rig.fov / 45.0);
            d.rig.want.1 = (d.rig.want.1 - dy * 0.0035 * (d.rig.fov / 45.0)).clamp(-1.55, 1.55);
        }
        ORBIT if middle => d.orbit.height = (d.orbit.height - dy * 0.01).clamp(-5.0, 40.0),
        ORBIT => {
            d.orbit.yaw -= dx * 0.006;
            d.orbit.pitch = (d.orbit.pitch + dy * 0.005).clamp(-1.4, 1.5);
        }
        FOLLOW => {
            d.follow.side = (d.follow.side + dx * 0.01).clamp(-20.0, 20.0);
            d.follow.up = (d.follow.up - dy * 0.01).clamp(-3.0, 20.0);
        }
        _ => return false,
    }
    true
}

/// The wheel while the director's camera is in charge: the free camera's speed, the orbit's distance, the follow's
/// distance behind. True when taken.
pub fn wheel(w: &mut World, notches: f32) -> bool {
    let d = &mut w.director;
    if !d.open || notches == 0.0 {
        return d.open && d.mode != GAME;
    }
    match d.mode {
        FLY => {
            d.prefs.speed = (d.prefs.speed * 1.18f32.powf(notches)).clamp(0.05, 400.0);
            let s = d.prefs.speed;
            d.say(format!("Fly speed {s:.1} yalms a second"));
        }
        ORBIT => d.orbit.dist = (d.orbit.dist * (1.0 - notches * 0.08)).clamp(0.3, 400.0),
        FOLLOW => d.follow.back = (d.follow.back - notches * 0.3).clamp(-30.0, 60.0),
        _ => return false,
    }
    true
}

/// Whether the character stays put: the free camera has the movement keys (WASD, Q / E), unless the window lets the
/// character walk.
pub fn holds_movement(w: &World) -> bool {
    let d = &w.director;
    d.open && d.mode == FLY && d.playing.is_none() && d.prefs.hold_character && !w.typing && !d.typing
}

/// Handheld shake at time `t`: small yaw / pitch / roll wanders (degrees x amount), from sums of sines.
fn shake_at(s: &Shake, t: f32) -> (f32, f32, f32) {
    if s.amount <= 0.0 {
        return (0.0, 0.0, 0.0);
    }
    let f = s.speed.clamp(0.05, 5.0);
    let n = |a: f32, b: f32, c: f32| {
        (t * f * a).sin() * 0.55 + (t * f * b + 1.3).sin() * 0.3 + (t * f * c + 2.1).sin() * 0.15
    };
    let k = s.amount.to_radians();
    (n(1.1, 2.7, 6.3) * k, n(0.9, 2.3, 5.1) * k * 0.7, n(0.7, 1.9, 4.4) * s.amount * 0.5)
}

/// The director's camera, None when the game's has the view.
pub fn camera(w: &World) -> Option<Camera> {
    let d = &w.director;
    if !d.open {
        return None;
    }
    let mut key = if let Some(t) = d.playing.or(d.scrub) {
        let mut k = sample(&d.keys, t, d.ease, d.looped)?;
        if d.aim != FREE
            && let Some(p) = aim_point(w, d.aim)
        {
            (k.yaw, k.pitch) = angles_of(p - Vec3::from(k.eye));
        }
        k
    } else {
        match d.mode {
            FLY | FOLLOW => {
                let mut k = d.rig.key();
                if d.mode == FLY
                    && d.aim != FREE
                    && let Some(p) = aim_point(w, d.aim)
                {
                    (k.yaw, k.pitch) = angles_of(p - d.rig.eye);
                }
                k
            }
            ORBIT => {
                let o = &d.orbit;
                let centre = match o.subject {
                    FREE => o.point,
                    s => subject(w, s).map(|(feet, _)| feet)?,
                } + Vec3::NEG_Y * o.height;
                let eye = centre - dir_of(o.yaw, -o.pitch) * o.dist;
                let (yaw, pitch) = angles_of(centre - eye);
                Key { eye: eye.to_array(), yaw, pitch, fov: d.rig.fov, roll: d.rig.roll, secs: 0.0, hold: 0.0 }
            }
            _ => return None,
        }
    };
    let (sy, sp, sr) = shake_at(&d.shake, d.started.elapsed().as_secs_f32());
    key.yaw += sy;
    key.pitch += sp;
    key.roll += sr;
    let dir = dir_of(key.yaw, key.pitch);
    let eye = Vec3::from(key.eye);
    Some(Camera {
        eye,
        target: eye + dir * 10.0,
        up: up_of(dir, key.roll),
        fov_y: key.fov.clamp(5.0, 120.0).to_radians(),
    })
}

/// Whether the zone's whole scenery is drawn (no camera list).
pub fn all_scenery(w: &World) -> bool {
    w.director.open && w.director.prefs.scene.all_scenery
}

/// The weather in force from the director, if it sets one.
pub fn weather(w: &World) -> Option<u16> {
    let d = &w.director;
    (d.open && d.prefs.scene.weather_on).then_some(d.prefs.scene.weather.min(19))
}

/// The clock the director holds (the hour, the time-lapse, the moon), if it holds one; the lighting and the sky's
/// effects read it as they read an event's held clock.
pub fn clock(w: &World) -> Option<super::super::event::HeldClock> {
    let d = &w.director;
    let s = &d.prefs.scene;
    if !d.open || !(s.time_on || s.moon_on) {
        return None;
    }
    let minute = if s.time_on {
        (d.lapse_hour.rem_euclid(24.0) * 60.0) as u32 % 1440
    } else {
        (clock_hour(w) * 60.0) as u32 % 1440
    };
    let day = s.moon_on.then(|| moon_day(s.moon, minute));
    Some(super::super::event::HeldClock { day, minute, moon: s.moon_on })
}

/// A day whose moon is phase `phase` (0..11) at `minute` (event.rs HeldClock::moon_weekday: ((day x 1440 + minute +
/// 720) / 1440 / 7) % 12).
pub fn moon_day(phase: u8, minute: u32) -> u32 {
    let base = (phase as u32 % 12) * 7 + 12 * 7 * 100;
    // The +720 rolls past midnight after noon: start a day early then.
    if minute >= 720 { base + 2 } else { base + 3 }
}

/// The grade in force, if the look is on.
pub fn grade(w: &World) -> Option<crate::render::nostalgia::Grade> {
    let d = &w.director;
    let l = &d.prefs.scene.look;
    (d.open && l.on && !l.is_neutral()).then(|| l.grade())
}

/// Director Mode's hiding (event::hide adds these): ourselves, other players (the party kept if asked), NPCs,
/// monsters, Companions. Doors, lifts and ships stay.
pub fn hide(w: &World, out: &mut std::collections::HashSet<u32>) {
    let d = &w.director;
    let s = &d.prefs.scene;
    if !d.open {
        return;
    }
    if s.hide_me {
        out.insert(w.my_id);
    }
    if !(s.hide_players || s.hide_npcs || s.hide_mobs || s.hide_companions) {
        return;
    }
    for (id, e) in &w.entities {
        if *id == w.my_id || e.door != 0 {
            continue;
        }
        let pc = id >> 24 == 0;
        let hide = if pc {
            s.hide_players && !(s.keep_party && w.party.members().any(|m| m.id == *id))
        } else if e.marks.trust {
            s.hide_companions
        } else if e.mob {
            s.hide_mobs
        } else {
            s.hide_npcs && w.actors.get(id).is_some_and(|a| a.character.is_some())
        };
        if hide {
            out.insert(*id);
        }
    }
}

/// The path drawn in the world: the curve in pale blue, each key's place and the way it looks, the selected one gold.
pub fn path_lines(w: &World) -> Vec<crate::render::LineVertex> {
    let d = &w.director;
    let mut out = Vec::new();
    if !d.open || !d.show_path || d.keys.is_empty() || !d.guides_shown() || d.scrub.is_some() {
        return out;
    }
    let line = |out: &mut Vec<crate::render::LineVertex>, a: Vec3, b: Vec3, c: [f32; 4]| {
        out.push(crate::render::LineVertex { pos: a.to_array(), color: c });
        out.push(crate::render::LineVertex { pos: b.to_array(), color: c });
    };
    let curve = [0.56, 0.70, 0.90, 1.0];
    let total = d.path_seconds();
    if d.keys.len() >= 2 && total > 0.0 {
        let n = ((total * 12.0) as usize).clamp(16, 1200);
        let mut last: Option<Vec3> = None;
        for i in 0..=n {
            let t = total * i as f32 / n as f32;
            if let Some(k) = sample(&d.keys, t, EASE_LINEAR, d.looped) {
                let p = Vec3::from(k.eye);
                if let Some(q) = last {
                    line(&mut out, q, p, curve);
                }
                last = Some(p);
            }
        }
    }
    for (i, k) in d.keys.iter().enumerate() {
        let p = Vec3::from(k.eye);
        let c = if d.selected == Some(i) { [0.84, 0.88, 0.94, 1.0] } else { [0.89, 0.79, 0.56, 1.0] };
        let s = 0.25;
        line(&mut out, p - Vec3::X * s, p + Vec3::X * s, c);
        line(&mut out, p - Vec3::Y * s, p + Vec3::Y * s, c);
        line(&mut out, p - Vec3::Z * s, p + Vec3::Z * s, c);
        line(&mut out, p, p + dir_of(k.yaw, k.pitch) * 1.2, c);
    }
    out
}

// ---------------------------------------------------------------------------------------------------------------
// The path's timing and spline.

/// The path's length: holds, and travel between keys (around to the first when looping).
pub fn path_seconds(keys: &[Key], looped: bool) -> f32 {
    let n = keys.len();
    if n == 0 {
        return 0.0;
    }
    let travel: f32 = keys.iter().take(if looped { n } else { n - 1 }).map(|k| k.secs.max(0.05)).sum();
    let hold: f32 = keys.iter().map(|k| k.hold.max(0.0)).sum();
    travel + hold
}

fn smooth(u: f32) -> f32 {
    let u = u.clamp(0.0, 1.0);
    u * u * (3.0 - 2.0 * u)
}

/// Gentle start and stop over the whole path that still moves at an even pace in the middle: smoothstep on the first
/// and last fifth, linear between, its slope matched.
fn ease_path(u: f32) -> f32 {
    let u = u.clamp(0.0, 1.0);
    let a = 0.2;
    // The speed ramps up over [0, a] by a smoothstep, holds v = 1 / (1 - a), and ramps down over [1 - a, 1]; the
    // place is its integral: the ramp's is x^3 - x^4 / 2 (of 3x^2 - 2x^3), 1/2 over the whole ramp.
    let v = 1.0 / (1.0 - a);
    if u < a {
        let x = u / a;
        v * a * (x * x * x - 0.5 * x * x * x * x)
    } else if u > 1.0 - a {
        1.0 - ease_path(1.0 - u)
    } else {
        v * a * 0.5 + v * (u - a)
    }
}

/// Centripetal Catmull-Rom between p1 and p2 at u (Barry-Goldman).
fn catmull(p0: Vec3, p1: Vec3, p2: Vec3, p3: Vec3, u: f32) -> Vec3 {
    let knot = |a: Vec3, b: Vec3| a.distance(b).sqrt().max(1e-4);
    let t0 = 0.0;
    let t1 = t0 + knot(p0, p1);
    let t2 = t1 + knot(p1, p2);
    let t3 = t2 + knot(p2, p3);
    let t = t1 + (t2 - t1) * u;
    let lerp = |a: Vec3, b: Vec3, ta: f32, tb: f32| {
        if (tb - ta).abs() < 1e-6 { a } else { a * ((tb - t) / (tb - ta)) + b * ((t - ta) / (tb - ta)) }
    };
    let a1 = lerp(p0, p1, t0, t1);
    let a2 = lerp(p1, p2, t1, t2);
    let a3 = lerp(p2, p3, t2, t3);
    let b1 = lerp(a1, a2, t0, t2);
    let b2 = lerp(a2, a3, t1, t3);
    lerp(b1, b2, t1, t2)
}

/// Uniform Catmull-Rom for the angles and lens (no overshoot worth worrying about at these spacings).
fn catmull_u(p0: f32, p1: f32, p2: f32, p3: f32, u: f32) -> f32 {
    let (u2, u3) = (u * u, u * u * u);
    0.5 * ((2.0 * p1)
        + (-p0 + p2) * u
        + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * u2
        + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * u3)
}

/// The keys' yaws unwrapped so each turns the short way from the one before.
fn unwrapped(keys: &[Key]) -> Vec<f32> {
    let mut out: Vec<f32> = Vec::with_capacity(keys.len());
    for k in keys {
        let y = match out.last() {
            Some(&p) => {
                let mut d = (k.yaw - p).rem_euclid(std::f32::consts::TAU);
                if d > std::f32::consts::PI {
                    d -= std::f32::consts::TAU;
                }
                p + d
            }
            None => k.yaw,
        };
        out.push(y);
    }
    out
}

/// The camera on the path at `t` seconds (0..path_seconds), with the path's easing.
pub fn sample(keys: &[Key], t: f32, ease: u8, looped: bool) -> Option<Key> {
    let n = keys.len();
    if n == 0 {
        return None;
    }
    if n == 1 {
        return Some(keys[0]);
    }
    let total = path_seconds(keys, looped);
    let t = if ease == EASE_SMOOTH && !looped { ease_path(t / total.max(1e-3)) * total } else { t.clamp(0.0, total) };
    // Find the segment: each key holds, then travels to the next.
    let segs = if looped { n } else { n - 1 };
    let mut at = 0.0;
    let mut seg = segs - 1;
    let mut u = 1.0;
    let mut held = None;
    for i in 0..segs {
        let hold = keys[i].hold.max(0.0);
        if t < at + hold {
            held = Some(i);
            break;
        }
        at += hold;
        let s = keys[i].secs.max(0.05);
        if t < at + s || i == segs - 1 {
            seg = i;
            u = ((t - at) / s).clamp(0.0, 1.0);
            break;
        }
        at += s;
    }
    if let Some(i) = held {
        return Some(keys[i]);
    }
    if ease == EASE_EACH {
        u = smooth(u);
    }
    let idx = |i: isize| -> usize {
        if looped { i.rem_euclid(n as isize) as usize } else { i.clamp(0, n as isize - 1) as usize }
    };
    let s = seg as isize;
    let (i0, i1, i2, i3) = (idx(s - 1), idx(s), idx(s + 1), idx(s + 2));
    let eye = |i: usize| Vec3::from(keys[i].eye);
    // The ends repeat their point; give them a mirrored neighbour so the curve leaves straight.
    let p1 = eye(i1);
    let p2 = eye(i2);
    let p0 = if !looped && s == 0 { p1 * 2.0 - p2 } else { eye(i0) };
    let p3 = if !looped && s as usize + 1 >= n - 1 { p2 * 2.0 - p1 } else { eye(i3) };
    let pos = catmull(p0, p1, p2, p3, u);
    // Angles: unwrapped, with the loop's wrap carried across.
    let mut yaws = unwrapped(keys);
    let wrap_y = |i: isize, yaws: &Vec<f32>| -> f32 {
        let k = idx(i);
        let mut y = yaws[k];
        if looped {
            let turns = i.div_euclid(n as isize) as f32;
            let lap = {
                let mut d = (yaws[0] - yaws[n - 1]).rem_euclid(std::f32::consts::TAU);
                if d > std::f32::consts::PI {
                    d -= std::f32::consts::TAU;
                }
                yaws[n - 1] + d - yaws[0]
            };
            y += turns * lap;
        }
        y
    };
    let (y0, y1, y2, y3) = (wrap_y(s - 1, &yaws), wrap_y(s, &yaws), wrap_y(s + 1, &yaws), wrap_y(s + 2, &yaws));
    yaws.clear();
    let f = |g: &dyn Fn(&Key) -> f32| catmull_u(g(&keys[i0]), g(&keys[i1]), g(&keys[i2]), g(&keys[i3]), u);
    let yaw = catmull_u(y0, y1, y2, y3, u);
    let pitch = f(&|k| k.pitch).clamp(-1.56, 1.56);
    // The lens and roll ease between their two keys only (no overshoot on a zoom).
    let lu = smooth(u);
    let fov = keys[i1].fov + (keys[i2].fov - keys[i1].fov) * lu;
    let roll = keys[i1].roll + (keys[i2].roll - keys[i1].roll) * lu;
    Some(Key { eye: pos.to_array(), yaw, pitch, fov, roll, secs: 0.0, hold: 0.0 })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn k(x: f32, yaw: f32) -> Key {
        Key { eye: [x, 0.0, 0.0], yaw, pitch: 0.0, fov: 45.0, roll: 0.0, secs: 2.0, hold: 0.0 }
    }

    #[test]
    fn path_passes_through_its_keys() {
        let keys = vec![k(0.0, 0.0), k(10.0, 0.5), k(20.0, 1.0)];
        assert_eq!(path_seconds(&keys, false), 4.0);
        for (t, x) in [(0.0, 0.0), (2.0, 10.0), (4.0, 20.0)] {
            let s = sample(&keys, t, EASE_LINEAR, false).unwrap();
            assert!((s.eye[0] - x).abs() < 1e-3, "t {t}: {}", s.eye[0]);
        }
        // Halfway along a straight line is halfway.
        let s = sample(&keys, 1.0, EASE_LINEAR, false).unwrap();
        assert!((s.eye[0] - 5.0).abs() < 0.05, "{}", s.eye[0]);
    }

    #[test]
    fn smooth_ease_starts_and_ends_still_and_is_monotonic() {
        let mut last = 0.0;
        for i in 0..=1000 {
            let u = i as f32 / 1000.0;
            let e = ease_path(u);
            assert!(e + 1e-4 >= last, "not monotonic at {u}: {e} < {last}");
            last = e;
        }
        assert!(ease_path(0.0).abs() < 1e-5);
        assert!((ease_path(1.0) - 1.0).abs() < 1e-4);
        assert!((ease_path(0.5) - 0.5).abs() < 1e-3);
        // Slow at the very start.
        assert!(ease_path(0.01) < 0.002);
    }

    #[test]
    fn yaw_turns_the_short_way() {
        let a = 3.0;
        let b = -3.0; // 0.28 rad the other side of pi
        let keys = vec![k(0.0, a), k(10.0, b)];
        let s = sample(&keys, 1.0, EASE_LINEAR, false).unwrap();
        let mid = (s.yaw).rem_euclid(std::f32::consts::TAU);
        assert!((mid - std::f32::consts::PI).abs() < 0.1, "{mid}");
    }

    #[test]
    fn holds_keep_the_camera_still() {
        let mut keys = vec![k(0.0, 0.0), k(10.0, 0.0)];
        keys[0].hold = 1.0;
        assert_eq!(path_seconds(&keys, false), 3.0);
        let s = sample(&keys, 0.5, EASE_LINEAR, false).unwrap();
        assert_eq!(s.eye[0], 0.0);
        let s = sample(&keys, 3.0, EASE_LINEAR, false).unwrap();
        assert!((s.eye[0] - 10.0).abs() < 1e-3);
    }

    #[test]
    fn a_loop_comes_back_to_the_start() {
        let keys = vec![k(0.0, 0.0), k(10.0, 0.0), k(5.0, 0.0)];
        assert_eq!(path_seconds(&keys, true), 6.0);
        let s = sample(&keys, 6.0, EASE_LINEAR, true).unwrap();
        assert!(s.eye[0].abs() < 1e-3, "{}", s.eye[0]);
    }

    #[test]
    fn moon_days_give_their_phase() {
        for phase in 0..12u8 {
            for minute in [0u32, 300, 719, 720, 1000, 1439] {
                let day = moon_day(phase, minute);
                let c = super::super::super::event::HeldClock { day: Some(day), minute, moon: true };
                assert_eq!(c.moon_weekday().unwrap().0, phase as u32, "phase {phase} minute {minute}");
            }
        }
    }

    #[test]
    fn angles_round_trip() {
        for (y, p) in [(0.0f32, 0.0f32), (1.0, 0.3), (-2.5, -0.7), (3.0, 1.2)] {
            let (y2, p2) = angles_of(dir_of(y, p));
            assert!((y - y2).abs() < 1e-4 && (p - p2).abs() < 1e-4, "{y} {p} -> {y2} {p2}");
        }
        // A positive pitch looks up (Y is down).
        assert!(dir_of(0.0, 0.5).y < 0.0);
    }

    #[test]
    fn neutral_look_needs_no_pass() {
        assert!(Look::neutral().is_neutral());
        for (name, l) in look_presets().into_iter().skip(1) {
            assert!(!l.is_neutral(), "{name}");
        }
    }
}
