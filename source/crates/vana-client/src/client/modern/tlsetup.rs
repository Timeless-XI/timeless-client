//! Timeless XI: the setup wizard's window (client/timeless/setup.rs does the work) and the small kit it shares with
//! the Update Required card (tlgate.rs): the night backdrop with drifting sand, Pewter glass cards, serif titles,
//! steel-blue buttons, diamond choices, and the hourglass-gold progress bar. Timeless View's palette (Pewter Hairline),
//! drawn with egui alone: nothing of the game's is loaded yet when the setup runs.

use super::super::timeless::manifest::Manifest;
use super::super::timeless::setup::{self, Plan, STEPS, Source, Work};
use super::theme::{SERIF, STRONG};
use egui::{
    Align2, Color32, Context, CursorIcon, FontFamily, FontId, Id, Key, LayerId, Order, Painter, Pos2, Rect, Sense,
    Shape, Stroke, StrokeKind, Ui, pos2, vec2,
};
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

// Timeless View, Pewter Hairline (assets/timeless/theme.json).
pub(super) const CHARCOAL: Color32 = Color32::from_rgb(0x0b, 0x0c, 0x13);
pub(super) const SURFACE: Color32 = Color32::from_rgb(0x16, 0x1a, 0x26);
pub(super) const PEWTER: Color32 = Color32::from_rgb(0x6b, 0x78, 0x92);
pub(super) const SILVER: Color32 = Color32::from_rgb(0xd6, 0xe1, 0xf0);
pub(super) const FOCUS: Color32 = Color32::from_rgb(0x8f, 0xb2, 0xe6);
pub(super) const SAND: Color32 = Color32::from_rgb(0xf0, 0xc4, 0x6e);
pub(super) const TEXT: Color32 = Color32::from_rgb(0xe6, 0xec, 0xf5);
pub(super) const MUTED: Color32 = Color32::from_rgb(0xa3, 0xae, 0xc3);
pub(super) const ALERT: Color32 = Color32::from_rgb(0xe0, 0x60, 0x6c);

pub(super) fn serif(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(SERIF.into()))
}

pub(super) fn body(size: f32) -> FontId {
    FontId::new(size, FontFamily::Proportional)
}

pub(super) fn strong(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(STRONG.into()))
}

/// Whether the Timeless fonts are in (the serif family); egui's own otherwise.
fn has(ctx: &Context, family: &str) -> bool {
    ctx.fonts(|f| f.families().iter().any(|x| *x == FontFamily::Name(family.into())))
}

/// `serif` / `strong` when installed, else egui's Proportional (the Classic interface may not have them).
pub(super) fn font(ctx: &Context, f: FontId) -> FontId {
    match &f.family {
        FontFamily::Name(n) if !has(ctx, n) => FontId::new(f.size, FontFamily::Proportional),
        _ => f,
    }
}

/// Text in spaced capitals, its left edge at `pos.x` (or centred on it with `centre`).
pub(super) fn spaced(
    p: &Painter,
    pos: Pos2,
    text: &str,
    f: FontId,
    colour: Color32,
    spacing: f32,
    centre: bool,
) -> Rect {
    let mut job = egui::text::LayoutJob::default();
    job.append(
        text,
        0.0,
        egui::TextFormat { font_id: f, color: colour, extra_letter_spacing: spacing, ..Default::default() },
    );
    let g = p.layout_job(job);
    let x = if centre { pos.x - (g.size().x - spacing) / 2.0 } else { pos.x };
    let r = Rect::from_min_size(pos2(x, pos.y), g.size());
    p.galley(r.min, g, colour);
    r
}

/// Wrapped text in `width`, top-left at `pos`; its height.
pub(super) fn para(p: &Painter, pos: Pos2, text: &str, f: FontId, colour: Color32, width: f32) -> f32 {
    let g = p.layout(text.to_string(), f, colour, width);
    let h = g.size().y;
    p.galley(pos, g, colour);
    h
}

/// The night behind everything: deep blue at the centre to charcoal at the edges, a few stars, and gold sand drifting
/// down as in the logo's hourglass.
pub(super) fn backdrop(p: &Painter, screen: Rect, t: f64) {
    p.add(super::rows::gradient(screen, Color32::from_rgb(0x12, 0x17, 0x26), CHARCOAL, false));
    let glow = screen.center() + vec2(-screen.width() * 0.18, -screen.height() * 0.08);
    for i in 0..14 {
        let f = i as f32 / 13.0;
        p.circle_filled(
            glow,
            screen.height() * (0.15 + 0.6 * f),
            Color32::from_rgba_unmultiplied(0x8f, 0xb2, 0xe6, (5.0 * (1.0 - f)) as u8),
        );
    }
    // A fixed scatter (no randomness frame to frame), each star breathing on its own phase.
    let mut seed = 0x9e37_79b9u32;
    let mut next = || {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        (seed % 10_000) as f32 / 10_000.0
    };
    for _ in 0..90 {
        let (x, y, ph, s) = (next(), next(), next(), next());
        let a = 0.25 + 0.35 * (0.5 + 0.5 * ((t as f32) * (0.4 + s) + ph * 6.28).sin());
        let pos = pos2(screen.min.x + x * screen.width(), screen.min.y + y * screen.height());
        p.circle_filled(pos, 0.6 + s * 0.9, Color32::from_white_alpha((a * 120.0) as u8));
    }
    for _ in 0..26 {
        let (x, ph, sp, s) = (next(), next(), next(), next());
        let fall = ((t as f32) * (0.012 + sp * 0.02) + ph).fract();
        let pos = pos2(
            screen.min.x + (x + 0.015 * ((t as f32) * 0.3 + ph * 9.0).sin()) * screen.width(),
            screen.min.y + fall * screen.height(),
        );
        let a = (fall * (1.0 - fall) * 4.0).min(1.0) * 0.55;
        p.circle_filled(pos, 0.8 + s, SAND.gamma_multiply(a));
    }
    // A soft vignette.
    let edge = screen.width().max(screen.height());
    for i in 0..10 {
        let f = i as f32 / 9.0;
        p.rect_stroke(
            screen.shrink(f * edge * 0.06),
            0.0,
            Stroke::new(edge * 0.0065, Color32::from_black_alpha((26.0 * (1.0 - f)) as u8)),
            StrokeKind::Inside,
        );
    }
}

/// A Pewter glass card: the glass darkening downward, a pewter edge with a faint hairline inside, a soft shadow and,
/// on top, the logo's small diamond over a fading rule.
pub(super) fn card(p: &Painter, r: Rect) {
    for i in 1..=6 {
        let s = i as f32 * 2.5;
        p.rect_filled(
            r.expand(s).translate(vec2(0.0, s * 0.6)),
            4.0 + s,
            Color32::from_black_alpha((26 - i * 4) as u8),
        );
    }
    p.add(super::rows::gradient(r, SURFACE.gamma_multiply(0.97), CHARCOAL.gamma_multiply(0.97), false));
    p.rect_stroke(r, 2.0, Stroke::new(1.0, PEWTER), StrokeKind::Inside);
    p.rect_stroke(r.shrink(3.0), 1.0, Stroke::new(0.6, PEWTER.gamma_multiply(0.28)), StrokeKind::Inside);
    p.line_segment(
        [r.left_top() + vec2(6.0, 1.5), r.right_top() + vec2(-6.0, 1.5)],
        Stroke::new(1.0, Color32::from_white_alpha(10)),
    );
    let x = r.center().x;
    super::borders::fade_rule(p, r.min.x + 40.0, r.max.x - 40.0, r.min.y, PEWTER.gamma_multiply(0.9));
    diamond(p, pos2(x, r.min.y), 5.0, SILVER, Some(PEWTER));
}

pub(super) fn diamond(p: &Painter, c: Pos2, s: f32, fill: Color32, edge: Option<Color32>) {
    p.add(Shape::convex_polygon(
        vec![pos2(c.x, c.y - s * 1.3), pos2(c.x + s, c.y), pos2(c.x, c.y + s * 1.3), pos2(c.x - s, c.y)],
        fill,
        edge.map_or(Stroke::NONE, |e| Stroke::new(0.7, e)),
    ));
}

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Look {
    Primary,
    Secondary,
}

/// A button: the label in spaced serif capitals; Primary a steel-blue glass with the focus edge, Secondary a pewter
/// outline. Whether it was clicked.
pub(super) fn button(ui: &mut Ui, r: Rect, id: &str, label: &str, look: Look, enabled: bool) -> bool {
    let resp = ui.interact(r, Id::unique(("tl button", id)), if enabled { Sense::click() } else { Sense::hover() });
    let hover = enabled && resp.hovered();
    if hover {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }
    let p = ui.painter();
    let fade = if enabled { 1.0 } else { 0.4 };
    match look {
        Look::Primary => {
            let (top, bottom) = if hover {
                (Color32::from_rgb(0x2a, 0x3d, 0x60), Color32::from_rgb(0x18, 0x24, 0x3b))
            } else {
                (Color32::from_rgb(0x20, 0x2f, 0x4b), Color32::from_rgb(0x12, 0x1a, 0x2b))
            };
            if hover {
                for i in 1..=4 {
                    p.rect_stroke(
                        r.expand(i as f32 * 1.5),
                        3.0,
                        Stroke::new(1.5, FOCUS.gamma_multiply(0.10 / i as f32)),
                        StrokeKind::Outside,
                    );
                }
            }
            p.add(super::rows::gradient(r, top.gamma_multiply(fade), bottom.gamma_multiply(fade), false));
            p.rect_stroke(
                r,
                2.0,
                Stroke::new(1.0, FOCUS.gamma_multiply(fade * if hover { 1.0 } else { 0.8 })),
                StrokeKind::Inside,
            );
            p.line_segment(
                [r.left_top() + vec2(4.0, 1.5), r.right_top() + vec2(-4.0, 1.5)],
                Stroke::new(1.0, Color32::from_white_alpha(if hover { 34 } else { 20 })),
            );
        }
        Look::Secondary => {
            if hover {
                p.rect_filled(r, 2.0, Color32::from_white_alpha(8));
            }
            p.rect_stroke(
                r,
                2.0,
                Stroke::new(1.0, PEWTER.gamma_multiply(fade * if hover { 1.2 } else { 0.85 })),
                StrokeKind::Inside,
            );
        }
    }
    let f = font(ui.ctx(), serif(r.height() * 0.42));
    let colour = if look == Look::Primary { SILVER } else { TEXT.gamma_multiply(0.9) };
    let g = p.layout_no_wrap(label.to_uppercase(), f.clone(), colour);
    let spacing = r.height() * 0.06;
    let w = g.size().x + spacing * label.chars().count() as f32;
    spaced(
        p,
        pos2(r.center().x - w / 2.0 + spacing / 2.0, r.center().y - g.size().y / 2.0),
        &label.to_uppercase(),
        f,
        colour.gamma_multiply(fade),
        spacing,
        false,
    );
    enabled && resp.clicked()
}

/// A tick box with its label; whether it changed.
fn tick(ui: &mut Ui, at: Pos2, id: &str, label: &str, on: &mut bool) -> bool {
    let f = body(15.0);
    let g = ui.painter().layout_no_wrap(label.to_string(), f.clone(), TEXT);
    let r = Rect::from_min_size(at, vec2(22.0 + g.size().x, 20.0));
    let resp = ui.interact(r, Id::unique(("tl tick", id)), Sense::click());
    if resp.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }
    let changed = resp.clicked();
    if changed {
        *on = !*on;
    }
    let p = ui.painter();
    let b = Rect::from_min_size(at + vec2(0.0, 2.0), vec2(15.0, 15.0));
    p.rect_filled(b, 2.0, CHARCOAL);
    p.rect_stroke(b, 2.0, Stroke::new(1.0, if resp.hovered() { FOCUS } else { PEWTER }), StrokeKind::Inside);
    if *on {
        diamond(p, b.center(), 3.6, SAND, None);
    }
    p.galley(at + vec2(24.0, 10.0 - g.size().y / 2.0), g, TEXT);
    changed
}

/// The hourglass-gold progress bar: a dark track with a pewter edge, the sand filling it with a light at its head.
pub(super) fn sand_bar(p: &Painter, r: Rect, f: Option<f32>, t: f64) {
    p.rect_filled(r, r.height() / 2.0, CHARCOAL);
    p.rect_stroke(r, r.height() / 2.0, Stroke::new(1.0, PEWTER.gamma_multiply(0.8)), StrokeKind::Inside);
    let inner = r.shrink(2.0);
    match f {
        Some(f) => {
            let w = inner.width() * f.clamp(0.0, 1.0);
            if w > 1.0 {
                let fill = Rect::from_min_size(inner.min, vec2(w, inner.height()));
                p.add(super::rows::gradient(fill, Color32::from_rgb(0xb0, 0x84, 0x3c), SAND, true));
                // A slow shimmer along the sand.
                let x = fill.min.x + ((t as f32 * 0.35).fract() * (fill.width() + 80.0)) - 40.0;
                let band = Rect::from_min_max(
                    pos2((x - 30.0).max(fill.min.x), fill.min.y),
                    pos2((x + 30.0).min(fill.max.x), fill.max.y),
                );
                if band.width() > 0.0 {
                    p.rect_filled(band, inner.height() / 2.0, Color32::from_white_alpha(22));
                }
                let head = pos2(fill.max.x, fill.center().y);
                for i in 0..5 {
                    p.circle_filled(head, 2.0 + i as f32 * 2.2, SAND.gamma_multiply(0.10 - i as f32 * 0.018));
                }
            }
        }
        None => {
            // Not sized yet: a grain of light running along the track.
            let x = inner.min.x + ((t as f32 * 0.6).fract()) * inner.width();
            for i in 0..6 {
                p.circle_filled(
                    pos2(x, inner.center().y),
                    1.5 + i as f32 * 2.0,
                    SAND.gamma_multiply(0.16 - i as f32 * 0.025),
                );
            }
        }
    }
}

/// "1.2 GB" / "340 MB".
pub(super) fn bytes(n: u64) -> String {
    if n >= 1_000_000_000 { format!("{:.1} GB", n as f64 / 1e9) } else { format!("{:.0} MB", n as f64 / 1e6) }
}

/// "about 12 minutes left" from a rate.
fn eta(left: u64, rate: f64) -> String {
    if rate <= 1.0 {
        return String::new();
    }
    let s = left as f64 / rate;
    match s {
        s if s < 60.0 => "less than a minute left".into(),
        s if s < 5400.0 => format!("about {} minutes left", (s / 60.0).ceil() as u64),
        s => format!("about {:.1} hours left", s / 3600.0),
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Page {
    Welcome,
    Choose,
    Working,
    Done,
    Failed,
}

#[derive(Clone, Copy)]
enum Pick {
    Dir,
    Source,
}

/// What the app does after a frame of the wizard.
pub enum SetupAction {
    None,
    Quit,
    /// The install finished: the config gets the Timeless XI server, these game files and the Timeless interface.
    Configure {
        game_dir: PathBuf,
        manifest: Option<Manifest>,
    },
    /// Start the installed client and close.
    Play(PathBuf),
}

pub struct SetupUi {
    page: Page,
    sources: Vec<Source>,
    chosen: usize,
    dir: PathBuf,
    desktop: bool,
    start_menu: bool,
    play_now: bool,
    picked: Arc<Mutex<Option<(Pick, Option<PathBuf>)>>>,
    picking: bool,
    note: String,
    work: Option<Arc<Work>>,
    plan: Option<Plan>,
    error: String,
    /// Bytes a second, smoothed, and the last sample (time, bytes, phase).
    rate: f64,
    sample: Option<(f64, u64, String)>,
    fonts: bool,
    configured: bool,
}

impl SetupUi {
    pub fn new() -> Self {
        // Finding installs reads a few folders and the registry: quick, done once.
        let sources = setup::sources();
        Self {
            page: Page::Welcome,
            sources,
            chosen: 0,
            dir: setup::default_dir(),
            desktop: true,
            start_menu: true,
            play_now: true,
            picked: Default::default(),
            picking: false,
            note: String::new(),
            work: None,
            plan: None,
            error: String::new(),
            rate: 0.0,
            sample: None,
            fonts: false,
            configured: false,
        }
    }

    fn plan(&self) -> Plan {
        Plan {
            dir: self.dir.clone(),
            source: self.sources[self.chosen.min(self.sources.len() - 1)].clone(),
            desktop: self.desktop,
            start_menu: self.start_menu,
        }
    }

    fn pick(&mut self, what: Pick) {
        self.picking = true;
        let (start, out) = (self.dir.clone(), self.picked.clone());
        let title = match what {
            Pick::Dir => "Install Timeless XI in",
            Pick::Source => "Your FINAL FANTASY XI folder (or Square Enix's installer)",
        };
        std::thread::spawn(move || {
            let mut d = rfd::FileDialog::new().set_title(title);
            if let Some(dir) = start.ancestors().find(|d| d.is_dir()) {
                d = d.set_directory(dir);
            }
            *out.lock().unwrap() = Some((what, d.pick_folder()));
        });
    }

    fn begin(&mut self) {
        let plan = self.plan();
        if let Some(free) = setup::free_space(&plan.dir)
            && free < plan.source.space()
        {
            self.note = format!(
                "There isn't room on that drive: it has {} free and needs about {}. Free some space or choose another folder.",
                bytes(free),
                bytes(plan.source.space())
            );
            return;
        }
        self.note.clear();
        self.work = Some(setup::start(plan.clone()));
        self.plan = Some(plan);
        self.page = Page::Working;
        self.rate = 0.0;
        self.sample = None;
    }

    /// One frame of the wizard.
    pub fn ui(&mut self, ctx: &Context) -> SetupAction {
        if !self.fonts {
            super::theme::install_fonts_for(ctx, true);
            self.fonts = true;
        }
        let mut action = SetupAction::None;
        // The folder picker's answer.
        if let Some((what, answer)) = self.picked.lock().unwrap().take() {
            self.picking = false;
            if let Some(dir) = answer {
                match what {
                    Pick::Dir => {
                        // A folder picked for the client gets its own TimelessXI folder unless it is one.
                        self.dir = if dir
                            .file_name()
                            .is_some_and(|n| n.to_string_lossy().eq_ignore_ascii_case("TimelessXI"))
                        {
                            dir
                        } else {
                            dir.join("TimelessXI")
                        };
                    }
                    Pick::Source => match setup::source_at(&dir) {
                        Ok(s) => {
                            let at = self.sources.iter().position(|x| *x == s).unwrap_or_else(|| {
                                self.sources.insert(0, s);
                                0
                            });
                            self.chosen = at;
                            self.note.clear();
                        }
                        Err(e) => self.note = format!("{e:#}"),
                    },
                }
            }
        }
        // The install's end.
        if self.page == Page::Working
            && let Some(w) = &self.work
            && let Some(end) = w.end.lock().unwrap().take()
        {
            match end {
                Ok(manifest) => {
                    self.page = Page::Done;
                    if !self.configured {
                        self.configured = true;
                        let game_dir = self.plan.as_ref().map(Plan::game_dir).unwrap_or_default();
                        action = SetupAction::Configure { game_dir, manifest };
                    }
                }
                Err(e) => {
                    self.error = e;
                    self.page = Page::Failed;
                }
            }
        }
        let screen = ctx.content_rect();
        let t = ctx.input(|i| i.time);
        let bg = ctx.layer_painter(LayerId::new(Order::Background, Id::unique("tl setup backdrop")));
        backdrop(&bg, screen, t);
        // The logo on the left, over the steps.
        let left = Rect::from_min_size(screen.min, vec2(screen.width() * 0.38, screen.height()));
        let lw = (left.width() - 70.0).min(380.0);
        let logo = Rect::from_center_size(
            pos2(left.center().x, screen.min.y + 48.0 + lw / super::tltitle::LOGO_ASPECT / 2.0),
            vec2(lw, lw / super::tltitle::LOGO_ASPECT),
        );
        super::tltitle::hero_logo(ctx, &bg, logo, 1.0);
        let tag = font(ctx, body(11.5));
        spaced(
            &bg,
            pos2(logo.center().x, logo.max.y + 16.0),
            "A LEVEL 75 FINAL FANTASY XI PRIVATE SERVER",
            tag,
            FOCUS.gamma_multiply(0.9),
            3.0,
            true,
        );
        self.rail(ctx, &bg, pos2(left.min.x + 64.0, logo.max.y + 70.0));
        spaced(
            &bg,
            pos2(left.min.x + 64.0, screen.max.y - 34.0),
            &format!("TIMELESS XI SETUP  ·  {}", super::super::timeless::manifest::current()),
            font(ctx, body(10.5)),
            MUTED.gamma_multiply(0.6),
            1.5,
            false,
        );
        // The page on the right.
        let area =
            Rect::from_min_max(pos2(left.max.x, screen.min.y + 36.0), pos2(screen.max.x - 40.0, screen.max.y - 36.0));
        egui::Area::new(Id::unique("tl setup")).order(Order::Middle).fixed_pos(area.min).show(ctx, |ui| {
            ui.set_min_size(area.size());
            card(ui.painter(), area);
            let inner = area.shrink2(vec2(40.0, 34.0));
            let a = match self.page {
                Page::Welcome => self.welcome(ui, inner),
                Page::Choose => self.choose(ui, inner),
                Page::Working => self.working(ui, inner, t),
                Page::Done => self.done(ui, inner),
                Page::Failed => self.failed(ui, inner),
            };
            if !matches!(a, SetupAction::None) {
                action = a;
            }
        });
        if self.page == Page::Working {
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        }
        action
    }

    /// The four stages down the left: a gold diamond once done, a silver one for where we are.
    fn rail(&self, ctx: &Context, p: &Painter, at: Pos2) {
        let stage = match self.page {
            Page::Welcome => 0,
            Page::Choose => 1,
            Page::Working | Page::Failed => 2,
            Page::Done => 3,
        };
        let names = ["Welcome", "Game files", "Install", "Play"];
        for (i, name) in names.iter().enumerate() {
            let y = at.y + i as f32 * 40.0;
            let (fill, colour) = match i.cmp(&stage) {
                std::cmp::Ordering::Less => (SAND, TEXT.gamma_multiply(0.75)),
                std::cmp::Ordering::Equal => (SILVER, SILVER),
                std::cmp::Ordering::Greater => (CHARCOAL, MUTED.gamma_multiply(0.55)),
            };
            if i + 1 < names.len() {
                p.line_segment(
                    [pos2(at.x, y + 9.0), pos2(at.x, y + 31.0)],
                    Stroke::new(1.0, PEWTER.gamma_multiply(0.45)),
                );
            }
            diamond(p, pos2(at.x, y), 5.0, fill, Some(PEWTER));
            let num = ["I", "II", "III", "IV"][i];
            p.text(pos2(at.x + 20.0, y), Align2::LEFT_CENTER, num, font(ctx, serif(15.0)), colour.gamma_multiply(0.7));
            spaced(p, pos2(at.x + 52.0, y - 9.0), &name.to_uppercase(), font(ctx, serif(17.0)), colour, 2.2, false);
        }
    }

    fn title(ui: &Ui, inner: Rect, kicker: &str, title: &str) -> f32 {
        let ctx = ui.ctx().clone();
        let p = ui.painter();
        spaced(p, inner.min, kicker, font(&ctx, body(11.5)), FOCUS, 3.0, false);
        let h = para(p, inner.min + vec2(0.0, 22.0), title, font(&ctx, serif(34.0)), SILVER, inner.width());
        let y = inner.min.y + 30.0 + h;
        super::borders::fade_rule(p, inner.min.x, inner.max.x, y, PEWTER.gamma_multiply(0.7));
        y + 18.0
    }

    fn buttons(ui: &mut Ui, inner: Rect, back: Option<&str>, next: Option<(&str, bool)>) -> (bool, bool) {
        let h = 42.0;
        let y = inner.max.y - h;
        let mut out = (false, false);
        if let Some((label, enabled)) = next {
            let r = Rect::from_min_size(pos2(inner.max.x - 210.0, y), vec2(210.0, h));
            out.1 = button(ui, r, "next", label, Look::Primary, enabled);
        }
        if let Some(label) = back {
            let r = Rect::from_min_size(pos2(inner.max.x - 210.0 - 14.0 - 150.0, y), vec2(150.0, h));
            out.0 = button(ui, r, "back", label, Look::Secondary, true);
        }
        out
    }

    fn welcome(&mut self, ui: &mut Ui, inner: Rect) -> SetupAction {
        let y = Self::title(ui, inner, "WELCOME, ADVENTURER", "Your journey through Vana'diel begins here");
        let ctx = ui.ctx().clone();
        let p = ui.painter();
        let w = inner.width();
        let mut y = y;
        y += para(
            p,
            pos2(inner.min.x, y),
            "This sets up everything Timeless XI needs, start to finish. When it's done you go straight to the title \
             screen, ready to log in.",
            font(&ctx, body(16.5)),
            TEXT,
            w,
        ) + 22.0;
        let items = [
            ("The Timeless XI client", "Our own client, with Timeless View, hotbars and built-in fishing."),
            ("FINAL FANTASY XI's game files", "From Square Enix's free client, or the copy you already have."),
            ("Your connection to the server", "Set up for you. No PlayOnline, no launcher, no add-ons."),
        ];
        for (head, line) in items {
            diamond(p, pos2(inner.min.x + 6.0, y + 10.0), 4.5, SAND, Some(PEWTER));
            p.text(pos2(inner.min.x + 24.0, y), Align2::LEFT_TOP, head, font(&ctx, strong(16.0)), SILVER);
            y += 22.0;
            y += para(p, pos2(inner.min.x + 24.0, y), line, font(&ctx, body(14.5)), MUTED, w - 24.0) + 16.0;
        }
        let found = self.sources.iter().find(|s| !matches!(s, Source::Download));
        let note = match found {
            Some(Source::Installed { .. }) => {
                "FINAL FANTASY XI is already on this computer, so there's nothing big to download."
            }
            Some(Source::Installer(_)) => {
                "Square Enix's installer is already in your Downloads, so there's nothing big to download."
            }
            _ => "You'll need about 25 GB of free space and an internet connection for the game download.",
        };
        para(p, pos2(inner.min.x, y + 4.0), note, font(&ctx, body(14.5)), FOCUS.gamma_multiply(0.95), w);
        let enter = ui.ctx().input(|i| i.key_pressed(Key::Enter));
        let (quit, next) = Self::buttons(ui, inner, Some("Exit"), Some(("Begin", true)));
        if next || enter {
            self.page = Page::Choose;
        }
        if quit { SetupAction::Quit } else { SetupAction::None }
    }

    fn choose(&mut self, ui: &mut Ui, inner: Rect) -> SetupAction {
        let mut y = Self::title(ui, inner, "STEP II  ·  GAME FILES", "Where they come from");
        let ctx = ui.ctx().clone();
        let w = inner.width();
        for i in 0..self.sources.len() {
            let s = self.sources[i].clone();
            let r = Rect::from_min_size(pos2(inner.min.x, y), vec2(w, 52.0));
            let resp = ui.interact(r, Id::unique(("tl source", i)), Sense::click());
            if resp.hovered() {
                ctx.set_cursor_icon(CursorIcon::PointingHand);
            }
            if resp.clicked() {
                self.chosen = i;
                self.note.clear();
            }
            let on = self.chosen == i;
            let p = ui.painter();
            if on {
                p.add(super::rows::gradient(
                    r,
                    Color32::from_rgba_unmultiplied(0x8f, 0xb2, 0xe6, 40),
                    Color32::TRANSPARENT,
                    true,
                ));
                super::borders::fade_rule(p, r.min.x, r.max.x, r.min.y, SILVER.gamma_multiply(0.5));
                super::borders::fade_rule(p, r.min.x, r.max.x, r.max.y, SILVER.gamma_multiply(0.5));
            } else if resp.hovered() {
                p.add(super::rows::gradient(r, Color32::from_white_alpha(8), Color32::TRANSPARENT, true));
            }
            diamond(
                p,
                pos2(r.min.x + 14.0, r.center().y),
                5.5,
                if on { SAND } else { CHARCOAL },
                Some(if on { SILVER } else { PEWTER }),
            );
            p.text(
                pos2(r.min.x + 34.0, r.min.y + 9.0),
                Align2::LEFT_TOP,
                s.title(),
                font(&ctx, strong(16.0)),
                if on { SILVER } else { TEXT },
            );
            // One line: a long path is cut at the row's end rather than wrapping out of it.
            let detail = p.layout_no_wrap(s.detail(), font(&ctx, body(13.0)), MUTED);
            let clip = Rect::from_min_max(pos2(r.min.x + 34.0, r.min.y), pos2(r.max.x - 6.0, r.max.y));
            p.with_clip_rect(clip).galley(pos2(r.min.x + 34.0, r.min.y + 29.0), detail, MUTED);
            y += 56.0;
        }
        // Another folder.
        let other = Rect::from_min_size(pos2(inner.min.x + 34.0, y + 4.0), vec2(280.0, 20.0));
        let resp = ui.interact(other, Id::unique("tl other source"), Sense::click());
        if resp.hovered() {
            ctx.set_cursor_icon(CursorIcon::PointingHand);
        }
        ui.painter().text(
            other.min,
            Align2::LEFT_TOP,
            "Have it somewhere else? Choose the folder…",
            font(&ctx, body(13.5)),
            if resp.hovered() { SILVER } else { FOCUS },
        );
        if resp.clicked() && !self.picking {
            self.pick(Pick::Source);
        }
        y += 34.0;
        super::borders::fade_rule(ui.painter(), inner.min.x, inner.max.x, y, PEWTER.gamma_multiply(0.5));
        y += 14.0;
        // Where the client goes, and the shortcuts.
        let p = ui.painter();
        spaced(p, pos2(inner.min.x, y), "INSTALL TIMELESS XI IN", font(&ctx, body(11.0)), MUTED, 2.0, false);
        let path = p.layout(self.dir.display().to_string(), font(&ctx, body(14.0)), TEXT, w - 150.0);
        p.galley(pos2(inner.min.x, y + 20.0), path, TEXT);
        let change = Rect::from_min_size(pos2(inner.max.x - 120.0, y + 12.0), vec2(120.0, 30.0));
        if button(ui, change, "change dir", "Change", Look::Secondary, !self.picking) {
            self.pick(Pick::Dir);
        }
        y += 50.0;
        tick(ui, pos2(inner.min.x, y), "desktop", "Desktop shortcut", &mut self.desktop);
        tick(ui, pos2(inner.min.x + 200.0, y), "start", "Start menu shortcut", &mut self.start_menu);
        let plan = self.plan();
        let free = setup::free_space(&plan.dir);
        let need = plan.source.space();
        let line = match free {
            Some(f) if need > 0 => format!("Needs about {}  ·  {} free on that drive", bytes(need), bytes(f)),
            _ if need > 0 => format!("Needs about {} of free space", bytes(need)),
            _ => "No extra space needed for the game files".into(),
        };
        let short = free.is_some_and(|f| f < need);
        ui.painter().text(
            pos2(inner.min.x, y + 28.0),
            Align2::LEFT_TOP,
            line,
            font(&ctx, body(13.0)),
            if short { ALERT } else { MUTED },
        );
        if !self.note.is_empty() {
            para(ui.painter(), pos2(inner.min.x, y + 48.0), &self.note, font(&ctx, body(13.0)), ALERT, w - 390.0);
        }
        let enter = ctx.input(|i| i.key_pressed(Key::Enter));
        let esc = ctx.input(|i| i.key_pressed(Key::Escape));
        let (back, next) = Self::buttons(ui, inner, Some("Back"), Some(("Install", !self.picking)));
        if back || esc {
            self.page = Page::Welcome;
        } else if next || (enter && !self.picking) {
            self.begin();
        }
        SetupAction::None
    }

    fn working(&mut self, ui: &mut Ui, inner: Rect, t: f64) -> SetupAction {
        let mut y = Self::title(ui, inner, "STEP III", "Preparing your adventure");
        let ctx = ui.ctx().clone();
        let Some(work) = self.work.clone() else { return SetupAction::None };
        let step = work.step.load(Ordering::Relaxed);
        let p = ui.painter();
        for (i, name) in STEPS.iter().enumerate() {
            let c = pos2(inner.min.x + 8.0, y + 11.0);
            match i.cmp(&step) {
                std::cmp::Ordering::Less => {
                    diamond(p, c, 5.0, SAND, Some(PEWTER));
                    p.text(
                        pos2(c.x + 22.0, y),
                        Align2::LEFT_TOP,
                        *name,
                        font(&ctx, body(15.5)),
                        TEXT.gamma_multiply(0.8),
                    );
                }
                std::cmp::Ordering::Equal => {
                    super::rows::hourglass(p, c, 18.0, SILVER, SAND);
                    p.text(pos2(c.x + 22.0, y), Align2::LEFT_TOP, *name, font(&ctx, strong(15.5)), SILVER);
                }
                std::cmp::Ordering::Greater => {
                    diamond(p, c, 5.0, CHARCOAL, Some(PEWTER.gamma_multiply(0.6)));
                    p.text(
                        pos2(c.x + 22.0, y),
                        Align2::LEFT_TOP,
                        *name,
                        font(&ctx, body(15.5)),
                        MUTED.gamma_multiply(0.6),
                    );
                }
            }
            y += 34.0;
        }
        y += 14.0;
        let pr = &work.progress;
        let phase = pr.phase.lock().unwrap().clone();
        let (done, total) = (pr.done.load(Ordering::Relaxed), pr.total.load(Ordering::Relaxed));
        // The rate, per phase, smoothed.
        let now = t;
        match &self.sample {
            Some((t0, b0, ph)) if *ph == phase && now - t0 >= 1.0 => {
                let r = done.saturating_sub(*b0) as f64 / (now - t0);
                self.rate = if self.rate == 0.0 { r } else { self.rate * 0.8 + r * 0.2 };
                self.sample = Some((now, done, phase.clone()));
            }
            Some((_, _, ph)) if *ph == phase => {}
            _ => {
                self.rate = 0.0;
                self.sample = Some((now, done, phase.clone()));
            }
        }
        para(p, pos2(inner.min.x, y), &phase, font(&ctx, body(15.0)), TEXT, inner.width());
        y += 28.0;
        let bar = Rect::from_min_size(pos2(inner.min.x, y), vec2(inner.width(), 14.0));
        sand_bar(p, bar, (total > 0).then(|| done as f32 / total as f32), t);
        y += 24.0;
        if total > 0 {
            let left = eta(total.saturating_sub(done), self.rate);
            let line = if left.is_empty() {
                format!("{} of {}", bytes(done), bytes(total))
            } else {
                format!("{} of {}  ·  {left}", bytes(done), bytes(total))
            };
            p.text(pos2(inner.min.x, y), Align2::LEFT_TOP, line, font(&ctx, body(13.0)), MUTED);
        }
        y += 40.0;
        let tip = if phase.starts_with("Downloading") {
            "You can close this window: the download picks up where it left off when you run setup again."
        } else {
            "This can take a while on the first install. Grab a drink, it's nearly time."
        };
        para(p, pos2(inner.min.x, y), tip, font(&ctx, body(13.5)), MUTED.gamma_multiply(0.8), inner.width());
        SetupAction::None
    }

    fn done(&mut self, ui: &mut Ui, inner: Rect) -> SetupAction {
        let mut y = Self::title(ui, inner, "STEP IV", "Timeless XI is ready");
        let ctx = ui.ctx().clone();
        let p = ui.painter();
        y += para(
            p,
            pos2(inner.min.x, y),
            "Everything is in place. Log in with your Timeless XI account on the title screen, or choose Create Account \
             there to make one. Your adventure starts the moment you step in.",
            font(&ctx, body(16.0)),
            TEXT,
            inner.width(),
        ) + 22.0;
        if let Some(plan) = &self.plan {
            for (k, v) in [
                ("CLIENT", plan.exe().display().to_string()),
                ("GAME FILES", plan.game_dir().join("FINAL FANTASY XI").display().to_string()),
                ("SERVER", super::super::timeless::manifest::DEFAULT_HOST.to_string()),
            ] {
                spaced(p, pos2(inner.min.x, y), k, font(&ctx, body(11.0)), MUTED, 2.0, false);
                let h = para(
                    p,
                    pos2(inner.min.x + 110.0, y - 2.0),
                    &v,
                    font(&ctx, body(13.5)),
                    TEXT,
                    inner.width() - 110.0,
                );
                y += h.max(18.0) + 9.0;
            }
        }
        y += 12.0;
        tick(ui, pos2(inner.min.x, y), "play now", "Start Timeless XI now", &mut self.play_now);
        let enter = ctx.input(|i| i.key_pressed(Key::Enter));
        let label = if self.play_now { "Play" } else { "Finish" };
        let (_, next) = Self::buttons(ui, inner, None, Some((label, true)));
        if next || enter {
            return match (&self.plan, self.play_now) {
                (Some(plan), true) => SetupAction::Play(plan.exe()),
                _ => SetupAction::Quit,
            };
        }
        SetupAction::None
    }

    fn failed(&mut self, ui: &mut Ui, inner: Rect) -> SetupAction {
        let mut y = Self::title(ui, inner, "SOMETHING WENT WRONG", "Setup couldn't finish");
        let ctx = ui.ctx().clone();
        let p = ui.painter();
        y += para(
            p,
            pos2(inner.min.x, y),
            "Nothing is lost: what's already done is kept, and Try Again carries on from there.",
            font(&ctx, body(16.0)),
            TEXT,
            inner.width(),
        ) + 16.0;
        para(p, pos2(inner.min.x, y), &self.error, font(&ctx, body(13.5)), ALERT, inner.width());
        let (back, again) = Self::buttons(ui, inner, Some("Back"), Some(("Try Again", true)));
        if again {
            self.begin();
        } else if back {
            self.page = Page::Choose;
        }
        SetupAction::None
    }
}

/// `modern-shot --setup <page>`: a page with sample values (no folders read, nothing installed).
pub fn mock(page: &str) -> SetupUi {
    let sources = vec![
        Source::Installed { game_dir: PathBuf::from(r"C:\Program Files (x86)\PlayOnline\SquareEnix"), writable: false },
        Source::Installer(PathBuf::from(r"C:\Users\you\Downloads\FFXIFullSetup_US.part1.exe")),
        Source::Download,
    ];
    let mut s = SetupUi {
        page: match page {
            "choose" => Page::Choose,
            "working" => Page::Working,
            "done" => Page::Done,
            "failed" => Page::Failed,
            _ => Page::Welcome,
        },
        sources,
        chosen: 2,
        dir: PathBuf::from(r"C:\Users\you\AppData\Local\Programs\TimelessXI"),
        desktop: true,
        start_menu: true,
        play_now: true,
        picked: Default::default(),
        picking: false,
        note: String::new(),
        work: None,
        plan: None,
        error: "Downloading FFXIFullSetup_US.part3.rar: connection reset by peer".into(),
        rate: 9_500_000.0,
        sample: None,
        fonts: false,
        configured: true,
    };
    s.plan = Some(s.plan());
    if s.page == Page::Working {
        let w = Work::default();
        w.step.store(1, Ordering::Relaxed);
        *w.progress.phase.lock().unwrap() = "Downloading FINAL FANTASY XI from Square Enix (part 3 of 5)".into();
        w.progress.done.store(3_912_000_000, Ordering::Relaxed);
        w.progress.total.store(7_734_210_712, Ordering::Relaxed);
        s.sample = Some((1e9, 0, "Downloading FINAL FANTASY XI from Square Enix (part 3 of 5)".into()));
        s.work = Some(Arc::new(w));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_and_time_left_read_well() {
        assert_eq!(bytes(7_734_210_712), "7.7 GB");
        assert_eq!(bytes(340_000_000), "340 MB");
        assert_eq!(eta(30_000_000, 1_000_000.0), "less than a minute left");
        assert_eq!(eta(600_000_000, 1_000_000.0), "about 10 minutes left");
        assert_eq!(eta(7_200_000_000, 1_000_000.0), "about 2.0 hours left");
        assert_eq!(eta(1, 0.0), "");
    }
}
