//! Timeless XI: the Select Character screen's windows in Timeless View (owner, 2026-11-22: "close to what I want but
//! they need a little cleaning up and a bit more life"): the highlighted character's card, the list's rows and the
//! log-in prompt, each laid out as Timeless View's (Pewter Hairline: spaced serif capitals over fading rules, steel
//! blue for focus, the hourglass's gold sand for an accent), with a little motion: a card or prompt eases in when it
//! changes or opens, and a glint of light runs along their rules now and then.

use super::super::select::RowView;
use super::{Modern, kit, paint};
use egui::{Align2, Color32, FontFamily, FontId, Id, Pos2, Rect, Sense, Ui, pos2, vec2};

/// `text` in spaced capitals (Timeless View's titles), placed by `align` at `at`; the rect it takes.
pub fn spaced(ui: &Ui, at: Pos2, align: Align2, text: &str, size: f32, colour: Color32, spacing: f32) -> Rect {
    let mut job = egui::text::LayoutJob::default();
    let font = FontId::new(size, super::menu_family());
    let format = egui::TextFormat { font_id: font, color: colour, extra_letter_spacing: spacing, ..Default::default() };
    job.append(&text.to_uppercase(), 0.0, format);
    let galley = ui.painter().layout_job(job);
    let rect = align.anchor_size(at, galley.size());
    ui.painter().galley(rect.min, galley, colour);
    rect
}

/// A rule from `x0` to `x1` at `y`, clear at both ends, with a glint of light running along it every few seconds.
pub fn glint_rule(ui: &Ui, m: &Modern, x0: f32, x1: f32, y: f32, seed: f32) {
    let p = ui.painter();
    super::borders::fade_rule(p, x0, x1, y, m.theme.color("gold").gamma_multiply(0.4));
    let t = ui.input(|i| i.time) as f32;
    // A run of 2.2 s, then a rest: 7 s a cycle, each rule on its own phase.
    let k = ((t + seed * 3.1) / 7.0).fract() * 7.0 / 2.2;
    if k < 1.0 {
        let w = (x1 - x0).max(1.0);
        let cx = x0 + w * (k * 1.3 - 0.15);
        let len = w * 0.22;
        let a = (k * std::f32::consts::PI).sin();
        let c = m.theme.color("focus");
        let steps = 16;
        for s in 0..steps {
            let f0 = s as f32 / steps as f32;
            let f1 = (s + 1) as f32 / steps as f32;
            let (xa, xb) = (cx - len * 0.5 + len * f0, cx - len * 0.5 + len * f1);
            if xb < x0 || xa > x1 {
                continue;
            }
            let bell = (1.0 - ((f0 + f1) - 1.0).abs()).powi(2);
            p.line_segment(
                [pos2(xa.max(x0), y), pos2(xb.min(x1), y)],
                egui::Stroke::new(1.2, c.gamma_multiply(0.9 * a * bell)),
            );
        }
    }
    ui.ctx().request_repaint();
}

/// 0..1: how far in a window has eased since `key` changed (or it opened after being away a moment).
pub fn appear(ui: &Ui, id: &str, key: u64, secs: f32) -> f32 {
    let id = Id::unique(("tl appear", id));
    let now = ui.input(|i| i.time);
    let (k0, t0, seen) = ui.ctx().data(|d| d.get_temp::<(u64, f64, f64)>(id)).unwrap_or((key ^ 1, now, now));
    let t0 = if k0 != key || now - seen > 1.0 { now } else { t0 };
    ui.ctx().data_mut(|d| d.insert_temp(id, (key, t0, now)));
    let k = (((now - t0) as f32) / secs).clamp(0.0, 1.0);
    if k < 1.0 {
        ui.ctx().request_repaint();
    }
    1.0 - (1.0 - k).powi(3)
}

fn hash(s: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

/// The highlighted character's card: their job's mark and name, the job and level in spaced capitals under a glinting
/// rule, then race, area and world as small captioned lines. It eases in when the cursor moves.
pub fn card(ui: &mut Ui, m: &Modern, r: &RowView, labels: &[String; 5], name_colour: Color32) {
    let k = appear(ui, "select card", hash(&r.name), 0.35);
    ui.multiply_opacity(k);
    ui.add_space((1.0 - k) * 8.0);
    let [world, race, job, level, area] = &r.info;
    let width = ui.available_width();
    let (head, _) = ui.allocate_exact_size(vec2(width, 52.0), Sense::hover());
    let p = ui.painter();
    let mut x = head.min.x + 2.0;
    if let Some(s) = super::hud::job_icon(r.job).and_then(|j| m.art.sprite(&j)) {
        p.add(paint::image(&s, pos2(x, head.center().y - 19.0), 38.0, Color32::WHITE));
        x += 48.0;
    }
    p.text(
        pos2(x, head.min.y + 17.0),
        Align2::LEFT_CENTER,
        &r.name,
        FontId::new(m.theme.title * 1.45, FontFamily::Name(super::STRONG.into())),
        name_colour,
    );
    let jl = if level.is_empty() { job.clone() } else { format!("{job}  ·  Lv {level}") };
    spaced(
        ui,
        pos2(x + 1.0, head.min.y + 42.0),
        Align2::LEFT_CENTER,
        &jl,
        m.theme.ui * 0.95,
        m.theme.color("focus"),
        1.6,
    );
    ui.add_space(6.0);
    let y = ui.cursor().top();
    glint_rule(ui, m, head.min.x - 4.0, head.max.x + 4.0, y, 0.0);
    ui.add_space(10.0);
    // The facts: a small spaced caption, its value beside it.
    for (label, value) in [(&labels[1], race), (&labels[4], area), (&labels[0], world)] {
        if value.is_empty() {
            continue;
        }
        let (row, _) = ui.allocate_exact_size(vec2(width, 22.0), Sense::hover());
        spaced(
            ui,
            pos2(row.min.x + 2.0, row.center().y),
            Align2::LEFT_CENTER,
            label,
            m.theme.small * 0.92,
            m.theme.color("muted"),
            1.4,
        );
        ui.painter().text(
            pos2(row.min.x + 92.0, row.center().y),
            Align2::LEFT_CENTER,
            value,
            FontId::proportional(m.theme.ui * 1.02),
            m.theme.color("text"),
        );
    }
    ui.add_space(2.0);
}

/// A row of the list: the job's mark, the name, the job and level at the right; race and area under it, muted. The
/// cursor's row has the steel-blue band (no hourglass: the ring's aura marks them).
pub fn row(
    ui: &mut Ui,
    m: &Modern,
    r: &RowView,
    on: bool,
    focused: bool,
    width: f32,
    height: f32,
    name_colour: Color32,
) -> egui::Response {
    let (rect, resp) =
        ui.allocate_exact_size(vec2(width, height), if focused { Sense::click() } else { Sense::hover() });
    let face = match () {
        _ if on && focused => "controls/list_row_selected",
        _ if on || focused && resp.hovered() => "controls/list_row_hover",
        _ => "controls/list_row_normal",
    };
    super::rows::face_with(ui.painter(), m, rect, face, Color32::WHITE, true);
    let [_, race, job, level, area] = &r.info;
    let p = ui.painter();
    let dim = if on { 1.0 } else { 0.72 };
    let mut x = rect.min.x + 12.0;
    if let Some(s) = super::hud::job_icon(r.job).and_then(|j| m.art.sprite(&j)) {
        p.add(paint::image(&s, pos2(x, rect.center().y - 14.0), 28.0, Color32::WHITE.gamma_multiply(dim)));
        x += 38.0;
    }
    let top = rect.min.y + rect.height() * 0.36;
    let low = rect.min.y + rect.height() * 0.72;
    p.text(
        pos2(x, top),
        Align2::LEFT_CENTER,
        &r.name,
        FontId::new(m.theme.title * 1.12, FontFamily::Name(super::STRONG.into())),
        name_colour.gamma_multiply(if on { 1.0 } else { 0.85 }),
    );
    let jl = if level.is_empty() { job.clone() } else { format!("{job} {level}") };
    p.text(
        pos2(rect.max.x - 12.0, top),
        Align2::RIGHT_CENTER,
        jl,
        FontId::proportional(m.theme.ui),
        if on { m.theme.color("focus") } else { m.theme.color("muted") },
    );
    let under = if area.is_empty() { race.clone() } else { format!("{race}  ·  {area}") };
    p.text(
        pos2(x, low),
        Align2::LEFT_CENTER,
        under,
        FontId::proportional(m.theme.small),
        m.theme.color("muted").gamma_multiply(dim),
    );
    resp
}

/// The log-in prompt: the question centred over a glinting rule, the two buttons centred under it; it eases in.
pub fn prompt(ui: &mut Ui, m: &Modern, question: &str, labels: [&str; 2], yes: bool, size: f32) -> Option<bool> {
    let k = appear(ui, "select prompt", hash(question), 0.3);
    ui.multiply_opacity(k);
    ui.add_space(4.0 + (1.0 - k) * 6.0);
    ui.vertical_centered(|ui| {
        ui.label(egui::RichText::new(question).size(size).family(super::menu_family()).color(m.theme.color("gold")));
    });
    ui.add_space(8.0);
    let r = ui.max_rect();
    let y = ui.cursor().top();
    glint_rule(ui, m, r.min.x + r.width() * 0.08, r.max.x - r.width() * 0.08, y, 0.5);
    ui.add_space(12.0);
    let out = kit::yes_no_centred(ui, m, "", labels, yes, size * 0.85);
    ui.add_space(2.0);
    out
}

// ---------------------------------------------------------------------------------------------------------------
// Round 4 mock-ups (owner, 2026-11-22): the card with the character's face, the job plaque and their nation; the card
// with a level medallion; the rows with the job plaque and something new at the right.
// ---------------------------------------------------------------------------------------------------------------

/// Which card and row the Select screen draws. The owner chose the level medallion and "last played" (2026-11-22);
/// the others stay for the mock-ups (modern-shot's `VANA_SELECT_MOCK`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CardStyle {
    Plain,
    Portrait,
    #[default]
    Medallion,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum RowRight {
    /// The job and level (round 3).
    Job,
    /// The nation and its rank.
    Nation,
    /// When the character last played on this computer.
    #[default]
    Played,
}

/// When each character last logged in on this computer: `timeless-last-played.json` beside config.json, "world:name"
/// to unix seconds.
fn played_path() -> std::path::PathBuf {
    crate::config::config_path().with_file_name("timeless-last-played.json")
}

fn played_map() -> std::collections::HashMap<String, u64> {
    std::fs::read_to_string(played_path()).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
}

fn unix_now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

/// Notes a log-in (the Select screen's Yes).
pub fn record_played(world: &str, name: &str) {
    let mut map = played_map();
    map.insert(format!("{world}:{name}"), unix_now());
    if let Ok(s) = serde_json::to_string_pretty(&map) {
        let _ = std::fs::write(played_path(), s);
    }
}

/// "Today", "Yesterday", "3 days ago", "2 weeks ago", "4 months ago", "A year ago", by whole days of `utc_offset`.
pub fn played_label(at: u64, now: u64, utc_offset: i64) -> String {
    let day = |t: u64| (t as i64 + utc_offset).div_euclid(86_400);
    let days = (day(now) - day(at)).max(0);
    match days {
        0 => "Today".into(),
        1 => "Yesterday".into(),
        2..=13 => format!("{days} days ago"),
        14..=59 => format!("{} weeks ago", days / 7),
        60..=364 => format!("{} months ago", days / 30),
        365..=729 => "A year ago".into(),
        _ => format!("{} years ago", days / 365),
    }
}

/// The row's "last played" text: the mock-up's sample, else this computer's record (re-read every few seconds).
fn played_text(style: &Style, world: &str, name: &str) -> Option<String> {
    if let Some(s) = style.played.get(name) {
        return Some(s.clone());
    }
    thread_local! {
        static CACHE: std::cell::RefCell<(u64, std::collections::HashMap<String, u64>)> = Default::default();
    }
    let t = unix_now();
    let offset = super::super::cfgapply::utc_offset();
    CACHE.with(|c| {
        let mut c = c.borrow_mut();
        if t >= c.0 + 3 {
            *c = (t, played_map());
        }
        c.1.get(&format!("{world}:{name}")).map(|at| played_label(*at, t, offset))
    })
}

#[derive(Clone, Default)]
pub struct Style {
    pub card: CardStyle,
    pub right: RowRight,
    /// Each character's face (by name), drawn by the ring's renderer.
    pub portraits: std::collections::HashMap<String, egui::TextureId>,
    /// Each character's last time played (by name), as shown ("Today", "3 days ago").
    pub played: std::collections::HashMap<String, String>,
}

thread_local! {
    pub static STYLE: std::cell::RefCell<Style> = std::cell::RefCell::new(Style::default());
}

pub fn nation_name(n: u8) -> &'static str {
    match n {
        0 => "San d'Oria",
        1 => "Bastok",
        _ => "Windurst",
    }
}

fn job_code(j: u8) -> Option<&'static str> {
    (1..=22).contains(&j).then(|| super::hud::job(j))
}

/// A texture in a circle (a fan with its UVs), `uv` the square of the texture to show.
fn round_image(p: &egui::Painter, tex: egui::TextureId, c: Pos2, r: f32, uv: Rect, tint: Color32) {
    let n = 48;
    let mut mesh = egui::Mesh::with_texture(tex);
    mesh.vertices.push(egui::epaint::Vertex { pos: c, uv: uv.center(), color: tint });
    for k in 0..=n {
        let a = k as f32 / n as f32 * std::f32::consts::TAU;
        let (s, co) = a.sin_cos();
        mesh.vertices.push(egui::epaint::Vertex {
            pos: c + vec2(co, s) * r,
            uv: uv.center() + vec2(co * uv.width(), s * uv.height()) * 0.5,
            color: tint,
        });
    }
    for k in 0..n as u32 {
        mesh.indices.extend_from_slice(&[0, k + 1, k + 2]);
    }
    p.add(mesh);
}

/// The frame about a round mark: a pewter ring, a hairline inside it and a small diamond at the top (the windows'
/// ornament), with a soft steel-blue light behind.
fn round_frame(p: &egui::Painter, m: &Modern, c: Pos2, r: f32, lit: f32) {
    let t = p.ctx().input(|i| i.time) as f32;
    let breath = 0.75 + 0.25 * (t * std::f32::consts::TAU / 5.0).sin();
    for k in 0..6 {
        let rr = r + 2.0 + k as f32 * 2.2;
        p.circle_stroke(
            c,
            rr,
            egui::Stroke::new(2.2, m.theme.color("focus").gamma_multiply(0.05 * lit * breath * (6 - k) as f32 / 6.0)),
        );
    }
    p.circle_stroke(c, r, egui::Stroke::new(1.6, super::borders::lighter(m.theme.color("bronze"), 0.3)));
    p.circle_stroke(c, r - 3.5, egui::Stroke::new(1.0, m.theme.color("gold").gamma_multiply(0.22)));
    let d = 4.5;
    let top = pos2(c.x, c.y - r);
    p.add(egui::Shape::convex_polygon(
        vec![top + vec2(0.0, -d), top + vec2(d, 0.0), top + vec2(0.0, d), top + vec2(-d, 0.0)],
        m.theme.color("gold"),
        egui::Stroke::NONE,
    ));
}

/// The card with a round mark at its left (the character's face, or their level), the name with the job plaque beside
/// it, their nation (and rank) under it, then the facts under a glinting rule.
pub fn card_marked(ui: &mut Ui, m: &Modern, r: &RowView, labels: &[String; 5], name_colour: Color32, style: &Style) {
    let k = appear(ui, "select card", hash(&r.name), 0.35);
    ui.multiply_opacity(k);
    ui.add_space((1.0 - k) * 8.0);
    let [world, race, job, level, area] = &r.info;
    let width = ui.available_width();
    let (head, _) = ui.allocate_exact_size(vec2(width, 96.0), Sense::hover());
    let p = ui.painter().clone();
    let rad = 42.0;
    let c = pos2(head.min.x + rad + 4.0, head.center().y + 2.0);
    match style.card {
        CardStyle::Portrait => {
            p.circle_filled(c, rad, Color32::from_rgb(0x10, 0x13, 0x1b));
            if let Some(tex) = style.portraits.get(&r.name) {
                // The face fills the circle: the square about the head of a 1:1 head shot.
                let uv = Rect::from_center_size(pos2(0.5, 0.43), vec2(0.46, 0.46));
                round_image(&p, *tex, c, rad - 1.0, uv, Color32::WHITE);
            }
            round_frame(&p, m, c, rad, 1.0);
        }
        _ => {
            // The level medallion: the level large in the serif, LEVEL small over it, a thin arc in the job's colour.
            p.circle_filled(c, rad, Color32::from_rgb(0x10, 0x13, 0x1b).gamma_multiply(0.9));
            let jc = job_code(r.job).map(|j| m.theme.color(&format!("job_{}", j.to_ascii_lowercase())));
            if let Some(jc) = jc {
                let n = 40;
                let pts: Vec<Pos2> = (0..=n)
                    .map(|i| {
                        let a = std::f32::consts::PI * (0.75 + 1.5 * i as f32 / n as f32);
                        c + vec2(a.cos(), a.sin()) * (rad - 6.0)
                    })
                    .collect();
                p.add(egui::Shape::line(pts, egui::Stroke::new(2.0, jc.gamma_multiply(0.85))));
            }
            spaced(
                ui,
                pos2(c.x, c.y - 17.0),
                Align2::CENTER_CENTER,
                "Level",
                m.theme.small * 0.72,
                m.theme.color("muted"),
                1.6,
            );
            p.text(
                pos2(c.x, c.y + 6.0),
                Align2::CENTER_CENTER,
                level,
                FontId::new(m.theme.title * 2.1, FontFamily::Name(super::SERIF.into())),
                m.theme.color("gold"),
            );
            round_frame(&p, m, c, rad, 0.6);
        }
    }
    let x = c.x + rad + 18.0;
    let name_font = FontId::new(m.theme.title * 1.45, FontFamily::Name(super::STRONG.into()));
    let name_y = head.min.y + 30.0;
    let g = p.layout_no_wrap(r.name.clone(), name_font, name_colour);
    let name_w = g.size().x;
    p.galley(pos2(x, name_y - g.size().y / 2.0), g, name_colour);
    match style.card {
        CardStyle::Portrait => {
            // The job plaque beside the name (the party list's: main over sub, the level at its corner).
            if let Some(main) = job_code(r.job) {
                let lv = level.parse().unwrap_or(0);
                super::hud::tl_job_plaque(&p, m, x + name_w + 14.0, name_y, main, job_code(r.sub_job), lv, true);
            }
        }
        _ => {
            // The job plaque beside the name (main over sub), its level left to the medallion.
            let _ = job;
            if let Some(main) = job_code(r.job) {
                super::hud::tl_job_plaque(&p, m, x + name_w + 14.0, name_y, main, job_code(r.sub_job), 0, true);
            }
        }
    }
    let nation = match r.rank {
        0 => nation_name(r.nation).to_string(),
        k => format!("{}  ·  Rank {k}", nation_name(r.nation)),
    };
    spaced(
        ui,
        pos2(x + 1.0, name_y + 30.0),
        Align2::LEFT_CENTER,
        &nation,
        m.theme.ui * 0.9,
        m.theme.color("focus"),
        1.6,
    );
    ui.add_space(4.0);
    let y = ui.cursor().top();
    glint_rule(ui, m, head.min.x - 4.0, head.max.x + 4.0, y, 0.0);
    ui.add_space(10.0);
    for (label, value) in [(&labels[1], race), (&labels[4], area), (&labels[0], world)] {
        if value.is_empty() {
            continue;
        }
        let (row, _) = ui.allocate_exact_size(vec2(width, 22.0), Sense::hover());
        spaced(
            ui,
            pos2(row.min.x + 2.0, row.center().y),
            Align2::LEFT_CENTER,
            label,
            m.theme.small * 0.92,
            m.theme.color("muted"),
            1.4,
        );
        ui.painter().text(
            pos2(row.min.x + 92.0, row.center().y),
            Align2::LEFT_CENTER,
            value,
            FontId::proportional(m.theme.ui * 1.02),
            m.theme.color("text"),
        );
    }
    ui.add_space(2.0);
}

/// A row with the job plaque at its left (job, sub job and level), the name, race · area under it, and at the right
/// the nation and rank, or when they last played.
#[allow(clippy::too_many_arguments)]
pub fn row_plaque(
    ui: &mut Ui,
    m: &Modern,
    r: &RowView,
    on: bool,
    focused: bool,
    width: f32,
    height: f32,
    name_colour: Color32,
    style: &Style,
) -> egui::Response {
    let (rect, resp) =
        ui.allocate_exact_size(vec2(width, height), if focused { Sense::click() } else { Sense::hover() });
    let face = match () {
        _ if on && focused => "controls/list_row_selected",
        _ if on || focused && resp.hovered() => "controls/list_row_hover",
        _ => "controls/list_row_normal",
    };
    super::rows::face_with(ui.painter(), m, rect, face, Color32::WHITE, true);
    let [_, race, _, level, area] = &r.info;
    let p = ui.painter().clone();
    let mut x = rect.min.x + 10.0;
    if let Some(main) = job_code(r.job) {
        x += super::hud::tl_job_plaque(
            &p,
            m,
            x,
            rect.center().y + 1.0,
            main,
            job_code(r.sub_job),
            level.parse().unwrap_or(0),
            on,
        );
    }
    let top = rect.min.y + rect.height() * 0.36;
    let low = rect.min.y + rect.height() * 0.72;
    p.text(
        pos2(x, top),
        Align2::LEFT_CENTER,
        &r.name,
        FontId::new(m.theme.title * 1.12, FontFamily::Name(super::STRONG.into())),
        name_colour.gamma_multiply(if on { 1.0 } else { 0.85 }),
    );
    let dim = if on { 1.0 } else { 0.72 };
    // The race under the name (the area is on the card; the right side has its own news).
    let _ = area;
    p.text(
        pos2(x, low),
        Align2::LEFT_CENTER,
        race,
        FontId::proportional(m.theme.small),
        m.theme.color("muted").gamma_multiply(dim),
    );
    let right = rect.max.x - 12.0;
    let lit = if on { m.theme.color("focus") } else { m.theme.color("muted") };
    match style.right {
        RowRight::Nation => {
            spaced(ui, pos2(right, top), Align2::RIGHT_CENTER, nation_name(r.nation), m.theme.small * 0.95, lit, 1.4);
            if r.rank > 0 {
                spaced(
                    ui,
                    pos2(right, low),
                    Align2::RIGHT_CENTER,
                    &format!("Rank {}", r.rank),
                    m.theme.small * 0.9,
                    m.theme.color("muted").gamma_multiply(dim),
                    1.4,
                );
            }
        }
        RowRight::Played => {
            if let Some(when) = played_text(style, &r.info[0], &r.name) {
                spaced(
                    ui,
                    pos2(right, top),
                    Align2::RIGHT_CENTER,
                    "Last played",
                    m.theme.small * 0.8,
                    m.theme.color("muted").gamma_multiply(dim),
                    1.4,
                );
                p.text(pos2(right, low), Align2::RIGHT_CENTER, when, FontId::proportional(m.theme.small * 1.05), lit);
            }
        }
        RowRight::Job => {}
    }
    resp
}

#[cfg(test)]
mod tests {
    use super::played_label;

    #[test]
    fn last_played_by_calendar_day() {
        let day: i64 = 86_400;
        // 23:00 and 01:00 the next day (UTC-7): yesterday, though only two hours apart.
        let off = -7 * 3600;
        let late = (10 * day + 23 * 3600 - off) as u64;
        assert_eq!(played_label(late, late + 2 * 3600, off), "Yesterday");
        assert_eq!(played_label(late, late + 60, off), "Today");
        assert_eq!(played_label(late, late + 3 * day as u64, off), "3 days ago");
        assert_eq!(played_label(late, late + 20 * day as u64, off), "2 weeks ago");
        assert_eq!(played_label(late, late + 100 * day as u64, off), "3 months ago");
        assert_eq!(played_label(late, late + 400 * day as u64, off), "A year ago");
    }
}
