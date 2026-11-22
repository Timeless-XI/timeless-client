//! Row looks: the pack's framed rows, and lighter styles offered beside them (modern-shot --rows); Gilt is the one in
//! use (user, 2026-10-27: the framed rows "too busy", but not plain either).

use super::{Modern, paint};
use egui::{Align2, Color32, FontId, Mesh, Painter, Rect, Sense, Shape, Stroke, Ui, pos2, vec2};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Style {
    /// The pack's row art: framed rows, a diamond bullet, the boxed arrow.
    Framed,
    /// No frames: hairlines between rows, the cursor a blue band fading right with a gold edge.
    Hairline,
    /// Rounded rows with a faint border, the cursor filled blue with a gold border.
    Soft,
    /// Bare rows, the cursor between two fading gold rules with a gold diamond.
    #[default]
    Gilt,
    /// Raised rows (a light top, a dark bottom), the cursor blue with a soft glow.
    Bevel,
    /// Timeless XI: the Timeless interface's (Timeless View, Pewter Hairline, 2026-11-19): the cursor's row a steel-blue
    /// band fading right between two fading hairlines, the logo's hourglass (gold sand) for its mark; a faint light
    /// under the pointer. Always the Timeless interface's.
    Dusk,
}

impl Style {
    pub const ALL: [Style; 6] = [Style::Framed, Style::Hairline, Style::Soft, Style::Gilt, Style::Bevel, Style::Dusk];

    pub fn name(self) -> &'static str {
        match self {
            Style::Framed => "framed",
            Style::Hairline => "hairline",
            Style::Soft => "soft",
            Style::Gilt => "gilt",
            Style::Bevel => "bevel",
            Style::Dusk => "dusk",
        }
    }

    pub fn from_name(s: &str) -> Option<Style> {
        Style::ALL.into_iter().find(|k| k.name() == s)
    }
}

/// A row's state: the cursor's (or its open category), hovered, disabled.
#[derive(Clone, Copy, Default)]
pub struct State {
    pub on: bool,
    pub hover: bool,
    pub off: bool,
    /// Timeless XI: no hourglass on the cursor's row (the Select screen's list, owner 2026-11-22).
    pub bare: bool,
}

/// A left-to-right (`across`) or top-to-bottom gradient over `r`.
pub(super) fn gradient(r: Rect, a: Color32, b: Color32, across: bool) -> Shape {
    let mut m = Mesh::default();
    let (tr, bl) = if across { (b, a) } else { (a, b) };
    m.colored_vertex(r.left_top(), a);
    m.colored_vertex(r.right_top(), tr);
    m.colored_vertex(r.right_bottom(), b);
    m.colored_vertex(r.left_bottom(), bl);
    m.add_triangle(0, 1, 2);
    m.add_triangle(0, 2, 3);
    Shape::mesh(m)
}

/// A line across `r`'s width at `y`, clear at both ends and `c` in the middle.
fn rule(p: &Painter, r: Rect, y: f32, c: Color32) {
    let mid = r.center().x;
    p.add(gradient(Rect::from_min_max(pos2(r.min.x, y), pos2(mid, y + 1.0)), Color32::TRANSPARENT, c, true));
    p.add(gradient(Rect::from_min_max(pos2(mid, y), pos2(r.max.x, y + 1.0)), c, Color32::TRANSPARENT, true));
}

/// The row's background.
/// The row style drawn: the Timeless interface's own (Dusk), else the chosen one.
fn current(m: &Modern) -> Style {
    if m.theme.timeless() { Style::Dusk } else { m.row_style.get() }
}

fn back(p: &Painter, m: &Modern, r: Rect, s: State, style: Style) {
    let (gold, focus, bronze) = (m.theme.color("gold"), m.theme.color("focus"), m.theme.color("bronze"));
    match style {
        Style::Dusk => {
            if s.on {
                let band = Rect::from_min_max(r.min, pos2(r.min.x + r.width() * 0.8, r.max.y));
                p.add(gradient(band, focus.gamma_multiply(0.30), focus.gamma_multiply(0.04), true));
                p.add(gradient(
                    Rect::from_min_max(pos2(band.max.x, r.min.y), r.max),
                    focus.gamma_multiply(0.04),
                    Color32::TRANSPARENT,
                    true,
                ));
                for y in [r.min.y, r.max.y - 1.0] {
                    p.add(gradient(
                        Rect::from_min_max(pos2(r.min.x, y), pos2(r.max.x, y + 1.0)),
                        gold.gamma_multiply(0.34),
                        Color32::TRANSPARENT,
                        true,
                    ));
                }
                // In the row's left margin, clear of the icons every list draws from x + 10.
                if !s.bare {
                    hourglass(p, pos2(r.min.x + 5.0, r.center().y), 13.0, gold, m.theme.color("tp"));
                }
            } else if s.hover {
                p.rect_filled(r.shrink2(vec2(0.0, 1.0)), 0.0, gold.gamma_multiply(0.06));
            }
        }
        Style::Framed => {
            let sprite = match () {
                _ if s.off => "controls/list_row_disabled",
                _ if s.on => "controls/list_row_selected",
                _ if s.hover => "controls/list_row_hover",
                _ => "controls/list_row_normal",
            };
            if let Some(sp) = m.art.sprite(sprite) {
                p.add(paint::nine_slice(&sp, r, r.height() / sp.size.y, Color32::WHITE));
            }
        }
        Style::Hairline => {
            if s.on {
                p.add(gradient(r, focus.gamma_multiply(0.55), focus.gamma_multiply(0.0), true));
                p.rect_filled(
                    Rect::from_min_max(pos2(r.min.x, r.min.y + 3.0), pos2(r.min.x + 3.0, r.max.y - 3.0)),
                    1.5,
                    gold,
                );
            } else if s.hover {
                p.rect_filled(r, 0.0, Color32::from_white_alpha(14));
            }
            p.line_segment(
                [pos2(r.min.x + 10.0, r.max.y + 1.0), pos2(r.max.x - 10.0, r.max.y + 1.0)],
                Stroke::new(1.0, bronze.gamma_multiply(0.35)),
            );
        }
        Style::Soft => {
            let r = r.shrink2(vec2(2.0, 1.0));
            let (fill, edge) = match () {
                _ if s.off => (Color32::from_black_alpha(40), bronze.gamma_multiply(0.12)),
                _ if s.on => (focus.gamma_multiply(0.5), gold.gamma_multiply(0.75)),
                _ if s.hover => (Color32::from_white_alpha(18), bronze.gamma_multiply(0.7)),
                _ => (Color32::from_black_alpha(64), bronze.gamma_multiply(0.25)),
            };
            p.rect(r, 6.0, fill, Stroke::new(1.0, edge), egui::StrokeKind::Inside);
        }
        Style::Gilt => {
            if s.on {
                let mid = r.center().x;
                p.add(gradient(
                    Rect::from_min_max(r.min, pos2(mid, r.max.y)),
                    gold.gamma_multiply(0.02),
                    gold.gamma_multiply(0.13),
                    true,
                ));
                p.add(gradient(
                    Rect::from_min_max(pos2(mid, r.min.y), r.max),
                    gold.gamma_multiply(0.13),
                    gold.gamma_multiply(0.02),
                    true,
                ));
                rule(p, r, r.min.y, gold.gamma_multiply(0.9));
                rule(p, r, r.max.y - 1.0, gold.gamma_multiply(0.9));
                // In the row's left margin, clear of the icons every list draws from x + 10.
                let c = pos2(r.min.x + 5.5, r.center().y);
                p.add(Shape::convex_polygon(
                    vec![pos2(c.x, c.y - 3.5), pos2(c.x + 3.5, c.y), pos2(c.x, c.y + 3.5), pos2(c.x - 3.5, c.y)],
                    gold,
                    Stroke::NONE,
                ));
            } else if s.hover {
                p.rect_filled(r, 0.0, gold.gamma_multiply(0.06));
            }
        }
        Style::Bevel => {
            let (top, bottom) = match () {
                _ if s.on => (focus.gamma_multiply(0.8), focus.gamma_multiply(0.35)),
                _ if s.hover => (Color32::from_rgb(0x34, 0x3e, 0x4a), Color32::from_rgb(0x1a, 0x20, 0x27)),
                _ if s.off => (Color32::from_rgb(0x1d, 0x22, 0x28), Color32::from_rgb(0x14, 0x17, 0x1b)),
                _ => (Color32::from_rgb(0x2a, 0x32, 0x3c), Color32::from_rgb(0x14, 0x18, 0x1e)),
            };
            let r = r.shrink2(vec2(1.0, 0.0));
            p.add(gradient(r, top, bottom, false));
            p.line_segment([r.left_top(), r.right_top()], Stroke::new(1.0, Color32::from_white_alpha(22)));
            p.line_segment([r.left_bottom(), r.right_bottom()], Stroke::new(1.0, Color32::from_black_alpha(110)));
            if s.on {
                p.rect_stroke(
                    r.expand(1.0),
                    3.0,
                    Stroke::new(1.0, focus.gamma_multiply(0.5)),
                    egui::StrokeKind::Outside,
                );
                p.rect_stroke(
                    r.expand(2.5),
                    4.0,
                    Stroke::new(2.0, focus.gamma_multiply(0.18)),
                    egui::StrokeKind::Outside,
                );
            }
        }
    }
}

/// Timeless XI: the logo's hourglass, `h` tall, centred at `c`: two curved glass walls, the stepped caps, a stream and
/// a pile of `sand` in the lower bulb (the Pewter rows' cursor).
pub fn hourglass(p: &Painter, c: egui::Pos2, h: f32, glass: Color32, sand: Color32) {
    let (hw, hh) = (h * 0.27, h * 0.42);
    // The logo's wall, top to neck to bottom, as a cubic bezier pair (x by side).
    let wall = |side: f32| -> Vec<egui::Pos2> {
        let seg = |p0: (f32, f32), p1: (f32, f32), p2: (f32, f32), p3: (f32, f32)| {
            (0..=8).map(move |i| {
                let t = i as f32 / 8.0;
                let u = 1.0 - t;
                let x = u * u * u * p0.0 + 3.0 * u * u * t * p1.0 + 3.0 * u * t * t * p2.0 + t * t * t * p3.0;
                let y = u * u * u * p0.1 + 3.0 * u * u * t * p1.1 + 3.0 * u * t * t * p2.1 + t * t * t * p3.1;
                pos2(c.x + side * x * hw, c.y + y * hh)
            })
        };
        seg((1.0, -1.0), (1.0, -0.4), (0.1, -0.18), (0.06, 0.0))
            .chain(seg((0.06, 0.0), (0.1, 0.18), (1.0, 0.4), (1.0, 1.0)).skip(1))
            .collect()
    };
    let st = Stroke::new(1.1, glass);
    p.add(Shape::line(wall(-1.0), st));
    p.add(Shape::line(wall(1.0), st));
    let cap = |y: f32| Rect::from_center_size(pos2(c.x, y), vec2(hw * 2.0 + 3.0, 1.8));
    p.rect_filled(cap(c.y - hh - 0.9), 0.5, glass);
    p.rect_filled(cap(c.y + hh + 0.9), 0.5, glass);
    p.line_segment([pos2(c.x, c.y + 0.5), pos2(c.x, c.y + hh * 0.6)], Stroke::new(0.7, sand));
    p.add(Shape::convex_polygon(
        vec![
            pos2(c.x - hw * 0.8, c.y + hh - 0.2),
            pos2(c.x - hw * 0.3, c.y + hh * 0.66),
            pos2(c.x + hw * 0.3, c.y + hh * 0.66),
            pos2(c.x + hw * 0.8, c.y + hh - 0.2),
        ],
        sand,
        Stroke::NONE,
    ));
}

/// The mark of a row that opens a submenu, at its right: an arrow toward the side the submenu opens on.
fn more(p: &Painter, m: &Modern, r: Rect, s: State, style: Style, left: bool) {
    let c = if s.on && style != Style::Bevel {
        m.theme.color("gold")
    } else if s.on {
        Color32::WHITE
    } else {
        m.theme.color("muted")
    };
    let at = pos2(r.max.x - 16.0, r.center().y);
    // The tip's and the tail's x.
    let (tip, tail) = if left { (at.x - 3.0, at.x + 2.0) } else { (at.x + 2.0, at.x - 3.0) };
    match style {
        Style::Framed => {
            if let Some(sp) = m.art.sprite(if left { "controls/arrow_left" } else { "controls/arrow_right" }) {
                p.add(paint::image(&sp, pos2(r.max.x - 26.0, r.center().y - 8.0), 16.0, Color32::WHITE));
            }
        }
        Style::Bevel => {
            let (tip, tail) = if left { (at.x - 3.0, at.x + 3.0) } else { (at.x + 3.0, at.x - 3.0) };
            p.add(Shape::convex_polygon(
                vec![pos2(tail, at.y - 5.0), pos2(tip, at.y), pos2(tail, at.y + 5.0)],
                c,
                Stroke::NONE,
            ));
        }
        _ => {
            p.line_segment([pos2(tail, at.y - 5.0), pos2(tip, at.y)], Stroke::new(1.6, c));
            p.line_segment([pos2(tip, at.y), pos2(tail, at.y + 5.0)], Stroke::new(1.6, c));
        }
    }
}

/// The label's colour.
fn ink(m: &Modern, s: State, style: Style) -> Color32 {
    match () {
        _ if s.off => m.theme.color("muted"),
        _ if s.on && matches!(style, Style::Hairline | Style::Gilt) => m.theme.color("gold"),
        _ => m.theme.color("text"),
    }
}

/// A row's background from its face in the pack's names ("controls/list_row_selected", "_hover", "_disabled",
/// "_normal") in the current style: the framed art, else the style's look; a tint other than white dims it.
pub fn face(p: &Painter, m: &Modern, r: Rect, face: &str, tint: Color32) {
    face_with(p, m, r, face, tint, false)
}

/// `face` without the cursor row's mark (Dusk's hourglass) when `bare`.
pub fn face_with(p: &Painter, m: &Modern, r: Rect, face: &str, tint: Color32, bare: bool) {
    let style = current(m);
    if style == Style::Framed || !face.starts_with("controls/list_row_") {
        if let Some(s) = m.art.sprite(face) {
            p.add(paint::nine_slice(&s, r, (r.height() / s.size.y).min(1.0), tint));
        }
        return;
    }
    let s = State {
        on: face.ends_with("_selected"),
        hover: face.ends_with("_hover"),
        off: face.ends_with("_disabled"),
        bare,
    };
    let mut q = p.clone();
    if tint != Color32::WHITE {
        q.multiply_opacity(0.55);
    }
    back(&q, m, r, s, style);
}

/// The label colour of a row (gold on the cursor in the Hairline and Gilt styles).
pub fn label_colour(m: &Modern, on: bool, off: bool) -> Color32 {
    ink(m, State { on, off, ..Default::default() }, current(m))
}

/// The submenu mark at a row's right in the current style; the rect it takes (a click there opens the submenu).
pub fn more_mark(p: &Painter, m: &Modern, r: Rect, on: bool) -> Rect {
    more(p, m, r, State { on, ..Default::default() }, current(m), false);
    Rect::from_center_size(pos2(r.max.x - 18.0, r.center().y), vec2(22.0, 22.0))
}

/// A menu row in the current style (Modern::row_style): its icon, its label, the submenu mark when `sub` (pointing left
/// when `left`, the submenu opening at that side); hovered by the pointer or `hover`. Its response.
pub fn menu_row(
    ui: &mut Ui,
    m: &Modern,
    icon: &str,
    label: &str,
    on: bool,
    off: bool,
    sub: bool,
    left: bool,
    hover: bool,
) -> egui::Response {
    let style = current(m);
    let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 32.0), Sense::click());
    let s = State { on, off, hover: hover || resp.hovered(), ..Default::default() };
    let p = ui.painter();
    back(p, m, r, s, style);
    let face = super::menu_family();
    if let Some(sp) = m.art.sprite(icon) {
        p.add(paint::image(
            &sp,
            pos2(r.min.x + 20.0, r.center().y - 11.0),
            22.0,
            if off { Color32::from_gray(110) } else { Color32::WHITE },
        ));
    }
    p.text(
        pos2(r.min.x + 50.0, r.center().y),
        Align2::LEFT_CENTER,
        label,
        FontId::new(m.theme.ui, face),
        ink(m, s, style),
    );
    if sub {
        more(p, m, r, s, style, left);
    }
    ui.add_space(2.0);
    resp
}
