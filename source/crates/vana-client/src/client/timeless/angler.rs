//! Timeless XI: Timeless Angler, built into the client (it was the Ashita add-on timelessangler 1.39). The fishing
//! HUD (the pool you stand at, rod and bait, the fish they hook here with the real chance per cast, skill, moon and
//! bite time, the Fishing Spot Fatigue Tracker, the session) and the school of fish banner. Drawn by
//! `client/modern/tlangler.rs` in every interface.
//!
//! Everything mirrors the Timeless XI server: `src/map/utils/fishingutils.cpp` (Phoenix core + TIMELESS patches) and
//! `modules/timeless_fishing` (bar game, schools of fish). The fishing tables are the server's, embedded as
//! `assets/timeless/angler.json` (`tools/timeless/angler_data.py <server> <out>`; run it again after a fishing table
//! changes, then rebuild). The fishing area test follows GoFish by StrixNivea (BSD), rewritten to the server's code.
//!
//! The fishing spot is a mirror: the server keeps the real one. Casts are counted when our status turns 56 (a cast
//! the server took); the server's lines ("hit a school", "moved on", "exhausted", "let the school ... swim off") and
//! the catch lines come from the chat log's tap (`ChatLog::tap`).

use super::super::World;
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use std::sync::OnceLock;

// ---------------------------------------------------------------------------------------------------------------
// The server's fishing tables
// ---------------------------------------------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct Fish {
    pub n: String,
    pub lv: i32,
    /// 0 small, 1 large.
    pub sz: i32,
    pub lg: i32,
    /// 1: an item, not a fish.
    pub it: i32,
    pub rar: i32,
    pub hp: i32,
    pub mp: i32,
    pub mo: i32,
    pub fl: i32,
    pub ki: u16,
    pub qo: i32,
}

#[derive(Deserialize)]
pub struct Bait {
    pub n: String,
    /// 1: a lure.
    pub t: i32,
    pub fl: i32,
}

#[derive(Deserialize)]
pub struct Rod {
    pub n: String,
    pub sz: i32,
    pub lg: i32,
}

#[derive(Deserialize)]
pub struct Area {
    pub n: String,
    /// 0 whole zone, 1 cylinder, 2 polygon.
    pub bt: i32,
    pub bh: f32,
    pub br: f32,
    pub cx: f32,
    pub cy: f32,
    pub cz: f32,
    pub g: u32,
    #[serde(default)]
    pub pts: Vec<[f32; 3]>,
}

#[derive(Deserialize)]
pub struct Mob {
    pub n: String,
    pub lv: i32,
    pub a: u32,
    pub nm: i32,
    pub qo: i32,
}

#[derive(Deserialize)]
pub struct Zone {
    pub n: String,
    pub city: bool,
    pub areas: BTreeMap<u32, Area>,
    pub mobs: Vec<Mob>,
}

#[derive(Deserialize)]
pub struct Data {
    pub built: String,
    pub fish: HashMap<u16, Fish>,
    pub bait: HashMap<u16, Bait>,
    pub aff: HashMap<u16, HashMap<u16, i32>>,
    pub rods: HashMap<u16, Rod>,
    pub groups: HashMap<u32, Vec<u16>>,
    pub zones: HashMap<u16, Zone>,
    pub sell: HashMap<u16, u32>,
    pub gear: HashMap<u16, i32>,
}

/// The tables, read once.
pub fn data() -> &'static Data {
    static D: OnceLock<Data> = OnceLock::new();
    D.get_or_init(|| {
        serde_json::from_str(include_str!("../../../assets/timeless/angler.json")).expect("assets/timeless/angler.json")
    })
}

// ---------------------------------------------------------------------------------------------------------------
// Vana'diel time and the moon
// ---------------------------------------------------------------------------------------------------------------

pub const MOON_NEW: u8 = 0;
pub const MOON_FULL: u8 = 4;
pub const MOON_NAMES: [&str; 8] = [
    "New Moon",
    "Waxing Crescent",
    "First Quarter",
    "Waxing Gibbous",
    "Full Moon",
    "Waning Gibbous",
    "Last Quarter",
    "Waning Crescent",
];

#[derive(Clone, Copy, Debug, Default)]
pub struct VDate {
    pub hour: i32,
    pub month: i32,
    /// 0..100.
    pub moon_pct: i32,
    /// 0 turning, 1 waning, 2 waxing.
    pub moon_dir: i32,
    /// The server's moon bucket (fishingutils GetMoonPhase): 0 new .. 4 full .. 7 waning crescent.
    pub moon: u8,
}

/// The date from the client's clock (Earth seconds since the Vana'diel epoch, `vana_now`), its moon exactly as the
/// server works it out (common/vana_time.h moon::get_phase / get_direction: days since year 0 + 26, mod 84; the epoch
/// is year 886, so that is the clock's day + 38): Full at 0, New at 42.
pub fn vdate(t: u64) -> VDate {
    let p = crate::dat::msgfmt::VanaParts::of(t);
    let m = ((t / 3456 + 38) % 84) as f64;
    let pct = if m >= 42.0 { (100.0 * ((m - 42.0) / 42.0)).round() } else { (100.0 * (1.0 - m / 42.0)).round() } as i32;
    let dir = if m == 42.0 || m == 0.0 {
        0
    } else if m < 42.0 {
        1
    } else {
        2
    };
    VDate { hour: p.hour, month: p.month, moon_pct: pct, moon_dir: dir, moon: moon_bucket(pct, dir) }
}

/// fishingutils::GetMoonPhase, exactly (gaps fall back to 0 = New Moon, as on the server).
pub fn moon_bucket(phase: i32, direction: i32) -> u8 {
    if phase <= 5 || (phase <= 10 && direction == 1) {
        return 0;
    }
    if (7..=38).contains(&phase) && direction == 2 {
        return 1;
    }
    if (40..=55).contains(&phase) && direction == 2 {
        return 2;
    }
    if (57..=88).contains(&phase) && direction == 2 {
        return 3;
    }
    if phase >= 95 || (phase >= 90 && direction == 2) {
        return 4;
    }
    if (62..=93).contains(&phase) && direction == 1 {
        return 5;
    }
    if (45..=60).contains(&phase) && direction == 1 {
        return 6;
    }
    if (12..43).contains(&phase) && direction == 1 {
        return 7;
    }
    0
}

// ---------------------------------------------------------------------------------------------------------------
// Catch patterns (fishingutils.h)
// ---------------------------------------------------------------------------------------------------------------

fn cosp(x: f32, a: f32, b: f32) -> f32 {
    (0.5 * (a * x + b).cos() + 0.5).clamp(0.0, 1.0)
}

fn moon_pat(pat: i32, x: f32) -> Option<f32> {
    Some(match pat {
        1 => cosp(x, 1.75, 0.10),
        2 => cosp(x, 1.75, 3.30),
        3 => (1.0 - x / 7.0).clamp(0.0, 1.0),
        4 => cosp(x, 0.90, 3.14),
        5 => cosp(x, 0.90, 0.00),
        _ => return None,
    })
}

pub fn hour_mod(pat: i32, hour: i32) -> f32 {
    let h = hour as f32;
    let m = match pat {
        1 => cosp(h, 0.82, 0.16),
        2 => (hour != 5 && hour != 17) as i32 as f32 * 0.5 + 0.5,
        3 => (hour == 5 || hour == 17) as i32 as f32 * 0.5 + 0.5,
        4 => (hour > 19 || hour < 4) as i32 as f32 * 0.5 + 0.5,
        5 => cosp(h, 0.60, 3.50),
        6 => cosp(h, 0.53, 0.00),
        7 => cosp(h, 0.23, 3.53),
        _ => 0.5,
    };
    m + 0.25
}

pub fn moon_mod(pat: i32, moon: u8) -> f32 {
    moon_pat(pat, moon as f32).unwrap_or(1.0) + 0.25
}

pub fn month_mod(pat: i32, month: i32) -> f32 {
    const M: [(f32, f32); 10] = [
        (0.40, -1.60),
        (0.60, -1.00),
        (0.50, 3.05),
        (1.04, 0.00),
        (0.40, 3.50),
        (0.90, -2.00),
        (0.49, 1.63),
        (1.04, -2.60),
        (0.49, -1.25),
        (0.50, 0.53),
    ];
    let m = if (1..=10).contains(&pat) {
        let (a, b) = M[pat as usize - 1];
        cosp((month - 1) as f32, a, b)
    } else {
        0.5
    };
    m + 0.25
}

pub const HOUR_TEXT: [&str; 8] = [
    "Any time",
    "Night, morning and afternoon",
    "Any hour but 05:00 / 17:00",
    "Turn of the tide (05:00 / 17:00)",
    "Night (20:00-03:00)",
    "Morning and afternoon",
    "Day and night peaks",
    "Daytime",
];
pub const MOON_TEXT: [&str; 6] = [
    "Any moon",
    "New and Full Moon",
    "Half moons (not New / Full)",
    "New Moon, fading toward Full",
    "Full Moon",
    "New Moon",
];
pub const MONTH_TEXT: [&str; 11] = [
    "All year",
    "Spring and summer",
    "Spring and winter",
    "Summer",
    "Winter and summer",
    "Summer and fall",
    "Late spring and late fall",
    "Fall",
    "Spring and fall",
    "Spring",
    "Winter",
];

/// A pattern's text (out of range: the first).
pub fn pattern_text(list: &[&'static str], pat: i32) -> &'static str {
    list.get(pat.max(0) as usize).copied().unwrap_or(list[0])
}

/// The moon's multipliers on the catch mix (0..1, 1 = the moon's best for that pool): fish, items, monsters.
pub fn moon_pools(moon: u8) -> (f32, f32, f32) {
    let x = moon as f32;
    (moon_pat(4, x).unwrap_or(1.0), moon_pat(2, x).unwrap_or(1.0), moon_pat(3, x).unwrap_or(1.0))
}

// ---------------------------------------------------------------------------------------------------------------
// Fishing areas (fishingutils isInsideCylinder / isInsidePoly, the first match by area id)
// ---------------------------------------------------------------------------------------------------------------

#[derive(Clone, Copy)]
struct P {
    x: f32,
    z: f32,
}

fn on_segment(p: P, q: P, r: P) -> bool {
    q.x <= p.x.max(r.x) && q.x >= p.x.min(r.x) && q.z <= p.z.max(r.z) && q.z >= p.z.min(r.z)
}

fn orientation(p: P, q: P, r: P) -> i32 {
    let v = (q.z - p.z).round() * (r.x - q.x) - (q.x - p.x) * (r.z - q.z);
    if v == 0.0 {
        0
    } else if v > 0.0 {
        1
    } else {
        2
    }
}

fn intersect(p1: P, q1: P, p2: P, q2: P) -> bool {
    let (o1, o2, o3, o4) =
        (orientation(p1, q1, p2), orientation(p1, q1, q2), orientation(p2, q2, p1), orientation(p2, q2, q1));
    (o1 != o2 && o3 != o4)
        || (o1 == 0 && on_segment(p1, p2, q1))
        || (o2 == 0 && on_segment(p1, q2, q1))
        || (o3 == 0 && on_segment(p2, p1, q2))
        || (o4 == 0 && on_segment(p2, q1, q2))
}

fn inside_poly(a: &Area, x: f32, y: f32, z: f32) -> bool {
    let half = (a.bh / 2.0).floor();
    if y < a.cy - half || y > a.cy + half || a.pts.len() < 3 {
        return false;
    }
    let poly: Vec<P> = a.pts.iter().map(|p| P { x: p[0], z: p[2] }).collect();
    let n = poly.len();
    let p = P { x, z };
    // As the server has it: the ray's far point is (MAX_POINTS, y = p.z, z = 0), and only x and z are compared.
    let extreme = P { x: 10000.0, z: 0.0 };
    let (mut count, mut i) = (0, 0);
    loop {
        let nx = (i + 1) % n;
        if intersect(poly[i], poly[nx], p, extreme) {
            if orientation(poly[i], p, poly[nx]) == 0 {
                return on_segment(poly[i], p, poly[nx]);
            }
            count += 1;
        }
        i = nx;
        if i == 0 {
            break;
        }
    }
    count % 2 == 1
}

fn inside_cyl(a: &Area, x: f32, y: f32, z: f32) -> bool {
    let half = (a.bh / 2.0).floor();
    if y < a.cy - half || y > a.cy + half {
        return false;
    }
    let (dx, dz) = ((x - a.cx).abs(), (z - a.cz).abs());
    if dx > a.br || dz > a.br {
        return false;
    }
    dx + dz <= a.br || dx * dx + dz * dz <= a.br * a.br
}

/// The fishing area at a point (server coordinates).
pub fn find_area(zone: &Zone, x: f32, y: f32, z: f32) -> Option<(u32, &Area)> {
    zone.areas.iter().find_map(|(id, a)| {
        let hit = match a.bt {
            0 => true,
            1 => inside_cyl(a, x, y, z),
            2 => inside_poly(a, x, y, z),
            _ => false,
        };
        hit.then_some((*id, a))
    })
}

// ---------------------------------------------------------------------------------------------------------------
// Hook chances (fishingutils CalculateHookChance / FishingCheck with the TIMELESS no-empty-catch patch)
// ---------------------------------------------------------------------------------------------------------------

const BAIT_LURE: i32 = 1;
const BAITFLAG_POOR_FISH: i32 = 0x08;
const BAITFLAG_SHELLFISH: i32 = 0x40;
pub const FISHERMANS_APRON: u16 = 14400;
pub const FISHERMANS_SMOCK: u16 = 11337;
pub const FISHERS_ROPE: u16 = 11768;

/// Lu Shang's (and +1) / Ebisu (and +1): the low-fish bite bonus' family (the +1 rods count as their originals).
fn rod_family(rod: u16) -> Option<(i32, i32)> {
    match rod {
        17386 | 19320 => Some((10, 15)),
        17011 | 19321 => Some((15, 13)),
        _ => None,
    }
}

fn has(flags: i32, f: i32) -> bool {
    flags & f != 0
}

#[derive(Clone, Copy)]
pub struct When {
    pub skill: i32,
    pub hour: i32,
    pub moon: u8,
    pub month: i32,
}

pub fn hook_chance(d: &Data, fish: u16, bait: u16, rod: Option<&Rod>, c: When) -> i32 {
    let (Some(f), Some(b)) = (d.fish.get(&fish), d.bait.get(&bait)) else { return 0 };
    let power = d.aff.get(&fish).and_then(|a| a.get(&bait)).copied().unwrap_or(0);
    let modifier =
        ((moon_mod(f.mp, c.moon) * 3.0 + hour_mod(f.hp, c.hour) * 2.0 + month_mod(f.mo, c.month)) / 3.0).max(0.0);
    let mut hc = (25.0 * modifier).floor() as i32;
    let lure = b.t == BAIT_LURE;
    hc += match power {
        1 => {
            if lure {
                30
            } else {
                35
            }
        }
        2 => {
            if lure {
                60
            } else {
                65
            }
        }
        3 => {
            if lure {
                75
            } else {
                80
            }
        }
        _ => 0,
    };
    if f.lv > c.skill {
        hc -= (((f.lv - c.skill) as f32 * 0.25).floor() as i32).clamp(0, hc.max(0));
    }
    if c.skill - 10 > f.lv {
        hc -= (((c.skill - 10 - f.lv) as f32 * 0.15).floor() as i32).clamp(0, hc.max(0));
    }
    if let Some(r) = rod.filter(|r| r.lg != 1) {
        if f.sz < r.sz {
            hc -= 3.clamp(0, hc.max(0));
        } else if r.sz < f.sz {
            hc -= 5.clamp(0, hc.max(0));
        }
    }
    if has(b.fl, BAITFLAG_SHELLFISH) && has(f.fl, 1) {
        hc += 50;
    }
    if f.rar < 1000 {
        hc = hc * f.rar / 1000;
    }
    hc.clamp(20, 120)
}

/// One fish this bait hooks here.
#[derive(Clone, Debug)]
pub struct Hooked {
    pub id: u16,
    pub chance: i32,
    /// Its share of the fish pool.
    pub share: f32,
    /// Its chance per cast (fish, items and monsters together = 1).
    pub overall: f32,
}

#[derive(Clone, Debug, Default)]
pub struct Spot {
    pub fish: Vec<Hooked>,
    /// Fish here this bait doesn't hook.
    pub other: Vec<u16>,
    pub items: Vec<u16>,
    pub mobs: usize,
    /// Fish / items / monsters per cast.
    pub mix: (f32, f32, f32),
    pub nothing: bool,
}

pub struct SpotCtx {
    pub zone: u16,
    pub area: u32,
    pub when: When,
    pub bait: u16,
    pub rod: u16,
    pub weather: u16,
    pub body: u16,
}

/// What this spot gives for the gear in hand (the add-on's L.spot).
pub fn spot(d: &Data, ctx: &SpotCtx, has_ki: &dyn Fn(u16) -> bool) -> Spot {
    let mut res = Spot::default();
    let Some(zone) = d.zones.get(&ctx.zone) else { return res };
    let Some(area) = zone.areas.get(&ctx.area) else { return res };
    let rod = d.rods.get(&ctx.rod);
    let bait = d.bait.get(&ctx.bait);
    let c = ctx.when;
    let mp = c.moon as f32;
    let m = |pat: i32| moon_pat(pat, mp).unwrap_or(1.0);
    let (mut fish_w, mut item_w, mut mob_w) = if zone.city {
        ((15.0 * m(4)).floor() as i32, 25 + (20.0 * m(2)).floor() as i32, 0)
    } else {
        ((25.0 * m(4)).floor() as i32, 10 + (15.0 * m(2)).floor() as i32, 15 + (15.0 * m(3)).floor() as i32)
    };
    let (mut total, mut maxc) = (0, 0);
    for &id in d.groups.get(&area.g).map_or(&[][..], |g| g.as_slice()) {
        let Some(f) = d.fish.get(&id) else { continue };
        if f.it == 0 {
            let ki_ok = f.ki == 0 || has_ki(f.ki);
            let hooks = bait.is_some() && d.aff.get(&id).is_some_and(|a| a.contains_key(&ctx.bait));
            if hooks && ki_ok {
                let hc = hook_chance(d, id, ctx.bait, rod, c);
                res.fish.push(Hooked { id, chance: hc, share: 0.0, overall: 0.0 });
                total += hc;
                maxc = maxc.max(hc);
            } else {
                res.other.push(id);
            }
        } else if f.qo == 0 {
            res.items.push(id);
        }
    }
    res.mobs = zone.mobs.iter().filter(|m| m.nm == 0 && m.qo == 0 && (m.a == 0 || m.a == ctx.area)).count();
    if !res.fish.is_empty() {
        fish_w = (maxc + fish_w).clamp(10, 120);
    }
    let wmod = match ctx.weather {
        6 => 1.1,
        7 => 1.2,
        _ => 1.0,
    };
    fish_w = (fish_w as f32 * wmod).floor() as i32;
    if (ctx.body == FISHERMANS_APRON || ctx.body == FISHERMANS_SMOCK) && item_w > 0 {
        item_w -= (item_w as f32 * 0.25).floor() as i32;
    }
    if bait.is_some_and(|b| has(b.fl, BAITFLAG_POOR_FISH)) && fish_w > 0 {
        fish_w -= (fish_w as f32 * 0.25).floor() as i32;
        item_w = fish_w + (fish_w as f32 * 0.10).floor() as i32;
    }
    // TIMELESS: a pool only counts if something in it can be picked; no empty catches.
    if res.fish.is_empty() {
        fish_w = 0;
    }
    if res.items.is_empty() {
        item_w = 0;
    }
    if res.mobs == 0 {
        mob_w = 0;
    }
    let all = fish_w + item_w + mob_w;
    res.nothing = all == 0;
    if all > 0 {
        res.mix = (fish_w as f32 / all as f32, item_w as f32 / all as f32, mob_w as f32 / all as f32);
    }
    // Lu Shang / Ebisu: once a fish is picked, FishPoolWeight gets initialBonus + uint8(...) for that cast (only when
    // skill > fish level + 7); the uint8 wraps at 256 on the server, copied here.
    let fam = rod_family(ctx.rod);
    let (mut mi, mut mm, mut mf) = (0.0, 0.0, 0.0);
    for e in res.fish.iter_mut() {
        let f = &d.fish[&e.id];
        e.share = if total > 0 { e.chance as f32 / total as f32 } else { 0.0 };
        let mut add = 0;
        if let Some((ib, dv)) = fam
            && c.skill > f.lv + 7
        {
            let diff = c.skill - f.lv;
            add = ib + ((diff * (1 + diff / dv)) / (f.sz + 1)) % 256;
        }
        let denom = (fish_w + add + item_w + mob_w) as f32;
        if fish_w > 0 && denom > 0.0 {
            e.overall = e.share * (fish_w + add) as f32 / denom;
            mi += e.share * item_w as f32 / denom;
            mm += e.share * mob_w as f32 / denom;
            mf += e.overall;
        }
    }
    if fish_w > 0 && !res.fish.is_empty() {
        res.mix = (mf, mi, mm);
    }
    res.fish.sort_by(|a, b| b.overall.total_cmp(&a.overall));
    res
}

/// Chance per cast: a school hooks only fish (the fish's shares); an exhausted spot gives half the casts no bite.
pub struct Odds {
    pub fish: HashMap<u16, f32>,
    pub item: f32,
    pub mob: f32,
    pub none: f32,
}

pub fn cast_odds(sp: &Spot, school: bool, exhausted: bool) -> Odds {
    let mut o = Odds { fish: HashMap::new(), item: 0.0, mob: 0.0, none: 0.0 };
    if sp.nothing {
        o.none = 1.0;
        return o;
    }
    if school && !sp.fish.is_empty() {
        o.fish = sp.fish.iter().map(|e| (e.id, e.share)).collect();
        return o;
    }
    let k = if exhausted { 0.5 } else { 1.0 };
    o.fish = sp.fish.iter().map(|e| (e.id, e.overall * k)).collect();
    (o.item, o.mob, o.none) = (sp.mix.1 * k, sp.mix.2 * k, 1.0 - k);
    o
}

/// fishingutils::GetHookTime: seconds from the cast to the bite (a school halves it, at least 4).
pub fn bite_wait(moon: u8, hour: i32, waist: u16, school: bool) -> i32 {
    let mut w = 13;
    if moon == MOON_NEW || moon == MOON_FULL {
        w -= 4;
    }
    if hour == 5 || hour == 17 {
        w -= 1;
    }
    if waist == FISHERS_ROPE {
        w -= 1;
    }
    w = w.max(7);
    if school {
        w = ((w as f32 * 0.5 + 0.5).floor() as i32).max(4);
    }
    w
}

// ---------------------------------------------------------------------------------------------------------------
// The Timeless bar game's difficulty (timeless_fishing gapOf / computeParams)
// ---------------------------------------------------------------------------------------------------------------

fn high_rod_bonus(rod: u16) -> f32 {
    match rod {
        17386 => 3.0,
        19320 => 4.0,
        17011 => 5.0,
        19321 => 6.0,
        _ => 0.0,
    }
}

fn round_away(x: f32) -> i32 {
    if x >= 0.0 { (x + 0.5).floor() as i32 } else { -((-x + 0.5).floor() as i32) }
}

/// Levels the fish is above the angler (negative = below): gear counts a second time; each level over 100 counts 0.4,
/// gear and a legendary rod (against the over-100 part) take it down.
pub fn gap(d: &Data, fish: u16, skill: i32, gear: i32, rod: u16) -> i32 {
    let Some(f) = d.fish.get(&fish) else { return 0 };
    let gear = gear.max(0) as f32;
    if f.it == 1 || f.lv <= 100 {
        return round_away(f.lv as f32 - skill as f32 - gear);
    }
    let over = 0.4 * (f.lv - 100) as f32;
    let bonus = gear + high_rod_bonus(rod).min(over);
    round_away(100.0 + over - bonus - skill as f32)
}

pub fn difficulty(d: &Data, fish: u16, skill: i32, gear: i32, rod: u16) -> i32 {
    let Some(f) = d.fish.get(&fish) else { return 40 };
    let mut v = 40.0 + 1.5 * gap(d, fish, skill, gear, rod) as f32;
    if f.sz == 1 {
        v += 8.0;
    }
    if f.lg == 1 {
        v += 8.0;
    }
    (v.clamp(5.0, 100.0) + 0.5).floor() as i32
}

/// The label and its step (1 easy .. 5 very hard).
pub fn difficulty_label(v: i32) -> (&'static str, u8) {
    match v {
        ..=40 => ("Easy", 1),
        41..=55 => ("Normal", 2),
        56..=70 => ("Tricky", 3),
        71..=85 => ("Hard", 4),
        _ => ("Very hard", 5),
    }
}

/// Fish 30+ levels below the angler land 2 per catch, 50+ land 3 (stack permitting).
pub fn multi_catch(d: &Data, fish: u16, skill: i32, gear: i32, rod: u16) -> u32 {
    match -gap(d, fish, skill, gear, rod) {
        50.. => 3,
        30..=49 => 2,
        _ => 1,
    }
}

/// fishingutils::FishingSkillup: the chance (0..1) of a skill-up on a win, from the REAL skill (no gear).
pub fn skillup_chance(fish_lv: i32, real: i32, city: bool) -> f32 {
    if fish_lv <= real || fish_lv - real > 50 {
        return 0.0;
    }
    let diff = (fish_lv - real) as f32;
    let nd = (-0.5 * (2.0 * std::f32::consts::PI).ln() - 5f32.ln() - (diff - 11.0).powi(2) / 50.0).exp();
    let mx = ((nd * 200.0).floor() as i32 + (100 - real) / 10 - real / 10).max(4);
    let mut roll = 90;
    if !city {
        roll -= 10;
    }
    if real < 50 {
        roll -= 20 - real / 3;
    }
    (mx as f32 / roll as f32).clamp(0.0, 1.0)
}

/// Where a fish or item bites: (zone, area name), by zone name.
pub fn fish_where(d: &Data, fish: u16) -> Vec<(u16, String)> {
    let mut out: Vec<(u16, String)> = d
        .zones
        .iter()
        .flat_map(|(&zid, z)| {
            z.areas
                .values()
                .filter(|a| d.groups.get(&a.g).is_some_and(|g| g.contains(&fish)))
                .map(move |a| (zid, a.n.clone()))
        })
        .collect();
    out.sort_by(|a, b| d.zones[&a.0].n.cmp(&d.zones[&b.0].n).then(a.1.cmp(&b.1)));
    out
}

// ---------------------------------------------------------------------------------------------------------------
// Schools of fish: a mirror of the server's per-character fishing spot
// ---------------------------------------------------------------------------------------------------------------

pub const SPOT_RADIUS: f32 = 20.0;
pub const SPOT_IDLE: u64 = 600;
pub const EXHAUSTED_AT: u32 = 76;
pub const SCHOOL_IDLE: u64 = 300;
pub const SCHOOL_MIN: u32 = 6;
pub const SCHOOL_MAX: u32 = 12;
pub const POND_CHANCE: f32 = 0.05;
pub const POND_COOLDOWN: u64 = 300;

/// Pond zones (server isPondZone): one spot for the whole pool, never exhausted, 5 % a cast, a new school 5 minutes
/// after the last one ended. Keep in step with timeless_fishing.cpp.
pub fn is_pond(zone: u16) -> bool {
    matches!(zone, 142 | 154 | 157 | 158 | 159 | 166 | 174 | 184 | 196 | 208)
}

pub fn school_chance(cast: u32) -> f32 {
    match cast {
        ..=15 => 0.08,
        16..=40 => 0.05,
        41..=75 => 0.03,
        _ => 0.0,
    }
}

#[derive(Clone, Debug, Default)]
pub struct Hole {
    pub zone: u16,
    pub x: f32,
    pub z: f32,
    pub casts: u32,
    pub last: u64,
    /// This spot's one school has come.
    pub used: bool,
    pub active: bool,
    pub school_casts: u32,
    pub school_catches: u32,
    /// A pond's next school, not before (survives a new spot).
    pub next_school: u64,
    /// The last school's casts and catches.
    pub last_school: Option<(u32, u32)>,
}

impl Hole {
    pub fn end_school(&mut self, now: u64) {
        if self.active {
            self.last_school = Some((self.school_casts, self.school_catches));
        }
        self.active = false;
        self.next_school = now + POND_COOLDOWN;
    }

    fn far(&self, zone: u16, x: f32, z: f32) -> bool {
        let (dx, dz) = (x - self.x, z - self.z);
        !is_pond(zone) && dx * dx + dz * dz >= SPOT_RADIUS * SPOT_RADIUS
    }

    /// A cast here would start a new spot.
    pub fn fresh(&self, zone: u16, x: f32, z: f32, now: u64) -> bool {
        self.casts == 0 || self.zone != zone || self.far(zone, x, z) || now.saturating_sub(self.last) >= SPOT_IDLE
    }

    /// A running school ends at once on a zone change, 20+ yalms off the spot or 5 minutes without a cast.
    pub fn check(&mut self, zone: u16, x: f32, z: f32, now: u64) -> Option<&'static str> {
        if !self.active {
            return None;
        }
        let why = if self.zone != zone {
            "you changed zones"
        } else if self.far(zone, x, z) {
            "you left the fishing spot"
        } else if now.saturating_sub(self.last) >= SCHOOL_IDLE {
            "no casts for 5 minutes"
        } else {
            return None;
        };
        self.end_school(now);
        Some(why)
    }

    /// One cast the server took; true when it started a new spot.
    pub fn cast(&mut self, zone: u16, x: f32, z: f32, now: u64) -> bool {
        let fresh = self.fresh(zone, x, z, now);
        if fresh {
            *self = Hole { zone, x, z, next_school: self.next_school, ..Default::default() };
        }
        self.casts += 1;
        self.last = now;
        if self.active {
            self.school_casts += 1;
        }
        fresh
    }

    /// The server's fishing lines.
    pub fn message(&mut self, text: &str, now: u64) -> Option<Msg> {
        if text.contains("hit a school of fish") {
            (self.active, self.used, self.school_casts, self.school_catches) = (true, true, 1, 0);
            Some(Msg::Hit)
        } else if text.contains("school of fish has moved on") {
            if !self.active {
                return Some(Msg::Quiet);
            }
            self.end_school(now);
            Some(Msg::Gone)
        } else if text.contains("let the school of fish swim off") {
            if self.active {
                self.end_school(now);
            }
            Some(Msg::LetGo)
        } else if text.contains("appears to be exhausted") {
            self.casts = self.casts.max(EXHAUSTED_AT);
            self.active = false;
            Some(Msg::Exhausted)
        } else {
            None
        }
    }

    /// The next cast's school chance and what the spot is doing.
    pub fn odds(&self, zone: u16, x: f32, z: f32, now: u64) -> HoleOdds {
        let fresh = self.fresh(zone, x, z, now);
        let next = if fresh { 1 } else { self.casts + 1 };
        let pond = is_pond(zone);
        let mut o = HoleOdds { next, fresh, pond, ..Default::default() };
        if self.active && !fresh {
            (o.now, o.ahead) = (1.0, 1.0);
        } else if pond {
            o.wait = self.next_school.saturating_sub(now);
            o.now = if o.wait > 0 { 0.0 } else { POND_CHANCE };
        } else if self.used && !fresh {
            o.used = true;
        } else {
            o.now = school_chance(next);
            o.ahead = 1.0 - (next..EXHAUSTED_AT).map(|k| 1.0 - school_chance(k)).product::<f32>();
        }
        o
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Msg {
    Hit,
    Gone,
    Quiet,
    LetGo,
    Exhausted,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct HoleOdds {
    pub next: u32,
    pub now: f32,
    pub ahead: f32,
    pub fresh: bool,
    pub pond: bool,
    pub used: bool,
    pub wait: u64,
}

/// While a school runs (the hit cast + 6..12 more), `done` school casts so far: the casts left (least, most) and the
/// chance the next cast is still in it.
pub fn school_left(done: u32) -> (u32, u32, f32) {
    let extra = done.saturating_sub(1);
    let (lo, hi) = (SCHOOL_MIN.saturating_sub(extra), SCHOOL_MAX.saturating_sub(extra));
    let possible = (SCHOOL_MIN..=SCHOOL_MAX).filter(|&n| n >= extra).count();
    let ok = (SCHOOL_MIN..=SCHOOL_MAX).filter(|&n| n > extra).count();
    (lo, hi, if possible > 0 { ok as f32 / possible as f32 } else { 0.0 })
}

/// "3-9 casts left" / "up to 9 casts left" / "1 cast left" / "last cast".
pub fn casts_left(lo: u32, hi: u32) -> String {
    if hi == 0 {
        "last cast".into()
    } else if lo == hi {
        format!("{hi} cast{} left", if hi == 1 { "" } else { "s" })
    } else if lo == 0 {
        format!("up to {hi} casts left")
    } else {
        format!("{lo}-{hi} casts left")
    }
}

// ---------------------------------------------------------------------------------------------------------------
// The world's Angler state
// ---------------------------------------------------------------------------------------------------------------

/// Saved per character (`timeless-angler-<name>.json` beside config.json).
#[derive(Clone, serde::Serialize, Deserialize)]
#[serde(default)]
pub struct Prefs {
    /// The HUD shows.
    pub hud: bool,
    /// ... only with a rod in hand (or while fishing, or in a school).
    pub hud_auto: bool,
    /// The school of fish banner.
    pub banner: bool,
}

impl Default for Prefs {
    fn default() -> Self {
        Prefs { hud: true, hud_auto: true, banner: true }
    }
}

#[derive(Default)]
pub struct Session {
    /// Fishing skill gained.
    pub skill: f32,
    /// Unix seconds of the first cast.
    pub first: Option<u64>,
    pub casts: u32,
    pub catches: u32,
    pub fish: u32,
    pub lost: u32,
    pub schools: u32,
    /// Caught, by item id.
    pub by: HashMap<u16, u32>,
}

/// The guide window.
#[derive(Clone, Default)]
pub struct Guide {
    pub open: bool,
    pub tab: usize,
    pub search: String,
    pub zone_filter: String,
    pub zone: Option<u16>,
    pub fish: Option<u16>,
    /// A filter field has the keyboard (the game's keys stay off meanwhile).
    pub typing: bool,
}

#[derive(Default)]
pub struct Angler {
    pub guide: Guide,
    pub prefs: Prefs,
    prefs_for: String,
    pub hole: Hole,
    pub session: Session,
    /// A notice under the banner ("The school of fish moved on ...") and until when (unix seconds).
    pub notice: Option<(String, u64)>,
    /// The catch on the line (s2c 0x115's item id; 0 unknown), and a monster.
    pub hooked: Option<(u16, bool)>,
    /// The status we last saw (a cast = turning 56 from not fishing).
    last_status: u8,
    /// What the HUD shows, worked out four times a second.
    pub view: Option<View>,
    next_view: f32,
}

/// The HUD's figures.
#[derive(Clone, Default)]
pub struct View {
    pub zone: u16,
    pub zone_known: bool,
    pub area: Option<String>,
    pub rod: u16,
    pub bait: u16,
    pub waist: u16,
    pub skill_raw: i32,
    /// Guild rank (0 Amateur ..) and whether the skill is at the rank's cap.
    pub rank: u16,
    pub capped: bool,
    pub gear: i32,
    pub date: VDate,
    pub city: bool,
    pub spot: Option<Spot>,
    pub x: f32,
    pub z: f32,
    pub fishing: bool,
}

impl View {
    pub fn skill(&self) -> i32 {
        self.skill_raw + self.gear
    }
    pub fn has_rod(&self) -> bool {
        data().rods.contains_key(&self.rod)
    }
}

/// Unix seconds.
pub const RANKS: [&str; 12] = [
    "Amateur",
    "Recruit",
    "Initiate",
    "Novice",
    "Apprentice",
    "Journeyman",
    "Craftsman",
    "Artisan",
    "Adept",
    "Veteran",
    "Expert",
    "Authority",
];

pub fn now_secs() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

fn prefs_path(name: &str) -> std::path::PathBuf {
    let safe: String = name.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-').collect();
    crate::config::config_path().with_file_name(format!("timeless-angler-{safe}.json"))
}

impl Angler {
    pub fn save_prefs(&self) {
        if self.prefs_for.is_empty() {
            return;
        }
        if let Ok(s) = serde_json::to_string_pretty(&self.prefs)
            && let Err(e) = std::fs::write(prefs_path(&self.prefs_for), s)
        {
            log::warn!("angler: saving the settings: {e}");
        }
    }
}

fn equipped(w: &World, slot: usize) -> u16 {
    w.inv.equip.get(slot).copied().flatten().and_then(|(c, i)| w.inv.item(c, i)).filter(|&id| id != 0xffff).unwrap_or(0)
}

/// Fishing statuses (old and new): 50..62.
pub fn fishing_now(status: u8) -> bool {
    (50..=62).contains(&status)
}

/// Per frame: the cast counter, a school's end, the server's lines, the HUD's figures.
pub fn tick(w: &mut World, dt: f32) {
    if w.name.is_empty() || w.zone.is_none() {
        return;
    }
    if w.angler.prefs_for != w.name {
        w.angler.prefs_for = w.name.clone();
        w.angler.prefs = std::fs::read_to_string(prefs_path(&w.name))
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
    }
    let now = now_secs();
    let zone = w.zone.unwrap_or(0);
    let (x, z) = (w.pos.x, w.pos.z);
    // A cast: our status turns 56 (NewFishingStart) from a non-fishing one.
    let status = w.my_motion.status;
    if status == 56 && !fishing_now(w.angler.last_status) {
        let fresh = w.angler.hole.cast(zone, x, z, now);
        let s = &mut w.angler.session;
        s.casts += 1;
        s.first.get_or_insert(now);
        log::info!("angler: cast {} at this spot{}", w.angler.hole.casts, if fresh { " (a new spot)" } else { "" });
    }
    w.angler.last_status = status;
    // The server's lines and the catch lines.
    for line in std::mem::take(&mut w.chat.tap) {
        on_line(w, &line, now);
    }
    if let Some(why) = w.angler.hole.check(zone, x, z, now) {
        let (c, n) = w.angler.hole.last_school.unwrap_or_default();
        w.angler.notice =
            Some((format!("The school of fish moved on ({why}).\nYou caught {n} fish in {c} casts."), now + 8));
    }
    if w.angler.notice.as_ref().is_some_and(|n| now >= n.1) {
        w.angler.notice = None;
    }
    w.angler.next_view -= dt;
    if w.angler.next_view <= 0.0 {
        w.angler.next_view = 0.25;
        w.angler.view = Some(view(w));
    }
}

fn view(w: &World) -> View {
    let d = data();
    let zone = w.zone.unwrap_or(0);
    let craft = w.skill_base.and_then(|b| b.get(48).copied()).filter(|&v| v != 0xffff && v != 0x8000).unwrap_or(0);
    let gear: i32 =
        [4usize, 5, 6, 7, 8, 9, 10].iter().map(|&s| d.gear.get(&equipped(w, s)).copied().unwrap_or(0)).sum();
    let date = vdate(super::super::vana_now(w));
    let mut v = View {
        zone,
        zone_known: d.zones.contains_key(&zone),
        rod: equipped(w, 2),
        bait: equipped(w, 3),
        waist: equipped(w, 10),
        skill_raw: ((craft >> 5) & 0x3ff) as i32,
        rank: craft & 0x1f,
        capped: craft & 0x8000 != 0,
        gear,
        city: d.zones.get(&zone).is_some_and(|z| z.city),
        date,
        x: w.pos.x,
        z: w.pos.z,
        fishing: fishing_now(w.my_motion.status),
        ..Default::default()
    };
    if let Some(zd) = d.zones.get(&zone)
        && let Some((aid, a)) = find_area(zd, w.pos.x, w.pos.y, w.pos.z)
    {
        v.area = Some(a.n.clone());
        let ctx = SpotCtx {
            zone,
            area: aid,
            when: When { skill: v.skill(), hour: date.hour, moon: date.moon, month: date.month },
            bait: v.bait,
            rod: v.rod,
            weather: w.weather,
            body: equipped(w, 5),
        };
        v.spot = Some(spot(d, &ctx, &|k| w.has_key_item(k)));
    }
    v
}

fn on_line(w: &mut World, line: &str, now: u64) {
    let a = &mut w.angler;
    if line.contains("school of fish") || line.contains("fishing spot") || line.contains("fishing hole") {
        match a.hole.message(line, now) {
            Some(Msg::Hit) => {
                a.notice = None;
                a.session.schools += 1;
            }
            Some(Msg::Gone) => {
                if let Some((c, n)) = a.hole.last_school {
                    a.notice = Some((
                        format!(
                            "The school of fish moved on. You caught {n} fish in {c} casts.\nMove 20 or more yalms away to \
                             start a new fishing spot."
                        ),
                        now + 10,
                    ));
                }
            }
            Some(Msg::Exhausted) => {
                a.notice = Some((
                    "This fishing spot is exhausted (75 casts).\nMove 20 or more yalms away to start a new one.".into(),
                    now + 10,
                ));
            }
            _ => {}
        }
        return;
    }
    let mine = line.starts_with(&format!("{} caught ", w.name)) || line.starts_with("You caught ");
    if mine {
        let n = line
            .split(" caught ")
            .nth(1)
            .and_then(|r| r.split(' ').next())
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(1);
        a.session.catches += 1;
        let (id, monster) = a.hooked.take().unwrap_or((0, false));
        if !monster && id != 0 {
            *a.session.by.entry(id).or_default() += n;
            if data().fish.get(&id).is_some_and(|f| f.it == 0) {
                a.session.fish += n;
            }
        }
        if a.hole.active {
            a.hole.school_catches += n;
        }
    } else if let Some(up) =
        line.split("fishing skill rises ").nth(1).and_then(|r| r.split(' ').next()).and_then(|n| n.parse::<f32>().ok())
    {
        a.session.skill += up;
    } else if ["lost your catch", "line snaps", "rod breaks", "You give up", "didn't catch anything"]
        .iter()
        .any(|k| line.contains(k))
    {
        a.session.lost += 1;
        a.hooked = None;
    }
}

/// The catch on the line (the Timeless 0x115).
pub fn on_bite(w: &mut World, item: u16, monster: bool) {
    w.angler.hooked = Some((item, monster));
}

/// Lets the running school go: c2s 0x110 mode 5 para 11 (the server says "You let the school of fish swim off.").
pub fn let_school_go(w: &mut World) {
    if !w.angler.hole.active {
        return;
    }
    super::super::fishing::send_para(w, 11, 5, 0);
    let now = now_secs();
    w.angler.hole.end_school(now);
    let (c, n) = w.angler.hole.last_school.unwrap_or_default();
    w.angler.notice = Some((format!("You let the school of fish go.\nYou caught {n} fish in {c} casts."), now + 6));
}

pub fn reset_session(w: &mut World) {
    w.angler.session = Session::default();
}

/// `/angler [hud [on|off] | banner [on|off] | auto | school end | spot reset]`.
pub fn command(w: &mut World, args: &[&str]) {
    let a = &mut w.angler;
    let onoff = |v: Option<&&str>, cur: bool| match v.map(|s| s.to_ascii_lowercase()).as_deref() {
        Some("on") => true,
        Some("off") => false,
        _ => !cur,
    };
    match args.first().map(|s| s.to_ascii_lowercase()).as_deref() {
        None | Some("guide") => a.guide.open = !a.guide.open,
        Some("hud") | Some("show") | Some("hide") => {
            a.prefs.hud = match args.first().map(|s| s.to_ascii_lowercase()).as_deref() {
                Some("show") => true,
                Some("hide") => false,
                _ => onoff(args.get(1), a.prefs.hud),
            };
            a.save_prefs();
        }
        Some("auto") => {
            a.prefs.hud_auto = onoff(args.get(1), a.prefs.hud_auto);
            a.save_prefs();
        }
        Some("banner") => {
            a.prefs.banner = onoff(args.get(1), a.prefs.banner);
            a.save_prefs();
        }
        Some("school") if args.get(1).is_some_and(|s| s.eq_ignore_ascii_case("end")) => let_school_go(w),
        Some("spot") | Some("hole") if args.get(1).is_some_and(|s| s.eq_ignore_ascii_case("reset")) => {
            a.hole = Hole { next_school: a.hole.next_school, ..Default::default() };
        }
        Some("session") if args.get(1).is_some_and(|s| s.eq_ignore_ascii_case("reset")) => reset_session(w),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_loads() {
        let d = data();
        assert!(d.fish.len() > 100 && d.zones.len() > 80 && !d.bait.is_empty() && !d.rods.is_empty());
        assert!(d.zones.values().all(|z| z.areas.values().all(|a| d.groups.contains_key(&a.g))));
    }

    #[test]
    fn moon_buckets() {
        assert_eq!(moon_bucket(0, 0), MOON_NEW);
        assert_eq!(moon_bucket(100, 0), MOON_FULL);
        assert_eq!(moon_bucket(50, 2), 2);
        assert_eq!(moon_bucket(50, 1), 6);
        // A whole cycle of the client's clock gives every bucket.
        let seen: std::collections::HashSet<u8> = (0..84u64).map(|day| vdate(day * 3456 + 1000).moon).collect();
        assert_eq!(seen.len(), 8);
    }

    #[test]
    fn spot_odds_sum_to_one() {
        let d = data();
        let mut checked = 0;
        for (&zid, z) in &d.zones {
            for &aid in z.areas.keys() {
                for &bait in d.bait.keys() {
                    let ctx = SpotCtx {
                        zone: zid,
                        area: aid,
                        when: When { skill: 60, hour: 5, moon: 4, month: 3 },
                        bait,
                        rod: 17386,
                        weather: 0,
                        body: 0,
                    };
                    let sp = spot(d, &ctx, &|_| true);
                    if sp.nothing {
                        continue;
                    }
                    let o = cast_odds(&sp, false, false);
                    let sum: f32 = o.fish.values().sum::<f32>() + o.item + o.mob + o.none;
                    assert!((sum - 1.0).abs() < 1e-3, "zone {zid} area {aid} bait {bait}: {sum}");
                    checked += 1;
                }
            }
        }
        assert!(checked > 500);
    }

    #[test]
    fn difficulty_matches_the_server_notes() {
        let d = data();
        let id = d.fish.iter().find(|(_, f)| f.n == "Gugrusaurus").map(|(&i, _)| i).unwrap();
        assert_eq!(difficulty(d, id, 100, 0, 0), 80);
        assert_eq!(difficulty(d, id, 100, 4, 17011), 67);
    }

    #[test]
    fn spot_and_school_rules() {
        let mut h = Hole::default();
        assert!(h.cast(100, 0.0, 0.0, 1000));
        assert!(!h.cast(100, 5.0, 5.0, 1010));
        assert!(h.fresh(100, 30.0, 0.0, 1020));
        assert!(h.fresh(100, 0.0, 0.0, 1010 + SPOT_IDLE));
        assert_eq!(h.message("You've hit a school of fish! Keep fishing here", 1015), Some(Msg::Hit));
        assert!(h.active);
        h.cast(100, 1.0, 1.0, 1020);
        assert_eq!(h.school_casts, 2);
        assert_eq!(h.check(100, 40.0, 0.0, 1030), Some("you left the fishing spot"));
        assert_eq!(h.last_school, Some((2, 0)));
        assert!(h.odds(100, 0.0, 0.0, 1031).used);
        // Ponds never wear out and wait 5 minutes between schools.
        let mut p = Hole::default();
        p.cast(174, 0.0, 0.0, 0);
        p.message("hit a school of fish", 1);
        p.end_school(10);
        let o = p.odds(174, 50.0, 0.0, 20);
        assert!(o.pond && o.wait > 0 && o.now == 0.0);
        assert_eq!(school_left(1), (6, 12, 1.0));
        assert_eq!(casts_left(0, 3), "up to 3 casts left");
    }

    #[test]
    fn bite_waits() {
        assert_eq!(bite_wait(2, 12, 0, false), 13);
        assert_eq!(bite_wait(4, 5, FISHERS_ROPE, false), 7);
        assert_eq!(bite_wait(2, 12, 0, true), 7);
        assert_eq!(bite_wait(4, 5, FISHERS_ROPE, true), 4);
    }
}
