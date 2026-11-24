//! Timeless XI: the Timeless title's hero logo over the reel (layout 1: centred in the upper part of the screen), the
//! match cut's afterglow in its crystal, the copyright line the retail art carried, and /titlecam's guide in the world.

use super::super::World;
use super::super::timeless::title::{LOGO_CRYSTAL, LogoPlace};
use super::{Modern, Panels};
use egui::{
    Align2, Color32, Context, FontFamily, FontId, Id, LayerId, Order, Rect, Shape, Stroke, TextureHandle, pos2, vec2,
};

// The logo with a hole where its hourglass moves, and the hourglass's moving part as 210 frames over 7 s (30 a second,
// 14 a row): the star crystal turning, the sand falling, the wisps. Frame 0 alone stands in while the sheet decodes off
// the UI thread, so the logo shows at once. From the v4 logo artwork (how: claude/title_screen.md).
const BASE_PNG: &[u8] = include_bytes!("../../../assets/timeless/title_logo_base.png");
const SAND0_PNG: &[u8] = include_bytes!("../../../assets/timeless/title_logo_sand0.png");
const SAND_PNG: &[u8] = include_bytes!("../../../assets/timeless/title_logo_sand.png");
const FRAMES: usize = 210;
const COLS: usize = 14;
const LOOP_SECS: f32 = 7.0;
/// Where the hourglass frames sit in the logo, as fractions of it: x0, y0, x1, y1 (logo pixels 556..782 x 158..522).
const SAND_RECT: [f32; 4] = [556.0 / 1329.0, 158.0 / 846.0, 782.0 / 1329.0, 522.0 / 846.0];
/// The line under the logo (the website's).
const TAGLINE: &str = "A LEVEL 75 FINAL FANTASY XI PRIVATE SERVER";

fn decode(png_bytes: &[u8]) -> Option<egui::ColorImage> {
    let mut d = png::Decoder::new(std::io::Cursor::new(png_bytes));
    d.set_transformations(png::Transformations::normalize_to_color8() | png::Transformations::ALPHA);
    let mut r = d.read_info().ok()?;
    let mut buf = vec![0u8; r.output_buffer_size()?];
    let info = r.next_frame(&mut buf).ok()?;
    buf.truncate(info.buffer_size());
    Some(egui::ColorImage::from_rgba_unmultiplied([info.width as usize, info.height as usize], &buf))
}

fn texture(ctx: &Context, key: &str, png_bytes: &[u8]) -> Option<TextureHandle> {
    let id = Id::unique(key);
    if let Some(t) = ctx.data(|d| d.get_temp::<TextureHandle>(id)) {
        return Some(t);
    }
    let t = ctx.load_texture(key, decode(png_bytes)?, egui::TextureOptions::LINEAR);
    ctx.data_mut(|d| d.insert_temp(id, t.clone()));
    Some(t)
}

/// The frame sheet once a worker thread has decoded it (it is large); none until then.
fn sheet(ctx: &Context) -> Option<TextureHandle> {
    use std::sync::{Mutex, OnceLock};
    static DECODED: OnceLock<Mutex<Option<egui::ColorImage>>> = OnceLock::new();
    let id = Id::unique("timeless/title_logo_sand");
    if let Some(t) = ctx.data(|d| d.get_temp::<TextureHandle>(id)) {
        return Some(t);
    }
    let started = DECODED.get().is_some();
    let slot = DECODED.get_or_init(|| Mutex::new(None));
    if !started {
        let c = ctx.clone();
        std::thread::spawn(move || {
            let img = decode(SAND_PNG);
            if img.is_none() {
                log::error!("title logo: the hourglass sheet does not decode");
            }
            *slot.lock().unwrap() = img;
            c.request_repaint();
        });
        return None;
    }
    let img = slot.lock().unwrap().take()?;
    let t = ctx.load_texture("timeless/title_logo_sand", img, egui::TextureOptions::LINEAR);
    ctx.data_mut(|d| d.insert_temp(id, t.clone()));
    Some(t)
}

/// The UV rect of hourglass frame `i` in the sheet.
fn frame_uv(i: usize) -> Rect {
    let rows = FRAMES.div_ceil(COLS);
    let (c, r) = ((i % FRAMES) % COLS, (i % FRAMES) / COLS);
    let (w, h) = (1.0 / COLS as f32, 1.0 / rows as f32);
    Rect::from_min_size(pos2(c as f32 * w, r as f32 * h), vec2(w, h))
}

/// Timeless XI: the logo's hourglass (the moving part) over the logo drawn in `rect`, at `a`.
fn hourglass(ctx: &Context, p: &egui::Painter, rect: Rect, a: f32) {
    let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
    // The hourglass, playing at 30 frames a second.
    let at = |f: f32| rect.min + vec2(f * rect.width(), 0.0);
    let glass = Rect::from_min_max(
        pos2(at(SAND_RECT[0]).x, rect.min.y + SAND_RECT[1] * rect.height()),
        pos2(at(SAND_RECT[2]).x, rect.min.y + SAND_RECT[3] * rect.height()),
    );
    let tint = Color32::from_white_alpha((a * 255.0) as u8);
    match sheet(ctx) {
        Some(sand) => {
            let t = ctx.input(|i| i.time) as f32 % LOOP_SECS / LOOP_SECS * FRAMES as f32;
            p.image(sand.id(), glass, frame_uv(t as usize), tint);
        }
        None => {
            if let Some(still) = texture(ctx, "timeless/title_logo_sand0", SAND0_PNG) {
                p.image(still.id(), glass, uv, tint);
            }
        }
    }
}

/// Timeless XI: the whole hero logo (no tagline) in `rect` at `alpha`, the hourglass moving: the setup wizard and the
/// Update Required card draw it where there is no title reel.
pub(super) fn hero_logo(ctx: &Context, p: &egui::Painter, rect: Rect, alpha: f32) {
    let px = (rect.width() * ctx.pixels_per_point()).round().max(1.0) as usize;
    let Some(base) = small_logo(ctx, px) else { return };
    let a = alpha.clamp(0.0, 1.0);
    // Down to the "Reforged" line, which is drawn as text below: at this size the picture's letters would be a few
    // pixels tall.
    let cut = REFORGED_TOP / LOGO_H;
    let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, cut));
    // On whole pixels, so the picture made at this size is drawn one to one (not blurred between pixels).
    let ppp = ctx.pixels_per_point();
    let snap = |v: f32| (v * ppp).round() / ppp;
    let min = pos2(snap(rect.min.x), snap(rect.min.y));
    let size = vec2(px as f32 / ppp, (px as f32 * LOGO_H / (LOGO_H * LOGO_ASPECT)).round() / ppp);
    let top = Rect::from_min_size(min, vec2(size.x, size.y * cut));
    p.image(base.id(), top, uv, Color32::from_white_alpha((a * 255.0) as u8));
    reforged(ctx, p, rect, a);
    hourglass(ctx, p, rect, a);
    ctx.request_repaint();
}

/// The logo picture's height in pixels, and the top of its "Reforged" line (the rules, diamonds and letters).
const LOGO_H: f32 = 846.0;
const REFORGED_TOP: f32 = 780.0;

/// The "— ◆ R E F O R G E D ◆ —" line under the small logo, as text and lines placed where the picture has them,
/// the letters at a size that reads.
fn reforged(ctx: &Context, p: &egui::Painter, rect: Rect, a: f32) {
    let k = rect.width() / (LOGO_H * LOGO_ASPECT);
    let at = |x: f32| rect.min.x + x * k;
    let y = rect.min.y + 808.5 * k;
    let ink = Color32::from_rgb(0xd6, 0xe2, 0xf2);
    // The letters as large as the picture's (at least 10 points, easy to read), centred; the diamonds just
    // outside them and the rules outside those, as in the picture.
    let size = (24.0 * k).max(10.0);
    let f = super::tlsetup::font(ctx, super::tlsetup::serif(size));
    let spacing = (34.0 * k).max(size * 0.45);
    let cx = at(668.0);
    let h = p.layout_no_wrap("R".into(), f.clone(), ink).size().y;
    let mut job = egui::text::LayoutJob::default();
    let tint = ink.gamma_multiply(a * 0.92);
    job.append(
        "REFORGED",
        0.0,
        egui::TextFormat { font_id: f, color: tint, extra_letter_spacing: spacing, ..Default::default() },
    );
    let g = p.layout_job(job);
    // The letters' own extent (no spacing beyond the first and last).
    let gl = || g.rows.iter().flat_map(|r| r.glyphs.iter());
    let x0 = gl().map(|gl| gl.pos.x).fold(f32::MAX, f32::min);
    let x1 = gl().map(|gl| gl.pos.x + gl.advance_width).fold(0.0f32, f32::max);
    let half = (x1 - x0) / 2.0;
    p.galley(pos2(cx - half - x0, y - h / 2.0), g.clone(), tint);
    let d = (4.0 * k).max(2.0);
    let gap = (size * 0.9).max(22.0 * k);
    let rule = (110.0 * k).max(size * 2.5);
    let line = egui::Stroke::new(1.0, ink.gamma_multiply(a * 0.55));
    for side in [-1.0f32, 1.0] {
        let dx = cx + side * (half + gap);
        p.add(egui::Shape::convex_polygon(
            vec![pos2(dx, y - d), pos2(dx + d, y), pos2(dx, y + d), pos2(dx - d, y)],
            ink.gamma_multiply(a * 0.8),
            egui::Stroke::NONE,
        ));
        let r0 = dx + side * (d + 4.0 * k.max(0.5));
        p.line_segment([pos2(r0, y), pos2(r0 + side * rule, y)], line);
    }
}

/// The logo picture shrunk to `px` pixels wide with an area average (a GPU's single sample per pixel skips the thin
/// strokes of its letters); the last size made is kept.
fn small_logo(ctx: &Context, px: usize) -> Option<TextureHandle> {
    let id = Id::unique("timeless/title_logo_small");
    if let Some((w, t)) = ctx.data(|d| d.get_temp::<(usize, TextureHandle)>(id)) {
        if w.abs_diff(px) * 50 <= px {
            return Some(t);
        }
    }
    let src = decode(BASE_PNG)?;
    let [sw, sh] = src.size;
    let w = px.clamp(16, sw);
    let h = ((sh as f32) * w as f32 / sw as f32).round().max(1.0) as usize;
    let img = area_average(&src, w, h);
    let t = ctx.load_texture("timeless/title_logo_small", img, egui::TextureOptions::LINEAR);
    ctx.data_mut(|d| d.insert_temp(id, (px, t.clone())));
    Some(t)
}

/// `src` resampled to `w`×`h` by averaging each output pixel's footprint (premultiplied, so edges keep their colour).
fn area_average(src: &egui::ColorImage, w: usize, h: usize) -> egui::ColorImage {
    let [sw, sh] = src.size;
    let (fx, fy) = (sw as f32 / w as f32, sh as f32 / h as f32);
    let mut out = Vec::with_capacity(w * h);
    for oy in 0..h {
        let (y0, y1) = (oy as f32 * fy, (oy + 1) as f32 * fy);
        for ox in 0..w {
            let (x0, x1) = (ox as f32 * fx, (ox + 1) as f32 * fx);
            let mut acc = [0f32; 4];
            let mut wsum = 0f32;
            for sy in (y0.floor() as usize)..(y1.ceil() as usize).min(sh) {
                let wy = (y1.min(sy as f32 + 1.0) - y0.max(sy as f32)).max(0.0);
                for sx in (x0.floor() as usize)..(x1.ceil() as usize).min(sw) {
                    let wx = (x1.min(sx as f32 + 1.0) - x0.max(sx as f32)).max(0.0);
                    let c = src.pixels[sy * sw + sx];
                    let wgt = wx * wy;
                    for (i, v) in c.to_array().iter().enumerate() {
                        acc[i] += *v as f32 * wgt;
                    }
                    wsum += wgt;
                }
            }
            let v = |i: usize| (acc[i] / wsum.max(1e-6)).round().clamp(0.0, 255.0) as u8;
            out.push(Color32::from_rgba_premultiplied(v(0), v(1), v(2), v(3)));
        }
    }
    egui::ColorImage::new([w, h], out)
}

/// The logo's width over its height.
pub(super) const LOGO_ASPECT: f32 = 1329.0 / 846.0;

impl Modern {
    /// The logo at `alpha`, with `glow` (the match cut's afterglow) and `flare` (its light, as the world goes) lighting
    /// its crystal.
    pub fn title_logo(&self, ctx: &Context, place: &LogoPlace, alpha: f32, glow: f32, flare: f32) {
        let screen = ctx.content_rect();
        let p = ctx.layer_painter(LayerId::new(Order::Background, Id::unique("timeless title logo")));
        if alpha <= 0.002 {
            return;
        }
        let Some(base) = texture(ctx, "timeless/title_logo_base", BASE_PNG) else { return };
        let r = place.rect([screen.width(), screen.height()]);
        let rect = Rect::from_min_size(pos2(screen.min.x + r[0], screen.min.y + r[1]), vec2(r[2], r[3]));
        let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
        let a = alpha.clamp(0.0, 1.0);
        p.image(base.id(), rect, uv, Color32::from_white_alpha((a * 255.0) as u8));
        // The line under the logo, as on the website (spaced capitals in the focus blue).
        let size = (rect.height() * 0.035).clamp(12.0, 24.0);
        let spacing = size * 0.3;
        let mut job = egui::text::LayoutJob::default();
        let colour = self.theme.color("focus").gamma_multiply(a * 0.9);
        job.append(
            TAGLINE,
            0.0,
            egui::TextFormat {
                font_id: FontId::new(size, FontFamily::Proportional),
                color: colour,
                extra_letter_spacing: spacing,
                ..Default::default()
            },
        );
        let g = p.layout_job(job);
        let top = rect.max.y + rect.height() * 0.075;
        p.galley(pos2(rect.center().x - g.size().x / 2.0 + spacing / 2.0, top), g, colour);
        hourglass(ctx, &p, rect, a);
        ctx.request_repaint();
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
