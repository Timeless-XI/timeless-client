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
