//! Timeless XI: the Timeless Fishing bar mini-game, built into the client (it was the Ashita add-on
//! `timelessfishing` 2.16). Every interface has it: a Timeless XI server rewrites the fish's s2c 0x115 into the bar
//! game's settings (magic 0x5855), and the client plays that instead of the arrow game. On any other server the
//! 0x115 is the retail one and the arrow game plays as usual.
//!
//! The protocol is the add-on's, unchanged, so the server module (`modules/timeless_fishing` in timeless-server)
//! needs nothing new:
//! - c2s 0x110 mode 5 para 10 (`HELLO`) with no fish on the line: "this client plays the bar game". Sent on every
//!   zone-in and right before every cast; the server refuses casts from clients that never said it.
//! - s2c 0x115 with magic 0x5855 (layout below): a bite. Kind 0 = a fight on the bar (fish and monsters), kind 1 =
//!   an item or chest (no bar: Enter reels it in, Escape lets it go).
//! - c2s 0x110 mode 3 (the end): para 1 a Perfect win, 0 a win, 300 lost, 200 gave up; para2 = the 0x115's
//!   special, unchanged. Then the client's own release (mode 4) once the result's motion is over.
//! - c2s 0x110 mode 5 para 5: "you don't know how much longer..." (10 s left); para 6 / 7 / 8: the rod pose
//!   (centre / left / right), sent when it changes.
//!
//! 0x115 layout (offsets from the packet start): 0x04 u16 magic, 0x06 u8 kind, 0x07 u8 behaviour (0 mixed, 1
//! smooth, 2 sinker, 3 floater, 4 dart), 0x08 u16 difficulty 0-100, 0x0A u16 bar per mille of the track, 0x0C u8
//! fill / 0x0D u8 drain per mille a second (v3; v2 had them as u16 at 0x0C / 0x0E), 0x0E u16 the catch's item id
//! (v3), 0x10 u16 time limit in seconds, 0x12 u8 flags (1 legendary, 2 big, 4 monster, 8 chest), 0x13 u8 count,
//! 0x14 u32 special.

/// "TX": layout v2 (no item id).
pub const MAGIC_V2: u16 = 0x5854;
/// "UX": layout v3.
pub const MAGIC_V3: u16 = 0x5855;

/// c2s 0x110 mode 5 paras.
pub const HELLO: i32 = 10;
pub const WARN: i32 = 5;
pub const POSE_CENTER: i32 = 6;
pub const POSE_LEFT: i32 = 7;
pub const POSE_RIGHT: i32 = 8;

/// c2s 0x110 mode 3 paras.
pub const END_PERFECT: i32 = 1;
pub const END_WIN: i32 = 0;
pub const END_GIVE_UP: i32 = 200;
pub const END_LOST: i32 = 300;

const METER_START: f32 = 0.30;
/// The catch zone (track 0..1, centre 0.5).
const ZONE_ACCEL: f32 = 2.6;
const ZONE_SPRING: f32 = 6.0;
const ZONE_DAMP: f32 = 3.2;
const ZONE_VMAX: f32 = 1.4;
/// A finished round stays on screen this long ("Caught!", "It got away...").
pub const SHOW_AFTER: f32 = 1.5;

pub const BEHAVIOURS: [&str; 5] = ["mixed", "smooth", "sinker", "floater", "dart"];

/// A Timeless bite, as the server sent it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bite {
    /// 0 a fight on the bar, 1 an item or chest.
    pub kind: u8,
    pub behaviour: u8,
    pub difficulty: u16,
    /// The catch zone, a share of the track.
    pub bar: f32,
    /// The catch meter's fill and drain a second.
    pub fill: f32,
    pub drain: f32,
    pub item: u16,
    pub time_limit: u16,
    pub flags: u8,
    pub count: u8,
    pub special: u32,
}

impl Bite {
    pub fn legendary(&self) -> bool {
        self.flags & 1 != 0
    }
    pub fn monster(&self) -> bool {
        self.flags & 4 != 0
    }
    pub fn chest(&self) -> bool {
        self.flags & 8 != 0
    }
}

fn u16_at(p: &[u8], o: usize) -> u16 {
    p.get(o..o + 2).map_or(0, |b| u16::from_le_bytes([b[0], b[1]]))
}

/// The s2c 0x115 (the whole packet, header included) as a Timeless bite; None for retail's (the arrow game).
pub fn parse(p: &[u8]) -> Option<Bite> {
    let magic = u16_at(p, 4);
    if magic != MAGIC_V2 && magic != MAGIC_V3 {
        return None;
    }
    let byte = |o: usize| p.get(o).copied().unwrap_or(0);
    let (fill, drain, item) = if magic == MAGIC_V3 {
        (byte(0x0c) as f32, byte(0x0d) as f32, u16_at(p, 0x0e))
    } else {
        (u16_at(p, 0x0c) as f32, u16_at(p, 0x0e) as f32, 0)
    };
    Some(Bite {
        kind: byte(6),
        behaviour: byte(7),
        difficulty: u16_at(p, 8).min(100),
        bar: u16_at(p, 0x0a) as f32 / 1000.0,
        fill: fill / 1000.0,
        drain: drain / 1000.0,
        item,
        time_limit: u16_at(p, 0x10),
        flags: byte(0x12),
        count: byte(0x13),
        special: p.get(0x14..0x18).map_or(0, |b| u32::from_le_bytes([b[0], b[1], b[2], b[3]])),
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Done {
    Win,
    Lose,
    GiveUp,
}

/// One round on the bar (the add-on's `new_round` / `step`).
#[derive(Clone, Debug)]
pub struct Round {
    pub bite: Bite,
    pub practice: bool,
    pub d: f32,
    pub t: f32,
    pub meter: f32,
    pub perfect: bool,
    pub warned: bool,
    pub bar_x: f32,
    bar_v: f32,
    pub fish_x: f32,
    fish_v: f32,
    target: f32,
    pub done: Option<Done>,
    pub done_t: f32,
    pub inside: bool,
    retarget: f32,
    amp: f32,
    k: f32,
    accel: f32,
    rng: u64,
    /// The 10-seconds-left warning is due (sent by the caller).
    pub warn_now: bool,
    /// The end report was handed over to be sent.
    pub reported: bool,
}

impl Round {
    pub fn new(bite: Bite, practice: bool, seed: u64) -> Round {
        let d = bite.difficulty.min(100) as f32;
        let (mut retarget, mut amp, mut k, mut accel) =
            (0.35 + 0.025 * d, (0.15 + 0.006 * d).min(0.9), 1.2 + 0.03 * d, 6.0 + 0.06 * d);
        if bite.behaviour == 1 {
            k *= 0.6;
            retarget *= 0.6;
            accel = 3.0;
        }
        if bite.behaviour == 4 {
            amp = (amp * 1.4).min(0.95);
            retarget *= 1.5;
        }
        Round {
            bite,
            practice,
            d,
            t: 0.0,
            meter: METER_START,
            perfect: true,
            warned: false,
            bar_x: 0.5,
            bar_v: 0.0,
            fish_x: 0.5,
            fish_v: 0.0,
            target: 0.5,
            done: None,
            done_t: 0.0,
            inside: false,
            retarget,
            amp,
            k,
            accel,
            rng: seed | 1,
            warn_now: false,
            reported: false,
        }
    }

    /// A practice round: a fish at your skill with this difficulty (the server's own formulas).
    pub fn practice(d: u16, behaviour: u8, monster: bool, seed: u64) -> Round {
        let df = d.min(100) as f32;
        let bite = Bite {
            kind: 0,
            behaviour: behaviour.min(4),
            difficulty: d.min(100),
            bar: 0.28,
            fill: 0.20 - 0.0012 * df,
            drain: 0.06 + 0.0016 * df,
            item: 0,
            time_limit: 60,
            flags: if monster { 4 } else { 0 },
            count: 1,
            special: 0,
        };
        Round::new(bite, true, seed)
    }

    /// 0..1.
    fn random(&mut self) -> f32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        (self.rng >> 40) as f32 / (1u64 << 24) as f32
    }

    pub fn finish(&mut self, d: Done) {
        if self.done.is_none() {
            self.done = Some(d);
            self.done_t = self.t;
        }
    }

    /// The end report's para for a finished round.
    pub fn end_para(&self) -> Option<i32> {
        Some(match self.done? {
            Done::Win if self.perfect => END_PERFECT,
            Done::Win => END_WIN,
            Done::GiveUp => END_GIVE_UP,
            Done::Lose => END_LOST,
        })
    }

    /// Advances `dt` seconds in steps of at most 10 ms (the same physics at any frame rate). `dir`: -1 steering
    /// left, 1 right, 0 none.
    pub fn advance(&mut self, dt: f32, dir: f32) {
        let n = (dt / 0.01).ceil().max(1.0) as usize;
        for _ in 0..n {
            self.step(dt / n as f32, dir);
        }
    }

    fn step(&mut self, dt: f32, dir: f32) {
        self.t += dt;
        if self.done.is_some() {
            return;
        }
        // Catch zone: pushed by the keys, springs back to the centre when let go.
        let a = if dir != 0.0 {
            let a = dir.signum() * ZONE_ACCEL;
            if self.bar_v * dir < 0.0 { a * 1.5 } else { a }
        } else {
            -ZONE_SPRING * (self.bar_x - 0.5) - ZONE_DAMP * self.bar_v
        };
        self.bar_v = (self.bar_v + a * dt).clamp(-ZONE_VMAX, ZONE_VMAX);
        self.bar_x += self.bar_v * dt;
        let half = self.bite.bar / 2.0;
        if self.bar_x < half {
            (self.bar_x, self.bar_v) = (half, 0.0);
        }
        if self.bar_x > 1.0 - half {
            (self.bar_x, self.bar_v) = (1.0 - half, 0.0);
        }

        // The fish.
        let beh = self.bite.behaviour;
        if self.random() < self.retarget * dt {
            let mut spread = (self.random() * 2.0 - 1.0) * self.amp;
            if beh == 2 {
                spread -= 0.25 * self.amp; // a sinker pulls left
            }
            if beh == 3 {
                spread += 0.25 * self.amp; // a floater pulls right
            }
            self.target = (self.fish_x + spread).clamp(0.03, 0.97);
        }
        let fdir = self.target - self.fish_x;
        let mut k = self.k;
        if beh == 2 {
            k *= if fdir < 0.0 { 1.8 } else { 0.75 };
        }
        if beh == 3 {
            k *= if fdir > 0.0 { 1.8 } else { 0.75 };
        }
        let want = fdir * k;
        self.fish_v += (want - self.fish_v) * (self.accel * dt).min(1.0);
        if beh == 4 && self.random() < (0.2 + 0.01 * self.d) * dt {
            self.fish_v += (self.random() * 2.0 - 1.0) * (0.6 + 0.01 * self.d); // a dart's burst
        }
        self.fish_x += self.fish_v * dt;
        if self.fish_x < 0.02 {
            (self.fish_x, self.fish_v) = (0.02, self.fish_v.abs() * 0.5);
        }
        if self.fish_x > 0.98 {
            (self.fish_x, self.fish_v) = (0.98, -self.fish_v.abs() * 0.5);
        }

        // The catch meter.
        self.inside = (self.fish_x - self.bar_x).abs() <= half;
        if self.inside {
            self.meter += self.bite.fill * dt;
        } else {
            self.meter -= self.bite.drain * dt;
            if self.t > 0.5 {
                self.perfect = false;
            }
        }
        if self.meter >= 1.0 {
            self.meter = 1.0;
            self.finish(Done::Win);
            return;
        }
        if self.meter <= 0.0 {
            self.meter = 0.0;
            self.finish(Done::Lose);
            return;
        }
        if !self.practice {
            let left = self.time_limit() - self.t;
            if left <= 10.0 && !self.warned {
                self.warned = true;
                self.warn_now = true;
            }
            if left <= 0.0 {
                self.finish(Done::Lose);
            }
        }
    }

    pub fn time_limit(&self) -> f32 {
        if self.bite.time_limit == 0 { 60.0 } else { self.bite.time_limit as f32 }
    }

    /// The rod pose for the fish's place, pulling against it: fish on the right -> para 8, on the left -> para 7,
    /// the middle 10 % neutral (a side from 5 % off centre, back to neutral inside 4 %). In game para 7 shows the
    /// rod held to the right and 8 to the left (owner, 2026-11-07), hence the swap.
    pub fn pose(&self, current: i32) -> i32 {
        let off = self.fish_x - 0.5;
        if off > 0.05 {
            POSE_RIGHT
        } else if off < -0.05 {
            POSE_LEFT
        } else if off.abs() < 0.04 {
            POSE_CENTER
        } else {
            current
        }
    }
}

/// An item or chest on the line, waiting for Enter (reel in) or Escape (let it go).
#[derive(Clone, Debug)]
pub struct Pending {
    pub bite: Bite,
    pub t: f32,
    pub warned: bool,
}

impl Pending {
    pub fn time_limit(&self) -> f32 {
        if self.bite.time_limit == 0 { 60.0 } else { self.bite.time_limit as f32 }
    }
}

/// What is on screen.
#[derive(Clone, Debug)]
pub enum Play {
    Bar(Round),
    Item(Pending),
}

/// The world's Timeless fishing state.
#[derive(Default)]
pub struct TlFish {
    pub play: Option<Play>,
    /// The end report to send (para, para2), tried again each frame until the send gate lets it through.
    pub end_due: Option<(i32, u32)>,
    /// The zone the hello was last sent in (a hello is due on each zone-in).
    pub hello_zone: Option<u16>,
    /// The rod pose last sent, and when (the round's clock).
    pub pose: i32,
    pub pose_at: f32,
}

impl TlFish {
    /// A round or item box is waiting for the player (the native game's keys are ours).
    pub fn active(&self) -> bool {
        match &self.play {
            Some(Play::Bar(r)) => r.done.is_none(),
            Some(Play::Item(_)) => true,
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn packet(kind: u8, beh: u8, d: u16, bar: u16, fill: u8, drain: u8, item: u16, flags: u8) -> Vec<u8> {
        let mut p = vec![0u8; 0x18];
        p[0] = 0x15;
        p[1] = 0x0c;
        p[4..6].copy_from_slice(&MAGIC_V3.to_le_bytes());
        p[6] = kind;
        p[7] = beh;
        p[8..10].copy_from_slice(&d.to_le_bytes());
        p[0x0a..0x0c].copy_from_slice(&bar.to_le_bytes());
        p[0x0c] = fill;
        p[0x0d] = drain;
        p[0x0e..0x10].copy_from_slice(&item.to_le_bytes());
        p[0x10..0x12].copy_from_slice(&60u16.to_le_bytes());
        p[0x12] = flags;
        p[0x13] = 2;
        p[0x14..0x18].copy_from_slice(&1234u32.to_le_bytes());
        p
    }

    #[test]
    fn parses_v3_and_ignores_retail() {
        let b = parse(&packet(0, 4, 55, 280, 134, 148, 4472, 5)).unwrap();
        assert_eq!((b.kind, b.behaviour, b.difficulty, b.item, b.count, b.special), (0, 4, 55, 4472, 2, 1234));
        assert!((b.bar - 0.28).abs() < 1e-6 && (b.fill - 0.134).abs() < 1e-6 && (b.drain - 0.148).abs() < 1e-6);
        assert!(b.legendary() && b.monster() && !b.chest());
        let mut retail = packet(0, 0, 0, 0, 0, 0, 0, 0);
        retail[4..6].copy_from_slice(&300u16.to_le_bytes()); // a stamina
        assert_eq!(parse(&retail), None);
    }

    #[test]
    fn a_held_zone_on_the_fish_wins_perfect() {
        // A calm fish that never moves: the zone sits on it from the start.
        let mut r = Round::practice(5, 1, false, 7);
        r.retarget = 0.0;
        r.advance(10.0, 0.0);
        assert_eq!(r.done, Some(Done::Win));
        assert_eq!(r.end_para(), Some(END_PERFECT));
    }

    #[test]
    fn a_zone_held_away_loses() {
        let mut r = Round::practice(100, 0, false, 9);
        for _ in 0..3000 {
            r.advance(0.01, -1.0);
            r.fish_x = 0.95; // keep the fish far right
        }
        assert_eq!(r.done, Some(Done::Lose));
        assert_eq!(r.end_para(), Some(END_LOST));
    }

    #[test]
    fn the_time_limit_warns_then_loses() {
        let mut b = Round::practice(50, 0, false, 3).bite;
        (b.fill, b.drain, b.time_limit) = (0.0, 0.0, 20);
        let mut r = Round::new(b, false, 3);
        r.advance(10.5, 0.0);
        assert!(r.warn_now && r.done.is_none());
        r.advance(10.0, 0.0);
        assert_eq!(r.done, Some(Done::Lose));
    }

    #[test]
    fn physics_do_not_depend_on_the_frame_rate() {
        let mut a = Round::practice(60, 0, false, 11);
        let mut b = a.clone();
        a.advance(1.0, 1.0);
        for _ in 0..10 {
            b.advance(0.1, 1.0);
        }
        assert!((a.bar_x - b.bar_x).abs() < 1e-3);
    }

    #[test]
    fn pose_pulls_against_the_fish() {
        let mut r = Round::practice(10, 0, false, 1);
        r.fish_x = 0.8;
        assert_eq!(r.pose(POSE_CENTER), POSE_RIGHT);
        r.fish_x = 0.2;
        assert_eq!(r.pose(POSE_CENTER), POSE_LEFT);
        r.fish_x = 0.545;
        assert_eq!(r.pose(POSE_LEFT), POSE_LEFT);
        r.fish_x = 0.51;
        assert_eq!(r.pose(POSE_LEFT), POSE_CENTER);
    }
}
