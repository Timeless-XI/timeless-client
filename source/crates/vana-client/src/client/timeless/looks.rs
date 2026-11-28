//! Timeless XI: custom looks for every playable race (Mithra faces 0-24, the other races' faces 0-15). The player
//! picks hair colours (one or two tone), eye colour, face markings, lip tint and nose shade with sliders; the client
//! recolours the face texture itself and swaps it into our own model. Nothing is sent: only this client sees it for now.
//!
//! The pictures are `assets/timeless/looks/mithra<style>_*.png` (timeless-tools Custom_Faces/looks: the style's HD face
//! with neutral grey hair, iris, lips and nose and no markings, plus layer masks), and `faces.json` (each face's own
//! colours and nearest markings, the starting point and Reset). The other races recolour the face texture the model
//! already has (`Character::face_texture`): `looks/races/r<race>_s<style>_m*.png` are their masks (512 px, the Mithra
//! markings carried over in 3D) and `races.json` each face's layer medians (the neutral grey's middle) and colours:
//! - base: RGBA, the neutral face texture (1024 x 1024);
//! - m1: hair weight, tip distance (0 crown .. 1 hair ends), iris, lips;
//! - m2: nose, markings forehead / cheeks / brow;
//! - m3: markings arc / slash / freckles / crescent;
//! - m4: markings chevrons / constellation, unused, unused.
//!
//! Markings stay on the skin: the masks are clipped below the headband, off the hair and the eyes. A mirrored face mesh
//! draws its second copy (the character's right side) with the same texture; a marking on one side only gives that
//! copy a texture of its own (`ActorGpu::set_texture_sides`).
//!
//! A neutral layer's grey has its middle tone at 0.5: below it the colour is darkened (multiply), above it lightened
//! toward white (screen), so the drawn shading and strands stay.

use super::super::World;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

pub const SIZE: u32 = 1024;
/// The face textures a custom look replaces and their style (1..8): HD (faces 0-15), the Timeless colours (16-24),
/// and the stock ones (A, B).
pub const FACE_TEXTURES: [(&str, u8); 41] = [
    ("tim     tl_h00hd", 1),
    ("tim     tl_h01hd", 1),
    ("tim     tl_h02hd", 2),
    ("tim     tl_h03hd", 2),
    ("tim     tl_h04hd", 3),
    ("tim     tl_h05hd", 3),
    ("tim     tl_h06hd", 4),
    ("tim     tl_h07hd", 4),
    ("tim     tl_h08hd", 5),
    ("tim     tl_h09hd", 5),
    ("tim     tl_h10hd", 6),
    ("tim     tl_h11hd", 6),
    ("tim     tl_h12hd", 7),
    ("tim     tl_h13hd", 7),
    ("tim     tl_h14hd", 8),
    ("tim     tl_h15hd", 8),
    ("tim     tl_m1s_1", 1),
    ("tim     tl_m2b_1", 2),
    ("tim     tl_m2r_1", 2),
    ("tim     tl_m3b_1", 3),
    ("tim     tl_m4b_1", 4),
    ("tim     tl_m5s_1", 5),
    ("tim     tl_m6b_1", 6),
    ("tim     tl_m7r_1", 7),
    ("tim     tl_m8r_1", 8),
    ("tim     mt_hed_1", 1),
    ("tim     mt_h11_2", 1),
    ("tim     mt_hed_2", 2),
    ("tim     mt_h21_2", 2),
    ("tim     mt_h41_1", 3),
    ("tim     mt_h41_2", 3),
    ("tim     mt_h51_1", 4),
    ("tim     mt_h51_2", 4),
    ("tim     mt_h81_1", 5),
    ("tim     mt_h81_2", 5),
    ("tim     mt_h71_1", 6),
    ("tim     mt_h71_2", 6),
    ("tim     mt_h31_1", 7),
    ("tim     mt_h31_2", 7),
    ("tim     mt_h61_2", 8),
    ("tim     mt_h61_1", 8),
];
/// Each style's median skin brightness in its base (0..255): markings follow the skin's shading against it.
const SKIN_MID: [f32; 8] = [90.3, 89.9, 88.7, 86.4, 91.6, 91.9, 87.4, 90.7];
/// The markings, in mask order (Mithra; the other races have RACE_MARKS in the same nine slots).
pub const MARKS: [&str; 12] = [
    "Swept crest",
    "Cheek stripes",
    "Brow & cheek line",
    "Eye arc",
    "Claw slashes",
    "Freckles",
    "Crescent & tears",
    "War chevrons",
    "Constellation",
    // 2026-11-23 owner picks: m2 R (the old nose channel), m4 B, m4 A.
    "Whisker marks",
    "Fang tears",
    "Sunset band",
];
/// The markings that can go on one side only (under one eye).
pub const SIDED: [bool; 12] = [false, true, true, true, true, false, true, true, true, false, false, false];

/// How a race marking is drawn: in the marking colour, as darker skin (freckles) or as a raised pink scar.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Ink,
    Freckle,
    Scar,
}
use Kind::{Freckle, Ink, Scar};

/// Each race's own markings (owner picks 2026-11-23), in mask slot order (m2 G B A, m3 R G B A, m4 R G); drawn on both
/// sides. Made by Custom_Faces/looks/race_marks.py - keep the two lists the same.
pub fn race_marks(race: u8) -> &'static [(&'static str, Kind)] {
    match race {
        1 => &[("Freckles", Freckle), ("Steam line", Ink), ("Smelter's soot", Ink), ("Cartographer's cross", Ink), ("Foreman's chevrons", Ink)],
        2 => &[("Freckles", Freckle), ("Steam line", Ink), ("Cartographer's cross", Ink), ("Filigree scroll", Ink), ("Envoy's crescent", Ink)],
        3 => &[
            ("Freckles", Freckle),
            ("Fleur sigil", Ink),
            ("Shield bars", Ink),
            ("Sword scar", Scar),
            ("Claw rake", Scar),
            ("Iron jaw", Ink),
            ("Lance stripes", Ink),
            ("Knight's chevron", Ink),
        ],
        4 => &[("Freckles", Freckle), ("Fleur sigil", Ink), ("Shield bars", Ink), ("Lance stripes", Ink), ("Blade tear", Ink), ("Knight's chevron", Ink)],
        5 => &[("Mana drop", Ink), ("Freckles", Freckle), ("Star map", Ink)],
        6 => &[("Mana drop", Ink), ("Freckles", Freckle), ("Crescent moon", Ink), ("Petal spray", Ink), ("Comet trail", Ink), ("Twin stars", Ink)],
        8 => &[("Freckles", Freckle), ("Ember streak", Ink), ("Ash stripes", Ink), ("Brow band", Ink)],
        _ => &[],
    }
}

/// The marking names the window lists for a race.
pub fn mark_names(race: u8) -> Vec<&'static str> {
    if race == 7 { MARKS.to_vec() } else { race_marks(race).iter().map(|m| m.0).collect() }
}
/// A marking's sides (`Settings::marks`): bit 0 the left (the mesh as stored), bit 1 the right (its mirrored copy).
pub const LEFT: u8 = 1;
pub const RIGHT: u8 = 2;
pub const BOTH: u8 = 3;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// The custom look is drawn.
    pub on: bool,
    /// A Mithra of another face style shows face 6 (this client only) to try the look.
    pub try_on: bool,
    pub hair: [u8; 3],
    pub two_tone: bool,
    pub tip: [u8; 3],
    /// Where the tip colour starts (0 crown .. 1 hair ends) and how wide the blend is.
    pub tip_start: f32,
    pub tip_blend: f32,
    pub eyes: [u8; 3],
    pub lips: [u8; 3],
    /// 0 grey .. 1 black.
    pub nose: f32,
    /// Per marking (MARKS order): 0 off, LEFT, RIGHT or BOTH.
    #[serde(deserialize_with = "marks_de")]
    pub marks: Vec<u8>,
    pub mark_colour: [u8; 3],
    pub mark_strength: f32,
    /// Cheek blush, 0 none .. 1 a soft flush (owner 2026-11-28: "subtle and realistic").
    pub blush: f32,
}

impl Default for Settings {
    /// Off; the colours close to face 6A. A character's first settings are its own face's (`seed`).
    fn default() -> Self {
        Settings {
            on: false,
            try_on: false,
            hair: [232, 226, 216],
            two_tone: false,
            tip: [214, 178, 118],
            tip_start: 0.6,
            tip_blend: 0.2,
            eyes: [111, 143, 168],
            lips: [154, 106, 90],
            nose: 0.6,
            marks: vec![BOTH, BOTH, 0, 0, 0, 0, 0, 0, 0],
            mark_colour: [90, 52, 32],
            mark_strength: 0.85,
            blush: 0.0,
        }
    }
}

/// Markings as saved: sides per marking, or (the first proof) a tick per marking; padded to every marking.
fn marks_de<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
    let v: Vec<serde_json::Value> = Deserialize::deserialize(d)?;
    let mut out: Vec<u8> = v
        .iter()
        .map(|x| match x {
            serde_json::Value::Bool(b) => if *b { BOTH } else { 0 },
            serde_json::Value::Number(n) => n.as_u64().unwrap_or(0).min(3) as u8,
            _ => 0,
        })
        .collect();
    out.resize(MARKS.len(), 0);
    Ok(out)
}

/// A face's own colours (faces.json).
#[derive(Deserialize)]
struct Seed {
    hair: [u8; 3],
    two_tone: bool,
    tip: [u8; 3],
    eyes: [u8; 3],
    lips: [u8; 3],
    marks: Vec<u8>,
    mark_colour: [u8; 3],
}

fn seeds() -> &'static HashMap<u16, Seed> {
    static S: OnceLock<HashMap<u16, Seed>> = OnceLock::new();
    S.get_or_init(|| serde_json::from_str(include_str!("../../../assets/timeless/looks/faces.json")).unwrap_or_default())
}

/// A Mithra face's own look as settings (off; the other settings as given), None for a face we have no look for.
pub fn seed(face: u16, keep: &Settings) -> Option<Settings> {
    let f = seeds().get(&face)?;
    let mut marks = f.marks.clone();
    marks.resize(MARKS.len(), 0);
    Some(Settings {
        on: keep.on,
        try_on: keep.try_on,
        hair: f.hair,
        two_tone: f.two_tone,
        tip: f.tip,
        eyes: f.eyes,
        lips: f.lips,
        marks,
        mark_colour: f.mark_colour,
        ..Settings::default()
    })
}

/// One model per style (owner, 2026-11-19: the A and B faces share a hair style and the custom look covers the rest):
/// with a custom look, the other races' B faces are drawn as their style's A face; Mithra already uses a neutral base
/// per style.
pub fn model_face(race: u8, face: u16) -> u16 {
    match race {
        1..=6 | 8 if face <= 15 => face & !1,
        _ => face,
    }
}

/// The faces with a custom look (Mithra 0-24, the other races 0-15).
pub fn supported(race: u8, face: u16) -> bool {
    match race {
        7 => face <= 24,
        1..=6 | 8 => face <= 15,
        _ => false,
    }
}

/// Another race's face (races.json): layer medians (hair, iris, lips, nose, skin on 0..255), its own colours, which
/// layers it has, and whether its markings have a right side of their own (a face mesh that isn't mirrored).
#[derive(Deserialize)]
pub struct RaceFace {
    /// The race (from the races.json key).
    #[serde(default)]
    pub race: u8,
    /// The face number (Mithra: which face's markings come out).
    #[serde(default)]
    pub face: u16,
    pub style: u8,
    med: [f32; 5],
    hair: [u8; 3],
    two_tone: bool,
    tip: [u8; 3],
    eyes: [u8; 3],
    lips: [u8; 3],
    pub has: Has,
    pub sides: bool,
}

#[derive(Deserialize, Clone, Copy)]
pub struct Has {
    pub iris: bool,
    pub lips: bool,
    pub nose: bool,
    /// Face markings: Mithra only for now (owner 2026-11-23: the other races get their own sets later).
    #[serde(default)]
    pub marks: bool,
    /// The race (the window's marking list).
    #[serde(default)]
    pub race: u8,
}

fn race_faces() -> &'static HashMap<String, RaceFace> {
    static S: OnceLock<HashMap<String, RaceFace>> = OnceLock::new();
    S.get_or_init(|| {
        let mut m: HashMap<String, RaceFace> =
            serde_json::from_str(include_str!("../../../assets/timeless/looks/races/races.json")).unwrap_or_default();
        for (k, f) in m.iter_mut() {
            f.race = k.split('_').next().and_then(|r| r.parse().ok()).unwrap_or(0);
        }
        m
    })
}

pub fn race_face(race: u8, face: u16) -> Option<&'static RaceFace> {
    race_faces().get(&format!("{race}_{face}"))
}

/// The layers a face has (the window hides the others).
pub fn layers_of(race: u8, face: u16) -> Has {
    match race_face(race, face) {
        Some(f) if race != 7 => Has { marks: !race_marks(race).is_empty(), race, ..f.has },
        _ => Has { iris: true, lips: true, nose: true, marks: true, race: 7 },
    }
}

/// Any race's face as settings: its own colours (Mithra: and nearest markings); the other settings as given.
pub fn seed_any(race: u8, face: u16, keep: &Settings) -> Option<Settings> {
    if race == 7 && race_face(7, face).is_none() {
        return seed(face, keep);
    }
    let f = race_face(race, face)?;
    Some(Settings {
        hair: f.hair,
        two_tone: f.two_tone,
        tip: f.tip,
        eyes: f.eyes,
        lips: f.lips,
        // Another race's own markings (the slots mean something else on each race).
        marks: vec![0; MARKS.len()],
        ..keep.clone()
    })
}

/// The race names for the window, by race id.
pub const RACES: [(u8, &str); 8] = [
    (1, "Hume M"),
    (2, "Hume F"),
    (3, "Elvaan M"),
    (4, "Elvaan F"),
    (5, "Tarutaru M"),
    (6, "Tarutaru F"),
    (7, "Mithra"),
    (8, "Galka"),
];

impl Settings {
    /// A marking's sides, a both-sides-only one never one-sided.
    pub fn mark(&self, k: usize) -> u8 {
        let v = self.marks.get(k).copied().unwrap_or(0) & BOTH;
        if v != 0 && !SIDED[k] { BOTH } else { v }
    }

    /// Some marking is on one side only: the two sides need textures of their own.
    pub fn one_sided(&self) -> bool {
        (0..MARKS.len()).any(|k| matches!(self.mark(k), LEFT | RIGHT))
    }
}

/// The world's custom-look state.
#[derive(Default)]
pub struct Looks {
    pub settings: Settings,
    /// The character the settings belong to (loaded on a name change).
    pub name: String,
    pub open: bool,
    /// The face (race, face, texture) and settings the texture on the GPU was made from; a newer one waits for its mips.
    made: Option<(MadeKey, Settings)>,
    /// The new textures (left, right when one-sided) waiting for their mips.
    pending: Option<(u64, wgpu::BindGroup, Option<wgpu::BindGroup>, (MadeKey, Settings))>,
    current: Option<(wgpu::BindGroup, Option<wgpu::BindGroup>)>,
    /// A look being made on a worker thread (never on the game's own: the sliders must not stutter the game).
    job: Option<std::sync::mpsc::Receiver<Made>>,
    /// The face's Prep (another race's face), kept while the face stays the same.
    prep: Option<(PrepKey, std::sync::Arc<Prep>)>,
    /// Bumped when `try_on` changes: our look loads again.
    pub try_on_ver: u32,
    pub try_on_seen: u32,
    /// Our model has a face a custom look is made for (the window says when it does not), and its style (1..8).
    pub fits: bool,
    pub style: u8,
    /// Our face (the server's look, before any try-on), for Reset.
    pub face: Option<(u8, u16)>,
    /// No settings saved for this character yet: they start from its face once the face is known.
    needs_seed: bool,
    dirty_save: Option<std::time::Instant>,
    /// Testing: our model drawn as another race / face here (race, face); nothing is sent.
    pub preview: Option<(u8, u16)>,
    /// The face drawn now (the preview's or our own) and its layers.
    pub shown: Option<(u8, u16)>,
    /// The free camera (orbits our character; the drag and wheel steer it).
    pub cam: FreeCam,
    /// Characters' back faces lit by their flipped normal (the inside of manes and hair not near black; render
    /// `two_sided_actors`). On unless turned off (`timeless-light.json`), for every character on this PC.
    pub light_both_sides: bool,
    light_loaded: bool,
    /// The window's "Hold to see retail lighting" is held (both sides off while held).
    pub hold_retail: bool,
    /// Debug: characters drawn in a flat colour per texture (not saved).
    pub debug_pieces: bool,
}

fn light_path() -> std::path::PathBuf {
    crate::config::config_path().with_file_name("timeless-light.json")
}

/// "Light both sides" changed (the window): kept for every character on this PC.
pub fn set_light_both_sides(l: &mut Looks, on: bool) {
    l.light_both_sides = on;
    let _ = std::fs::write(light_path(), if on { "{\"both_sides\": true}" } else { "{\"both_sides\": false}" });
}

/// The free camera: around our character's head, `yaw` from its front, `pitch` up, `dist` away, `height` (0 feet .. 1
/// the head's top), turning by itself with `spin`.
#[derive(Clone, Copy)]
pub struct FreeCam {
    pub on: bool,
    pub yaw: f32,
    pub pitch: f32,
    pub dist: f32,
    pub height: f32,
    pub spin: bool,
    pub last: Option<std::time::Instant>,
}

impl Default for FreeCam {
    fn default() -> Self {
        FreeCam { on: false, yaw: 0.0, pitch: 0.1, dist: 1.0, height: 0.93, spin: false, last: None }
    }
}

impl FreeCam {
    pub fn reset_view(&mut self) {
        let on = self.on;
        let spin = self.spin;
        *self = FreeCam { on, spin, ..FreeCam::default() };
    }
}

type MadeKey = (u8, u16, String);
type PrepKey = (u8, u16, String, u32, u32);

/// A look made on the worker: for these settings, its left (and when one-sided, right) texture, w x h, and the Prep it
/// used (kept for the next one).
struct Made {
    want: (MadeKey, Settings),
    w: u32,
    h: u32,
    left: Vec<u8>,
    right: Option<Vec<u8>>,
    prep: Option<(PrepKey, std::sync::Arc<Prep>)>,
}

struct Layers {
    base: Vec<u8>,
    m1: Vec<u8>,
    m2: Vec<u8>,
    m3: Vec<u8>,
    m4: Vec<u8>,
    /// Cheek blush (grey in R, same size as m1).
    blush: Vec<u8>,
}

fn png(bytes: &[u8]) -> Vec<u8> {
    png_sized(bytes, SIZE)
}

fn png_sized(bytes: &[u8], size: u32) -> Vec<u8> {
    let dec = png::Decoder::new(std::io::Cursor::new(bytes));
    let Ok(mut r) = dec.read_info() else { return vec![0; (size * size * 4) as usize] };
    let mut buf = vec![0; r.output_buffer_size().unwrap_or(0)];
    match r.next_frame(&mut buf) {
        Ok(info) if info.width == size && info.height == size && info.color_type == png::ColorType::Rgba => {
            buf.truncate(info.buffer_size());
            buf
        }
        _ => vec![0; (size * size * 4) as usize],
    }
}

/// A square RGBA mask of 512 or 1024 (a style whose texture was remapped to 2048 has 1024 masks); else RACE_SIZE zeros.
fn png_square(bytes: &[u8]) -> Vec<u8> {
    let dec = png::Decoder::new(std::io::Cursor::new(bytes));
    let fail = || vec![0; (RACE_SIZE * RACE_SIZE * 4) as usize];
    let Ok(mut r) = dec.read_info() else { return fail() };
    let mut buf = vec![0; r.output_buffer_size().unwrap_or(0)];
    match r.next_frame(&mut buf) {
        Ok(info)
            if info.width == info.height
                && matches!(info.width, 512 | 1024)
                && info.color_type == png::ColorType::Rgba =>
        {
            buf.truncate(info.buffer_size());
            buf
        }
        _ => fail(),
    }
}

/// The other races' mask size (the usual one; see png_square).
pub const RACE_SIZE: u32 = 512;

/// Another race's style masks (m1..m4, and m5 / m6 when the right side has its own), decoded when first used.
pub struct RaceLayers {
    m: Vec<Vec<u8>>,
    /// The cheek blush map (grey), the same size and layout as m1; empty when the style has none.
    blush: Vec<u8>,
}

pub fn race_layers(race: u8, style: u8) -> Option<&'static RaceLayers> {
    static L: OnceLock<HashMap<(u8, u8), OnceLock<RaceLayers>>> = OnceLock::new();
    let all = L.get_or_init(|| {
        super::look_files::RACE_MASKS.iter().map(|(r, s, _)| ((*r, *s), OnceLock::new())).collect()
    });
    let cell = all.get(&(race, style))?;
    Some(cell.get_or_init(|| {
        let files = super::look_files::RACE_MASKS.iter().find(|(r, s, _)| (*r, *s) == (race, style)).map(|x| x.2);
        let blush = super::look_files::BLUSH.iter().find(|(r, s, _)| (*r, *s) == (race, style)).map_or(Vec::new(), |x| png_grey(x.2));
        RaceLayers { m: files.unwrap_or(&[]).iter().map(|b| png_square(b)).collect(), blush }
    }))
}

/// A mask's four channels at texture position (u, v) in 0..1, bilinear.
fn sample(m: &[u8], u: f32, v: f32) -> [f32; 4] {
    let n = ((m.len() / 4) as f64).sqrt() as usize;
    if n == 0 {
        return [0.0; 4];
    }
    let x = (u * n as f32 - 0.5).clamp(0.0, (n - 1) as f32);
    let y = (v * n as f32 - 0.5).clamp(0.0, (n - 1) as f32);
    let (x0, y0) = (x as usize, y as usize);
    let (x1, y1) = ((x0 + 1).min(n - 1), (y0 + 1).min(n - 1));
    let (fx, fy) = (x - x0 as f32, y - y0 as f32);
    let px = |xx: usize, yy: usize, c: usize| m[(yy * n + xx) * 4 + c] as f32;
    std::array::from_fn(|c| {
        let a = px(x0, y0, c) * (1.0 - fx) + px(x1, y0, c) * fx;
        let b = px(x0, y1, c) * (1.0 - fx) + px(x1, y1, c) * fx;
        (a * (1.0 - fy) + b * fy) / 255.0
    })
}

/// Another race's face texture (RGBA w x h, the model's own) recoloured. A mirrored face: the markings on `side`; one
/// with sides of its own (`sides`): every marking on its own side(s) in one texture.
pub fn compose_race(rl: &'static RaceLayers, f: &'static RaceFace, tex: &[u8], w: u32, h: u32, s: &Settings, side: u8) -> Vec<u8> {
    compose_prep(&Prep::new(rl, f, tex.to_vec(), w, h), s, side)
}

/// What a face texture's look needs that doesn't change with the sliders, worked out once per face: the hair / iris /
/// lips / nose masks at the texture's size and the hair's grey (the strands' shading). Making a look is then a few mixes
/// a texel (the /look sliders: 2026-11-22, a look took 100+ ms a move and stuttered the game).
pub struct Prep {
    rl: &'static RaceLayers,
    f: &'static RaceFace,
    w: u32,
    h: u32,
    tex: Vec<u8>,
    /// m1 (hair, tip, iris, lips) and m2 (nose, markings) at each texel.
    a: Vec<[u8; 4]>,
    b: Vec<[u8; 4]>,
    /// The hair's grey at each texel (0..255, middle tone 128).
    g: Vec<u8>,
    /// The cheek blush weight at each texel (empty: none).
    bl: Vec<u8>,
    /// The texels with any marking: index, the nine markings' cover (MARKS order) and the right-side masks' seven.
    marks: Vec<(u32, [u8; 12], [u8; 7])>,
}

impl Prep {
    pub fn new(rl: &'static RaceLayers, f: &'static RaceFace, tex: Vec<u8>, w: u32, h: u32) -> Self {
        let n = (w * h) as usize;
        // Mithra: the face's own markings out of the look (owner 2026-11-24), the skin there only; the custom
        // markings take their place.
        let un = mithra_unmark(f, w, h);
        let mut tex = tex;
        if let Some(un) = &un {
            for (t, u) in tex.chunks_exact_mut(4).zip(un.chunks_exact(4)) {
                if u[3] > 0 {
                    let k = u[3] as f32 / 255.0;
                    for c in 0..3 {
                        t[c] = (t[c] as f32 * (1.0 - k) + u[c] as f32 * k + 0.5) as u8;
                    }
                }
            }
        }
        let (mut a, mut b, mut g) = (vec![[0u8; 4]; n], vec![[0u8; 4]; n], vec![0u8; n]);
        let none = vec![0u8; 0];
        let (m1, m2) = (rl.m.first().unwrap_or(&none), rl.m.get(1).unwrap_or(&none));
        if !m1.is_empty() {
            let ks = mask_scale(rl, w, f.race);
            let level = hair_level(rl, &tex, w, h, ks);
            let mh = f.med[0].max(20.0) / 255.0;
            let threads = std::thread::available_parallelism().map_or(4, |n| n.get()).clamp(1, 8);
            let per = (h as usize).div_ceil(threads) * w as usize;
            let (tex, level, un) = (&tex, &level, un.as_deref());
            std::thread::scope(|sc| {
                for (k, ((ca, cb), cg)) in a.chunks_mut(per).zip(b.chunks_mut(per)).zip(g.chunks_mut(per)).enumerate() {
                    sc.spawn(move || {
                        for (j, ((pa, pb), pg)) in ca.iter_mut().zip(cb.iter_mut()).zip(cg.iter_mut()).enumerate() {
                            let i = k * per + j;
                            let (x, y) = (i % w as usize, i / w as usize);
                            let (u, v) = ((x as f32 + 0.5) / w as f32, (y as f32 + 0.5) / h as f32);
                            let q = |c: [f32; 4]| c.map(|v| (v * 255.0 + 0.5) as u8);
                            let mut sa = sample(m1, u * ks, v * ks);
                            if let Some(un) = un {
                                sa[0] *= 1.0 - un[i * 4 + 3] as f32 / 255.0;
                            }
                            *pa = q(sa);
                            *pb = q(sample(m2, u * ks, v * ks));
                            if sa[0] > 0.0 {
                                let p = &tex[i * 4..i * 4 + 3];
                                let l = (0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32) / 255.0;
                                // The HD strands' contrast eased (their dark gaps went black, their highlights white,
                                // whatever the colour).
                                let gg = l * 0.5 / hair_med(mh, level_at(level, u, v));
                                let gg = (0.5 + (gg - 0.5) * if gg > 0.5 { 0.3 } else { 0.55 }).clamp(0.0, 1.0);
                                *pg = (gg * 255.0 + 0.5) as u8;
                            }
                        }
                    });
                }
            });
        }
        // Markings: a short list of the texels they touch.
        let mut marks = Vec::new();
        let (m3, m4, m5, m6) = (rl.m.get(2), rl.m.get(3), rl.m.get(4), rl.m.get(5));
        if let (Some(m3), Some(m4)) = (m3, m4) {
            let ks = mask_scale(rl, w, f.race);
            for i in 0..n {
                let (x, y) = (i % w as usize, i / w as usize);
                let (u, v) = (ks * (x as f32 + 0.5) / w as f32, ks * (y as f32 + 0.5) / h as f32);
                let (c3, d) = (sample(m3, u, v), sample(m4, u, v));
                let bb = b[i];
                let q = |v: f32| (v * 255.0 + 0.5) as u8;
                // MARKS order; Mithra's last three (whisker marks, fang tears, sunset band) in m2 R, m4 B, m4 A.
                let (x0, x1, x2) = if f.race == 7 { (bb[0], q(d[2]), q(d[3])) } else { (0, 0, 0) };
                let left = [bb[1], bb[2], bb[3], q(c3[0]), q(c3[1]), q(c3[2]), q(c3[3]), q(d[0]), q(d[1]), x0, x1, x2];
                let mut right = [0u8; 7];
                if let (Some(m5), Some(m6)) = (m5.filter(|m| !m.is_empty()), m6.filter(|m| !m.is_empty())) {
                    let (e, g6) = (sample(m5, u, v), sample(m6, u, v));
                    right = [q(e[0]), q(e[1]), q(e[2]), q(e[3]), q(g6[0]), q(g6[1]), q(g6[2])];
                }
                if left.iter().chain(&right).any(|v| *v > 0) {
                    marks.push((i as u32, left, right));
                }
            }
        }
        let mut bl = Vec::new();
        if !rl.blush.is_empty() {
            let ks = mask_scale(rl, w, f.race);
            bl = (0..n)
                .map(|i| {
                    let (x, y) = (i % w as usize, i / w as usize);
                    let (u, v) = (ks * (x as f32 + 0.5) / w as f32, ks * (y as f32 + 0.5) / h as f32);
                    (sample(&rl.blush, u, v)[0] * 255.0 + 0.5) as u8
                })
                .collect();
        }
        Prep { rl, f, w, h, tex, a, b, g, bl, marks }
    }
}

/// A Mithra face's markings-out layer at the texture's size (1024 only), None for other faces.
fn mithra_unmark(f: &RaceFace, w: u32, h: u32) -> Option<Vec<u8>> {
    if f.race != 7 || (w, h) != (1024, 1024) {
        return None;
    }
    let (_, b) = super::look_files::MITHRA_UNMARK.iter().find(|(k, _)| *k == f.face)?;
    let v = png_square(b);
    (v.len() == 1024 * 1024 * 4 && v.iter().skip(3).step_by(4).any(|a| *a > 0)).then_some(v)
}

/// A look from a face's Prep (see compose_race).
pub fn compose_prep(p: &Prep, s: &Settings, side: u8) -> Vec<u8> {
    let mut out = p.tex.clone();
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get()).clamp(1, 8);
    let per = (p.h as usize).div_ceil(threads) * p.w as usize * 4;
    std::thread::scope(|sc| {
        for (k, chunk) in out.chunks_mut(per).enumerate() {
            sc.spawn(move || compose_race_rows(p, chunk, k * per / 4, s, side));
        }
    });
    marks_pass(p, &mut out, s, side);
    out
}

/// The markings over a composed look (only the texels they touch).
fn marks_pass(p: &Prep, out: &mut [u8], s: &Settings, side: u8) {
    let f = p.f;
    // Each race's own markings (2026-11-23): drawn on both sides, in the nine slots; no right-side masks.
    // Mithra (on its own HD face, 2026-11-24): the twelve markings in ink, each on the sides picked.
    const MITHRA: [(&str, Kind); 12] = [("", Ink); 12];
    let list: &[(&str, Kind)] = if f.race == 7 { &MITHRA } else { race_marks(f.race) };
    let left_on: Vec<bool> = (0..12)
        .map(|k| k < list.len() && if f.race == 7 { s.mark(k) & side != 0 } else { s.marks.get(k).copied().unwrap_or(0) != 0 })
        .collect();
    let right_on = [false; 12];
    let right = false;
    if !(left_on.iter().any(|o| *o) && s.mark_strength > 0.0) {
        return;
    }
    let rosy = matches!(f.race, 5 | 6);
    let strength = s.mark_strength.clamp(0.0, 1.0);
    let msk = f.med[4].max(20.0) / 255.0;
    let mark = unit(s.mark_colour);
    for (i, left, rt) in &p.marks {
        let i = *i as usize;
        let t = &p.tex[i * 4..i * 4 + 3];
        let l = (0.299 * t[0] as f32 + 0.587 * t[1] as f32 + 0.114 * t[2] as f32) / 255.0;
        let px = &mut out[i * 4..i * 4 + 3];
        let mut c = [px[0] as f32 / 255.0, px[1] as f32 / 255.0, px[2] as f32 / 255.0];
        let sk = l / msk;
        let mc = mark.map(|x| (x * sk).min(1.0));
        for (k, v) in left.iter().enumerate() {
            if left_on[k] && *v > 0 {
                let a = *v as f32 / 255.0 * strength;
                c = match list[k].1 {
                    Ink => mix(c, mc, a),
                    // The skin itself, darker and warmer (Tarutaru: rosy).
                    Freckle => {
                        let t = if rosy { [0.78, 0.6, 0.55] } else { [0.66, 0.55, 0.48] };
                        mix(c, [c[0] * t[0], c[1] * t[1], c[2] * t[2]], a * 0.75)
                    }
                    // A healed scar: raised, lighter and pinker than the skin round it.
                    Scar => mix(c, [(c[0] * 1.3 + 0.16).min(1.0), (c[1] * 1.3 + 0.03).min(1.0), (c[2] * 1.3 + 0.04).min(1.0)], a * 0.9),
                };
            }
        }
        if right {
            // Right-side masks, MARKS order: cheeks, brow, arc, slash, crescent, chevrons, constellation.
            for (k, v) in [1usize, 2, 3, 4, 6, 7, 8].iter().zip(rt) {
                if right_on[*k] && *v > 0 {
                    c = mix(c, mc, *v as f32 / 255.0 * strength);
                }
            }
        }
        for k in 0..3 {
            px[k] = (c[k].clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
        }
    }
}

#[allow(clippy::too_many_arguments)]
/// The hair's local brightness (LEVEL x LEVEL cells over the texture, hair-weighted mean luma; cells without hair take
/// their neighbours'), so near-black parts of a mane (Galka 1A's back) recolour as hair, not as black.
const LEVEL: usize = 32;
/// A 1024 mask set is laid out for a remapped 2048 face (the old layout in its top-left quarter); on a face that isn't
/// remapped (the stock 128 face, an older HD one) the masks are read from that quarter: (u, v) x 0.5.
/// Mithra's masks are 1024 in the face's own layout (no remap): read whole.
fn mask_scale(rl: &RaceLayers, w: u32, race: u8) -> f32 {
    let big = rl.m.first().is_some_and(|m| m.len() == 1024 * 1024 * 4);
    if big && w < 2048 && race != 7 { 0.5 } else { 1.0 }
}

fn hair_level(rl: &RaceLayers, tex: &[u8], w: u32, h: u32, k: f32) -> Vec<f32> {
    let (mut sum, mut wt) = (vec![0f32; LEVEL * LEVEL], vec![0f32; LEVEL * LEVEL]);
    let Some(m1) = rl.m.first() else { return sum };
    for y in (0..h as usize).step_by(2) {
        for x in (0..w as usize).step_by(2) {
            let a = sample(m1, k * (x as f32 + 0.5) / w as f32, k * (y as f32 + 0.5) / h as f32)[0];
            if a < 0.5 {
                continue;
            }
            let p = &tex[(y * w as usize + x) * 4..];
            let l = (0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32) / 255.0;
            let k = (y * LEVEL / h as usize) * LEVEL + x * LEVEL / w as usize;
            sum[k] += a * l;
            wt[k] += a;
        }
    }
    let mut lv: Vec<f32> = sum.iter().zip(&wt).map(|(s, w)| if *w > 0.0 { s / w } else { -1.0 }).collect();
    for _ in 0..LEVEL {
        let prev = lv.clone();
        let mut any = false;
        for k in 0..LEVEL * LEVEL {
            if prev[k] >= 0.0 {
                continue;
            }
            let (x, y) = ((k % LEVEL) as i32, (k / LEVEL) as i32);
            let (mut s, mut n) = (0.0, 0);
            for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                let (xx, yy) = (x + dx, y + dy);
                if (0..LEVEL as i32).contains(&xx) && (0..LEVEL as i32).contains(&yy) {
                    let v = prev[yy as usize * LEVEL + xx as usize];
                    if v >= 0.0 {
                        s += v;
                        n += 1;
                    }
                }
            }
            if n > 0 {
                lv[k] = s / n as f32;
            } else {
                any = true;
            }
        }
        if !any {
            break;
        }
    }
    lv.iter().map(|v| v.max(0.0)).collect()
}

/// The hair's level at (u, v), bilinear between cell centres.
fn level_at(lv: &[f32], u: f32, v: f32) -> f32 {
    let x = (u * LEVEL as f32 - 0.5).clamp(0.0, (LEVEL - 1) as f32);
    let y = (v * LEVEL as f32 - 0.5).clamp(0.0, (LEVEL - 1) as f32);
    let (x0, y0) = (x as usize, y as usize);
    let (x1, y1) = ((x0 + 1).min(LEVEL - 1), (y0 + 1).min(LEVEL - 1));
    let (fx, fy) = (x - x0 as f32, y - y0 as f32);
    let p = |xx: usize, yy: usize| lv[yy * LEVEL + xx];
    (p(x0, y0) * (1.0 - fx) + p(x1, y0) * fx) * (1.0 - fy) + (p(x0, y1) * (1.0 - fx) + p(x1, y1) * fx) * fy
}

/// The hair's median for a texel: where the hair around it is darker than the face's hair median, mostly its own
/// level (dark parts keep their strands and take the colour).
fn hair_med(mh: f32, local: f32) -> f32 {
    if local >= mh || local <= 0.0 {
        return mh;
    }
    (mh.ln() * 0.3 + local.max(0.012).ln() * 0.7).exp()
}

fn compose_race_rows(p: &Prep, out: &mut [u8], at: usize, s: &Settings, _side: u8) {
    let f = p.f;
    if p.rl.m.first().is_none_or(|m| m.is_empty()) {
        return;
    }
    let (hair, tip, eyes, lips) = (tone(unit(s.hair), Tone::Hair), tone(unit(s.tip), Tone::Hair), tone(unit(s.eyes), Tone::Eyes), tone(unit(s.lips), Tone::Lips));
    let nose = [nose_grey(s.nose); 3];
    let (t0, t1) = (s.tip_start - s.tip_blend, s.tip_start + s.tip_blend.max(0.005));
    // Floors (hair and skin 20, iris, lips and nose 40, as the tools' preview): a layer nearly black on the face
    // (Galka lips) would blow every texel of it to white.
    let fl = [20.0, 40.0, 40.0, 40.0, 20.0];
    let [_, mi, ml, mn, _] = std::array::from_fn(|k| f.med[k].max(fl[k]) / 255.0);
    let hw = |n: f32| n.clamp(0.0, 1.0);
    let un = |v: u8| v as f32 / 255.0;
    // The hair colour at each grey, worked out once (two-tone: per texel, by its tip level).
    let one: Vec<[f32; 3]> = if s.two_tone { Vec::new() } else { (0..256).map(|g| hair_shade(g as f32 / 255.0, hair)).collect() };
    for (i, px) in out.chunks_exact_mut(4).enumerate() {
        let k = at + i;
        let a = p.a[k];
        let n0 = p.b[k][0];
        let bw = if s.blush > 0.0 { p.bl.get(k).copied().unwrap_or(0) } else { 0 };
        if a == [0; 4] && n0 == 0 && bw == 0 {
            continue;
        }
        let mut c = [px[0] as f32 / 255.0, px[1] as f32 / 255.0, px[2] as f32 / 255.0];
        let l = 0.299 * c[0] + 0.587 * c[1] + 0.114 * c[2];
        if a[0] > 0 {
            let hc = if s.two_tone {
                hair_shade(un(p.g[k]), mix(hair, tip, smoothstep(t0, t1, un(a[1]))))
            } else {
                one[p.g[k] as usize]
            };
            c = mix(c, hc, un(a[0]));
        }
        // Iris, lips, nose: their highlights eased (a dark layer's median would blow them to white).
        let knee = |g: f32| if g > 0.5 { 0.5 + (g - 0.5) * 0.4 } else { g };
        if a[2] > 0 {
            c = mix(c, shade(hw(knee(l * 0.5 / mi)), eyes), un(a[2]));
        }
        if a[3] > 0 {
            c = mix(c, shade(hw(knee(l * 0.5 / ml)), lips), un(a[3]));
        }
        if bw > 0 {
            c = blush(c, un(bw) * s.blush.clamp(0.0, 1.0));
        }
        // Nose: left as the model's own (owner 2026-11-23, every race) - no recolour.
        let _ = (n0, mn, nose);
        for k in 0..3 {
            px[k] = (c[k].clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
        }
    }
}

/// The hair's colour at a strand grey g (0..1, middle tone 0.5). 2026-11-22 (owner: colours "super washed out ...
/// after about 80-100"): the strands' shading is a multiple of the colour (darker strands stay darker however bright
/// the colour), highlights a lighter shade with a little white (more for dark colours, so black hair keeps its shine).
fn hair_shade(g: f32, col: [f32; 3]) -> [f32; 3] {
    let t = (g - 0.5) * 2.0;
    let dark = (t.min(0.0) * 1.25).exp();
    let lite = t.max(0.0);
    let wl = 0.22 + 0.2 * (1.0 - luma(col));
    std::array::from_fn(|k| col[k] * dark * (1.0 + lite * 0.45) + (1.0 - col[k]) * lite * wl)
}

/// Cheek blush: the skin itself a little warmer and rosier (red kept, green and blue drawn down), never a paint over
/// it, so it follows every skin tone and the face's own shading. `a` 0..1 (map weight x the slider).
fn blush(c: [f32; 3], a: f32) -> [f32; 3] {
    let t = [c[0] * 1.04 + 0.015, c[1] * 0.86, c[2] * 0.88];
    mix(c, t, (a * 0.6).clamp(0.0, 1.0))
}

/// A grey PNG as RGBA (the grey in R), any square size; empty if it can't be read.
fn png_grey(bytes: &[u8]) -> Vec<u8> {
    let dec = png::Decoder::new(std::io::Cursor::new(bytes));
    let Ok(mut r) = dec.read_info() else { return Vec::new() };
    let mut buf = vec![0; r.output_buffer_size().unwrap_or(0)];
    match r.next_frame(&mut buf) {
        Ok(info) if info.width == info.height && info.color_type == png::ColorType::Grayscale && info.bit_depth == png::BitDepth::Eight => {
            buf.truncate(info.buffer_size());
            buf.iter().flat_map(|g| [*g, 0, 0, 0]).collect()
        }
        _ => Vec::new(),
    }
}

fn luma(c: [f32; 3]) -> f32 {
    0.299 * c[0] + 0.587 * c[1] + 0.114 * c[2]
}

/// What a picked colour is used for: each kind is kept in its own realistic range (owner 2026-11-28: "keep colours a
/// little more muted ... still let players choose a spectrum of colours ... extra muted on the eye colour").
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tone {
    Hair,
    Eyes,
    Lips,
}

impl Tone {
    /// (most saturation a colour keeps, soft cap on its brightest channel, where that cap starts to bend)
    fn range(self) -> (f32, f32, f32) {
        match self {
            Tone::Hair => (0.40, 0.72, 0.50),
            Tone::Lips => (0.34, 0.66, 0.46),
            Tone::Eyes => (0.22, 0.56, 0.40),
        }
    }
}

/// A picked colour eased into its kind's range: every hue stays choosable, but saturation rolls off smoothly toward
/// the kind's ceiling (so a pure red reads as a deep natural red, a pure blue eye as a grey-blue) and the brightest
/// channel is soft-capped so strands, highlights and the eye's shine still have room above it.
pub fn tone(c: [f32; 3], kind: Tone) -> [f32; 3] {
    let (smax, cap, knee) = kind.range();
    let lum = luma(c);
    let (mx, mn) = (c[0].max(c[1]).max(c[2]), c[0].min(c[1]).min(c[2]));
    let sat = if mx > 1e-4 { (mx - mn) / mx } else { 0.0 };
    let mut c = c;
    if sat > 1e-4 {
        let to = smax * (sat / smax).tanh();
        let k = to / sat;
        c = c.map(|v| (lum + (v - lum) * k).max(0.0));
    }
    if kind == Tone::Eyes {
        // irises are never near-black or near-white: keep their brightness in a believable band
        let l = luma(c).max(1e-4);
        let want = l.clamp(0.10, 0.50);
        c = c.map(|v| v * want / l);
    }
    let mx = c[0].max(c[1]).max(c[2]);
    if mx > knee {
        let to = knee + (cap - knee) * ((mx - knee) / (cap - knee)).tanh();
        c = c.map(|v| v * to / mx);
    }
    c
}

/// The same easing on an 8-bit colour, for the creator's swatches and sliders (what the player sees there is what
/// the character gets).
pub fn tone_u8(c: [u8; 3], kind: Tone) -> [u8; 3] {
    let t = tone(c.map(|v| v as f32 / 255.0), kind);
    t.map(|v| (v * 255.0).round().clamp(0.0, 255.0) as u8)
}

macro_rules! style_layers {
    ($n:literal) => {
        Layers {
            base: png(include_bytes!(concat!("../../../assets/timeless/looks/mithra", $n, "_base.png"))),
            m1: png(include_bytes!(concat!("../../../assets/timeless/looks/mithra", $n, "_m1.png"))),
            m2: png(include_bytes!(concat!("../../../assets/timeless/looks/mithra", $n, "_m2.png"))),
            m3: png(include_bytes!(concat!("../../../assets/timeless/looks/mithra", $n, "_m3.png"))),
            m4: png(include_bytes!(concat!("../../../assets/timeless/looks/mithra", $n, "_m4.png"))),
            blush: png_grey(include_bytes!(concat!("../../../assets/timeless/looks/blush/mithra", $n, "_blush.png"))),
        }
    };
}

/// A style's pictures (1..8), decoded when first used.
fn layers(style: u8) -> &'static Layers {
    static L: [OnceLock<Layers>; 8] = [const { OnceLock::new() }; 8];
    let k = style.clamp(1, 8);
    L[k as usize - 1].get_or_init(|| match k {
        1 => style_layers!("1"),
        2 => style_layers!("2"),
        3 => style_layers!("3"),
        4 => style_layers!("4"),
        5 => style_layers!("5"),
        6 => style_layers!("6"),
        7 => style_layers!("7"),
        _ => style_layers!("8"),
    })
}

fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// A neutral grey `g` (0..1, middle tone 0.5) in colour `c` (0..1).
fn shade(g: f32, c: [f32; 3]) -> [f32; 3] {
    if g <= 0.5 {
        c.map(|v| v * g * 2.0)
    } else {
        let k = (g - 0.5) * 2.0 * 0.8;
        c.map(|v| v + (1.0 - v) * k)
    }
}

fn mix(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

fn unit(c: [u8; 3]) -> [f32; 3] {
    c.map(|v| v as f32 / 255.0)
}

/// The nose's grey for the slider: 0 grey .. 1 black.
fn nose_grey(n: f32) -> f32 {
    0.42 + (0.03 - 0.42) * n.clamp(0.0, 1.0)
}

/// A style's recoloured face texture (RGBA, SIZE x SIZE) for a side (LEFT or RIGHT: the markings on that side).
pub fn compose(style: u8, s: &Settings, side: u8) -> Vec<u8> {
    let l = layers(style);
    let skin_mid = SKIN_MID[style.clamp(1, 8) as usize - 1];
    let mut out = vec![0u8; l.base.len()];
    let rows = SIZE as usize;
    let row_bytes = rows * 4;
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get()).clamp(1, 8);
    let per = rows.div_ceil(threads) * row_bytes;
    std::thread::scope(|sc| {
        for (k, chunk) in out.chunks_mut(per).enumerate() {
            let at = k * per;
            sc.spawn(move || compose_into(l, s, side, skin_mid, at, chunk));
        }
    });
    out
}

fn compose_into(l: &Layers, s: &Settings, side: u8, skin_mid: f32, at: usize, out: &mut [u8]) {
    let on: Vec<bool> = (0..MARKS.len()).map(|k| s.mark(k) & side != 0).collect();
    let (hair, tip, eyes, lips, mark) =
        (tone(unit(s.hair), Tone::Hair), tone(unit(s.tip), Tone::Hair), tone(unit(s.eyes), Tone::Eyes), tone(unit(s.lips), Tone::Lips), unit(s.mark_colour));
    let ng = nose_grey(s.nose);
    let nose = [ng; 3];
    let (t0, t1) = (s.tip_start - s.tip_blend, s.tip_start + s.tip_blend.max(0.005));
    for (i, px) in out.chunks_exact_mut(4).enumerate() {
        let o = at + i * 4;
        let b = &l.base[o..o + 4];
        let (m1, m2, m3, m4) = (&l.m1[o..o + 4], &l.m2[o..o + 4], &l.m3[o..o + 4], &l.m4[o..o + 4]);
        let w = |v: u8| v as f32 / 255.0;
        let mut c = [w(b[0]), w(b[1]), w(b[2])];
        let g = 0.299 * c[0] + 0.587 * c[1] + 0.114 * c[2];
        if m1[0] > 0 {
            let col = if s.two_tone { mix(hair, tip, smoothstep(t0, t1, w(m1[1]))) } else { hair };
            c = mix(c, hair_shade(g, col), w(m1[0]));
        }
        if m1[2] > 0 {
            c = mix(c, shade(g, eyes), w(m1[2]));
        }
        if m1[3] > 0 {
            c = mix(c, shade(g, lips), w(m1[3]));
        }
        if s.blush > 0.0 {
            if let Some(bw) = l.blush.get(o).filter(|v| **v > 0) {
                c = blush(c, w(*bw) * s.blush.clamp(0.0, 1.0));
            }
        }
        // Nose: left as the model's own (owner 2026-11-23, every race) - no recolour.
        let _ = nose;
        let cover = [m2[1], m2[2], m2[3], m3[0], m3[1], m3[2], m3[3], m4[0], m4[1], m2[0], m4[2], m4[3]];
        let sk = g * 255.0 / skin_mid;
        for (on, v) in on.iter().zip(cover) {
            if *on && v > 0 {
                let mc = mark.map(|x| (x * sk).min(1.0));
                c = mix(c, mc, w(v) * s.mark_strength.clamp(0.0, 1.0));
            }
        }
        for k in 0..3 {
            px[k] = (c[k].clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
        }
        px[3] = b[3];
    }
}

fn path(name: &str) -> std::path::PathBuf {
    let safe: String = name.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-').collect();
    crate::config::config_path().with_file_name(format!("timeless-look-{safe}.json"))
}

impl Looks {
    pub fn save(&self) {
        if self.name.is_empty() {
            return;
        }
        if let Ok(s) = serde_json::to_string_pretty(&self.settings) {
            let _ = std::fs::write(path(&self.name), s);
        }
    }

    /// Back to our face's own look (or the defaults), keeping on / try-on.
    pub fn reset(&mut self) {
        let keep = self.settings.clone();
        self.settings = self
            .preview
            .or(self.face)
            .and_then(|(r, f)| seed_any(r, f, &keep))
            .unwrap_or(Settings { on: keep.on, try_on: keep.try_on, ..Settings::default() });
    }

    /// The settings changed (the window): saved a moment later; a try-on change reloads our look.
    pub fn changed(&mut self, before: &Settings) {
        if before.try_on != self.settings.try_on || before.on != self.settings.on {
            self.try_on_ver = self.try_on_ver.wrapping_add(1);
        }
        self.dirty_save = Some(std::time::Instant::now());
    }
}

/// `/look` (also `/appearance`): opens or closes the window; `/look on|off`, `/look reset`, `/look light [on|off]`.
pub fn command(w: &mut World, args: &[&str]) {
    match args.first().map(|a| a.to_ascii_lowercase()) {
        Some(a) if a == "on" || a == "off" => {
            let before = w.looks.settings.clone();
            w.looks.settings.on = a == "on";
            w.looks.changed(&before);
        }
        // `/look light on|off`: "Light both sides" (lighting compare without the window).
        Some(a) if a == "light" => {
            let on = match args.get(1).map(|x| x.to_ascii_lowercase()) {
                Some(x) if x == "on" => true,
                Some(x) if x == "off" => false,
                _ => !w.looks.light_both_sides,
            };
            set_light_both_sides(&mut w.looks, on);
        }
        Some(a) if a == "reset" => {
            let before = w.looks.settings.clone();
            w.looks.reset();
            w.looks.changed(&before);
        }
        _ => w.looks.open = !w.looks.open,
    }
}

/// Our look with the try-on: a Mithra face without a custom look (25-31) shows face 6A here.
pub fn try_on(l: &Looks, look: crate::dat::character::Look) -> crate::dat::character::Look {
    if let Some((race, face)) = l.preview {
        return crate::dat::character::Look { race, face: model_face(race, face), ..look };
    }
    if l.settings.on && l.settings.try_on && look.race == 7 && !supported(look.race, look.face) {
        crate::dat::character::Look { face: 10, ..look }
    } else if l.settings.on && supported(look.race, look.face) {
        crate::dat::character::Look { face: model_face(look.race, look.face), ..look }
    } else {
        look
    }
}

/// Show our model as another race / face (None: our own), starting from that face's own colours.
pub fn set_preview(l: &mut Looks, p: Option<(u8, u16)>) {
    if l.preview == p {
        return;
    }
    l.preview = p;
    l.try_on_ver = l.try_on_ver.wrapping_add(1);
    let before = l.settings.clone();
    if let Some((r, f)) = p.or(l.face)
        && let Some(s) = seed_any(r, f, &l.settings)
    {
        l.settings = s;
    }
    l.changed(&before);
}

/// The free camera's drag (from the game's look drag): turn and tilt; `raise` (middle button) moves it up and down.
pub fn cam_drag(w: &mut World, dx: f64, dy: f64, raise: bool) {
    let c = &mut w.looks.cam;
    if raise {
        c.height = (c.height - dy as f32 * 0.002).clamp(0.0, 1.2);
    } else {
        c.yaw -= dx as f32 * 0.006;
        c.pitch = (c.pitch + dy as f32 * 0.005).clamp(-1.3, 1.4);
    }
}

/// The free camera's wheel: nearer / farther.
pub fn cam_wheel(w: &mut World, d: f32) {
    let c = &mut w.looks.cam;
    c.dist = (c.dist * (1.0 - d * 0.1)).clamp(0.15, 12.0);
}

/// The free camera's view, None when it's off or we have no model.
pub fn free_camera(w: &World) -> Option<crate::render::Camera> {
    use glam::Vec3;
    let c = &w.looks.cam;
    if !c.on {
        return None;
    }
    let my_dir = w.net.me.lock().map(|m| m.dir).unwrap_or(0);
    let p = super::super::actor_place(w, w.my_id, my_dir)?;
    let a = w.actors.get(&w.my_id)?;
    // The head's top: attach point 12 as the dressing camera finds it (Y is down), else 1.8 x scale.
    let top = a
        .character
        .as_ref()
        .and_then(|ch| ch.skeleton.points.get(12).copied())
        .and_then(|(bone, off)| {
            let bones = if a.bones.is_empty() { &a.rest } else { &a.bones };
            bones.get(bone as usize).map(|b| -b.at(off).y * a.scale)
        })
        .filter(|t| *t > 0.1)
        .unwrap_or(1.8 * a.scale.max(0.3));
    let target = p.pos - Vec3::Y * top * c.height;
    let f = Vec3::new(p.forward.x, 0.0, p.forward.z).try_normalize().unwrap_or(Vec3::Z);
    let side = Vec3::new(-f.z, 0.0, f.x);
    let flat = f * c.yaw.cos() + side * c.yaw.sin();
    let dir = flat * c.pitch.cos() - Vec3::Y * c.pitch.sin();
    let dist = c.dist * (top / 1.8).clamp(0.4, 1.6);
    Some(crate::render::Camera { eye: target + dir * dist, target, up: Vec3::NEG_Y, fov_y: 40f32.to_radians() })
}

/// Per frame after posing: the settings loaded for our character, a changed look composed and uploaded (at most
/// ten times a second), and our model's face texture swapped once its mips are in.
pub fn apply(gpu: &crate::render::Gpu, w: &mut World, r: &crate::render::ZoneRenderer) {
    let l = &mut w.looks;
    if l.name != w.name && !w.name.is_empty() {
        l.name = w.name.clone();
        let saved: Option<Settings> =
            std::fs::read_to_string(path(&w.name)).ok().and_then(|s| serde_json::from_str(&s).ok());
        l.needs_seed = saved.is_none();
        l.settings = saved.unwrap_or_default();
        (l.made, l.pending, l.current, l.job) = (None, None, None, None);
        l.try_on_ver = l.try_on_ver.wrapping_add(1);
    }
    if let Some(t) = l.dirty_save
        && t.elapsed().as_secs_f32() > 1.0
    {
        l.dirty_save = None;
        l.save();
    }
    if !l.light_loaded {
        l.light_loaded = true;
        l.light_both_sides = !std::fs::read_to_string(light_path()).is_ok_and(|t| t.contains("false"));
    }
    l.face = w.look_base.get(&w.my_id).map(|lk| (lk.race, lk.face));
    l.shown = l.preview.or(l.face).map(|(r, f)| (r, if l.settings.on || l.preview.is_some() { model_face(r, f) } else { f }));
    let now = std::time::Instant::now();
    if l.cam.on && l.cam.spin
        && let Some(t) = l.cam.last
    {
        l.cam.yaw += now.duration_since(t).as_secs_f32().min(0.1) * 0.5;
    }
    l.cam.last = Some(now);
    if l.needs_seed
        && let Some((race, face)) = l.face
    {
        l.needs_seed = false;
        if supported(race, face) {
            l.reset();
        }
    }
    let Some(a) = w.actors.get_mut(&w.my_id) else { return };
    let Some(g) = a.gpu.as_mut() else { return };
    let (race, face) = l.shown.unwrap_or((0, 0));
    // Mithra: the style's own neutral face (by the texture's name); the other races: the model's face texture.
    let found = if race == 7 {
        FACE_TEXTURES.iter().find_map(|(n, st)| g.texture_slot(&[n]).map(|k| (k, *st, n.to_string())))
    } else {
        race_face(race, face).and_then(|f| {
            let name = a.character.as_ref()?.face_texture.clone()?;
            g.texture_slot(&[name.as_str()]).map(|k| (k, f.style, name))
        })
    };
    let Some((slot, style, tex_name)) = found else {
        l.fits = false;
        return;
    };
    (l.fits, l.style) = (true, style);
    if !l.settings.on {
        // Off: the model's own texture again (its GPU side taken anew from the model, next frame).
        if l.current.take().is_some() {
            a.gpu = None;
        }
        l.made = None;
        l.pending = None;
        l.job = None;
        return;
    }
    let want = ((race, face, tex_name.clone()), l.settings.clone());
    // A finished look: up to the GPU (its mips are made there; it shows once they are done).
    if let Some(rx) = &l.job {
        match rx.try_recv() {
            Ok(m) => {
                l.job = None;
                if m.prep.is_some() {
                    l.prep = m.prep;
                }
                let up = |rgba: Vec<u8>| {
                    r.actor_texture(gpu, &crate::dat::zone::Texture::new("timeless look".into(), 0, m.w, m.h, rgba))
                };
                let (left, seq) = up(m.left);
                let right = m.right.map(up);
                let seq = right.as_ref().map_or(seq, |r| r.1.max(seq));
                l.pending = Some((seq, left, right.map(|r| r.0), m.want));
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {}
            Err(std::sync::mpsc::TryRecvError::Disconnected) => l.job = None,
        }
    }
    // A new look wanted: one worker at a time, so a fast slider skips straight to its latest value.
    if l.job.is_none() && l.made.as_ref() != Some(&want) && l.pending.as_ref().is_none_or(|p| p.3 != want) {
        let one_sided = want.1.one_sided();
        let (tx, rx) = std::sync::mpsc::channel();
        // Mithra: on its own HD face like the other races (2026-11-24); the neutral base only without that data.
        if race == 7 && (race_face(7, face).is_none() || race_layers(7, style).is_none()) {
            let w2 = want.clone();
            std::thread::spawn(move || {
                let left = compose(style, &w2.1, LEFT);
                let right = one_sided.then(|| compose(style, &w2.1, RIGHT));
                let _ = tx.send(Made { want: w2, w: SIZE, h: SIZE, left, right, prep: None });
            });
        } else {
            let Some(ch) = a.character.as_ref() else { return };
            let Some(t) = ch.textures.iter().find(|t| t.name == tex_name) else { return };
            let (Some(f), Some(rl)) = (race_face(race, face), race_layers(race, style)) else { return };
            let (w0, h0) = (t.width, t.height);
            let key: PrepKey = (race, face, tex_name.clone(), w0, h0);
            let have = l.prep.as_ref().filter(|p| p.0 == key).map(|p| p.1.clone());
            // A face with sides of its own carries both sides in one texture.
            let right_too = one_sided && !f.sides;
            let tex = if have.is_none() { Some(t.rgba().to_vec()) } else { None };
            let w2 = want.clone();
            std::thread::spawn(move || {
                let (prep, fresh) = match have {
                    Some(p) => (p, None),
                    None => {
                        let p = std::sync::Arc::new(Prep::new(rl, f, tex.unwrap_or_default(), w0, h0));
                        (p.clone(), Some((key, p)))
                    }
                };
                let left = compose_prep(&prep, &w2.1, LEFT);
                let right = right_too.then(|| compose_prep(&prep, &w2.1, RIGHT));
                let _ = tx.send(Made { want: w2, w: w0, h: h0, left, right, prep: fresh });
            });
        }
        l.job = Some(rx);
    }
    if let Some((seq, ..)) = &l.pending
        && gpu.mips_done() >= *seq
    {
        let (_, left, right, s) = l.pending.take().unwrap();
        l.current = Some((left, right));
        l.made = Some(s);
    }
    if let Some((left, right)) = &l.current {
        g.set_texture_sides(slot, left, right.as_ref().unwrap_or(left));
    }
}

/// Timeless XI: a custom look on one actor outside the world (the Create Character screen's model,
/// client/timeless/ring.rs): the same making as `apply` (a worker thread, then the GPU, shown once its mips are done),
/// for a given model, race, face and settings.
#[derive(Default)]
pub struct ActorLook {
    made: Option<(MadeKey, Settings)>,
    pending: Option<(u64, wgpu::BindGroup, Option<wgpu::BindGroup>, (MadeKey, Settings))>,
    current: Option<(wgpu::BindGroup, Option<wgpu::BindGroup>)>,
    job: Option<std::sync::mpsc::Receiver<Made>>,
    prep: Option<(PrepKey, std::sync::Arc<Prep>)>,
}

impl ActorLook {
    /// The look is on the model (made and uploaded).
    pub fn shown(&self) -> bool {
        self.current.is_some() && self.job.is_none() && self.pending.is_none()
    }

    /// A new model on the actor: what was made for the old one goes.
    pub fn forget(&mut self) {
        (self.made, self.pending, self.current, self.job) = (None, None, None, None);
    }

    /// The look `s` on `g` (the model `ch`, race 1..8, face as drawn); off or not supported: the model's own face.
    pub fn update(
        &mut self,
        gpu: &crate::render::Gpu,
        r: &crate::render::ZoneRenderer,
        g: &mut crate::render::ActorGpu,
        ch: &crate::dat::character::Character,
        race: u8,
        face: u16,
        s: &Settings,
    ) {
        if !s.on {
            self.forget();
            return;
        }
        let found = if race == 7 {
            FACE_TEXTURES.iter().find_map(|(n, st)| g.texture_slot(&[n]).map(|k| (k, *st, n.to_string())))
        } else {
            race_face(race, face).and_then(|f| {
                let name = ch.face_texture.clone()?;
                g.texture_slot(&[name.as_str()]).map(|k| (k, f.style, name))
            })
        };
        let Some((slot, style, tex_name)) = found else { return };
        let want = ((race, face, tex_name.clone()), s.clone());
        if let Some(rx) = &self.job {
            match rx.try_recv() {
                Ok(m) => {
                    self.job = None;
                    if m.prep.is_some() {
                        self.prep = m.prep;
                    }
                    let up = |rgba: Vec<u8>| {
                        r.actor_texture(gpu, &crate::dat::zone::Texture::new("timeless look".into(), 0, m.w, m.h, rgba))
                    };
                    let (left, seq) = up(m.left);
                    let right = m.right.map(up);
                    let seq = right.as_ref().map_or(seq, |r| r.1.max(seq));
                    self.pending = Some((seq, left, right.map(|r| r.0), m.want));
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
                Err(std::sync::mpsc::TryRecvError::Disconnected) => self.job = None,
            }
        }
        if self.job.is_none() && self.made.as_ref() != Some(&want) && self.pending.as_ref().is_none_or(|p| p.3 != want) {
            let one_sided = want.1.one_sided();
            let (tx, rx) = std::sync::mpsc::channel();
            if race == 7 && (race_face(7, face).is_none() || race_layers(7, style).is_none()) {
                let w2 = want.clone();
                std::thread::spawn(move || {
                    let left = compose(style, &w2.1, LEFT);
                    let right = one_sided.then(|| compose(style, &w2.1, RIGHT));
                    let _ = tx.send(Made { want: w2, w: SIZE, h: SIZE, left, right, prep: None });
                });
            } else {
                let Some(t) = ch.textures.iter().find(|t| t.name == tex_name) else { return };
                let (Some(f), Some(rl)) = (race_face(race, face), race_layers(race, style)) else { return };
                let (w0, h0) = (t.width, t.height);
                let key: PrepKey = (race, face, tex_name.clone(), w0, h0);
                let have = self.prep.as_ref().filter(|p| p.0 == key).map(|p| p.1.clone());
                let right_too = one_sided && !f.sides;
                let tex = if have.is_none() { Some(t.rgba().to_vec()) } else { None };
                let w2 = want.clone();
                std::thread::spawn(move || {
                    let (prep, fresh) = match have {
                        Some(p) => (p, None),
                        None => {
                            let p = std::sync::Arc::new(Prep::new(rl, f, tex.unwrap_or_default(), w0, h0));
                            (p.clone(), Some((key, p)))
                        }
                    };
                    let left = compose_prep(&prep, &w2.1, LEFT);
                    let right = right_too.then(|| compose_prep(&prep, &w2.1, RIGHT));
                    let _ = tx.send(Made { want: w2, w: w0, h: h0, left, right, prep: fresh });
                });
            }
            self.job = Some(rx);
        }
        if let Some((seq, ..)) = &self.pending
            && gpu.mips_done() >= *seq
        {
            let (_, left, right, made) = self.pending.take().unwrap();
            self.current = Some((left, right));
            self.made = Some(made);
        }
        if let Some((left, right)) = &self.current {
            g.set_texture_sides(slot, left, right.as_ref().unwrap_or(left));
        }
    }
}

/// The settings a new character's look starts from: its face's own colours, on (None: no look for this face).
pub fn start_for(race: u8, face: u16) -> Option<Settings> {
    let mut s = seed_any(race, face, &Settings::default())?;
    s.on = true;
    Some(s)
}

/// Saves `s` as the look of the character `name` (the Create Character screen, once it is made).
pub fn save_for(name: &str, s: &Settings) {
    if let Ok(t) = serde_json::to_string_pretty(s) {
        let _ = std::fs::write(path(name), t);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layers_decode_at_full_size() {
        for style in 1..=8u8 {
            let l = layers(style);
            for v in [&l.base, &l.m1, &l.m2, &l.m3, &l.m4] {
                assert_eq!(v.len(), (SIZE * SIZE * 4) as usize, "style {style}");
            }
            // The masks are not empty: hair, iris, lips, nose and every marking cover some texels.
            let any = |v: &Vec<u8>, c: usize| v.chunks_exact(4).filter(|p| p[c] > 128).count();
            for c in 0..4 {
                assert!(any(&l.m1, c) > 100, "style {style} m1 channel {c}");
            }
            for c in 0..4 {
                assert!(any(&l.m2, c) > 50, "style {style} m2 channel {c}");
            }
            for c in 0..4 {
                assert!(any(&l.m3, c) > 20, "style {style} m3 channel {c}");
            }
            for c in 0..2 {
                assert!(any(&l.m4, c) > 20, "style {style} m4 channel {c}");
            }
        }
    }

    #[test]
    fn every_face_has_a_look() {
        for face in 0..=24u16 {
            let s = seed(face, &Settings::default()).expect("seed");
            assert_eq!(s.marks.len(), MARKS.len());
            assert!(!s.on);
        }
        assert!(seed(25, &Settings::default()).is_none());
        // Every face's HD texture name is known, styles 1..8.
        for face in 0..16 {
            let n = format!("tim     tl_h{face:02}hd");
            assert_eq!(FACE_TEXTURES.iter().find(|t| t.0 == n).map(|t| t.1), Some(face / 2 + 1));
        }
    }

    #[test]
    fn hair_follows_the_colour() {
        for style in [1u8, 6] {
            let l = layers(style);
            // A texel well inside the hair.
            let i = l.m1.chunks_exact(4).position(|p| p[0] == 255 && p[1] > 200).expect("hair") * 4;
            let red = compose(style, &Settings { hair: [200, 30, 30], ..Settings::default() }, LEFT);
            let blue = compose(style, &Settings { hair: [30, 30, 200], ..Settings::default() }, LEFT);
            assert!(red[i] > red[i + 2] && blue[i + 2] > blue[i]);
            // Markings off: the skin under the crest is the base's.
            let off = compose(style, &Settings { marks: vec![0; MARKS.len()], ..Settings::default() }, LEFT);
            let j = l.m2.chunks_exact(4).position(|p| p[1] == 255).expect("crest") * 4;
            assert_eq!(&off[j..j + 3], &l.base[j..j + 3]);
            // A constellation under the left eye only: on the left texture, not the right.
            let s = Settings { marks: vec![0, 0, 0, 0, 0, 0, 0, 0, LEFT], ..Settings::default() };
            assert!(s.one_sided());
            let k = l.m4.chunks_exact(4).position(|p| p[1] == 255).expect("star") * 4;
            let (lt, rt) = (compose(style, &s, LEFT), compose(style, &s, RIGHT));
            assert_ne!(&lt[k..k + 3], &l.base[k..k + 3]);
            assert_eq!(&rt[k..k + 3], &l.base[k..k + 3]);
        }
        // A both-sides-only marking is never one-sided; the first proof's ticks still load.
        assert_eq!(Settings { marks: vec![LEFT], ..Settings::default() }.mark(0), BOTH);
        assert_eq!(Settings { marks: vec![0, RIGHT], ..Settings::default() }.mark(1), RIGHT);
        let old: Settings = serde_json::from_str(r#"{"marks":[true,false,true,false,false,false]}"#).unwrap();
        assert_eq!(old.marks, vec![BOTH, 0, BOTH, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn every_race_face_has_masks_and_colours() {
        for (race, _) in RACES {
            if race == 7 {
                continue;
            }
            for face in 0..16u16 {
                let f = race_face(race, face).expect("races.json");
                let rl = race_layers(race, f.style).expect("masks");
                assert!(rl.m.len() >= 4, "race {race} style {}", f.style);
                assert_eq!(rl.m.len() == 6, f.sides, "race {race} style {}", f.style);
                for m in &rl.m {
                    assert!([512usize, 1024].iter().any(|n| m.len() == n * n * 4));
                }
                // Hair covers some of every face.
                assert!(rl.m[0].chunks_exact(4).filter(|p| p[0] > 128).count() > 500, "race {race} face {face}");
                assert!(seed_any(race, face, &Settings::default()).is_some());
            }
        }
        // Elvaan F's face isn't mirrored: its markings have right sides of their own.
        assert!(race_face(4, 0).unwrap().sides && !race_face(1, 0).unwrap().sides);
    }

    #[test]
    fn race_markings_show() {
        // Every race's own markings (owner 2026-11-23): each one changes the face, freckles darken the skin.
        for race in [1u8, 2, 3, 4, 5, 6, 8] {
            let list = race_marks(race);
            assert!(!list.is_empty() && list.len() <= 9);
            assert!(layers_of(race, 0).marks);
            let mut shown = vec![0; list.len()];
            for style in 1..=8u8 {
                let face = (style as u16 - 1) * 2;
                let f = race_face(race, face).unwrap();
                let rl = race_layers(race, f.style).unwrap();
                let n = 256u32;
                let tex = vec![150u8; (n * n * 4) as usize];
                let prep = Prep::new(rl, f, tex, n, n);
                let off = compose_prep(&prep, &Settings { marks: vec![0; MARKS.len()], ..Settings::default() }, BOTH);
                for (k, (name, kind)) in list.iter().enumerate() {
                    let mut marks = vec![0; MARKS.len()];
                    marks[k] = BOTH;
                    let on = compose_prep(&prep, &Settings { marks, ..Settings::default() }, BOTH);
                    let diff: Vec<usize> = (0..(n * n) as usize).filter(|i| on[i * 4..i * 4 + 3] != off[i * 4..i * 4 + 3]).collect();
                    // A marking can be under a style's bangs (markings stay behind hair): counted below.
                    if diff.len() > 3 {
                        shown[k] += 1;
                    }
                    if *kind == Kind::Freckle && !diff.is_empty() {
                        let i = diff[diff.len() / 2];
                        assert!(on[i * 4 + 2] < off[i * 4 + 2], "race {race} style {style} freckles darken");
                    }
                }
            }
            for (k, n) in shown.iter().enumerate() {
                assert!(*n >= 5, "race {race} {}: shows on {n} of 8 styles", list[k].0);
            }
        }
        assert_eq!(mark_names(7).len(), 12);
    }

    #[test]
    fn race_hair_follows_the_colour() {
        let f = race_face(1, 0).unwrap();
        let rl = race_layers(1, f.style).unwrap();
        // A grey 256 x 256 texture: hair texels take the colour, others stay grey.
        let n = 256u32;
        let tex = vec![120u8; (n * n * 4) as usize];
        let red = compose_race(rl, f, &tex, n, n, &Settings { hair: [200, 30, 30], ..Settings::default() }, LEFT);
        let m = &rl.m[0];
        let k = (0..(n * n) as usize)
            .find(|i| {
                let (x, y) = (i % n as usize, i / n as usize);
                sample(m, (x as f32 + 0.5) / n as f32, (y as f32 + 0.5) / n as f32)[0] > 0.99
            })
            .expect("hair");
        assert!(red[k * 4] > red[k * 4 + 2] + 40, "{:?}", &red[k * 4..k * 4 + 4]);
        let j = (0..(n * n) as usize)
            .find(|i| {
                let (x, y) = (i % n as usize, i / n as usize);
                let (u, v) = ((x as f32 + 0.5) / n as f32, (y as f32 + 0.5) / n as f32);
                sample(&rl.m[0], u, v)[0] == 0.0 && sample(&rl.m[0], u, v)[2] == 0.0 && sample(&rl.m[0], u, v)[3] == 0.0
            })
            .unwrap();
        assert_eq!(&red[j * 4..j * 4 + 3], &[120, 120, 120]);
    }

    /// `cargo test looks_dump -- --ignored`: the "Crimson ombre" preview's texture to /tmp (checked against the tool's).
    #[test]
    #[ignore]
    fn looks_dump() {
        let s = Settings {
            hair: [0x3a, 0x14, 0x14],
            two_tone: true,
            tip: [0xd0, 0x30, 0x2c],
            tip_start: 0.55,
            tip_blend: 0.25,
            eyes: [0xd8, 0xa0, 0x38],
            lips: [0xa8, 0x3a, 0x3a],
            nose: (0.42 - 0.08) / 0.39,
            marks: vec![0, 0, 0, BOTH, 0, 0, 0, 0, 0],
            mark_colour: [0x2a, 0x14, 0x10],
            mark_strength: 0.85,
            ..Settings::default()
        };
        let t = std::time::Instant::now();
        let v = compose(6, &s, LEFT);
        eprintln!("compose {:?}", t.elapsed());
        std::fs::write("/tmp/look_rust.rgba", v).unwrap();
    }

    /// `cargo test looks_bench -- --ignored --nocapture`: how long one look takes to make (the /look sliders).
    #[test]
    #[ignore]
    fn looks_bench() {
        let s = Settings { hair: [200, 30, 30], ..Settings::default() };
        let _ = compose(6, &s, LEFT);
        let t = std::time::Instant::now();
        for _ in 0..5 { let _ = compose(6, &s, LEFT); }
        eprintln!("mithra compose {:?}", t.elapsed() / 5);
        for (r, face, n) in [(1u8, 0u16, 1024u32), (8, 0, 2048)] {
            let f = race_face(r, face).unwrap();
            let rl = race_layers(r, f.style).unwrap();
            let tex = vec![120u8; (n * n * 4) as usize];
            let t = std::time::Instant::now();
            let prep = Prep::new(rl, f, tex.clone(), n, n);
            eprintln!("race {r} {n} prep {:?}", t.elapsed());
            let t = std::time::Instant::now();
            for _ in 0..3 { let _ = compose_prep(&prep, &s, LEFT); }
            eprintln!("race {r} {n} compose {:?}", t.elapsed() / 3);
            let t = std::time::Instant::now();
            let _ = hair_level(rl, &tex, n, n, mask_scale(rl, n, f.race));
            eprintln!("  hair_level {:?}", t.elapsed());
        }
    }
}
