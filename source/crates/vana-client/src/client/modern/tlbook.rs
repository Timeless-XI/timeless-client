//! Timeless XI: the Actions window (the Actions button over the Timeless hotbars; timeless/book.rs has the data).
//! Books as tabs: the Ability Book (Job Abilities by job with the level each comes at; Weapon Skills by weapon type
//! with the skill each needs), the Spell Book (by magic type; the tooltip names every job that casts it and at what
//! level), the Item Book (usable items are dragged from the Items list), the Misc Book (Wide Scan, Map, Region Info
//! ...) and Commands. Every tile drags onto a hotbar slot (modern/tlbar.rs `carry` lands it).

use super::super::World;
use super::super::timeless::book::{self, ABILITY_PAGES, BOOKS, BookState, JOBS, MAGIC, WEAPONS};
use super::super::timeless::hotbar::{self, Slot};
use super::tlbar::{glyph, icon, outlined, plate};
use super::{Panels, Title};
use egui::{Align2, Color32, CornerRadius, Pos2, Rect, RichText, Sense, Stroke, pos2, vec2};

const TILE: f32 = 40.0;
const CELL: egui::Vec2 = egui::Vec2::new(60.0, 62.0);

/// One tile of a page.
struct Tile {
    slot: Slot,
    icon: Option<(egui::TextureId, Rect)>,
    caption: String,
    /// Learned (abilities, weapon skills, spells); everything else is always lit.
    have: bool,
    title: String,
    body: String,
}

/// A chip of a page's selector (a job, a weapon type, a magic type): its label, picked, marked (our main or sub job).
struct Chip {
    label: String,
    on: bool,
    mark: bool,
}

#[derive(Default)]
struct Mouse {
    book: Option<usize>,
    page: Option<usize>,
    chip: Option<usize>,
    drag: Option<usize>,
    /// A tile clicked: onto the first empty slot.
    add: Option<usize>,
    hover: Option<(usize, Pos2)>,
    items: bool,
    close: bool,
}

pub fn draw(p: &mut Panels, w: &mut World) {
    let Some(mut st) = w.tl_bar.book else { return };
    if !hotbar::active(w) {
        return;
    }
    let (main, sub) = w.stats.as_ref().map_or((0, 0), |s| (s.main.0, s.sub.0));
    let jobs = book::ability_jobs();
    if !jobs.contains(&st.job) {
        st.job = if jobs.contains(&main) { main } else { jobs.first().copied().unwrap_or(1) };
    }
    st.book = st.book.min(BOOKS.len() - 1);
    let (chips, tiles) = page(p, w, &st, main, sub, &jobs);
    let m = p.m;
    let ctx = p.ctx.clone();
    let note = match (JOBS.get(main as usize), JOBS.get(sub as usize)) {
        (Some(a), Some(b)) if !a.is_empty() && !b.is_empty() => format!("{a} / {b}"),
        (Some(a), _) if !a.is_empty() => a.to_string(),
        _ => String::new(),
    };
    let mut mouse = Mouse::default();
    p.panel(
        "tl_actions",
        Some(Title {
            icon: Some("icons/abilities"),
            text: "Actions",
            note: (!note.is_empty()).then_some(note.as_str()),
            name: false,
        }),
        CELL.x * 10.0 + 24.0,
        |ui| {
            ui.horizontal_wrapped(|ui| {
                for (k, b) in BOOKS.iter().enumerate() {
                    if ui.selectable_label(k == st.book, RichText::new(*b).size(m.theme.ui)).clicked() {
                        mouse.book = Some(k);
                    }
                }
            });
            if st.book == 0 {
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    for (k, pg) in ABILITY_PAGES.iter().enumerate() {
                        let t = RichText::new(*pg).size(m.theme.small).color(m.theme.color("gold"));
                        if ui.selectable_label(k == st.page, t).clicked() {
                            mouse.page = Some(k);
                        }
                    }
                });
            }
            ui.separator();
            if !chips.is_empty() {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
                    for (k, c) in chips.iter().enumerate() {
                        let mut t = RichText::new(&c.label).size(m.theme.small);
                        if c.mark {
                            t = t.color(m.theme.color("tp"));
                        }
                        if ui.selectable_label(c.on, t).clicked() {
                            mouse.chip = Some(k);
                        }
                    }
                });
                ui.add_space(4.0);
            }
            match st.book {
                2 => item_book(ui, m, &mut mouse),
                _ => {
                    egui::ScrollArea::vertical()
                        .id_salt(("tl book", st.book, st.page, st.job, st.weapon, st.magic))
                        .max_height(CELL.y * 6.0)
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
                            if tiles.is_empty() {
                                let empty = if (st.book, st.page) == (0, 2) {
                                    "Summon an avatar to see its Blood Pacts."
                                } else {
                                    "Nothing here yet."
                                };
                                ui.label(RichText::new(empty).color(m.theme.color("muted")));
                            }
                            ui.horizontal_wrapped(|ui| {
                                ui.spacing_mut().item_spacing = vec2(2.0, 4.0);
                                for (k, t) in tiles.iter().enumerate() {
                                    let (r, resp) = ui.allocate_exact_size(CELL, Sense::click_and_drag());
                                    tile(ui.painter(), m, r, t, resp.hovered());
                                    if resp.drag_started() {
                                        mouse.drag = Some(k);
                                    }
                                    if resp.clicked() {
                                        mouse.add = Some(k);
                                    }
                                    if resp.hovered() {
                                        mouse.hover = Some((k, r.center_top()));
                                    }
                                }
                            });
                        });
                }
            }
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Click a tile to put it on the next empty slot, or drag it onto one. Lit tiles are ones you have now.")
                        .size(m.theme.small)
                        .color(m.theme.color("muted")),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Close").clicked() {
                        mouse.close = true;
                    }
                });
            });
        },
    );
    if let Some(k) = mouse.book {
        st.book = k;
        w.sounds.push(crate::audio::Ui::Cursor as u32);
    }
    if let Some(k) = mouse.page {
        st.page = k;
        w.sounds.push(crate::audio::Ui::Cursor as u32);
    }
    if let Some(k) = mouse.chip {
        match (st.book, st.page) {
            (0, 0) => st.job = jobs.get(k).copied().unwrap_or(st.job),
            (0, 2) => st.pact = k,
            (0, _) => st.weapon = k,
            (1, _) => st.magic = k,
            _ => {}
        }
        w.sounds.push(crate::audio::Ui::Cursor as u32);
    }
    if let Some(k) = mouse.drag
        && let Some(t) = tiles.get(k)
    {
        w.tl_bar.carry = Some(t.slot.clone());
    }
    if let Some(k) = mouse.add
        && let Some(t) = tiles.get(k)
    {
        add_next(w, t.slot.clone());
    }
    if let Some((k, at)) = mouse.hover
        && w.tl_bar.carry.is_none()
        && let Some(t) = tiles.get(k)
    {
        m.tip_at(&ctx, at, &t.title, &t.body);
    }
    if mouse.items {
        book::run_misc(w, "items");
    }
    w.tl_bar.book = if mouse.close { None } else { Some(st) };
    if mouse.close {
        w.sounds.push(crate::audio::Ui::Cancel as u32);
    }
}

/// A tile clicked (or an item row): onto the first empty slot, which glints. Locked or full: the error sound and a line.
pub(super) fn add_next(w: &mut World, s: Slot) {
    if w.tl_bar.locked {
        w.sounds.push(crate::audio::Ui::Error as u32);
        return;
    }
    match w.tl_bar.add_next(s) {
        Some((b, k)) => {
            w.tl_bar.ready_at[b][k] = Some(std::time::Instant::now());
            w.sounds.push(crate::audio::Ui::Select as u32);
        }
        None => w.sounds.push(crate::audio::Ui::Error as u32),
    }
}

/// The Item Book: how items get on the bars, and a button for the Items list.
fn item_book(ui: &mut egui::Ui, m: &super::Modern, mouse: &mut Mouse) {
    ui.add_space(8.0);
    ui.label(
        RichText::new("Items go on the hotbar straight from your inventory.")
            .size(m.theme.ui)
            .color(m.theme.color("text")),
    );
    ui.add_space(4.0);
    ui.label(
        RichText::new(
            "Open your Items list, then drag any usable item (food, medicine, scrolls, usable gear) onto a hotbar slot. \
             The slot shows how many you carry and uses one on yourself when pressed.",
        )
        .size(m.theme.small)
        .color(m.theme.color("muted")),
    );
    ui.add_space(8.0);
    if ui.button("Open Items").clicked() {
        mouse.items = true;
    }
    ui.add_space(8.0);
}

/// A tile: frame, icon (dimmed when not learned), caption under it.
fn tile(painter: &egui::Painter, m: &super::Modern, r: Rect, t: &Tile, hovered: bool) {
    let ir = Rect::from_min_size(pos2(r.center().x - TILE / 2.0, r.min.y + 2.0), vec2(TILE, TILE));
    painter.rect_filled(ir.expand(2.0), CornerRadius::same(6), m.theme.color("charcoal"));
    let tint = if t.have { Color32::WHITE } else { Color32::from_gray(95) };
    match t.icon {
        Some((tex, uv)) if plate(&t.slot).is_some() => {
            let c = m.theme.color(plate(&t.slot).unwrap_or("focus"));
            painter.rect_filled(ir, CornerRadius::same(4), c.gamma_multiply(0.4));
            painter.image(tex, ir.shrink(5.0), uv, tint);
        }
        Some((tex, uv)) => {
            painter.image(tex, ir, uv, tint);
        }
        None => glyph(m, painter, ir, &t.slot),
    }
    let edge =
        if hovered { Stroke::new(1.6, m.theme.color("focus")) } else { Stroke::new(1.0, m.theme.color("bronze")) };
    painter.rect_stroke(ir.expand(2.0), CornerRadius::same(6), edge, egui::StrokeKind::Inside);
    let col = if t.have { m.theme.color("text") } else { m.theme.color("muted") };
    let caption: String = t.caption.chars().take(10).collect();
    outlined(painter, pos2(r.center().x, ir.max.y + 4.0), Align2::CENTER_TOP, &caption, 11.0, col);
}

/// The page's selector chips and tiles.
fn page(p: &Panels, w: &World, st: &BookState, main: u8, sub: u8, jobs: &[u8]) -> (Vec<Chip>, Vec<Tile>) {
    let mut chips = Vec::new();
    let mut tiles = Vec::new();
    let named = |s: &Slot| hotbar::name(w, s);
    match (st.book, st.page) {
        (0, 0) => {
            for &j in jobs {
                chips.push(Chip { label: JOBS[j as usize].to_string(), on: j == st.job, mark: j == main || j == sub });
            }
            for a in book::job_abilities(st.job) {
                let s = Slot::Ability { id: a.id, to: None };
                let title = named(&s);
                if title.is_empty() {
                    continue;
                }
                let recast = crate::dat::recasts::ability(a.id).map(recast_text).unwrap_or_default();
                let body =
                    format!("{}{recast}\nClick to add, or drag onto a slot.", book::levels_text(&book::ability_levels(a.id)));
                tiles.push(Tile {
                    icon: icon(p, w, &s),
                    caption: format!("Lv. {}", a.level),
                    have: book::has_ability(w, a.id),
                    title,
                    body,
                    slot: s,
                });
            }
        }
        (0, 2) => {
            // The Blood Pacts of the avatar we have out (the game's lists hold only its pacts).
            let acts = w.actions.as_ref();
            for (k, ty) in book::PACTS.iter().enumerate() {
                // The list's name: the Pet Commands row that opens it ("Blood Pact: Rage").
                let label = acts
                    .and_then(|a| a.abilities.iter().position(|x| x.kind == 2 && x.sub as u16 == *ty))
                    .map(|i| super::super::magic::ability_name(w, i as u16))
                    .filter(|n| !n.is_empty())
                    .unwrap_or_else(|| if k == 0 { "Rage".into() } else { "Ward".into() });
                chips.push(Chip { label, on: k == st.pact, mark: false });
            }
            let ty = book::PACTS[st.pact.min(book::PACTS.len() - 1)];
            if let Some(a) = acts {
                for i in super::super::magic::ability_rows(w, ty) {
                    let Some(x) = a.abilities.get(i as usize) else { continue };
                    let s = Slot::Ability { id: x.id, to: None };
                    let title = named(&s);
                    if title.is_empty() {
                        continue;
                    }
                    let cost = if x.cost > 0 { format!("{} MP", x.cost) } else { String::new() };
                    tiles.push(Tile {
                        icon: icon(p, w, &s),
                        caption: cost.clone(),
                        have: hotbar::usable(w, &s) || hotbar::why(w, &s) != Some(hotbar::Why::Learn),
                        title,
                        body: format!("{cost}\nClick to add, or drag onto a slot."),
                        slot: s,
                    });
                }
            }
        }
        (0, _) => {
            for (k, (_, label)) in WEAPONS.iter().enumerate() {
                chips.push(Chip { label: label.to_string(), on: k == st.weapon, mark: false });
            }
            let weapon = WEAPONS[st.weapon.min(WEAPONS.len() - 1)].1;
            for ws in book::weapon_skills(st.weapon) {
                let s = Slot::Ability { id: ws.id, to: None };
                let title = named(&s);
                if title.is_empty() {
                    continue;
                }
                let who = if ws.jobs.is_empty() {
                    format!("Any job that can use a {}", weapon.to_lowercase())
                } else {
                    ws.jobs.iter().map(|j| JOBS[*j as usize]).collect::<Vec<_>>().join(", ")
                };
                let quest = if ws.quest { "\nLearned through a quest." } else { "" };
                tiles.push(Tile {
                    icon: icon(p, w, &s),
                    caption: format!("{}", ws.skill),
                    have: book::has_ability(w, ws.id),
                    title,
                    body: format!("{weapon} skill {}\n{who}{quest}\nClick to add, or drag onto a slot.", ws.skill),
                    slot: s,
                });
            }
        }
        (1, _) => {
            for (k, (_, label)) in MAGIC.iter().enumerate() {
                chips.push(Chip { label: label.to_string(), on: k == st.magic, mark: false });
            }
            let ty = MAGIC[st.magic.min(MAGIC.len() - 1)].0;
            for sp in book::spells_of(w, ty) {
                let s = Slot::Spell { id: sp.id, to: None };
                let title = named(&s);
                if title.is_empty() {
                    continue;
                }
                // The caption: our main job's level, else the sub job's, else the lowest.
                let pick = sp
                    .levels
                    .iter()
                    .find(|(j, _)| *j == main)
                    .or_else(|| sp.levels.iter().find(|(j, _)| *j == sub))
                    .or_else(|| sp.levels.iter().min_by_key(|(_, l)| *l));
                let caption = match pick {
                    Some((j, l)) if *j == main || *j == sub => format!("Lv. {l}"),
                    Some((j, l)) => format!("{} {l}", JOBS[*j as usize]),
                    None => String::new(),
                };
                tiles.push(Tile {
                    icon: icon(p, w, &s),
                    caption,
                    have: book::has_spell(w, sp.id),
                    title,
                    body: format!("{}\nClick to add, or drag onto a slot.", book::levels_text(&sp.levels)),
                    slot: s,
                });
            }
        }
        (3, _) => {
            for misc in &book::MISC {
                let s = if misc.key == "attack" { Slot::Attack } else { Slot::Misc { key: misc.key.to_string() } };
                tiles.push(Tile {
                    icon: icon(p, w, &s),
                    caption: misc.label.to_string(),
                    have: true,
                    title: misc.label.to_string(),
                    body: format!("{}\nClick to add, or drag onto a slot.", misc.help),
                    slot: s,
                });
            }
        }
        (4, _) => {
            for s in hotbar::choices(w, 4) {
                let title = named(&s);
                let caption = match &s {
                    Slot::Command { line } => {
                        line.trim_start_matches('/').split_whitespace().next().unwrap_or("").to_string()
                    }
                    _ => title.clone(),
                };
                tiles.push(Tile {
                    icon: icon(p, w, &s),
                    caption,
                    have: true,
                    body: "Runs as if typed in the chat line. Commands you type in chat show here too.\nClick to add, or drag onto a slot."
                        .to_string(),
                    title,
                    slot: s,
                });
            }
        }
        _ => {}
    }
    (chips, tiles)
}

fn recast_text(secs: u32) -> String {
    match secs {
        0 => String::new(),
        s if s >= 3600 => format!("\nRecast {} h", s / 3600),
        s if s >= 60 && s % 60 == 0 => format!("\nRecast {} min", s / 60),
        s if s >= 60 => format!("\nRecast {}:{:02}", s / 60, s % 60),
        s => format!("\nRecast {s} s"),
    }
}
