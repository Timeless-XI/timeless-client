//! Timeless XI: custom looks for every Mithra face (styles 1-8: faces 0-15 and the Timeless colours 16-24). The player
//! picks hair colours (one or two tone), eye colour, face markings, lip tint and nose shade with sliders; the client
//! recolours the face texture itself and swaps it into our own model. Nothing is sent: only this client sees it for now.
//!
//! The pictures are `assets/timeless/looks/mithra<style>_*.png` (timeless-tools Custom_Faces/looks: the style's HD face
//! with neutral grey hair, iris, lips and nose and no markings, plus layer masks), and `faces.json` (each face's own
//! colours and nearest markings, the starting point and Reset):
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

/// The faces with a custom look (Mithra 0-24).
pub fn supported(race: u8, face: u16) -> bool {
    race == 7 && face <= 24
}

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
    /// The style and settings the texture on the GPU was made from; a newer one waits for its mips.
    made: Option<(u8, Settings)>,
    made_at: Option<std::time::Instant>,
    /// The new textures (left, right when one-sided) waiting for their mips.
    pending: Option<(u64, wgpu::BindGroup, Option<wgpu::BindGroup>, (u8, Settings))>,
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
}

struct Layers {
    base: Vec<u8>,
    m1: Vec<u8>,
    m2: Vec<u8>,
    m3: Vec<u8>,
    m4: Vec<u8>,
}

fn png(bytes: &[u8]) -> Vec<u8> {
    let dec = png::Decoder::new(std::io::Cursor::new(bytes));
    let Ok(mut r) = dec.read_info() else { return vec![0; (SIZE * SIZE * 4) as usize] };
    let mut buf = vec![0; r.output_buffer_size().unwrap_or(0)];
    match r.next_frame(&mut buf) {
        Ok(info) if info.width == SIZE && info.height == SIZE && info.color_type == png::ColorType::Rgba => {
            buf.truncate(info.buffer_size());
            buf
        }
        _ => vec![0; (SIZE * SIZE * 4) as usize],
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
            .face
            .and_then(|(_, f)| seed(f, &keep))
            .unwrap_or(Settings { on: keep.on, try_on: keep.try_on, ..Settings::default() });
    }

    /// The settings changed (the window): saved a moment later; a try-on change reloads our look.
    pub fn changed(&mut self, before: &Settings) {
        if before.try_on != self.settings.try_on {
            self.try_on_ver = self.try_on_ver.wrapping_add(1);
        }
        self.dirty_save = Some(std::time::Instant::now());
    }
}

/// `/look` (also `/appearance`): opens or closes the window; `/look on|off`, `/look reset`.
pub fn command(w: &mut World, args: &[&str]) {
    match args.first().map(|a| a.to_ascii_lowercase()) {
        Some(a) if a == "on" || a == "off" => {
            let before = w.looks.settings.clone();
            w.looks.settings.on = a == "on";
            w.looks.changed(&before);
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
    if l.settings.on && l.settings.try_on && look.race == 7 && !supported(look.race, look.face) {
        crate::dat::character::Look { face: 10, ..look }
    } else {
        look
    }
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
    l.face = w.look_base.get(&w.my_id).map(|lk| (lk.race, lk.face));
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
    let Some((slot, style)) = FACE_TEXTURES.iter().find_map(|(n, st)| g.texture_slot(&[n]).map(|k| (k, *st))) else {
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
    let want = (style, l.settings.clone());
    let busy = l.made_at.is_some_and(|t| t.elapsed().as_secs_f32() < 0.1);
    if l.made.as_ref() != Some(&want) && !busy && l.pending.as_ref().is_none_or(|p| p.3 != want) {
        let up = |side: u8| {
            let rgba = compose(style, &want.1, side);
            let t = crate::dat::zone::Texture::new("timeless look".into(), 0, SIZE, SIZE, rgba);
            r.actor_texture(gpu, &t)
        };
        let (left, seq) = up(LEFT);
        let right = want.1.one_sided().then(|| up(RIGHT));
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
