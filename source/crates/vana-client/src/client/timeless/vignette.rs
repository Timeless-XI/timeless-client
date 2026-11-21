//! Timeless XI: the Select Character screen's vignettes (Interface = Timeless). Instead of retail's ring of
//! characters in the lobby zone, the highlighted character is shown living a short moment where they left off: the
//! zone they logged out in, at the spot, hour and weather of their last session, under the title's nostalgia filter.
//!
//! Three vignettes (the owner's proof set, 2026-11-20; no /heal pose, since most players log out healing):
//! - **Fishing**: they logged out fishing. Rod out, line in the water, a fresh cast now and then; the camera drifts
//!   around behind their shoulder, looking out over the water.
//! - **In town**: a city zone. A slow stroll a few steps up and back, with long stops to look about; a closer camera.
//! - **Walking**: anywhere else. A steady walk along open ground, out and back, the camera tracking alongside.
//!
//! Where they left off comes from `timeless-vignettes.json` beside config.json, which the client keeps up to date
//! while in the world (`Recorder`): zone, place, facing, fishing (and the rod), the hour and the weather. A character
//! with no record there yet, or whose record is for another zone than the lobby says, gets an open, flat spot in the
//! lobby's zone picked from the zone's own collision (`find_spot`).

use super::super::ground::Ground;
use super::super::select::{FxLibs, LobbyFx, Target};
use super::scout::{self, Scene};
use crate::dat::character::{Animation, Character as Model, Look, PoseLayer, load_character};
use crate::dat::weather::{Lighting, ZoneWeather};
use crate::dat::{DatIndex, zone};
use crate::motionq::{Enq, Queues};
use crate::render::ui::{UiBatch, UiRenderer};
use crate::render::{ActorGpu, Camera, Gpu, ZoneRenderer};
use anyhow::{Context, Result};
use glam::{Mat4, Vec3};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, channel};

/// The vignette's effect voices (select.rs keeps the other lobby scenes' bases).
pub const VOICES: u32 = 0x0500_0000;

/// City zones: the "in town" vignette (era zones: the three nations, Jeuno, the towns, Aht Urhgan, the Safehold).
pub const TOWNS: [u16; 25] = [
    26, 48, 50, 53, 230, 231, 232, 233, 234, 235, 236, 237, 238, 239, 240, 241, 242, 243, 244, 245, 246, 247, 248, 249,
    250,
];

/// Norg (a town, but its zone id sits apart).
const NORG: u16 = 252;

pub fn is_town(zone: u16) -> bool {
    TOWNS.contains(&zone) || zone == NORG
}

/// Walk speeds (units a second) and the walk motion's rate: the walk plays at rate 1 at a third of the run (5 / s),
/// as the world's (motion.rs `choose`).
const WALK: (f32, f32) = (1.6, 0.96);
const STROLL: (f32, f32) = (1.2, 0.72);

/// Retail's rod files by race (client/fishrod.rs ROD_FILES): + the rod's kind (range look + 5) - 5.
const ROD_FILES: [u32; 9] = [39307, 39307, 39339, 39371, 39403, 39435, 39435, 39467, 39499];

// ---------------------------------------------------------------------------------------------------------------
// Where they left off.
// ---------------------------------------------------------------------------------------------------------------

/// A character's last moment in the world.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Record {
    pub zone: u16,
    pub pos: [f32; 3],
    /// The model's facing, x and z.
    pub forward: [f32; 2],
    #[serde(default)]
    pub fishing: bool,
    /// The range slot's look (the rod) while fishing.
    #[serde(default)]
    pub rod: u16,
    /// Vana'diel minutes into the day.
    #[serde(default = "default_minute")]
    pub minute: f32,
    #[serde(default = "default_weather")]
    pub weather: String,
}

fn default_minute() -> f32 {
    17.5 * 60.0
}

fn default_weather() -> String {
    "suny".into()
}

#[derive(Default, Serialize, Deserialize)]
struct Book {
    #[serde(default)]
    characters: BTreeMap<String, Record>,
}

pub fn book_path() -> std::path::PathBuf {
    crate::config::config_path().with_file_name("timeless-vignettes.json")
}

fn book_key(name: &str) -> String {
    name.trim().to_ascii_lowercase()
}

fn read_book() -> Book {
    std::fs::read_to_string(book_path()).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
}

pub fn record_of(name: &str) -> Option<Record> {
    read_book().characters.remove(&book_key(name))
}

fn write_record(name: &str, r: &Record) {
    let mut book = read_book();
    book.characters.insert(book_key(name), r.clone());
    match serde_json::to_string_pretty(&book) {
        Ok(s) => {
            if let Err(e) = std::fs::write(book_path(), s) {
                log::warn!("timeless-vignettes.json: {e}");
            }
        }
        Err(e) => log::warn!("timeless-vignettes.json: {e}"),
    }
}

/// Keeps our character's last moment while in the world, and writes it out every half minute and on the way out.
#[derive(Default)]
pub struct Recorder {
    last: Option<(String, Record)>,
    saved: Option<(String, Record)>,
    noted: Option<std::time::Instant>,
    written: Option<std::time::Instant>,
}

impl Recorder {
    /// Every frame: in the world, our moment (once a second); anywhere else, the last one written out.
    pub(in crate::client) fn tick(&mut self, phase: &super::super::Phase) {
        let super::super::Phase::World(w) = phase else {
            self.save();
            return;
        };
        if self.noted.is_some_and(|t| t.elapsed().as_secs_f32() < 1.0) {
            return;
        }
        self.noted = Some(std::time::Instant::now());
        if let Some(r) = moment(w) {
            self.last = Some((w.name.clone(), r));
        }
        if self.written.is_none_or(|t| t.elapsed().as_secs_f32() >= 30.0) {
            self.written = Some(std::time::Instant::now());
            self.save();
        }
    }

    fn save(&mut self) {
        if let Some((name, r)) = &self.last
            && self.last != self.saved
        {
            write_record(name, r);
            self.saved = self.last.clone();
        }
    }
}

/// Our moment now: none in a Mog House, between zones, or before our name is known.
fn moment(w: &super::super::World) -> Option<Record> {
    let zone = w.zone?;
    if w.mog_zone || w.name.trim().is_empty() || !w.pos.is_finite() {
        return None;
    }
    let forward =
        w.my_draw_yaw.map_or_else(|| Vec3::new(w.yaw.sin(), 0.0, w.yaw.cos()), |t| Vec3::new(t.cos(), 0.0, -t.sin()));
    let fishing = super::super::fishing::any_fishing_status(w.my_motion.status);
    let rod = match w.actors.get(&w.my_id).and_then(|a| a.source.as_ref()) {
        Some(super::super::Source::Pc(l)) => l.range,
        _ => 0,
    };
    let minute =
        w.clock.map_or(default_minute(), |(gt, at)| crate::dat::weather::vana_minute(gt, at.elapsed().as_secs_f32()));
    let weather = crate::dat::weather::weather_tag(super::super::event::weather(w)).to_string();
    Some(Record {
        zone,
        pos: w.pos.to_array(),
        forward: [forward.x, forward.z],
        fishing,
        rod,
        minute: minute.rem_euclid(1440.0),
        weather,
    })
}

// ---------------------------------------------------------------------------------------------------------------
// What a vignette needs, loaded off the UI thread.
// ---------------------------------------------------------------------------------------------------------------

/// The character to show.
#[derive(Clone, Debug)]
pub struct Request {
    pub name: String,
    pub look: Look,
    pub size: u8,
    pub zone: u16,
}

impl Request {
    /// What tells one request from another: the name, the look and the zone.
    pub fn key(&self) -> String {
        format!("{}:{:?}:{}:{}", self.name, self.look, self.size, self.zone)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    Walk,
    Town,
    Fish,
}

pub struct VignetteData {
    key: String,
    zone_id: u32,
    scene: zone::ZoneScene,
    ground: Ground,
    weather: Option<ZoneWeather>,
    fx: Option<FxLibs>,
    model: Model,
    size: u8,
    /// The model's motions and, fishing, the fishing files' (searched in that order).
    anims: Vec<Animation>,
    /// The rod and its motions, fishing.
    rod: Option<(Model, Vec<Animation>)>,
    kind: Kind,
    /// Walking: the way out, a point every half unit, on the ground. Fishing: the one spot.
    path: Vec<Vec3>,
    forward: Vec3,
    /// The camera's side of the heading (+1 / -1, scout.rs `turn`'s sense).
    side: f32,
    /// The body's height (posed in its idle, at its size), for the framing.
    height: f32,
    /// Scenery the camera must not look through.
    occ: scout::Scenery,
    /// What the scene came from, for the log and the shot tool.
    source: &'static str,
    minute: f32,
    weather_tag: String,
}

impl VignetteData {
    pub fn load(index: &Arc<DatIndex>, keys: &Arc<zone::Keys>, req: &Request) -> Result<Self> {
        Self::load_with(index, keys, req, record_of(&req.name), None)
    }

    /// `load` with the record given (vignette-shot).
    pub fn load_with(
        index: &Arc<DatIndex>,
        keys: &Arc<zone::Keys>,
        req: &Request,
        record: Option<Record>,
        pick: Option<usize>,
    ) -> Result<Self> {
        let zone_id = req.zone as u32;
        let file = index.read(zone::file_id(zone_id))?;
        let scene = zone::load_scene(keys, &file)?;
        let ground = Ground::build(&scene);
        let index_scene = scout::SceneIndex::build(&scene);
        let weather = ZoneWeather::parse(&file).ok();
        let fx = FxLibs::load_zone(index, keys, zone_id, &file);
        let model = load_character(index, &req.look).context("the character's model")?;
        let record = record.filter(|r| r.zone == req.zone);
        let town = is_town(req.zone);
        let seed = seed_of(&req.name);
        let mut anims = model.animations.clone();
        let mut rod = None;
        let mut kind = if town { Kind::Town } else { Kind::Walk };
        if let (Some(r), Some(pc)) = (record.as_ref().filter(|r| r.fishing), model.pc) {
            // The fishing motion files (client/fishrod.rs `files`), and the rod as its own model.
            let v = if r.rod == 0xc { 2 } else { 1 };
            let mut fish = Vec::new();
            for id in super::super::ride::motion_files(&pc, v) {
                match crate::dat::motionfile::MotionFile::load(index, id) {
                    Ok(f) => fish.extend(f.animations),
                    Err(e) => log::warn!("vignette fishing motions {id}: {e:#}"),
                }
            }
            let kind_of = if r.rod >= 0x55 { 5 } else { r.rod as u32 + 5 };
            let rod_id = ROD_FILES[(req.look.race as usize).min(8)] + kind_of - 5;
            let rod_model = crate::dat::npc::load_file_model(index, rod_id)
                .map_err(|e| log::warn!("vignette rod {rod_id}: {e:#}"))
                .ok();
            if !fish.is_empty() {
                anims.extend(fish);
                rod = rod_model.map(|m| {
                    let a = m.animations.clone();
                    (m, a)
                });
                kind = Kind::Fish;
            }
        }
        // Where: the zone's most scenic place (the scouted table, else scouted now), one of the best three by the
        // character's name; failing that, where they left off, the Home Point, or any open ground.
        let len = if kind == Kind::Town { 8.0 } else { 12.0 };
        let scenes = scout::table(req.zone);
        // Scouted now: around where they left off, else the Home Point, else anywhere.
        let near: Vec<Vec3> =
            record.as_ref().map(|r| Vec3::from(r.pos)).or_else(|| home_point(req.zone)).into_iter().collect();
        let from_table = |v: Vec<Scene>| (!v.is_empty()).then_some(v);
        let (picked, mut source) = match kind {
            Kind::Fish => match scenes.and_then(|t| from_table(t.fish)) {
                Some(v) => (v, "the zone's scouted fishing scenes"),
                None => (
                    scout::scout_fish(&ground, (&scene, &index_scene), 30, 3, seed, &near),
                    "fishing scenes scouted now",
                ),
            },
            _ => match scenes.and_then(|t| from_table(t.walk)) {
                Some(v) => (v, "the zone's scouted scenes"),
                None => {
                    (scout::scout_walk(&ground, (&scene, &index_scene), len, 45, 3, seed, &near), "scenes scouted now")
                }
            },
        };
        let mut spot_scene = picked.get(pick.unwrap_or((seed % picked.len().max(1) as u64) as usize)).copied();
        if spot_scene.is_none() {
            let spot = record.as_ref().and_then(|r| {
                let p = Vec3::from(r.pos);
                let y = ground.settle(p.x, p.z, p.y)?;
                let f = Vec3::new(r.forward[0], 0.0, r.forward[1]).normalize_or(Vec3::X);
                Some((Vec3::new(p.x, y, p.z), f))
            });
            source = if spot.is_some() { "where they left off" } else { "a picked spot" };
            let (at, f) = match spot.or_else(|| home_spot(&ground, req.zone, seed)) {
                Some(s) => s,
                None => find_spot(&ground, seed).context("no open ground in the zone")?,
            };
            // Fishing: facing the water (the nearest heading that looks at it).
            let f = if kind == Kind::Fish && !ground.water_ahead(at, f) {
                (1..12)
                    .flat_map(|k| [k as f32 * 15.0, -(k as f32) * 15.0])
                    .map(|deg| turn(f, deg))
                    .find(|g| ground.water_ahead(at, *g))
                    .unwrap_or(f)
            } else {
                f
            };
            spot_scene = Some(Scene { at: at.to_array(), dir: [f.x, f.z], len, side: 1.0, score: 0.0 });
        }
        let spot_scene = spot_scene.unwrap();
        let start = ground
            .settle(spot_scene.at[0], spot_scene.at[2], spot_scene.at[1])
            .map_or(spot_scene.at(), |y| Vec3::new(spot_scene.at[0], y, spot_scene.at[2]));
        let forward = spot_scene.dir();
        let side = if spot_scene.side < 0.0 { -1.0 } else { 1.0 };
        let path = match kind {
            Kind::Fish => vec![start],
            _ => scout::walk_points(&ground, start, forward, spot_scene.len.max(1.0)),
        };
        // The scenery around the scene, for the camera.
        let mid = path[path.len() / 2];
        let occ = index_scene.local(&scene, mid, scout::xz(*path.last().unwrap() - path[0]).length() * 0.5 + 10.0);
        // The body's height in its idle pose, at its size (y points down: min is the top).
        let idle: Vec<_> = model.find_animations("idl").into_iter().map(|a| (a, 0.0)).collect();
        let rest = crate::render::character::pose_character(&model, &idle);
        let height = ((rest.max.y - rest.min.y) * model.scale(req.size)).clamp(0.6, 4.0);
        let (minute, weather_tag) =
            record.as_ref().map_or((default_minute(), default_weather()), |r| (r.minute, r.weather.clone()));
        log::info!(
            "vignette {}: zone {zone_id}, {kind:?} from {source} (score {:.2}, {} path points), {:.1} h {weather_tag}, \
             {height:.2} tall",
            req.name,
            spot_scene.score,
            path.len(),
            minute / 60.0
        );
        Ok(Self {
            key: req.key(),
            zone_id,
            scene,
            ground,
            weather,
            fx,
            model,
            size: req.size,
            anims,
            rod,
            kind,
            path,
            forward,
            side,
            height,
            occ,
            source,
            minute,
            weather_tag,
        })
    }
}

fn seed_of(name: &str) -> u64 {
    name.bytes().fold(0xcbf2_9ce4_8422_2325u64, |h, b| (h ^ b as u64).wrapping_mul(0x0100_0000_01b3)) | 1
}

fn xz(v: Vec3) -> Vec3 {
    Vec3::new(v.x, 0.0, v.z)
}

/// `f` turned by `deg` about the vertical.
fn turn(f: Vec3, deg: f32) -> Vec3 {
    let (s, c) = deg.to_radians().sin_cos();
    Vec3::new(f.x * c - f.z * s, 0.0, f.x * s + f.z * c).normalize_or(Vec3::X)
}

/// The way out from `start`: the first heading near the facing that walks most of `len` (the ground's own walk, so
/// walls, rails and cliffs stop it), as points every half unit.
fn walk_path(ground: &Ground, start: Vec3, forward: Vec3, len: f32) -> Vec<Vec3> {
    let mut best: Option<(f32, Vec3)> = None;
    for deg in [0.0, 25.0, -25.0, 50.0, -50.0, 80.0, -80.0, 120.0, -120.0, 180.0] {
        let d = turn(forward, deg);
        let end = ground.walk(start, start + d * len, 0.0);
        let reach = xz(end - start).length();
        if best.is_none_or(|b| reach > b.0 + 0.5) {
            best = Some((reach, d));
        }
        if reach >= len * 0.8 {
            break;
        }
    }
    let (reach, d) = best.unwrap_or((0.0, forward));
    let mut path = vec![start];
    let mut p = start;
    let mut gone = 0.0;
    while gone < reach - 0.25 {
        let next = ground.walk(p, p + d * 0.5, 0.0);
        let step = xz(next - p).length();
        // Stuck, or a drop or climb no walk takes in half a unit.
        if step < 0.05 || (next.y - p.y).abs() > 0.8 {
            break;
        }
        gone += step;
        p = next;
        path.push(p);
    }
    path
}

fn path_dir(path: &[Vec3]) -> Option<Vec3> {
    let (a, b) = (path.first()?, path.last()?);
    let d = xz(*b - *a);
    (d.length() > 0.5).then(|| d.normalize())
}

/// The zone's first Home Point (assets/timeless/vignette_spots.json, from the server's data by
/// tools/timeless/vignette_spots.py): street level in every town.
fn home_point(zone: u16) -> Option<Vec3> {
    static SPOTS: std::sync::LazyLock<std::collections::HashMap<String, [f32; 3]>> = std::sync::LazyLock::new(|| {
        serde_json::from_str(include_str!("../../../assets/timeless/vignette_spots.json")).unwrap_or_default()
    });
    SPOTS.get(&zone.to_string()).map(|p| Vec3::from(*p))
}

/// A spot for a character with no record, by their zone's Home Point: a few steps from the crystal along the
/// heading (of eight, turned by `seed`) with the most room to walk.
fn home_spot(ground: &Ground, zone: u16, seed: u64) -> Option<(Vec3, Vec3)> {
    let hp = home_point(zone)?;
    let y = ground.settle(hp.x, hp.z, hp.y)?;
    let hp = Vec3::new(hp.x, y, hp.z);
    let base = (seed % 8) as f32 * 45.0;
    let mut best: Option<(f32, Vec3)> = None;
    for k in 0..8 {
        let d = turn(Vec3::X, base + k as f32 * 45.0);
        let path = walk_path(ground, hp, d, 12.0);
        let reach = path.last().map_or(0.0, |e| xz(*e - hp).length());
        if best.is_none_or(|b| reach > b.0 + 0.5) {
            best = Some((reach, d));
        }
    }
    let (reach, d) = best?;
    if reach < 4.0 {
        return None;
    }
    // Clear of the crystal.
    let at = ground.walk(hp, hp + d * 3.0, 0.0);
    Some((at, d))
}

/// A spot for a character with no record: a flat, open floor with room to walk, picked from the zone's collision by
/// `seed` (big faces weigh more, so open ground wins over ledges and roofs).
pub fn find_spot(ground: &Ground, seed: u64) -> Option<(Vec3, Vec3)> {
    let mut faces: Vec<(Vec3, f32)> = ground
        .all_tris()
        .filter_map(|[a, b, c]| {
            let n = (*b - *a).cross(*c - *a);
            let area = n.length() * 0.5;
            (area > 1.5 && n.normalize_or_zero().y.abs() > 0.95).then(|| ((*a + *b + *c) / 3.0, area))
        })
        .collect();
    if faces.is_empty() {
        return None;
    }
    let total: f32 = faces.iter().map(|f| f.1).sum();
    let mut s = seed;
    let mut rand = || {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        (s % 1_000_000) as f32 / 1_000_000.0
    };
    // Up to 60 area-weighted tries; the first with 9 units of walk and open sky over it.
    let mut best: Option<(f32, Vec3, Vec3)> = None;
    faces.sort_by(|a, b| a.0.x.total_cmp(&b.0.x).then(a.0.z.total_cmp(&b.0.z)));
    for _ in 0..60 {
        let mut pick = rand() * total;
        let Some(&(c, _)) = faces.iter().find(|f| {
            pick -= f.1;
            pick <= 0.0
        }) else {
            continue;
        };
        let Some(y) = ground.settle(c.x, c.z, c.y) else { continue };
        let p = Vec3::new(c.x, y, c.z);
        // A roof or ceiling over it: not outdoors.
        if ground.raycast(p - Vec3::Y * 0.5, p - Vec3::Y * 12.0).is_some() {
            continue;
        }
        let f = turn(Vec3::X, rand() * 360.0);
        let path = walk_path(ground, p, f, 10.0);
        let reach = path.last().map_or(0.0, |e| xz(*e - p).length());
        let dir = path_dir(&path).unwrap_or(f);
        if best.is_none_or(|b| reach > b.0) {
            best = Some((reach, p, dir));
        }
        if reach >= 9.0 {
            break;
        }
    }
    best.filter(|b| b.0 >= 3.0).map(|b| (b.1, b.2))
}

// ---------------------------------------------------------------------------------------------------------------
// The vignette playing.
// ---------------------------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
enum Act {
    /// Walking the path toward `dir` (+1 out, -1 back).
    Walk,
    /// Standing (the idle) for the seconds left; `mid`: a stop part way along, the walk goes on the same way after.
    Stop {
        left: f32,
        mid: bool,
    },
    /// Turning to walk back: from, to (radians about the vertical), seconds left of the whole.
    Turn {
        from: f32,
        to: f32,
        left: f32,
        whole: f32,
    },
    /// Fishing: the line in the water (fh1) for the seconds left, then a fresh cast.
    Hold {
        left: f32,
    },
    Cast {
        left: f32,
    },
}

fn yaw_of(f: Vec3) -> f32 {
    f.z.atan2(f.x)
}

fn dir_of(yaw: f32) -> Vec3 {
    Vec3::new(yaw.cos(), 0.0, yaw.sin())
}

/// Facing `forward` (model +x toward it) at `feet`, at the model's size (select.rs placement).
fn placement(feet: Vec3, forward: Vec3, scale: f32) -> Mat4 {
    let f = xz(forward).normalize_or(Vec3::X);
    Mat4::from_cols(f.extend(0.0), Vec3::Y.extend(0.0), f.cross(Vec3::Y).extend(0.0), feet.extend(1.0))
        * Mat4::from_scale(Vec3::splat(scale))
}

/// A motion's chunks by region (dat::character find_animations' rule over `anims`): (region, index, frames, speed).
/// A PC body with no skirt set takes no region 2 chunk (client/actmotion.rs `chunks`).
fn chunks(anims: &[Animation], name: &str, skirtless: bool) -> Vec<(u8, usize, f32, f32)> {
    let prefix = name.get(..3).unwrap_or(name).as_bytes();
    let mut seen = [false; 10];
    let mut out = Vec::new();
    for (i, a) in anims.iter().enumerate() {
        let n = a.name.trim_end().as_bytes();
        if n.len() != 4 || !n.starts_with(prefix) || !(b'0'..=b'6').contains(&n[3]) {
            continue;
        }
        let r = n[3] - b'0';
        if seen[r as usize] || (r == 2 && skirtless) {
            continue;
        }
        seen[r as usize] = true;
        out.push((r, i, a.frames as f32, a.speed));
    }
    out
}

/// Plays `name` as the status motion (looping), blended in over `blend` ticks. False when the model has none.
fn play(q: &mut Queues<usize>, anims: &[Animation], name: &str, skirtless: bool, rate: f32, blend: f32) -> bool {
    let c = chunks(anims, name, skirtless);
    if c.is_empty() {
        return false;
    }
    q.enqueue_motion(c, 0, &Enq::status(rate, blend, true), |_| rate);
    true
}

/// A motion's length in seconds at `rate` (its region 0 chunk, else its first).
fn seconds_of(anims: &[Animation], name: &str, rate: f32) -> Option<f32> {
    let c = chunks(anims, name, false);
    let (_, _, frames, speed) = c.iter().find(|c| c.0 == 0).or(c.first()).copied()?;
    Some(frames / (speed * 0.5 * rate).max(1e-3) / 60.0)
}

/// The vignettes' grade: the title's filter, a little stronger (owner, 2026-11-20): softer colour, lifted blacks,
/// more halation, a deeper vignette, more grain and softer edges.
fn stronger(g: Option<crate::render::nostalgia::Grade>) -> Option<crate::render::nostalgia::Grade> {
    g.map(|g| crate::render::nostalgia::Grade {
        lift: g.lift.map(|l| l * 1.3),
        shadow_tint: g.shadow_tint.map(|t| t * 1.3),
        high_tint: g.high_tint.map(|t| t * 1.3),
        saturation: g.saturation * 0.86,
        contrast: g.contrast * 1.03,
        halation: (g.halation + 0.1).min(1.0),
        vignette: (g.vignette + 0.15).min(1.0),
        grain: g.grain * 1.35,
        soft_edges: (g.soft_edges + 0.15).min(1.0),
        ..g
    })
}

fn smooth(from: Vec3, to: Vec3, dt: f32, secs: f32) -> Vec3 {
    from + (to - from) * (1.0 - (-dt / secs.max(1e-3)).exp())
}

pub struct Vignette {
    data: VignetteData,
    zone: Option<ZoneRenderer>,
    actor: Option<ActorGpu>,
    rod_actor: Option<ActorGpu>,
    q: Queues<usize>,
    rod_q: Queues<usize>,
    skirtless: bool,
    effects: Option<LobbyFx>,
    ui: Option<UiRenderer>,
    nostalgia: Option<crate::render::nostalgia::Nostalgia>,
    grade: Option<crate::render::nostalgia::Grade>,
    /// Seconds since it started.
    time: f32,
    /// Along the path (units), the way it walks (+1 / -1), and the facing.
    s: f32,
    way: f32,
    feet: Vec3,
    facing: Vec3,
    act: Act,
    /// The stop part way along has happened on this leg.
    stopped_mid: bool,
    /// The camera: its angle off the scene's heading (degrees), and the eye, target and focal it eased to.
    theta: f32,
    cam: Option<(Vec3, Vec3, f32)>,
    /// 0..1: fades in when it starts and out when another character is wanted.
    pub alpha: f32,
    pub leaving: bool,
}

impl Vignette {
    pub fn new(mut data: VignetteData) -> Self {
        let effects = data.fx.take().map(|l| {
            let mut fx = LobbyFx::new(l, data.minute, VOICES);
            fx.set_weather(&data.weather_tag);
            fx
        });
        let skirtless = data.model.pc.as_ref().is_some_and(|pc| pc.skirt == 0);
        let feet = data.path[0];
        let facing = match data.kind {
            Kind::Fish => data.forward,
            _ => path_dir(&data.path).unwrap_or(data.forward),
        };
        let grade = stronger(super::title::filter_preset(&super::title::Reel::load().filter));
        let mut v = Self {
            data,
            zone: None,
            actor: None,
            rod_actor: None,
            q: Queues::default(),
            rod_q: Queues::default(),
            skirtless,
            effects,
            ui: None,
            nostalgia: None,
            grade,
            time: 0.0,
            s: 0.0,
            way: 1.0,
            feet,
            facing,
            act: Act::Stop { left: 0.8, mid: false },
            stopped_mid: false,
            theta: scout::FRONT_DEG,
            cam: None,
            alpha: 0.0,
            leaving: false,
        };
        match v.data.kind {
            Kind::Fish => {
                v.act = Act::Hold { left: 9.0 };
                v.motion("fh1", 1.0, 1.0);
            }
            _ => {
                if v.data.path.len() < 3 {
                    // Nowhere to walk: stand and look about.
                    v.act = Act::Stop { left: f32::INFINITY, mid: false };
                }
                v.motion("idl", 1.0, 1.0);
            }
        }
        v
    }

    pub fn key(&self) -> &str {
        &self.data.key
    }

    /// The model's motion (and the rod's, fishing).
    fn motion(&mut self, name: &str, rate: f32, blend: f32) {
        let ok = play(&mut self.q, &self.data.anims, name, self.skirtless, rate, blend);
        if !ok && name != "idl" {
            play(&mut self.q, &self.data.anims, "idl", self.skirtless, 1.0, blend);
        }
        if let Some((_, rod_anims)) = &self.data.rod {
            play(&mut self.rod_q, rod_anims, name, false, rate, blend);
        }
    }

    fn path_len(&self) -> f32 {
        self.data.path.windows(2).map(|w| xz(w[1] - w[0]).length()).sum()
    }

    /// The point `s` units along the path, and the path's way there.
    fn along(&self, s: f32) -> (Vec3, Vec3) {
        let p = &self.data.path;
        let mut left = s.max(0.0);
        for w in p.windows(2) {
            let d = xz(w[1] - w[0]).length();
            if left <= d && d > 1e-4 {
                return (w[0].lerp(w[1], left / d), xz(w[1] - w[0]).normalize());
            }
            left -= d;
        }
        let last = *p.last().unwrap();
        (last, path_dir(p).unwrap_or(self.facing))
    }

    pub fn advance(&mut self, dt: f32) {
        let dt = dt.min(0.1);
        self.time += dt;
        if let Some(fx) = &mut self.effects {
            fx.advance(dt);
        }
        let to = if self.leaving { 0.0 } else { 1.0 };
        let rate = if self.leaving { 2.5 } else { 0.8 };
        self.alpha = if to > self.alpha { (self.alpha + dt * rate).min(to) } else { (self.alpha - dt * rate).max(to) };
        match self.data.kind {
            Kind::Fish => self.fish(dt),
            Kind::Town => self.walk(dt, STROLL, (5.5, 7.5), true),
            Kind::Walk => self.walk(dt, WALK, (2.5, 4.0), false),
        }
        self.q.tick(dt * 60.0);
        self.rod_q.tick(dt * 60.0);
    }

    fn fish(&mut self, dt: f32) {
        match &mut self.act {
            Act::Hold { left } => {
                *left -= dt;
                if *left <= 0.0 {
                    // A fresh cast (fsh0: fh0, then the hold fh1).
                    let secs = seconds_of(&self.data.anims, "fh0", 1.0).unwrap_or(2.3).clamp(0.5, 6.0);
                    self.act = Act::Cast { left: secs };
                    self.motion("fh0", 1.0, 8.0);
                }
            }
            Act::Cast { left } => {
                *left -= dt;
                if *left <= 0.0 {
                    // 24..34 s with the line out before the next cast.
                    let wait = 24.0 + (self.time * 7.31).sin().abs() * 10.0;
                    self.act = Act::Hold { left: wait };
                    self.motion("fh1", 1.0, 12.0);
                }
            }
            _ => self.act = Act::Hold { left: 20.0 },
        }
    }

    /// Out along the path and back, stopping at each end (`stops` seconds, low..high) and, in town, once part way.
    fn walk(&mut self, dt: f32, (speed, rate): (f32, f32), stops: (f32, f32), town: bool) {
        let len = self.path_len();
        let stop_for = |t: f32| stops.0 + (t * 3.17).sin().abs() * (stops.1 - stops.0);
        match self.act {
            Act::Walk => {
                self.s += speed * dt * self.way;
                let (p, d) = self.along(self.s.clamp(0.0, len));
                self.feet = p;
                self.facing = d * self.way;
                let mid = (self.s - len * 0.5).abs() < speed * dt * 1.5;
                if town && mid && !self.stopped_mid {
                    self.stopped_mid = true;
                    self.act = Act::Stop { left: stop_for(self.time) * 0.6, mid: true };
                    self.motion("idl", 1.0, 14.0);
                } else if self.s >= len || self.s <= 0.0 {
                    self.s = self.s.clamp(0.0, len);
                    self.act = Act::Stop { left: stop_for(self.time), mid: false };
                    self.motion("idl", 1.0, 14.0);
                }
            }
            Act::Stop { left, mid } => {
                let left = left - dt;
                if left > 0.0 {
                    self.act = Act::Stop { left, mid };
                } else if mid {
                    self.act = Act::Walk;
                    self.motion("wlk", rate, 12.0);
                } else {
                    // Turn about at an end (and to the path's way at the very start).
                    let at_start = self.s <= 0.0;
                    let want = if at_start { 1.0 } else { -1.0 };
                    let (_, d) = self.along(self.s);
                    let to = yaw_of(d * want);
                    let from = yaw_of(self.facing);
                    let mut delta = (to - from).rem_euclid(std::f32::consts::TAU);
                    if delta > std::f32::consts::PI {
                        delta -= std::f32::consts::TAU;
                    }
                    let whole = (delta.abs() / std::f32::consts::PI * 0.9).max(0.05);
                    self.act = Act::Turn { from, to: from + delta, left: whole, whole };
                    self.way = want;
                    self.stopped_mid = false;
                }
            }
            Act::Turn { from, to, left, whole } => {
                let left = left - dt;
                let t = 1.0 - (left / whole).clamp(0.0, 1.0);
                let e = t * t * (3.0 - 2.0 * t);
                self.facing = dir_of(from + (to - from) * e);
                if left > 0.0 {
                    self.act = Act::Turn { from, to, left, whole };
                } else {
                    self.act = Act::Walk;
                    self.motion("wlk", rate, 12.0);
                }
            }
            _ => self.act = Act::Stop { left: 1.0, mid: false },
        }
    }

    /// The camera's angle off the scene's heading (degrees, toward `side`) for this moment. It always opens in front
    /// of the character (`FRONT_DEG`) with the view behind them.
    ///
    /// Walking (`dir` = the way toward the camera): in front while they walk in; round to the side while they stop
    /// at the near end; behind a shoulder while they walk back into the view; in front again as they turn at the
    /// far end. Fishing (`dir` = the water): in front over the water, round to the side, over the shoulder looking
    /// out, and back (a 56 s cycle).
    fn theta_wants(&self) -> f32 {
        let front = scout::FRONT_DEG;
        match self.data.kind {
            Kind::Fish => {
                let t = self.time.rem_euclid(56.0);
                let ease = |a: f32, b: f32, t0: f32, t1: f32| {
                    let k = ((t - t0) / (t1 - t0)).clamp(0.0, 1.0);
                    a + (b - a) * k * k * (3.0 - 2.0 * k)
                };
                match t {
                    t if t < 9.0 => front,
                    t if t < 21.0 => ease(front, 95.0, 9.0, 21.0),
                    t if t < 31.0 => ease(95.0, 158.0, 21.0, 31.0),
                    t if t < 40.0 => 158.0,
                    _ => ease(158.0, front, 40.0, 56.0),
                }
            }
            Kind::Walk | Kind::Town => match self.act {
                Act::Walk if self.way > 0.0 => front,
                Act::Stop { mid: true, .. } if self.way > 0.0 => front + 12.0,
                Act::Stop { mid: true, .. } => 34.0,
                Act::Stop { .. } if self.s > 0.5 && self.way > 0.0 => 82.0,
                // Turning back at the near end and walking away: behind a shoulder, the view ahead of them.
                Act::Turn { .. } | Act::Walk if self.way < 0.0 || self.s > 0.5 => 34.0,
                _ => front,
            },
        }
    }

    /// The camera for this frame: the framing for the body's height (it fills `FILL` of the frame, head and feet
    /// in), eased round to `theta_wants`, kept off the ground and out of walls; pulled in by a wall, it widens to keep
    /// the whole body in frame.
    fn camera(&mut self, dt: f32) -> Camera {
        let h = self.data.height;
        let want = self.theta_wants();
        // The orbit eases (about 2 s) so a change of angle reads as a deliberate move.
        self.theta += (want - self.theta) * (1.0 - (-dt / 2.0).exp());
        if self.cam.is_none() {
            self.theta = want;
        }
        let d = scout::frame_distance(h, scout::FOCAL, scout::FILL);
        let ground = &self.data.ground;
        let chest = self.feet - Vec3::Y * (h * 0.62);
        // The wanted angle; a tree or post in the way: closer (the focal widens to keep the framing), then a little
        // further round either way. The first that sees the whole body.
        let place = |deg: f32, k: f32| {
            let dir = scout::turn(self.data.forward, deg * self.data.side);
            let (eye, target) = scout::eye_target(self.feet, h, dir, d * k);
            let mut eye = scout::above_ground(ground, eye);
            if let Some(hit) = ground.cam_ray(chest, eye) {
                eye = chest + (hit - chest) * 0.88;
            }
            (eye, target)
        };
        let tries = [(0.0, 1.0), (0.0, 0.8), (0.0, 0.62), (14.0, 1.0), (-14.0, 1.0), (28.0, 0.8), (-28.0, 0.8)];
        let (eye, target) = tries
            .iter()
            .map(|(off, k)| place(self.theta + off, *k))
            .find(|(e, _)| scout::body_visible(ground, &self.data.occ, *e, self.feet, h))
            .unwrap_or_else(|| place(self.theta, 0.62));
        let focal = scout::frame_focal(h, (eye - target).length(), scout::FILL).clamp(240.0, scout::FOCAL);
        let (e, t, f) = match self.cam {
            None => (eye, target, focal),
            // Tight on the body (it moves), the focal a little slower.
            Some((e, t, f)) => {
                (smooth(e, eye, dt, 0.25), smooth(t, target, dt, 0.2), f + (focal - f) * (1.0 - (-dt / 0.4).exp()))
            }
        };
        self.cam = Some((e, t, f));
        Camera { eye: e, target: t, up: Vec3::NEG_Y, fov_y: crate::dat::title::fov_y(f) }
    }

    pub fn take_sounds(&mut self) -> Vec<crate::fx::sound::SoundCmd> {
        self.effects.as_mut().map(|fx| fx.take_sounds()).unwrap_or_default()
    }

    pub fn pause_sounds(&mut self) -> Vec<crate::fx::sound::SoundCmd> {
        self.effects.as_mut().map(|fx| fx.pause_sounds()).unwrap_or_default()
    }

    pub fn stop_sounds(&mut self) -> Vec<crate::fx::sound::SoundCmd> {
        self.effects.as_mut().map(|fx| fx.stop_sounds()).unwrap_or_default()
    }

    pub fn after_submit(&mut self, gpu: &Gpu) {
        if let Some(fx) = &mut self.effects {
            fx.after_submit(gpu);
        }
    }

    /// The zone, the character (and rod), the zone's effects, the fade, then the nostalgia filter.
    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        gpu: &Gpu,
        format: wgpu::TextureFormat,
        encoder: &mut wgpu::CommandEncoder,
        target: Target,
        depth: &wgpu::TextureView,
        aspect: f32,
        alive: Option<Vec<u32>>,
        draw_distance: u32,
        dt: f32,
    ) {
        let (view, tex, size) = target;
        let cam = self.camera(dt);
        let zone_id = self.data.zone_id as u16;
        let env = self.data.weather.as_ref().and_then(|w| w.at(&self.data.weather_tag, self.data.minute));
        let lighting = env.map(|e| {
            let k = crate::dat::weather::clipping(e.draw_dist, 1.0, zone_id).1;
            Lighting::from_env_k(&e, self.data.minute, k)
        });
        let reach = crate::dat::weather::clipping(env.map_or(100.0, |e| e.draw_dist), 1.0, zone_id).0;
        let far = (draw_distance != 100).then_some(reach).filter(|d| *d > 0.0);
        let zone = self.zone.get_or_insert_with(|| ZoneRenderer::new(gpu, &self.data.scene, format));
        zone.lighting = lighting;
        let scale = self.data.model.scale(self.data.size);
        let place = placement(self.feet, self.facing, scale);
        let actor = self.actor.get_or_insert_with(|| zone.new_actor(gpu, &self.data.model));
        let mut layers = Vec::new();
        let over = self.q.layers(&mut layers);
        let pl: Vec<PoseLayer> = layers
            .iter()
            .map(|l| PoseLayer {
                anim: &self.data.anims[l.anim],
                frame: l.frame,
                weight: l.weight,
                start: l.start,
                over: l.over,
            })
            .collect();
        let mesh = crate::render::character::pose_character_layers(&self.data.model, &pl, over);
        zone.update_actor(gpu, actor, &mesh, place);
        if let Some((rod, rod_anims)) = &self.data.rod {
            let ra = self.rod_actor.get_or_insert_with(|| zone.new_actor(gpu, rod));
            let mut rl = Vec::new();
            let rover = self.rod_q.layers(&mut rl);
            let rpl: Vec<PoseLayer> = rl
                .iter()
                .map(|l| PoseLayer {
                    anim: &rod_anims[l.anim],
                    frame: l.frame,
                    weight: l.weight,
                    start: l.start,
                    over: l.over,
                })
                .collect();
            let rmesh = crate::render::character::pose_character_layers(rod, &rpl, rover);
            zone.update_actor(gpu, ra, &rmesh, place);
        }
        // All the zone's scenery (list 0), as the title reel's `all_scenery`: the camera lists are made for players
        // on the paths, and a camera off them (over a drop, on a terrace) leaves the far side of the zone undrawn.
        zone.set_camera_list(gpu, 0, far);
        let actors: Vec<&ActorGpu> = self.actor.iter().chain(self.rod_actor.iter()).collect();
        zone.render(gpu, encoder, view, depth, &cam, aspect, &actors);
        if let Some(fx) = &mut self.effects {
            fx.draw(gpu, format, encoder, (view, tex, size), depth, zone, &cam, aspect, alive);
        }
        let dim = 1.0 - self.alpha.clamp(0.0, 1.0);
        if dim > 0.0 {
            let ui = self.ui.get_or_insert_with(|| UiRenderer::new(gpu, format));
            let mut batch = UiBatch::default();
            batch.push_rect(0.0, 0.0, size[0] as f32, size[1] as f32, [0.0, 0.0, 0.0, dim * dim * (3.0 - 2.0 * dim)]);
            ui.render(gpu, encoder, view, size, size, None, &batch);
        }
        if let (Some(g), Some(t)) = (self.grade.as_ref(), tex) {
            let n = self.nostalgia.get_or_insert_with(|| crate::render::nostalgia::Nostalgia::new(gpu, format));
            n.draw(gpu, encoder, view, t, size, g, [0.0; 3]);
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------
// The stage: the vignette shown, the next one loading, the recorder.
// ---------------------------------------------------------------------------------------------------------------

#[derive(Default)]
pub struct Stage {
    pub recorder: Recorder,
    current: Option<Vignette>,
    /// Loaded while the current one fades out.
    next: Option<Vignette>,
    loading: Option<(String, Receiver<Result<VignetteData>>)>,
    /// The key last asked for, and one that failed (not asked again).
    wanted: String,
    failed: String,
    sounds: Vec<crate::fx::sound::SoundCmd>,
}

impl Stage {
    /// The Select screen can fade in: a vignette is up, or the wanted one failed.
    pub fn ready(&self) -> bool {
        self.current.is_some() || (!self.failed.is_empty() && self.failed == self.wanted)
    }

    /// The key of the character last asked for.
    pub fn wanted(&self) -> &str {
        &self.wanted
    }

    /// `req` is the highlighted character. Starts its load when it is a new one (the shown one starts to fade).
    pub fn want(&mut self, req: Request, data: impl FnOnce() -> Option<(Arc<DatIndex>, Arc<zone::Keys>)>) {
        let key = req.key();
        if key == self.wanted {
            return;
        }
        self.wanted = key.clone();
        if let Some(c) = &mut self.current {
            c.leaving = c.key() != key;
        }
        if self.current.as_ref().is_some_and(|c| c.key() == key) {
            self.next = None;
            return;
        }
        if self.next.as_ref().is_some_and(|n| n.key() == key) || self.failed == key {
            return;
        }
        let Some((index, keys)) = data() else { return };
        let (tx, rx) = channel();
        self.loading = Some((key, rx));
        std::thread::spawn(move || {
            let _ = tx.send(VignetteData::load(&index, &keys, &req));
        });
    }

    pub fn advance(&mut self, dt: f32) {
        if let Some((key, rx)) = &self.loading
            && let Ok(res) = rx.try_recv()
        {
            let key = key.clone();
            self.loading = None;
            match res {
                Ok(d) if d.key == self.wanted => {
                    let v = Vignette::new(d);
                    if self.current.is_none() {
                        self.current = Some(v);
                    } else {
                        self.next = Some(v);
                    }
                }
                Ok(_) => {}
                Err(e) => {
                    log::warn!("vignette: {e:#}");
                    self.failed = key;
                }
            }
        }
        // The old one gone dark: the next takes its place.
        if self.current.as_ref().is_some_and(|c| c.leaving && c.alpha <= 0.0) {
            if let Some(mut old) = self.current.take() {
                self.sounds.extend(old.stop_sounds());
            }
            self.current = self.next.take();
        }
        if let Some(c) = &mut self.current {
            c.advance(dt);
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        gpu: &Gpu,
        format: wgpu::TextureFormat,
        encoder: &mut wgpu::CommandEncoder,
        target: Target,
        depth: &wgpu::TextureView,
        aspect: f32,
        alive: Option<Vec<u32>>,
        draw_distance: u32,
        dt: f32,
    ) -> bool {
        let Some(c) = &mut self.current else { return false };
        c.render(gpu, format, encoder, target, depth, aspect, alive, draw_distance, dt);
        true
    }

    pub fn take_sounds(&mut self) -> Vec<crate::fx::sound::SoundCmd> {
        let mut out = std::mem::take(&mut self.sounds);
        if let Some(c) = &mut self.current {
            out.extend(c.take_sounds());
        }
        out
    }

    pub fn pause_sounds(&mut self) -> Vec<crate::fx::sound::SoundCmd> {
        self.current.as_mut().map(|c| c.pause_sounds()).unwrap_or_default()
    }

    pub fn after_submit(&mut self, gpu: &Gpu) {
        if let Some(c) = &mut self.current {
            c.after_submit(gpu);
        }
    }

    /// Off the Select screen: everything goes (the stops for its voices returned); the recorder stays.
    pub fn clear(&mut self) -> Vec<crate::fx::sound::SoundCmd> {
        let mut out = std::mem::take(&mut self.sounds);
        if self.current.is_none() && self.next.is_none() && self.loading.is_none() && self.wanted.is_empty() {
            return out;
        }
        for v in [self.current.take(), self.next.take()].into_iter().flatten() {
            let mut v = v;
            out.extend(v.stop_sounds());
        }
        self.loading = None;
        self.wanted.clear();
        self.failed.clear();
        out
    }
}

/// `vana-client vignette-shot`: a vignette played offscreen for `seconds`, a PNG at each of `at` (seconds), for
/// checking a character's moment without the lobby.
pub fn shot(
    req: &Request,
    record: Option<Record>,
    pick: Option<usize>,
    at: &[f32],
    out_dir: &std::path::Path,
    size: [u32; 2],
) -> Result<()> {
    let cfg = crate::config::Config::load();
    let index = Arc::new(DatIndex::open(&cfg.ffxi_dir())?);
    let keys = Arc::new(zone::Keys::from_ffximain(&std::fs::read(crate::dat::ffximain(&cfg.ffxi_dir()))?)?);
    let data = VignetteData::load_with(&index, &keys, req, record, pick)?;
    println!(
        "{:?} from {} at {:?} heading {:?}, {} path points, {:.2} tall",
        data.kind,
        data.source,
        data.path[0],
        data.forward,
        data.path.len(),
        data.height
    );
    let mut v = Vignette::new(data);
    v.effects = None;
    let gpu = Gpu::headless()?;
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let [w, h] = size;
    let target = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("vignette"),
        size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = target.create_view(&Default::default());
    let depth = crate::render::depth_view(&gpu.device, w, h);
    std::fs::create_dir_all(out_dir)?;
    let row = (w * 4).div_ceil(256) * 256;
    let dt = 1.0 / 30.0;
    let mut t = 0.0f32;
    let mut shots: Vec<f32> = at.to_vec();
    shots.sort_by(f32::total_cmp);
    for want in shots {
        // Play up to the shot's time (the camera eases every frame).
        while t + dt <= want {
            v.advance(dt);
            let _ = v.camera(dt);
            t += dt;
        }
        let mut enc = gpu.device.create_command_encoder(&Default::default());
        v.render(&gpu, format, &mut enc, (&view, Some(&target), size), &depth, w as f32 / h as f32, None, 100, 0.0);
        let readback = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("vignette readback"),
            size: (row * h) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        enc.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &target,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(row), rows_per_image: Some(h) },
            },
            wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        );
        gpu.queue.submit([enc.finish()]);
        readback.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        gpu.device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None })?;
        let data = readback.slice(..).get_mapped_range().map_err(|e| anyhow::anyhow!("readback: {e:?}"))?;
        let mut img = Vec::with_capacity((w * h * 4) as usize);
        for y in 0..h {
            let o = (y * row) as usize;
            img.extend_from_slice(&data[o..o + (w * 4) as usize]);
        }
        for px in img.chunks_mut(4) {
            px[3] = 255;
        }
        let path = out_dir.join(format!("vignette_{:05.1}.png", want));
        let f = std::fs::File::create(&path)?;
        let mut e = png::Encoder::new(std::io::BufWriter::new(f), w, h);
        e.set_color(png::ColorType::Rgba);
        e.set_depth(png::BitDepth::Eight);
        e.write_header()?.write_image_data(&img)?;
        let (eye, _, _) = v.cam.unwrap_or_default();
        println!(
            "{}: feet {:.1} {:.1} {:.1}, {:?}, sees {} ({} occluders)",
            path.display(),
            v.feet.x,
            v.feet.y,
            v.feet.z,
            v.act,
            v.data.occ.sees(eye, v.feet, v.data.height),
            v.data.occ.len()
        );
    }
    Ok(())
}

/// `vana-client vignette-scout`: each zone's best scenes into `out` (assets/timeless/vignette_scenes.json), kept
/// beside the zones already there. `tries` candidate spots a zone.
pub fn scout_zones(zones: &[u16], out: &std::path::Path, tries: usize, seeds: Option<&std::path::Path>) -> Result<()> {
    let seeds: std::collections::HashMap<String, Vec<[f32; 3]>> = seeds
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    let cfg = crate::config::Config::load();
    let index = DatIndex::open(&cfg.ffxi_dir())?;
    let keys = zone::Keys::from_ffximain(&std::fs::read(crate::dat::ffximain(&cfg.ffxi_dir()))?)?;
    let mut table: BTreeMap<u16, scout::ZoneScenes> = std::fs::read_to_string(out)
        .ok()
        .and_then(|s| serde_json::from_str::<BTreeMap<String, scout::ZoneScenes>>(&s).ok())
        .map(|m| m.into_iter().filter_map(|(k, v)| Some((k.parse().ok()?, v))).collect())
        .unwrap_or_default();
    for &z in zones {
        let t = std::time::Instant::now();
        let file = match index.read(zone::file_id(z as u32)) {
            Ok(f) => f,
            Err(e) => {
                println!("zone {z}: {e:#}");
                continue;
            }
        };
        let scene = zone::load_scene(&keys, &file)?;
        let ground = Ground::build(&scene);
        let index_scene = scout::SceneIndex::build(&scene);
        let len = if is_town(z) { 8.0 } else { 12.0 };
        let seed = 0x7173_0000 + z as u64;
        let near: Vec<Vec3> =
            seeds.get(&z.to_string()).map(|v| v.iter().map(|p| Vec3::from(*p)).collect()).unwrap_or_default();
        let walk = scout::scout_walk(&ground, (&scene, &index_scene), len, tries, 3, seed, &near);
        let fish = scout::scout_fish(&ground, (&scene, &index_scene), tries / 2, 3, seed, &near);
        println!(
            "zone {z}: {} walk (best {:.2}), {} fish (best {:.2}) in {:.1} s",
            walk.len(),
            walk.first().map_or(0.0, |s| s.score),
            fish.len(),
            fish.first().map_or(0.0, |s| s.score),
            t.elapsed().as_secs_f32()
        );
        table.insert(z, scout::ZoneScenes { walk, fish });
    }
    let named: BTreeMap<String, &scout::ZoneScenes> = table.iter().map(|(k, v)| (k.to_string(), v)).collect();
    std::fs::write(out, serde_json::to_string_pretty(&named)? + "\n")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn towns() {
        assert!(is_town(245) && is_town(50) && is_town(252));
        assert!(!is_town(104) && !is_town(251));
    }

    #[test]
    fn turning_keeps_level() {
        let f = turn(Vec3::X, 90.0);
        assert!(f.y == 0.0 && (f.length() - 1.0).abs() < 1e-5);
        assert!((yaw_of(dir_of(1.2)) - 1.2).abs() < 1e-5);
    }

    #[test]
    fn record_round_trips() {
        let r = Record {
            zone: 245,
            pos: [1.0, -2.0, 3.0],
            forward: [0.0, 1.0],
            fishing: true,
            rod: 14,
            minute: 600.0,
            weather: "rain".into(),
        };
        let s = serde_json::to_string(&r).unwrap();
        assert_eq!(serde_json::from_str::<Record>(&s).unwrap(), r);
        // An older file without the optional fields still reads.
        let old: Record = serde_json::from_str(r#"{"zone":1,"pos":[0,0,0],"forward":[1,0]}"#).unwrap();
        assert!(!old.fishing && old.weather == "suny");
    }

    #[test]
    fn chunks_take_one_per_region() {
        let anim = |n: &str| Animation {
            name: n.into(),
            frames: 10,
            speed: 1.0,
            tracks: Arc::from(Vec::new()),
            unkeyed: Arc::from(Vec::new()),
        };
        let a = vec![anim("wlk0"), anim("wlk1"), anim("wlk2"), anim("wlk0"), anim("idl0"), anim("fh10")];
        let c = chunks(&a, "wlk", false);
        assert_eq!(c.iter().map(|c| (c.0, c.1)).collect::<Vec<_>>(), vec![(0, 0), (1, 1), (2, 2)]);
        assert_eq!(chunks(&a, "wlk", true).len(), 2);
        assert_eq!(chunks(&a, "fh1", false).len(), 1);
    }
}
