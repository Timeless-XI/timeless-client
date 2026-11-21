//! Timeless XI: scouting a zone for the Select screen's vignettes (client/timeless/vignette.rs). The owner's brief
//! (2026-11-20): the character stands in the most scenic part of their zone, framed from the front with the scenery
//! behind them; the walk comes toward the camera; nothing cut off, nothing clipped.
//!
//! A **scene** is a place and a heading. Walking: the character starts at `at` with the view at their back and walks
//! `len` along `dir` toward the camera. Fishing: they stand at `at` facing the water along `dir`, and the camera
//! starts over the water looking back at them, the shore behind.
//!
//! Scouting reads only the zone's collision: candidate spots on open ground (walkable, not under cover, room to
//! walk), each judged by the view a camera in front of the character would have past them (rays through the frame
//! around the character: open distance and sky, depth and variety, a drop to a valley below; a wall close behind
//! counts against it) and by room for the camera along the whole walk. `vana-client vignette-scout` runs it over
//! zones into `assets/timeless/vignette_scenes.json`, which the client reads first; a zone missing there is
//! scouted on the loader thread with fewer samples.

use super::super::ground::Ground;
use glam::Vec3;
use serde::{Deserialize, Serialize};

/// The nominal character height the scout frames (a Hume), and how much of the frame the body fills.
pub const NOMINAL_HEIGHT: f32 = 1.9;
pub const FILL: f32 = 0.52;
/// The camera's focal (title.rs focal: 192 / tan(fov / 2)) for a walking and a standing shot.
pub const FOCAL: f32 = 520.0;
/// The front camera's angle off the heading (degrees), toward `side`.
pub const FRONT_DEG: f32 = 22.0;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Scene {
    pub at: [f32; 3],
    /// x, z.
    pub dir: [f32; 2],
    /// Walking: the walk's length; fishing: 0.
    #[serde(default)]
    pub len: f32,
    /// The camera's side of the heading (+1 right, -1 left).
    pub side: f32,
    #[serde(default)]
    pub score: f32,
    /// The opening camera's angle off `dir` (degrees, toward `side`), when someone chose it (a `/vigspot` place);
    /// else `FRONT_DEG`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub front: Option<f32>,
}

impl Scene {
    pub fn at(&self) -> Vec3 {
        Vec3::from(self.at)
    }

    pub fn dir(&self) -> Vec3 {
        Vec3::new(self.dir[0], 0.0, self.dir[1]).normalize_or(Vec3::X)
    }
}

/// A zone's scenes, best first.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ZoneScenes {
    #[serde(default)]
    pub walk: Vec<Scene>,
    #[serde(default)]
    pub fish: Vec<Scene>,
}

/// The scouted table (assets/timeless/vignette_scenes.json): zone id -> scenes.
static TABLE: std::sync::LazyLock<std::collections::HashMap<String, ZoneScenes>> = std::sync::LazyLock::new(|| {
    // VANA_VIGNETTE_SCENES: a table to try in place of the built-in one (vignette-scout's output).
    let file = std::env::var_os("VANA_VIGNETTE_SCENES").and_then(|p| std::fs::read_to_string(p).ok());
    serde_json::from_str(file.as_deref().unwrap_or(include_str!("../../../assets/timeless/vignette_scenes.json")))
        .unwrap_or_default()
});

pub fn table(zone: u16) -> Option<ZoneScenes> {
    TABLE.get(&zone.to_string()).cloned()
}

/// The zones in the scouted table.
pub fn table_zones() -> Vec<u16> {
    let mut z: Vec<u16> = TABLE.keys().filter_map(|k| k.parse().ok()).collect();
    z.sort();
    z
}

pub fn xz(v: Vec3) -> Vec3 {
    Vec3::new(v.x, 0.0, v.z)
}

/// `f` turned by `deg` about the vertical (toward +z from +x).
pub fn turn(f: Vec3, deg: f32) -> Vec3 {
    let (s, c) = deg.to_radians().sin_cos();
    Vec3::new(f.x * c - f.z * s, 0.0, f.x * s + f.z * c).normalize_or(Vec3::X)
}

/// The camera distance that frames a body `h` tall at `focal` so it fills `fill` of the frame's height.
pub fn frame_distance(h: f32, focal: f32, fill: f32) -> f32 {
    h * focal / (fill * 2.0 * 192.0)
}

/// The focal that frames a body `h` tall from `d` away at `fill` (the inverse of `frame_distance`).
pub fn frame_focal(h: f32, d: f32, fill: f32) -> f32 {
    d * fill * 2.0 * 192.0 / h.max(0.1)
}

/// The eye for a body at `feet`, `h` tall, from `d` away along `dir` (level with the upper chest), and the target.
pub fn eye_target(feet: Vec3, h: f32, dir: Vec3, d: f32) -> (Vec3, Vec3) {
    let chest = feet - Vec3::Y * (h * 0.62);
    let eye = chest + xz(dir).normalize_or(Vec3::X) * d - Vec3::Y * (h * 0.04);
    let target = feet - Vec3::Y * (h * 0.56);
    (eye, target)
}

/// Every drawn placement's world box and mesh, once a zone (`SceneIndex::local` cuts the triangles near a spot).
pub struct SceneIndex {
    placed: Vec<(Vec3, Vec3, glam::Mat4, usize)>,
}

impl SceneIndex {
    pub fn build(scene: &crate::dat::zone::ZoneScene) -> Self {
        // A name can stand for several meshes (the parts of one object): all of them.
        let mut by_name: std::collections::HashMap<&str, Vec<usize>> = Default::default();
        for (i, m) in scene.meshes.iter().enumerate() {
            by_name.entry(m.name.as_str()).or_default().push(i);
        }
        let bounds: Vec<(Vec3, Vec3)> = scene
            .meshes
            .iter()
            .map(|m| {
                let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
                for v in m.models.iter().flat_map(|x| x.vertices.iter()) {
                    (lo, hi) = (lo.min(Vec3::from(v.pos)), hi.max(Vec3::from(v.pos)));
                }
                (lo, hi)
            })
            .collect();
        let mut placed = Vec::new();
        for p in scene.placements.iter().filter(|p| p.drawn() && !p.world) {
            let names = p.lod_names(|n| by_name.contains_key(n));
            let Some(list) = by_name.get(names[2].as_ref()).or_else(|| by_name.get(p.name.as_str())) else { continue };
            let m = glam::Mat4::from_cols_array_2d(&p.matrix());
            for &mi in list {
                let (lo, hi) = bounds[mi];
                if lo.x > hi.x || !lo.is_finite() || !hi.is_finite() {
                    continue;
                }
                let (mut a, mut b) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
                for k in 0..8 {
                    let c = Vec3::new(
                        if k & 1 == 0 { lo.x } else { hi.x },
                        if k & 2 == 0 { lo.y } else { hi.y },
                        if k & 4 == 0 { lo.z } else { hi.z },
                    );
                    let w = m.transform_point3(c);
                    (a, b) = (a.min(w), b.max(w));
                }
                placed.push((a, b, m, mi));
            }
        }
        Self { placed }
    }

    /// The drawn triangles within `r` (across the ground) of `centre`.
    pub fn local(&self, scene: &crate::dat::zone::ZoneScene, centre: Vec3, r: f32) -> Scenery {
        let mut out = Scenery::default();
        let (lo, hi) = (centre - Vec3::new(r, 60.0, r), centre + Vec3::new(r, 60.0, r));
        for (a, b, m, mi) in &self.placed {
            if a.cmpgt(hi).any() || b.cmplt(lo).any() {
                continue;
            }
            for model in &scene.meshes[*mi].models {
                let v = &model.vertices;
                let mut push = |i0: u16, i1: u16, i2: u16| {
                    if i0 == i1 || i1 == i2 || i0 == i2 {
                        return;
                    }
                    let (Some(p0), Some(p1), Some(p2)) = (v.get(i0 as usize), v.get(i1 as usize), v.get(i2 as usize))
                    else {
                        return;
                    };
                    let t = [p0, p1, p2].map(|p| m.transform_point3(Vec3::from(p.pos)));
                    let (tl, th) = (t[0].min(t[1]).min(t[2]), t[0].max(t[1]).max(t[2]));
                    if tl.cmpgt(hi).any() || th.cmplt(lo).any() {
                        return;
                    }
                    out.add(t);
                };
                let ix = &model.indices;
                if model.strip {
                    for i in 0..ix.len().saturating_sub(2) {
                        push(ix[i], ix[i + 1], ix[i + 2]);
                    }
                } else {
                    for c in ix.chunks_exact(3) {
                        push(c[0], c[1], c[2]);
                    }
                }
            }
        }
        out
    }
}

/// The scenery's own triangles near a spot (trees, posts, statues, walls the collision leaves out), so the camera
/// never looks through any of it at the character.
#[derive(Default)]
pub struct Scenery {
    tris: Vec<[Vec3; 3]>,
    cells: std::collections::HashMap<(i32, i32), Vec<u32>>,
    /// Faces too big for the cells (a wide sheet of ground): tested on every segment.
    big: Vec<u32>,
}

const SCENERY_CELL: f32 = 2.0;

fn cell_of(x: f32, z: f32) -> (i32, i32) {
    ((x / SCENERY_CELL).floor() as i32, (z / SCENERY_CELL).floor() as i32)
}

impl Scenery {
    fn add(&mut self, t: [Vec3; 3]) {
        let i = self.tris.len() as u32;
        let (lo, hi) = (t[0].min(t[1]).min(t[2]), t[0].max(t[1]).max(t[2]));
        let (c0, c1) = (cell_of(lo.x, lo.z), cell_of(hi.x, hi.z));
        if (c1.0 - c0.0 + 1) * (c1.1 - c0.1 + 1) > 256 {
            self.big.push(i);
        } else {
            for x in c0.0..=c1.0 {
                for z in c0.1..=c1.1 {
                    self.cells.entry((x, z)).or_default().push(i);
                }
            }
        }
        self.tris.push(t);
    }

    pub fn len(&self) -> usize {
        self.tris.len()
    }

    /// A face on the segment `a -> b`, more than 0.35 from the eye and 0.15 from the body.
    pub fn blocks(&self, a: Vec3, b: Vec3) -> bool {
        self.first_hit(a, b).is_some_and(|t| t > 0.35 && t < (b - a).length() - 0.15)
    }

    /// The nearest face along `a -> b` (distance from `a`), checking every face in the cells the segment crosses.
    pub fn first_hit(&self, a: Vec3, b: Vec3) -> Option<f32> {
        let mut best: Option<f32> = None;
        let d = b - a;
        let len = d.length();
        if len < 0.8 {
            return None;
        }
        let dir = d / len;
        let mut seen = std::collections::HashSet::new();
        let steps = (len / (SCENERY_CELL * 0.5)).ceil() as i32;
        let mut ids: Vec<u32> = self.big.clone();
        for s in 0..=steps {
            let p = a + d * (s as f32 / steps.max(1) as f32);
            ids.extend(self.cells.get(&cell_of(p.x, p.z)).into_iter().flatten());
        }
        {
            for &i in &ids {
                if !seen.insert(i) {
                    continue;
                }
                let [v0, v1, v2] = self.tris[i as usize];
                let (e1, e2) = (v1 - v0, v2 - v0);
                let h = dir.cross(e2);
                let det = e1.dot(h);
                if det.abs() < 1e-7 {
                    continue;
                }
                let inv = 1.0 / det;
                let sv = a - v0;
                let u = inv * sv.dot(h);
                if !(0.0..=1.0).contains(&u) {
                    continue;
                }
                let q = sv.cross(e1);
                let v = inv * dir.dot(q);
                if v < 0.0 || u + v > 1.0 {
                    continue;
                }
                let t = inv * e2.dot(q);
                if t > 0.35 && t < len - 0.15 && best.is_none_or(|b| t < b) {
                    best = Some(t);
                }
            }
        }
        best
    }

    /// Room for a body `h` tall standing at `feet`: no scenery through it (a rock or trunk the collision leaves
    /// out, which a walk would pass straight through).
    pub fn room_for(&self, feet: Vec3, h: f32) -> bool {
        let up = |k: f32| feet - Vec3::Y * (h * k);
        let r = 0.35;
        let segs = [
            (up(0.05) + Vec3::X * r, up(0.05) - Vec3::X * r),
            (up(0.05) + Vec3::Z * r, up(0.05) - Vec3::Z * r),
            (up(0.5) + Vec3::X * r, up(0.5) - Vec3::X * r),
            (up(0.5) + Vec3::Z * r, up(0.5) - Vec3::Z * r),
            (up(0.9) + Vec3::X * r, up(0.9) - Vec3::X * r),
        ];
        segs.iter().all(|(a, b)| self.any_hit(*a, *b).is_none()) && self.any_hit(up(0.1), up(1.0)).is_none()
    }

    /// Any face on the segment (no margins).
    fn any_hit(&self, a: Vec3, b: Vec3) -> Option<f32> {
        let (len, dir) = ((b - a).length(), (b - a).normalize_or_zero());
        let mut ids: Vec<u32> = self.big.clone();
        for p in [a, b, (a + b) * 0.5] {
            ids.extend(self.cells.get(&cell_of(p.x, p.z)).into_iter().flatten());
        }
        ids.into_iter().find_map(|i| {
            let [v0, v1, v2] = self.tris[i as usize];
            let (e1, e2) = (v1 - v0, v2 - v0);
            let h = dir.cross(e2);
            let det = e1.dot(h);
            if det.abs() < 1e-7 {
                return None;
            }
            let inv = 1.0 / det;
            let sv = a - v0;
            let u = inv * sv.dot(h);
            let q = sv.cross(e1);
            let v = inv * dir.dot(q);
            let t = inv * e2.dot(q);
            ((0.0..=1.0).contains(&u) && v >= 0.0 && u + v <= 1.0 && t >= 0.0 && t <= len).then_some(t)
        })
    }

    /// The camera at `eye` sees the whole body: head, chest and knees, past the scenery.
    pub fn sees(&self, eye: Vec3, feet: Vec3, h: f32) -> bool {
        // The body's outline and a little room round it, not only its middle: a trunk beside them crowds the shot.
        let side = xz((feet - eye).cross(Vec3::Y)).normalize_or(Vec3::X);
        [(0.95, 0.0), (0.62, 0.0), (0.3, 0.0), (0.7, 0.5), (0.7, -0.5), (0.3, 0.4), (0.3, -0.4), (0.12, 0.0)]
            .iter()
            .all(|(k, s)| !self.blocks(eye, feet - Vec3::Y * (h * k) + side * (h * s)))
    }
}

/// The whole body seen from `eye`: past the scenery's own faces and past the collision (a ridge in front of the
/// legs, a wall's edge).
pub fn body_visible(ground: &Ground, scenery: &Scenery, eye: Vec3, feet: Vec3, h: f32) -> bool {
    scenery.sees(eye, feet, h)
        && [0.95f32, 0.5, 0.12].iter().all(|k| {
            let p = feet - Vec3::Y * (h * k);
            let len = (p - eye).length();
            ground.raycast(eye, p).is_none_or(|t| t > len - 0.12)
        })
}

/// The eye kept off the ground: at least 0.6 over the floor under it.
pub fn above_ground(ground: &Ground, eye: Vec3) -> Vec3 {
    match ground.nearest(eye.x, eye.z, eye.y, 6.0) {
        Some(f) if f < eye.y + 0.6 => Vec3::new(eye.x, f - 0.6, eye.z),
        _ => eye,
    }
}

/// How free the line from the body (chest) to the eye is, 0..1 (1: nothing in the way).
pub fn clear(ground: &Ground, chest: Vec3, eye: Vec3) -> f32 {
    let len = (eye - chest).length().max(1e-3);
    ground.cam_ray(chest, eye).map_or(1.0, |h| ((h - chest).length() / len).clamp(0.0, 1.0))
}

/// The view past a body: a camera at `eye` looking at `target`, rays through the frame outside the body's middle.
/// Higher is better; a wall close behind the body and an empty frame both score low.
pub fn view_score(ground: &Ground, eye: Vec3, target: Vec3, rays: (usize, usize), reach: f32) -> f32 {
    let fwd = (target - eye).normalize_or(Vec3::X);
    let right = xz(fwd.cross(Vec3::NEG_Y)).normalize_or(Vec3::Z);
    let up = right.cross(fwd).normalize_or(Vec3::NEG_Y);
    // The frame at FOCAL, 16:9.
    let ty = 192.0 / FOCAL;
    let tx = ty * 16.0 / 9.0;
    let (nx, ny) = rays;
    let (mut n, mut sky, mut near, mut mid, mut far, mut drop, mut void) = (0.0f32, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    let mut logs = Vec::new();
    let body = (target - eye).length();
    for j in 0..ny {
        for i in 0..nx {
            let u = (i as f32 + 0.5) / nx as f32 * 2.0 - 1.0;
            let v = (j as f32 + 0.5) / ny as f32 * 2.0 - 1.0;
            // The body's middle (the character stands there) says nothing about the view.
            if u.abs() < 0.22 && v < 0.45 {
                continue;
            }
            let d = (fwd + right * (u * tx) + up * (v * ty)).normalize();
            n += 1.0;
            match ground.raycast(eye, eye + d * reach) {
                // Nothing below the horizon: past the zone's edge (the void under a backdrop).
                None if v < -0.15 => void += 1.0,
                None => sky += 1.0,
                Some(t) => {
                    // Past the body only: what is closer than it is the camera's own business (`clear`).
                    let past = t - body;
                    if past < 6.0 {
                        near += 1.0;
                    } else if past < 35.0 {
                        mid += 1.0;
                    } else {
                        far += 1.0;
                        if v < 0.0 {
                            drop += 1.0;
                        }
                    }
                    logs.push(t.max(0.5).ln());
                }
            }
        }
    }
    if n == 0.0 {
        return 0.0;
    }
    let (sky, near, mid, far, drop, void) = (sky / n, near / n, mid / n, far / n, drop / n, void / n);
    let variety = if logs.len() > 2 {
        let m = logs.iter().sum::<f32>() / logs.len() as f32;
        (logs.iter().map(|l| (l - m) * (l - m)).sum::<f32>() / logs.len() as f32).sqrt()
    } else {
        0.0
    };
    // Some sky (a third of the frame is best), mostly far ground and things, a little mid-ground, variety.
    let sky_term = (1.0 - (sky - 0.32).abs() / 0.32).clamp(0.0, 1.0);
    far * 1.0 + mid * 0.45 + sky_term * 0.55 + (variety / 1.4).min(1.0) * 0.35 + drop * 0.6 - near * 2.2 - void * 3.0
}

/// Open ground a character can stand on: walkable surface, not under cover, headroom.
fn open_ground(ground: &Ground, p: Vec3) -> bool {
    let (kind, covered) = ground.surface(p);
    // 1 path .. 7 wood (0 objects such as roofs, 8 / 9 water).
    if !(1..=7).contains(&kind) || covered || ground.covered(p - Vec3::Y * 1.5) {
        return false;
    }
    ground.raycast(p - Vec3::Y * 0.3, p - Vec3::Y * 4.5).is_none()
}

/// The walk from `start` along `dir`: points every half unit (the ground's own walk, so walls and drops stop it).
pub fn walk_points(ground: &Ground, start: Vec3, dir: Vec3, len: f32) -> Vec<Vec3> {
    let mut path = vec![start];
    let mut p = start;
    let mut gone = 0.0;
    while gone < len - 0.25 {
        let next = ground.walk(p, p + dir * 0.5, 0.0);
        let step = xz(next - p).length();
        if step < 0.2 || (next.y - p.y).abs() > 0.6 {
            break;
        }
        gone += step;
        p = next;
        path.push(p);
    }
    path
}

/// Faces to sample: flat enough to stand on, big faces weighing more.
fn floor_faces(ground: &Ground) -> Vec<(Vec3, f32)> {
    ground
        .all_tris()
        .filter_map(|[a, b, c]| {
            let n = (*b - *a).cross(*c - *a);
            let area = n.length() * 0.5;
            (area > 0.6 && n.normalize_or_zero().y.abs() > 0.88).then(|| ((*a + *b + *c) / 3.0, area))
        })
        .collect()
}

struct Rng(u64);

impl Rng {
    fn f(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % 1_000_000) as f32 / 1_000_000.0
    }
}

/// Picks of open ground: around `near` (places players reach: spawn points, NPCs, where they left off) when any,
/// else area-weighted over the zone's floors.
fn candidates(ground: &Ground, faces: &[(Vec3, f32)], tries: usize, seed: u64, near: &[Vec3]) -> Vec<Vec3> {
    if !near.is_empty() {
        let mut rng = Rng(seed | 1);
        let mut out = Vec::new();
        for _ in 0..tries * 6 {
            if out.len() >= tries {
                break;
            }
            let c = near[((rng.f() * near.len() as f32) as usize).min(near.len() - 1)];
            let (a, r) = (rng.f() * std::f32::consts::TAU, rng.f().sqrt() * 22.0);
            let (x, z) = (c.x + a.cos() * r, c.z + a.sin() * r);
            let Some(y) = ground.nearest(x, z, c.y, 6.0) else { continue };
            let p = Vec3::new(x, y, z);
            if open_ground(ground, p) {
                out.push(p);
            }
        }
        if !out.is_empty() {
            return out;
        }
    }
    let total: f32 = faces.iter().map(|f| f.1).sum();
    if total <= 0.0 {
        return Vec::new();
    }
    // Prefix sums for the weighted pick.
    let mut acc = Vec::with_capacity(faces.len());
    let mut s = 0.0;
    for f in faces {
        s += f.1;
        acc.push(s);
    }
    let mut rng = Rng(seed | 1);
    let mut out = Vec::new();
    for _ in 0..tries * 4 {
        if out.len() >= tries {
            break;
        }
        let k = acc.partition_point(|&a| a < rng.f() * total).min(faces.len() - 1);
        let c = faces[k].0;
        let Some(y) = ground.settle(c.x, c.z, c.y) else { continue };
        let p = Vec3::new(c.x, y, c.z);
        if open_ground(ground, p) {
            out.push(p);
        }
    }
    // A zone that is all under cover (a dungeon, a building): any walkable floor.
    if out.is_empty() {
        for _ in 0..tries * 2 {
            if out.len() >= tries {
                break;
            }
            let k = acc.partition_point(|&a| a < rng.f() * total).min(faces.len() - 1);
            let c = faces[k].0;
            let Some(y) = ground.settle(c.x, c.z, c.y) else { continue };
            let p = Vec3::new(c.x, y, c.z);
            if (1..=7).contains(&ground.surface(p).0) && ground.raycast(p - Vec3::Y * 0.3, p - Vec3::Y * 3.0).is_none()
            {
                out.push(p);
            }
        }
    }
    out
}

/// How good a walk scene is (and the side its camera works best from); None when it does not work at all.
pub fn judge_walk(
    ground: &Ground,
    occ: &Scenery,
    start: Vec3,
    dir: Vec3,
    len: f32,
    rays: (usize, usize),
) -> Option<(f32, f32)> {
    let path = walk_points(ground, start, dir, len);
    let end = *path.last()?;
    let reach = xz(end - start).length();
    if reach < len * 0.8 {
        return None;
    }
    // Nothing the walk would pass through (scenery without collision: rocks, trunks).
    if !path.iter().step_by(2).all(|p| occ.room_for(*p, NOMINAL_HEIGHT)) {
        return None;
    }
    // Level enough to read as a stroll, no steep climb.
    if (end.y - start.y).abs() > reach * 0.35 {
        return None;
    }
    let h = NOMINAL_HEIGHT;
    let d = frame_distance(h, FOCAL, FILL);
    let mut best: Option<(f32, f32)> = None;
    for side in [1.0f32, -1.0] {
        let front = turn(dir, FRONT_DEG * side);
        // Room for the camera along the walk (start, middle, end), and at the end's arc to the side.
        let mut room = 0.0;
        for k in [0, path.len() / 2, path.len() - 1] {
            let p = path[k];
            let (eye, _) = eye_target(p, h, front, d);
            let eye = above_ground(ground, eye);
            let seen = body_visible(ground, occ, eye, p, h);
            // The opening shot (the walk's start) must show the whole body.
            if k == 0 && !seen {
                room = -10.0;
            }
            room += clear(ground, p - Vec3::Y * (h * 0.62), eye) * if seen { 1.0 } else { 0.2 };
        }
        let (eye_s, _) = eye_target(end, h, turn(dir, 80.0 * side), d);
        let eye_s = above_ground(ground, eye_s);
        room += 0.5
            * clear(ground, end - Vec3::Y * (h * 0.62), eye_s)
            * if body_visible(ground, occ, eye_s, end, h) { 1.0 } else { 0.3 };
        let room = room / 3.5;
        if room < 0.75 {
            continue;
        }
        // The opening shot counts most (the start: the whole view behind them), then the walk's end.
        let (e0, t0) = eye_target(start, h, front, d);
        let (e1, t1) = eye_target(end, h, front, d);
        let view = 0.65 * view_score(ground, above_ground(ground, e0), t0, rays, 220.0)
            + 0.35 * view_score(ground, above_ground(ground, e1), t1, rays, 220.0);
        let score = view + room * 0.4;
        if best.is_none_or(|b| score > b.0) {
            best = Some((score, side));
        }
    }
    best
}

/// The zone's best walk scenes: up to `keep`, at least 30 apart. `tries` spots, each in 12 headings.
pub fn scout_walk(
    ground: &Ground,
    zone: (&crate::dat::zone::ZoneScene, &SceneIndex),
    len: f32,
    tries: usize,
    keep: usize,
    seed: u64,
    near: &[Vec3],
) -> Vec<Scene> {
    let faces = floor_faces(ground);
    let rays = if tries > 150 { (14, 8) } else { (9, 5) };
    let mut all = Vec::new();
    for p in candidates(ground, &faces, tries, seed, near) {
        let occ = &zone.1.local(zone.0, p, len + 9.0);
        let mut best: Option<Scene> = None;
        for k in 0..12 {
            let dir = turn(Vec3::X, k as f32 * 30.0);
            if let Some((score, side)) = judge_walk(ground, occ, p, dir, len, rays)
                && best.is_none_or(|b| score > b.score)
            {
                best = Some(Scene { at: p.to_array(), dir: [dir.x, dir.z], len, side, score, front: None });
            }
        }
        all.extend(best);
    }
    pick(all, keep)
}

/// Fishing at `at` facing `dir` (the water): the front view (over the water, back at them and the shore) and the
/// view over their shoulder out over the water.
pub fn judge_fish(ground: &Ground, occ: &Scenery, at: Vec3, dir: Vec3, rays: (usize, usize)) -> Option<(f32, f32)> {
    let h = NOMINAL_HEIGHT;
    let d = frame_distance(h, FOCAL, FILL);
    let chest = at - Vec3::Y * (h * 0.62);
    if !occ.room_for(at, h) {
        return None;
    }
    let mut best: Option<(f32, f32)> = None;
    for side in [1.0f32, -1.0] {
        let front = turn(dir, FRONT_DEG * side);
        let back = turn(dir, 160.0 * side);
        let (ef, tf) = eye_target(at, h, front, d);
        let (eb, tb) = eye_target(at, h, back, d);
        let (ef, eb) = (above_ground(ground, ef), above_ground(ground, eb));
        if !body_visible(ground, occ, ef, at, h) {
            continue;
        }
        let seen = |e: Vec3| if body_visible(ground, occ, e, at, h) { 1.0 } else { 0.2 };
        let room = (clear(ground, chest, ef) * seen(ef) + clear(ground, chest, eb) * seen(eb)) / 2.0;
        if room < 0.7 {
            continue;
        }
        let score =
            0.6 * view_score(ground, ef, tf, rays, 220.0) + 0.4 * view_score(ground, eb, tb, rays, 220.0) + room * 0.3;
        if best.is_none_or(|b| score > b.0) {
            best = Some((score, side));
        }
    }
    best
}

/// The zone's best fishing scenes: open ground with water ahead.
pub fn scout_fish(
    ground: &Ground,
    zone: (&crate::dat::zone::ZoneScene, &SceneIndex),
    tries: usize,
    keep: usize,
    seed: u64,
    near: &[Vec3],
) -> Vec<Scene> {
    let faces = floor_faces(ground);
    let rays = if tries > 150 { (14, 8) } else { (9, 5) };
    let mut all = Vec::new();
    for p in candidates(ground, &faces, tries * 3, seed ^ 0x5eed, near) {
        // Water ahead first (cheap), then the scenery.
        if !(0..16).any(|k| ground.water_ahead(p, turn(Vec3::X, k as f32 * 22.5))) {
            continue;
        }
        let occ = &zone.1.local(zone.0, p, 9.0);
        let mut best: Option<Scene> = None;
        for k in 0..16 {
            let dir = turn(Vec3::X, k as f32 * 22.5);
            if !ground.water_ahead(p, dir) {
                continue;
            }
            if let Some((score, side)) = judge_fish(ground, occ, p, dir, rays)
                && best.is_none_or(|b| score > b.score)
            {
                best = Some(Scene { at: p.to_array(), dir: [dir.x, dir.z], len: 0.0, side, score, front: None });
            }
        }
        all.extend(best);
    }
    // Weak fishing spots are puddles and fountains: none rather than those.
    all.retain(|s| s.score > 0.25);
    pick(all, keep)
}

/// Best first, each at least 30 from the ones kept before it.
fn pick(mut all: Vec<Scene>, keep: usize) -> Vec<Scene> {
    all.sort_by(|a, b| b.score.total_cmp(&a.score));
    let mut out: Vec<Scene> = Vec::new();
    for s in all {
        if out.len() >= keep {
            break;
        }
        if out.iter().all(|o| (o.at() - s.at()).length() > 30.0) {
            out.push(Scene { score: (s.score * 1000.0).round() / 1000.0, ..s });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn framing_round_trips() {
        let d = frame_distance(1.9, 520.0, 0.52);
        assert!((frame_focal(1.9, d, 0.52) - 520.0).abs() < 1e-3);
        // A Hume at 520 stands about 5 away.
        assert!(d > 4.0 && d < 6.0);
    }

    #[test]
    fn the_body_fits_the_frame() {
        // Eye and target as the vignette frames them: the head and the feet both inside the frame's height.
        let h = 1.9;
        let d = frame_distance(h, FOCAL, FILL);
        let (eye, target) = eye_target(Vec3::ZERO, h, Vec3::X, d);
        let fwd = (target - eye).normalize();
        let ty = 192.0 / FOCAL;
        for p in [Vec3::ZERO, -Vec3::Y * h] {
            let v = p - eye;
            let depth = v.dot(fwd);
            let up = -(v - fwd * depth).y / depth;
            assert!(up.abs() < ty * 0.97, "{p:?}: {up} of {ty}");
        }
    }

    #[test]
    fn scenes_read_back() {
        let s = Scene { at: [1.0, 2.0, 3.0], dir: [0.0, 1.0], len: 12.0, side: -1.0, score: 1.5, front: Some(30.0) };
        let z = ZoneScenes { walk: vec![s], fish: Vec::new() };
        let j = serde_json::to_string(&z).unwrap();
        let back: ZoneScenes = serde_json::from_str(&j).unwrap();
        assert_eq!(back.walk[0], s);
    }
}
