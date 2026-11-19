//! Timeless XI: Timeless Angler on screen (the figures are `timeless/angler.rs`). Every interface: the HUD (a panel the
//! player can drag) and the school of fish banner at the top of the screen, in the Timeless palette. The layout is the
//! add-on's (timelessangler 1.39): zone and fishing pool, rod and bait, the fish they hook here with the chance per
//! cast, fishing skill, moon and bite time, the Fishing Spot Fatigue Tracker (with End school), the session, and the
//! Fish / Hide buttons.

use super::super::World;
use super::super::timeless::angler::{self, View};
use crate::inventory::ItemInfo;
use egui::{Align2, Color32, Context, CornerRadius, FontId, Id, Order, RichText, Stroke, pos2, vec2};

fn rgb(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

struct K;
impl K {
    fn text() -> Color32 {
        rgb(230, 236, 245)
    }
    fn muted() -> Color32 {
        rgb(152, 163, 184)
    }
    fn dim() -> Color32 {
        rgb(110, 120, 140)
    }
    fn gold() -> Color32 {
        rgb(227, 201, 142)
    }
    fn blue() -> Color32 {
        rgb(143, 178, 230)
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
        Color32::from_rgba_unmultiplied(10, 11, 18, 225)
    }
    fn edge() -> Color32 {
        rgb(40, 48, 64)
    }
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
    hide: bool,
    reset_session: bool,
    guide: bool,
}

const TABS: [&str; 7] = ["Current Zone", "Tracker", "Moon Effects", "Zones", "Fish", "Schools", "Settings"];

impl super::Modern {
    /// The Angler's HUD and the school banner, if they show.
    pub fn tl_angler(&self, ctx: &Context, w: &mut World) {
        let Some(v) = w.angler.view.clone() else { return };
        let asks = Asks::default();
        let mut asks = asks;
        self.tl_angler_banner(ctx, w, &v);
        let a = &w.angler;
        let school = a.hole.active;
        if a.prefs.hud && (!a.prefs.hud_auto || v.has_rod() || school || v.fishing) {
            self.tl_angler_hud(ctx, w, &v, &mut asks);
        }
        // The guide: its state and settings are copied out, drawn against the world, then put back.
        if w.angler.guide.open || asks.guide {
            let (mut g, mut prefs) = (w.angler.guide.clone(), w.angler.prefs.clone());
            if asks.guide {
                g.open = !g.open;
            }
            g.typing = false;
            if g.open {
                self.tl_angler_guide(ctx, w, &v, &mut g, &mut prefs, &mut asks);
            }
            let changed = prefs.hud != w.angler.prefs.hud
                || prefs.hud_auto != w.angler.prefs.hud_auto
                || prefs.banner != w.angler.prefs.banner;
            (w.angler.guide, w.angler.prefs) = (g, prefs);
            if changed {
                w.angler.save_prefs();
            }
        }
        if asks.end_school {
            angler::let_school_go(w);
        }
        if asks.reset_session {
            angler::reset_session(w);
        }
        if asks.hide {
            w.angler.prefs.hud = false;
            w.angler.save_prefs();
            w.chat.push_colour("Angler HUD hidden. Type /angler hud to bring it back.".into(), super::super::WHITE);
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
                egui::Frame::new()
                    .fill(K::panel())
                    .stroke(Stroke::new(1.0, K::edge()))
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(egui::Margin::symmetric(12, 10))
                    .show(ui, |ui| {
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
                        ui.label(RichText::new(head).font(FontId::proportional(15.0)).color(K::gold()).strong());
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
                                    text(ui, "Hook", K::muted());
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
                                    text(ui, &format!("+{} more", sp.fish.len() - 5), K::dim());
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
                            RichText::new(format!("{wait} seconds per bite"))
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
                            let (lo, hi, _) = angler::school_left(a.hole.school_casts);
                            ui.horizontal(|ui| {
                                text(
                                    ui,
                                    &format!(
                                        "School of fish! Cast #{}  -  {}",
                                        a.hole.school_casts,
                                        angler::casts_left(lo, hi)
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
                            text(ui, &format!("Cast #{} - No school remaining", a.hole.casts), K::muted());
                            text(ui, "Move 20+ yalms away for a fresh fishing spot.", K::muted());
                        } else if o.next >= angler::EXHAUSTED_AT {
                            text(ui, &format!("Cast #{} - this fishing spot is exhausted. Move on!", a.hole.casts), K::red());
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
                                    "{} fish   {:.0} catches/hr   {} gil",
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
                            if ui.small_button(if a.guide.open { "Close guide" } else { "Open guide" }).clicked() {
                                asks.guide = true;
                            }
                            if ui.small_button("Hide").on_hover_text("Hide this HUD. Bring it back with /angler hud").clicked() {
                                asks.hide = true;
                            }
                        });
                    });
            });
    }

    fn tl_angler_banner(&self, ctx: &Context, w: &World, v: &View) {
        let a = &w.angler;
        if !a.prefs.banner || (!a.hole.active && a.notice.is_none()) {
            return;
        }
        let top = pos2(ctx.content_rect().center().x, ctx.content_rect().top() + 80.0);
        egui::Area::new(Id::unique("tl angler banner"))
            .order(Order::Foreground)
            .interactable(false)
            .pivot(Align2::CENTER_TOP)
            .fixed_pos(top)
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(K::panel())
                    .stroke(Stroke::new(2.0, K::gold()))
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(egui::Margin::symmetric(16, 10))
                    .show(ui, |ui| {
                        let small = FontId::proportional(13.0);
                        if a.hole.active {
                            let t = ctx.input(|i| i.time) as f32;
                            let pulse = 0.75 + 0.25 * (t * 4.0).sin();
                            ui.label(
                                RichText::new("SCHOOL OF FISH!")
                                    .font(FontId::proportional(22.0))
                                    .strong()
                                    .color(K::gold().gamma_multiply(pulse)),
                            );
                            let (lo, hi, p) = angler::school_left(a.hole.school_casts);
                            let wait = angler::bite_wait(v.date.moon, v.date.hour, v.waist, true);
                            ui.label(RichText::new("+ Every cast hooks a fish").font(small.clone()).color(K::green()));
                            ui.label(
                                RichText::new(format!("+ Fish bite in half the time (about {wait} s)"))
                                    .font(small.clone())
                                    .color(K::green()),
                            );
                            ui.label(
                                RichText::new(format!(
                                    "School cast #{}  -  {}",
                                    a.hole.school_casts,
                                    angler::casts_left(lo, hi)
                                ))
                                .font(small.clone())
                                .color(K::text()),
                            );
                            // Pips: the cast that found it + up to 12 more; gold done, blue certain, faint maybe.
                            let (r, _) = ui.allocate_exact_size(vec2(13.0 * 18.0, 12.0), egui::Sense::hover());
                            for i in 0..13u32 {
                                let x = r.min.x + i as f32 * 18.0;
                                let c = if i < a.hole.school_casts {
                                    K::gold()
                                } else if i < 7 {
                                    K::blue()
                                } else {
                                    K::gold().gamma_multiply(0.3)
                                };
                                ui.painter().rect_filled(
                                    egui::Rect::from_min_size(pos2(x, r.min.y), vec2(14.0, 10.0)),
                                    CornerRadius::same(2),
                                    c,
                                );
                            }
                            ui.label(
                                RichText::new(format!("Chance your next cast is still in the school: {}", pct(p)))
                                    .font(small.clone())
                                    .color(K::muted()),
                            );
                            ui.label(
                                RichText::new(format!("Fish caught from this school: {}", a.hole.school_catches))
                                    .font(small.clone())
                                    .color(K::muted()),
                            );
                        } else if let Some((n, _)) = &a.notice {
                            ui.label(RichText::new(n).font(small.clone()).color(K::gold()));
                        }
                    });
            });
    }
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
        egui::Window::new(RichText::new("Timeless Angler").color(K::gold()))
            .id(Id::unique("tl angler guide"))
            .open(&mut open)
            .default_size(vec2(720.0, 480.0))
            .resizable(true)
            .collapsible(false)
            .frame(
                egui::Frame::new()
                    .fill(K::panel())
                    .stroke(Stroke::new(1.0, K::edge()))
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(egui::Margin::same(12)),
            )
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for (k, t) in TABS.iter().enumerate() {
                        let on = g.tab == k;
                        if ui
                            .selectable_label(on, RichText::new(*t).color(if on { K::gold() } else { K::text() }))
                            .clicked()
                        {
                            g.tab = k;
                        }
                    }
                });
                ui.separator();
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
        if !open {
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
            for h in ["Fish", "Lv", "Difficulty", "Skill-Up", "Per Catch"] {
                ui.label(RichText::new(h).color(K::muted()));
            }
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
                ui.label(f.lv.to_string());
                ui.label(RichText::new(format!("{label} ({dv})")).color(step_colour(step)));
                let up = if v.capped { 0.0 } else { angler::skillup_chance(f.lv, v.skill_raw, v.city) };
                ui.label(if up > 0.0 { pct(up) } else { "N/A".into() });
                let n = angler::multi_catch(d, id, v.skill(), v.gear, v.rod);
                ui.label(RichText::new(n.to_string()).color(if n > 1 { K::green() } else { K::text() }));
                ui.end_row();
            }
        });
    }

    fn tab_here(&self, ctx: &Context, ui: &mut egui::Ui, w: &World, v: &View, g: &mut angler::Guide) {
        let d = angler::data();
        let zone = d.zones.get(&v.zone).map_or_else(|| format!("Zone {}", v.zone), |z| z.n.clone());
        ui.label(RichText::new(&zone).size(18.0).color(K::gold()));
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
        ui.add_space(4.0);
        ui.label(
            RichText::new(format!(
                "Current Hook Ratio:  Fish {}   Items {}   Monsters {}{}",
                pct(fish),
                pct(o.item),
                pct(o.mob),
                if o.none > 0.0 { format!("   No bite {}", pct(o.none)) } else { String::new() }
            ))
            .color(K::blue()),
        );
        ui.add_space(6.0);
        ui.label(RichText::new("Available Targets").size(15.0).color(K::gold()));
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
        ui.label(RichText::new(format!("Fish caught: {}", s.fish)).size(18.0).color(K::gold()));
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
        ui.label(format!(
            "Casts {}   Catches {}   Lost {}   Schools found {}   Skill +{:.1}",
            s.casts, s.catches, s.lost, s.schools, s.skill
        ));
        let hrs = (t.max(60) as f32) / 3600.0;
        let mut by: Vec<(u16, u32)> = s.by.iter().map(|(&k, &n)| (k, n)).collect();
        by.sort_by(|a, b| b.1.cmp(&a.1));
        let mut total = 0u64;
        egui::Grid::new("tl tracker").num_columns(5).striped(true).spacing(vec2(14.0, 3.0)).show(ui, |ui| {
            for h in ["Catch", "Caught", "Per Hour", "NPC Sell", "Total Gil"] {
                ui.label(RichText::new(h).color(K::muted()));
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
                ui.label(n.to_string());
                ui.label(RichText::new(format!("{:.0}", n as f32 / hrs)).color(K::blue()));
                ui.label(if each > 0 { commas(each) } else { "-".into() });
                ui.label(RichText::new(commas(each * n as u64)).color(K::gold()));
                ui.end_row();
            }
        });
        ui.label(
            RichText::new(format!(
                "Total: {} gil  ({} gil per hour)",
                commas(total),
                commas((total as f32 / hrs) as u64)
            ))
            .color(K::gold()),
        );
    }

    fn tab_zones(&self, ctx: &Context, ui: &mut egui::Ui, w: &World, v: &View, g: &mut angler::Guide) {
        let d = angler::data();
        ui.horizontal(|ui| {
            ui.label(RichText::new("Fishing Zones").color(K::gold()));
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
            ui.label(RichText::new(self.name_of(w, id)).size(18.0).color(K::gold()));
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
                RichText::new(format!("Skill-up chance {} (a Perfect catch: 75%)", pct(up))).color(K::text())
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
        ui.label(RichText::new("Best Time").color(K::gold()));
        ui.label(format!(
            "{}   |   {}   |   {}",
            angler::pattern_text(&angler::HOUR_TEXT, f.hp),
            angler::pattern_text(&angler::MOON_TEXT, f.mp),
            angler::pattern_text(&angler::MONTH_TEXT, f.mo)
        ));
        ui.add_space(4.0);
        ui.label(RichText::new("Bait and Lures").color(K::gold()));
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
        ui.label(RichText::new("Where to Find It").color(K::gold()));
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
            .size(17.0)
            .color(K::gold()),
        );
    });
    ui.label(RichText::new(format!("Vana'diel time {:02}:00, month {}", dt.hour, dt.month)).color(K::muted()));
    ui.add_space(6.0);
    ui.label(RichText::new("Current Effects").color(K::gold()));
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
        for h in ["Effect", "Now", "Best At"] {
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
        ui.label("Skill-ups");
        ui.label("No effect");
        ui.label("-");
        ui.end_row();
    });
    ui.add_space(6.0);
    ui.label(RichText::new("Bite Timer").color(K::gold()));
    let wait = angler::bite_wait(dt.moon, dt.hour, v.waist, false);
    ui.label(format!("A fish bites about {wait} seconds after you cast."));
    let fast = dt.moon == angler::MOON_NEW || dt.moon == angler::MOON_FULL;
    for (on, t) in [
        (fast, "-4 s at a New or Full Moon"),
        (dt.hour == 5 || dt.hour == 17, "-1 s at the turn of the tide (05:00 / 17:00)"),
        (v.waist == angler::FISHERS_ROPE, "-1 s with a Fisher's Rope"),
        (false, "Half the wait in a school of fish (at least 4 s)"),
    ] {
        ui.label(RichText::new(t).color(if on { K::green() } else { K::muted() }));
    }
}

fn tab_schools(ui: &mut egui::Ui, w: &World, v: &View, asks: &mut Asks) {
    let a = &w.angler;
    let now = angler::now_secs();
    ui.label(RichText::new("Current Fishing Spot").size(16.0).color(K::gold()));
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
        ui.label(format!("School chance next cast: {}   Before the spot is exhausted: {}", pct(o.now), pct(o.ahead)));
    }
    ui.add_space(6.0);
    ui.label(RichText::new("School Odds").color(K::gold()));
    egui::Grid::new("tl school odds").num_columns(2).striped(true).spacing(vec2(16.0, 3.0)).show(ui, |ui| {
        ui.label(RichText::new("Casts").color(K::muted()));
        ui.label(RichText::new("Chance per Cast").color(K::muted()));
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
    ui.add_space(6.0);
    ui.label(RichText::new("How Schools of Fish Work").color(K::gold()));
    for t in [
        "Every cast counts toward the fishing spot where you made your first cast there.",
        "Casting 20+ yalms away, in another zone, or after 10 minutes without a cast starts a new spot.",
        "Each spot can have one school. It lasts the cast that finds it plus 6-12 more.",
        "In a school every cast hooks a fish, and fish bite in half the time.",
        "A school ends if you zone, move 20+ yalms off the spot, or don't cast for 5 minutes.",
        "After 75 casts the spot is exhausted: no schools, and only half your casts get a bite.",
    ] {
        ui.label(format!("- {t}"));
    }
    ui.add_space(6.0);
    ui.label(RichText::new("Pond Zones").color(K::gold()));
    ui.label("Yughott Grotto, Dragon's Aery, Middle / Upper / Lower Delkfutt's Tower, Temple of Uggalepih, Ranguemont Pass, Kuftal Tunnel, Gusgen Mines and Quicksand Caves: the whole zone is one fishing spot that never wears out. Every cast has a 5% school chance, and a new school can come 5 minutes after the last one ended.");
}

fn tab_settings(ui: &mut egui::Ui, prefs: &mut angler::Prefs) {
    ui.checkbox(&mut prefs.hud, "Show the HUD");
    ui.checkbox(&mut prefs.hud_auto, "Only with a rod in hand (or while fishing, or in a school)");
    ui.checkbox(&mut prefs.banner, "School of fish banner");
    ui.add_space(6.0);
    ui.label(RichText::new("Commands: /angler (this guide), /angler hud, /angler banner, /angler auto, /angler school end, /angler spot reset, /angler session reset. Drag the HUD by any empty part of it.").color(K::muted()));
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
    let f = pct as f32 / 100.0;
    let n = 24;
    let lit: Vec<egui::Pos2> = (0..=n)
        .map(|k| {
            let a = std::f32::consts::PI * (k as f32 / n as f32 - 0.5);
            pos2(c.x + r * a.cos(), c.y + r * a.sin())
        })
        .chain((0..=n).rev().map(|k| {
            let a = std::f32::consts::PI * (k as f32 / n as f32 - 0.5);
            pos2(c.x + r * (1.0 - 2.0 * f) * a.cos(), c.y + r * a.sin())
        }))
        .map(|q| if dir == 1 { pos2(2.0 * c.x - q.x, q.y) } else { q })
        .collect();
    if f > 0.02 {
        p.add(egui::Shape::Path(egui::epaint::PathShape {
            points: lit,
            closed: true,
            fill: K::gold(),
            stroke: egui::epaint::PathStroke::NONE,
        }));
    }
    p.circle_stroke(c, r, Stroke::new(1.0, K::muted()));
}
