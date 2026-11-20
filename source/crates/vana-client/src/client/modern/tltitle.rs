//! Timeless XI: the Timeless title's hero logo over the reel (layout 1: centred in the upper part of the screen), the
//! match cut's afterglow in its crystal, the copyright line the retail art carried, and /titlecam's guide in the world.

use super::super::World;
use super::super::timeless::title::{LOGO_CRYSTAL, LogoPlace};
use super::{Modern, Panels};
use egui::{
    Align2, Color32, Context, FontFamily, FontId, Id, LayerId, Order, Rect, Shape, Stroke, TextureHandle, pos2, vec2,
};

const LOGO_PNG: &[u8] = include_bytes!("../../../assets/timeless/title_logo.png");

fn logo_texture(ctx: &Context) -> Option<TextureHandle> {
    let id = Id::unique("timeless title logo");
    if let Some(t) = ctx.data(|d| d.get_temp::<TextureHandle>(id)) {
        return Some(t);
    }
    let mut d = png::Decoder::new(std::io::Cursor::new(LOGO_PNG));
    d.set_transformations(png::Transformations::normalize_to_color8() | png::Transformations::ALPHA);
    let mut r = d.read_info().ok()?;
    let mut buf = vec![0u8; r.output_buffer_size()?];
    let info = r.next_frame(&mut buf).ok()?;
    buf.truncate(info.buffer_size());
    let img = egui::ColorImage::from_rgba_unmultiplied([info.width as usize, info.height as usize], &buf);
    let t = ctx.load_texture("timeless/title_logo", img, egui::TextureOptions::LINEAR);
    ctx.data_mut(|d| d.insert_temp(id, t.clone()));
    Some(t)
}

impl Modern {
    /// The logo at `alpha`, with `glow` (the match cut's afterglow) and `flare` (its light, as the world goes) lighting
    /// its crystal.
    pub fn title_logo(&self, ctx: &Context, place: &LogoPlace, alpha: f32, glow: f32, flare: f32) {
        let screen = ctx.content_rect();
        let p = ctx.layer_painter(LayerId::new(Order::Background, Id::unique("timeless title logo")));
        let muted = self.theme.color("muted");
        // The game's notice, which the retail title art carried (top left: the help bar holds the bottom).
        p.text(
            pos2(screen.min.x + 14.0, screen.min.y + 10.0),
            Align2::LEFT_TOP,
            "FINAL FANTASY XI \u{a9} SQUARE ENIX CO., LTD.",
            FontId::new(11.0, FontFamily::Proportional),
            muted.gamma_multiply(0.7),
        );
        if alpha <= 0.002 {
            return;
        }
        let Some(tex) = logo_texture(ctx) else { return };
        let r = place.rect([screen.width(), screen.height()]);
        let rect = Rect::from_min_size(pos2(screen.min.x + r[0], screen.min.y + r[1]), vec2(r[2], r[3]));
        let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
        p.image(tex.id(), rect, uv, Color32::from_white_alpha((alpha.clamp(0.0, 1.0) * 255.0) as u8));
        // The crystal's light: soft rings of the focus colour, strongest at the cut, fading after it.
        let light = (glow + flare * 0.6).min(1.2) * alpha.max(flare);
        if light > 0.01 {
            let c = pos2(rect.min.x + LOGO_CRYSTAL[0] * rect.width(), rect.min.y + LOGO_CRYSTAL[1] * rect.height());
            let base = rect.height() * 0.11;
            let focus = self.theme.color("focus");
            for i in 0..10 {
                let f = i as f32 / 9.0;
                let a = light * 0.10 * (1.0 - f).powf(1.5);
                p.add(Shape::circle_filled(c, base * (0.15 + f), focus.gamma_multiply(a)));
            }
            p.add(Shape::circle_filled(c, base * 0.08, Color32::from_white_alpha((light.min(1.0) * 180.0) as u8)));
        }
    }
}

/// /titlecam guide: where the logo sits on the title and the cross its crystal is on, to frame a match-cut crystal.
pub fn guide(p: &mut Panels, w: &World) {
    if !w.tl_cam.guide {
        return;
    }
    let place = w.tl_cam.reel.as_ref().map(|r| r.logo).unwrap_or_default();
    let screen = p.ctx.content_rect();
    let painter = p.ctx.layer_painter(LayerId::new(Order::Foreground, Id::unique("timeless titlecam guide")));
    let r = place.rect([screen.width(), screen.height()]);
    let rect = Rect::from_min_size(pos2(screen.min.x + r[0], screen.min.y + r[1]), vec2(r[2], r[3]));
    let gold = p.m.theme.color("gold").gamma_multiply(0.6);
    painter.rect_stroke(rect, 0.0, Stroke::new(1.0, gold), egui::StrokeKind::Inside);
    let c = pos2(rect.min.x + LOGO_CRYSTAL[0] * rect.width(), rect.min.y + LOGO_CRYSTAL[1] * rect.height());
    let s = Stroke::new(1.5, p.m.theme.color("focus"));
    painter.line_segment([c - vec2(14.0, 0.0), c + vec2(14.0, 0.0)], s);
    painter.line_segment([c - vec2(0.0, 14.0), c + vec2(0.0, 14.0)], s);
    painter.circle_stroke(c, 6.0, s);
    painter.text(
        pos2(rect.center().x, rect.max.y + 6.0),
        Align2::CENTER_TOP,
        "Title logo: frame the crystal on the cross, then /titlecam key",
        FontId::new(13.0, FontFamily::Proportional),
        gold,
    );
}
