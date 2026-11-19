//! Timeless XI: the Timeless Fishing bar game and item box on screen (the logic is `timeless/fishing.rs`). Drawn in
//! every interface (Classic, Modern and Timeless) over the game, in the Timeless palette, so everyone fishing on
//! Timeless XI sees the same thing. The look is the add-on's (timelessfishing 2.16): a panel in the middle of the
//! screen, the water track with the catch zone and the centre line, the fish as its own inventory icon (the Timeless
//! hourglass when there is none, a pulsing red "!" for a monster, a gold ring for a legendary), the catch meter under
//! it, the title left and the hint right.

use super::super::World;
use super::super::timeless::fishing::{Done, Play, Round};
use crate::inventory::ItemInfo;
use egui::{Align2, Color32, Context, CornerRadius, FontId, Id, LayerId, Order, Pos2, Rect, Stroke, pos2, vec2};

const FULL_UV: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));

fn rgba(r: u8, g: u8, b: u8, a: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(r, g, b, (a * 255.0).round() as u8)
}

/// The Timeless palette (the website's and the add-on's).
struct Pal;
impl Pal {
    fn panel() -> Color32 {
        rgba(10, 11, 18, 0.92)
    }
    fn edge() -> Color32 {
        rgba(40, 48, 64, 1.0)
    }
    fn track() -> Color32 {
        rgba(17, 20, 29, 1.0)
    }
    fn water_top() -> Color32 {
        rgba(16, 26, 44, 1.0)
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
        rgba(230, 236, 245, 1.0)
    }
    fn muted() -> Color32 {
        rgba(152, 163, 184, 1.0)
    }
    fn gold() -> Color32 {
        rgba(232, 200, 120, 1.0)
    }
    fn red() -> Color32 {
        rgba(214, 48, 48, 1.0)
    }
    fn white() -> Color32 {
        rgba(245, 245, 250, 1.0)
    }
}

fn lerp(a: Color32, b: Color32, f: f32) -> Color32 {
    let m = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * f).round() as u8;
    Color32::from_rgba_unmultiplied(m(a.r(), b.r()), m(a.g(), b.g()), m(a.b(), b.b()), m(a.a(), b.a()))
}

impl super::Modern {
    /// The bar game or item box, if one is up. Called every frame in the world, in every interface.
    pub fn tl_fishing(&self, ctx: &Context, w: &World) {
        let Some(play) = w.tl_fish.play.as_ref() else { return };
        let painter = ctx.layer_painter(LayerId::new(Order::Foreground, Id::unique("tl fishing")));
        let screen = ctx.content_rect();
        match play {
            Play::Bar(r) => self.tl_fish_bar(ctx, w, &painter, screen, r),
            Play::Item(p) => {
                let left = (p.time_limit() - p.t).max(0.0);
                self.tl_fish_item(ctx, w, &painter, screen, p.bite.item, p.bite.chest(), p.t, left / p.time_limit());
            }
        }
    }

    fn tl_item_icon(&self, ctx: &Context, w: &World, id: u16) -> Option<egui::TextureId> {
        let info = w.item_info.as_ref()?;
        (id != 0 && id != 0xffff).then(|| self.item_icon(ctx, info, id)).flatten()
    }

    fn tl_fish_bar(&self, ctx: &Context, w: &World, gp: &egui::Painter, screen: Rect, g: &Round) {
        let (pw, ph) = (600.0, 112.0);
        let panel = Rect::from_center_size(screen.center(), vec2(pw, ph));
        gp.rect(panel, CornerRadius::same(10), Pal::panel(), Stroke::new(1.5, Pal::edge()), egui::StrokeKind::Inside);

        // The water track, darker at the bottom.
        let track = Rect::from_min_size(panel.min + vec2(18.0, 26.0), vec2(pw - 36.0, 50.0));
        gp.rect_filled(track, CornerRadius::same(4), Pal::track());
        gp.rect_filled(
            Rect::from_min_max(track.min, pos2(track.max.x, track.center().y)),
            CornerRadius::same(4),
            Pal::water_top(),
        );
        gp.rect_stroke(track, CornerRadius::same(4), Stroke::new(1.0, Pal::edge()), egui::StrokeKind::Inside);

        // The catch zone.
        let x_at = |f: f32| track.min.x + track.width() * f;
        let zone = Rect::from_min_max(
            pos2(x_at(g.bar_x - g.bite.bar / 2.0), track.min.y + 2.0),
            pos2(x_at(g.bar_x + g.bite.bar / 2.0), track.max.y - 2.0),
        );
        gp.rect(
            zone,
            CornerRadius::same(4),
            Pal::zone(g.inside),
            Stroke::new(1.2, Pal::zone_edge()),
            egui::StrokeKind::Inside,
        );

        // The centre line (the zone's resting place).
        let cx = x_at(0.5);
        gp.line_segment([pos2(cx, track.min.y - 4.0), pos2(cx, track.max.y + 4.0)], Stroke::new(1.2, Pal::white()));

        // The fish: its icon, the hourglass, or the monster's "!".
        let at = pos2(x_at(g.fish_x), track.center().y);
        let size = 34.0;
        if g.bite.monster() {
            alert(gp, at, size * 0.45, g.t);
        } else if let Some(tex) = self.tl_item_icon(ctx, w, g.bite.item) {
            gp.circle_filled(at, size * 0.62, Pal::glow());
            gp.image(tex, Rect::from_center_size(at, vec2(size, size)), FULL_UV, Color32::WHITE);
        } else {
            gp.circle_filled(at, size * 0.62, Pal::glow());
            super::tlbar::hourglass(gp, self, Rect::from_center_size(at, vec2(size, size)), 0.6, 1.0, g.t);
        }
        if g.bite.legendary() {
            gp.circle_stroke(at, size * 0.62, Stroke::new(1.5, Pal::gold()));
        }

        // The catch meter.
        let meter = Rect::from_min_size(pos2(track.min.x, track.max.y + 10.0), vec2(track.width(), 8.0));
        gp.rect_filled(meter, CornerRadius::same(4), Pal::track());
        let fill = Rect::from_min_size(meter.min + vec2(1.0, 1.0), vec2((meter.width() - 2.0) * g.meter, 6.0));
        gp.rect_filled(fill, CornerRadius::same(3), lerp(rgba(61, 95, 148, 1.0), rgba(214, 225, 240, 1.0), g.meter));
        gp.rect_stroke(meter, CornerRadius::same(4), Stroke::new(1.0, Pal::edge()), egui::StrokeKind::Inside);

        // Title and hint.
        let title = if g.bite.monster() {
            "Monster!"
        } else if g.practice {
            "Practice"
        } else {
            "Reel it in"
        };
        gp.text(
            panel.min + vec2(18.0, 5.0),
            Align2::LEFT_TOP,
            title,
            FontId::proportional(15.0),
            if g.bite.monster() { Pal::red() } else { Pal::muted() },
        );
        let (hint, col) = match g.done {
            Some(Done::Win) if g.perfect => ("Perfect!", Pal::gold()),
            Some(Done::Win) => ("Caught!", Pal::text()),
            Some(Done::Lose) => ("It got away...", Pal::text()),
            Some(Done::GiveUp) => ("You give up.", Pal::text()),
            None => ("LEFT / RIGHT: steer   ESC: give up", Pal::text()),
        };
        gp.text(pos2(panel.max.x - 18.0, panel.min.y + 6.0), Align2::RIGHT_TOP, hint, FontId::proportional(14.0), col);
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
    ) {
        let (pw, ph) = (360.0, 80.0);
        let panel = Rect::from_center_size(screen.center(), vec2(pw, ph));
        gp.rect(panel, CornerRadius::same(10), Pal::panel(), Stroke::new(1.5, Pal::edge()), egui::StrokeKind::Inside);

        // The icon slot with a soft pulsing gold glow.
        let pulse = 0.5 + 0.5 * (t * std::f32::consts::PI * 1.25).sin();
        let slot = Rect::from_min_size(panel.min + vec2(14.0, 12.0), vec2(52.0, 52.0));
        gp.rect_filled(slot.expand(3.0), CornerRadius::same(10), rgba(232, 195, 90, 0.10 + 0.25 * pulse));
        gp.rect(
            slot,
            CornerRadius::same(8),
            rgba(14, 20, 32, 1.0),
            Stroke::new(1.5, Pal::gold()),
            egui::StrokeKind::Inside,
        );
        let c = slot.center() - vec2(0.0, 2.0 * (t * std::f32::consts::PI * 1.25).sin());
        match (!chest).then(|| self.tl_item_icon(ctx, w, item)).flatten() {
            Some(tex) => {
                gp.image(tex, Rect::from_center_size(c, vec2(40.0, 40.0)), FULL_UV, Color32::WHITE);
            }
            None => super::tlbar::hourglass(gp, self, Rect::from_center_size(c, vec2(36.0, 36.0)), 0.6, 1.0, t),
        }

        // The text and keys.
        let tx = slot.max.x + 14.0;
        gp.text(
            pos2(tx, panel.min.y + 9.0),
            Align2::LEFT_TOP,
            "Something is on the line!",
            FontId::proportional(14.0),
            Pal::muted(),
        );
        let name =
            if chest { Some("Treasure Chest".to_string()) } else { w.item_info.as_ref().and_then(|i| i.name(item)) };
        if let Some(name) = name {
            gp.text(pos2(tx, panel.min.y + 26.0), Align2::LEFT_TOP, name, FontId::proportional(16.0), Pal::gold());
        }
        let mut x = keycap(gp, pos2(tx, panel.min.y + 50.0), "ENTER");
        x = gp
            .text(
                pos2(x + 6.0, panel.min.y + 50.0),
                Align2::LEFT_TOP,
                "Reel in",
                FontId::proportional(14.0),
                Pal::text(),
            )
            .max
            .x;
        x = keycap(gp, pos2(x + 18.0, panel.min.y + 50.0), "ESC");
        gp.text(
            pos2(x + 6.0, panel.min.y + 50.0),
            Align2::LEFT_TOP,
            "Release",
            FontId::proportional(14.0),
            Pal::text(),
        );

        // Time left: blue, then amber, then red.
        let f = left.clamp(0.0, 1.0);
        let col = if f > 0.3 {
            rgba(127, 196, 232, 1.0)
        } else if f > 0.15 {
            rgba(232, 176, 74, 1.0)
        } else {
            Pal::red()
        };
        let bar =
            Rect::from_min_max(pos2(panel.min.x + 2.0, panel.max.y - 5.0), pos2(panel.max.x - 2.0, panel.max.y - 2.0));
        gp.rect_filled(bar, CornerRadius::same(1), rgba(14, 20, 32, 1.0));
        gp.rect_filled(Rect::from_min_size(bar.min, vec2(bar.width() * f, bar.height())), CornerRadius::same(1), col);
    }
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
