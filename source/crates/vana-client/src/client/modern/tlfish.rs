//! Timeless XI: the Timeless Fishing bar game and item box on screen (the logic is `timeless/fishing.rs`). Drawn in
//! every interface (Classic, Modern and Timeless) over the game, in the Timeless palette, so everyone fishing on
//! Timeless XI sees the same thing. Look (owner 2026-11-19, from the HUD concepts canvas): the bar game is "Tide
//! Ledger" (A): a header strip under a gold rule with the catch's icon, its name, how it swims, a difficulty pill and
//! the time left, then the water track (catch zone, centre line, the fish as its own inventory icon, the Timeless
//! hourglass when there is none, a pulsing red "!" for a monster, a gold ring for a legendary), a ticked gold catch
//! meter, and a footer with the keys and the round's status. The item box is the "ring" (B): the item's icon inside
//! a ring that runs down with the time left, its name, and two pill keys.
//!
//! Owner 2026-11-19: with Interface = Timeless (Timeless View), every fishing window (these two, the Angler's HUD, the
//! School of Fish window and the Almanac, tlangler.rs) takes Timeless View's Pewter Hairline look: its palette
//! (charcoal glass, pewter edge, pale silver titles, steel-blue focus, the hourglass's gold sand), its Glass frame
//! (`borders::window`), titles in its serif. The Classic and Modern interfaces keep the Dusk look.

use super::super::World;
use super::super::timeless::fishing::{Done, Play, Round};
use crate::inventory::ItemInfo;
use egui::{Align2, Color32, Context, CornerRadius, FontId, Id, LayerId, Order, Pos2, Rect, Stroke, pos2, vec2};

const FULL_UV: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));

fn rgba(r: u8, g: u8, b: u8, a: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(r, g, b, (a * 255.0).round() as u8)
}

thread_local! {
    /// The fishing windows are in Timeless View's look this frame (Interface = Timeless, not Classic).
    static PEWTER: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Timeless View's Pewter Hairline look for the fishing windows this frame (set by `set_pewter`).
pub(super) fn pewter() -> bool {
    PEWTER.with(|c| c.get())
}

impl super::Modern {
    /// Whether this frame's fishing windows take Timeless View's look: Interface = Timeless (not in Classic, which
    /// keeps the Dusk look with the Modern interface).
    pub(super) fn set_pewter(&self, w: &World) {
        PEWTER.with(|c| c.set(w.modern && self.theme.timeless()));
    }

    /// A fishing window's frame behind `add`: Timeless View's Glass (`borders::window`, the crest on windows big
    /// enough) with `margin`, else `dusk` (the Classic / Modern look).
    pub(super) fn fish_frame<R>(
        &self,
        ui: &mut egui::Ui,
        dusk: egui::Frame,
        margin: egui::Margin,
        add: impl FnOnce(&mut egui::Ui) -> R,
    ) -> R {
        if pewter() {
            let bg = ui.painter().add(egui::Shape::Noop);
            let inner = egui::Frame::NONE.inner_margin(margin).show(ui, add);
            ui.painter().set(bg, super::borders::window(self, "window_normal", inner.response.rect));
            inner.inner
        } else {
            dusk.show(ui, add).inner
        }
    }
}

/// The font of a fishing window's title: Timeless View's serif, else the proportional face.
pub(super) fn title_font(size: f32) -> FontId {
    if pewter() { FontId::new(size * 1.05, super::menu_family()) } else { FontId::proportional(size) }
}

/// The palette: style D (Dusk Remastered) for the panel and text in Classic / Modern, Timeless View's Pewter
/// Hairline in Timeless (theme.json); the water stays blue.
struct Pal;
impl Pal {
    fn panel() -> Color32 {
        if pewter() { rgba(22, 26, 38, 0.95) } else { rgba(40, 32, 66, 0.94) }
    }
    fn edge() -> Color32 {
        if pewter() { rgba(107, 120, 146, 1.0) } else { rgba(77, 66, 117, 1.0) }
    }
    fn track() -> Color32 {
        if pewter() { rgba(11, 12, 19, 1.0) } else { rgba(20, 22, 44, 1.0) }
    }
    fn water_top() -> Color32 {
        rgba(16, 26, 44, 1.0)
    }
    /// The header's name: pale silver in Timeless View (its titles), the Dusk gold elsewhere.
    fn title() -> Color32 {
        if pewter() { rgba(214, 225, 240, 1.0) } else { Self::gold() }
    }
    /// The header strip's rule: silver fading in Timeless View, gold elsewhere.
    fn rule() -> Color32 {
        if pewter() { rgba(214, 225, 240, 0.45) } else { Self::gold() }
    }
    fn zone(inside: bool) -> Color32 {
        rgba(143, 178, 230, if inside { 0.48 } else { 0.30 })
    }
    fn zone_edge() -> Color32 {
        rgba(214, 225, 240, 0.85)
    }
    fn glow() -> Color32 {
        rgba(61, 95, 148, 0.45)
    }
    fn text() -> Color32 {
        if pewter() { rgba(230, 236, 245, 1.0) } else { rgba(246, 239, 224, 1.0) }
    }
    fn muted() -> Color32 {
        if pewter() { rgba(163, 174, 195, 1.0) } else { rgba(194, 182, 207, 1.0) }
    }
    fn gold() -> Color32 {
        rgba(240, 196, 110, 1.0)
    }
    fn red() -> Color32 {
        rgba(214, 48, 48, 1.0)
    }
    /// A sunken track (the item ring's, the pills', the footer's rule).
    fn sunk() -> Color32 {
        if pewter() { rgba(34, 40, 56, 1.0) } else { rgba(58, 48, 92, 1.0) }
    }
    /// The focus colour: Timeless View's steel blue, Dusk's lilac.
    fn focus() -> Color32 {
        if pewter() { rgba(143, 178, 230, 1.0) } else { rgba(169, 149, 214, 1.0) }
    }
    fn white() -> Color32 {
        rgba(245, 245, 250, 1.0)
    }
}

impl super::Modern {
    /// The bar game or item box, if one is up. Called every frame in the world, in every interface.
    pub fn tl_fishing(&self, ctx: &Context, w: &World) {
        let Some(play) = w.tl_fish.play.as_ref() else { return };
        self.set_pewter(w);
        let painter = ctx.layer_painter(LayerId::new(Order::Foreground, Id::unique("tl fishing")));
        let screen = ctx.content_rect();
        match play {
            Play::Bar(r) => self.tl_fish_bar(ctx, w, &painter, screen, r),
            Play::Item(p) => {
                let left = (p.time_limit() - p.t).max(0.0);
                self.tl_fish_item(
                    ctx,
                    w,
                    &painter,
                    screen,
                    p.bite.item,
                    p.bite.chest(),
                    p.t,
                    left / p.time_limit(),
                    left,
                );
            }
        }
    }

    fn tl_item_icon(&self, ctx: &Context, w: &World, id: u16) -> Option<egui::TextureId> {
        let info = w.item_info.as_ref()?;
        (id != 0 && id != 0xffff).then(|| self.item_icon(ctx, info, id)).flatten()
    }

    fn tl_fish_bar(&self, ctx: &Context, w: &World, gp: &egui::Painter, screen: Rect, g: &Round) {
        let (pw, ph) = (600.0, 164.0);
        let panel = Rect::from_center_size(screen.center(), vec2(pw, ph));
        let head_h = 44.0;
        if pewter() {
            // Timeless View: the Glass frame (crest on the top edge), the header under a silver rule fading out.
            gp.add(super::borders::window(self, "window_normal", panel));
            super::borders::fade_rule(gp, panel.min.x + 6.0, panel.max.x - 6.0, panel.min.y + head_h, Pal::rule());
        } else {
            gp.rect_filled(panel, CornerRadius::same(10), Pal::panel());
            gp.rect_filled(
                Rect::from_min_size(panel.min, vec2(pw, head_h)),
                CornerRadius { nw: 10, ne: 10, sw: 0, se: 0 },
                Pal::track(),
            );
            gp.line_segment(
                [pos2(panel.min.x, panel.min.y + head_h), pos2(panel.max.x, panel.min.y + head_h)],
                Stroke::new(1.5, Pal::rule()),
            );
            gp.rect_stroke(panel, CornerRadius::same(10), Stroke::new(1.5, Pal::edge()), egui::StrokeKind::Inside);
        }

        // Header: icon slot, name, how it swims; difficulty pill and time left on the right.
        let slot = Rect::from_min_size(panel.min + vec2(14.0, 7.0), vec2(30.0, 30.0));
        gp.rect(
            slot,
            CornerRadius::same(if pewter() { 2 } else { 6 }),
            Pal::panel(),
            Stroke::new(1.5, if g.bite.monster() { Pal::red() } else { Pal::title() }),
            egui::StrokeKind::Inside,
        );
        if g.bite.monster() {
            alert(gp, slot.center(), 9.0, g.t);
        } else {
            self.catch_icon(ctx, w, gp, g.bite.item, slot.center(), 24.0, g.t);
        }
        let name = if g.bite.monster() {
            "Monster!".to_string()
        } else {
            w.item_info
                .as_ref()
                .and_then(|i| i.name(g.bite.item))
                .unwrap_or_else(|| if g.practice { "Practice".to_string() } else { "Fish on the line".to_string() })
        };
        let nx = slot.max.x + 12.0;
        gp.text(
            pos2(nx, panel.min.y + 5.0),
            Align2::LEFT_TOP,
            name,
            title_font(16.0),
            if g.bite.monster() { Pal::red() } else { Pal::title() },
        );
        let mut sub =
            if g.bite.monster() { "Something big is fighting back" } else { swimmer(g.bite.behaviour) }.to_string();
        if g.bite.legendary() {
            sub.push_str("  -  Legendary");
        }
        if g.practice && !g.bite.monster() && g.bite.item != 0 {
            sub.push_str("  -  Practice");
        }
        gp.text(pos2(nx, panel.min.y + 25.0), Align2::LEFT_TOP, sub, FontId::proportional(12.0), Pal::muted());
        let left = gp.text(
            pos2(panel.max.x - 16.0, panel.min.y + head_h / 2.0),
            Align2::RIGHT_CENTER,
            mmss(self_left(g)),
            FontId::proportional(14.0),
            Pal::text(),
        );
        let (label, col) = difficulty(g.bite.difficulty);
        let galley = gp.layout_no_wrap(label.to_string(), FontId::proportional(12.0), col);
        let pill = Rect::from_min_max(
            pos2(left.min.x - 12.0 - galley.size().x - 22.0, panel.min.y + 11.0),
            pos2(left.min.x - 12.0, panel.min.y + 33.0),
        );
        gp.rect(pill, CornerRadius::same(11), Pal::sunk(), Stroke::new(1.0, col), egui::StrokeKind::Inside);
        gp.galley(pos2(pill.min.x + 11.0, pill.center().y - galley.size().y / 2.0), galley, col);

        // The water track, darker at the bottom.
        let track = Rect::from_min_size(panel.min + vec2(18.0, head_h + 12.0), vec2(pw - 36.0, 46.0));
        gp.rect_filled(track, CornerRadius::same(5), Pal::track());
        gp.rect_filled(
            Rect::from_min_max(track.min, pos2(track.max.x, track.center().y)),
            CornerRadius::same(5),
            Pal::water_top(),
        );
        gp.rect_stroke(track, CornerRadius::same(5), Stroke::new(1.0, Pal::edge()), egui::StrokeKind::Inside);

        // The catch zone.
        let x_at = |f: f32| track.min.x + track.width() * f;
        let zone = Rect::from_min_max(
            pos2(x_at(g.bar_x - g.bite.bar / 2.0), track.min.y + 3.0),
            pos2(x_at(g.bar_x + g.bite.bar / 2.0), track.max.y - 3.0),
        );
        gp.rect(
            zone,
            CornerRadius::same(5),
            Pal::zone(g.inside),
            Stroke::new(1.5, Pal::zone_edge()),
            egui::StrokeKind::Inside,
        );

        // The centre line (the zone's resting place).
        let cx = x_at(0.5);
        gp.line_segment([pos2(cx, track.min.y - 4.0), pos2(cx, track.max.y + 4.0)], Stroke::new(1.2, Pal::white()));

        // The fish: its icon, the hourglass, or the monster's "!".
        let at = pos2(x_at(g.fish_x), track.center().y);
        let size = 32.0;
        if g.bite.monster() {
            alert(gp, at, size * 0.45, g.t);
        } else {
            gp.circle_filled(at, size * 0.6, Pal::glow());
            self.catch_icon(ctx, w, gp, g.bite.item, at, size, g.t);
        }
        if g.bite.legendary() {
            gp.circle_stroke(at, size * 0.62, Stroke::new(1.5, Pal::gold()));
        }

        // The catch meter: "Catch", a gold bar with quarter ticks, the percentage.
        let my = track.max.y + 12.0;
        gp.text(pos2(track.min.x, my + 5.0), Align2::LEFT_CENTER, "Catch", FontId::proportional(12.0), Pal::muted());
        let meter = Rect::from_min_max(pos2(track.min.x + 46.0, my), pos2(track.max.x - 48.0, my + 10.0));
        gp.rect_filled(meter, CornerRadius::same(5), Pal::track());
        if g.meter > 0.0 {
            gp.rect_filled(
                Rect::from_min_size(meter.min, vec2(meter.width() * g.meter.min(1.0), meter.height())),
                CornerRadius::same(5),
                Pal::gold(),
            );
        }
        for q in 1..4 {
            let qx = meter.min.x + meter.width() * q as f32 / 4.0;
            gp.line_segment([pos2(qx, meter.min.y), pos2(qx, meter.max.y)], Stroke::new(1.0, Pal::panel()));
        }
        gp.rect_stroke(meter, CornerRadius::same(5), Stroke::new(1.0, Pal::edge()), egui::StrokeKind::Inside);
        gp.text(
            pos2(track.max.x, my + 5.0),
            Align2::RIGHT_CENTER,
            format!("{}%", (g.meter * 100.0).round() as i32),
            FontId::proportional(13.0),
            Pal::gold(),
        );

        // Footer: the keys, and the round's status on the right.
        let fy = panel.max.y - 30.0;
        gp.line_segment([pos2(panel.min.x + 1.0, fy), pos2(panel.max.x - 1.0, fy)], Stroke::new(1.0, Pal::sunk()));
        let ky = fy + 6.0;
        let mut x = keycap(gp, pos2(panel.min.x + 18.0, ky), "LEFT / RIGHT");
        x = gp
            .text(pos2(x + 6.0, ky + 9.0), Align2::LEFT_CENTER, "Steer", FontId::proportional(13.0), Pal::muted())
            .max
            .x;
        x = keycap(gp, pos2(x + 12.0, ky), "ESC");
        gp.text(pos2(x + 6.0, ky + 9.0), Align2::LEFT_CENTER, "Give up", FontId::proportional(13.0), Pal::muted());
        let status = match g.done {
            Some(Done::Win) if g.perfect => Some(("Perfect!", Pal::gold())),
            Some(Done::Win) => Some(("Caught!", Pal::text())),
            Some(Done::Lose) => Some(("It got away...", Pal::text())),
            Some(Done::GiveUp) => Some(("You give up.", Pal::text())),
            None if g.perfect && g.t > 0.5 => Some(("Perfect so far", rgba(120, 200, 140, 1.0))),
            None => None,
        };
        if let Some((s, c)) = status {
            gp.text(pos2(panel.max.x - 18.0, ky + 9.0), Align2::RIGHT_CENTER, s, FontId::proportional(13.0), c);
        }
    }

    /// The catch's inventory icon centred on `c`, or the Timeless hourglass when it has none.
    #[allow(clippy::too_many_arguments)]
    fn catch_icon(&self, ctx: &Context, w: &World, gp: &egui::Painter, item: u16, c: Pos2, size: f32, t: f32) {
        match self.tl_item_icon(ctx, w, item) {
            Some(tex) => {
                gp.image(tex, Rect::from_center_size(c, vec2(size, size)), FULL_UV, Color32::WHITE);
            }
            None => super::tlbar::hourglass(gp, self, Rect::from_center_size(c, vec2(size, size)), 0.6, 1.0, t),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn tl_fish_item(
        &self,
        ctx: &Context,
        w: &World,
        gp: &egui::Painter,
        screen: Rect,
        item: u16,
        chest: bool,
        t: f32,
        left: f32,
        secs: f32,
    ) {
        let (pw, ph) = (440.0, 136.0);
        let panel = Rect::from_center_size(screen.center(), vec2(pw, ph));
        if pewter() {
            gp.add(super::borders::window(self, "window_normal", panel));
        } else {
            gp.rect(
                panel,
                CornerRadius::same(18),
                Pal::panel(),
                Stroke::new(1.5, Pal::edge()),
                egui::StrokeKind::Inside,
            );
        }

        // The time ring around the icon: blue, then amber, then red.
        let f = left.clamp(0.0, 1.0);
        let ring = if f > 0.3 {
            rgba(127, 196, 232, 1.0)
        } else if f > 0.15 {
            rgba(232, 176, 74, 1.0)
        } else {
            Pal::red()
        };
        let c = pos2(panel.min.x + 22.0 + 46.0, panel.center().y);
        let r = 44.0;
        gp.circle_stroke(c, r, Stroke::new(6.0, Pal::sunk()));
        arc(
            gp,
            c,
            r,
            -std::f32::consts::FRAC_PI_2,
            -std::f32::consts::FRAC_PI_2 + std::f32::consts::TAU * f,
            Stroke::new(6.0, ring),
        );
        // The icon disc with a soft pulsing gold edge.
        let pulse = 0.5 + 0.5 * (t * std::f32::consts::PI * 1.25).sin();
        gp.circle_filled(c, 30.0, Pal::track());
        gp.circle_stroke(c, 30.0, Stroke::new(1.5, rgba(240, 196, 110, 0.6 + 0.4 * pulse)));
        let ic = c - vec2(0.0, 1.5 * (t * std::f32::consts::PI * 1.25).sin());
        if chest {
            super::tlbar::hourglass(gp, self, Rect::from_center_size(ic, vec2(34.0, 34.0)), 0.6, 1.0, t);
        } else {
            self.catch_icon(ctx, w, gp, item, ic, 34.0, t);
        }

        // The text and the two pill keys.
        let tx = c.x + r + 22.0;
        gp.text(
            pos2(tx, panel.min.y + 20.0),
            Align2::LEFT_TOP,
            format!("Something is on the line  -  {}", mmss(secs)),
            FontId::proportional(13.0),
            Pal::muted(),
        );
        let name = if chest {
            "Treasure Chest".to_string()
        } else {
            w.item_info.as_ref().and_then(|i| i.name(item)).unwrap_or_else(|| "Something heavy".to_string())
        };
        gp.text(pos2(tx, panel.min.y + 40.0), Align2::LEFT_TOP, name, title_font(19.0), Pal::title());
        let x = pill(gp, pos2(tx, panel.min.y + 84.0), "ENTER", "Reel in", true);
        pill(gp, pos2(x + 8.0, panel.min.y + 84.0), "ESC", "Release", false);
    }
}

/// How the fish swims (the 0x115's behaviour byte).
fn swimmer(b: u8) -> &'static str {
    match b {
        1 => "Smooth swimmer",
        2 => "Sinker",
        3 => "Floater",
        4 => "Darter",
        _ => "Mixed swimmer",
    }
}

/// The difficulty's label and colour: the Angler's cut-offs (angler.rs difficulty labels).
fn difficulty(d: u16) -> (&'static str, Color32) {
    match d {
        0..=40 => ("Easy", rgba(120, 200, 140, 1.0)),
        41..=55 => ("Normal", Pal::focus()),
        56..=70 => ("Tricky", Pal::gold()),
        71..=85 => ("Hard", rgba(232, 130, 80, 1.0)),
        _ => ("Very hard", rgba(224, 96, 108, 1.0)),
    }
}

fn self_left(g: &Round) -> f32 {
    (g.time_limit() - g.t).max(0.0)
}

fn mmss(secs: f32) -> String {
    let s = secs.round().max(0.0) as u32;
    format!("{}:{:02}", s / 60, s % 60)
}

/// An arc from angle `a0` to `a1` (radians, 0 = right, clockwise on screen).
pub(super) fn arc(gp: &egui::Painter, c: Pos2, r: f32, a0: f32, a1: f32, stroke: Stroke) {
    if a1 - a0 < 0.01 {
        return;
    }
    let n = ((48.0 * (a1 - a0) / std::f32::consts::TAU).ceil() as usize).max(4);
    let pts: Vec<Pos2> = (0..=n)
        .map(|i| {
            let a = a0 + (a1 - a0) * i as f32 / n as f32;
            pos2(c.x + r * a.cos(), c.y + r * a.sin())
        })
        .collect();
    gp.add(egui::Shape::line(pts, stroke));
}

/// A pill key ("ENTER   Reel in"), filled for the main one; the x it ends at.
fn pill(gp: &egui::Painter, at: Pos2, key: &str, label: &str, main: bool) -> f32 {
    let galley = gp.layout_no_wrap(format!("{key}   {label}"), FontId::proportional(13.0), Pal::text());
    let r = Rect::from_min_size(at, vec2(galley.size().x + 26.0, 28.0));
    gp.rect(
        r,
        CornerRadius::same(14),
        if main { Pal::sunk() } else { Pal::track() },
        Stroke::new(1.0, if main { Pal::focus() } else { Pal::edge() }),
        egui::StrokeKind::Inside,
    );
    gp.galley(
        pos2(r.min.x + 13.0, r.center().y - galley.size().y / 2.0),
        galley,
        if main { Pal::text() } else { Pal::muted() },
    );
    r.max.x
}

/// A key's cap ("ENTER"); the x it ends at.
fn keycap(gp: &egui::Painter, at: Pos2, label: &str) -> f32 {
    let galley = gp.layout_no_wrap(label.to_string(), FontId::proportional(13.0), Pal::text());
    let r = Rect::from_min_size(at, vec2(galley.size().x + 14.0, 18.0));
    gp.rect_filled(r, CornerRadius::same(4), rgba(34, 42, 60, 1.0));
    gp.rect_filled(
        Rect::from_min_max(pos2(r.min.x, r.max.y - 3.0), r.max),
        CornerRadius::same(4),
        rgba(70, 82, 112, 1.0),
    );
    gp.rect_stroke(r, CornerRadius::same(4), Stroke::new(1.0, rgba(90, 103, 136, 1.0)), egui::StrokeKind::Inside);
    gp.galley(pos2(r.min.x + 7.0, r.center().y - galley.size().y / 2.0), galley, Pal::text());
    r.max.x
}

/// The monster's pulsing, flashing red "!".
fn alert(gp: &egui::Painter, c: Pos2, size: f32, t: f32) {
    let pulse = 0.5 + 0.5 * (t * std::f32::consts::PI * 3.0).sin();
    let r = size * (0.9 + 0.15 * pulse);
    gp.circle_filled(c, r * (1.35 + 0.35 * pulse), rgba(214, 48, 48, 0.18 + 0.22 * pulse));
    gp.circle_filled(c, r, rgba(214 + (41.0 * pulse) as u8, 48 + (42.0 * pulse) as u8, 48 + (26.0 * pulse) as u8, 1.0));
    gp.circle_stroke(c, r, Stroke::new(1.5, rgba(255, 190, 190, 0.9)));
    let bw = r * 0.26;
    gp.rect_filled(
        Rect::from_min_max(pos2(c.x - bw / 2.0, c.y - r * 0.62), pos2(c.x + bw / 2.0, c.y + r * 0.18)),
        CornerRadius::same((bw / 2.0) as u8),
        Pal::white(),
    );
    gp.circle_filled(pos2(c.x, c.y + r * 0.46), bw * 0.62, Pal::white());
}
