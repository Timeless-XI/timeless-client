//! Timeless XI: the Timeless Aquarium (item 75), the Elder fish furniture.
//!
//! The server keeps which Elder fish a player has caught in the item's exdata byte 2 (bit n = Elder fish n,
//! timeless-server modules/timeless_aquarium); this draws the tank with exactly those fish in it, like a flowerpot
//! draws its plant. The tank, the eight fish and their baked textures come from one bundle,
//! `assets/timeless/aquarium/aquarium.bin` (built by the Elder fish tools, `export_aquarium.py`):
//!
//! - `b"TLAQ"`, u32 version 3, u32 entries; entry = u8 kind (0 mesh, 1 PNG, 2 raw), u16 name length, name, u32
//!   length, bytes.
//! - Mesh = one GROUP, drawn as its own piece so it can move on its own: u8 animation kind, u16 parameter count,
//!   f32 parameters, u32 models; model = u16 texture-name length, texture name, u16 blending (MMB bits: 0x8000
//!   blended, 0x2000 two-sided), u8 variant, u32 vertices, u32 indices, vertices (pos f32x3, normal f32x3, colour
//!   u32 BGRA with 0x80 = 1.0, uv f32x2), indices u16 (a triangle list).
//! - Animation (`Anim`): 0 none; 1 spin (pivot xyz, axis xyz, period s); 2 bob (pivot xyz, amplitude, period s,
//!   phase, yaw degrees); 3 fall (direction xyz, spacing, period s): a looped slide by one spacing; 4 swim (a
//!   fish's body) and 5 swim with a beating tail (its tail fin): lap s, start (0-1), sway rad, beat Hz, tail joint
//!   x, wag rad, wag lag rad, sample count n, then n samples (x, y, z, yaw, pitch, roll) evenly spaced in time over
//!   one lap (the Elder fish tools' `swim.py`, which this must match); 6 slither (one segment of an eel): kind 4's
//!   parameters (sway and wag off), then dt (how many seconds behind it follows the path), wave amplitude, wave
//!   phase and wave slope, so the segments trail each other round the loop with a wave running down the body.
//! - Variant: 0 always; 1 lit (1+ fish) / 2 dark (none); crown and finials 3 off (none) / 4 dim (1-7) / 5 full
//!   (all 8).
//! - Groups: `tank` (the cabinet, night sky, sand, ruins, kelp, the hourglass), `crown` (the star crystal on the
//!   crest, spins), `hgcrystal` (the hourglass's star crystal, spins), `grains` (falling sand), `fish0`..`fish7`
//!   (each swims its own loop round the tank) and `fish0t`..`fish7t` (their tail fins); Rimecoil, the eel (bit 5),
//!   is `fish5s0`..`fish5s11` instead, its segments from tail to head, so the whole body slithers. The PNGs by texture name;
//!   `icon32` the item icon in the item DAT layout (dat/items.rs `icon_rgba`).
//!
//! Model space is furniture space: origin at the middle of the 6 x 4 footprint on the floor, -y up.

use crate::dat::furniture::Furn;
use crate::dat::zone::{Mmb, Model, Placement, SwayCurves, Texture, Vertex, ZoneScene};
use anyhow::{Context, Result, bail};
use std::collections::HashMap;
use std::sync::OnceLock;

/// The item id (timeless-server modules/timeless_aquarium/sql).
pub const ITEM: u16 = 75;
/// The exdata byte holding the Elder fish bits.
pub const MASK_BYTE: usize = 2;
/// Its furniture entry (the item has none in the retail table): a floor piece, 6 x 4 cells, about 2.7 tall (matches
/// timeless-server's item_furnishing row).
pub const FURN: Furn = Furn { flags: 0, model: 0, size: [6, 4], height: 267 };
/// The mesh name the scene's placements refer to.
pub const MESH: &str = "tlaquarium";

static BUNDLE: &[u8] = include_bytes!("../../../assets/timeless/aquarium/aquarium.bin");

/// A furniture file id of our own for a group of a fill (no DAT has it): keeps each its own loaded set.
pub fn file_id(group: usize, mask: u8) -> u32 {
    0x7A00_0000 | ((group as u32 & 0xff) << 8) | mask as u32
}

/// The groups always drawn, then the fish.
const GROUPS: [&str; 4] = ["tank", "crown", "hgcrystal", "grains"];

/// A group's movement, in its piece's model space (furniture space, -y up).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Anim {
    pub kind: u8,
    pub p: &'static [f32],
}

impl Anim {
    /// The model-space transform at `t` seconds.
    pub fn local(&self, t: f32) -> glam::Mat4 {
        use glam::{Mat4, Quat, Vec3};
        let p = self.p;
        let tau = std::f32::consts::TAU;
        let need = match self.kind {
            1 | 2 => 7,
            3 => 5,
            4 | 5 | 6 => 8,
            _ => 0,
        };
        if p.len() < need {
            return Mat4::IDENTITY;
        }
        match self.kind {
            1 if p[6] != 0.0 => {
                let (c, axis) = (Vec3::new(p[0], p[1], p[2]), Vec3::new(p[3], p[4], p[5]).normalize_or_zero());
                let r = Quat::from_axis_angle(axis, tau * (t / p[6]).rem_euclid(1.0));
                Mat4::from_translation(c) * Mat4::from_quat(r) * Mat4::from_translation(-c)
            }
            2 if p[4] != 0.0 => {
                let c = Vec3::new(p[0], p[1], p[2]);
                let w = tau * (t / p[4]).rem_euclid(1.0) + p[5];
                let off = Vec3::new(p[3] * w.sin(), 0.5 * p[3] * (2.0 * w).sin(), 0.3 * p[3] * w.cos());
                let r = Quat::from_rotation_y((p[6] * w.sin()).to_radians());
                Mat4::from_translation(off + c) * Mat4::from_quat(r) * Mat4::from_translation(-c)
            }
            3 if p[4] != 0.0 => Mat4::from_translation(Vec3::new(p[0], p[1], p[2]) * p[3] * (t / p[4]).rem_euclid(1.0)),
            4 | 5 if p[0] > 0.0 => swim(p, t, self.kind == 5),
            6 if p[0] > 0.0 => slither(p, t),
            _ => Mat4::IDENTITY,
        }
    }

    pub fn moves(&self) -> bool {
        self.kind != 0
    }
}

/// One segment of an eel at `t` (kind 6): where the path had it `dt` seconds ago, shifted sideways by a wave that runs
/// from head to tail and turned to follow the wave's slope: `swim(t - dt) * T(0, 0, a sin w) * Ry(-atan(s cos w))`,
/// with `w = 2 pi beat t - phase`.
fn slither(p: &[f32], t: f32) -> glam::Mat4 {
    use glam::{Mat4, Vec3};
    let n = p[7] as usize;
    let Some(e) = p.get(8 + n * 6..8 + n * 6 + 4) else {
        return Mat4::IDENTITY;
    };
    let (dt, amp, phase, slope) = (e[0], e[1], e[2], e[3]);
    let w = std::f32::consts::TAU * p[3] * t - phase;
    swim(p, t - dt, false)
        * Mat4::from_translation(Vec3::new(0.0, 0.0, amp * w.sin()))
        * Mat4::from_rotation_y(-(slope * w.cos()).atan())
}

/// A fish on its loop at `t` (kinds 4 and 5): the samples interpolated, then its body's sway, and for the tail
/// fin the beat about its joint:
/// `T(pos) * Ry(yaw) * Rz(pitch) * Rx(roll) * Ry(sway) [* T(joint) * Ry(wag) * T(-joint)]`.
fn swim(p: &[f32], t: f32, tail: bool) -> glam::Mat4 {
    use glam::{Mat4, Vec3};
    let (lap, start, sway, beat, joint, wag, lag) = (p[0], p[1], p[2], p[3], p[4], p[5], p[6]);
    let n = p[7] as usize;
    let s = &p[8..];
    if n == 0 || s.len() < n * 6 {
        return Mat4::IDENTITY;
    }
    let x = ((t / lap + start).rem_euclid(1.0) * n as f32).min(n as f32 - 1e-3);
    let i0 = x.floor() as usize % n;
    let i1 = (i0 + 1) % n;
    let f = x - x.floor();
    let (a, b) = (&s[i0 * 6..i0 * 6 + 6], &s[i1 * 6..i1 * 6 + 6]);
    let lerp = |k: usize| a[k] + (b[k] - a[k]) * f;
    let pi = std::f32::consts::PI;
    let dyaw = (b[3] - a[3] + pi).rem_euclid(2.0 * pi) - pi;
    let yaw = a[3] + dyaw * f;
    let w = std::f32::consts::TAU * beat * t;
    let mut m = Mat4::from_translation(Vec3::new(lerp(0), lerp(1), lerp(2)))
        * Mat4::from_rotation_y(yaw)
        * Mat4::from_rotation_z(lerp(4))
        * Mat4::from_rotation_x(lerp(5))
        * Mat4::from_rotation_y(sway * w.sin());
    if tail && wag != 0.0 {
        let j = Vec3::new(joint, 0.0, 0.0);
        m = m * Mat4::from_translation(j) * Mat4::from_rotation_y(wag * (w - lag).sin()) * Mat4::from_translation(-j);
    }
    m
}

/// The Elder fish bits from a piece's exdata.
pub fn mask(ext: &[u8]) -> u8 {
    ext.get(MASK_BYTE).copied().unwrap_or(0)
}

struct Bundle {
    /// Each group: its animation kind and parameters, and its models.
    meshes: HashMap<String, (u8, Vec<f32>, &'static [u8])>,
    pngs: HashMap<String, &'static [u8]>,
    raws: HashMap<String, &'static [u8]>,
}

fn bundle() -> Result<&'static Bundle> {
    static B: OnceLock<Result<Bundle, String>> = OnceLock::new();
    B.get_or_init(|| parse(BUNDLE).map_err(|e| format!("{e:#}")))
        .as_ref()
        .map_err(|e| anyhow::anyhow!("aquarium bundle: {e}"))
}

fn parse(b: &'static [u8]) -> Result<Bundle> {
    let mut r = Reader { b, at: 0 };
    if r.take(4)? != b"TLAQ" {
        bail!("not a TLAQ bundle");
    }
    let version = r.u32()?;
    if version != 3 {
        bail!("version {version}");
    }
    let n = r.u32()?;
    let mut out = Bundle { meshes: HashMap::new(), pngs: HashMap::new(), raws: HashMap::new() };
    for _ in 0..n {
        let kind = r.u8()?;
        let len = r.u16()? as usize;
        let name = String::from_utf8_lossy(r.take(len)?).into_owned();
        let size = r.u32()? as usize;
        let data = r.take(size)?;
        match kind {
            0 => {
                let mut g = Reader { b: data, at: 0 };
                let kind = g.u8()?;
                let count = g.u16()? as usize;
                let p = (0..count).map(|_| g.f32()).collect::<Result<Vec<_>>>()?;
                out.meshes.insert(name, (kind, p, &data[g.at..]));
            }
            1 => {
                out.pngs.insert(name, data);
            }
            _ => {
                out.raws.insert(name, data);
            }
        }
    }
    Ok(out)
}

struct Reader {
    b: &'static [u8],
    at: usize,
}

impl Reader {
    fn take(&mut self, n: usize) -> Result<&'static [u8]> {
        let s = self.b.get(self.at..self.at + n).context("bundle cut short")?;
        self.at += n;
        Ok(s)
    }
    fn u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into()?))
    }
    fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into()?))
    }
    fn f32(&mut self) -> Result<f32> {
        Ok(f32::from_le_bytes(self.take(4)?.try_into()?))
    }
}

/// The variants a fill shows: the tank lit once any fish is in, the crown's star crystal dim, then blazing at all
/// eight.
fn variants(mask: u8) -> [u8; 3] {
    match mask.count_ones() {
        0 => [0, 2, 3],
        8 => [0, 1, 5],
        _ => [0, 1, 4],
    }
}

/// A mesh's models whose variant is wanted, with the texture names they use.
fn models(data: &'static [u8], want: &[u8], out: &mut Vec<Model>, textures: &mut Vec<String>) -> Result<()> {
    let mut r = Reader { b: data, at: 0 };
    let count = r.u32()?;
    for _ in 0..count {
        let len = r.u16()? as usize;
        let texture = String::from_utf8_lossy(r.take(len)?).into_owned();
        let blending = r.u16()?;
        let variant = r.u8()?;
        let (nv, ni) = (r.u32()? as usize, r.u32()? as usize);
        let mut vertices = Vec::with_capacity(nv);
        for _ in 0..nv {
            let pos = [r.f32()?, r.f32()?, r.f32()?];
            let normal = [r.f32()?, r.f32()?, r.f32()?];
            let color = r.u32()?;
            let uv = [r.f32()?, r.f32()?];
            vertices.push(Vertex { pos, normal, color, uv, delta: [0.0; 3] });
        }
        let mut indices = Vec::with_capacity(ni);
        for _ in 0..ni {
            indices.push(r.u16()?);
        }
        if !want.contains(&variant) {
            continue;
        }
        if !textures.contains(&texture) {
            textures.push(texture.clone());
        }
        out.push(Model { texture, blending, vertices, indices, strip: false });
    }
    Ok(())
}

fn decode_png(bytes: &[u8]) -> Result<(u32, u32, Vec<u8>)> {
    let mut d = png::Decoder::new(std::io::Cursor::new(bytes));
    d.set_transformations(png::Transformations::normalize_to_color8() | png::Transformations::ALPHA);
    let mut r = d.read_info()?;
    let mut buf = vec![0u8; r.output_buffer_size().context("png size")?];
    let info = r.next_frame(&mut buf)?;
    buf.truncate(info.buffer_size());
    Ok((info.width, info.height, buf))
}

/// The decoded textures, once.
fn texture(name: &str) -> Result<Texture> {
    static CACHE: OnceLock<std::sync::Mutex<HashMap<String, Texture>>> = OnceLock::new();
    let cache = CACHE.get_or_init(Default::default);
    if let Some(t) = cache.lock().ok().and_then(|c| c.get(name).cloned()) {
        return Ok(t);
    }
    let png = bundle()?.pngs.get(name).with_context(|| format!("no texture {name}"))?;
    let (w, h, rgba) = decode_png(png)?;
    let alpha = rgba.chunks_exact(4).any(|p| p[3] < 250);
    let t = Texture::new(name.to_string(), alpha as u8, w, h, rgba);
    if let Ok(mut c) = cache.lock() {
        c.insert(name.to_string(), t.clone());
    }
    Ok(t)
}

/// One drawn group of a fill: its index (for `file_id`), movement and scene (one mesh, `MESH`).
pub struct Part {
    pub group: usize,
    pub anim: Anim,
    pub scene: ZoneScene,
}

/// The most segments an eel can have, and the group number of segment `k` of fish `bit` (after the bodies and tails).
pub const SEGMENTS: usize = 16;
pub fn segment_group(bit: usize, k: usize) -> usize {
    GROUPS.len() + 16 + bit * SEGMENTS + k
}

/// The groups of a fill: the tank and its moving parts, then the fish of `mask` (each body, then its tail fin
/// when it has one). Group numbers: the tank's groups, then fish n's body at `GROUPS.len() + n`, its tail at
/// `GROUPS.len() + 8 + n`.
pub fn parts(mask: u8) -> Result<Vec<Part>> {
    let b = bundle()?;
    let want = variants(mask);
    let mut names: Vec<(String, usize)> = GROUPS.iter().enumerate().map(|(k, g)| (g.to_string(), k)).collect();
    for bit in (0..8).filter(|bit| mask & (1 << bit) != 0) {
        let body = format!("fish{bit}");
        if b.meshes.contains_key(&body) || !b.meshes.contains_key(&format!("fish{bit}s0")) {
            names.push((body, GROUPS.len() + bit));
        }
        let tail = format!("fish{bit}t");
        if b.meshes.contains_key(&tail) {
            names.push((tail, GROUPS.len() + 8 + bit));
        }
        // an eel's segments
        for k in 0..SEGMENTS {
            let seg = format!("fish{bit}s{k}");
            if !b.meshes.contains_key(&seg) {
                break;
            }
            names.push((seg, segment_group(bit, k)));
        }
    }
    let mut out = Vec::new();
    for (name, group) in names {
        let (kind, p, data) = b.meshes.get(&name).with_context(|| format!("no group {name}"))?;
        out.push(Part { group, anim: Anim { kind: *kind, p: p.as_slice() }, scene: scene_of(data, &want)? });
    }
    Ok(out)
}

fn scene_of(data: &'static [u8], want: &[u8]) -> Result<ZoneScene> {
    let mut list = Vec::new();
    let mut names = Vec::new();
    models(data, want, &mut list, &mut names)?;
    let (mut min, mut max) = ([f32::MAX; 3], [f32::MIN; 3]);
    for v in list.iter().flat_map(|m| &m.vertices) {
        for k in 0..3 {
            min[k] = min[k].min(v.pos[k]);
            max[k] = max[k].max(v.pos[k]);
        }
    }
    let textures = names.iter().map(|n| texture(n)).collect::<Result<Vec<_>>>()?;
    Ok(ZoneScene {
        regions: Vec::new(),
        collision: None,
        placements: Vec::<Placement>::new(),
        meshes: vec![Mmb { name: MESH.into(), min, max, models: list, animated: false, mesh_type: 0 }],
        textures,
        warnings: Vec::new(),
        light_keys: Vec::new(),
        paths: Default::default(),
        camera_lists: Default::default(),
        list_base: 0,
        sway: SwayCurves::default(),
        bumps: Vec::new(),
    })
}

/// The item icon in the item DAT layout.
pub fn icon_record() -> Option<&'static [u8]> {
    bundle().ok()?.raws.get("icon32").copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn textures(parts: &[Part]) -> Vec<String> {
        parts.iter().flat_map(|p| p.scene.meshes[0].models.iter().map(|m| m.texture.clone())).collect()
    }

    /// Every fill builds: the tank's groups, a body (and, but for Rimecoil, a tail fin) per fish bit, the right
    /// crown and lighting, and the whole piece fits its 6 x 4 cells.
    #[test]
    fn fills() {
        let empty = parts(0).unwrap();
        assert_eq!(empty.len(), GROUPS.len());
        let t = textures(&empty);
        assert!(t.iter().any(|n| n.starts_with("tlqcrow") && n.ends_with("_3")), "crown off when empty: {t:?}");
        assert!(t.iter().all(|n| !n.ends_with("_1")), "nothing lit when empty");
        let one = parts(1 << 4).unwrap();
        assert_eq!(one.len(), GROUPS.len() + 2);
        assert_eq!(one[GROUPS.len()].group, GROUPS.len() + 4);
        assert_eq!(one[GROUPS.len() + 1].group, GROUPS.len() + 8 + 4);
        let t = textures(&one);
        assert!(t.iter().any(|n| n.starts_with("tlqf4_")) && !t.iter().any(|n| n.starts_with("tlqf0_")));
        assert!(t.iter().any(|n| n.starts_with("tlqf4t_")), "the tail fin is its own group");
        assert!(t.iter().any(|n| n.starts_with("tlqcrow") && n.ends_with("_4")), "crown dim with some fish");
        let full = parts(0xff).unwrap();
        assert_eq!(full.len(), GROUPS.len() + 7 + 7 + 12, "Rimecoil is twelve segments, not a body and a tail");
        let eel = parts(1 << 5).unwrap();
        assert_eq!(eel.len(), GROUPS.len() + 12);
        assert!(
            eel[GROUPS.len()..].iter().enumerate().all(|(k, p)| p.group == segment_group(5, k) && p.anim.kind == 6)
        );
        assert!(textures(&full).iter().any(|n| n.starts_with("tlqcrow") && n.ends_with("_5")), "crown full at eight");
        let mut ids: Vec<_> = full.iter().map(|p| p.group).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), full.len(), "each group its own file id");
        let (mut min, mut max) = ([f32::MAX; 3], [f32::MIN; 3]);
        for p in full.iter().filter(|p| p.group < GROUPS.len()) {
            let mesh = &p.scene.meshes[0];
            for m in &mesh.models {
                assert!(p.scene.textures.iter().any(|t| t.name == m.texture));
                assert!(m.indices.iter().all(|&i| (i as usize) < m.vertices.len()));
                assert_eq!(m.indices.len() % 3, 0);
            }
            for k in 0..3 {
                min[k] = min[k].min(mesh.min[k]);
                max[k] = max[k].max(mesh.max[k]);
            }
        }
        assert!(max[0] - min[0] <= 3.61 && max[2] - min[2] <= 2.41, "fits 6 x 4 cells: {min:?} {max:?}");
        assert!(max[1] <= 0.001 && max[1] > -0.01 && min[1] > -2.7, "stands on the floor, -y up: {min:?} {max:?}");
        assert!(-min[1] * 100.0 <= FURN.height as f32 + 1.0, "the entry's height covers it: {min:?}");
    }

    /// The moving parts: the crown spins, the grains fall, the tank stays put; every fish swims its loop inside
    /// the tank, its tail fin beating about a joint that stays on the body.
    #[test]
    fn movement() {
        let full = parts(0xff).unwrap();
        let by = |g: usize| full.iter().find(|p| p.group == g).unwrap().anim;
        assert!(!by(0).moves());
        let crown = by(1);
        assert_eq!(crown.kind, 1);
        let c = glam::Vec3::new(crown.p[0], crown.p[1], crown.p[2]);
        // the pivot stays where it is, a point beside it goes round
        assert!((crown.local(3.0).transform_point3(c) - c).length() < 1e-4);
        let side = c + glam::Vec3::X * 0.05;
        assert!((crown.local(crown.p[6] / 4.0).transform_point3(side) - side).length() > 0.03);
        assert!((crown.local(crown.p[6]).transform_point3(side) - side).length() < 1e-3);
        let grains = by(3);
        assert_eq!(grains.kind, 3);
        let y = |t: f32| grains.local(t).w_axis.y;
        assert!(y(grains.p[4] * 0.5) > y(0.0), "falls toward +y (down in furniture space)");
        let (tank_min, tank_max) = {
            let m = &full[0].scene.meshes[0];
            (glam::Vec3::from(m.min), glam::Vec3::from(m.max))
        };
        for bit in (0..8).filter(|&bit| bit != 5) {
            let body = by(GROUPS.len() + bit);
            assert_eq!(body.kind, 4);
            let lap = body.p[0];
            let (mut lo, mut hi) = (glam::Vec3::splat(f32::MAX), glam::Vec3::splat(f32::MIN));
            for k in 0..200 {
                let at = body.local(lap * k as f32 / 200.0).w_axis.truncate();
                lo = lo.min(at);
                hi = hi.max(at);
                assert!(at.cmpgt(tank_min).all() && at.cmplt(tank_max).all(), "fish {bit} inside the tank: {at}");
            }
            assert!((hi - lo).max_element() > 0.6, "fish {bit} swims, not just drifts: {lo} {hi}");
            if let Some(tail) = full.iter().find(|p| p.group == GROUPS.len() + 8 + bit).map(|p| p.anim) {
                assert_eq!(tail.kind, 5);
                let j = glam::Vec3::new(tail.p[4], 0.0, 0.0);
                for t in [0.4f32, 3.3, 11.9] {
                    let (bm, tm) = (body.local(t), tail.local(t));
                    assert!((bm.transform_point3(j) - tm.transform_point3(j)).length() < 1e-4, "the joint holds");
                    let tip = j - glam::Vec3::X * 0.1;
                    let _ = (bm.transform_point3(tip) - tm.transform_point3(tip)).length();
                }
                let beat = (0..40)
                    .map(|k| {
                        let t = k as f32 * 0.05;
                        let tip = j - glam::Vec3::X * 0.1;
                        (body.local(t).transform_point3(tip) - tail.local(t).transform_point3(tip)).length()
                    })
                    .fold(0f32, f32::max);
                assert!(beat > 0.01, "fish {bit}'s tail beats: {beat}");
            }
        }
        // Rimecoil slithers: its segments stay a body length together round the whole lap, each inside the tank,
        // and a segment swings from side to side as the wave runs down it
        let segs: Vec<_> = (0..12).map(|k| by(segment_group(5, k))).collect();
        let lap = segs[0].p[0];
        for k in 0..100 {
            let t = lap * k as f32 / 100.0;
            let at: Vec<glam::Vec3> = segs.iter().map(|a| a.local(t).w_axis.truncate()).collect();
            for w in at.windows(2) {
                assert!((w[1] - w[0]).length() < 0.15, "segments hold together at {t}: {}", (w[1] - w[0]).length());
            }
            assert!(at.iter().all(|a| a.cmpgt(tank_min).all() && a.cmplt(tank_max).all()), "the eel inside the tank");
            let span = (at[11] - at[0]).length();
            assert!(span > 0.6 && span < 1.0, "head to tail about its length: {span}");
        }
        let tail = segs[0];
        let side = |t: f32| {
            let base = slither_base(tail, t);
            (base.inverse() * tail.local(t)).w_axis.z
        };
        let swing =
            (0..40).map(|k| side(k as f32 * 0.05)).fold((f32::MAX, f32::MIN), |(lo, hi), z| (lo.min(z), hi.max(z)));
        assert!(swing.1 - swing.0 > 0.05, "the tail swings side to side: {swing:?}");
        // the same maths as swim.py: segments 0 and 7 at 9.1 s
        let s0 = segs[0].local(9.1).transform_point3(glam::Vec3::new(0.02, 0.01, 0.0));
        let s7 = segs[7].local(9.1).transform_point3(glam::Vec3::new(0.02, 0.01, 0.0));
        assert!((s0 - glam::Vec3::new(0.940_679, -1.649_565, -0.579_681)).length() < 2e-3, "segment 0 {s0}");
        assert!((s7 - glam::Vec3::new(1.009_693, -1.643_569, -0.009_225)).length() < 2e-3, "segment 7 {s7}");
        // the same maths as the tools' swim.py: Moonfin (bit 4) at 7.3 s
        let (body, tail) = (by(GROUPS.len() + 4), by(GROUPS.len() + 8 + 4));
        let b = body.local(7.3).transform_point3(glam::Vec3::new(0.1, 0.05, 0.02));
        let t = tail.local(7.3).transform_point3(glam::Vec3::new(-0.4, 0.02, 0.03));
        assert!((b - glam::Vec3::new(0.045_907, -1.348_229, -0.760_380)).length() < 2e-3, "body {b}");
        assert!((t - glam::Vec3::new(0.529_529, -1.411_366, -0.817_873)).length() < 2e-3, "tail {t}");
    }

    /// A segment's place on the path, before its wave.
    fn slither_base(a: Anim, t: f32) -> glam::Mat4 {
        let n = a.p[7] as usize;
        super::swim(a.p, t - a.p[8 + n * 6], false)
    }

    #[test]
    fn icon_and_mask() {
        let rec = icon_record().unwrap();
        assert_eq!(rec.len(), 0x11 + 40 + 1024 + 32 * 32);
        assert_eq!(mask(&[0, 0, 0b1010_0001, 0, 0, 0, 1, 2, 3, 0]), 0b1010_0001);
        assert_eq!(file_id(7, 5), 0x7A00_0705);
    }
}
