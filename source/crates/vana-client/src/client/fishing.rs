//! Fishing, the 2013 mini-game (MODLOG "Fishing"): /fish's checks and request latch, the bite wait, the fight (arrows
//! against the fish's stamina) and its end. The client never decides a catch: Enter sends the stamina left (c2s 0x110
//! mode 3) and LSB answers with the result status.

use super::World;
use super::game_ui::GameUi;
use super::input::{Input, mode};
use super::net::NetCommand;
use crate::fx::sched::Who;

use super::timeless::fishing::{self as tl, Done, Pending, Play, Round};

/// NewFishingStart (0x38) .. NewFishingStop (0x3e).
pub fn fishing_status(s: u8) -> bool {
    (0x38..=0x3e).contains(&s)
}

/// FUN_100956c0: any of the fishing statuses, the old ones too (6, 0x26..0x2b, 0x32..0x35) and 0x38..0x3e.
pub fn any_fishing_status(s: u8) -> bool {
    matches!(s, 6 | 0x26..=0x2b | 0x32..=0x35 | 0x38..=0x3e)
}

/// s2c 0x115 (LAB_1009fd30), packet offsets 4..0x17.
#[derive(Clone, Copy, Debug)]
pub struct FishParams {
    pub stamina: u16,
    pub delay: u16,
    pub regen: u16,
    pub moves: u16,
    pub damage: u16,
    pub heal: u16,
    pub time: u16,
    pub sense: u8,
    pub special: u32,
}

/// G (DAT_10487ee8) and I (DAT_10487ee4).
struct Game {
    max: i32,
    cur: i32,
    shown: i32,
    delay: i32,
    /// G[4]: the stamina change a tick (regen - 0x80).
    regen: i32,
    /// G[5]: move x 20.
    moves: i32,
    damage: i32,
    heal: i32,
    /// G[8]: ticks left.
    time: i32,
    warned: bool,
    large: bool,
    dir: u8,
    hit_timer: i32,
    special: u32,
    gold: bool,
}

/// Our player's fishing state (entity +0x13c, the DAT_10487e.. globals) while the fishing model is set up.
pub struct Fishing {
    /// +0x13c bits 8..10.
    sub: u8,
    /// DAT_1048315c: Enter or a click during the wait.
    early: bool,
    /// +0x1e8: no early end for 3 s.
    lockout: i32,
    /// DAT_10487ef8: frames since the bite check went.
    frames: u32,
    game: Option<Game>,
    /// DAT_10487ec8: -1 before the first tick, 0 idle, 1 between arrows, 2 an arrow up.
    arrow: i8,
    /// DAT_10487ecc: the next-arrow accumulator, then the arrow's window.
    acc: i32,
    t: i32,
    good: u32,
    bad: u32,
    /// DAT_10487efc: axis 4's edge latch.
    latch: bool,
    /// +0x13c bits 12..17: frames until the arrow effects are cleared again.
    clear: u8,
    rng: u32,
    /// The arrow up was answered (a hit or a wrong key).
    answered: bool,
    /// Retail's frames at 30 a second (u = 2 ticks), and what came in since the last.
    step: f32,
    enter: bool,
    enter_free: bool,
    escape: bool,
    click: bool,
    axis: f32,
}

/// One retail frame at its 30 fps (FUN_10014cf0 = 2.0, FUN_10014d60 = 2).
const U: i32 = 2;

fn name(s: &[u8; 4]) -> u32 {
    u32::from_le_bytes(*s)
}

impl Fishing {
    fn rand(&mut self) -> i32 {
        self.rng = self.rng.wrapping_mul(214_013).wrapping_add(2_531_011);
        (self.rng >> 16 & 0x7fff) as i32
    }
}

/// c2s 0x110 (FUN_10088380): UniqueNo, para, ActIndex, mode, 0, para2.
/// FUN_10088380: false when FUN_100fc3c0 refuses it (its 1 s gate or a full queue); the controller then stays and
/// tries again next frame.
fn send(w: &mut World, para: i32, mode: u8, para2: u32) -> bool {
    if !super::net::call_gate(&mut w.call_gate, super::net::C2S_FISHING, &w.net.stats, std::time::Instant::now()) {
        log::trace!("fishing: c2s 0x110 mode {mode} refused by FUN_100fc3c0, tried again next frame");
        return false;
    }
    let mut body = w.my_id.to_le_bytes().to_vec();
    body.extend(para.to_le_bytes());
    body.extend(w.my_index.to_le_bytes());
    body.extend([mode, 0]);
    body.extend(para2.to_le_bytes());
    log::info!("fishing: c2s 0x110 mode {mode} para {para} para2 {para2}");
    let _ = w.net.commands.send(NetCommand::Packet { id: super::net::C2S_FISHING, body });
    true
}

/// Timeless XI: a Timeless server's rod poses (status 52 / 53, from an older client's pose reports) while our bar
/// game is up still count as fishing.
fn timeless_pose(w: &World, s: u8) -> bool {
    matches!(s, 0x34 | 0x35) && w.tl_fish.play.as_ref().is_some_and(|p| !matches!(p, Play::Bar(r) if r.practice))
}

/// The fishing statuses and the request latch keep us in place (FUN_100964f0).
pub fn locked(w: &World) -> bool {
    w.fish_pending
        || fishing_status(w.my_motion.status)
        || fishing_status(w.server_status)
        || w.tl_fish.active()
        || timeless_pose(w, w.my_motion.status)
}

/// /fish (FUN_1007abf0) after the status gate: "You cannot fish here." (group 7 0x49) without water ahead or in the
/// Cloister of Tides; nothing while the latch is set or already fishing; else c2s 0x01A category 0x0e and the latch.
pub fn command(w: &mut World) {
    let water = w.ground.as_ref().is_some_and(|g| g.water_ahead(w.pos, forward(w)));
    if !water || w.zone == Some(211) {
        super::textcmd::refuse(w, 0x49);
        return;
    }
    if w.fish_pending || fishing_status(w.my_motion.status) {
        return;
    }
    // Timeless XI: a practice round gives way to the real thing.
    if matches!(&w.tl_fish.play, Some(Play::Bar(r)) if r.practice) {
        w.tl_fish.play = None;
    }
    // Timeless XI: say this client plays the bar game right before the cast (a Timeless server refuses casts from
    // clients that never said it; any other server ignores it with no fish on the line).
    if send(w, super::timeless::fishing::HELLO, 5, 0) {
        w.tl_fish.hello_zone = w.zone;
    }
    super::textcmd::act(w, (w.my_id, w.my_index), 0x0e, 0);
    w.fish_pending = true;
}

pub(super) fn forward(w: &World) -> glam::Vec3 {
    glam::Vec3::new(w.yaw.sin(), 0.0, w.yaw.cos())
}

/// s2c 0x115: one fish a game (ignored while one is held); keen sense plays DAT 3705 `main` on us.
pub fn on_fish(w: &mut World, p: FishParams) {
    log::info!("fishing: 0x115 {p:?}");
    let Some(f) = w.fishing.as_mut() else { return };
    if f.game.is_some() {
        return;
    }
    let stamina = p.stamina as i32;
    f.game = Some(Game {
        max: stamina,
        cur: stamina,
        shown: stamina,
        delay: (p.delay as i32).max(1),
        regen: p.regen as i32 - 0x80,
        moves: (p.moves as i32).max(1) * 20,
        damage: p.damage as i32,
        heal: p.heal as i32,
        time: p.time as i32 * 60,
        warned: false,
        large: p.sense & 1 != 0,
        dir: 0,
        hit_timer: 0,
        special: p.special,
        gold: false,
    });
    if p.sense & 2 != 0 {
        let me = Who::Actor(w.my_id);
        w.effects.on_event_scheduler(0xaf0 + 0x389, name(b"main"), me, me);
    }
}

/// Timeless XI: s2c 0x115 from a Timeless server: the bar game (a fight) or the item box (an item or chest) instead
/// of the arrow game.
pub fn on_timeless(w: &mut World, b: tl::Bite) {
    log::info!("fishing: Timeless bite {b:?}");
    if w.tl_fish.active() {
        return;
    }
    w.tl_fish.end_due = None;
    w.tl_fish.pose = tl::POSE_CENTER;
    w.tl_fish.pose_at = 0.0;
    w.tl_fish.play = Some(if b.kind == 1 {
        Play::Item(Pending { bite: b, t: 0.0, warned: false })
    } else {
        let seed = w.started.elapsed().as_nanos() as u64 ^ (w.my_id as u64) << 32;
        Play::Bar(Round::new(b, false, seed))
    });
}

/// Timeless XI: `/tfish practice [0-100] [mixed|smooth|sinker|floater|dart] [monster]`: a practice round (no fish, no
/// packets) at that difficulty; ESC ends it.
pub fn practice(w: &mut World, args: &[&str]) {
    if w.tl_fish.active() || fishing_status(w.my_motion.status) {
        return;
    }
    let d = args.first().and_then(|a| a.parse::<u16>().ok()).unwrap_or(40);
    let beh = args.iter().find_map(|a| tl::BEHAVIOURS.iter().position(|b| a.eq_ignore_ascii_case(b))).unwrap_or(0);
    let monster = args.iter().any(|a| a.eq_ignore_ascii_case("monster"));
    let seed = w.started.elapsed().as_nanos() as u64 | 1;
    w.tl_fish.play = Some(Play::Bar(Round::practice(d, beh as u8, monster, seed)));
}

/// Timeless XI: one frame of the bar game or item box (the native game's frames don't run meanwhile). Steering is
/// the movement axis (the fishing statuses already hold the character in place); Escape gives up; Enter reels an
/// item in.
fn timeless_frame(w: &mut World, enter: bool, escape: bool, dir: f32, dt: f32) {
    let dt = dt.clamp(0.0, 0.05);
    let Some(play) = w.tl_fish.play.as_mut() else { return };
    let mut out: Vec<(i32, u8, u32)> = Vec::new();
    let mut end: Option<(i32, u32)> = None;
    let mut clear = false;
    match play {
        Play::Bar(r) => {
            if r.done.is_none() && escape {
                if r.practice {
                    clear = true;
                } else {
                    r.finish(Done::GiveUp);
                }
            }
            let steer = if dir == 0.0 { 0.0 } else { dir.signum() };
            r.advance(dt, steer);
            if std::mem::take(&mut r.warn_now) {
                out.push((tl::WARN, 5, 0));
            }
            if r.practice {
                clear |= r.done.is_some() && r.t - r.done_t > tl::SHOW_AFTER;
            } else if r.done.is_some() && !r.reported {
                r.reported = true;
                end = r.end_para().map(|p| (p, r.bite.special));
            }
            // The rod pose (mode 5 para 6/7/8) is not sent: the server shows it by rewriting our status to 52 / 53,
            // which this client reads as the end of fishing (the round vanished, the character could walk off).
            // Bring it back once the client plays those statuses as a fishing pose.
        }
        Play::Item(p) => {
            p.t += dt;
            let left = p.time_limit() - p.t;
            if enter {
                end = Some((tl::END_WIN, p.bite.special));
            } else if escape {
                end = Some((tl::END_GIVE_UP, p.bite.special));
            } else if left <= 0.0 {
                end = Some((tl::END_LOST, p.bite.special));
            } else if left <= 10.0 && !p.warned {
                p.warned = true;
                out.push((tl::WARN, 5, 0));
            }
            if end.is_some() {
                clear = true;
            }
        }
    }
    if clear {
        w.tl_fish.play = None;
    }
    for (para, m, para2) in out {
        // A pose or warning the send gate refuses is simply skipped (the next change sends again).
        let _ = send(w, para, m, para2);
    }
    if let Some(e) = end {
        w.tl_fish.end_due = Some(e);
    }
}

/// Timeless XI: sends a due end report (again each frame until the send gate lets it through); then the native
/// game's sub-states take over (the result, then the release).
fn timeless_end(w: &mut World, f: &mut Fishing) {
    let Some((para, special)) = w.tl_fish.end_due else { return };
    if send(w, para, 3, special) {
        w.tl_fish.end_due = None;
        super::event::cancel_queues(w);
        f.sub = 2;
    }
}

/// A mouse release in the field (key functions 0x8b / 0x8c mode 5).
pub fn click(w: &mut World) {
    if let Some(f) = w.fishing.as_mut() {
        f.click = true;
    }
}

/// Per frame from the key functions (FUN_10089100 for our player): the latch, the fishing model's state, then
/// retail's frames.
pub fn update(w: &mut World, input: &mut Input, dt: f32, window: bool) {
    let status = w.my_motion.status;
    if matches!(status, 1..=4) || fishing_status(status) {
        w.fish_pending = false;
    }
    // Timeless XI: the bar game's hello on each zone-in (and the practice round, which needs no fishing).
    if w.zone.is_some() && w.tl_fish.hello_zone != w.zone && send(w, tl::HELLO, 5, 0) {
        w.tl_fish.hello_zone = w.zone;
    }
    if !fishing_status(status) && !timeless_pose(w, status) {
        if matches!(&w.tl_fish.play, Some(Play::Bar(r)) if r.practice) {
            let escape = !window && input.test(0x0f, mode::PRESS);
            timeless_frame(w, false, escape, input.axis(4), dt);
        } else {
            w.tl_fish.play = None;
            w.tl_fish.end_due = None;
        }
        if let Some(mut f) = w.fishing.take() {
            clear_arrows(w, &mut f, false);
            w.fish_gauge = None;
            stop_all(w);
        }
        return;
    }
    // FUN_1008f740: the fishing model set up; the bite wait from 0x037, else 900..1500 ticks.
    let mut f = match w.fishing.take() {
        Some(f) => f,
        None => {
            let seed = w.started.elapsed().as_nanos() as u32 ^ w.my_id;
            let mut f = Fishing {
                sub: 0,
                early: false,
                lockout: 180,
                frames: 0,
                game: None,
                arrow: -1,
                acc: 0,
                t: 0,
                good: 0,
                bad: 0,
                latch: false,
                clear: 0,
                answered: false,
                rng: seed,
                step: 0.0,
                enter: false,
                enter_free: false,
                escape: false,
                click: false,
                axis: 0.0,
            };
            if w.hook_wait <= 0 {
                w.hook_wait = f.rand() % 601 + 900;
            }
            f
        }
    };
    // Key function 0x0e (Enter, mode 1) any time, with no window for the wait; 0x0f (Escape) with no window.
    let enter = input.test(0x0e, mode::PRESS);
    let escape = !window && input.test(0x0f, mode::PRESS);
    // Timeless XI: the bar game or item box replaces the arrow game's frames until its end report is sent.
    if w.tl_fish.active() || w.tl_fish.end_due.is_some() {
        timeless_frame(w, enter, escape, input.axis(4), dt);
        timeless_end(w, &mut f);
        f.step = 0.0;
        w.fishing = Some(f);
        return;
    }
    if enter {
        f.enter = true;
        f.enter_free |= !window;
    }
    f.escape |= escape;
    if let Some(Play::Bar(r)) = w.tl_fish.play.as_mut() {
        // A finished round stays on screen a moment ("Caught!").
        r.t += dt;
        if r.t - r.done_t > tl::SHOW_AFTER {
            w.tl_fish.play = None;
        }
    }
    let a = input.axis(4);
    if a != 0.0 {
        f.axis = a;
    }
    f.step += dt * 30.0;
    while f.step >= 1.0 {
        f.step -= 1.0;
        frame(w, &mut f);
        (f.enter, f.enter_free, f.escape, f.click, f.axis) = (false, false, false, false, 0.0);
    }
    w.fishing = Some(f);
}

fn running(w: &World, s: &[u8; 4]) -> bool {
    w.effects.actor_scheduler_running(w.my_id, name(s))
}

fn stop(w: &mut World, list: &[&[u8; 4]]) {
    for s in list {
        w.effects.stop_actor_scheduler(w.my_id, name(s));
        if let Some(rod) = super::fishrod::rod_of(w, w.my_id) {
            w.effects.stop_actor_scheduler(rod, name(s));
        }
    }
}

/// Starts `s` on the player model and its rod (FUN_10089850's +0xa4 calls).
fn start_both(w: &mut World, s: &[u8; 4]) {
    let me = w.my_id;
    w.effects.on_scheduler(me, me, name(s));
    if let Some(rod) = super::fishrod::rod_of(w, me) {
        w.effects.on_scheduler(rod, rod, name(s));
    }
}

fn stop_all(w: &mut World) {
    stop(w, &[b"fsh0", b"fsh1", b"fsh2", b"fsh3", b"fsh4", b"fsh5", b"fsh6", b"fsh7", b"fsh8", b"fsh9"]);
}

/// FUN_1008a080: DAT 3727's `maiN` on us; the clear counter to 1.
fn arrow_fx(w: &mut World, f: &mut Fishing, n: u32) {
    let digit = b"0123456789ab"[n.min(11) as usize];
    let me = Who::Actor(w.my_id);
    w.effects.on_event_scheduler(0xaf0 + 0x39f, u32::from_le_bytes([b'm', b'a', b'i', digit]), me, me);
    f.clear = 1;
}

/// FUN_10089f80: (gold x 2 - dir + 1) x 3 + k; k 0 show, 1 hit, 2 clear.
fn arrow_index(dir: u8, gold: bool, k: u32) -> u32 {
    ((gold as i32 * 2 - dir as i32 + 1) * 3) as u32 + k
}

/// FUN_10089ff0: the counter runs down; at 0 every arrow's clear plays (with `again`, once more 30 frames on).
fn clear_arrows(w: &mut World, f: &mut Fishing, again: bool) {
    if f.clear == 0 {
        return;
    }
    f.clear -= 1;
    if f.clear == 0 {
        for n in [2, 5, 8, 11] {
            arrow_fx(w, f, n);
        }
        f.clear = if again { 30 } else { 0 };
    }
}

/// FUN_10089e40: the movement axis, non-zero only on a press's first frame.
fn axis_edge(f: &mut Fishing) -> f32 {
    let v = f.axis;
    match (f.latch, v != 0.0) {
        (false, true) => {
            f.latch = true;
            v
        }
        (true, true) => 0.0,
        (_, false) => {
            f.latch = false;
            0.0
        }
    }
}

/// FUN_10177730 in the fight (no map: true).
fn water(w: &World) -> bool {
    w.ground.as_ref().is_none_or(|g| g.water_ahead(w.pos, forward(w)))
}

/// One retail frame of FUN_10089100's sub-states.
fn frame(w: &mut World, f: &mut Fishing) {
    let status = w.my_motion.status;
    match f.sub {
        // Waiting for a bite.
        0 => {
            // A result status: the end (and the rest of this case, as the asm falls through).
            if (0x39..=0x3e).contains(&status) {
                f.sub = 3;
            }
            f.clear = 0;
            if !f.early {
                if w.hook_wait > 0 {
                    w.hook_wait -= U;
                } else if send(w, 0, 2, 0) {
                    (f.sub, f.frames) = (1, 0);
                    return;
                }
            }
            if f.enter_free || f.click {
                f.early = true;
            }
            if f.lockout > 0 {
                f.lockout -= U;
                return;
            }
            // No FUN_10127ea0 after this one.
            if (f.escape || f.early) && send(w, 201, 3, 0) {
                f.sub = 3;
            }
        }
        // The bite check sent, then the fight.
        1 => {
            if (0x3a..=0x3e).contains(&status) {
                f.sub = 3;
                return;
            }
            let check = f.frames & 0x3f == 0;
            f.frames = f.frames.wrapping_add(1);
            if check && !water(w) && send(w, 200, 3, 0) {
                super::event::cancel_queues(w);
                f.sub = 3;
                return;
            }
            stop(w, &[b"fsh0"]);
            if f.game.is_none() {
                return;
            }
            tick(w, f);
            let Some(g) = f.game.as_mut() else { return };
            let sent = if f.enter {
                let pct = if g.max == 0 { 0 } else { (g.cur as f64 * 100.0 / g.max as f64) as i32 };
                let special = g.special;
                send(w, pct, 3, special)
            } else if !f.escape {
                g.time -= U;
                if g.time <= 0 {
                    send(w, 300, 3, 0)
                } else {
                    if !g.warned && g.time < 420 {
                        let left = g.time / 60;
                        if send(w, left, 5, 0) {
                            g.warned = true;
                        }
                    }
                    return;
                }
            } else {
                send(w, 200, 3, 0)
            };
            // FUN_10127ea0 after an admitted end (0x100894a1): the event message queues are emptied.
            if sent {
                super::event::cancel_queues(w);
                f.sub = 2;
            }
        }
        // The end sent: the result.
        2 => {
            if (0x3a..=0x3e).contains(&status) {
                f.sub = 3;
            } else if status != 0x39 {
                stop(w, &[b"fsh7", b"fsh8", b"fsh9"]);
                f.sub = 3;
            }
        }
        // Ending: the release once the result's motion is over.
        3 => {
            clear_arrows(w, f, true);
            stop(w, &[b"fsh0", b"fsh7", b"fsh8", b"fsh9"]);
            if fishing_status(status) && [b"fsh2", b"fsh3", b"fsh4", b"fsh5", b"fsh6"].iter().any(|s| running(w, s)) {
                return;
            }
            if send(w, 0, 4, 0) {
                f.sub = 4;
                stop_all(w);
                f.game = None;
            }
            w.fish_gauge = None;
            clear_arrows(w, f, true);
        }
        _ => {}
    }
}

/// FUN_10089850: the fight's frame.
fn tick(w: &mut World, f: &mut Fishing) {
    let Some(mut g) = f.game.take() else { return };
    if g.cur == 0 {
        // An exhausted fish: no arrows, no regen.
        clear_arrows(w, f, true);
    } else {
        if f.arrow == -1 {
            // DAT_10485ff2: music slot 7, 129 small / 136 large.
            w.music.set_slot(7, if g.large { 0x88 } else { 0x81 });
            f.arrow = 0;
        }
        if f.arrow == 0 {
            (f.acc, f.t, f.good, f.bad, f.arrow) = (0, 0, 0, 0, 1);
            // FUN_10089ee0: gold when special >= r1 + r2 (1..50 each).
            let r1 = (f.rand() % 51).max(1);
            let r2 = (f.rand() % 51).max(1);
            g.gold = g.special / (r1 + r2) as u32 != 0;
        }
        if f.arrow == 1 {
            f.t += U;
            // A key before the arrow: the fish gets back half its arrow heal.
            if f.t > 30 && axis_edge(f) != 0.0 {
                g.cur += (U as f32 * g.heal as f32 * 0.5) as i32;
                f.bad = 0;
            }
            f.acc += U * g.moves;
            if f.acc >= 6000 && g.cur != 0 {
                g.hit_timer = 0;
                g.dir = (f.rand() & 1) as u8;
                f.acc = (f.rand() % (2 * g.delay) + 2) * 10;
                stop(w, &[b"fsh7", b"fsh8", b"fsh9"]);
                start_both(w, if g.dir == 0 { b"fsh9" } else { b"fsh8" });
                arrow_fx(w, f, arrow_index(g.dir, g.gold, 0));
                (f.t, f.good, f.bad) = (0, 0, 0);
                if g.gold {
                    f.acc = (f.acc as f32 * 0.7) as i32;
                }
                (f.arrow, f.answered) = (2, false);
                log::debug!(
                    "fishing: arrow {} gold {} for {} ticks",
                    if g.dir == 0 { "right" } else { "left" },
                    g.gold,
                    f.acc
                );
            }
        }
        if f.arrow == 2 {
            g.hit_timer -= U;
            if g.hit_timer < 0 {
                w.effects.on_scheduler(w.my_id, w.my_id, name(if g.large { b"hitl" } else { b"hits" }));
                g.hit_timer = (f.rand() % 6 + 5) * 60;
            }
            f.acc -= U;
            if f.acc <= 0 {
                f.acc = 0;
                stop(w, &[b"fsh8", b"fsh9"]);
                start_both(w, b"fsh7");
                f.arrow = 0;
                clear_arrows(w, f, true);
                if !f.answered {
                    log::debug!("fishing: arrow missed, stamina {}", g.cur);
                }
            } else {
                let v = axis_edge(f);
                let right = if g.dir == 0 { v > 0.0 } else { v < 0.0 };
                let wrong = if g.dir == 0 { v < 0.0 } else { v > 0.0 };
                f.good += right as u32;
                f.bad += wrong as u32;
                if f.good > f.bad {
                    g.cur -= g.damage * if g.gold { 2 } else { 1 };
                    f.acc = 0;
                    arrow_fx(w, f, arrow_index(g.dir, g.gold, 1));
                    (f.good, f.bad, f.answered) = (0, 0, true);
                    log::debug!("fishing: arrow hit, stamina {}", g.cur.max(0));
                } else if f.bad > f.good {
                    g.cur += (if g.gold { (g.heal as f32 * 0.25) as i32 } else { g.heal }).max(1);
                    f.acc = 0;
                    arrow_fx(w, f, arrow_index(g.dir, g.gold, 2));
                    (f.good, f.bad, f.answered) = (0, 0, true);
                    log::debug!("fishing: wrong key, stamina {}", g.cur);
                }
            }
        }
        g.cur = (g.cur + g.regen * U).clamp(0, g.max);
    }
    // The shown value eases by 0.15 of the gap (all of it within 1); the party window's +0x3a percent.
    if g.max != 0 {
        let d = g.cur - g.shown;
        let step = d as f32 * 0.15;
        g.shown = (g.shown as f32 + if step.abs() <= 1.0 { d as f32 } else { step }) as i32;
        w.fish_gauge = Some((g.shown as i64 * 100 / g.max as i64).clamp(0, 100) as u8);
    }
    f.game = Some(g);
}

/// The fish's gauge (percent) while it shows: fishing, no event holding the UI.
pub(super) fn gauge(w: &World) -> Option<u8> {
    w.fish_gauge.filter(|_| fishing_status(w.my_motion.status) && !super::event::active(w))
}

/// FUN_10202af0 at the end of the party window's draw: "Fish" and its gauge at (W / 2 - 58, 0.2 H), while fishing
/// and no event holds the UI.
pub fn draw_gauge(gu: &mut GameUi, w: &World, sw: f32, sh: f32) {
    let Some(pct) = gauge(w).filter(|_| !w.modern) else { return };
    let (x, y) = ((sw / 2.0).trunc() - 58.0, (sh / 2.0 * 0.4).trunc());
    // FUN_101cb0d0: windowps 97 at 2 x pct / 100 wide from (4, 1), framesus 98 dimmed at 0.
    if pct > 0 {
        gu.sprite_scaled_blend("menu    windowps", 97, x + 4.0, y + 8.0, 2.0 * pct as f32 / 100.0, 1.0, 0x8080_8080, 8);
    }
    gu.sprite("menu    framesus", 98, x, y + 7.0, if pct == 0 { 0x3050_5050 } else { 0x8080_8080 });
    let tint = gu.ncol(if pct > 0 { 0 } else { 9 });
    gu.shape_fit("Fish", x, y, 102.0, tint);
}
