//! Timeless XI: the Create Character screen's windows in Timeless View (owner, 2026-11-23: concept A, "Hall of
//! Origins"). The character stands before Altana (client/timeless/studio.rs); round them:
//! - at the top, the six steps as a path of diamonds (gold behind, silver where you are; a step behind can be clicked);
//! - at the left, the race's card: its words, its stats with this job, and the choices so far;
//! - at the right, the step: Race, Likeness (style and size), Colors (the custom look: hair, eyes, markings,
//!   lips, nose), Job, Homeland, Name (then the world, the pass and the checks as retail's);
//! - at the bottom, the camera's keys (drag to turn, the wheel to zoom, F for the face, R to be surprised).
//!
//! Its words are Timeless View's: spaced serif capitals over fading rules, steel blue for focus, the hourglass's gold.

use super::super::creator::{Page, Pointer};
use super::super::game_ui::GameUi;
use super::super::timeless::looks::{self, BOTH, LEFT, MARKS, RIGHT, SIDED, Settings};
use super::craft::button;
use super::lobby::{field, pick_row, step_rows};

use super::tlselect::{appear, glint_rule, spaced};
use super::{Modern, Panels, Title};
use egui::{Align2, Color32, FontFamily, FontId, Id, Pos2, Rect, Sense, Stroke, Ui, pos2, vec2};

/// The steps as shown (owner: "Spell Colors right. Calling = Job"): their names and retail's step numbers.
pub const STEPS: [(&str, usize); 6] =
    [("Race", 0), ("Likeness", 1), ("Colors", 2), ("Job", 4), ("Homeland", 6), ("Name", 5)];
const ROMAN: [&str; 6] = ["I", "II", "III", "IV", "V", "VI"];

/// Where retail's step `step` is on the path (the confirm and the create: past the end).
pub fn place_of(step: usize) -> usize {
    STEPS.iter().position(|s| s.1 == step).unwrap_or(STEPS.len())
}

/// The step after `step` (the name step's next is the world, then the confirm: retail's).
pub fn next_step(step: usize) -> usize {
    STEPS.get(place_of(step) + 1).map_or(7, |s| s.1)
}

/// The step before `step` (the race step's: none).
pub fn prev_step(step: usize) -> Option<usize> {
    match place_of(step) {
        0 => None,
        k => STEPS.get(k - 1).map(|s| s.1),
    }
}

/// The races' words (Timeless's own), by retail's race row: Hume ♂ ♀, Elvaan ♂ ♀, Tarutaru ♂ ♀, Mithra, Galka.
const RACE_WORDS: [(&str, &str); 8] = [
    ("Adaptable folk found in every corner of Vana'diel.", "Even in every art: no strength, no weakness."),
    ("Adaptable folk found in every corner of Vana'diel.", "Even in every art: no strength, no weakness."),
    ("Proud knights of the forests of San d'Oria.", "Strong of arm and spirit; slow to magic."),
    ("Proud knights of the forests of San d'Oria.", "Strong of arm and spirit; slow to magic."),
    ("Small of stature, vast of magical talent.", "Born to the arcane arts of Windurst."),
    ("Small of stature, vast of magical talent.", "Born to the arcane arts of Windurst."),
    ("Hunters from the southern isles.", "Quick of hand and keen of eye."),
    ("Wandering giants of Bastok's mines and forges.", "Great of strength and endurance."),
];

/// The jobs' marks and words, by retail's job row.
const JOBS: [(&str, &str); 6] = [
    ("WAR", "Master of every weapon; strength and fury."),
    ("MNK", "Fists of iron, a body forged by discipline."),
    ("WHM", "Healer and protector, light of the party."),
    ("BLM", "Wielder of the elements' destructive magic."),
    ("RDM", "Blade and spell held in balance."),
    ("THF", "Swift hands, sharp eyes, and treasure."),
];

/// The stats with no game data (the offscreen shot).
const SAMPLE_STATS: [u8; 7] = [5, 8, 5, 8, 6, 5, 6];

/// A step's choices: the game's words, or English with no game data (the offscreen shot).
fn rows(gu: Option<&GameUi>, step: usize) -> Vec<String> {
    if let Some(g) = gu {
        return step_rows(g, step);
    }
    let l: &[&str] = match step {
        0 => &["Hume ♂", "Hume ♀", "Elvaan ♂", "Elvaan ♀", "Tarutaru ♂", "Tarutaru ♀", "Mithra", "Galka"],
        3 => &["Small", "Medium", "Large"],
        4 => &["Warrior", "Monk", "White Mage", "Black Mage", "Red Mage", "Thief"],
        6 => &["San d'Oria", "Bastok", "Windurst"],
        _ => &[],
    };
    l.iter().map(|s| s.to_string()).collect()
}

const HAIR: [[u8; 3]; 12] = [
    [24, 22, 26],
    [58, 40, 30],
    [86, 52, 30],
    [150, 92, 48],
    [222, 186, 120],
    [232, 226, 216],
    [150, 150, 158],
    [168, 44, 36],
    [226, 120, 60],
    [60, 84, 150],
    [70, 150, 120],
    [170, 110, 190],
];
const EYES: [[u8; 3]; 12] = [
    [111, 143, 168],
    [74, 138, 208],
    [40, 80, 150],
    [64, 176, 112],
    [90, 120, 70],
    [216, 160, 56],
    [176, 92, 40],
    [92, 58, 36],
    [190, 60, 60],
    [150, 100, 200],
    [200, 200, 210],
    [40, 36, 40],
];
const LIPS: [[u8; 3]; 8] = [
    [154, 106, 90],
    [192, 112, 104],
    [214, 140, 140],
    [168, 58, 58],
    [120, 70, 100],
    [96, 64, 52],
    [70, 50, 60],
    [150, 120, 160],
];
const MARK: [[u8; 3]; 8] = [
    [90, 52, 32],
    [30, 20, 18],
    [232, 232, 240],
    [150, 40, 36],
    [40, 60, 120],
    [200, 150, 60],
    [60, 110, 80],
    [120, 70, 150],
];

fn c32(c: [u8; 3]) -> Color32 {
    Color32::from_rgb(c[0], c[1], c[2])
}

/// 0..1 hue, saturation and value.
#[cfg_attr(not(test), allow(dead_code))]
fn to_hsv(c: [u8; 3]) -> [f32; 3] {
    let [r, g, b] = c.map(|x| x as f32 / 255.0);
    let (max, min) = (r.max(g).max(b), r.min(g).min(b));
    let d = max - min;
    let h = if d <= 1e-6 {
        0.0
    } else if max == r {
        ((g - b) / d).rem_euclid(6.0) / 6.0
    } else if max == g {
        ((b - r) / d + 2.0) / 6.0
    } else {
        ((r - g) / d + 4.0) / 6.0
    };
    [h, if max <= 1e-6 { 0.0 } else { d / max }, max]
}

fn from_hsv([h, s, v]: [f32; 3]) -> [u8; 3] {
    let h6 = (h.rem_euclid(1.0)) * 6.0;
    let c = v * s;
    let x = c * (1.0 - (h6 % 2.0 - 1.0).abs());
    let (r, g, b) = match h6 as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = v - c;
    [r, g, b].map(|k| ((k + m) * 255.0).round().clamp(0.0, 255.0) as u8)
}

/// A small spaced caption.
fn caption(ui: &mut Ui, m: &Modern, text: &str) {
    let w = ui.available_width();
    let (r, _) = ui.allocate_exact_size(vec2(w, 20.0), Sense::hover());
    spaced(
        ui,
        pos2(r.min.x + 2.0, r.center().y),
        Align2::LEFT_CENTER,
        text,
        m.theme.small * 0.92,
        m.theme.color("muted"),
        1.6,
    );
}

fn words(ui: &mut Ui, m: &Modern, text: &str, colour: Color32) {
    ui.label(egui::RichText::new(text).size(m.theme.small).color(colour));
}

/// A choice: its text (and a muted line under it), lit when chosen; clicking chooses (Next goes on).
fn choice(ui: &mut Ui, m: &Modern, text: &str, under: Option<&str>, on: bool, live: bool) -> egui::Response {
    let w = ui.available_width();
    let h = if under.is_some() { 46.0 } else { 32.0 };
    let (rect, resp) = ui.allocate_exact_size(vec2(w, h), if live { Sense::click() } else { Sense::hover() });
    let face = match () {
        _ if on => "controls/list_row_selected",
        _ if live && resp.hovered() => "controls/list_row_hover",
        _ => "controls/list_row_normal",
    };
    super::rows::face_with(ui.painter(), m, rect, face, Color32::WHITE, true);
    let p = ui.painter();
    let top = if under.is_some() { rect.min.y + 16.0 } else { rect.center().y };
    p.text(
        pos2(rect.min.x + 16.0, top),
        Align2::LEFT_CENTER,
        text,
        FontId::new(m.theme.ui * 1.05, FontFamily::Name(super::STRONG.into())),
        if on { m.theme.color("gold") } else { m.theme.color("text") },
    );
    if let Some(u) = under {
        p.text(
            pos2(rect.min.x + 16.0, rect.min.y + 34.0),
            Align2::LEFT_CENTER,
            u,
            FontId::proportional(m.theme.small),
            m.theme.color("muted").gamma_multiply(if on { 1.0 } else { 0.8 }),
        );
    }
    if on {
        diamond(p, pos2(rect.max.x - 16.0, rect.center().y), 4.0, m.theme.color("focus"));
    }
    ui.add_space(2.0);
    resp
}

fn diamond(p: &egui::Painter, c: Pos2, r: f32, fill: Color32) {
    p.add(egui::Shape::convex_polygon(
        vec![pos2(c.x, c.y - r), pos2(c.x + r, c.y), pos2(c.x, c.y + r), pos2(c.x - r, c.y)],
        fill,
        Stroke::NONE,
    ));
}

/// The path of steps across the top: the title, a hairline from the first step to the last (gold up to here), a
/// diamond and roman numeral and name at each. A step behind can be clicked to go back to it.
fn header(ui: &mut Ui, m: &Modern, step: usize, live: bool, out: &mut Pointer) {
    let screen = ui.ctx().content_rect();
    let cx = screen.center().x;
    let top = screen.min.y + screen.height() * 0.03;
    spaced(ui, pos2(cx, top), Align2::CENTER_TOP, "Create Character", m.theme.title * 1.25, m.theme.color("text"), 4.0);
    let here = place_of(step).min(STEPS.len() - 1);
    let done = place_of(step) >= STEPS.len();
    let span = (screen.width() * 0.5).min(900.0);
    let (x0, x1) = (cx - span / 2.0, cx + span / 2.0);
    let y = top + m.theme.title * 1.25 + 46.0;
    let p = ui.painter();
    p.line_segment([pos2(x0, y), pos2(x1, y)], Stroke::new(1.0, m.theme.color("muted").gamma_multiply(0.35)));
    let reach = if done { 1.0 } else { here as f32 / (STEPS.len() - 1) as f32 };
    p.line_segment(
        [pos2(x0, y), pos2(x0 + (x1 - x0) * reach, y)],
        Stroke::new(2.0, m.theme.color("gold").gamma_multiply(0.85)),
    );
    for (k, (name, retail)) in STEPS.iter().enumerate() {
        let x = x0 + (x1 - x0) * k as f32 / (STEPS.len() - 1) as f32;
        let at = pos2(x, y);
        let behind = k < here || done;
        let current = k == here && !done;
        let hit = Rect::from_center_size(pos2(x, y + 4.0), vec2(110.0, 70.0));
        let resp = ui.interact(
            hit,
            Id::unique(("tl create path", k)),
            if behind && live { Sense::click() } else { Sense::hover() },
        );
        let p = ui.painter();
        if current {
            let t = ui.input(|i| i.time) as f32;
            let pulse = 0.75 + 0.25 * (t * 2.2).sin();
            p.circle_filled(at, 15.0, m.theme.color("focus").gamma_multiply(0.18 * pulse));
            p.circle_filled(at, 9.0, m.theme.color("focus").gamma_multiply(0.22 * pulse));
            diamond(p, at, 8.0, m.theme.color("text"));
        } else if behind {
            diamond(p, at, if resp.hovered() { 7.0 } else { 6.0 }, m.theme.color("gold"));
        } else {
            diamond(p, at, 5.0, m.theme.color("muted").gamma_multiply(0.6));
            diamond(p, at, 3.0, Color32::from_rgb(14, 17, 25));
        }
        let colour = if current {
            m.theme.color("text")
        } else if behind {
            m.theme.color("gold").gamma_multiply(if resp.hovered() { 1.0 } else { 0.85 })
        } else {
            m.theme.color("muted").gamma_multiply(0.8)
        };
        spaced(ui, pos2(x, y - 14.0), Align2::CENTER_BOTTOM, ROMAN[k], m.theme.small * 0.9, colour, 1.0);
        let size = if current { m.theme.ui } else { m.theme.small };
        spaced(ui, pos2(x, y + 14.0), Align2::CENTER_TOP, name, size, colour, 1.8);
        if behind && live && resp.clicked() {
            out.goto = Some(*retail);
        }
    }
}

/// The race's card: its name, its words, its stats with the job chosen (bars), and the choices made so far.
fn card(ui: &mut Ui, m: &Modern, gu: Option<&GameUi>, c: &Page) {
    let v = c.view;
    let k = appear(ui, "tl create card", v.race as u64, 0.35);
    ui.multiply_opacity(k);
    let width = ui.available_width();
    ui.add_space(4.0);
    ui.label(
        egui::RichText::new(rows(gu, 0).get(v.race).cloned().unwrap_or_default())
            .size(m.theme.title * 1.4)
            .family(FontFamily::Name(super::STRONG.into()))
            .color(m.theme.color("text")),
    );
    let (a, b) = RACE_WORDS[v.race.min(7)];
    ui.add_space(2.0);
    words(ui, m, a, m.theme.color("text").gamma_multiply(0.85));
    words(ui, m, b, m.theme.color("muted"));
    ui.add_space(8.0);
    let y = ui.cursor().top();
    glint_rule(ui, m, ui.max_rect().min.x - 4.0, ui.max_rect().max.x + 4.0, y, 0.2);
    ui.add_space(12.0);
    let stats = gu.map_or_else(|| SAMPLE_STATS.to_vec(), |g| g.creator_stat_values(v.race, v.job));
    let top = stats.iter().copied().max().unwrap_or(1).max(1) as f32;
    for (n, s) in ["STR", "DEX", "VIT", "AGI", "INT", "MND", "CHR"].iter().zip(&stats) {
        let (r, _) = ui.allocate_exact_size(vec2(width, 24.0), Sense::hover());
        spaced(
            ui,
            pos2(r.min.x + 2.0, r.center().y),
            Align2::LEFT_CENTER,
            n,
            m.theme.small * 0.92,
            m.theme.color("muted"),
            1.4,
        );
        let (bx0, bx1) = (r.min.x + 64.0, r.max.x - 40.0);
        let bar = Rect::from_min_max(pos2(bx0, r.center().y - 3.0), pos2(bx1, r.center().y + 3.0));
        let p = ui.painter();
        p.rect_filled(bar, 2.0, Color32::from_rgb(20, 24, 34));
        let f = (*s as f32 / (top * 1.08)).clamp(0.05, 1.0);
        let fill = Rect::from_min_max(bar.min, pos2(bx0 + (bx1 - bx0) * f, bar.max.y));
        p.add(super::rows::gradient(fill, m.theme.color("focus").gamma_multiply(0.55), m.theme.color("focus"), true));
        p.rect_stroke(bar, 2.0, Stroke::new(1.0, m.theme.color("gold").gamma_multiply(0.2)), egui::StrokeKind::Outside);
        p.text(
            pos2(r.max.x - 2.0, r.center().y),
            Align2::RIGHT_CENTER,
            s.to_string(),
            FontId::proportional(m.theme.ui),
            m.theme.color("text"),
        );
    }
    // The choices so far.
    let here = place_of(v.step);
    if here >= 1 {
        ui.add_space(8.0);
        let y = ui.cursor().top();
        super::borders::fade_rule(
            ui.painter(),
            ui.max_rect().min.x,
            ui.max_rect().max.x,
            y,
            m.theme.color("gold").gamma_multiply(0.3),
        );
        ui.add_space(10.0);
        let sizes = rows(gu, 3);
        let mut facts =
            vec![("Likeness", format!("Style {}  ·  {}", v.face + 1, sizes.get(v.size).cloned().unwrap_or_default()))];
        if here >= 4 {
            facts.push(("Job", rows(gu, 4).get(v.job).cloned().unwrap_or_default()));
        }
        if here >= 5 {
            facts.push(("Homeland", rows(gu, 6).get(v.nation).cloned().unwrap_or_default()));
        }
        for (label, value) in facts {
            let (r, _) = ui.allocate_exact_size(vec2(width, 22.0), Sense::hover());
            spaced(
                ui,
                pos2(r.min.x + 2.0, r.center().y),
                Align2::LEFT_CENTER,
                label,
                m.theme.small * 0.9,
                m.theme.color("muted"),
                1.4,
            );
            ui.painter().text(
                pos2(r.min.x + 122.0, r.center().y),
                Align2::LEFT_CENTER,
                value,
                FontId::proportional(m.theme.ui),
                m.theme.color("text"),
            );
        }
    }
    ui.add_space(4.0);
}

/// Colour picks: the palette as chips, then the colour's hue, depth and light as three bars to drag.
fn colour(ui: &mut Ui, m: &Modern, id: &str, c: &mut [u8; 3], picks: &[[u8; 3]]) {
    let w = ui.available_width();
    let per = 6usize;
    let size = (w - (per - 1) as f32 * 8.0) / per as f32;
    for row in picks.chunks(per) {
        let (r, _) = ui.allocate_exact_size(vec2(w, size * 0.62), Sense::hover());
        for (k, pick) in row.iter().enumerate() {
            let chip = Rect::from_min_size(pos2(r.min.x + k as f32 * (size + 8.0), r.min.y), vec2(size, size * 0.62));
            let resp = ui.interact(chip, Id::unique(("tl chip", id, k, pick)), Sense::click());
            let p = ui.painter();
            p.rect_filled(chip, 3.0, c32(*pick));
            let on = *pick == *c;
            let edge = if on {
                Stroke::new(2.0, m.theme.color("text"))
            } else if resp.hovered() {
                Stroke::new(1.0, m.theme.color("focus"))
            } else {
                Stroke::new(1.0, m.theme.color("gold").gamma_multiply(0.25))
            };
            p.rect_stroke(chip, 3.0, edge, egui::StrokeKind::Outside);
            if resp.clicked() {
                *c = *pick;
            }
        }
        ui.add_space(6.0);
    }
    ui.add_space(2.0);
    // Red, green and blue (owner, 2026-11-23): each bar runs from none to full of its channel, the others as
    // they are; the number at the right.
    let before = *c;
    for (k, name) in ["R", "G", "B"].iter().enumerate() {
        let (r, _) = ui.allocate_exact_size(vec2(w, 22.0), Sense::hover());
        spaced(
            ui,
            pos2(r.min.x + 2.0, r.center().y),
            Align2::LEFT_CENTER,
            name,
            m.theme.small * 0.9,
            m.theme.color("muted"),
            1.2,
        );
        ui.painter().text(
            pos2(r.max.x - 2.0, r.center().y),
            Align2::RIGHT_CENTER,
            before[k].to_string(),
            FontId::proportional(m.theme.small),
            m.theme.color("text"),
        );
        let bar =
            Rect::from_min_max(pos2(r.min.x + 26.0, r.center().y - 5.0), pos2(r.max.x - 44.0, r.center().y + 5.0));
        let resp = ui.interact(bar.expand2(vec2(6.0, 6.0)), Id::unique(("tl strip", id, k)), Sense::click_and_drag());
        let p = ui.painter();
        let ends = |t: u8| {
            let mut e = before;
            e[k] = t;
            c32(e)
        };
        let mut mesh = egui::Mesh::default();
        for (s, col) in [ends(0), ends(255)].into_iter().enumerate() {
            let x = bar.min.x + bar.width() * s as f32;
            mesh.colored_vertex(pos2(x, bar.min.y), col);
            mesh.colored_vertex(pos2(x, bar.max.y), col);
        }
        mesh.add_triangle(0, 1, 2);
        mesh.add_triangle(1, 2, 3);
        p.add(egui::Shape::mesh(mesh));
        p.rect_stroke(
            bar,
            2.0,
            Stroke::new(1.0, m.theme.color("gold").gamma_multiply(0.25)),
            egui::StrokeKind::Outside,
        );
        let x = bar.min.x + bar.width() * before[k] as f32 / 255.0;
        p.circle_filled(pos2(x, bar.center().y), 6.5, Color32::from_rgb(12, 14, 20));
        p.circle_stroke(pos2(x, bar.center().y), 6.5, Stroke::new(1.5, m.theme.color("text")));
        if let Some(pos) = resp.interact_pointer_pos().filter(|_| resp.dragged() || resp.clicked()) {
            c[k] = (((pos.x - bar.min.x) / bar.width()).clamp(0.0, 1.0) * 255.0).round() as u8;
        }
    }
}

/// A slider in Timeless's way: a hairline with a knob, a word at each end.
fn slider(ui: &mut Ui, m: &Modern, id: &str, v: &mut f32, range: (f32, f32), ends: (&str, &str)) {
    let w = ui.available_width();
    let (r, _) = ui.allocate_exact_size(vec2(w, 26.0), Sense::hover());
    let p = ui.painter();
    let font = FontId::proportional(m.theme.small * 0.95);
    p.text(pos2(r.min.x + 2.0, r.center().y), Align2::LEFT_CENTER, ends.0, font.clone(), m.theme.color("muted"));
    p.text(pos2(r.max.x - 2.0, r.center().y), Align2::RIGHT_CENTER, ends.1, font, m.theme.color("muted"));
    let (x0, x1) = (r.min.x + 70.0, r.max.x - 70.0);
    let resp = ui.interact(
        Rect::from_min_max(pos2(x0 - 8.0, r.min.y), pos2(x1 + 8.0, r.max.y)),
        Id::unique(("tl slider", id)),
        Sense::click_and_drag(),
    );
    let p = ui.painter();
    p.line_segment(
        [pos2(x0, r.center().y), pos2(x1, r.center().y)],
        Stroke::new(2.0, m.theme.color("muted").gamma_multiply(0.4)),
    );
    let t = ((*v - range.0) / (range.1 - range.0)).clamp(0.0, 1.0);
    let x = x0 + (x1 - x0) * t;
    p.line_segment([pos2(x0, r.center().y), pos2(x, r.center().y)], Stroke::new(2.0, m.theme.color("focus")));
    diamond(p, pos2(x, r.center().y), 7.0, m.theme.color("text"));
    if let Some(pos) = resp.interact_pointer_pos().filter(|_| resp.dragged() || resp.clicked()) {
        let t = ((pos.x - x0) / (x1 - x0)).clamp(0.0, 1.0);
        *v = range.0 + (range.1 - range.0) * t;
    }
}

/// A row of small toggles; the one lit is `v`'s.
fn toggles<T: PartialEq + Copy>(ui: &mut Ui, m: &Modern, id: &str, v: &mut T, opts: &[(T, &str)], width: f32) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        for (k, (val, label)) in opts.iter().enumerate() {
            let (r, resp) = ui.allocate_exact_size(vec2(width, 24.0), Sense::click());
            let on = *v == *val;
            let p = ui.painter();
            p.rect_filled(
                r,
                3.0,
                if on { m.theme.color("focus").gamma_multiply(0.28) } else { Color32::from_rgb(18, 21, 30) },
            );
            p.rect_stroke(
                r,
                3.0,
                Stroke::new(
                    1.0,
                    if on {
                        m.theme.color("focus")
                    } else if resp.hovered() {
                        m.theme.color("muted")
                    } else {
                        m.theme.color("gold").gamma_multiply(0.22)
                    },
                ),
                egui::StrokeKind::Inside,
            );
            p.text(
                r.center(),
                Align2::CENTER_CENTER,
                *label,
                FontId::proportional(m.theme.small * 0.95),
                if on { m.theme.color("text") } else { m.theme.color("muted") },
            );
            if resp.clicked() {
                *v = *val;
            }
            let _ = (k, id);
        }
    });
}

/// The Colors step: tabs for each part the likeness has, its colours, and a reset to the style's own.
fn colors(
    ui: &mut Ui,
    m: &Modern,
    s: &mut Settings,
    has: looks::Has,
    original: impl Fn() -> Option<Settings>,
    out: &mut Pointer,
) {
    let mut tabs = vec![("Hair", 0usize)];
    if has.iris {
        tabs.push(("Eyes", 1));
    }
    if has.marks {
        tabs.push(("Markings", 2));
    }
    if has.lips {
        tabs.push(("Lips", 3));
    }
    // Nose colour removed (owner 2026-11-23: noses stay the original look on every race).
    let id = Id::unique("tl create colours tab");
    let mut tab = ui.ctx().data(|d| d.get_temp::<usize>(id)).unwrap_or(0);
    if !tabs.iter().any(|t| t.1 == tab) {
        tab = 0;
    }
    let w = ui.available_width();
    let (r, _) = ui.allocate_exact_size(vec2(w, 30.0), Sense::hover());
    // Each tab as wide as its word (spaced capitals), the room left shared out.
    let font = FontId::new(m.theme.small, super::menu_family());
    let widths: Vec<f32> = tabs
        .iter()
        .map(|(n, _)| {
            ui.painter().layout_no_wrap(n.to_uppercase(), font.clone(), Color32::WHITE).size().x + n.len() as f32 * 1.4
        })
        .collect();
    let spare = ((w - widths.iter().sum::<f32>()) / tabs.len() as f32).max(0.0);
    let mut x = r.min.x;
    for (k, (name, t)) in tabs.iter().enumerate() {
        let cell = Rect::from_min_size(pos2(x, r.min.y), vec2(widths[k] + spare, 30.0));
        x += widths[k] + spare;
        let resp = ui.interact(cell, Id::unique(("tl create tab", k)), Sense::click());
        let on = tab == *t;
        let colour = if on {
            m.theme.color("text")
        } else if resp.hovered() {
            m.theme.color("focus")
        } else {
            m.theme.color("muted")
        };
        spaced(ui, cell.center(), Align2::CENTER_CENTER, name, m.theme.small, colour, 1.4);
        if on {
            ui.painter().line_segment(
                [pos2(cell.min.x + 10.0, cell.max.y - 2.0), pos2(cell.max.x - 10.0, cell.max.y - 2.0)],
                Stroke::new(2.0, m.theme.color("focus")),
            );
        }
        if resp.clicked() {
            tab = *t;
        }
    }
    ui.ctx().data_mut(|d| d.insert_temp(id, tab));
    super::borders::fade_rule(
        ui.painter(),
        r.min.x,
        r.max.x,
        r.max.y + 1.0,
        m.theme.color("gold").gamma_multiply(0.25),
    );
    ui.add_space(10.0);
    match tab {
        0 => {
            colour(ui, m, "hair", &mut s.hair, &HAIR);
            ui.add_space(6.0);
            toggles(ui, m, "two", &mut s.two_tone, &[(false, "One Tone"), (true, "Two-Tone")], 110.0);
            if s.two_tone {
                ui.add_space(6.0);
                caption(ui, m, "Toward the Ends");
                colour(ui, m, "tip", &mut s.tip, &HAIR);
                slider(ui, m, "tip start", &mut s.tip_start, (0.15, 0.95), ("Crown", "Ends"));
                slider(ui, m, "tip blend", &mut s.tip_blend, (0.02, 0.45), ("Sharp", "Soft"));
            }
        }
        1 => colour(ui, m, "eyes", &mut s.eyes, &EYES),
        2 => {
            s.marks.resize(MARKS.len(), 0);
            // Each race's own markings (2026-11-23); only Mithra's have sides.
            let mithra = has.race == 7;
            if mithra {
                words(ui, m, "Left and right are your character's own.", m.theme.color("muted"));
            }
            ui.add_space(4.0);
            for (k, name) in looks::mark_names(has.race).iter().enumerate() {
                ui.horizontal(|ui| {
                    let (r, _) = ui.allocate_exact_size(vec2(150.0, 24.0), Sense::hover());
                    let on = s.marks[k] != 0;
                    ui.painter().text(
                        pos2(r.min.x + 2.0, r.center().y),
                        Align2::LEFT_CENTER,
                        *name,
                        FontId::proportional(m.theme.small),
                        if on { m.theme.color("text") } else { m.theme.color("muted") },
                    );
                    let v = &mut s.marks[k];
                    if mithra && SIDED[k] {
                        toggles(ui, m, name, v, &[(0, "Off"), (LEFT, "L"), (BOTH, "Both"), (RIGHT, "R")], 46.0);
                    } else {
                        if *v != 0 {
                            *v = BOTH;
                        }
                        toggles(ui, m, name, v, &[(0, "Off"), (BOTH, "On")], 46.0);
                    }
                });
            }
            if s.marks.iter().any(|m| *m != 0) {
                ui.add_space(6.0);
                caption(ui, m, "Marking Color");
                colour(ui, m, "marks", &mut s.mark_colour, &MARK);
                slider(ui, m, "mark strength", &mut s.mark_strength, (0.2, 1.0), ("Faint", "Bold"));
            }
        }
        3 => colour(ui, m, "lips", &mut s.lips, &LIPS),
        _ => slider(ui, m, "nose", &mut s.nose, (0.0, 1.0), ("Grey", "Black")),
    }
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        // This tab's part back to the style's own (the other tabs keep theirs).
        if button(ui, m, "Original", 150.0, false, false)
            && let Some(o) = original()
        {
            match tab {
                0 => {
                    (s.hair, s.two_tone, s.tip) = (o.hair, o.two_tone, o.tip);
                    (s.tip_start, s.tip_blend) = (o.tip_start, o.tip_blend);
                }
                1 => s.eyes = o.eyes,
                2 => (s.marks, s.mark_colour, s.mark_strength) = (o.marks.clone(), o.mark_colour, o.mark_strength),
                3 => s.lips = o.lips,
                _ => s.nose = o.nose,
            }
        }
        if button(ui, m, "Surprise Me", 150.0, false, false) {
            out.surprise = true;
        }
    });
}

/// Random colours for a look (R, Surprise Me): hair from the palettes half the time and any colour the rest, now
/// and then two-tone; eyes and lips of any hue; markings (Mithra keep their style's and get new colours, others
/// sometimes gain one or two); the nose's shade.
pub fn surprise(s: &mut Settings, has: looks::Has, seed: u64, mithra: bool) {
    let x = std::cell::Cell::new(seed | 1);
    let next = || {
        let mut v = x.get();
        v ^= v << 13;
        v ^= v >> 7;
        v ^= v << 17;
        x.set(v);
        v
    };
    let unit = || (next() % 10_000) as f32 / 10_000.0;
    let any = |s_lo: f32, s_hi: f32, v_lo: f32, v_hi: f32| {
        let (h, sat, v) = (unit(), unit(), unit());
        from_hsv([h, s_lo + (s_hi - s_lo) * sat, v_lo + (v_hi - v_lo) * v])
    };
    let pick = |n: usize| (next() % n as u64) as usize;
    s.hair = if pick(2) == 0 { HAIR[pick(HAIR.len())] } else { any(0.15, 0.85, 0.12, 0.95) };
    s.two_tone = pick(4) == 0;
    if s.two_tone {
        s.tip = any(0.2, 0.9, 0.3, 1.0);
        s.tip_start = 0.35 + (pick(40) as f32) / 100.0;
        s.tip_blend = 0.05 + (pick(30) as f32) / 100.0;
    }
    if has.iris {
        s.eyes = any(0.35, 0.9, 0.3, 0.9);
    }
    if has.lips {
        s.lips = if pick(3) == 0 { any(0.3, 0.8, 0.3, 0.8) } else { LIPS[pick(LIPS.len())] };
    }
    s.marks.resize(MARKS.len(), 0);
    if !mithra {
        s.marks.iter_mut().for_each(|m| *m = 0);
        // The race's own markings (2026-11-23), on both sides.
        let n = looks::mark_names(has.race).len();
        if n > 0 {
            for _ in 0..pick(3) {
                s.marks[pick(n)] = BOTH;
            }
        }
    }
    if s.marks.iter().any(|m| *m != 0) {
        s.mark_colour = if pick(2) == 0 { MARK[pick(MARK.len())] } else { any(0.3, 0.9, 0.15, 0.9) };
        s.mark_strength = 0.5 + pick(50) as f32 / 100.0;
    }
}

/// The step's window: its name, a line of help, its choices, Back and Next.
#[allow(clippy::too_many_arguments)]
fn step_window(
    ui: &mut Ui,
    m: &Modern,
    gu: Option<&GameUi>,
    c: &Page,
    look: Option<&mut Settings>,
    has: looks::Has,
    live: bool,
    out: &mut Pointer,
) {
    let v = c.view;
    let here = place_of(v.step);
    let k = appear(ui, "tl create step", v.step as u64, 0.3);
    ui.multiply_opacity(k);
    ui.add_space((1.0 - k) * 8.0);
    let help = match v.step {
        0 => "Five peoples walk Vana'diel. Which blood will carry your legend?",
        1 => "Choose a likeness and a size. Its colors are yours next.",
        2 => "Make the colors your own: hair, eyes, markings.",
        4 => "Your first calling. Every job can be learned in time.",
        6 => "The nation you will serve, and where you will begin.",
        _ if v.world.is_some() => "",
        _ => "Name your character.",
    };
    if c.status || !help.is_empty() {
        let line = if c.status { c.help.as_str() } else { help };
        words(ui, m, line, if c.status { m.theme.color("gold") } else { m.theme.color("muted") });
        ui.add_space(8.0);
    }
    match v.step {
        0 => {
            let list = rows(gu, 0);
            for (k, t) in list.iter().enumerate() {
                if choice(ui, m, t, None, k == v.race, live).clicked() && live {
                    out.row.0 = Some(k);
                }
            }
        }
        1 => {
            let w = ui.available_width();
            let gap = 8.0;
            let tw = (w - gap * 3.0) / 4.0;
            for row in 0..2 {
                let (r, _) = ui.allocate_exact_size(vec2(w, 74.0), Sense::hover());
                for col in 0..4 {
                    let i = row * 4 + col;
                    let tile = Rect::from_min_size(pos2(r.min.x + col as f32 * (tw + gap), r.min.y), vec2(tw, 68.0));
                    let resp = ui.interact(
                        tile,
                        Id::unique(("tl create style", i)),
                        if live { Sense::click() } else { Sense::hover() },
                    );
                    let on = i == v.face;
                    let p = ui.painter();
                    p.add(super::rows::gradient(
                        tile,
                        if on { Color32::from_rgb(0x2a, 0x36, 0x4e) } else { Color32::from_rgb(0x1d, 0x22, 0x30) },
                        Color32::from_rgb(0x0e, 0x11, 0x18),
                        false,
                    ));
                    let edge = if on {
                        Stroke::new(1.5, m.theme.color("focus"))
                    } else if resp.hovered() && live {
                        Stroke::new(1.0, m.theme.color("muted"))
                    } else {
                        Stroke::new(1.0, m.theme.color("gold").gamma_multiply(0.22))
                    };
                    p.rect_stroke(tile, 3.0, edge, egui::StrokeKind::Inside);
                    p.text(
                        pos2(tile.center().x, tile.min.y + 28.0),
                        Align2::CENTER_CENTER,
                        ROMAN_STYLE[i],
                        FontId::new(m.theme.title * 1.3, FontFamily::Name(super::SERIF.into())),
                        if on { m.theme.color("gold") } else { m.theme.color("text").gamma_multiply(0.8) },
                    );
                    spaced(
                        ui,
                        pos2(tile.center().x, tile.max.y - 12.0),
                        Align2::CENTER_CENTER,
                        "Style",
                        m.theme.small * 0.78,
                        m.theme.color("muted"),
                        1.2,
                    );
                    if resp.clicked() && live {
                        out.row.0 = Some(i);
                    }
                }
            }
            ui.add_space(8.0);
            caption(ui, m, "Size");
            let sizes = rows(gu, 3);
            let mut sz = v.size;
            let opts: Vec<(usize, &str)> = sizes.iter().enumerate().map(|(k, s)| (k, s.as_str())).collect();
            toggles(ui, m, "size", &mut sz, &opts, (w - 8.0) / 3.0 - 1.0);
            if sz != v.size && live {
                out.size = Some(sz as u8);
            }
        }
        2 => match look {
            Some(s) => {
                let (race, face) = (v.race as u8 + 1, (v.face * 2) as u16);
                colors(ui, m, s, has, || looks::start_for(race, face), out)
            }
            None => words(ui, m, "This likeness wears its own colors.", m.theme.color("muted")),
        },
        4 => {
            let names = rows(gu, 4);
            let w = ui.available_width();
            for (k, (code, line)) in JOBS.iter().enumerate() {
                let on = k == v.job;
                let (rect, resp) =
                    ui.allocate_exact_size(vec2(w, 50.0), if live { Sense::click() } else { Sense::hover() });
                let face = match () {
                    _ if on => "controls/list_row_selected",
                    _ if live && resp.hovered() => "controls/list_row_hover",
                    _ => "controls/list_row_normal",
                };
                super::rows::face_with(ui.painter(), m, rect, face, Color32::WHITE, true);
                let pw =
                    super::hud::tl_job_plaque(ui.painter(), m, rect.min.x + 12.0, rect.center().y, code, None, 0, on);
                let x = rect.min.x + 24.0 + pw;
                let p = ui.painter();
                p.text(
                    pos2(x, rect.min.y + 17.0),
                    Align2::LEFT_CENTER,
                    names.get(k).map_or(*code, |s| s.as_str()),
                    FontId::new(m.theme.ui * 1.05, FontFamily::Name(super::STRONG.into())),
                    if on { m.theme.color("gold") } else { m.theme.color("text") },
                );
                p.text(
                    pos2(x, rect.min.y + 35.0),
                    Align2::LEFT_CENTER,
                    *line,
                    FontId::proportional(m.theme.small),
                    m.theme.color("muted"),
                );
                if resp.clicked() && live {
                    out.row.0 = Some(k);
                }
                ui.add_space(2.0);
            }
        }
        6 => {
            let list = rows(gu, 6);
            for (k, t) in list.iter().enumerate() {
                if choice(ui, m, t, None, k == v.nation, live).clicked() && live {
                    out.row.0 = Some(k);
                }
            }
            let (title, body) = gu
                .map_or_else(|| (rows(None, 6)[v.nation.min(2)].clone(), String::new()), |g| g.nation_words(v.nation));
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(title)
                    .size(m.theme.ui)
                    .family(FontFamily::Name(super::STRONG.into()))
                    .color(m.theme.color("gold")),
            );
            ui.add_space(4.0);
            words(ui, m, &body, m.theme.color("text").gamma_multiply(0.9));
        }
        5 if !v.name_hidden => name_step(ui, m, gu, c, live, out),
        _ => {}
    }
    // Back and Next (the name step's own buttons are its Next).
    if v.step != 5 {
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            let back = prev_step(v.step).map_or("Leave".to_string(), |p| STEPS[place_of(p)].0.to_string());
            if button(ui, m, &format!("‹  {back}"), 150.0, false, !live) {
                out.back = true;
            }
            let w = ui.available_width();
            ui.add_space((w - 150.0).max(0.0));
            let next = STEPS.get(here + 1).map_or("Next", |s| s.0);
            if button(ui, m, &format!("{next}  ›"), 150.0, true, !live) {
                out.enter = true;
            }
        });
    }
}

const ROMAN_STYLE: [&str; 8] = ["1", "2", "3", "4", "5", "6", "7", "8"];

/// The name, Random and OK; over it, the world list and the pass (retail's).
fn name_step(ui: &mut Ui, m: &Modern, gu: Option<&GameUi>, c: &Page, live: bool, out: &mut Pointer) {
    let v = c.view;
    let typing = v.world.is_none();
    field(ui, m, v.name, "", typing.then_some(v.name.chars().count()));
    if typing {
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if button(ui, m, "‹  Homeland", 130.0, false, !live) {
                out.back = true;
            }
            if button(ui, m, "Random", 110.0, false, !live) {
                out.random = true;
            }
            if button(ui, m, "OK", 110.0, true, !live || v.name.is_empty()) {
                out.enter = true;
            }
        });
    }
    if let Some(w) = &v.world {
        ui.add_space(10.0);
        caption(ui, m, &gu.map_or("World".into(), |g| g.lobby_msg(1, 0, "World")));
        ui.add_space(2.0);
        match w.pass {
            None => {
                egui::ScrollArea::vertical()
                    .id_salt("tl create worlds")
                    .max_height(ui.ctx().content_rect().height() * 0.4)
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        for (k, t) in w.rows.iter().enumerate() {
                            let resp = pick_row(ui, m, t, k == w.cursor, live);
                            if live && resp.clicked() {
                                out.world.1 = Some(k);
                            }
                        }
                    });
            }
            Some((pass, caret, _)) => {
                words(ui, m, w.rows.get(w.cursor).map_or("", |s| s.as_str()), m.theme.color("text"));
                ui.add_space(4.0);
                field(ui, m, pass, &gu.map_or(String::new(), |g| g.lobby_msg(8, 0x3e, "")), Some(caret));
                ui.add_space(6.0);
                if button(ui, m, "OK", 140.0, true, !live) {
                    out.enter = true;
                }
            }
        }
    }
}

/// The camera's keys along the bottom.
fn hints(ui: &mut Ui, m: &Modern, step: usize) {
    let screen = ui.ctx().content_rect();
    let mut keys = vec![("Drag", "Turn"), ("Wheel", "Zoom"), ("F", "Close-Up")];
    if step != 5 {
        keys.push(("R", "Surprise Me"));
    }
    let font = FontId::proportional(m.theme.small);
    let key_font = FontId::new(m.theme.small * 0.92, FontFamily::Name(super::STRONG.into()));
    let p = ui.painter();
    let widths: Vec<(f32, f32)> = keys
        .iter()
        .map(|(k, l)| {
            let kw = p.layout_no_wrap(k.to_string(), key_font.clone(), Color32::WHITE).size().x + 16.0;
            let lw = p.layout_no_wrap(l.to_string(), font.clone(), Color32::WHITE).size().x;
            (kw, lw)
        })
        .collect();
    let total: f32 = widths.iter().map(|(a, b)| a + b + 8.0).sum::<f32>() + 28.0 * (keys.len() - 1) as f32;
    let y = screen.max.y - screen.height() * 0.045;
    let mut x = screen.center().x - total / 2.0;
    for ((k, l), (kw, lw)) in keys.iter().zip(widths) {
        let r = Rect::from_min_size(pos2(x, y - 12.0), vec2(kw, 24.0));
        p.rect_filled(r, 4.0, Color32::from_rgba_unmultiplied(16, 19, 28, 210));
        p.rect_stroke(r, 4.0, Stroke::new(1.0, m.theme.color("muted").gamma_multiply(0.6)), egui::StrokeKind::Inside);
        p.text(r.center(), Align2::CENTER_CENTER, *k, key_font.clone(), m.theme.color("text"));
        p.text(
            pos2(r.max.x + 8.0, y),
            Align2::LEFT_CENTER,
            *l,
            font.clone(),
            m.theme.color("text").gamma_multiply(0.8),
        );
        x += kw + 8.0 + lw + 28.0;
    }
}

/// The whole screen. `look`: the colours being chosen (none: the style keeps its own); `has`: the parts it has.
pub(super) fn screen(
    p: &mut Panels,
    gu: Option<&GameUi>,
    c: &Page,
    look: Option<&mut Settings>,
    has: looks::Has,
    out: &mut Pointer,
) {
    let m = p.m;
    let ctx = p.ctx;
    let v = c.view;
    let screen = ctx.content_rect();
    let live = c.free && c.ok.is_none() && c.confirm.is_none();
    egui::Area::new(Id::unique("tl create top"))
        .order(egui::Order::Background)
        .fixed_pos(screen.min)
        .interactable(true)
        .show(ctx, |ui| {
            ui.set_min_size(screen.size());
            header(ui, m, v.step, live, out);
            if v.step < 7 {
                hints(ui, m, v.step);
            }
        });
    if v.step < 7 {
        let gap = screen.width() * 0.022;
        let top = screen.min.y + screen.height() * 0.2;
        p.panel_at("tl create card", None, 360.0, pos2(screen.min.x + gap, top), |ui| card(ui, m, gu, c));
        let width = 430.0;
        let here = place_of(v.step).min(5);
        let note = format!("{} of VI", ROMAN[here]);
        p.panel_at(
            "tl create step",
            Some(Title { icon: None, text: STEPS[here].0, note: Some(&note), name: false }),
            width,
            pos2(screen.max.x - gap - width, top),
            |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("tl create step scroll")
                    .max_height(screen.height() * 0.66)
                    .auto_shrink([false, true])
                    .show(ui, |ui| step_window(ui, m, gu, c, look, has, live, out));
            },
        );
    }
    prompts(p, gu, c, out);
    // The camera: a drag (not on a window) turns them, the wheel zooms.
    if live {
        // The wheel zooms, off the windows (the drag that turns them is the app's held-pointer drag: client/mod.rs
        // look()).
        let free = |p: Pos2| ctx.layer_id_at(p).is_none_or(|l| l.order == egui::Order::Background);
        let (at, wheel) = ctx.input(|i| (i.pointer.hover_pos(), i.smooth_scroll_delta.y));
        if wheel != 0.0 && at.is_some_and(free) {
            out.zoom += wheel / 600.0;
        }
    }
}

/// The confirm and the ok window, centred, as the Select screen's log-in prompt.
fn prompts(p: &mut Panels, gu: Option<&GameUi>, c: &Page, out: &mut Pointer) {
    let m = p.m;
    let screen = p.ctx.content_rect();
    let width = 440.0;
    // At the right, where the step's window was: the character stays in full view.
    let at = pos2(screen.max.x - screen.width() * 0.022 - width, screen.min.y + screen.height() * 0.2);
    if let Some(text) = &c.ok {
        p.panel_at("tl create ok", None, width, at, |ui| {
            ui.add_space(10.0);
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new(text).size(m.theme.ui).color(m.theme.color("text")));
                ui.add_space(10.0);
                if button(ui, m, "OK", 130.0, true, false) {
                    out.ok = true;
                }
            });
            ui.add_space(4.0);
        });
    } else if let Some((text, yes)) = &c.confirm {
        let caption = |k: u16, d: &str| {
            gu.and_then(|g| g.child_caption("menu    ptc6yesn", k))
                .filter(|c| !c.is_empty())
                .unwrap_or_else(|| d.to_string())
        };
        let (y, n) = (caption(1, "Yes"), caption(2, "No"));
        p.panel_at("tl create confirm", None, width, at, |ui| {
            ui.add_space(10.0);
            if let Some(a) = super::tlselect::prompt(ui, m, text, [&y, &n], *yes, m.theme.title * 1.15) {
                out.yes.1 = Some(a);
            }
            ui.add_space(4.0);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps_in_order() {
        let mut s = 0;
        let mut seen = vec![s];
        while next_step(s) != 7 {
            s = next_step(s);
            seen.push(s);
        }
        assert_eq!(seen, vec![0, 1, 2, 4, 6, 5]);
        assert_eq!(prev_step(5), Some(6));
        assert_eq!(prev_step(6), Some(4));
        assert_eq!(prev_step(0), None);
    }

    #[test]
    fn hsv_round_trip() {
        for c in [[24, 22, 26], [168, 44, 36], [60, 84, 150], [232, 226, 216], [0, 0, 0], [255, 255, 255]] {
            let back = from_hsv(to_hsv(c));
            for k in 0..3 {
                assert!((back[k] as i32 - c[k] as i32).abs() <= 1, "{c:?} -> {back:?}");
            }
        }
    }
}
