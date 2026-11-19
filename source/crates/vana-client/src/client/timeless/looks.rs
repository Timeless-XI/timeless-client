//! Timeless XI: custom looks (proof on Mithra face style 6). The player picks hair colours (one or two tone), eye
//! colour, face markings, lip tint and nose shade with sliders; the client recolours the face texture itself and swaps
//! it into our own model. Nothing is sent: only this client sees it for now.
//!
//! The pictures are `assets/timeless/looks/mithra6_*.png` (timeless-tools Custom_Faces/looks: the HD face with neutral
//! grey hair, iris, lips and nose and no markings, plus layer masks):
//! - base: RGBA, the neutral face texture (1024 x 1024);
//! - m1: hair weight, tip distance (0 crown .. 1 hair ends), iris, lips;
//! - m2: nose, markings forehead / cheeks / brow;
//! - m3: markings arc / slash / freckles, unused.
//!
//! A neutral layer's grey has its middle tone at 0.5: below it the colour is darkened (multiply), above it lightened
//! toward white (screen), so the drawn shading and strands stay.

use super::super::World;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

pub const SIZE: u32 = 1024;
/// The model textures the custom look replaces: Mithra style 6 (faces 6A, 6B, Timeless 22), HD and stock names.
pub const TEXTURES: [&str; 5] =
    ["tim     tl_h10hd", "tim     tl_h11hd", "tim     tl_m6b_1", "tim     mt_h71_1", "tim     mt_h71_2"];
/// The face block's median skin brightness in the base (0..255): markings follow the skin's shading against it.
const SKIN_MID: f32 = 94.5;
/// The markings, in mask order.
pub const MARKS: [&str; 6] = ["Forehead stripes", "Cheek stripes", "Brow & cheek line", "Eye arc", "Claw slashes", "Freckles"];

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
    pub marks: [bool; 6],
    pub mark_colour: [u8; 3],
    pub mark_strength: f32,
}

impl Default for Settings {
    /// Close to the game's face 6A: muted white hair, grey-blue eyes, the tiger stripes.
    fn default() -> Self {
        Settings {
            on: true,
            try_on: false,
            hair: [232, 226, 216],
            two_tone: false,
            tip: [214, 178, 118],
            tip_start: 0.6,
            tip_blend: 0.2,
            eyes: [111, 143, 168],
            lips: [154, 106, 90],
            nose: 0.6,
            marks: [true, true, false, false, false, false],
            mark_colour: [90, 52, 32],
            mark_strength: 0.85,
        }
    }
}

/// The world's custom-look state.
#[derive(Default)]
pub struct Looks {
    pub settings: Settings,
    /// The character the settings belong to (loaded on a name change).
    pub name: String,
    pub open: bool,
    /// The settings the texture on the GPU was made from; a newer one waits for its mips.
    made: Option<Settings>,
    made_at: Option<std::time::Instant>,
    pending: Option<(u64, wgpu::BindGroup, Settings)>,
    current: Option<wgpu::BindGroup>,
    /// Bumped when `try_on` changes: our look loads again.
    pub try_on_ver: u32,
    pub try_on_seen: u32,
    /// Our model has a face style 6 texture (the window says when it does not).
    pub fits: bool,
    dirty_save: Option<std::time::Instant>,
}

struct Layers {
    base: Vec<u8>,
    m1: Vec<u8>,
    m2: Vec<u8>,
    m3: Vec<u8>,
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

fn layers() -> &'static Layers {
    static L: OnceLock<Layers> = OnceLock::new();
    L.get_or_init(|| Layers {
        base: png(include_bytes!("../../../assets/timeless/looks/mithra6_base.png")),
        m1: png(include_bytes!("../../../assets/timeless/looks/mithra6_m1.png")),
        m2: png(include_bytes!("../../../assets/timeless/looks/mithra6_m2.png")),
        m3: png(include_bytes!("../../../assets/timeless/looks/mithra6_m3.png")),
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

/// The recoloured face texture (RGBA, SIZE x SIZE).
pub fn compose(s: &Settings) -> Vec<u8> {
    let l = layers();
    let mut out = vec![0u8; l.base.len()];
    let rows = SIZE as usize;
    let row_bytes = rows * 4;
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get()).clamp(1, 8);
    let per = rows.div_ceil(threads) * row_bytes;
    std::thread::scope(|sc| {
        for (k, chunk) in out.chunks_mut(per).enumerate() {
            let at = k * per;
            sc.spawn(move || compose_into(l, s, at, chunk));
        }
    });
    out
}

fn compose_into(l: &Layers, s: &Settings, at: usize, out: &mut [u8]) {
    let (hair, tip, eyes, lips, mark) = (unit(s.hair), unit(s.tip), unit(s.eyes), unit(s.lips), unit(s.mark_colour));
    let ng = nose_grey(s.nose);
    let nose = [ng; 3];
    let (t0, t1) = (s.tip_start - s.tip_blend, s.tip_start + s.tip_blend.max(0.005));
    for (i, px) in out.chunks_exact_mut(4).enumerate() {
        let o = at + i * 4;
        let b = &l.base[o..o + 4];
        let (m1, m2, m3) = (&l.m1[o..o + 4], &l.m2[o..o + 4], &l.m3[o..o + 4]);
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
        let cover = [m2[1], m2[2], m2[3], m3[0], m3[1], m3[2]];
        let sk = g * 255.0 / SKIN_MID;
        for (on, v) in s.marks.iter().zip(cover) {
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
            w.looks.settings = Settings { try_on: before.try_on, ..Settings::default() };
            w.looks.changed(&before);
        }
        _ => w.looks.open = !w.looks.open,
    }
}

/// Our look with the try-on: a Mithra of another face style shows face 6A here.
pub fn try_on(l: &Looks, look: crate::dat::character::Look) -> crate::dat::character::Look {
    let style6 = matches!(look.face, 10 | 11 | 22);
    if l.settings.on && l.settings.try_on && look.race == 7 && !style6 {
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
        l.settings = std::fs::read_to_string(path(&w.name))
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        (l.made, l.pending, l.current) = (None, None, None);
        l.try_on_ver = l.try_on_ver.wrapping_add(1);
    }
    if let Some(t) = l.dirty_save
        && t.elapsed().as_secs_f32() > 1.0
    {
        l.dirty_save = None;
        l.save();
    }
    let Some(a) = w.actors.get_mut(&w.my_id) else { return };
    let Some(g) = a.gpu.as_mut() else { return };
    let Some(slot) = g.texture_slot(&TEXTURES) else {
        l.fits = false;
        return;
    };
    l.fits = true;
    if !l.settings.on {
        // Off: the model's own texture again (its GPU side taken anew from the model, next frame).
        if l.current.take().is_some() {
            a.gpu = None;
        }
        l.made = None;
        l.pending = None;
        return;
    }
    let want = l.settings.clone();
    let busy = l.made_at.is_some_and(|t| t.elapsed().as_secs_f32() < 0.1);
    if l.made.as_ref() != Some(&want) && !busy && l.pending.as_ref().is_none_or(|p| p.2 != want) {
        let rgba = compose(&want);
        let t = crate::dat::zone::Texture::new("timeless look".into(), 0, SIZE, SIZE, rgba);
        let (bind, seq) = r.actor_texture(gpu, &t);
        l.pending = Some((seq, bind, want.clone()));
        l.made_at = Some(std::time::Instant::now());
    }
    if let Some((seq, ..)) = &l.pending
        && gpu.mips_done() >= *seq
    {
        let (_, bind, s) = l.pending.take().unwrap();
        l.current = Some(bind);
        l.made = Some(s);
    }
    if let Some(b) = &l.current {
        g.set_texture(slot, b);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layers_decode_at_full_size() {
        let l = layers();
        for v in [&l.base, &l.m1, &l.m2, &l.m3] {
            assert_eq!(v.len(), (SIZE * SIZE * 4) as usize);
        }
        // The masks are not empty: hair, iris, lips, nose and every marking cover some texels.
        let any = |v: &Vec<u8>, c: usize| v.chunks_exact(4).filter(|p| p[c] > 128).count();
        for c in 0..4 {
            assert!(any(&l.m1, c) > 100, "m1 channel {c}");
        }
        for c in 0..4 {
            assert!(any(&l.m2, c) > 50, "m2 channel {c}");
        }
        for c in 0..3 {
            assert!(any(&l.m3, c) > 20, "m3 channel {c}");
        }
    }

    #[test]
    fn hair_follows_the_colour() {
        let l = layers();
        // A texel well inside the hair (the long side locks).
        let i = l.m1.chunks_exact(4).position(|p| p[0] == 255 && p[1] > 200).expect("hair") * 4;
        let red = compose(&Settings { hair: [200, 30, 30], ..Settings::default() });
        let blue = compose(&Settings { hair: [30, 30, 200], ..Settings::default() });
        assert!(red[i] > red[i + 2] && blue[i + 2] > blue[i]);
        // Markings off: the skin under the forehead stripes is the base's.
        let off = compose(&Settings { marks: [false; 6], ..Settings::default() });
        let j = l.m2.chunks_exact(4).position(|p| p[1] == 255).expect("stripe") * 4;
        assert_eq!(&off[j..j + 3], &l.base[j..j + 3]);
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
            marks: [false, false, false, true, false, false],
            mark_colour: [0x2a, 0x14, 0x10],
            mark_strength: 0.85,
            ..Settings::default()
        };
        let t = std::time::Instant::now();
        let v = compose(&s);
        eprintln!("compose {:?}", t.elapsed());
        std::fs::write("/tmp/look_rust.rgba", v).unwrap();
    }
}
