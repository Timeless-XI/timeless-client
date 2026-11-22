//! Timeless XI: the Select Character screen's ring (owner, 2026-11-22). Interface = Timeless shows the account's
//! characters standing on the round platform in the middle of the Hall of the Gods' statue room, facing a camera
//! mounted at the platform's centre, with the robed statues always behind them. Moving between characters swings the
//! camera round to the one highlighted, who stands in a slight warm aura; the others wait in cool shadow.
//!
//! Placement (the server allows three characters, owner 2026-11-22):
//! - one: in front of Altana (+z);
//! - two: opposite each other, the first before Altana, the second before the chained angel (−z);
//! - three: the first before Altana, the other two spaced equally (a triangle: 120° apart).
//!
//! The hour and weather are the title reel's for the hall (20:00, fair), under the title screen's filter.

use super::super::ground::Ground;
use super::super::select::{FxLibs, LobbyFx, Target};
use crate::dat::character::{Animation, Character as Model, Look, PoseLayer, load_character};
use crate::dat::weather::{Lighting, ZoneWeather};
use crate::dat::{DatIndex, zone};
use crate::motionq::{Enq, Queues};
use crate::render::ui::{UiBatch, UiRenderer};
use crate::render::{ActorGpu, Camera, Gpu, ZoneRenderer};
use anyhow::{Context, Result};
use glam::{Mat4, Vec3, Vec4};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, channel};

/// The Hall of the Gods.
pub const ZONE: u16 = 251;
/// The ring's effect voices (select.rs keeps the other lobby scenes' bases).
pub const VOICES: u32 = 0x0500_0000;
/// The ring's radius (inside the platform's rose-window circle, about 6.5), the camera's eye above the floor, its
/// target's height 10 out, and the focal (dat::title fov_y). `VANA_RING=radius,eye,target,focal` tries others.
#[derive(Clone, Copy)]
struct Framing {
    radius: f32,
    eye_up: f32,
    target_up: f32,
    focal: f32,
}

fn framing() -> Framing {
    let d = Framing { radius: 3.8, eye_up: 0.9, target_up: 2.6, focal: 320.0 };
    let Some(v) = std::env::var("VANA_RING").ok() else { return d };
    let v: Vec<f32> = v.split(',').filter_map(|x| x.trim().parse().ok()).collect();
    match v[..] {
        [radius, eye_up, target_up, focal] => Framing { radius, eye_up, target_up, focal },
        _ => d,
    }
}
/// The hall's hour and weather (the title reel's hall_of_the_gods shot).
const MINUTE: f32 = 20.0 * 60.0;
const WEATHER: &str = "suny";
/// The platform's floor, should the collision not answer.
const FLOOR: f32 = -13.1;
const GLOW: &str = "tl_ring_glow";

/// Where each character stands (degrees from +x toward +z), by how many there are.
pub fn angles(n: usize) -> Vec<f32> {
    match n {
        0 => Vec::new(),
        1 => vec![90.0],
        2 => vec![90.0, -90.0],
        3 => vec![90.0, -30.0, 210.0],
        // More than the server allows: evenly round, the first still before Altana.
        n => (0..n).map(|i| 90.0 - 360.0 * i as f32 / n as f32).collect(),
    }
}

fn dir_of(deg: f32) -> Vec3 {
    let r = deg.to_radians();
    Vec3::new(r.cos(), 0.0, r.sin())
}

/// The shortest turn from `from` to `to` (degrees, −180..180).
fn short(from: f32, to: f32) -> f32 {
    (to - from + 540.0).rem_euclid(360.0) - 180.0
}

#[derive(Clone, Debug, PartialEq)]
pub struct Member {
    pub name: String,
    pub look: Look,
    pub size: u8,
}

/// The account's characters in the Select screen's order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Request {
    pub members: Vec<Member>,
}

impl Request {
    pub fn key(&self) -> String {
        self.members.iter().map(|m| format!("{}:{:?}:{};", m.name, m.look, m.size)).collect()
    }
}

struct Body {
    model: Model,
    anims: Vec<Animation>,
    size: u8,
    skirtless: bool,
    /// Posed in its idle, at its size.
    height: f32,
    feet: Vec3,
    facing: Vec3,
    angle: f32,
}

pub struct RingData {
    key: String,
    scene: zone::ZoneScene,
    weather: Option<ZoneWeather>,
    fx: Option<FxLibs>,
    floor: f32,
    bodies: Vec<Body>,
}

impl RingData {
    pub fn load(index: &Arc<DatIndex>, keys: &Arc<zone::Keys>, req: &Request) -> Result<Self> {
        let zone_id = ZONE as u32;
        let file = index.read(zone::file_id(zone_id))?;
        let scene = zone::load_scene(keys, &file)?;
        let ground = Ground::build(&scene);
        let weather = ZoneWeather::parse(&file).ok();
        let fx = FxLibs::load_zone(index, keys, zone_id, &file);
        let floor = ground.settle(0.0, 0.0, FLOOR - 1.0).unwrap_or(FLOOR);
        let mut bodies = Vec::new();
        for (m, angle) in req.members.iter().zip(angles(req.members.len())) {
            let model = load_character(index, &m.look).with_context(|| format!("{}'s model", m.name))?;
            let idle: Vec<_> = model.find_animations("idl").into_iter().map(|a| (a, 0.0)).collect();
            let rest = crate::render::character::pose_character(&model, &idle);
            let height = ((rest.max.y - rest.min.y) * model.scale(m.size)).clamp(0.6, 4.0);
            let at = dir_of(angle) * framing().radius;
            let y = ground.settle(at.x, at.z, floor - 1.0).unwrap_or(floor);
            bodies.push(Body {
                anims: model.animations.clone(),
                skirtless: model.pc.as_ref().is_some_and(|pc| pc.skirt == 0),
                model,
                size: m.size,
                height,
                feet: Vec3::new(at.x, y, at.z),
                facing: -dir_of(angle),
                angle,
            });
        }
        log::info!("select ring: {} characters, floor {floor:.2}", bodies.len());
        Ok(Self { key: req.key(), scene, weather, fx, floor, bodies })
    }
}

/// Facing `forward` (model +x toward it) at `feet`, at the model's size (select.rs placement).
fn placement(feet: Vec3, forward: Vec3, scale: f32) -> Mat4 {
    let f = Vec3::new(forward.x, 0.0, forward.z).normalize_or(Vec3::X);
    Mat4::from_cols(f.extend(0.0), Vec3::Y.extend(0.0), f.cross(Vec3::Y).extend(0.0), feet.extend(1.0))
        * Mat4::from_scale(Vec3::splat(scale))
}

/// A motion's chunks by region: (region, index, frames, speed). A PC body with no skirt takes no region 2 chunk.
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

/// Plays `name` as the status motion (looping), blended in over `blend` ticks.
fn play(q: &mut Queues<usize>, anims: &[Animation], name: &str, skirtless: bool, rate: f32, blend: f32) -> bool {
    let c = chunks(anims, name, skirtless);
    if c.is_empty() {
        return false;
    }
    q.enqueue_motion(c, 0, &Enq::status(rate, blend, true), |_| rate);
    true
}

/// A soft round light (white, the alpha falling off from the middle) for the aura, its pool and the motes.
fn glow_texture() -> (u32, u32, Vec<u8>) {
    let n = 64u32;
    let mut px = Vec::with_capacity((n * n * 4) as usize);
    for y in 0..n {
        for x in 0..n {
            let dx = (x as f32 + 0.5) / n as f32 * 2.0 - 1.0;
            let dy = (y as f32 + 0.5) / n as f32 * 2.0 - 1.0;
            let a = (-(dx * dx + dy * dy) * 3.2).exp() * (1.0 - (dx * dx + dy * dy).min(1.0));
            px.extend_from_slice(&[255, 255, 255, (a * 255.0) as u8]);
        }
    }
    (n, n, px)
}

/// The ring as shown: the hall, the characters, the camera at the centre.
pub struct RingScene {
    data: RingData,
    zone: Option<ZoneRenderer>,
    actors: Vec<Option<ActorGpu>>,
    queues: Vec<Queues<usize>>,
    effects: Option<LobbyFx>,
    ui: Option<UiRenderer>,
    glow_loaded: bool,
    nostalgia: Option<crate::render::nostalgia::Nostalgia>,
    grade: Option<crate::render::nostalgia::Grade>,
    /// The highlighted character, and each one's aura 0..1 (eased).
    focus: usize,
    aura: Vec<f32>,
    /// The camera's heading (degrees, the way it looks from the centre).
    yaw: f32,
    time: f32,
    /// 0..1: fades in when it starts and out when another roster is wanted.
    pub alpha: f32,
    pub leaving: bool,
}

impl RingScene {
    pub fn new(mut data: RingData, focus: usize) -> Self {
        let effects = data.fx.take().map(|l| {
            let mut fx = LobbyFx::new(l, MINUTE, VOICES);
            fx.set_weather(WEATHER);
            fx
        });
        let n = data.bodies.len();
        let focus = focus.min(n.saturating_sub(1));
        let mut queues: Vec<Queues<usize>> = (0..n).map(|_| Queues::default()).collect();
        for (q, b) in queues.iter_mut().zip(&data.bodies) {
            play(q, &b.anims, "idl", b.skirtless, 1.0, 1.0);
        }
        let yaw = data.bodies.get(focus).map_or(90.0, |b| b.angle);
        let aura = (0..n).map(|i| if i == focus { 1.0 } else { 0.0 }).collect();
        Self {
            actors: (0..n).map(|_| None).collect(),
            queues,
            data,
            zone: None,
            effects,
            ui: None,
            glow_loaded: false,
            nostalgia: None,
            grade: super::title::filter_preset(&super::title::Reel::load().filter),
            focus,
            aura,
            yaw,
            time: 0.0,
            alpha: 0.0,
            leaving: false,
        }
    }

    pub fn key(&self) -> &str {
        &self.data.key
    }

    pub fn set_focus(&mut self, i: usize) {
        self.focus = i.min(self.data.bodies.len().saturating_sub(1));
    }

    pub fn advance(&mut self, dt: f32) {
        let dt = dt.min(0.1);
        self.time += dt;
        let to = if self.leaving { 0.0 } else { 1.0 };
        let rate = if self.leaving { 2.5 } else { 0.9 };
        self.alpha = if to > self.alpha { (self.alpha + dt * rate).min(to) } else { (self.alpha - dt * rate).max(to) };
        if let Some(fx) = &mut self.effects {
            fx.advance(dt);
        }
        // The swing: eased on the shortest way round (about a second to settle).
        if let Some(b) = self.data.bodies.get(self.focus) {
            let d = short(self.yaw, b.angle);
            self.yaw += d * (1.0 - (-dt / 0.32).exp());
        }
        // The aura gathers on the one in view (once the camera is most of the way there) and leaves the others.
        let settled = self.data.bodies.get(self.focus).is_none_or(|b| short(self.yaw, b.angle).abs() < 20.0);
        for (i, a) in self.aura.iter_mut().enumerate() {
            let want = if i == self.focus && settled { 1.0 } else { 0.0 };
            let secs = if want > *a { 0.45 } else { 0.25 };
            *a += (want - *a) * (1.0 - (-dt / secs).exp());
        }
        for q in &mut self.queues {
            q.tick(dt * 60.0);
        }
    }

    /// The camera at the centre, looking out at the heading, with a slow breath of drift.
    fn camera(&self) -> Camera {
        let floor = self.data.floor;
        let f = framing();
        let yaw = self.yaw + (self.time * 0.21).sin() * 0.8;
        let lift = (self.time * 0.33).sin() * 0.04;
        let eye = Vec3::new(0.0, floor - f.eye_up - lift, 0.0);
        let target = dir_of(yaw) * 10.0 + Vec3::Y * (floor - f.target_up - lift);
        Camera { eye, target, up: Vec3::NEG_Y, fov_y: crate::dat::title::fov_y(f.focal) }
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

    /// The aura over the highlighted character: a soft warm light about the body, a pool at the feet, a faint shaft
    /// from above and a few motes rising (additive, under the filter so it blooms with it).
    fn aura_batch(&self, cam: &Camera, size: [u32; 2], batch: &mut UiBatch) {
        let (w, h) = (size[0] as f32, size[1] as f32);
        let vp = cam.view_proj(w / h);
        let to_px = |p: Vec3| -> Option<(f32, f32)> {
            let c = vp * Vec4::new(p.x, p.y, p.z, 1.0);
            (c.w > 0.1).then(|| ((c.x / c.w * 0.5 + 0.5) * w, (0.5 - c.y / c.w * 0.5) * h))
        };
        for (i, (b, &a)) in self.data.bodies.iter().zip(&self.aura).enumerate() {
            if a < 0.01 {
                continue;
            }
            let height = b.height;
            let (Some((fx, fy)), Some((_, hy))) = (to_px(b.feet), to_px(b.feet - Vec3::Y * height)) else { continue };
            let hp = (fy - hy).max(1.0);
            let warm = |k: f32| [1.0, 0.82, 0.58, k * a];
            // A soft halo about the body (faint: the body stays clear; the tint lights it).
            batch.push_image_add(GLOW, fx - hp * 0.55, hy - hp * 0.3, hp * 1.1, hp * 1.5, warm(0.18));
            // The pool at the feet.
            batch.push_image_add(GLOW, fx - hp * 0.62, fy - hp * 0.09, hp * 1.24, hp * 0.18, warm(0.42));
            // The shaft from above.
            batch.push_image_add(GLOW, fx - hp * 0.3, fy - hp * 3.2, hp * 0.6, hp * 3.3, warm(0.07));
            // Motes, rising and fading.
            for k in 0..14 {
                let seed = (i * 31 + k * 7) as f32;
                let phase = (self.time * (0.10 + 0.03 * (seed * 1.7).sin().abs()) + seed * 0.618).fract();
                let x = fx + (seed * 12.9898).sin() * hp * 0.32 + (self.time * 0.7 + seed).sin() * hp * 0.03;
                let y = fy - phase * hp * 1.25;
                let s = hp * (0.018 + 0.012 * (seed * 3.1).cos().abs());
                let fade = (phase * std::f32::consts::PI).sin();
                batch.push_image_add(GLOW, x - s, y - s, s * 2.0, s * 2.0, [1.0, 0.9, 0.7, 0.7 * a * fade]);
            }
        }
    }

    /// The hall, the characters, the hall's effects, the aura, the fade, then the title's filter.
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
    ) {
        let (view, tex, size) = target;
        let cam = self.camera();
        let env = self.data.weather.as_ref().and_then(|w| w.at(WEATHER, MINUTE));
        let lighting = env.map(|e| {
            let k = crate::dat::weather::clipping(e.draw_dist, 1.0, ZONE).1;
            Lighting::from_env_k(&e, MINUTE, k)
        });
        let reach = crate::dat::weather::clipping(env.map_or(100.0, |e| e.draw_dist), 1.0, ZONE).0;
        let far = (draw_distance != 100).then_some(reach).filter(|d| *d > 0.0);
        let zone = self.zone.get_or_insert_with(|| ZoneRenderer::new(gpu, &self.data.scene, format));
        zone.lighting = lighting;
        for (i, b) in self.data.bodies.iter().enumerate() {
            let actor = self.actors[i].get_or_insert_with(|| zone.new_actor(gpu, &b.model));
            let mut layers = Vec::new();
            let over = self.queues[i].layers(&mut layers);
            let pl: Vec<PoseLayer> = layers
                .iter()
                .map(|l| PoseLayer {
                    anim: &b.anims[l.anim],
                    frame: l.frame,
                    weight: l.weight,
                    start: l.start,
                    over: l.over,
                })
                .collect();
            let mesh = crate::render::character::pose_character_layers(&b.model, &pl, over);
            zone.update_actor(gpu, actor, &mesh, placement(b.feet, b.facing, b.model.scale(b.size)));
            // Warm and a little brighter in focus; the others in cool shadow.
            let a = self.aura[i];
            let (dim, lit) = ([0.62, 0.66, 0.78], [1.3, 1.2, 1.04]);
            actor.tint = [0, 1, 2].map(|c| dim[c] + (lit[c] - dim[c]) * a);
        }
        // All the hall's scenery (list 0), as the title reel's `all_scenery`.
        zone.set_camera_list(gpu, 0, far);
        let actors: Vec<&ActorGpu> = self.actors.iter().flatten().collect();
        zone.render(gpu, encoder, view, depth, &cam, aspect, &actors);
        if let Some(fx) = &mut self.effects {
            fx.draw(gpu, format, encoder, (view, tex, size), depth, zone, &cam, aspect, alive);
        }
        let mut batch = UiBatch::default();
        self.aura_batch(&cam, size, &mut batch);
        let ui = self.ui.get_or_insert_with(|| UiRenderer::new(gpu, format));
        if !self.glow_loaded {
            let (w, h, px) = glow_texture();
            ui.load_image(gpu, GLOW, w, h, &px);
            self.glow_loaded = true;
        }
        let dim = 1.0 - self.alpha.clamp(0.0, 1.0);
        if dim > 0.0 {
            batch.push_rect(0.0, 0.0, size[0] as f32, size[1] as f32, [0.0, 0.0, 0.0, dim * dim * (3.0 - 2.0 * dim)]);
        }
        ui.render(gpu, encoder, view, size, size, None, &batch);
        if let (Some(g), Some(t)) = (self.grade.as_ref(), tex) {
            let n = self.nostalgia.get_or_insert_with(|| crate::render::nostalgia::Nostalgia::new(gpu, format));
            n.draw(gpu, encoder, view, t, size, g, [0.0; 3]);
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------
// The stage: the ring shown, a new roster loading.
// ---------------------------------------------------------------------------------------------------------------

#[derive(Default)]
pub struct Ring {
    current: Option<RingScene>,
    next: Option<RingScene>,
    loading: Option<(String, Receiver<Result<RingData>>)>,
    wanted: String,
    failed: String,
    focus: usize,
    sounds: Vec<crate::fx::sound::SoundCmd>,
}

impl Ring {
    /// The Select screen can fade in: the ring is up, or it failed.
    pub fn ready(&self) -> bool {
        self.current.is_some() || (!self.failed.is_empty() && self.failed == self.wanted)
    }

    pub fn wanted(&self) -> &str {
        &self.wanted
    }

    /// The highlighted character (the Select screen's cursor).
    pub fn focus(&mut self, i: usize) {
        self.focus = i;
        if let Some(c) = &mut self.current {
            c.set_focus(i);
        }
    }

    /// `req`: the account's characters. A new roster loads while the shown one stays.
    pub fn want(&mut self, req: Request, data: impl FnOnce() -> Option<(Arc<DatIndex>, Arc<zone::Keys>)>) {
        let key = req.key();
        if key == self.wanted {
            return;
        }
        self.wanted = key.clone();
        if let Some(c) = &mut self.current {
            c.leaving = c.key() != key;
        }
        if self.current.as_ref().is_some_and(|c| c.key() == key) || self.failed == key {
            self.next = None;
            return;
        }
        let Some((index, keys)) = data() else { return };
        let (tx, rx) = channel();
        self.loading = Some((key, rx));
        std::thread::spawn(move || {
            let _ = tx.send(RingData::load(&index, &keys, &req));
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
                    let s = RingScene::new(d, self.focus);
                    if self.current.is_none() {
                        self.current = Some(s);
                    } else {
                        self.next = Some(s);
                    }
                }
                Ok(_) => {}
                Err(e) => {
                    log::warn!("select ring: {e:#}");
                    self.failed = key;
                }
            }
        }
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
    ) -> bool {
        let Some(c) = &mut self.current else { return false };
        c.render(gpu, format, encoder, target, depth, aspect, alive, draw_distance);
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

    /// Off the Select screen: everything goes (the stops for its voices returned).
    pub fn clear(&mut self) -> Vec<crate::fx::sound::SoundCmd> {
        let mut out = std::mem::take(&mut self.sounds);
        if self.current.is_none() && self.next.is_none() && self.loading.is_none() && self.wanted.is_empty() {
            return out;
        }
        for s in [self.current.take(), self.next.take()].into_iter().flatten() {
            let mut s = s;
            out.extend(s.stop_sounds());
        }
        self.loading = None;
        self.wanted.clear();
        self.failed.clear();
        out
    }
}

/// `vana-client ring-shot`: the ring played offscreen, a PNG at each step: the focus moves to `focus[k]` at
/// `at[k]` seconds; a frame is written at each `shots` time.
pub fn shot(
    req: &Request,
    moves: &[(f32, usize)],
    shots: &[f32],
    out_dir: &std::path::Path,
    size: [u32; 2],
) -> Result<()> {
    let cfg = crate::config::Config::load();
    let index = Arc::new(DatIndex::open(&cfg.ffxi_dir())?);
    let keys = Arc::new(zone::Keys::from_ffximain(&std::fs::read(crate::dat::ffximain(&cfg.ffxi_dir()))?)?);
    let data = RingData::load(&index, &keys, req)?;
    for b in &data.bodies {
        println!("at {:.0} deg: feet {:.2} {:.2} {:.2}, {:.2} tall", b.angle, b.feet.x, b.feet.y, b.feet.z, b.height);
    }
    let mut s = RingScene::new(data, moves.first().map_or(0, |m| m.1));
    s.effects = None;
    let gpu = Gpu::headless()?;
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let [w, h] = size;
    let target = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ring"),
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
    let mut shots = shots.to_vec();
    shots.sort_by(f32::total_cmp);
    for want in shots {
        while t + dt <= want {
            if let Some((_, f)) = moves.iter().rev().find(|(at, _)| *at <= t) {
                s.set_focus(*f);
            }
            s.advance(dt);
            t += dt;
        }
        let mut enc = gpu.device.create_command_encoder(&Default::default());
        s.render(&gpu, format, &mut enc, (&view, Some(&target), size), &depth, w as f32 / h as f32, None, 100);
        let readback = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ring readback"),
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
        let path = out_dir.join(format!("ring_{:05.1}.png", want));
        let f = std::fs::File::create(&path)?;
        let mut e = png::Encoder::new(std::io::BufWriter::new(f), w, h);
        e.set_color(png::ColorType::Rgba);
        e.set_depth(png::BitDepth::Eight);
        e.write_header()?.write_image_data(&img)?;
        println!("{}: yaw {:.1}, focus {}", path.display(), s.yaw, s.focus);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn places_by_count() {
        assert_eq!(angles(1), vec![90.0]);
        // Two: before Altana (+z) and opposite, before the chained angel (-z).
        let two = angles(2);
        assert!((dir_of(two[0]) - Vec3::Z).length() < 1e-5 && (dir_of(two[1]) + Vec3::Z).length() < 1e-5);
        // Three: a triangle, the first before Altana.
        let three = angles(3);
        assert_eq!(three[0], 90.0);
        for k in 0..3 {
            assert!((short(three[k], three[(k + 1) % 3]).abs() - 120.0).abs() < 1e-3);
        }
    }

    #[test]
    fn swings_the_short_way() {
        assert_eq!(short(90.0, -30.0), -120.0);
        assert_eq!(short(-30.0, 210.0), -120.0);
        assert_eq!(short(210.0, 90.0), -120.0);
        assert_eq!(short(170.0, -170.0), 20.0);
    }
}
