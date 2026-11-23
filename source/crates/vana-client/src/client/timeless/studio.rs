//! Timeless XI: the Create Character screen's stage (owner, 2026-11-23, concept A "Hall of Origins"). The new
//! character stands alone on the Hall of the Gods' round platform, before Altana, where the Select screen's first
//! character stands (client/timeless/ring.rs), with a fine ring of crystal light at their feet. Every choice shows on
//! them at once: a new race, style or size swaps the model where they stand (the ring flares), and the colours are the
//! custom look (client/timeless/looks.rs) drawn on the model as it will be in game.
//!
//! The filter is the title's at its very faintest (owner: "players can actually see pretty well what their character
//! will look like in game"), and the model takes no tint. The camera frames the whole body, or moves in to the face
//! (the Likeness and Colors steps; the wheel, F); dragging turns the character.

use super::super::ground::Ground;
use super::super::select::{FxLibs, LobbyFx, Target};
use super::looks::{ActorLook, Settings};
use super::ring::{FLOOR, GLOW, MINUTE, RING, WEATHER, dir_of, framing, glow_texture, placement, play, ring_texture};
use crate::dat::character::{Animation, Character as Model, Look, PoseLayer, load_character};
use crate::dat::weather::{Lighting, ZoneWeather};
use crate::dat::{DatIndex, zone};
use crate::motionq::Queues;
use crate::render::nostalgia::Grade;
use crate::render::ui::{UiBatch, UiRenderer};
use crate::render::{ActorGpu, Camera, Gpu, LineVertex, ZoneRenderer};
use anyhow::{Context, Result};
use glam::{Vec3, Vec4};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, channel};

/// Before Altana (+z), as the Select screen's first character.
const ANGLE: f32 = 90.0;
/// The stage's effect voices (the ring's are 0x0500_0000).
const VOICES: u32 = 0x0540_0000;
/// How much of the title's filter shows (owner: "only the very faintest filter").
const FILTER: f32 = 0.2;

/// The character to show: their model's look (the starting clothes) and size.
#[derive(Clone, Debug, PartialEq)]
pub struct Want {
    pub look: Look,
    pub size: u8,
}

pub struct Hall {
    scene: zone::ZoneScene,
    weather: Option<ZoneWeather>,
    fx: Option<FxLibs>,
    feet: Vec3,
}

impl Hall {
    pub fn load(index: &Arc<DatIndex>, keys: &Arc<zone::Keys>) -> Result<Self> {
        let zone_id = super::ring::ZONE as u32;
        let file = index.read(zone::file_id(zone_id))?;
        let scene = zone::load_scene(keys, &file)?;
        let ground = Ground::build(&scene);
        let weather = ZoneWeather::parse(&file).ok();
        let fx = FxLibs::load_zone(index, keys, zone_id, &file);
        let floor = ground.settle(0.0, 0.0, FLOOR - 1.0).unwrap_or(FLOOR);
        let at = dir_of(ANGLE) * framing().radius;
        let y = ground.settle(at.x, at.z, floor - 1.0).unwrap_or(floor);
        Ok(Self { scene, weather, fx, feet: Vec3::new(at.x, y, at.z) })
    }
}

pub struct Body {
    want: Want,
    model: Model,
    anims: Vec<Animation>,
    skirtless: bool,
    height: f32,
}

impl Body {
    pub fn load(index: &DatIndex, want: &Want) -> Result<Self> {
        let model = load_character(index, &want.look).context("the new character's model")?;
        let idle: Vec<_> = model.find_animations("idl").into_iter().map(|a| (a, 0.0)).collect();
        let rest = crate::render::character::pose_character(&model, &idle);
        let height = ((rest.max.y - rest.min.y) * model.scale(want.size)).clamp(0.6, 4.0);
        Ok(Self {
            want: want.clone(),
            anims: model.animations.clone(),
            skirtless: model.pc.as_ref().is_some_and(|pc| pc.skirt == 0),
            model,
            height,
        })
    }
}

/// The title's grade at `k` of its strength (0: none at all).
fn faint(g: &Grade, k: f32) -> Grade {
    let s = |v: [f32; 3]| v.map(|x| x * k);
    Grade {
        lift: s(g.lift),
        shadow_tint: s(g.shadow_tint),
        high_tint: s(g.high_tint),
        saturation: 1.0 + (g.saturation - 1.0) * k,
        contrast: 1.0 + (g.contrast - 1.0) * k,
        halation: g.halation * k,
        halation_tint: g.halation_tint,
        vignette: g.vignette * k,
        grain: g.grain * k,
        soft_edges: g.soft_edges * k,
        weave: 0.0,
        flicker: 0.0,
    }
}

fn ease(from: &mut f32, to: f32, dt: f32, secs: f32) {
    *from += (to - *from) * (1.0 - (-dt / secs).exp());
}

/// The camera's framing at zoom `z` (0 the whole body .. 1 the face) for one `h` tall: (lens, span, mid, eye), the
/// last three in fractions of the height (as ring.rs's close-up). Tarutaru's faces are half their height.
/// Below 0, out past the whole body to -1 (the platform round them).
fn framing_at(z: f32, taru: bool) -> (f32, f32, f32, f32) {
    let far = (300.0, 2.6, 0.45, 0.55);
    let full = (420.0, 1.45, 0.5, 0.6);
    let face = if taru { (900.0, 0.56, 0.66, 0.68) } else { (900.0, 0.42, 0.85, 0.86) };
    let (a, b, t) = if z < 0.0 { (full, far, -z.max(-1.0)) } else { (full, face, z.min(1.0)) };
    let k = t * t * (3.0 - 2.0 * t);
    let l = |x: f32, y: f32| x + (y - x) * k;
    (l(a.0, b.0), l(a.1, b.1), l(a.2, b.2), l(a.3, b.3))
}

/// The floor ring as triangles in the world, drawn against the scene's depth so the character's legs stand in front
/// of its far side: soft bands of light (`profile`: radius as a fraction of `r`, strength) round `centre`, lifted
/// off the floor a hair. Colours are gamma-space RGB with alpha.
fn ring_tris(centre: Vec3, r: f32, profile: &[(f32, f32)], colour: [f32; 3], alpha: f32) -> Vec<LineVertex> {
    const SEGMENTS: usize = 72;
    let lift = centre - Vec3::Y * 0.012;
    let at = |k: usize, f: f32| lift + dir_of(k as f32 * 360.0 / SEGMENTS as f32) * (r * f);
    let mut out = Vec::with_capacity(SEGMENTS * profile.len() * 6);
    let v = |p: Vec3, a: f32| LineVertex {
        pos: p.to_array(),
        color: [colour[0], colour[1], colour[2], (a * alpha).clamp(0.0, 1.0)],
    };
    for band in profile.windows(2) {
        let ((f0, a0), (f1, a1)) = (band[0], band[1]);
        for k in 0..SEGMENTS {
            let (p00, p01) = (at(k, f0), at(k + 1, f0));
            let (p10, p11) = (at(k, f1), at(k + 1, f1));
            if f0 <= 0.0 {
                out.extend([v(lift, a0), v(p10, a1), v(p11, a1)]);
            } else {
                out.extend([v(p00, a0), v(p10, a1), v(p11, a1), v(p00, a0), v(p11, a1), v(p01, a0)]);
            }
        }
    }
    out
}

/// The stage: the hall (loaded once), the character (a new model loads behind the shown one and swaps in).
#[derive(Default)]
pub struct Studio {
    files: Option<(Arc<DatIndex>, Arc<zone::Keys>)>,
    hall: Option<Hall>,
    hall_loading: Option<Receiver<Result<Hall>>>,
    hall_failed: bool,
    body: Option<Body>,
    body_loading: Option<(Want, Receiver<Result<Body>>)>,
    wanted: Option<Want>,
    zone: Option<ZoneRenderer>,
    actor: Option<ActorGpu>,
    queue: Queues<usize>,
    effects: Option<LobbyFx>,
    ui: Option<UiRenderer>,
    glow_loaded: bool,
    nostalgia: Option<crate::render::nostalgia::Nostalgia>,
    grade: Option<Grade>,
    look: ActorLook,
    settings: Option<Settings>,
    /// 0 the whole body .. 1 the face: wanted and eased.
    zoom_want: f32,
    zoom: f32,
    /// The character's turn from facing the camera (degrees): wanted and eased.
    turn_want: f32,
    turn: f32,
    /// The camera's height basis (eased to a new model's height).
    cam_h: f32,
    /// 1 as a new model swaps in, falling to 0 (the ring flares).
    swap: f32,
    time: f32,
    alpha: f32,
    sounds: Vec<crate::fx::sound::SoundCmd>,
    step_seen: Option<usize>,
}

impl Studio {
    /// The screen can fade in: the hall and a character are up, or the hall failed.
    pub fn ready(&self) -> bool {
        (self.hall.is_some() && self.body.is_some()) || self.hall_failed
    }

    /// The character wanted; `data`: the game files, asked for only when something must load.
    pub fn want(&mut self, w: Want, data: impl FnOnce() -> Option<(Arc<DatIndex>, Arc<zone::Keys>)>) {
        if self.wanted.as_ref() == Some(&w) && (self.hall.is_some() || self.hall_loading.is_some() || self.hall_failed)
        {
            return;
        }
        if self.files.is_none() {
            self.files = data();
        }
        let Some((index, keys)) = self.files.clone() else { return };
        if self.hall.is_none() && self.hall_loading.is_none() && !self.hall_failed {
            let (tx, rx) = channel();
            let (i, k) = (index.clone(), keys.clone());
            std::thread::spawn(move || {
                let _ = tx.send(Hall::load(&i, &k));
            });
            self.hall_loading = Some(rx);
        }
        if self.wanted.as_ref() != Some(&w) {
            self.wanted = Some(w.clone());
            if self.body.as_ref().is_some_and(|b| b.want == w) {
                self.body_loading = None;
            } else if self.body_loading.as_ref().is_none_or(|l| l.0 != w) {
                let (tx, rx) = channel();
                let w2 = w.clone();
                std::thread::spawn(move || {
                    let _ = tx.send(Body::load(&index, &w2));
                });
                self.body_loading = Some((w, rx));
            }
        }
    }

    /// The custom look on the character (None: their face as it comes).
    pub fn set_look(&mut self, s: Option<Settings>) {
        self.settings = s;
    }

    /// 0 the whole body .. 1 the face.
    pub fn set_zoom(&mut self, z: f32) {
        self.zoom_want = z.clamp(-1.0, 1.0);
    }

    pub fn zoom_wanted(&self) -> f32 {
        self.zoom_want
    }

    /// The screen's step (retail's numbers): a new one frames them for it (Likeness: head and shoulders, Colors: the
    /// face, the others: the whole body) and turns them back to the camera.
    pub fn on_step(&mut self, step: usize) {
        if self.step_seen == Some(step) {
            return;
        }
        self.step_seen = Some(step);
        self.zoom_want = match step {
            1 => 0.55,
            2 => 1.0,
            _ => 0.0,
        };
        self.face_front();
    }

    /// Turns the character by `deg` (a drag).
    pub fn turn_by(&mut self, deg: f32) {
        self.turn_want += deg;
    }

    /// Back to facing the camera, the shortest way round.
    pub fn face_front(&mut self) {
        self.turn -= (self.turn / 360.0).round() * 360.0;
        self.turn_want = 0.0;
    }

    fn take_loads(&mut self) {
        if let Some(rx) = &self.hall_loading
            && let Ok(r) = rx.try_recv()
        {
            self.hall_loading = None;
            match r {
                Ok(h) => self.install_hall(h),
                Err(e) => {
                    log::warn!("create character: the hall: {e:#}");
                    self.hall_failed = true;
                }
            }
        }
        if let Some((_, rx)) = &self.body_loading
            && let Ok(r) = rx.try_recv()
        {
            let (w, _) = self.body_loading.take().unwrap();
            match r {
                Ok(b) if self.wanted.as_ref() == Some(&w) => self.install_body(b),
                Ok(_) => {}
                Err(e) => log::warn!("create character: {e:#}"),
            }
        }
    }

    fn install_hall(&mut self, mut h: Hall) {
        self.effects = h.fx.take().map(|l| {
            let mut fx = LobbyFx::new(l, MINUTE, VOICES);
            fx.set_weather(WEATHER);
            fx
        });
        self.grade = super::title::filter_preset(&super::title::Reel::load().filter).map(|g| faint(&g, FILTER));
        self.zone = None;
        self.actor = None;
        self.hall = Some(h);
    }

    fn install_body(&mut self, b: Body) {
        let first = self.body.is_none();
        self.queue = Queues::default();
        play(&mut self.queue, &b.anims, "idl", b.skirtless, 1.0, 1.0);
        if first {
            self.cam_h = b.height;
        }
        self.body = Some(b);
        self.actor = None;
        self.look.forget();
        self.swap = if first { 0.0 } else { 1.0 };
    }

    pub fn advance(&mut self, dt: f32) {
        let dt = dt.min(0.1);
        self.take_loads();
        self.time += dt;
        let up = self.hall.is_some() && self.body.is_some();
        self.alpha = if up { (self.alpha + dt * 1.2).min(1.0) } else { self.alpha };
        if let Some(fx) = &mut self.effects {
            fx.advance(dt);
        }
        ease(&mut self.zoom, self.zoom_want, dt, 0.3);
        ease(&mut self.turn, self.turn_want, dt, 0.12);
        if let Some(b) = &self.body {
            ease(&mut self.cam_h, b.height, dt, 0.25);
        }
        self.swap = (self.swap - dt * 1.6).max(0.0);
        self.queue.tick(dt * 60.0);
    }

    fn camera(&self) -> Option<Camera> {
        let (hall, b) = (self.hall.as_ref()?, self.body.as_ref()?);
        let h = self.cam_h.max(0.6);
        let taru = matches!(b.want.look.race, 5 | 6);
        let (lens, span, mid, up) = framing_at(self.zoom, taru);
        let lift = (self.time * 0.33).sin() * 0.01 * h;
        let dist = (span * h * 0.5) / (192.0 / lens);
        // A slow breath of drift round them, less as the camera closes in.
        let sway = (self.time * 0.19).sin() * 2.0 * (1.0 - self.zoom * 0.7);
        let d = dir_of(ANGLE + sway);
        let eye = hall.feet - d * dist - Vec3::Y * (up * h + lift);
        let target = hall.feet - Vec3::Y * (mid * h + lift);
        Some(Camera { eye, target, up: Vec3::NEG_Y, fov_y: crate::dat::title::fov_y(lens) })
    }

    /// The ring of crystal light at their feet (ring.rs's, always lit here), a pool of deep blue inside it, three
    /// silver motes going slowly round it; a new model makes it flare.
    /// The ring itself, in the world (see `ring_tris`): its soft core, a pool of deep blue inside it, and as a new
    /// model arrives, a second ring opening out and fading.
    fn ring_world(&self) -> Vec<LineVertex> {
        use std::f32::consts::TAU;
        let (Some(hall), Some(b)) = (self.hall.as_ref(), self.body.as_ref()) else { return Vec::new() };
        let r = 0.55_f32.max(b.height * 0.32);
        let breath = 0.8 + 0.2 * (self.time * TAU / 5.0).sin();
        let flare = self.swap * self.swap;
        let mut out = ring_tris(
            hall.feet,
            r,
            &[(0.0, 0.16), (0.7, 0.1), (0.9, 0.04), (1.25, 0.0)],
            [0.24, 0.37, 0.58],
            1.0 + flare,
        );
        out.extend(ring_tris(
            hall.feet,
            r,
            &[(0.8, 0.0), (0.9, 0.22), (0.97, 0.75), (1.0, 1.0), (1.04, 0.6), (1.12, 0.18), (1.28, 0.0)],
            [0.66, 0.77, 0.92],
            (0.7 * breath + 0.3 * flare).min(1.0),
        ));
        if self.swap > 0.0 {
            let grow = 1.0 + (1.0 - self.swap) * 0.9;
            out.extend(ring_tris(
                hall.feet,
                r * grow,
                &[(0.9, 0.0), (1.0, 1.0), (1.1, 0.0)],
                [0.86, 0.9, 0.96],
                0.55 * self.swap,
            ));
        }
        out
    }

    fn ring_batch(&self, cam: &Camera, size: [u32; 2], batch: &mut UiBatch) {
        use std::f32::consts::TAU;
        let (Some(hall), Some(b)) = (self.hall.as_ref(), self.body.as_ref()) else { return };
        let (w, h) = (size[0] as f32, size[1] as f32);
        let vp = cam.view_proj(w / h);
        let to_px = |p: Vec3| -> Option<(f32, f32)> {
            let c = vp * Vec4::new(p.x, p.y, p.z, 1.0);
            (c.w > 0.1).then(|| ((c.x / c.w * 0.5 + 0.5) * w, (0.5 - c.y / c.w * 0.5) * h))
        };
        const STEEL: [f32; 3] = [0.62, 0.73, 0.88];
        const DEEP: [f32; 3] = [0.24, 0.37, 0.58];
        const SILVER: [f32; 3] = [0.84, 0.88, 0.94];
        let rgba = |c: [f32; 3], k: f32| [c[0], c[1], c[2], k];
        let breath = 0.8 + 0.2 * (self.time * TAU / 5.0).sin();
        let flare = self.swap * self.swap;
        let feet = hall.feet;
        let ring_r = 0.55_f32.max(b.height * 0.32);
        let _ = (breath, DEEP);
        let (Some((fx0, fy)), Some((_, hy))) = (to_px(feet), to_px(feet - Vec3::Y * b.height)) else { return };
        let hp = (fy - hy).abs().max(1.0);
        let near = (cam.eye - feet).length();
        for k in 0..3 {
            let a = self.time * 14.0 + k as f32 * 120.0;
            let p = feet + dir_of(a) * ring_r * 0.84;
            let Some((x, y)) = to_px(p) else { continue };
            // Behind them, where their legs stand: hidden (this overlay has no depth).
            if (cam.eye - p).length() > near && (x - fx0).abs() < hp * 0.16 {
                continue;
            }
            let tw = 0.6 + 0.4 * (self.time * 1.7 + k as f32 * 2.1).sin().abs();
            let sz = (hp * 0.012).clamp(2.0, 6.0) * tw;
            batch.push_image_add(GLOW, x - sz, y - sz, sz * 2.0, sz * 2.0, rgba(SILVER, 0.55 * tw));
        }
        // A faint column of light about them as a new model arrives.
        if flare > 0.01
            && let Some((fx, _)) = to_px(feet)
        {
            batch.push_image_add(GLOW, fx - hp * 0.35, fy - hp * 1.1, hp * 0.7, hp * 1.2, rgba(STEEL, 0.25 * flare));
        }
    }

    /// The hall, the character, the hall's effects, the ring, the fade, the faint filter.
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
        let Some(cam) = self.camera() else { return false };
        let ring = self.ring_world();
        let (Some(hall), Some(b)) = (self.hall.as_ref(), self.body.as_ref()) else { return false };
        let (view, tex, size) = target;
        let env = hall.weather.as_ref().and_then(|w| w.at(WEATHER, MINUTE));
        let lighting = env.map(|e| {
            let k = crate::dat::weather::clipping(e.draw_dist, 1.0, super::ring::ZONE).1;
            Lighting::from_env_k(&e, MINUTE, k)
        });
        let reach = crate::dat::weather::clipping(env.map_or(100.0, |e| e.draw_dist), 1.0, super::ring::ZONE).0;
        let far = (draw_distance != 100).then_some(reach).filter(|d| *d > 0.0);
        let zone = self.zone.get_or_insert_with(|| ZoneRenderer::new(gpu, &hall.scene, format));
        zone.lighting = lighting;
        let actor = self.actor.get_or_insert_with(|| zone.new_actor(gpu, &b.model));
        let mut layers = Vec::new();
        let over = self.queue.layers(&mut layers);
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
        let facing = dir_of(ANGLE + 180.0 + self.turn);
        zone.update_actor(gpu, actor, &mesh, placement(hall.feet, facing, b.model.scale(b.want.size)));
        // No tint: the character as the game draws them (`VANA_STUDIO_TINT` tries a lift).
        let tint = std::env::var("VANA_STUDIO_TINT").ok().and_then(|v| v.parse::<f32>().ok()).unwrap_or(1.0);
        actor.tint = [tint; 3];
        match &self.settings {
            Some(s) => self.look.update(gpu, zone, actor, &b.model, b.want.look.race, b.want.look.face, s),
            None => self.look.forget(),
        }
        zone.set_camera_list(gpu, 0, far);
        let actors: Vec<&ActorGpu> = self.actor.iter().collect();
        zone.render(gpu, encoder, view, depth, &cam, aspect, &actors);
        zone.set_world_shapes(gpu, [Vec::new(), ring]);
        zone.render_world_shapes(encoder, view, depth);
        if let Some(fx) = &mut self.effects {
            fx.draw(gpu, format, encoder, (view, tex, size), depth, zone, &cam, aspect, alive);
        }
        let mut batch = UiBatch::default();
        self.ring_batch(&cam, size, &mut batch);
        let ui = self.ui.get_or_insert_with(|| UiRenderer::new(gpu, format));
        if !self.glow_loaded {
            let (w, h, px) = glow_texture();
            ui.load_image(gpu, GLOW, w, h, &px);
            let (w, h, px) = ring_texture();
            ui.load_image(gpu, RING, w, h, &px);
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
        true
    }

    pub fn take_sounds(&mut self) -> Vec<crate::fx::sound::SoundCmd> {
        let mut out = std::mem::take(&mut self.sounds);
        if let Some(fx) = &mut self.effects {
            out.extend(fx.take_sounds());
        }
        out
    }

    pub fn pause_sounds(&mut self) -> Vec<crate::fx::sound::SoundCmd> {
        self.effects.as_mut().map(|fx| fx.pause_sounds()).unwrap_or_default()
    }

    pub fn after_submit(&mut self, gpu: &Gpu) {
        if let Some(fx) = &mut self.effects {
            fx.after_submit(gpu);
        }
    }

    /// Off the Create Character screen: everything goes (the stops for its voices returned).
    pub fn clear(&mut self) -> Vec<crate::fx::sound::SoundCmd> {
        if self.wanted.is_none() && self.hall.is_none() && self.hall_loading.is_none() {
            return Vec::new();
        }
        let mut out = std::mem::take(&mut self.sounds);
        if let Some(fx) = &mut self.effects {
            out.extend(fx.stop_sounds());
        }
        let files = self.files.take();
        *self = Self::default();
        self.files = files;
        out
    }
}

/// studio-shot's `--look`: seed (the face's own colours), none, or a look JSON file.
pub fn look_arg(w: &Want, arg: &str) -> Result<Option<Settings>> {
    Ok(match arg {
        "none" => None,
        "seed" => super::looks::start_for(w.look.race, w.look.face),
        path => Some(serde_json::from_str(&std::fs::read_to_string(path)?)?),
    })
}

/// `vana-client studio-shot`: the new character on the stage offscreen, a PNG at each time; `zoom` 0..1, `turn` in
/// degrees, `look` the custom look (None: their own face).
#[allow(clippy::too_many_arguments)]
pub fn shot(
    want: &Want,
    look: Option<Settings>,
    zoom: f32,
    turn: f32,
    swap_at: Option<(f32, Want)>,
    shots: &[f32],
    out_dir: &std::path::Path,
    size: [u32; 2],
) -> Result<()> {
    let cfg = crate::config::Config::load();
    let index = Arc::new(DatIndex::open(&cfg.ffxi_dir())?);
    let keys = Arc::new(zone::Keys::from_ffximain(&std::fs::read(crate::dat::ffximain(&cfg.ffxi_dir()))?)?);
    let mut s = Studio::default();
    s.install_hall(Hall::load(&index, &keys)?);
    s.effects = None;
    s.wanted = Some(want.clone());
    s.install_body(Body::load(&index, want)?);
    s.set_look(look);
    (s.zoom_want, s.zoom, s.turn_want, s.turn) = (zoom, zoom, turn, turn);
    let mut swap_at = swap_at.map(|(t, w)| (t, Body::load(&index, &w).map(|b| (w, b))));
    let gpu = Gpu::headless()?;
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let [w, h] = size;
    let target = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("studio"),
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
    for at in shots {
        while t + dt <= at {
            if swap_at.as_ref().is_some_and(|(st, _)| t >= *st)
                && let Some((_, Ok((w2, b)))) = swap_at.take()
            {
                s.wanted = Some(w2);
                s.install_body(b);
            }
            s.advance(dt);
            t += dt;
        }
        // The look is made on a thread: give it a moment, frame by frame, as the game would.
        for _ in 0..40 {
            let mut enc = gpu.device.create_command_encoder(&Default::default());
            s.render(&gpu, format, &mut enc, (&view, Some(&target), size), &depth, w as f32 / h as f32, None, 100);
            gpu.queue.submit([enc.finish()]);
            gpu.device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None })?;
            if s.settings.is_none() || s.look.shown() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(150));
        }
        let mut enc = gpu.device.create_command_encoder(&Default::default());
        s.render(&gpu, format, &mut enc, (&view, Some(&target), size), &depth, w as f32 / h as f32, None, 100);
        let readback = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("studio readback"),
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
        let path = out_dir.join(format!("studio_{:05.1}.png", at));
        let f = std::fs::File::create(&path)?;
        let mut e = png::Encoder::new(std::io::BufWriter::new(f), w, h);
        e.set_color(png::ColorType::Rgba);
        e.set_depth(png::BitDepth::Eight);
        e.write_header()?.write_image_data(&img)?;
        println!("{}", path.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_body_then_face() {
        let (lens0, span0, ..) = framing_at(0.0, false);
        let (lens1, span1, ..) = framing_at(1.0, false);
        assert!(lens1 > lens0 && span1 < span0);
        let (_, taru_span, ..) = framing_at(1.0, true);
        assert!(taru_span > span1);
    }
}
