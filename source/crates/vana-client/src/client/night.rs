//! Ours (Settings' Night Lighting Modern; not retail): nights that look like night. Original (the switch off) leaves
//! every part of retail's lighting as it was: nothing here runs but the lamps' upload, as before.
//!
//! * Darker, moonlit nights: from dusk (17:00 .. 20:00) to dawn (4:00 .. 7:00) the zone's ambient, sun / moon, fog,
//!   sky and clear colours are taken down and cooled toward moonlight (grade), by Night Darkness and the moon (a full
//!   moon is brighter than a new one). Outdoors only: a dungeon has no time of day, an interior keeps its room light.
//! * Lights that stand out: every point light in the table is brighter as the night deepens and a warm one (a lamp,
//!   a fire) flickers (zone.wgsl `lw`), and the lamps' glows get a soft halo (render/halo.rs).
//! * Lamps on a schedule of their own: a mod sets lamps within a radius to a level, faded (render.lamp_zone): the
//!   lamplighter lights them one by one.
//! * Lights placed outright in the table's free slots after the lamps (render::Placed): mods' (render.light: a
//!   lantern carried by a townsperson, a guard's torch, a lamp where the zone has none), the player's own light at
//!   night and in dungeons (Carry a Light), and the Mog House's fire furniture (campfires, hearths, lamps).
//!   They reach the scenery (ZoneRenderer::set_lamps, again as they move) and the actors (lamps::offer) as the lamps.

use super::lamps::{Glow, Lamp};
use crate::dat::weather::Lighting;
use crate::render::{LIGHTS, Placed};
use glam::Vec3;
use std::collections::HashMap;

/// The template a placed light's lamp entry carries (render/effects.rs LAMP_WGSL: its own values).
pub const PLACED: u32 = 0xfffe;
/// The last slot an actor's 8-bit light index can name (lamps.rs LAST).
const LAST: u32 = 254;
/// A moving light goes to the scenery again once it has moved this far, at most this often.
const MOVE: f32 = 0.75;
const RESEND: f32 = 0.25;

/// A light a mod puts in the world (render.light).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ModLight {
    pub at: Where,
    /// Yalms above the point (an entity's feet).
    pub height: f32,
    /// 1 = full (0x80 in the table).
    pub colour: [f32; 3],
    pub range: f32,
    pub intensity: f32,
    /// Its halo's size in yalms (0: none).
    pub halo: f32,
    /// Only after dark (faded in with the night).
    pub night_only: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Where {
    Point(Vec3),
    /// An entity by server id or target index (as game.entity takes it).
    Entity(u32),
}

/// A mod's lamps at a level (render.lamp_zone): the glows within `radius` of `at`, faded over `fade` seconds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LampZone {
    pub at: Vec3,
    pub radius: f32,
    pub level: f32,
    pub fade: f32,
}

/// A halo to draw (render/halo.rs).
pub use crate::render::halo::Halo;

/// What the frame knows of the place and the time (world_lighting).
#[derive(Clone, Copy, Debug, Default)]
pub struct Sky {
    /// 0 day .. 1 deep night.
    pub amount: f32,
    pub outdoor: bool,
    /// An interior's fixed light (Mog House, shops).
    pub fixed: bool,
    /// Lit at all (a zone with an environment).
    pub lit: bool,
}

/// A light to place this frame, before its slot.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Light {
    pub pos: Vec3,
    pub colour: [f32; 3],
    pub range: f32,
    pub intensity: f32,
    pub halo: f32,
}

#[derive(Default)]
pub struct State {
    pub sky: Sky,
    /// Each lamp slot's level as faded (the mods' lamp zones).
    levels: HashMap<u32, f32>,
    /// The player's own light, faded in and out (0..1).
    own: f32,
    /// This frame's placed lights with their slots.
    pub placed: Vec<Placed>,
    /// The lamps and placed lights for the actors (lamps::offer).
    pub offer: Vec<Lamp>,
    pub halos: Vec<Halo>,
    pub scale: Vec<f32>,
    /// What the scenery was last given, and when (a moving light resends).
    sent: Vec<(Vec3, u32)>,
    sent_age: f32,
    /// What the effects' lamp copy last had; Some when it changed this frame.
    uploaded: (Vec<(Vec3, u32, u32, u32)>, Vec<Placed>),
    pub upload: Option<(Vec<(Vec3, u32, u32, u32)>, Vec<Placed>)>,
    /// Seconds since the client started, for the flicker.
    pub clock: f32,
}

fn smoothstep(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// How far into the night Vana'diel minute `minute` (0..1440) is: 0 by day, rising 17:00 .. 20:00, 1 until 4:00,
/// falling to 0 by 7:00.
pub fn amount(minute: f32) -> f32 {
    let h = minute.rem_euclid(1440.0) / 60.0;
    if h >= 12.0 { smoothstep(17.0, 20.0, h) } else { 1.0 - smoothstep(4.0, 7.0, h) }
}

/// The moon's light for phase 0 (New Moon) .. 11 (as fx::Frame::moon_phase): 0 new, 1 full (phase 6).
pub fn moon(phase: u32) -> f32 {
    0.5 - 0.5 * (std::f32::consts::TAU * (phase % 12) as f32 / 12.0).cos()
}

/// The moon phase from the clock (FUN_10184dc0, as draw_effects works it out).
pub fn moon_phase(game_time: u32, since: f32) -> u32 {
    ((((game_time as f64 + since as f64) / 2.4 + 720.0) / 1440.0) as u64 / 7 % 12) as u32
}

/// The grade's strength: how far into the night, how dark the player wants it (0..1), outdoors only.
pub fn strength(amount: f32, darkness: f32, outdoor: bool, fixed: bool) -> f32 {
    if !outdoor || fixed { 0.0 } else { amount * darkness.clamp(0.0, 1.0) }
}

/// Moonlight: the colours a night takes on (bluer, the reds least).
const MOON: Vec3 = Vec3::new(0.72, 0.82, 1.12);

/// Takes a lighting down toward a moonlit night by `k` (0 none .. 1 the darkest), brighter under a full moon.
pub fn grade(l: &mut Lighting, k: f32, moon: f32) {
    if k <= 0.0 || l.fixed {
        return;
    }
    let tint = |c: Vec3, down: f32| -> Vec3 {
        let t = c.lerp(c * MOON, k);
        (t * (1.0 - down * k)).max(Vec3::ZERO)
    };
    let lift = 1.0 - 0.35 * moon;
    l.ambient = tint(l.ambient, 0.62 * lift);
    l.diffuse = tint(l.diffuse, 0.5 * lift);
    // Characters stay readable: a little less dark.
    l.actor_ambient = tint(l.actor_ambient, 0.5 * lift);
    l.actor_diffuse = tint(l.actor_diffuse, 0.4 * lift);
    l.two[1] = tint(l.two[1], 0.5 * lift);
    l.two[3] = tint(l.two[3], 0.5 * lift);
    l.fog = tint(l.fog, 0.68 * lift);
    l.clear = tint(l.clear, 0.68 * lift);
    if let Some((colours, _)) = l.sky.as_mut() {
        for c in colours.iter_mut() {
            *c = tint(*c, 0.68 * lift);
        }
    }
}

/// A fire furniture item's light (the Mog House's): colour, range, intensity, at what share of its height.
pub fn furniture_light(item: u16) -> Option<([f32; 3], f32, f32, f32)> {
    const WARM: [f32; 3] = [1.0, 0.62, 0.3];
    const CANDLE: [f32; 3] = [1.0, 0.75, 0.45];
    Some(match item {
        294 => (WARM, 9.0, 14.0, 0.5),                                    // campfire
        295 => (WARM, 12.0, 22.0, 0.6),                                   // bonfire
        3661 | 3705 => (WARM, 9.0, 12.0, 0.4),                            // stone hearth, far east hearth
        207 => (CANDLE, 7.0, 8.0, 0.8),                                   // fire lamp
        208 => ([0.55, 0.8, 1.0], 6.0, 6.0, 0.8),                         // ice lamp
        209 => ([0.55, 1.0, 0.6], 6.0, 6.0, 0.8),                         // wind lamp
        210 => ([1.0, 0.85, 0.45], 6.0, 6.0, 0.8),                        // earth lamp
        211 => ([0.8, 0.6, 1.0], 6.0, 6.0, 0.8),                          // lightning lamp
        212 => ([0.45, 0.65, 1.0], 6.0, 6.0, 0.8),                        // water lamp
        213 => ([1.0, 1.0, 0.9], 6.0, 7.0, 0.8),                          // light lamp
        214 => ([0.6, 0.4, 0.9], 5.0, 4.0, 0.8),                          // dark lamp
        429 => ([1.0, 0.45, 0.35], 6.0, 6.0, 0.7),                        // red storm lantern
        430 => ([0.45, 0.6, 1.0], 6.0, 6.0, 0.7),                         // blue
        431 => ([0.5, 1.0, 0.55], 6.0, 6.0, 0.7),                         // green
        432 => ([1.0, 0.9, 0.45], 6.0, 6.0, 0.7),                         // yellow
        433 => ([1.0, 1.0, 0.95], 6.0, 6.0, 0.7),                         // white
        203 | 204 | 205 | 454 | 456 | 457 | 458 => (WARM, 6.0, 6.0, 0.6), // bomb, pumpkin, mandragora, egg, gourd lanterns
        459 => (CANDLE, 5.0, 4.0, 0.9),                                   // candle holder
        319 => ([0.85, 0.9, 1.0], 5.0, 5.0, 0.7),                         // shell lamp
        118 => (CANDLE, 5.0, 4.0, 0.7),                                   // lamp egg
        _ => return None,
    })
}

/// What a frame gives the night: the settings, the lamps, the mods' lights and lamp zones resolved, the player.
pub struct Frame<'a> {
    pub on: bool,
    pub own_light: bool,
    pub dt: f32,
    pub lamps: &'a [Lamp],
    pub glows: &'a [Glow],
    /// The first free slot when there are no lamps (the zone's light keys).
    pub first: u32,
    /// The lamps changed this frame.
    pub new_lamps: bool,
    /// The mods' lights already at their points (night-only ones still flagged).
    pub mod_lights: &'a [(Light, bool)],
    pub zones: &'a [LampZone],
    /// The player's feet.
    pub me: Option<Vec3>,
    /// Fire furniture: position (the piece's base), item, height in yalms.
    pub fires: &'a [(Vec3, u16, f32)],
}

/// What to give the scenery (Some: the lamps and placed lights, by position and slot; quiet: a moving light's).
pub type Scenery = Option<(Vec<(Vec3, u32)>, bool)>;

impl State {
    /// The frame's placed lights, levels, halos and scale; returns what the scenery takes, if it changed.
    pub fn frame(&mut self, f: &Frame) -> Scenery {
        self.clock = (self.clock + f.dt) % 3600.0;
        let sky = self.sky;
        // The player's own light: at night outdoors, always in a dungeon (lit, not outdoors, not an interior).
        let dungeon = sky.lit && !sky.outdoor && !sky.fixed;
        let want = if f.on && f.own_light && (dungeon || (sky.outdoor && sky.amount > 0.45)) { 1.0 } else { 0.0 };
        self.own += (want - self.own).clamp(-f.dt, f.dt);
        let mut lights: Vec<Light> = Vec::new();
        if f.on {
            if let Some(me) = f.me.filter(|_| self.own > 0.01) {
                // Above the head (y points down), warm and soft.
                lights.push(Light {
                    pos: me + Vec3::new(0.0, -2.2, 0.0),
                    colour: [0.9, 0.76, 0.56],
                    range: 10.0,
                    intensity: 4.0 * self.own,
                    halo: 0.0,
                });
            }
            for &(l, night_only) in f.mod_lights {
                let k = if night_only { smoothstep(0.1, 0.6, sky.amount) } else { 1.0 };
                if k > 0.01 {
                    lights.push(Light { intensity: l.intensity * k, halo: l.halo * k.sqrt(), ..l });
                }
            }
            for &(pos, item, height) in f.fires {
                if let Some((colour, range, intensity, up)) = furniture_light(item) {
                    lights.push(Light {
                        pos: pos + Vec3::new(0.0, -height * up, 0.0),
                        colour,
                        range,
                        intensity,
                        halo: 0.0,
                    });
                }
            }
        }
        // Slots after the lamps', while there are any.
        let first = f.lamps.iter().map(|l| l.slot + 1).max().unwrap_or(f.first);
        self.placed.clear();
        let mut placed_halos = Vec::new();
        for (next, l) in (first..=LAST).zip(lights) {
            self.placed.push(Placed {
                pos: l.pos,
                slot: next,
                colour: l.colour,
                range: l.range,
                intensity: l.intensity,
            });
            if l.halo > 0.0 {
                placed_halos.push(Halo { pos: l.pos, slot: Some(next), colour: l.colour, size: l.halo, level: 1.0 });
            }
        }
        // The lamps' levels (the mods' zones), faded.
        let mut targets: HashMap<u32, (f32, f32)> = HashMap::new();
        if f.on {
            for g in f.glows {
                let Some(slot) = g.slot else { continue };
                // Where zones overlap, the lowest level wins.
                for z in f.zones.iter().filter(|z| z.at.distance(g.pos) <= z.radius) {
                    let e = targets.entry(slot).or_insert((1.0, z.fade));
                    if z.level <= e.0 {
                        *e = (z.level.clamp(0.0, 1.0), z.fade);
                    }
                }
            }
        }
        self.levels.retain(|s, v| {
            let (t, fade) = targets.get(s).copied().unwrap_or((1.0, 1.0));
            let step = if fade <= 0.0 { 1.0 } else { f.dt / fade };
            *v += (t - *v).clamp(-step, step);
            targets.contains_key(s) || *v < 0.999
        });
        for (s, (t, _)) in &targets {
            // A zone appearing takes its level at once only where the lamp was never seen (no fade from nothing).
            self.levels.entry(*s).or_insert(*t);
        }
        // Each slot's scale: brighter as the night deepens, times its level.
        self.scale.clear();
        self.scale.resize(LIGHTS, 1.0);
        if f.on {
            let boost = 1.0 + 0.6 * sky.amount;
            for v in self.scale.iter_mut() {
                *v = boost;
            }
            for (&s, &v) in &self.levels {
                if let Some(x) = self.scale.get_mut(s as usize) {
                    *x *= v;
                }
            }
        }
        // Halos for the glows (the placed lights' were added above).
        self.halos.clear();
        if f.on && sky.amount > 0.02 {
            for g in f.glows {
                let level = g.slot.and_then(|s| self.levels.get(&s).copied()).unwrap_or(1.0);
                let c = g.colour;
                let colour =
                    [((c >> 16) & 0xff) as f32 / 128.0, ((c >> 8) & 0xff) as f32 / 128.0, (c & 0xff) as f32 / 128.0];
                self.halos.push(Halo { pos: g.pos, slot: g.slot, colour, size: 1.4, level });
            }
            self.halos.extend(placed_halos);
        }
        // The actors' lamps: the lamps, then the placed lights.
        self.offer.clear();
        self.offer.extend_from_slice(f.lamps);
        self.offer.extend(self.placed.iter().map(|p| Lamp { pos: p.pos, slot: p.slot, template: PLACED, colour: 0 }));
        // The effects' lamp copy: when anything in it changed.
        let lamps: Vec<(Vec3, u32, u32, u32)> = f.lamps.iter().map(|l| (l.pos, l.slot, l.template, l.colour)).collect();
        self.upload = None;
        if f.new_lamps || self.uploaded.0 != lamps || self.uploaded.1 != self.placed {
            self.uploaded = (lamps.clone(), self.placed.clone());
            self.upload = Some((lamps, self.placed.clone()));
        }
        // The scenery's slots: the lamps' changes at once, a moving light now and then.
        let all: Vec<(Vec3, u32)> =
            f.lamps.iter().map(|l| (l.pos, l.slot)).chain(self.placed.iter().map(|p| (p.pos, p.slot))).collect();
        self.sent_age += f.dt;
        if f.new_lamps {
            self.sent = all.clone();
            self.sent_age = 0.0;
            return Some((all, false));
        }
        let moved = all.len() != self.sent.len()
            || all.iter().zip(&self.sent).any(|(a, b)| a.1 != b.1 || a.0.distance(b.0) > MOVE);
        if moved && (self.sent_age >= RESEND || all.len() != self.sent.len()) {
            self.sent = all.clone();
            self.sent_age = 0.0;
            return Some((all, true));
        }
        None
    }

    /// A lamp slot's level (the mods' lamp zones), 1 when none holds it.
    pub fn level(&self, slot: u32) -> f32 {
        self.levels.get(&slot).copied().unwrap_or(1.0)
    }

    /// The renderer's night uniforms: (on, seconds, amount, 0).
    pub fn uniform(&self, on: bool) -> [f32; 4] {
        if on { [1.0, self.clock, self.sky.amount, 0.0] } else { [0.0; 4] }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn night_amount_and_moon() {
        assert_eq!(amount(12.0 * 60.0), 0.0);
        assert_eq!(amount(22.0 * 60.0), 1.0);
        assert_eq!(amount(2.0 * 60.0), 1.0);
        assert!((amount(18.5 * 60.0) - 0.5).abs() < 1e-3);
        assert!((amount(5.5 * 60.0) - 0.5).abs() < 1e-3);
        assert_eq!(amount(8.0 * 60.0), 0.0);
        assert!(moon(0) < 1e-6 && (moon(6) - 1.0).abs() < 1e-6);
        assert_eq!(strength(1.0, 0.6, true, false), 0.6);
        assert_eq!(strength(1.0, 0.6, false, false), 0.0);
        assert_eq!(strength(1.0, 0.6, true, true), 0.0);
    }

    fn lit() -> Lighting {
        Lighting {
            dir: Vec3::Y,
            diffuse: Vec3::splat(0.6),
            ambient: Vec3::splat(0.5),
            actor_dir: Vec3::Y,
            actor_diffuse: Vec3::splat(0.6),
            actor_ambient: Vec3::splat(0.5),
            fog: Vec3::splat(0.4),
            fog_start: 10.0,
            fog_end: 100.0,
            clear: Vec3::splat(0.3),
            sky: None,
            area: Vec3::ONE,
            draw_dist: 100.0,
            two: [Vec3::Y, Vec3::splat(0.6), Vec3::Y, Vec3::splat(0.2)],
            fixed: false,
        }
    }

    #[test]
    fn grade_darkens_and_cools() {
        let mut l = lit();
        grade(&mut l, 0.0, 0.0);
        assert_eq!(l.ambient, Vec3::splat(0.5));
        grade(&mut l, 1.0, 0.0);
        assert!(l.ambient.x < 0.2 && l.ambient.z > l.ambient.x, "{:?}", l.ambient);
        assert!(l.actor_ambient.x > l.ambient.x, "characters a little less dark");
        let mut full = lit();
        grade(&mut full, 1.0, 1.0);
        assert!(full.ambient.x > l.ambient.x, "a full moon is brighter");
        let mut room = Lighting { fixed: true, ..lit() };
        grade(&mut room, 1.0, 0.0);
        assert_eq!(room.ambient, Vec3::splat(0.5), "an interior keeps its light");
    }

    fn lamp(x: f32, slot: u32) -> Lamp {
        Lamp { pos: Vec3::new(x, 0.0, 0.0), slot, template: 3, colour: 0 }
    }

    #[test]
    fn placed_lights_levels_and_scenery() {
        let mut s = State { sky: Sky { amount: 1.0, outdoor: true, fixed: false, lit: true }, ..Default::default() };
        let lamps = [lamp(0.0, 10), lamp(20.0, 11)];
        let glows = [
            Glow { pos: Vec3::new(0.0, 0.0, 0.0), slot: Some(10), colour: 0x80806040 },
            Glow { pos: Vec3::new(20.0, 0.0, 0.0), slot: Some(11), colour: 0x80806040 },
        ];
        let lantern =
            Light { pos: Vec3::new(5.0, 0.0, 0.0), colour: [1.0, 0.6, 0.3], range: 8.0, intensity: 10.0, halo: 0.5 };
        let zones = [LampZone { at: Vec3::ZERO, radius: 3.0, level: 0.0, fade: 2.0 }];
        let mut f = Frame {
            on: true,
            own_light: true,
            dt: 0.5,
            lamps: &lamps,
            glows: &glows,
            first: 5,
            new_lamps: true,
            mod_lights: &[(lantern, false)],
            zones: &zones,
            me: Some(Vec3::new(10.0, 0.0, 0.0)),
            fires: &[],
        };
        let sc = s.frame(&f).expect("new lamps go to the scenery");
        assert!(!sc.1);
        // The own light fades in (0.5 of 1 s) and comes first, then the lantern: slots 12, 13.
        assert_eq!(s.placed.len(), 2);
        assert_eq!((s.placed[0].slot, s.placed[1].slot), (12, 13));
        assert!((s.placed[0].intensity - 2.0).abs() < 1e-4);
        assert_eq!(sc.0.len(), 4);
        // The zone puts lamp 10 out at once (first seen), lamp 11 stays lit and brighter at night.
        assert_eq!(s.scale[10], 0.0);
        assert!((s.scale[11] - 1.6).abs() < 1e-4);
        assert_eq!(s.halos.len(), 3, "two glows and the lantern's halo");
        assert_eq!(s.offer.len(), 4);
        assert!(s.upload.is_some());
        // The lamplighter lights it: level 1 over 2 s.
        let lit = [LampZone { level: 1.0, ..zones[0] }];
        f.zones = &lit;
        f.new_lamps = false;
        s.frame(&f);
        assert!((s.scale[10] - 0.25 * 1.6).abs() < 1e-3, "{}", s.scale[10]);
        // Nothing moved: no scenery, no upload (the own light is full now, so one more frame).
        s.frame(&f);
        assert!(s.frame(&f).is_none());
        assert!(s.upload.is_none());
        // The player walks: their light follows, to the scenery a quarter second later at most.
        f.me = Some(Vec3::new(12.0, 0.0, 0.0));
        let moved = s.frame(&f);
        assert!(s.upload.is_some());
        assert!(moved.is_some_and(|m| m.1), "a moving light's resend is quiet");
        // Original: no placed lights, no halos, every slot as it is.
        f.on = false;
        f.new_lamps = true;
        let sc = s.frame(&f).unwrap();
        assert_eq!(sc.0.len(), 2);
        assert!(s.placed.is_empty() && s.halos.is_empty() && s.scale.iter().all(|&v| v == 1.0));
        assert_eq!(s.uniform(false), [0.0; 4]);
    }

    #[test]
    fn fire_furniture() {
        assert!(furniture_light(294).is_some() && furniture_light(295).is_some() && furniture_light(1).is_none());
        let mut s = State { sky: Sky { fixed: true, lit: true, ..Default::default() }, ..Default::default() };
        let f = Frame {
            on: true,
            own_light: true,
            dt: 1.0,
            lamps: &[],
            glows: &[],
            first: 3,
            new_lamps: false,
            mod_lights: &[],
            zones: &[],
            me: Some(Vec3::ZERO),
            fires: &[(Vec3::new(1.0, 0.0, 1.0), 294, 1.0)],
        };
        s.frame(&f);
        // No own light in a Mog House; the campfire in slot 3, half its height up.
        assert_eq!(s.placed.len(), 1);
        assert_eq!(s.placed[0].slot, 3);
        assert!((s.placed[0].pos.y + 0.5).abs() < 1e-5);
    }
}
