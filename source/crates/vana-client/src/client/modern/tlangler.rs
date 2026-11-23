//! Timeless XI: Timeless Angler on screen (the figures are `timeless/angler.rs`). Every interface: the HUD (a panel the
//! player can drag) and the school of fish banner at the top of the screen, in the Timeless palette. The layout is the
//! add-on's (timelessangler 1.39): zone and fishing pool, rod and bait, the fish they hook here with the chance per
//! cast, fishing skill, moon and bite time, the Fishing Spot Fatigue Tracker (with End school), the session, and the
//! Fish / Hide buttons. Owner 2026-11-19 (HUD concepts canvas): the Almanac is an index (skill, time and moon, the
//! spot's state, the pages) beside the page, and the school banner is the "Ring" School of Fish window.

use super::super::World;
use super::super::timeless::angler::{self, View};
use crate::inventory::ItemInfo;
use egui::{Align2, Color32, Context, CornerRadius, FontId, Id, Order, RichText, Stroke, pos2, vec2};

fn rgb(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

use super::tlfish::{pewter, title_font};

struct K;
impl K {
    // Timeless XI: Classic / Modern keep style D (Dusk Remastered): cream text, lilac-grey quiet text, cream-gold,
    // lilac for the blue. Timeless View (Interface = Timeless, owner 2026-11-19) takes its Pewter Hairline palette
    // (assets/timeless/theme.json): pale text, pale silver for "gold" (its titles), steel blue, charcoal glass.
    fn text() -> Color32 {
        if pewter() { rgb(230, 236, 245) } else { rgb(246, 239, 224) }
    }
    fn muted() -> Color32 {
        if pewter() { rgb(163, 174, 195) } else { rgb(194, 182, 207) }
    }
    fn dim() -> Color32 {
        if pewter() { rgb(112, 122, 145) } else { rgb(138, 126, 160) }
    }
    fn gold() -> Color32 {
        if pewter() { rgb(214, 225, 240) } else { rgb(243, 223, 178) }
    }
    /// The accent gold: the Dusk gold, Timeless View's hourglass sand (theme "tp").
    fn sand() -> Color32 {
        if pewter() { rgb(240, 196, 110) } else { rgb(243, 223, 178) }
    }
    fn blue() -> Color32 {
        if pewter() { rgb(143, 178, 230) } else { rgb(169, 149, 214) }
    }
    fn green() -> Color32 {
        rgb(120, 200, 140)
    }
    fn amber() -> Color32 {
        rgb(232, 176, 74)
    }
    fn red() -> Color32 {
        rgb(224, 96, 108)
    }
    fn panel() -> Color32 {
        if pewter() {
            Color32::from_rgba_unmultiplied(22, 26, 38, 235)
        } else {
            Color32::from_rgba_unmultiplied(40, 32, 66, 230)
        }
    }
    fn edge() -> Color32 {
        if pewter() { rgb(107, 120, 146) } else { rgb(77, 66, 117) }
    }
    /// The darker well behind a ring's centre and the Almanac's index (the fishing bar's track colour).
    fn well() -> Color32 {
        if pewter() { rgb(11, 12, 19) } else { rgb(20, 22, 44) }
    }
}

/// A section heading (owner 2026-11-19, one style everywhere): the title colour at 15 px (Timeless View's serif in
/// Timeless) over a thin rule (fading out in Timeless View).
fn heading(ui: &mut egui::Ui, s: &str) {
    ui.add_space(4.0);
    ui.label(RichText::new(s).font(title_font(15.0)).color(K::gold()));
    let (r, _) = ui.allocate_exact_size(vec2(ui.available_width().min(520.0), 3.0), egui::Sense::hover());
    if pewter() {
        super::borders::fade_rule(ui.painter(), r.min.x, r.max.x, r.min.y + 1.0, K::gold().gamma_multiply(0.35));
    } else {
        ui.painter()
            .line_segment([pos2(r.min.x, r.min.y + 1.0), pos2(r.max.x, r.min.y + 1.0)], Stroke::new(1.0, K::edge()));
    }
}

/// A number right-aligned in its grid cell.
fn num(ui: &mut egui::Ui, t: RichText) {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        ui.label(t);
    });
}

/// A grid header right-aligned (over a number column).
fn num_head(ui: &mut egui::Ui, s: &str) {
    num(ui, RichText::new(s).color(K::muted()));
}

/// A big number with its label under it, in a sunken tile.
fn tile(ui: &mut egui::Ui, value: &str, label: &str, c: Color32) {
    egui::Frame::new()
        .fill(K::well())
        .stroke(Stroke::new(1.0, K::edge().gamma_multiply(0.7)))
        .corner_radius(CornerRadius::same(if pewter() { 2 } else { 6 }))
        .inner_margin(egui::Margin::symmetric(12, 6))
        .show(ui, |ui| {
            ui.set_min_width(78.0);
            ui.vertical(|ui| {
                ui.label(RichText::new(value).font(FontId::proportional(20.0)).strong().color(c));
                ui.label(RichText::new(label).font(FontId::proportional(12.0)).color(K::muted()));
            });
        });
}

/// A check (on) or a cross (off) drawn in front of a line (fonts may lack the glyphs).
fn mark(ui: &mut egui::Ui, on: bool) {
    let (r, _) = ui.allocate_exact_size(vec2(14.0, 14.0), egui::Sense::hover());
    let p = ui.painter();
    let c = r.center();
    if on {
        let st = Stroke::new(1.8, K::green());
        p.line_segment([pos2(c.x - 4.5, c.y), pos2(c.x - 1.5, c.y + 3.5)], st);
        p.line_segment([pos2(c.x - 1.5, c.y + 3.5), pos2(c.x + 4.5, c.y - 4.0)], st);
    } else {
        let st = Stroke::new(1.4, K::dim());
        p.line_segment([pos2(c.x - 3.5, c.y - 3.5), pos2(c.x + 3.5, c.y + 3.5)], st);
        p.line_segment([pos2(c.x - 3.5, c.y + 3.5), pos2(c.x + 3.5, c.y - 3.5)], st);
    }
}

/// The hook ratio as one bar in four colours (fish, items, monsters, no bite) with the percentages under it.
fn hook_bar(ui: &mut egui::Ui, fish: f32, item: f32, mob: f32, none: f32) {
    let parts = [
        ("Fish", fish, K::blue()),
        ("Items", item, K::sand()),
        ("Monsters", mob, K::red()),
        ("No bite", none, K::dim()),
    ];
    let total: f32 = parts.iter().map(|p| p.1).sum::<f32>().max(0.0001);
    let w = ui.available_width().min(420.0);
    let (r, _) = ui.allocate_exact_size(vec2(w, 10.0), egui::Sense::hover());
    let p = ui.painter();
    p.rect_filled(r, CornerRadius::same(2), K::well());
    let mut x = r.min.x;
    for &(_, v, c) in &parts {
        if v <= 0.0 {
            continue;
        }
        let x1 = x + r.width() * v / total;
        p.rect_filled(egui::Rect::from_min_max(pos2(x, r.min.y), pos2(x1, r.max.y)), CornerRadius::ZERO, c);
        x = x1;
    }
    p.rect_stroke(r, CornerRadius::same(2), Stroke::new(1.0, K::edge()), egui::StrokeKind::Inside);
    ui.horizontal_wrapped(|ui| {
        for &(n, v, c) in &parts {
            if n == "No bite" && v <= 0.0 {
                continue;
            }
            let (sw, _) = ui.allocate_exact_size(vec2(9.0, 9.0), egui::Sense::hover());
            ui.painter().rect_filled(sw, CornerRadius::same(1), c);
            ui.label(RichText::new(format!("{n} {}", pct(v))).color(K::text()));
            ui.add_space(8.0);
        }
    });
}

fn pct(x: f32) -> String {
    format!("{}%", (x * 100.0).round() as i32)
}

fn pct1(x: f32) -> String {
    let p = x * 100.0;
    if p > 0.0 && p < 10.0 { format!("{p:.1}%") } else { format!("{}%", p.round() as i32) }
}

fn commas(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// The difficulty step's colour (easy green .. very hard red).
fn step_colour(step: u8) -> Color32 {
    match step {
        1 => K::green(),
        2 => K::text(),
        3 => K::amber(),
        4 => rgb(232, 130, 80),
        _ => K::red(),
    }
}

/// What the buttons asked for (done after drawing, with the world mutable).
#[derive(Default)]
struct Asks {
    end_school: bool,
    fish: bool,
    minimize: bool,
    close: bool,
    restore: bool,
    reset_session: bool,
    guide: bool,
    /// Open the Almanac at this page (the HUD's "+N more").
    page: Option<usize>,
    /// The Almanac question's answer.
    answer: Option<bool>,
}

const TABS: [&str; 7] = ["Current Zone", "Tracker", "Moon & Time", "Zones", "Fish", "Schools", "Settings"];

/// Each page's icon in Timeless View's index (its Pewter menu rows).
const PAGE_ICON: [&str; 7] =
    ["icons/water", "icons/timer", "icons/light", "icons/map", "icons/find", "icons/favorites", "icons/settings"];

/// One line under each page's title (and the index entry's tooltip).
const PAGE_NOTE: [&str; 7] = [
    "What bites where you are standing, with your rod and bait",
    "Everything you have caught this session",
    "The moon, the time of day and how fast fish bite",
    "Every fishing zone and what lives there",
    "Look up any fish or bait",
    "Schools of fish and your current fishing spot",
    "Which fishing windows show",
];

impl super::Modern {
    /// The Angler's HUD and the school banner, if they show.
    pub fn tl_angler(&self, ctx: &Context, w: &mut World) {
        let Some(v) = w.angler.view.clone() else { return };
        self.set_pewter(w);
        let mut asks = Asks::default();
        self.tl_angler_banner(ctx, w, &v);
        if let Some(left) = w.angler.ask {
            self.tl_angler_ask(ctx, left, &mut asks);
        }
        let a = &w.angler;
        let school = a.hole.active;
        if a.prefs.hud && (!a.prefs.hud_auto || v.has_rod() || school || v.fishing) {
            if !a.prefs.minimized {
                self.tl_angler_hud(ctx, w, &v, &mut asks);
            } else if !w.modern {
                // Minimized, Classic: a tab on the right edge (the Modern and Timeless interfaces draw it as one of
                // their edge windows, extras.rs angler_tab).
                self.tl_angler_tab(ctx, &mut asks);
            }
        }
        // The guide: its state and settings are copied out, drawn against the world, then put back.
        if w.angler.guide.open || asks.guide || asks.page.is_some() {
            let (mut g, mut prefs) = (w.angler.guide.clone(), w.angler.prefs.clone());
            if asks.guide {
                g.open = !g.open;
            }
            if let Some(k) = asks.page {
                (g.open, g.tab) = (true, k);
            }
            g.typing = false;
            if g.open {
                self.tl_angler_guide(ctx, w, &v, &mut g, &mut prefs, &mut asks);
            }
            let changed = prefs.hud != w.angler.prefs.hud
                || prefs.hud_auto != w.angler.prefs.hud_auto
                || prefs.banner != w.angler.prefs.banner
                || prefs.ask != w.angler.prefs.ask;
            (w.angler.guide, w.angler.prefs) = (g, prefs);
            if changed {
                w.angler.save_prefs();
            }
        }
        if let Some(yes) = asks.answer {
            angler::answer(w, yes);
        }
        if asks.end_school {
            angler::let_school_go(w);
        }
        if asks.reset_session {
            angler::reset_session(w);
        }
        if asks.minimize || asks.restore {
            w.angler.prefs.minimized = asks.minimize;
            w.angler.save_prefs();
        }
        if asks.close {
            angler::close_all(w);
        }
        if asks.fish && !v.fishing {
            super::super::textcmd::run(w, "/fish", super::super::textcmd::Mode::Typed);
        }
    }

    fn name_of(&self, w: &World, id: u16) -> String {
        w.item_info
            .as_ref()
            .and_then(|i| i.name(id))
            .filter(|n| !n.is_empty())
            .or_else(|| angler::data().fish.get(&id).map(|f| f.n.clone()))
            .or_else(|| angler::data().bait.get(&id).map(|b| b.n.clone()))
            .or_else(|| angler::data().rods.get(&id).map(|r| r.n.clone()))
            .unwrap_or_else(|| format!("Item {id}"))
    }

    fn icon_of(&self, ctx: &Context, w: &World, id: u16) -> Option<egui::TextureId> {
        let info = w.item_info.as_ref()?;
        self.item_icon(ctx, info, id)
    }

    fn tl_angler_hud(&self, ctx: &Context, w: &World, v: &View, asks: &mut Asks) {
        let d = angler::data();
        let a = &w.angler;
        let school = a.hole.active;
        let now = angler::now_secs();
        egui::Area::new(Id::unique("tl angler hud"))
            .order(Order::Middle)
            .movable(true)
            .default_pos(pos2(ctx.content_rect().right() - 360.0, ctx.content_rect().top() + 120.0))
            .show(ctx, |ui| {
                let dusk = egui::Frame::new()
                    .fill(K::panel())
                    .stroke(Stroke::new(1.0, K::edge()))
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(egui::Margin::symmetric(12, 10));
                self.fish_frame(ui, dusk, egui::Margin { left: 14, right: 14, top: 12, bottom: 10 }, |ui| {
                        ui.set_max_width(340.0);
                        ui.spacing_mut().item_spacing = vec2(6.0, 4.0);
                        let small = FontId::proportional(13.0);
                        let text = |ui: &mut egui::Ui, s: &str, c: Color32| {
                            ui.label(RichText::new(s).font(small.clone()).color(c));
                        };
                        // 1. Zone and fishing pool.
                        let zone = d.zones.get(&v.zone).map_or_else(|| format!("Zone {}", v.zone), |z| z.n.clone());
                        let head = match &v.area {
                            Some(p) if p != &zone && !p.eq_ignore_ascii_case("Whole Zone") => format!("{zone}  -  {p}"),
                            _ => zone,
                        };
                        ui.label(RichText::new(head).font(title_font(15.0)).color(K::gold()).strong());
                        if !v.zone_known {
                            text(ui, "No fishing in this zone.", K::muted());
                        } else if v.area.is_none() {
                            text(ui, "Not in a fishing pool: walk to the water.", K::muted());
                        }
                        // 2. Rod and bait.
                        ui.horizontal(|ui| {
                            text(ui, "Rod:", K::muted());
                            let rod = if v.rod > 0 { self.name_of(w, v.rod) } else { "none".into() };
                            text(ui, &rod, if v.has_rod() { K::text() } else { K::red() });
                            text(ui, "  Bait:", K::muted());
                            let bait = if v.bait > 0 { self.name_of(w, v.bait) } else { "none".into() };
                            text(ui, &bait, if d.bait.contains_key(&v.bait) { K::text() } else { K::red() });
                        });
                        // 3. What this rod and bait hook here, the real chance per cast.
                        let exhausted = !a.hole.fresh(v.zone, v.x, v.z, now) && a.hole.casts >= angler::EXHAUSTED_AT - 1;
                        match &v.spot {
                            Some(sp) if !sp.fish.is_empty() => {
                                let o = angler::cast_odds(sp, school, exhausted);
                                ui.add_space(2.0);
                                egui::Grid::new("tl angler fish").num_columns(3).spacing(vec2(10.0, 2.0)).show(ui, |ui| {
                                    text(ui, if school { "Target Fish (School)" } else { "Target Fish" }, K::muted());
                                    text(ui, "Lv", K::muted());
                                    text(ui, "Chance", K::muted());
                                    ui.end_row();
                                    for e in sp.fish.iter().take(5) {
                                        let dv = angler::difficulty(d, e.id, v.skill(), v.gear, v.rod);
                                        let (label, step) = angler::difficulty_label(dv);
                                        let r = ui.horizontal(|ui| {
                                            if let Some(t) = self.icon_of(ctx, w, e.id) {
                                                ui.image((t, vec2(16.0, 16.0)));
                                            }
                                            text(ui, &self.name_of(w, e.id), step_colour(step));
                                        });
                                        r.response.on_hover_text(format!("Mini-game difficulty for you: {label}"));
                                        text(ui, &d.fish.get(&e.id).map_or(0, |f| f.lv).to_string(), K::text());
                                        text(ui, &pct1(o.fish.get(&e.id).copied().unwrap_or(0.0)), K::blue());
                                        ui.end_row();
                                    }
                                });
                                if sp.fish.len() > 5 {
                                    let more = ui
                                        .add(
                                            egui::Label::new(
                                                RichText::new(format!("+{} more", sp.fish.len() - 5))
                                                    .font(small.clone())
                                                    .color(K::blue())
                                                    .underline(),
                                            )
                                            .sense(egui::Sense::click()),
                                        )
                                        .on_hover_text("See them all in the Almanac.");
                                    if more.clicked() {
                                        asks.page = Some(0);
                                    }
                                }
                                if !school {
                                    let mut rest = format!("Items {}  Monsters {}", pct(o.item), pct(o.mob));
                                    if o.none > 0.0 {
                                        rest += &format!("  No bite {}", pct(o.none));
                                    }
                                    text(ui, &rest, K::dim());
                                }
                            }
                            Some(sp) if sp.nothing => text(ui, "Nothing here bites on this bait.", K::red()),
                            Some(_) if !d.bait.contains_key(&v.bait) => {
                                text(ui, "Equip bait or a lure to see what you can hook.", K::amber())
                            }
                            Some(_) => text(ui, "No fish here take this bait (only items or monsters).", K::amber()),
                            None => {}
                        }
                        // 4. Fishing skill.
                        ui.separator();
                        ui.horizontal(|ui| {
                            text(ui, &format!("Fishing skill {}", v.skill()), K::gold());
                            if v.gear > 0 {
                                text(ui, &format!("(+{} gear)", v.gear), K::green());
                            }
                        })
                        .response
                        .on_hover_text(format!(
                            "Your skill {} plus {} from equipped fishing gear = {}.\nGear counts twice in the mini-game.",
                            v.skill_raw,
                            v.gear,
                            v.skill()
                        ));
                        // 5. Moon and bite time.
                        let dt = v.date;
                        let wait = angler::bite_wait(dt.moon, dt.hour, v.waist, school);
                        ui.horizontal(|ui| {
                            let (r, _) = ui.allocate_exact_size(vec2(14.0, 14.0), egui::Sense::hover());
                            moon(ui.painter(), r.center(), 6.0, dt.moon_pct, dt.moon_dir);
                            text(ui, &format!("{} ({}%)", angler::MOON_NAMES[dt.moon as usize], dt.moon_pct), K::muted());
                        });
                        let fast = dt.moon == angler::MOON_NEW || dt.moon == angler::MOON_FULL;
                        ui.label(
                            RichText::new(format!("Bites in about {wait} s"))
                                .font(small.clone())
                                .color(if fast { K::green() } else { K::muted() }),
                        )
                        .on_hover_text(
                            "About how long a fish takes to bite after you cast. Normally 13 seconds; faster with a New or \
                             Full Moon, at 05:00 or 17:00, with a Fisher's Rope, or in a school of fish.",
                        );
                        // 6. Fishing Spot Fatigue Tracker.
                        ui.separator();
                        ui.label(RichText::new("Fishing Spot Fatigue Tracker").font(FontId::proportional(14.0)).color(K::gold()));
                        let o = a.hole.odds(v.zone, v.x, v.z, now);
                        if !o.pond {
                            meter(ui, &a.hole, o.fresh);
                        }
                        if school {
                            let (_, hi, _) = angler::school_left(a.hole.school_casts);
                            ui.horizontal(|ui| {
                                text(
                                    ui,
                                    &format!(
                                        "School of fish! Cast #{}  -  {}",
                                        a.hole.school_casts,
                                        angler::fish_left(hi)
                                    ),
                                    K::gold(),
                                );
                                if ui
                                    .small_button("End school")
                                    .on_hover_text("Let the school go now (to fish for items or monsters again).")
                                    .clicked()
                                {
                                    asks.end_school = true;
                                }
                            });
                        } else if o.pond {
                            text(ui, "Pond zone: this fishing spot never wears out.", K::blue());
                            if o.wait > 0 {
                                text(ui, &format!("Next school possible in {}:{:02}", o.wait / 60, o.wait % 60), K::muted());
                            } else {
                                text(ui, &format!("School Chance: {}", pct(o.now)), K::text());
                            }
                        } else if o.fresh {
                            ui.label(RichText::new(format!("School Chance: {}", pct(o.now))).font(small.clone()).color(K::blue()))
                                .on_hover_text("Your next cast starts a fresh fishing spot.");
                        } else if o.used {
                            text(ui, "This spot's school is used up.", K::muted());
                            text(ui, "Move 20+ yalms away for a chance to find another school!", K::muted());
                        } else if o.next >= angler::EXHAUSTED_AT {
                            text(
                                ui,
                                "Spot exhausted: fish bite half as often.",
                                K::amber(),
                            );
                            text(
                                ui,
                                "Move 20+ yalms to fresh waters for full bites and a chance at a school!",
                                K::muted(),
                            );
                        } else {
                            text(ui, &format!("Cast #{} at this fishing spot", a.hole.casts), K::text());
                            ui.label(RichText::new(format!("School Chance: {}", pct(o.now))).font(small.clone()).color(K::text()))
                                .on_hover_text(format!(
                                    "Chance your next cast finds a school. {} chance of one before this fishing spot is \
                                     exhausted.",
                                    pct(o.ahead)
                                ));
                        }
                        if a.hole.casts > 0 && a.hole.zone == v.zone && !o.fresh && !o.pond {
                            let dist = ((v.x - a.hole.x).powi(2) + (v.z - a.hole.z).powi(2)).sqrt();
                            if dist > 1.5 {
                                text(
                                    ui,
                                    &format!("{dist:.1} yalms from this fishing spot (a new one starts at 20)"),
                                    if dist > 16.0 { K::amber() } else { K::dim() },
                                );
                            }
                        }
                        // 7. The session, once a cast was made.
                        if let Some(first) = a.session.first {
                            ui.separator();
                            let t = now.saturating_sub(first);
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Session").font(FontId::proportional(14.0)).color(K::gold()));
                                text(ui, &format!("{}:{:02}:{:02}", t / 3600, t / 60 % 60, t % 60), K::muted());
                                if ui.small_button("Reset session").clicked() {
                                    asks.reset_session = true;
                                }
                            });
                            let hrs = (t.max(60) as f32) / 3600.0;
                            let mut by: Vec<(u16, u32)> = a.session.by.iter().map(|(&k, &n)| (k, n)).collect();
                            by.sort_by(|x, y| y.1.cmp(&x.1));
                            let gil: u64 = by.iter().map(|&(id, n)| n as u64 * *d.sell.get(&id).unwrap_or(&0) as u64).sum();
                            text(
                                ui,
                                &format!(
                                    "{} fish  \u{b7}  {:.0} catches/hr  \u{b7}  {} gil",
                                    a.session.fish,
                                    a.session.catches as f32 / hrs,
                                    commas(gil)
                                ),
                                K::text(),
                            );
                            for &(id, n) in by.iter().take(3) {
                                ui.horizontal(|ui| {
                                    if let Some(t) = self.icon_of(ctx, w, id) {
                                        ui.image((t, vec2(14.0, 14.0)));
                                    }
                                    text(ui, &format!("{} x{}", self.name_of(w, id), n), K::text());
                                    text(ui, &format!("{:.0}/hr", n as f32 / hrs), K::blue());
                                    let each = *d.sell.get(&id).unwrap_or(&0) as u64;
                                    if each > 0 {
                                        text(ui, &format!("{} g", commas(each * n as u64)), K::gold());
                                    }
                                });
                            }
                        }
                        // 8. Buttons.
                        ui.separator();
                        ui.horizontal(|ui| {
                            if ui
                                .small_button(if v.fishing { "Fishing..." } else { "Fish" })
                                .on_hover_text("Cast your line (same as typing /fish).")
                                .clicked()
                            {
                                asks.fish = true;
                            }
                            if ui
                                .small_button(if a.guide.open { "Close Almanac" } else { "Open Almanac" })
                                .on_hover_text("The Angler's Almanac: fish, bait, the moon and schools of fish (/almanac).")
                                .clicked()
                            {
                                asks.guide = true;
                            }
                            if ui
                                .small_button("Minimize")
                                .on_hover_text("Fold the HUD into a Fishing tab on the screen's edge; click the tab to open it.")
                                .clicked()
                            {
                                asks.minimize = true;
                            }
                            if ui
                                .small_button("Close")
                                .on_hover_text(
                                    "Close every fishing window (this HUD, the School of Fish window, the Almanac). Fishing notices then \
                                     show in the chat; /angler show brings the windows back.",
                                )
                                .clicked()
                            {
                                asks.close = true;
                            }
                        });
                    });
            });
    }

    /// The minimized HUD in the Classic interface: a gold-lettered tab on the right edge (the edge windows' look:
    /// the word down the tab, double chevrons pointing into the screen); a click opens the HUD again.
    fn tl_angler_tab(&self, ctx: &Context, asks: &mut Asks) {
        let screen = ctx.content_rect();
        let font = FontId::proportional(14.0);
        let galley = ctx.fonts_mut(|f| f.layout_no_wrap("Fishing".into(), font, Color32::WHITE));
        let size = vec2(24.0, galley.size().x + 28.0);
        let r = egui::Rect::from_min_size(pos2(screen.max.x - size.x, screen.top() + screen.height() * 0.45), size);
        egui::Area::new(Id::unique("tl angler tab")).order(Order::Middle).fixed_pos(r.min).show(ctx, |ui| {
            let (rr, resp) = ui.allocate_exact_size(size, egui::Sense::click());
            let p = ui.painter();
            p.rect(
                rr,
                CornerRadius { nw: 6, sw: 6, ne: 0, se: 0 },
                K::panel(),
                Stroke::new(1.0, K::gold().gamma_multiply(0.6)),
                egui::StrokeKind::Inside,
            );
            let c = if resp.hovered() { K::text() } else { K::gold() };
            let gw = galley.size().x;
            p.add(
                egui::epaint::TextShape::new(pos2(rr.max.x - 4.0, rr.center().y - gw / 2.0 + 6.0), galley, c)
                    .with_angle(std::f32::consts::FRAC_PI_2),
            );
            // «  (pointing into the screen) at the top.
            for d in [-3.0, 3.0] {
                let x = rr.center().x + d;
                let y = rr.min.y + 9.0;
                p.line_segment([pos2(x + 2.0, y - 3.0), pos2(x - 1.0, y)], Stroke::new(1.5, c));
                p.line_segment([pos2(x - 1.0, y), pos2(x + 2.0, y + 3.0)], Stroke::new(1.5, c));
            }
            if resp.on_hover_text("Fishing HUD: click to open it.").clicked() {
                asks.restore = true;
            }
        });
    }

    /// "Open the Angler's Almanac?" for a few seconds after the first cast of a fishing session: high on the screen,
    /// clear of the bar game; Enter / Yes opens it, Escape / No or nothing closes it; a bar shows the time left.
    fn tl_angler_ask(&self, ctx: &Context, left: f32, asks: &mut Asks) {
        let at = pos2(ctx.content_rect().center().x, ctx.content_rect().top() + ctx.content_rect().height() * 0.24);
        egui::Area::new(Id::unique("tl angler ask"))
            .order(Order::Foreground)
            .pivot(Align2::CENTER_TOP)
            .fixed_pos(at)
            .show(ctx, |ui| {
                let dusk = egui::Frame::new()
                    .fill(K::panel())
                    .stroke(Stroke::new(1.5, K::gold()))
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(egui::Margin::symmetric(16, 10));
                self.fish_frame(ui, dusk, egui::Margin { left: 16, right: 16, top: 14, bottom: 10 }, |ui| {
                    ui.set_width(300.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new("Open the Angler's Almanac?")
                                .font(title_font(16.0))
                                .strong()
                                .color(K::gold()),
                        );
                        ui.label(
                            RichText::new("What bites here, your odds, the moon and schools of fish.")
                                .size(12.5)
                                .color(K::muted()),
                        );
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            let w = 300.0;
                            ui.add_space((w - 2.0 * 120.0 - 12.0) / 2.0);
                            let yes = egui::Button::new(RichText::new("Yes  (Enter)").color(K::gold()))
                                .stroke(Stroke::new(1.5, K::gold()))
                                .min_size(vec2(120.0, 24.0));
                            if ui.add(yes).clicked() {
                                asks.answer = Some(true);
                            }
                            ui.add_space(12.0);
                            let no = egui::Button::new(RichText::new("No  (Esc)").color(K::text()))
                                .min_size(vec2(120.0, 24.0));
                            if ui.add(no).clicked() {
                                asks.answer = Some(false);
                            }
                        });
                        ui.add_space(6.0);
                        // The time left, draining.
                        let (r, _) = ui.allocate_exact_size(vec2(300.0, 3.0), egui::Sense::hover());
                        let f = (left / angler::ASK_SECONDS).clamp(0.0, 1.0);
                        ui.painter().rect_filled(r, CornerRadius::same(1), K::edge());
                        ui.painter().rect_filled(
                            egui::Rect::from_min_size(r.min, vec2(r.width() * f, r.height())),
                            CornerRadius::same(1),
                            K::sand().gamma_multiply(0.8),
                        );
                    });
                });
            });
        ctx.request_repaint();
    }

    /// The School of Fish window (owner 2026-11-19, the "Ring" mock). Running: a ring of 13 segments (the cast that
    /// found the school + up to 12 more: gold done, lilac certain, faint maybe) around "Cast #N", the title, "up to N
    /// fish remain", the two perks as green pills, then the chance the next cast is still in the school and the fish
    /// caught from it. Ended: a ring that runs down while the notice shows, around the fish caught (75 when the spot
    /// is exhausted), the title, the catch and the notice's lines.
    fn tl_angler_banner(&self, ctx: &Context, w: &World, v: &View) {
        let a = &w.angler;
        if !a.prefs.banner || (!a.hole.active && a.notice.is_none()) {
            return;
        }
        let top = pos2(ctx.content_rect().center().x, ctx.content_rect().top() + 80.0);
        let exhausted = !a.hole.active && a.notice.as_ref().is_some_and(|n| n.exhausted);
        let edge = if a.hole.active {
            K::gold()
        } else if exhausted {
            K::amber()
        } else {
            K::edge()
        };
        egui::Area::new(Id::unique("tl angler banner"))
            .order(Order::Foreground)
            .interactable(false)
            .pivot(Align2::CENTER_TOP)
            .fixed_pos(top)
            .show(ctx, |ui| {
                let dusk = egui::Frame::new()
                    .fill(K::panel())
                    .stroke(Stroke::new(2.0, edge))
                    .corner_radius(CornerRadius::same(18))
                    .inner_margin(egui::Margin::symmetric(22, 18));
                self.fish_frame(ui, dusk, egui::Margin::symmetric(22, 18), |ui| {
                    let small = FontId::proportional(13.0);
                    let tau = std::f32::consts::TAU;
                    let up = -std::f32::consts::FRAC_PI_2;
                    if a.hole.active {
                        ui.horizontal(|ui| {
                            let (r, _) = ui.allocate_exact_size(vec2(118.0, 118.0), egui::Sense::hover());
                            let p = ui.painter();
                            let c = r.center();
                            let seg = tau / 13.0;
                            for i in 0..13u32 {
                                let col = if i < a.hole.school_casts {
                                    K::sand()
                                } else if i < 7 {
                                    K::blue()
                                } else {
                                    K::edge()
                                };
                                let a0 = up + i as f32 * seg + 0.06;
                                super::tlfish::arc(p, c, 52.0, a0, a0 + seg - 0.12, Stroke::new(9.0, col));
                            }
                            p.circle_filled(c, 30.0, K::well());
                            p.circle_stroke(c, 30.0, Stroke::new(1.5, K::sand()));
                            p.text(
                                c - vec2(0.0, 9.0),
                                Align2::CENTER_CENTER,
                                "Cast",
                                FontId::proportional(11.0),
                                K::muted(),
                            );
                            p.text(
                                c + vec2(0.0, 8.0),
                                Align2::CENTER_CENTER,
                                format!("#{}", a.hole.school_casts),
                                FontId::proportional(20.0),
                                K::sand(),
                            );
                            ui.add_space(14.0);
                            ui.vertical(|ui| {
                                let t = ctx.input(|i| i.time) as f32;
                                let pulse = 0.75 + 0.25 * (t * 4.0).sin();
                                ui.label(
                                    RichText::new("School of Fish!")
                                        .font(title_font(24.0))
                                        .strong()
                                        .color(K::sand().gamma_multiply(pulse)),
                                );
                                let (_, hi, chance) = angler::school_left(a.hole.school_casts);
                                let wait = angler::bite_wait(v.date.moon, v.date.hour, v.waist, true);
                                ui.label(
                                    RichText::new(format!(
                                        "School cast #{}  -  {}",
                                        a.hole.school_casts,
                                        angler::fish_left(hi)
                                    ))
                                    .font(small.clone())
                                    .color(K::muted()),
                                );
                                ui.add_space(4.0);
                                ui.horizontal(|ui| {
                                    green_pill(ui, "Every cast hooks a fish");
                                    green_pill(ui, &format!("Fish bite in half the time (about {wait} s)"));
                                });
                                ui.add_space(4.0);
                                ui.horizontal(|ui| {
                                    stat(ui, &pct(chance), "Next cast still in the school");
                                    ui.add_space(18.0);
                                    stat(ui, &a.hole.school_catches.to_string(), "Fish caught from this school");
                                });
                            });
                        });
                    } else if let Some(n) = &a.notice {
                        ui.horizontal(|ui| {
                            let (r, _) = ui.allocate_exact_size(vec2(92.0, 92.0), egui::Sense::hover());
                            let p = ui.painter();
                            let c = r.center();
                            let now = angler::now_secs();
                            let f = if n.secs == 0 { 0.0 } else { n.until.saturating_sub(now) as f32 / n.secs as f32 };
                            p.circle_stroke(c, 40.0, Stroke::new(6.0, K::edge()));
                            super::tlfish::arc(
                                p,
                                c,
                                40.0,
                                up,
                                up + tau * f.clamp(0.0, 1.0),
                                Stroke::new(6.0, if n.exhausted { K::amber() } else { K::blue() }),
                            );
                            p.circle_filled(c, 27.0, K::well());
                            match n.caught {
                                Some((fish, _)) => {
                                    p.text(
                                        c - vec2(0.0, 6.0),
                                        Align2::CENTER_CENTER,
                                        fish.to_string(),
                                        FontId::proportional(20.0),
                                        K::sand(),
                                    );
                                    p.text(
                                        c + vec2(0.0, 11.0),
                                        Align2::CENTER_CENTER,
                                        "caught",
                                        FontId::proportional(11.0),
                                        K::muted(),
                                    );
                                }
                                None => {
                                    p.text(
                                        c - vec2(0.0, 6.0),
                                        Align2::CENTER_CENTER,
                                        "75",
                                        FontId::proportional(20.0),
                                        K::amber(),
                                    );
                                    p.text(
                                        c + vec2(0.0, 11.0),
                                        Align2::CENTER_CENTER,
                                        "casts",
                                        FontId::proportional(11.0),
                                        K::muted(),
                                    );
                                }
                            }
                            ui.add_space(12.0);
                            ui.vertical(|ui| {
                                ui.label(RichText::new(&n.title).font(title_font(19.0)).strong().color(K::text()));
                                if let Some((fish, casts)) = n.caught {
                                    ui.label(
                                        RichText::new(format!("You caught {fish} fish in {casts} casts."))
                                            .font(FontId::proportional(14.0))
                                            .strong()
                                            .color(K::gold()),
                                    );
                                }
                                for l in &n.lines {
                                    ui.label(RichText::new(l).font(small.clone()).color(K::muted()));
                                }
                            });
                        });
                    }
                });
            });
    }
}

/// A green pill (a perk of the school of fish).
fn green_pill(ui: &mut egui::Ui, s: &str) {
    egui::Frame::new()
        .fill(Color32::from_rgb(28, 48, 40))
        .stroke(Stroke::new(1.0, K::green()))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(egui::Margin::symmetric(12, 4))
        .show(ui, |ui| {
            ui.label(RichText::new(s).font(FontId::proportional(12.0)).strong().color(K::green()));
        });
}

/// A big number with its label under it.
fn stat(ui: &mut egui::Ui, value: &str, label: &str) {
    ui.vertical(|ui| {
        ui.label(RichText::new(value).font(FontId::proportional(20.0)).strong().color(K::text()));
        ui.label(RichText::new(label).font(FontId::proportional(12.0)).color(K::muted()));
    });
}

impl super::Modern {
    fn tl_angler_guide(
        &self,
        ctx: &Context,
        w: &World,
        v: &View,
        g: &mut angler::Guide,
        prefs: &mut angler::Prefs,
        asks: &mut Asks,
    ) {
        let mut open = true;
        let tl = pewter();
        // Timeless View: no egui title bar; its own title row (spaced serif capitals over a fading rule, a close mark)
        // and the Glass frame drawn behind the window once its size is known.
        let margin =
            if tl { egui::Margin { left: 16, right: 16, top: 12, bottom: 14 } } else { egui::Margin::same(12) };
        let frame = if tl {
            egui::Frame::NONE.inner_margin(margin)
        } else {
            egui::Frame::new()
                .fill(K::panel())
                .stroke(Stroke::new(1.0, K::edge()))
                .corner_radius(CornerRadius::same(8))
                .inner_margin(margin)
        };
        let mut window = egui::Window::new(RichText::new("Angler's Almanac").color(K::gold()))
            .id(Id::unique("tl angler guide"))
            .default_size(vec2(900.0, 560.0))
            .resizable(true)
            .collapsible(false)
            .frame(frame);
        window = if tl { window.title_bar(false) } else { window.open(&mut open) };
        let (mut bg, mut closed) = (None, false);
        let shown = window.show(ctx, |ui| {
            if tl {
                bg = Some(ctx.layer_painter(ui.layer_id()).add(egui::Shape::Noop));
                let top = ui.max_rect();
                // The close mark at the top right.
                let x = egui::Rect::from_center_size(pos2(top.max.x - 8.0, top.min.y + 14.0), vec2(20.0, 20.0));
                let resp = ui.interact(x, Id::unique("tl angler guide close"), egui::Sense::click());
                let c = if resp.hovered() { K::text() } else { K::muted() };
                let (a, b) = (x.shrink(5.0).min, x.shrink(5.0).max);
                ui.painter().line_segment([a, b], Stroke::new(1.4, c));
                ui.painter().line_segment([pos2(a.x, b.y), pos2(b.x, a.y)], Stroke::new(1.4, c));
                if resp.on_hover_text("Close the Almanac").clicked() {
                    closed = true;
                }
                super::title_row(
                    ui,
                    self,
                    &super::Title { icon: None, text: "Angler's Almanac", note: None, name: false },
                );
                super::title_rule(ui, self, margin);
            }
            // Timeless XI (owner 2026-11-19, the "Field Journal" layout): an index down the left with the zone, your
            // skill, the time and moon, and the fishing spot's state, then one entry per page; the page on the right
            // under its title and a one-line note.
            let full = ui.available_height();
            ui.horizontal_top(|ui| {
                egui::Frame::new()
                    .fill(if tl { K::well().gamma_multiply(0.6) } else { K::well() })
                    .stroke(Stroke::new(1.0, if tl { K::edge().gamma_multiply(0.6) } else { K::edge() }))
                    .corner_radius(CornerRadius::same(if tl { 2 } else { 6 }))
                    .inner_margin(egui::Margin::symmetric(if tl { 6 } else { 12 }, 12))
                    .show(ui, |ui| {
                        // The frame inherits the row's layout: stack the index top to bottom.
                        ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                            ui.set_width(if tl { 200.0 } else { 186.0 });
                            ui.set_min_height((full - 26.0).max(120.0));
                            ui.spacing_mut().item_spacing = vec2(6.0, 4.0);
                            let d = angler::data();
                            let zone = d.zones.get(&v.zone).map_or_else(|| format!("Zone {}", v.zone), |z| z.n.clone());
                            ui.add(
                                egui::Label::new(RichText::new(zone).font(title_font(17.0)).strong().color(K::gold()))
                                    .wrap(),
                            );
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(format!("Fishing skill {}", v.skill()))
                                        .font(FontId::proportional(13.0))
                                        .color(K::text()),
                                );
                                if v.gear > 0 {
                                    ui.label(
                                        RichText::new(format!("(+{} gear)", v.gear))
                                            .font(FontId::proportional(13.0))
                                            .color(K::green()),
                                    );
                                }
                            });
                            ui.label(
                                RichText::new(format!(
                                    "{:02}:00  {} ({}%)",
                                    v.date.hour,
                                    angler::MOON_NAMES[(v.date.moon as usize).min(angler::MOON_NAMES.len() - 1)],
                                    v.date.moon_pct
                                ))
                                .font(FontId::proportional(13.0))
                                .color(K::muted()),
                            );
                            let h = &w.angler.hole;
                            ui.label(
                                RichText::new(if h.active {
                                    "School of Fish Active!".to_string()
                                } else {
                                    format!("Cast #{}", h.casts)
                                })
                                .font(FontId::proportional(13.0))
                                .color(if h.active { K::sand() } else { K::muted() }),
                            );
                            ui.add_space(4.0);
                            if tl {
                                let (r, _) =
                                    ui.allocate_exact_size(vec2(ui.available_width(), 3.0), egui::Sense::hover());
                                super::borders::fade_rule(
                                    ui.painter(),
                                    r.min.x,
                                    r.max.x,
                                    r.center().y,
                                    K::gold().gamma_multiply(0.35),
                                );
                            } else {
                                ui.separator();
                            }
                            ui.add_space(2.0);
                            for (k, t) in TABS.iter().enumerate() {
                                let on = g.tab == k;
                                let r = if tl {
                                    // Timeless View's menu rows: the steel-blue band and the hourglass on the page
                                    // that is open, the page's Pewter icon, the serif.
                                    super::rows::menu_row(ui, self, PAGE_ICON[k], t, on, false, false, false, false)
                                } else {
                                    ui.add_sized(
                                        [ui.available_width(), 26.0],
                                        egui::Button::selectable(
                                            on,
                                            RichText::new(*t).font(FontId::proportional(14.0)).color(if on {
                                                K::gold()
                                            } else {
                                                K::text()
                                            }),
                                        ),
                                    )
                                };
                                if r.on_hover_text(PAGE_NOTE[k]).clicked() {
                                    g.tab = k;
                                }
                            }
                        });
                    });
                ui.add_space(if tl { 12.0 } else { 8.0 });
                ui.vertical(|ui| {
                    let k = g.tab.min(TABS.len() - 1);
                    ui.label(RichText::new(TABS[k]).font(title_font(22.0)).strong().color(K::gold()));
                    ui.label(RichText::new(PAGE_NOTE[k]).font(FontId::proportional(13.0)).color(K::dim()));
                    if tl {
                        let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 5.0), egui::Sense::hover());
                        super::borders::fade_rule(
                            ui.painter(),
                            r.min.x,
                            r.max.x,
                            r.center().y,
                            K::gold().gamma_multiply(0.35),
                        );
                    } else {
                        ui.separator();
                    }
                    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| match g.tab {
                        0 => self.tab_here(ctx, ui, w, v, g),
                        1 => self.tab_tracker(ctx, ui, w, asks),
                        2 => tab_moon(ui, v),
                        3 => self.tab_zones(ctx, ui, w, v, g),
                        4 => self.tab_fish(ctx, ui, w, v, g),
                        5 => tab_schools(ui, w, v, asks),
                        _ => tab_settings(ui, prefs),
                    });
                });
            });
        });
        if let (Some(r), Some(bg)) = (shown, bg) {
            ctx.layer_painter(r.response.layer_id)
                .set(bg, super::borders::window(self, "window_normal", r.response.rect));
        }
        if !open || closed {
            g.open = false;
        }
    }

    fn fish_rows(
        &self,
        ctx: &Context,
        ui: &mut egui::Ui,
        w: &World,
        v: &View,
        list: &[u16],
        salt: &str,
        g: &mut angler::Guide,
    ) {
        let d = angler::data();
        egui::Grid::new(salt).num_columns(5).striped(true).spacing(vec2(12.0, 3.0)).show(ui, |ui| {
            ui.label(RichText::new("Fish").color(K::muted()));
            num_head(ui, "Lv");
            ui.label(RichText::new("Difficulty").color(K::muted()));
            num_head(ui, "Skill-up");
            num_head(ui, "Fish per catch");
            ui.end_row();
            for &id in list {
                let Some(f) = d.fish.get(&id) else { continue };
                let dv = angler::difficulty(d, id, v.skill(), v.gear, v.rod);
                let (label, step) = angler::difficulty_label(dv);
                let r = ui.horizontal(|ui| {
                    if let Some(t) = self.icon_of(ctx, w, id) {
                        ui.image((t, vec2(16.0, 16.0)));
                    }
                    ui.add(
                        egui::Label::new(RichText::new(self.name_of(w, id)).color(K::text()))
                            .sense(egui::Sense::click()),
                    )
                });
                if r.inner.clicked() {
                    (g.fish, g.tab) = (Some(id), 4);
                }
                num(ui, RichText::new(f.lv.to_string()));
                ui.label(RichText::new(label).color(step_colour(step))).on_hover_text(format!(
                    "Mini-game difficulty {dv} (5 easiest, 100 hardest) at your skill and gear."
                ));
                let up = if v.capped { 0.0 } else { angler::skillup_chance(f.lv, v.skill_raw, v.city) };
                num(ui, RichText::new(if up > 0.0 { pct(up) } else { "\u{2013}".into() }));
                let n = angler::multi_catch(d, id, v.skill(), v.gear, v.rod);
                num(ui, RichText::new(n.to_string()).color(if n > 1 { K::green() } else { K::text() }));
                ui.end_row();
            }
        });
    }

    fn tab_here(&self, ctx: &Context, ui: &mut egui::Ui, w: &World, v: &View, g: &mut angler::Guide) {
        let d = angler::data();
        let zone = d.zones.get(&v.zone).map_or_else(|| format!("Zone {}", v.zone), |z| z.n.clone());
        ui.label(RichText::new(&zone).font(title_font(18.0)).color(K::gold()));
        if let Some(a) = &v.area {
            ui.label(RichText::new(format!("Fishing pool: {a}")).color(K::muted()));
        }
        ui.label(
            RichText::new(format!(
                "Fishing skill {} ({}{})  -  rank {}",
                v.skill(),
                v.skill_raw,
                if v.gear > 0 { format!(" +{} gear", v.gear) } else { String::new() },
                angler::RANKS.get(v.rank as usize).copied().unwrap_or("?")
            ))
            .color(K::text()),
        );
        let Some(sp) = &v.spot else {
            ui.label(
                RichText::new(if v.zone_known {
                    "Walk to the water to see what bites here."
                } else {
                    "No fishing in this zone."
                })
                .color(K::muted()),
            );
            return;
        };
        let a = &w.angler;
        let now = angler::now_secs();
        let exhausted = !a.hole.fresh(v.zone, v.x, v.z, now) && a.hole.casts >= angler::EXHAUSTED_AT - 1;
        let o = angler::cast_odds(sp, a.hole.active, exhausted);
        let fish: f32 = o.fish.values().sum();
        heading(ui, "Current Hook Ratio");
        hook_bar(ui, fish, o.item, o.mob, o.none);
        heading(ui, "Available Targets");
        ui.label(RichText::new("Current Gear Targets").color(K::dim()));
        let list: Vec<u16> = sp.fish.iter().map(|e| e.id).collect();
        if list.is_empty() {
            ui.label(RichText::new("No fish here take this bait.").color(K::amber()));
        } else {
            self.fish_rows(ctx, ui, w, v, &list, "tl here fish", g);
        }
        if !sp.other.is_empty() {
            ui.add_space(6.0);
            egui::CollapsingHeader::new(
                RichText::new(format!("Other Available Fish ({})", sp.other.len())).color(K::gold()),
            )
            .id_salt("tl here other")
            .show(ui, |ui| {
                let other = sp.other.clone();
                self.fish_rows(ctx, ui, w, v, &other, "tl here other fish", g);
            });
        }
        if !sp.items.is_empty() {
            egui::CollapsingHeader::new(RichText::new(format!("Items ({})", sp.items.len())).color(K::gold()))
                .id_salt("tl here items")
                .show(ui, |ui| {
                    for &id in &sp.items {
                        ui.horizontal(|ui| {
                            if let Some(t) = self.icon_of(ctx, w, id) {
                                ui.image((t, vec2(16.0, 16.0)));
                            }
                            ui.label(self.name_of(w, id));
                        });
                    }
                });
        }
    }

    fn tab_tracker(&self, ctx: &Context, ui: &mut egui::Ui, w: &World, asks: &mut Asks) {
        let d = angler::data();
        let s = &w.angler.session;
        let now = angler::now_secs();
        let Some(first) = s.first else {
            ui.label(RichText::new("The session starts with your first cast.").color(K::muted()));
            return;
        };
        let t = now.saturating_sub(first);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("Session time: {}:{:02}:{:02}", t / 3600, t / 60 % 60, t % 60)).color(K::muted()),
            );
            if ui.small_button("Reset session").clicked() {
                asks.reset_session = true;
            }
        });
        let hrs = (t.max(60) as f32) / 3600.0;
        // The session's numbers as tiles (owner 2026-11-19): fish landed (2-3 per catch from fish far below your
        // skill) apart from catches.
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            tile(ui, &s.fish.to_string(), "Fish landed", K::sand());
            tile(ui, &s.catches.to_string(), "Catches", K::text());
            tile(ui, &format!("{:.0}", s.catches as f32 / hrs), "Catches/hr", K::blue());
            tile(ui, &s.casts.to_string(), "Casts", K::text());
            tile(ui, &s.lost.to_string(), "Lost", if s.lost > 0 { K::amber() } else { K::text() });
            tile(ui, &s.schools.to_string(), "Schools found", K::text());
            tile(ui, &format!("+{:.1}", s.skill), "Skill", K::green());
        });
        heading(ui, "Catches");
        let mut by: Vec<(u16, u32)> = s.by.iter().map(|(&k, &n)| (k, n)).collect();
        by.sort_by(|a, b| b.1.cmp(&a.1));
        let mut total = 0u64;
        egui::Grid::new("tl tracker").num_columns(5).striped(true).spacing(vec2(14.0, 3.0)).show(ui, |ui| {
            ui.label(RichText::new("Catch").color(K::muted()));
            for h in ["Caught", "Per hour", "NPC sell", "Total gil"] {
                num_head(ui, h);
            }
            ui.end_row();
            for &(id, n) in &by {
                let each = *d.sell.get(&id).unwrap_or(&0) as u64;
                total += each * n as u64;
                ui.horizontal(|ui| {
                    if let Some(tx) = self.icon_of(ctx, w, id) {
                        ui.image((tx, vec2(16.0, 16.0)));
                    }
                    ui.label(self.name_of(w, id));
                });
                num(ui, RichText::new(n.to_string()));
                num(ui, RichText::new(format!("{:.0}", n as f32 / hrs)).color(K::blue()));
                num(ui, RichText::new(if each > 0 { commas(each) } else { "\u{2013}".into() }));
                num(ui, RichText::new(commas(each * n as u64)).color(K::sand()));
                ui.end_row();
            }
        });
        ui.label(
            RichText::new(format!(
                "Total: {} gil  \u{b7}  {} gil per hour",
                commas(total),
                commas((total as f32 / hrs) as u64)
            ))
            .color(K::sand()),
        );
    }

    fn tab_zones(&self, ctx: &Context, ui: &mut egui::Ui, w: &World, v: &View, g: &mut angler::Guide) {
        let d = angler::data();
        ui.horizontal(|ui| {
            ui.label(RichText::new("Fishing Zones").font(title_font(15.0)).color(K::gold()));
            g.typing |= ui
                .add(egui::TextEdit::singleline(&mut g.zone_filter).hint_text("filter").desired_width(180.0))
                .has_focus();
        });
        let q = g.zone_filter.to_lowercase();
        let mut ids: Vec<u16> =
            d.zones.keys().copied().filter(|z| q.is_empty() || d.zones[z].n.to_lowercase().contains(&q)).collect();
        ids.sort_by(|a, b| d.zones[a].n.cmp(&d.zones[b].n));
        let sel = g.zone.unwrap_or(v.zone);
        ui.columns(2, |cols| {
            egui::ScrollArea::vertical().id_salt("tl zones list").max_height(380.0).show(&mut cols[0], |ui| {
                for z in &ids {
                    let mut name = RichText::new(&d.zones[z].n);
                    if *z == v.zone {
                        name = name.color(K::gold());
                    }
                    if ui.selectable_label(sel == *z, name).clicked() {
                        g.zone = Some(*z);
                    }
                }
            });
            let ui = &mut cols[1];
            let Some(z) = d.zones.get(&sel) else {
                ui.label(RichText::new("Pick a zone.").color(K::muted()));
                return;
            };
            ui.label(
                RichText::new(format!("{}{}", z.n, if z.city { "  [town]" } else { "" })).size(16.0).color(K::gold()),
            );
            for (aid, a) in &z.areas {
                let mut list: Vec<u16> = d.groups.get(&a.g).map_or(Vec::new(), |gr| {
                    gr.iter().copied().filter(|f| d.fish.get(f).is_some_and(|f| f.it == 0)).collect()
                });
                list.sort_by_key(|f| d.fish[f].lv);
                let here = sel == v.zone && v.area.as_deref() == Some(a.n.as_str());
                egui::CollapsingHeader::new(format!(
                    "{}  ({} fish){}",
                    a.n,
                    list.len(),
                    if here { "  - you are here" } else { "" }
                ))
                .id_salt(("tl zone area", sel, *aid))
                .default_open(here || z.areas.len() == 1)
                .show(ui, |ui| {
                    for id in list {
                        let r = ui.horizontal(|ui| {
                            if let Some(t) = self.icon_of(ctx, w, id) {
                                ui.image((t, vec2(16.0, 16.0)));
                            }
                            ui.add(
                                egui::Label::new(format!("{}  Lv {}", self.name_of(w, id), d.fish[&id].lv))
                                    .sense(egui::Sense::click()),
                            )
                        });
                        if r.inner.clicked() {
                            (g.fish, g.tab) = (Some(id), 4);
                        }
                    }
                });
            }
            let mobs: Vec<&angler::Mob> = z.mobs.iter().filter(|m| m.qo == 0).collect();
            egui::CollapsingHeader::new(format!("Monsters ({})", mobs.len())).id_salt(("tl zone mobs", sel)).show(
                ui,
                |ui| {
                    if mobs.is_empty() {
                        ui.label(
                            RichText::new(if z.city { "No monsters in towns." } else { "No monsters to hook here." })
                                .color(K::dim()),
                        );
                    }
                    for m in mobs {
                        ui.label(
                            RichText::new(format!("{}  Lv {}{}", m.n, m.lv, if m.nm == 1 { "  (NM)" } else { "" }))
                                .color(if m.nm == 1 { K::amber() } else { K::red() }),
                        );
                    }
                },
            );
        });
    }

    fn tab_fish(&self, ctx: &Context, ui: &mut egui::Ui, w: &World, v: &View, g: &mut angler::Guide) {
        let d = angler::data();
        ui.horizontal(|ui| {
            g.typing |= ui
                .add(egui::TextEdit::singleline(&mut g.search).hint_text("fish or bait name").desired_width(220.0))
                .has_focus();
        });
        let q = g.search.to_lowercase();
        let mut list: Vec<u16> = d
            .fish
            .iter()
            .filter(|(id, f)| {
                let n = self.name_of(w, **id).to_lowercase();
                (f.it == 0 || !q.is_empty())
                    && (q.is_empty()
                        || n.contains(&q)
                        || (q.len() >= 3
                            && d.aff
                                .get(id)
                                .is_some_and(|a| a.keys().any(|b| self.name_of(w, *b).to_lowercase().contains(&q)))))
            })
            .map(|(id, _)| *id)
            .collect();
        list.sort_by_key(|id| self.name_of(w, *id));
        if g.fish.is_none_or(|f| !d.fish.contains_key(&f)) {
            g.fish = list.first().copied();
        }
        ui.columns(2, |cols| {
            egui::ScrollArea::vertical().id_salt("tl fish list").max_height(380.0).show(&mut cols[0], |ui| {
                ui.label(RichText::new(format!("{} results", list.len())).color(K::dim()));
                for id in &list {
                    let f = &d.fish[id];
                    let r = ui.horizontal(|ui| {
                        if let Some(t) = self.icon_of(ctx, w, *id) {
                            ui.image((t, vec2(16.0, 16.0)));
                        }
                        let c = if f.lv > v.skill() + 10 { K::muted() } else { K::text() };
                        ui.selectable_label(
                            g.fish == Some(*id),
                            RichText::new(format!(
                                "{}   {}",
                                self.name_of(w, *id),
                                if f.it == 1 { "item".to_string() } else { f.lv.to_string() }
                            ))
                            .color(c),
                        )
                    });
                    if r.inner.clicked() {
                        g.fish = Some(*id);
                    }
                }
            });
            if let Some(id) = g.fish {
                self.fish_detail(ctx, &mut cols[1], w, v, id);
            }
        });
    }

    fn fish_detail(&self, ctx: &Context, ui: &mut egui::Ui, w: &World, v: &View, id: u16) {
        let d = angler::data();
        let Some(f) = d.fish.get(&id) else { return };
        ui.horizontal(|ui| {
            if let Some(t) = self.icon_of(ctx, w, id) {
                ui.image((t, vec2(32.0, 32.0)));
            }
            ui.label(RichText::new(self.name_of(w, id)).font(title_font(18.0)).color(K::gold()));
        });
        let mut tags = vec![if f.sz == 1 { "Large" } else { "Small" }];
        if f.lg == 1 {
            tags.push("Legendary");
        }
        if f.it == 1 {
            tags.push("Item");
        }
        if f.ki > 0 {
            tags.push("Needs a key item");
        }
        if f.qo == 1 {
            tags.push("Quest only");
        }
        ui.label(RichText::new(format!("Lv {}   {}", f.lv, tags.join("  |  "))).color(K::muted()));
        if f.it == 0 {
            let dv = angler::difficulty(d, id, v.skill(), v.gear, v.rod);
            let (label, step) = angler::difficulty_label(dv);
            ui.label(RichText::new(format!("Difficulty for you: {label} ({dv})")).color(step_colour(step)));
            let n = angler::multi_catch(d, id, v.skill(), v.gear, v.rod);
            if n > 1 {
                ui.label(RichText::new(format!("{n} per catch at your skill")).color(K::green()));
            }
            let up = if v.capped { 0.0 } else { angler::skillup_chance(f.lv, v.skill_raw, v.city) };
            ui.label(if up > 0.0 {
                RichText::new(format!("Skill-up chance {} (a Perfect catch: 50%)", pct(up))).color(K::text())
            } else if v.capped {
                RichText::new("No skill-ups: your skill is at your guild rank's cap.").color(K::muted())
            } else if f.lv <= v.skill_raw {
                RichText::new(format!(
                    "No skill-ups (at or below your skill). A Perfect catch lands one extra {}.",
                    self.name_of(w, id)
                ))
                .color(K::muted())
            } else {
                RichText::new("No skill-ups yet: more than 50 levels above your skill.").color(K::muted())
            });
        }
        if let Some(sell) = d.sell.get(&id) {
            ui.label(RichText::new(format!("NPC sell: {} gil each", commas(*sell as u64))).color(K::gold()));
        }
        ui.add_space(4.0);
        heading(ui, "Best Time");
        ui.label(format!(
            "{}   |   {}   |   {}",
            angler::pattern_text(&angler::HOUR_TEXT, f.hp),
            angler::pattern_text(&angler::MOON_TEXT, f.mp),
            angler::pattern_text(&angler::MONTH_TEXT, f.mo)
        ));
        ui.add_space(4.0);
        heading(ui, "Bait and Lures");
        let mut baits: Vec<(u16, i32)> =
            d.aff.get(&id).map_or(Vec::new(), |a| a.iter().map(|(b, p)| (*b, *p)).collect());
        baits.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        if baits.is_empty() {
            ui.label(RichText::new("No bait hooks it.").color(K::muted()));
        }
        for (b, p) in baits {
            ui.horizontal(|ui| {
                if let Some(t) = self.icon_of(ctx, w, b) {
                    ui.image((t, vec2(16.0, 16.0)));
                }
                ui.label(RichText::new(self.name_of(w, b)).color(if b == v.bait { K::green() } else { K::text() }));
                ui.label(
                    RichText::new(match p {
                        3 => "best",
                        2 => "good",
                        _ => "works",
                    })
                    .color(K::muted()),
                );
            });
        }
        ui.add_space(4.0);
        heading(ui, "Where to Find It");
        for (zid, area) in angler::fish_where(d, id) {
            let z = &d.zones[&zid].n;
            ui.label(if area.eq_ignore_ascii_case("Whole Zone") || &area == z {
                z.clone()
            } else {
                format!("{z}  -  {area}")
            });
        }
    }
}

/// 1 -> "1st", 2 -> "2nd", 3 -> "3rd", 4 -> "4th" ...
fn ordinal(n: u32) -> String {
    let suffix = match (n % 10, n % 100) {
        (_, 11..=13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    };
    format!("{n}{suffix}")
}

fn tab_moon(ui: &mut egui::Ui, v: &View) {
    let dt = v.date;
    ui.horizontal(|ui| {
        let (r, _) = ui.allocate_exact_size(vec2(28.0, 28.0), egui::Sense::hover());
        moon(ui.painter(), r.center(), 13.0, dt.moon_pct, dt.moon_dir);
        ui.label(
            RichText::new(format!(
                "{} ({}%){}",
                angler::MOON_NAMES[dt.moon as usize],
                dt.moon_pct,
                match dt.moon_dir {
                    2 => ", waxing",
                    1 => ", waning",
                    _ => "",
                }
            ))
            .font(title_font(17.0))
            .color(K::gold()),
        );
    });
    ui.label(
        RichText::new(format!("Vana'diel {:02}:00  \u{b7}  {} month", dt.hour, ordinal(dt.month as u32)))
            .color(K::muted()),
    );
    heading(ui, "Current Effects");
    let (f, i, m) = angler::moon_pools(dt.moon);
    let lvl = |x: f32| {
        if x <= 0.33 {
            ("Low", K::muted())
        } else if x >= 0.67 {
            ("High", K::green())
        } else {
            ("Normal", K::text())
        }
    };
    egui::Grid::new("tl moon effects").num_columns(3).striped(true).spacing(vec2(16.0, 3.0)).show(ui, |ui| {
        for h in ["Effect", "Now", "Best at"] {
            ui.label(RichText::new(h).color(K::muted()));
        }
        ui.end_row();
        let fast = dt.moon == angler::MOON_NEW || dt.moon == angler::MOON_FULL;
        ui.label("Bite speed");
        ui.label(RichText::new(if fast { "Fast (-4 s)" } else { "Normal" }).color(if fast {
            K::green()
        } else {
            K::text()
        }));
        ui.label("New and Full Moon");
        ui.end_row();
        for (name, x, best) in [("Fish", f, "Full Moon"), ("Items", i, "Half moons"), ("Monsters", m, "New Moon")] {
            let (t, c) = lvl(x);
            ui.label(name);
            ui.label(RichText::new(t).color(c));
            ui.label(best);
            ui.end_row();
        }
    });
    heading(ui, "Bite Timer");
    let wait = angler::bite_wait(dt.moon, dt.hour, v.waist, false);
    ui.label(format!("A fish bites about {wait} seconds after you cast."));
    let fast = dt.moon == angler::MOON_NEW || dt.moon == angler::MOON_FULL;
    for (on, t) in [
        (fast, "-4 s at a New or Full Moon"),
        (dt.hour == 5 || dt.hour == 17, "-1 s at the turn of the tide (05:00 / 17:00)"),
        (v.waist == angler::FISHERS_ROPE, "-1 s with a Fisher's Rope"),
        (false, "Half the wait in a School of Fish (at least 4 s)"),
    ] {
        ui.horizontal(|ui| {
            mark(ui, on);
            ui.label(RichText::new(t).color(if on { K::green() } else { K::muted() }));
        });
    }
}

fn tab_schools(ui: &mut egui::Ui, w: &World, v: &View, asks: &mut Asks) {
    let a = &w.angler;
    let now = angler::now_secs();
    heading(ui, "Current Fishing Spot");
    let o = a.hole.odds(v.zone, v.x, v.z, now);
    if o.fresh {
        ui.label("Your next cast starts a fresh fishing spot.");
    } else {
        ui.label(format!(
            "Cast #{}{}",
            a.hole.casts,
            if a.hole.active { format!("  -  school cast #{}", a.hole.school_casts) } else { String::new() }
        ));
    }
    if !o.pond {
        meter(ui, &a.hole, o.fresh);
    }
    if a.hole.active {
        if ui.button("End school").clicked() {
            asks.end_school = true;
        }
    } else if !o.used {
        ui.label(format!("School chance next cast: {}", pct(o.now)));
        ui.label(format!("Chance of a school before this spot runs out: {}", pct(o.ahead)));
    }
    heading(ui, "School Odds");
    egui::Grid::new("tl school odds").num_columns(2).striped(true).spacing(vec2(16.0, 3.0)).show(ui, |ui| {
        ui.label(RichText::new("Casts").color(K::muted()));
        ui.label(RichText::new("Chance per cast").color(K::muted()));
        ui.end_row();
        for (c, p, col) in [
            ("1-15", "8%", K::green()),
            ("16-40", "5%", K::blue()),
            ("41-75", "3%", K::muted()),
            ("76+", "None - the fishing spot is exhausted", K::red()),
        ] {
            ui.label(c);
            ui.label(RichText::new(p).color(col));
            ui.end_row();
        }
    });
    heading(ui, "How Schools of Fish Work");
    for (key, rest) in [
        (
            "A fishing spot",
            " is where you first cast. It resets if you move 20+ yalms away, change zone, or go 10 minutes without \
             casting.",
        ),
        ("One school per spot.", " Each school can last up to 13 casts."),
        ("In a school", " every cast hooks a fish, and fish bite in half the time."),
        ("A school ends", " if you zone, move 20+ yalms away, or don't cast for 5 minutes."),
        ("After 75 casts", " the spot is exhausted: half the bites, and no schools."),
    ] {
        bullet(ui, key, rest);
    }
    heading(ui, "Pond Zones");
    ui.label(
        RichText::new(
            "In these zones the whole zone is one fishing spot that never wears out. Every cast has a 5% school \
             chance, and a new school can come 5 minutes after the last one ended.",
        )
        .color(K::muted()),
    );
    ui.add_space(2.0);
    for z in [
        "Yughott Grotto",
        "Dragon's Aery",
        "Middle Delkfutt's Tower",
        "Upper Delkfutt's Tower",
        "Lower Delkfutt's Tower",
        "Temple of Uggalepih",
        "Ranguemont Pass",
        "Kuftal Tunnel",
        "Gusgen Mines",
        "Quicksand Caves",
    ] {
        ui.label(format!("\u{2022}  {z}"));
    }
}

/// A rule with its key words in bold, after a bullet.
fn bullet(ui: &mut egui::Ui, key: &str, rest: &str) {
    let mut job = egui::text::LayoutJob::default();
    let body = egui::TextFormat { font_id: FontId::proportional(14.0), color: K::text(), ..Default::default() };
    job.append("\u{2022}  ", 0.0, egui::TextFormat { color: K::muted(), ..body.clone() });
    job.append(
        key,
        0.0,
        egui::TextFormat {
            color: K::gold(),
            font_id: FontId::new(14.0, egui::FontFamily::Name(super::theme::STRONG.into())),
            ..body.clone()
        },
    );
    job.append(rest, 0.0, body);
    job.wrap.max_width = ui.available_width();
    ui.label(job);
}

fn tab_settings(ui: &mut egui::Ui, prefs: &mut angler::Prefs) {
    heading(ui, "Windows");
    ui.checkbox(&mut prefs.hud, "Show the HUD");
    ui.checkbox(&mut prefs.hud_auto, "Only when I'm holding a rod")
        .on_hover_text("The HUD also shows while you are fishing and while a School of Fish is running.");
    ui.checkbox(&mut prefs.banner, "School of Fish window");
    ui.checkbox(&mut prefs.ask, "Offer the Almanac on my first cast (again after 5 minutes without fishing)");
    heading(ui, "Commands");
    egui::Grid::new("tl angler commands").num_columns(2).striped(true).spacing(vec2(16.0, 3.0)).show(ui, |ui| {
        for (c, what) in [
            ("/angler  or  /almanac", "Open or close this window"),
            ("/angler show", "Show every fishing window"),
            ("/angler close", "Close every fishing window (notices go to the chat)"),
            ("/angler minimize", "Fold the HUD into a tab on the screen's edge, or unfold it"),
            ("/angler hud", "Show or hide the HUD"),
            ("/angler banner", "Show or hide the School of Fish window"),
            ("/angler auto", "HUD only when holding a rod, on or off"),
            ("/angler ask", "Offer the Almanac on the first cast, on or off"),
            ("/angler school end", "Let the current School of Fish go"),
            ("/angler spot reset", "Reset the HUD's count for this fishing spot"),
            ("/angler session reset", "Start a new session"),
        ] {
            ui.label(RichText::new(c).color(K::gold()));
            ui.label(RichText::new(what).color(K::text()));
            ui.end_row();
        }
    });
    ui.add_space(4.0);
    ui.label(RichText::new("Drag the HUD by any empty part of it.").color(K::muted()));
    ui.label(RichText::new(format!("Fishing tables from the server, built {}.", angler::data().built)).color(K::dim()));
}

/// The spot's 75 casts as a bar: 1-15 (8 %), 16-40 (5 %), 41-75 (3 %), the casts so far filled in.
fn meter(ui: &mut egui::Ui, h: &angler::Hole, fresh: bool) {
    let w = ui.available_width().clamp(160.0, 320.0);
    let (r, resp) = ui.allocate_exact_size(vec2(w, 9.0), egui::Sense::hover());
    let p = ui.painter();
    let x = |c: f32| r.min.x + r.width() * (c / 75.0).min(1.0);
    for (a, b, col) in [(0.0, 15.0, K::green()), (15.0, 40.0, K::blue()), (40.0, 75.0, K::dim())] {
        p.rect_filled(
            egui::Rect::from_min_max(pos2(x(a), r.min.y), pos2(x(b) - 1.0, r.max.y)),
            CornerRadius::same(2),
            col.gamma_multiply(0.35),
        );
    }
    let casts = if fresh { 0.0 } else { h.casts as f32 };
    if casts > 0.0 {
        p.rect_filled(
            egui::Rect::from_min_max(r.min, pos2(x(casts), r.max.y)),
            CornerRadius::same(2),
            if h.casts >= angler::EXHAUSTED_AT { K::red() } else { K::gold() },
        );
    }
    resp.on_hover_text("Casts at this fishing spot. School chance per cast: 1-15 8%, 16-40 5%, 41-75 3%, then none.");
}

/// The moon as a disc lit from the right while waxing, the left while waning.
fn moon(p: &egui::Painter, c: egui::Pos2, r: f32, pct: i32, dir: i32) {
    p.circle_filled(c, r, rgb(40, 44, 60));
    // The lit part, row by row: from the terminator (an ellipse, x = w (1 - 2f)) to the limb (x = w), w the disc's
    // half width at that row; lit on the right while waxing, on the left while waning. A strip of quads between the
    // two edges, so a crescent (not convex) fills cleanly.
    let f = (pct as f32 / 100.0).clamp(0.0, 1.0);
    if f > 0.02 {
        let n = 24;
        let side = if dir == 1 { -1.0 } else { 1.0 };
        let mut mesh = egui::Mesh::default();
        for k in 0..=n {
            let y = -r + 2.0 * r * k as f32 / n as f32;
            let w = (r * r - y * y).max(0.0).sqrt();
            let (inner, outer) = (w * (1.0 - 2.0 * f), w);
            mesh.colored_vertex(pos2(c.x + side * inner, c.y + y), K::gold());
            mesh.colored_vertex(pos2(c.x + side * outer, c.y + y), K::gold());
            if k > 0 {
                let b = 2 * k as u32;
                mesh.add_triangle(b - 2, b - 1, b);
                mesh.add_triangle(b - 1, b + 1, b);
            }
        }
        p.add(egui::Shape::mesh(mesh));
    }
    p.circle_stroke(c, r, Stroke::new(1.0, K::muted()));
}
