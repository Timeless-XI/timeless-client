//! Ours (Settings' Weather Modern; not retail): snow and blizzards drawn as real storms. Original (the switch off)
//! leaves retail's weather as it was.
//!
//! * The storm's strength eases in and out with the weather (snow light, a blizzard full), and the wind gusts every
//!   few seconds, turning slowly.
//! * Thick wind-driven snow round the camera (render/snow.rs): layers of flakes, streaked near the camera in the wind,
//!   none under a roof.
//! * A whiteout: the fog closes in and turns the white-grey of falling snow (darker at night), the sky goes into it,
//!   the sun flattens; it closes in more in a gust (grade).
//! * Snow building up on the ground and on what faces the sky (zone.wgsl's `weather`) over about five minutes (a
//!   snowfall half as much as a blizzard), steeper ground taking it as it deepens, and a dusting on characters'
//!   heads and shoulders; melting back more slowly after.
//! * Frost at the screen's edges in a blizzard's strongest gusts (Screen Frost; render/snow.rs).
//! * Breath: in the cold (any snowfall or blizzard, and always in the snowfields) every character, companion and
//!   townsperson breathes out a short stream of mist, in step with their idle motion's breathing (the breath out
//!   begins as the head, highest, starts to sink: `Rhythm`), else every few seconds (faster in a fight); it pushes
//!   forward, slows, rises and spreads with the wind (render/snow.rs Puff).
//! * A zone whose files have no effects for a weather the server sends (Xarcabard has no blizzard's) takes those of
//!   its milder kin (a blizzard's snow): `lighter`, used by the effects and the zone's lighting.

use crate::dat::weather::Lighting;
use glam::Vec3;

/// The server's weather numbers (xi::Weather) this draws.
pub const SNOW: u16 = 12;
pub const BLIZZARDS: u16 = 13;

/// How strong a storm the weather is: 0 none, a light snow, a full blizzard.
pub fn strength(weather: u16) -> f32 {
    match weather {
        SNOW => 0.35,
        BLIZZARDS => 1.0,
        _ => 0.0,
    }
}

/// A double weather's single kin (the zone's effects and light for it when the zone has none of its own).
pub fn lighter(tag: &str) -> Option<&'static str> {
    Some(match tag {
        "bliz" => "snow",
        "heat" => "dryw",
        "squl" | "sqal" => "rain",
        "sand" => "dust",
        "stom" => "wind",
        "bolt" => "thdr",
        "ligt" => "aura",
        "dark" => "fogd",
        _ => return None,
    })
}

fn smoothstep(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Seconds to ease the storm fully in, and out.
const RISE: f32 = 25.0;
const FALL: f32 = 20.0;
/// Seconds for the snow on the ground to build up fully, and to melt away.
const BUILD: f32 = 300.0;
const MELT: f32 = 900.0;
/// Seconds for snow to settle fully on someone standing in a blizzard, and to melt off them.
const SETTLE: f32 = 150.0;
const SETTLE_MELT: f32 = 400.0;
/// Seconds to fade the falling snow out under a roof and back.
const SHELTER: f32 = 1.5;

#[derive(Clone, Copy, Debug, Default)]
pub struct State {
    /// The storm's strength now (0..1), eased toward the weather's.
    pub level: f32,
    /// The wind's gust now (0..1).
    pub gust: f32,
    /// Where the wind blows (radians about the vertical; 0 toward +x).
    pub yaw: f32,
    /// Snow on the ground (0..1).
    pub accum: f32,
    /// The falling snow's share: 0 under a roof.
    pub open: f32,
    /// Frost at the screen's edges (0..1).
    pub frost: f32,
    /// Seconds since the client started (the gusts' clock, the flakes' flutter).
    pub clock: f32,
    /// How far the air has carried the snow (yalms, y down), added up each frame from the wind and the fall, wrapped
    /// (render/snow.rs PERIOD_XZ, PERIOD_Y): the flakes move by it, so a change in the wind changes their speed, not
    /// where they all are.
    pub drift: Vec3,
    /// How cold it is for breath to show (0..1), eased.
    pub cold: f32,
    /// Settings' Weather Modern, as the last frame had it.
    pub on: bool,
    zone: Option<u16>,
}

/// What a frame gives the storm.
#[derive(Clone, Copy, Debug)]
pub struct Frame {
    pub on: bool,
    pub dt: f32,
    pub weather: u16,
    pub zone: Option<u16>,
    /// A zone with a sky (not a dungeon, not an interior).
    pub outdoor: bool,
    /// Under a roof (the effects' covered).
    pub covered: bool,
}

impl State {
    pub fn frame(&mut self, f: &Frame) {
        let dt = f.dt.clamp(0.0, 0.5);
        self.clock += dt;
        self.on = f.on;
        let want = if f.on && f.outdoor { strength(f.weather) } else { 0.0 };
        if f.zone != self.zone {
            // A new zone: its storm as it is, and the snow it has had (not built up from nothing as we arrive).
            self.zone = f.zone;
            self.level = want;
            self.accum = (want * 1.4).min(1.0) * 0.6;
            self.open = if f.covered { 0.0 } else { 1.0 };
        }
        self.level =
            if want > self.level { (self.level + dt / RISE).min(want) } else { (self.level - dt / FALL).max(want) };
        // A snowfall lays down about half a blizzard's cover; a blizzard all of it.
        let cover = (self.level * 1.4).min(1.0);
        self.accum = if cover > self.accum {
            (self.accum + dt * cover / BUILD).min(cover)
        } else {
            (self.accum - dt / MELT).max(if f.on && f.outdoor { cover } else { 0.0 })
        };
        if !f.on || !f.outdoor {
            self.accum = 0.0;
        }
        let chill = if f.on && f.outdoor && (strength(f.weather) > 0.0 || f.zone.is_some_and(|z| COLD.contains(&z))) {
            1.0
        } else {
            0.0
        };
        self.cold = if chill > self.cold { (self.cold + dt / 3.0).min(1.0) } else { (self.cold - dt / 3.0).max(0.0) };
        let open = if f.covered { 0.0 } else { 1.0 };
        self.open =
            if open > self.open { (self.open + dt / SHELTER).min(1.0) } else { (self.open - dt / SHELTER).max(open) };
        self.gust = gust(self.clock);
        self.yaw = 0.7 + 0.45 * (self.clock * 0.031).sin() + 0.2 * (self.clock * 0.007 + 1.1).sin();
        let carry = self.wind() + Vec3::new(0.0, fall(self.gust), 0.0);
        let (pxz, py) = (crate::render::snow::PERIOD_XZ, crate::render::snow::PERIOD_Y);
        self.drift += carry * dt;
        self.drift = Vec3::new(self.drift.x.rem_euclid(pxz), self.drift.y.rem_euclid(py), self.drift.z.rem_euclid(pxz));
        self.frost = smoothstep(0.75, 1.0, self.level) * smoothstep(0.55, 0.95, self.gust) * self.open;
    }

    /// The wind's velocity for the flakes (yalms a second; y down, as the world's).
    pub fn wind(&self) -> Vec3 {
        // Kept under the fall for most of a gust: the snow slants, it doesn't fly flat.
        let speed = 0.8 + 2.2 * self.level + 2.8 * self.gust * self.level;
        Vec3::new(self.yaw.cos() * speed, 0.0, self.yaw.sin() * speed)
    }

    /// How thick the falling snow is (0..1 of the flakes).
    pub fn flakes(&self) -> f32 {
        (self.level * self.open * (0.85 + 0.15 * self.gust)).clamp(0.0, 1.0)
    }

    /// The wind's sound level (0..127), louder in a gust, quieter under a roof; None when there is no storm to hear.
    pub fn wind_level(&self, on: bool) -> Option<i32> {
        if !on || self.level < 0.05 {
            return None;
        }
        let l = self.level * (0.35 + 0.65 * self.gust) * (0.35 + 0.65 * self.open);
        Some((l * 100.0).round() as i32).filter(|&v| v > 0)
    }

    /// The snow settled on a character (0..1) a frame on: it builds while they stand still in falling snow (fully
    /// in a few minutes of blizzard, less in a snowfall), is shaken off as they move (a few seconds walking, faster
    /// running) or swing and cast, and melts slowly once the snow stops or they're under a roof. `speed` yalms a second.
    pub fn settle(&self, on: bool, snow: f32, dt: f32, speed: f32, busy: bool) -> f32 {
        if !on {
            return 0.0;
        }
        let dt = dt.clamp(0.0, 0.5);
        let falling = (self.level * 1.4).min(1.0) * self.open;
        let s = if speed > 0.3 {
            snow - dt * (0.12 + 0.07 * speed)
        } else if busy {
            snow - dt * 0.2
        } else if falling > snow {
            (snow + dt * falling / SETTLE).min(falling)
        } else {
            snow - dt / SETTLE_MELT
        };
        s.clamp(0.0, 1.0)
    }

    /// zone.wgsl's `weather`: x the snow on the ground, y the gust, z the storm, w 0.
    pub fn uniform(&self, on: bool) -> [f32; 4] {
        if !on {
            return [0.0; 4];
        }
        [self.accum, self.gust, self.level, 0.0]
    }

    /// The whiteout over a lighting: fog closer and the grey-white of falling snow, the sky into it, the sun flat.
    pub fn grade(&self, l: &mut Lighting) {
        grade(l, self.level, self.gust);
    }
}

/// The snowfields, cold enough to see your breath whatever the weather: Uleguerand Range, Beaucedine Glacier,
/// Xarcabard and their past ([S]) halves.
pub const COLD: [u16; 5] = [5, 111, 112, 136, 137];

/// Someone who breathes: their mouth (yalms, y down), which way they face (radians, 0 east, pi / 2 south),
/// whether they're working hard (fighting, running: faster breaths), and, when their idle motion breathes, where
/// they are in it: seconds since the chest last began to fall, and the cycle's length (`Rhythm`).
#[derive(Clone, Copy, Debug)]
pub struct Mouth {
    pub id: u32,
    pub pos: Vec3,
    pub yaw: f32,
    pub hard: bool,
    pub sync: Option<(f32, f32)>,
}

/// An idle motion's breathing, read once per motion off its posed skeleton: the frame where the head stands
/// highest (the lungs full, the breath out begins), and whether it rises and falls enough to call it breathing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rhythm {
    /// The motion it was read from (its tracks), to read again when the motion changes.
    pub key: usize,
    pub exhale: f32,
    pub ok: bool,
}

impl Rhythm {
    /// Read off the head's height (y down) at evenly spaced frames of one cycle.
    pub fn read(key: usize, heights: &[f32], frames: f32) -> Rhythm {
        let (lo, hi) = heights.iter().fold((f32::MAX, f32::MIN), |(l, h), &y| (l.min(y), h.max(y)));
        let top = heights.iter().enumerate().min_by(|a, b| a.1.total_cmp(b.1)).map_or(0, |(i, _)| i);
        let exhale = frames * top as f32 / heights.len().max(1) as f32;
        Rhythm { key, exhale, ok: heights.len() >= 4 && hi - lo > 0.002 }
    }

    /// Where they are in it now: seconds since the breath out began, and the cycle's length, from the motion's
    /// frame, frame count and frames a second; None when it doesn't breathe or the cycle is no breath's length.
    pub fn now(&self, frame: f32, frames: f32, fps: f32) -> Option<(f32, f32)> {
        if !self.ok || frames <= 0.0 || fps <= 0.0 {
            return None;
        }
        let period = frames / fps;
        (1.4..=8.0).contains(&period).then(|| ((frame - self.exhale).rem_euclid(frames) / fps, period))
    }
}

/// How long one wisp of a breath lasts (seconds), and how often a breath out lets one go.
const WISP_LIFE: f32 = 2.0;
const WISP_EVERY: f32 = 0.16;

impl State {
    /// This frame's breath: each breath out lets go a short stream of wisps that push forward from the mouth,
    /// slow, rise, spread and thin out, carried a little by the wind. In step with the idle motion's breathing
    /// where it has one, else every few seconds on their own rhythm (faster in a fight).
    pub fn breath(&self, mouths: &[Mouth]) -> Vec<crate::render::snow::Puff> {
        let mut out = Vec::new();
        if self.cold <= 0.0 {
            return out;
        }
        let wind = self.wind();
        for m in mouths {
            let seed = ((m.id.wrapping_mul(2_654_435_761) >> 8) & 0xffff) as f32 / 65535.0;
            let (t, period) = m.sync.unwrap_or_else(|| {
                let period = if m.hard { 2.0 } else { 3.4 } * (0.9 + 0.2 * seed);
                ((self.clock + seed * period).rem_euclid(period), period)
            });
            // A breath out takes a bit under half the cycle, up to a second.
            let exhale = (period * 0.42).min(1.0);
            let fwd = Vec3::new(m.yaw.cos(), 0.0, -m.yaw.sin());
            // This breath, and the last ones still thinning out.
            let mut back = 0.0;
            while back < WISP_LIFE + exhale {
                let since = t + back;
                let mut k = 0;
                while k as f32 * WISP_EVERY < exhale {
                    let born = k as f32 * WISP_EVERY;
                    k += 1;
                    let life = since - born;
                    if !(0.0..WISP_LIFE).contains(&life) {
                        continue;
                    }
                    let age = life / WISP_LIFE;
                    // Strongest early in the breath, trailing off.
                    let push = (std::f32::consts::PI * (born + WISP_EVERY * 0.5) / exhale).sin().max(0.0).powf(0.6)
                        * (1.0 - 0.35 * born / exhale);
                    let j = ((seed * 7.0 + k as f32 * 1.618).fract() - 0.5) * 0.06;
                    let side = Vec3::new(-fwd.z, 0.0, fwd.x);
                    let pos = m.pos
                        + fwd * (0.05 + 0.55 * (1.0 - (-2.4 * life).exp()))
                        + side * j * (0.5 + age)
                        + Vec3::new(0.0, -0.16 * age, 0.0)
                        + wind * 0.14 * life;
                    let alpha = self.cold * 0.62 * push * smoothstep(0.0, 0.06, age) * (1.0 - age).powf(1.5);
                    if alpha < 0.005 {
                        continue;
                    }
                    let size = 0.06 + 0.40 * age.sqrt() * (0.8 + 0.4 * push);
                    out.push(crate::render::snow::Puff { pos, size, alpha, seed: (seed + k as f32 * 0.37).fract(), age });
                }
                back += period;
            }
        }
        out
    }
}

/// How fast the snow falls (yalms a second), a little faster in a gust.
pub fn fall(gust: f32) -> f32 {
    2.2 + 0.6 * gust
}

/// The wind's gust at `t` seconds: calm stretches and gusts every several seconds, 0..1.
pub fn gust(t: f32) -> f32 {
    let g = 0.5 + 0.5 * (0.55 * (t * 0.31).sin() + 0.3 * (t * 0.87 + 1.3).sin() + 0.15 * (t * 2.3 + 0.4).sin());
    smoothstep(0.3, 0.95, g)
}

const LUMA: Vec3 = Vec3::new(0.3, 0.59, 0.11);
/// The white of a blizzard's air, a little blue.
const SNOWAIR: Vec3 = Vec3::new(0.96, 0.98, 1.04);

/// The whiteout at storm strength `k` and gust `gust`.
pub fn grade(l: &mut Lighting, k: f32, gust: f32) {
    if k <= 0.0 || l.fixed {
        return;
    }
    let w = (k * (0.7 + 0.3 * gust)).clamp(0.0, 1.0);
    // The air's colour: the fog's own brightness (dark at night), lifted toward white by the snow in it.
    let lum = l.fog.dot(LUMA).max(0.0);
    let bright = (lum * 1.25 + 0.08).min(0.92);
    let air = SNOWAIR * bright;
    l.fog = l.fog.lerp(air, w);
    // A blizzard hides everything past ~30-50 yalms (nearer in a gust); a light snow only softens the distance.
    let reach = 30.0 + 22.0 * (1.0 - gust);
    let end = l.fog_end.lerp_to(reach.min(l.fog_end), w);
    l.fog_end = end;
    l.fog_start = l.fog_start.lerp_to(l.fog_start.min(end * 0.12), w);
    l.clear = l.clear.lerp(air, w);
    if let Some((colours, _)) = l.sky.as_mut() {
        for c in colours.iter_mut() {
            *c = c.lerp(air, w);
        }
    }
    // Flat light: the sun weaker, the sky's light (ambient) a little stronger and greyer.
    let flat = |c: Vec3, down: f32| c * (1.0 - down * w);
    let grey = |c: Vec3, up: f32| {
        let g = Vec3::splat(c.dot(LUMA));
        c.lerp(g, 0.5 * w) * (1.0 + up * w)
    };
    l.diffuse = flat(l.diffuse, 0.6);
    l.actor_diffuse = flat(l.actor_diffuse, 0.45);
    l.two[1] = flat(l.two[1], 0.6);
    l.two[3] = flat(l.two[3], 0.6);
    l.ambient = grey(l.ambient, 0.25);
    l.actor_ambient = grey(l.actor_ambient, 0.3);
}

trait Lerp {
    fn lerp_to(self, to: f32, t: f32) -> f32;
}

impl Lerp for f32 {
    fn lerp_to(self, to: f32, t: f32) -> f32 {
        self + (to - self) * t
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(weather: u16) -> Frame {
        Frame { on: true, dt: 1.0, weather, zone: Some(112), outdoor: true, covered: false }
    }

    #[test]
    fn strengths_and_kin() {
        assert_eq!(strength(BLIZZARDS), 1.0);
        assert!(strength(SNOW) > 0.0 && strength(SNOW) < 0.5);
        assert_eq!(strength(1), 0.0);
        assert_eq!(lighter("bliz"), Some("snow"));
        assert_eq!(lighter("dark"), Some("fogd"));
        assert_eq!(lighter("snow"), None);
    }

    #[test]
    fn eases_in_and_out_and_builds_up() {
        let mut s = State::default();
        s.frame(&frame(SNOW));
        // Arriving: the zone's storm at once, some snow already down
        assert!((s.level - 0.35).abs() < 1e-5 && s.accum > 0.25);
        let mut f = frame(BLIZZARDS);
        for _ in 0..10 {
            s.frame(&f);
        }
        assert!(s.level > 0.5 && s.level < 0.6, "eases toward the blizzard (half-second steps at most): {}", s.level);
        for _ in 0..60 {
            s.frame(&f);
        }
        assert_eq!(s.level, 1.0);
        let before = s.accum;
        for _ in 0..200 {
            s.frame(&f);
        }
        assert!(s.accum > before + 0.15 && s.accum <= 1.0, "builds up: {before} -> {}", s.accum);
        f.weather = 1;
        for _ in 0..60 {
            s.frame(&f);
        }
        assert_eq!(s.level, 0.0);
        assert!(s.accum > 0.1, "the snow melts slowly: {}", s.accum);
    }

    #[test]
    fn off_indoors_and_under_a_roof() {
        let mut s = State::default();
        s.frame(&Frame { on: false, ..frame(BLIZZARDS) });
        assert_eq!((s.level, s.accum), (0.0, 0.0));
        assert_eq!(s.uniform(false), [0.0; 4]);
        let mut s = State::default();
        s.frame(&Frame { outdoor: false, ..frame(BLIZZARDS) });
        assert_eq!(s.level, 0.0);
        let mut s = State::default();
        s.frame(&frame(BLIZZARDS));
        assert!(s.flakes() > 0.8);
        let f = Frame { covered: true, dt: 0.5, ..frame(BLIZZARDS) };
        for _ in 0..4 {
            s.frame(&f);
        }
        assert_eq!(s.open, 0.0);
        assert_eq!(s.flakes(), 0.0);
        assert_eq!(s.frost, 0.0);
        assert_eq!(s.level, 1.0, "the storm itself goes on under the roof");
    }

    #[test]
    fn the_wind_is_heard() {
        let mut s = State { level: 1.0, gust: 1.0, open: 1.0, ..Default::default() };
        assert_eq!(s.wind_level(true), Some(100));
        assert_eq!(s.wind_level(false), None);
        s.gust = 0.0;
        assert_eq!(s.wind_level(true), Some(35));
        s.open = 0.0;
        assert!(s.wind_level(true).unwrap() < 15, "muffled under a roof");
        s.level = 0.0;
        assert_eq!(s.wind_level(true), None);
    }

    #[test]
    fn the_drift_goes_steadily_on() {
        let mut s = State::default();
        let f = Frame { dt: 0.016, ..frame(BLIZZARDS) };
        let mut last = s.drift;
        for _ in 0..2000 {
            s.frame(&f);
            // Each frame a small step (or a wrap): never a jump back
            let d = s.drift - last;
            let step = Vec3::new(
                d.x.rem_euclid(crate::render::snow::PERIOD_XZ),
                d.y.rem_euclid(crate::render::snow::PERIOD_Y),
                d.z.rem_euclid(crate::render::snow::PERIOD_XZ),
            );
            let step = Vec3::new(
                if step.x > 1.0 { step.x - crate::render::snow::PERIOD_XZ } else { step.x },
                step.y,
                if step.z > 1.0 { step.z - crate::render::snow::PERIOD_XZ } else { step.z },
            );
            assert!(step.length() < 0.2, "a small step: {step:?}");
            assert!(step.y > 0.0, "always falling");
            last = s.drift;
        }
    }

    #[test]
    fn breath_in_the_cold() {
        let mut s = State::default();
        let mouth = Mouth { id: 7, pos: Vec3::new(0.0, -1.6, 0.0), yaw: 0.0, hard: false, sync: None };
        // Warm and clear: none
        s.frame(&Frame { weather: 1, zone: Some(230), ..frame(1) });
        assert!(s.breath(&[mouth]).is_empty());
        // The snowfields in clear weather: breath
        let mut s = State::default();
        let f = Frame { dt: 0.1, weather: 1, zone: Some(112), ..frame(1) };
        let mut seen = 0;
        let mut ages = Vec::new();
        for _ in 0..80 {
            s.frame(&f);
            for p in s.breath(&[mouth]) {
                seen += 1;
                ages.push(p.age);
                assert!(p.alpha >= 0.0 && p.alpha <= 0.63 && p.size >= 0.06 && p.size <= 0.6);
                assert!(p.pos.x >= mouth.pos.x, "it goes the way they face (east)");
                assert!(p.pos.y <= mouth.pos.y + 1e-4, "and rises (y down)");
            }
        }
        assert!(seen > 10, "puffs come and go: {seen}");
        assert!(ages.iter().any(|&a| a < 0.2) && ages.iter().any(|&a| a > 0.8));
        // Two people breathe out of step
        let other = Mouth { id: 99, ..mouth };
        let a: Vec<bool> = (0..40)
            .map(|i| {
                s.clock = i as f32 * 0.1;
                !s.breath(&[mouth]).is_empty()
            })
            .collect();
        let b: Vec<bool> = (0..40)
            .map(|i| {
                s.clock = i as f32 * 0.1;
                !s.breath(&[other]).is_empty()
            })
            .collect();
        assert_ne!(a, b);
        // Off: none
        let mut s = State::default();
        s.frame(&Frame { on: false, zone: Some(112), ..frame(BLIZZARDS) });
        assert!(s.breath(&[mouth]).is_empty());
    }

    #[test]
    fn breath_keeps_time_with_the_idle() {
        // The head highest a quarter of the way through a 90-frame idle at 30 fps (3 s a breath)
        let heights: Vec<f32> = (0..32).map(|i| -1.6 - 0.01 * ((i as f32 / 32.0 - 0.25) * std::f32::consts::TAU).cos()).collect();
        let r = Rhythm::read(1, &heights, 90.0);
        assert!(r.ok && (r.exhale - 22.5).abs() < 3.0, "{r:?}");
        let (since, period) = r.now(22.5, 90.0, 30.0).unwrap();
        assert!(since < 0.1 && (period - 3.0).abs() < 1e-4);
        // Still: no breathing to keep time with; a stride-length cycle is no breath
        assert!(!Rhythm::read(1, &[-1.6; 32], 90.0).ok);
        assert!(r.now(0.0, 20.0, 30.0).is_none());
        // Synced: the breath shows just after the chest falls, not before
        let mut s = State::default();
        s.frame(&Frame { dt: 0.1, weather: 1, zone: Some(112), ..frame(1) });
        let at = |t: f32| Mouth { id: 7, pos: Vec3::ZERO, yaw: 0.0, hard: false, sync: Some((t, 3.0)) };
        assert!(s.breath(&[at(2.99)]).is_empty());
        assert!(!s.breath(&[at(0.3)]).is_empty());
        let a: f32 = s.breath(&[at(0.9)]).iter().map(|p| p.alpha).sum();
        let b: f32 = s.breath(&[at(2.6)]).iter().map(|p| p.alpha).sum();
        assert!(a > b, "fuller mid-breath than as it fades: {a} {b}");
    }

    #[test]
    fn snow_settles_and_shakes_off() {
        let mut s = State::default();
        let f = Frame { dt: 0.5, zone: Some(112), ..frame(BLIZZARDS) };
        for _ in 0..200 {
            s.frame(&f);
        }
        // Standing still: it settles over a couple of minutes
        let mut snow = 0.0;
        for _ in 0..60 {
            snow = s.settle(true, snow, 0.5, 0.0, false);
        }
        assert!(snow > 0.1 && snow < 0.4, "{snow}");
        for _ in 0..600 {
            snow = s.settle(true, snow, 0.5, 0.0, false);
        }
        assert!(snow > 0.95, "{snow}");
        // Two seconds running (5 yalms a second) shakes nearly all of it off; walking, about half
        let run = (0..8).fold(1.0, |x, _| s.settle(true, x, 0.25, 5.0, false));
        let walk = (0..8).fold(1.0, |x, _| s.settle(true, x, 0.25, 2.0, false));
        assert!(run < 0.1 && walk > run && walk < 0.8, "{run} {walk}");
        // A swing knocks some off; off is none
        assert!(s.settle(true, 1.0, 0.5, 0.0, true) < 1.0);
        assert_eq!(s.settle(false, 1.0, 0.5, 0.0, false), 0.0);
        // Clear weather: it melts slowly
        let mut c = State::default();
        c.frame(&Frame { dt: 0.5, zone: Some(112), ..frame(1) });
        let m = c.settle(true, 1.0, 0.5, 0.0, false);
        assert!(m < 1.0 && m > 0.99);
    }

    #[test]
    fn gusts_come_and_go() {
        let (mut lo, mut hi) = (1.0f32, 0.0f32);
        for i in 0..2000 {
            let g = gust(i as f32 * 0.1);
            lo = lo.min(g);
            hi = hi.max(g);
            assert!((0.0..=1.0).contains(&g));
        }
        assert!(lo < 0.05 && hi > 0.9, "calm and gusts: {lo} .. {hi}");
        let mut s = State::default();
        s.level = 1.0;
        s.gust = 1.0;
        let strong = s.wind().length();
        s.gust = 0.0;
        assert!(strong > s.wind().length() * 1.8);
    }

    fn lighting() -> Lighting {
        Lighting {
            dir: Vec3::new(0.0, 1.0, 0.0),
            diffuse: Vec3::splat(0.8),
            ambient: Vec3::new(0.4, 0.35, 0.3),
            actor_dir: Vec3::new(0.0, 1.0, 0.0),
            actor_diffuse: Vec3::splat(0.8),
            actor_ambient: Vec3::splat(0.4),
            fog: Vec3::new(0.5, 0.55, 0.6),
            fog_start: 60.0,
            fog_end: 200.0,
            clear: Vec3::new(0.3, 0.4, 0.6),
            sky: Some(([Vec3::new(0.2, 0.4, 0.8); 8], [0.0; 8])),
            area: Vec3::ONE,
            draw_dist: 200.0,
            two: [Vec3::Y, Vec3::splat(0.8), Vec3::Y, Vec3::splat(0.2)],
            fixed: false,
        }
    }

    #[test]
    fn the_whiteout() {
        let mut l = lighting();
        grade(&mut l, 1.0, 1.0);
        assert!(
            l.fog_end <= 31.0 && l.fog_start <= l.fog_end * 0.12 + 1e-3,
            "close fog: {} .. {}",
            l.fog_start,
            l.fog_end
        );
        assert!(l.fog_start >= 0.0);
        assert!((l.fog.x - l.fog.y).abs() < 0.05 && l.fog.z >= l.fog.x, "grey-white air: {:?}", l.fog);
        let sky = l.sky.unwrap().0[7];
        assert!((sky - l.fog).length() < 1e-4, "the sky goes into it");
        assert!(l.diffuse.x < 0.4 && l.ambient.x > 0.35, "flat light");
        // A light snow: farther fog, less change
        let mut m = lighting();
        grade(&mut m, 0.35, 0.0);
        assert!(m.fog_end > 120.0 && m.fog_end < 200.0, "{}", m.fog_end);
        // None, or an interior: unchanged
        let mut n = lighting();
        grade(&mut n, 0.0, 1.0);
        assert_eq!(n.fog_end, 200.0);
        let mut i = Lighting { fixed: true, ..lighting() };
        grade(&mut i, 1.0, 1.0);
        assert_eq!(i.fog_end, 200.0);
        // Night: the air stays dark
        let mut d = Lighting { fog: Vec3::splat(0.05), ..lighting() };
        grade(&mut d, 1.0, 1.0);
        assert!(d.fog.x < 0.2, "a night blizzard is dark grey: {:?}", d.fog);
    }
}
