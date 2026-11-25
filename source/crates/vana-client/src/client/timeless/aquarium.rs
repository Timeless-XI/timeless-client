//! Timeless XI: the Timeless Aquarium (item 75), the Elder fish furniture.
//!
//! The server keeps which Elder fish a player has caught in the item's exdata byte 2 (bit n = Elder fish n,
//! timeless-server modules/timeless_aquarium); this draws the tank with exactly those fish in it, like a flowerpot
//! draws its plant. The tank, the eight fish and their baked textures come from one bundle,
//! `assets/timeless/aquarium/aquarium.bin` (built by the Elder fish tools, `export_aquarium.py`):
//!
//! - `b"TLAQ"`, u32 version 1, u32 entries; entry = u8 kind (0 mesh, 1 PNG, 2 raw), u16 name length, name, u32
//!   length, bytes.
//! - Mesh (version 2) = one GROUP, drawn as its own piece so it can move on its own: u8 animation kind, f32 x 8
//!   parameters, u32 models; model = u16 texture-name length, texture name, u16 blending (MMB bits: 0x8000
//!   blended, 0x2000 two-sided), u8 variant, u32 vertices, u32 indices, vertices (pos f32x3, normal f32x3, colour
//!   u32 BGRA with 0x80 = 1.0, uv f32x2), indices u16 (a triangle list).
//! - Animation (`Anim`): 0 none; 1 spin (pivot xyz, axis xyz, period s); 2 bob (pivot xyz, amplitude, period s,
//!   phase, yaw degrees); 3 fall (direction xyz, spacing, period s): a looped slide by one spacing.
//! - Variant: 0 always; 1 lit (1+ fish) / 2 dark (none); crown 3 off (none) / 4 dim (1-7) / 5 full (all 8).
//! - Groups: `tank` (frame, night sky, sand, ruins, kelp, the hourglass), `crown` (the star
//!   crystal on top, spins), `hgcrystal` (the hourglass's star crystal, spins), `grains` (falling sand),
//!   `fish0`..`fish7` (placed in the tank, each drifts). The PNGs by
//!   texture name; `icon32` the item icon in the item DAT layout (dat/items.rs `icon_rgba`).
//!
//! Model space is furniture space: origin at the middle of the 3 x 2 footprint on the floor, -y up.

use crate::dat::furniture::Furn;
use crate::dat::zone::{Mmb, Model, Placement, SwayCurves, Texture, Vertex, ZoneScene};
use anyhow::{Context, Result, bail};
use std::collections::HashMap;
use std::sync::OnceLock;

/// The item id (timeless-server modules/timeless_aquarium/sql).
pub const ITEM: u16 = 75;
/// The exdata byte holding the Elder fish bits.
pub const MASK_BYTE: usize = 2;
/// Its furniture entry (the item has none in the retail table): a floor piece, 3 x 2 cells, 1.4 tall.
pub const FURN: Furn = Furn { flags: 0, model: 0, size: [3, 2], height: 140 };
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
    pub p: [f32; 8],
}

impl Anim {
    /// The model-space transform at `t` seconds.
    pub fn local(&self, t: f32) -> glam::Mat4 {
        use glam::{Mat4, Quat, Vec3};
        let p = &self.p;
        let tau = std::f32::consts::TAU;
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
            _ => Mat4::IDENTITY,
        }
    }

    pub fn moves(&self) -> bool {
        self.kind != 0
    }
}

/// The Elder fish bits from a piece's exdata.
pub fn mask(ext: &[u8]) -> u8 {
    ext.get(MASK_BYTE).copied().unwrap_or(0)
}

struct Bundle {
    meshes: HashMap<String, (Anim, &'static [u8])>,
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
    if version != 2 {
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
                let mut p = [0f32; 8];
                for v in &mut p {
                    *v = g.f32()?;
                }
                out.meshes.insert(name, (Anim { kind, p }, &data[g.at..]));
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

/// The groups of a fill: the tank and its moving parts, then the fish of `mask`.
pub fn parts(mask: u8) -> Result<Vec<Part>> {
    let b = bundle()?;
    let want = variants(mask);
    let fish = (0..8).filter(|bit| mask & (1 << bit) != 0).map(|bit| format!("fish{bit}"));
    let names: Vec<String> = GROUPS.iter().map(|g| g.to_string()).chain(fish).collect();
    let mut out = Vec::new();
    for name in names {
        let group = GROUPS
            .iter()
            .position(|g| *g == name)
            .unwrap_or_else(|| GROUPS.len() + name[4..].parse::<usize>().unwrap_or(0));
        let (anim, data) = b.meshes.get(&name).with_context(|| format!("no group {name}"))?;
        out.push(Part { group, anim: *anim, scene: scene_of(data, &want)? });
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

    /// Every fill builds: the six tank groups, one group per fish bit, the right crown and lighting.
    #[test]
    fn fills() {
        let empty = parts(0).unwrap();
        assert_eq!(empty.len(), GROUPS.len());
        let t = textures(&empty);
        assert!(t.iter().any(|n| n.starts_with("tlqcrow") && n.ends_with("_3")), "crown off when empty: {t:?}");
        assert!(t.iter().all(|n| !n.ends_with("_1")), "nothing lit when empty");
        let one = parts(1 << 4).unwrap();
        assert_eq!(one.len(), GROUPS.len() + 1);
        assert_eq!(one.last().unwrap().group, GROUPS.len() + 4);
        let t = textures(&one);
        assert!(t.iter().any(|n| n.starts_with("tlqf4_")) && !t.iter().any(|n| n.starts_with("tlqf0_")));
        assert!(t.iter().any(|n| n.starts_with("tlqcrow") && n.ends_with("_4")), "crown dim with some fish");
        let full = parts(0xff).unwrap();
        assert_eq!(full.len(), GROUPS.len() + 8);
        assert!(textures(&full).iter().any(|n| n.starts_with("tlqcrow") && n.ends_with("_5")), "crown full at eight");
        let (mut min, mut max) = ([f32::MAX; 3], [f32::MIN; 3]);
        for p in &full {
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
        assert!(max[0] - min[0] <= 1.81 && max[2] - min[2] <= 1.21, "fits 3 x 2 cells: {min:?} {max:?}");
        assert!(max[1] <= 0.001 && min[1] > -1.6, "stands on the floor, -y up: {min:?} {max:?}");
    }

    /// The moving parts: the crown spins, the fish drift, the grains fall; the tank stays put.
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
        for g in GROUPS.len()..GROUPS.len() + 8 {
            let a = by(g);
            assert_eq!(a.kind, 2);
            assert!(a.local(1.3).w_axis.truncate().length() < 0.1, "a drift, not a swim across the room");
        }
    }

    #[test]
    fn icon_and_mask() {
        let rec = icon_record().unwrap();
        assert_eq!(rec.len(), 0x11 + 40 + 1024 + 32 * 32);
        assert_eq!(mask(&[0, 0, 0b1010_0001, 0, 0, 0, 1, 2, 3, 0]), 0b1010_0001);
        assert_eq!(file_id(7, 5), 0x7A00_0705);
    }
}
