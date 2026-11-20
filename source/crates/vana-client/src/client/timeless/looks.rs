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
/// The markings, in mask order.
pub const MARKS: [&str; 9] = [
    "Swept crest",
    "Cheek stripes",
    "Brow & cheek line",
    "Eye arc",
    "Claw slashes",
    "Freckles",
    "Crescent & tears",
    "War chevrons",
    "Constellation",
];
/// The markings that can go on one side only (under one eye).
pub const SIDED: [bool; 9] = [false, true, true, true, true, false, true, true, true];
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
}

fn race_faces() -> &'static HashMap<String, RaceFace> {
    static S: OnceLock<HashMap<String, RaceFace>> = OnceLock::new();
    S.get_or_init(|| {
        serde_json::from_str(include_str!("../../../assets/timeless/looks/races/races.json")).unwrap_or_default()
    })
}

pub fn race_face(race: u8, face: u16) -> Option<&'static RaceFace> {
    race_faces().get(&format!("{race}_{face}"))
}

/// The layers a face has (the window hides the others).
pub fn layers_of(race: u8, face: u16) -> Has {
    match race_face(race, face) {
        Some(f) if race != 7 => f.has,
        _ => Has { iris: true, lips: true, nose: true },
    }
}

/// Any race's face as settings: its own colours (Mithra: and nearest markings); the other settings as given.
pub fn seed_any(race: u8, face: u16, keep: &Settings) -> Option<Settings> {
    if race == 7 {
        return seed(face, keep);
    }
    let f = race_face(race, face)?;
    Some(Settings {
        hair: f.hair,
        two_tone: f.two_tone,
        tip: f.tip,
        eyes: f.eyes,
        lips: f.lips,
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
    made_at: Option<std::time::Instant>,
    /// The new textures (left, right when one-sided) waiting for their mips.
    pending: Option<(u64, wgpu::BindGroup, Option<wgpu::BindGroup>, (MadeKey, Settings))>,
    current: Option<(wgpu::BindGroup, Option<wgpu::BindGroup>)>,
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

struct Layers {
    base: Vec<u8>,
    m1: Vec<u8>,
    m2: Vec<u8>,
    m3: Vec<u8>,
    m4: Vec<u8>,
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
}

pub fn race_layers(race: u8, style: u8) -> Option<&'static RaceLayers> {
    static L: OnceLock<HashMap<(u8, u8), OnceLock<RaceLayers>>> = OnceLock::new();
    let all = L.get_or_init(|| {
        super::look_files::RACE_MASKS.iter().map(|(r, s, _)| ((*r, *s), OnceLock::new())).collect()
    });
    let cell = all.get(&(race, style))?;
    Some(cell.get_or_init(|| {
        let files = super::look_files::RACE_MASKS.iter().find(|(r, s, _)| (*r, *s) == (race, style)).map(|x| x.2);
        RaceLayers { m: files.unwrap_or(&[]).iter().map(|b| png_square(b)).collect() }
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
pub fn compose_race(rl: &RaceLayers, f: &RaceFace, tex: &[u8], w: u32, h: u32, s: &Settings, side: u8) -> Vec<u8> {
    let mut out = tex.to_vec();
    let k = mask_scale(rl, w);
    let level = hair_level(rl, tex, w, h, k);
    let level = &level;
    let row_bytes = w as usize * 4;
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get()).clamp(1, 8);
    let per = (h as usize).div_ceil(threads) * row_bytes;
    std::thread::scope(|sc| {
        for (k, chunk) in out.chunks_mut(per).enumerate() {
            let y0 = k * per / row_bytes;
            sc.spawn(move || compose_race_rows(rl, f, chunk, w, h, y0, s, side, level));
        }
    });
    out
}

#[allow(clippy::too_many_arguments)]
/// The hair's local brightness (LEVEL x LEVEL cells over the texture, hair-weighted mean luma; cells without hair take
/// their neighbours'), so near-black parts of a mane (Galka 1A's back) recolour as hair, not as black.
const LEVEL: usize = 32;
/// A 1024 mask set is laid out for a remapped 2048 face (the old layout in its top-left quarter); on a face that isn't
/// remapped (the stock 128 face, an older HD one) the masks are read from that quarter: (u, v) x 0.5.
fn mask_scale(rl: &RaceLayers, w: u32) -> f32 {
    let big = rl.m.first().is_some_and(|m| m.len() == 1024 * 1024 * 4);
    if big && w < 2048 { 0.5 } else { 1.0 }
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

#[allow(clippy::too_many_arguments)]
fn compose_race_rows(rl: &RaceLayers, f: &RaceFace, out: &mut [u8], w: u32, h: u32, y0: usize, s: &Settings, side: u8, level: &[f32]) {
    let none = vec![0u8; 0];
    let mk = |k: usize| rl.m.get(k).unwrap_or(&none);
    let (m1, m2, m3, m4) = (mk(0), mk(1), mk(2), mk(3));
    if m1.is_empty() {
        return;
    }
    let (m5, m6) = (mk(4), mk(5));
    let right = f.sides && !m5.is_empty();
    // Which markings show on this texture, from the left masks and (sides of their own) the right masks.
    let left_on: Vec<bool> =
        (0..MARKS.len()).map(|k| if right { s.mark(k) & LEFT != 0 } else { s.mark(k) & side != 0 }).collect();
    let right_on: Vec<bool> = (0..MARKS.len()).map(|k| right && SIDED[k] && s.mark(k) & RIGHT != 0).collect();
    let (hair, tip, eyes, lips, mark) = (unit(s.hair), unit(s.tip), unit(s.eyes), unit(s.lips), unit(s.mark_colour));
    let nose = [nose_grey(s.nose); 3];
    let (t0, t1) = (s.tip_start - s.tip_blend, s.tip_start + s.tip_blend.max(0.005));
    // Floors (hair and skin 20, iris, lips and nose 40, as the tools' preview): a layer nearly black on the face
    // (Galka lips) would blow every texel of it to white.
    let fl = [20.0, 40.0, 40.0, 40.0, 20.0];
    let [mh, mi, ml, mn, msk] = std::array::from_fn(|k| f.med[k].max(fl[k]) / 255.0);
    let strength = s.mark_strength.clamp(0.0, 1.0);
    let ks = mask_scale(rl, w);
    for (i, px) in out.chunks_exact_mut(4).enumerate() {
        let (x, y) = (i % w as usize, y0 + i / w as usize);
        let (u, v) = ((x as f32 + 0.5) / w as f32, (y as f32 + 0.5) / h as f32);
        let (mu, mv) = (u * ks, v * ks);
        let a = sample(m1, mu, mv);
        let b = sample(m2, mu, mv);
        let c3 = sample(m3, mu, mv);
        let d = sample(m4, mu, mv);
        let hw = |n: f32| n.clamp(0.0, 1.0);
        let mut c = [px[0] as f32 / 255.0, px[1] as f32 / 255.0, px[2] as f32 / 255.0];
        let l = 0.299 * c[0] + 0.587 * c[1] + 0.114 * c[2];
        if a[0] > 0.0 {
            // The HD strands' contrast halved (their dark gaps went black, their highlights white, whatever the
            // colour), and the highlights a lighter shade of the colour, not white.
            let g = l * 0.5 / hair_med(mh, level_at(level, u, v));
            let g = hw(0.5 + (g - 0.5) * if g > 0.5 { 0.3 } else { 0.55 });
            let col = if s.two_tone { mix(hair, tip, smoothstep(t0, t1, a[1])) } else { hair };
            let hc = if g <= 0.5 { col.map(|v| v * g * 2.0) } else { col.map(|v| v + (1.0 - v) * (g - 0.5) * 2.0 * 0.35) };
            c = mix(c, hc, a[0]);
        }
        // Iris, lips, nose: their highlights eased (a dark layer's median would blow them to white).
        let knee = |g: f32| if g > 0.5 { 0.5 + (g - 0.5) * 0.4 } else { g };
        if a[2] > 0.0 {
            c = mix(c, shade(hw(knee(l * 0.5 / mi)), eyes), a[2]);
        }
        if a[3] > 0.0 {
            c = mix(c, shade(hw(knee(l * 0.5 / ml)), lips), a[3]);
        }
        if b[0] > 0.0 {
            c = mix(c, shade(hw(knee(l * 0.5 / mn)), nose), b[0]);
        }
        let cover = [b[1], b[2], b[3], c3[0], c3[1], c3[2], c3[3], d[0], d[1]];
        let sk = l / msk;
        let mc = mark.map(|x| (x * sk).min(1.0));
        for (k, v) in cover.iter().enumerate() {
            if left_on[k] && *v > 0.0 {
                c = mix(c, mc, v * strength);
            }
        }
        if right {
            let (e, g6) = (sample(m5, mu, mv), sample(m6, mu, mv));
            // Right-side masks, MARKS order: cheeks, brow, arc, slash, crescent, chevrons, constellation.
            for (k, v) in [(1, e[0]), (2, e[1]), (3, e[2]), (4, e[3]), (6, g6[0]), (7, g6[1]), (8, g6[2])] {
                if right_on[k] && v > 0.0 && !m6.is_empty() {
                    c = mix(c, mc, v * strength);
                }
            }
        }
        for k in 0..3 {
            px[k] = (c[k].clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
        }
    }
}

macro_rules! style_layers {
    ($n:literal) => {
        Layers {
            base: png(include_bytes!(concat!("../../../assets/timeless/looks/mithra", $n, "_base.png"))),
            m1: png(include_bytes!(concat!("../../../assets/timeless/looks/mithra", $n, "_m1.png"))),
            m2: png(include_bytes!(concat!("../../../assets/timeless/looks/mithra", $n, "_m2.png"))),
            m3: png(include_bytes!(concat!("../../../assets/timeless/looks/mithra", $n, "_m3.png"))),
            m4: png(include_bytes!(concat!("../../../assets/timeless/looks/mithra", $n, "_m4.png"))),
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
    let (hair, tip, eyes, lips, mark) = (unit(s.hair), unit(s.tip), unit(s.eyes), unit(s.lips), unit(s.mark_colour));
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
            c = mix(c, shade(g, col), w(m1[0]));
        }
        if m1[2] > 0 {
            c = mix(c, shade(g, eyes), w(m1[2]));
        }
        if m1[3] > 0 {
            c = mix(c, shade(g, lips), w(m1[3]));
        }
        if m2[0] > 0 {
            c = mix(c, shade(g, nose), w(m2[0]));
        }
        let cover = [m2[1], m2[2], m2[3], m3[0], m3[1], m3[2], m3[3], m4[0], m4[1]];
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
        (l.made, l.pending, l.current) = (None, None, None);
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
        return;
    }
    let want = ((race, face, tex_name.clone()), l.settings.clone());
    let busy = l.made_at.is_some_and(|t| t.elapsed().as_secs_f32() < 0.1);
    if l.made.as_ref() != Some(&want) && !busy && l.pending.as_ref().is_none_or(|p| p.3 != want) {
        let base = if race == 7 {
            None
        } else {
            let Some(ch) = a.character.as_ref() else { return };
            let Some(t) = ch.textures.iter().find(|t| t.name == tex_name) else { return };
            let (Some(f), Some(rl)) = (race_face(race, face), race_layers(race, style)) else { return };
            Some((t, f, rl))
        };
        let up = |side: u8| {
            let t = match &base {
                None => {
                    crate::dat::zone::Texture::new("timeless look".into(), 0, SIZE, SIZE, compose(style, &want.1, side))
                }
                Some((t, f, rl)) => {
                    let rgba = compose_race(rl, f, t.rgba(), t.width, t.height, &want.1, side);
                    crate::dat::zone::Texture::new("timeless look".into(), 0, t.width, t.height, rgba)
                }
            };
            r.actor_texture(gpu, &t)
        };
        let (left, seq) = up(LEFT);
        // A face with sides of its own carries both sides in one texture.
        let sided = base.as_ref().is_some_and(|b| b.1.sides);
        let right = (want.1.one_sided() && !sided).then(|| up(RIGHT));
        let seq = right.as_ref().map_or(seq, |r| r.1.max(seq));
        l.pending = Some((seq, left, right.map(|r| r.0), want.clone()));
        l.made_at = Some(std::time::Instant::now());
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
        assert_eq!(old.marks, vec![BOTH, 0, BOTH, 0, 0, 0, 0, 0, 0]);
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
}
