//! Timeless XI: the Timeless interface's hotbars on screen (client/timeless/hotbar.rs keeps them). Three bars of twelve
//! slots: each slot's icon with its key in the corner, a shadow when pressed, the recast as the Timeless hourglass
//! shrinking inside the icon as its sand drains, with the seconds left, and a flash when it is back. Click runs a slot,
//! right-click opens the picker for it, a drag moves it onto another slot (swapping) or, let go off the bars, clears it.

use super::super::World;
use super::super::magic::Kind;
use super::super::timeless::hotbar::{self, BARS, SLOTS, Slot, TABS};
use super::{Panels, Show, Title};
use egui::{Align2, Color32, CornerRadius, FontId, Id, LayerId, Order, Pos2, Rect, Sense, Shape, Stroke, Vec2, pos2, vec2};
use std::time::Instant;

/// A slot's side, the gap between slots, the bar-name column.
const S: f32 = 44.0;
const GAP: f32 = 4.0;
const NAME_W: f32 = 30.0;
/// The press shadow's and the ready flash's length in seconds.
const PRESS: f32 = 0.25;
const FLASH: f32 = 0.45;
/// A cleared slot's puff, in seconds.
const PUFF: f32 = 0.45;
/// The range ring's green.
const GREEN: Color32 = Color32::from_rgb(0x6c, 0xd0, 0x8a);

/// One slot as drawn this frame.
struct View {
    slot: Option<Slot>,
    icon: Option<(egui::TextureId, Rect)>,
    left: f32,
    total: f32,
    /// Why it can't be used now (dimmed, with a tag), none when it can.
    why: Option<hotbar::Why>,
    /// An item's (or the worn ammunition's) count and whether it runs low.
    count: Option<(u32, bool)>,
    /// The MP it costs.
    mp: Option<u16>,
    /// Its target someone else within reach (the green edge).
    ring: bool,
    /// Pressed while we were busy: runs when we can act.
    queued: bool,
    /// A two-hour pressed once: the next press within a moment uses it.
    armed: bool,
    /// A weapon skill that can go now (TP 1000, engaged, in reach): it glows.
    ws_ready: bool,
    /// The slot's own target's tag.
    target: Option<String>,
    /// The level it needs (above ours now).
    need: Option<u8>,
    press: f32,
    flash: f32,
    name: String,
    /// An Attack slot while engaged: lit red.
    fighting: bool,
}

/// What the mouse did on the bars this frame.
#[derive(Default)]
struct Mouse {
    click: Option<(usize, usize)>,
    menu: Option<(usize, usize)>,
    drag: Option<(usize, usize)>,
    hover: Option<(usize, usize, Pos2)>,
    /// A bar's label: clicked (shared <-> this job's) and hovered.
    toggle: Option<usize>,
    label: Option<(usize, Pos2)>,
    /// The Actions button: clicked, hovered.
    book: bool,
    book_tip: Option<Pos2>,
    /// The padlock: clicked, hovered.
    lock: bool,
    lock_tip: Option<Pos2>,
    /// The layout button: clicked, hovered.
    layout: bool,
    layout_tip: Option<Pos2>,
    /// Shift + the wheel over a bar: the bar and the page step.
    wheel: Option<(usize, i32)>,
    /// The pet bar: a slot clicked, dragged, hovered.
    pet_click: Option<usize>,
    pet_drag: Option<usize>,
    pet_hover: Option<(usize, Pos2)>,
}

/// Bar 1's strip's least width (the padlock, Actions, the layout button).
const STRIP_W: f32 = 160.0;

/// A bar's size in slots (scaled) and gaps.
fn slot_size(w: &World) -> (f32, f32) {
    let z = w.tl_bar.scale.clamp(0.75, 1.5);
    ((S * z).round(), (GAP * z).round().max(2.0))
}

/// Whether a bar shows now (bar 1's panel always does: its strip stays).
fn bar_shown(w: &World, b: usize) -> bool {
    let l = w.tl_bar.looks[b];
    !(l.show == 2 || (l.show == 1 && !hotbar::engaged(w)))
}

/// A bar's panel's height as drawn: the margins, bar 1's strip, its rows of slots. Worked out, not read back from
/// last frame, so the default places can't feed on themselves.
fn bar_height(w: &World, b: usize) -> f32 {
    let (sz, gap) = slot_size(w);
    let cols = (w.tl_bar.looks[b].cols as usize).clamp(3, 12);
    let rows = SLOTS.div_ceil(cols) as f32;
    let slots = if bar_shown(w, b) { rows * sz + (rows - 1.0) * gap } else { 0.0 };
    let strip = match b {
        0 if bar_shown(w, b) => STRIP + STRIP_GAP,
        0 => STRIP,
        _ => 0.0,
    };
    super::MARGIN.sum().y + strip + slots
}

/// The room between bar 1's slots and its strip under them.
const STRIP_GAP: f32 = 6.0;

/// How close (pixels) a let-go bar's edge must be to another's to snap to it, and the gap between snapped backdrops.
const SNAP: f32 = 14.0;
const SNAP_GAP: f32 = 4.0;

/// A bar panel's backdrop as seen: the panel's rect less its margins, plus the backdrop's 8 px.
fn seen(r: Rect) -> Rect {
    let m = super::MARGIN;
    Rect::from_min_max(
        r.min + vec2(m.left as f32 - 8.0, m.top as f32 - 8.0),
        r.max - vec2(m.right as f32 - 8.0, m.bottom as f32 - 8.0),
    )
}

/// Where a bar being let go at `r` snaps: the move that lines its backdrop up with another bar's or the chat window's
/// (edges level, or side by side / one over the other with a small gap), each axis on its own, when within SNAP.
fn snap_move(r: Rect, others: &[Rect]) -> Vec2 {
    let a = seen(r);
    let (mut bx, mut by): (Option<f32>, Option<f32>) = (None, None);
    let take = |best: &mut Option<f32>, d: f32| {
        if d.abs() <= SNAP && best.is_none_or(|b| d.abs() < b.abs()) {
            *best = Some(d);
        }
    };
    for o in others {
        // Only a neighbour near enough on the other axis lines up with this one.
        let near_y = a.min.y < o.max.y + SNAP * 3.0 && a.max.y > o.min.y - SNAP * 3.0;
        let near_x = a.min.x < o.max.x + SNAP * 3.0 && a.max.x > o.min.x - SNAP * 3.0;
        if near_y {
            take(&mut bx, o.min.x - a.min.x);
            take(&mut bx, o.max.x - a.max.x);
            take(&mut bx, o.max.x + SNAP_GAP - a.min.x);
            take(&mut bx, o.min.x - SNAP_GAP - a.max.x);
        }
        if near_x {
            take(&mut by, o.min.y - a.min.y);
            take(&mut by, o.max.y - a.max.y);
            take(&mut by, o.max.y + SNAP_GAP - a.min.y);
            take(&mut by, o.min.y - SNAP_GAP - a.max.y);
        }
    }
    vec2(bx.unwrap_or(0.0), by.unwrap_or(0.0))
}

/// The rects a bar snaps to: the other bars' backdrops shown last frame and the chat window.
fn snap_targets(p: &Panels, except: &str) -> Vec<Rect> {
    let rects = p.m.rects.borrow();
    let mut out: Vec<Rect> = BAR_IDS
        .iter()
        .chain(["tl_petbar"].iter())
        .filter(|id| **id != except)
        .filter_map(|id| p.this.get(*id).or(rects.get(*id)).copied().map(seen))
        .collect();
    if let Some(c) = p.this.get("chat").or(rects.get("chat")) {
        out.push(*c);
    }
    out
}

/// A bar let go: snapped (its place moved by the snap) and saved. While one is dragged: the guide lines it would snap
/// to.
fn snap(p: &mut Panels, w: &mut World) {
    let now = p.m.dragging.borrow().clone().filter(|d| BAR_IDS.contains(&d.as_str()) || d == "tl_petbar");
    let was = std::mem::replace(&mut w.tl_bar.moving, now.clone());
    let rect_of = |p: &Panels, id: &str| p.this.get(id).copied().or_else(|| p.m.rects.borrow().get(id).copied());
    if let Some(id) = &now
        && let Some(r) = rect_of(p, id)
    {
        let d = snap_move(r, &snap_targets(p, id));
        if d != Vec2::ZERO {
            let g = seen(r).translate(d);
            let gp = p.ctx.layer_painter(LayerId::new(Order::Foreground, Id::unique("tl hotbar snap")));
            gp.rect_stroke(g, CornerRadius::same(10), Stroke::new(1.5, p.m.theme.color("tp")), egui::StrokeKind::Outside);
        }
        return;
    }
    let Some(id) = was else { return };
    let Some(r) = rect_of(p, &id) else { return };
    let d = snap_move(r, &snap_targets(p, &id));
    if d == Vec2::ZERO {
        return;
    }
    let s = p.ctx.content_rect();
    let at = (r.min + d).round();
    let mut place = p.layout.get(&id).cloned().unwrap_or_default();
    place.dock = None;
    place.x = at.x / s.width().max(1.0);
    place.y = at.y / s.height().max(1.0);
    p.layout.insert(id, place);
    *p.changed = true;
}

/// The pet bar's panel's height.
fn pet_height(w: &World) -> f32 {
    let (sz, _) = slot_size(w);
    super::MARGIN.sum().y + (sz * 0.8).round() + 14.0
}

/// Each bar's panel (bar 1's carries the strip).
const BAR_IDS: [&str; BARS] = ["tl_hotbar", "tl_hotbar2", "tl_hotbar3"];

/// The strip over the bars holding the Actions button.
const STRIP: f32 = 24.0;

const FULL_UV: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));

pub fn draw(p: &mut Panels, w: &mut World) {
    if !hotbar::active(w) {
        return;
    }
    hotbar::sync(w);
    let pass = p.ctx.cumulative_pass_nr();
    w.tl_bar.party_hover = p.m.party_hover.get().filter(|(_, n)| pass.saturating_sub(*n) <= 1).map(|x| x.0);
    hotbar::tick(w);
    track(p, w);
    snap(p, w);
    place(p, w);
    let now = Instant::now();
    // Each slot's look, and the recasts that came back since last frame (their flash).
    let mut views: Vec<View> = Vec::with_capacity(BARS * SLOTS);
    let mut chime = false;
    for b in 0..BARS {
        for k in 0..SLOTS {
            let slot = w.tl_bar.bars[b][k].clone();
            let (left, total) = slot.as_ref().map_or((0.0, 0.0), |s| hotbar::recast(w, s));
            let running = left > 0.0;
            if w.tl_bar.recasting[b][k] && !running {
                w.tl_bar.ready_at[b][k] = Some(now);
                chime = true;
            }
            w.tl_bar.recasting[b][k] = running;
            let since = |t: Option<Instant>| t.map(|t| now.duration_since(t).as_secs_f32());
            let press = since(w.tl_bar.pressed[b][k]).map_or(0.0, |t| (1.0 - t / PRESS).max(0.0));
            let flash = since(w.tl_bar.ready_at[b][k]).map_or(0.0, |t| (1.0 - t / FLASH).max(0.0));
            let icon = slot.as_ref().and_then(|s| icon(p, w, s));
            let why = slot.as_ref().and_then(|s| hotbar::why(w, s));
            views.push(View {
                why,
                count: slot.as_ref().and_then(|s| hotbar::count(w, s)),
                mp: slot.as_ref().and_then(|s| hotbar::mp_cost(w, s)),
                ring: why.is_none() && slot.as_ref().and_then(|s| hotbar::in_range(w, s)) == Some(true),
                queued: w.tl_bar.queued.is_some_and(|(qb, qk, _)| (qb, qk) == (b, k)),
                armed: hotbar::armed(w) == Some((b, k)),
                ws_ready: why.is_none() && matches!(slot, Some(Slot::Ability { id, .. }) if id < 0x200),
                target: slot.as_ref().and_then(|s| hotbar::target_tag(s)).map(str::to_string),
                need: slot.as_ref().filter(|_| why == Some(hotbar::Why::Level)).and_then(|s| hotbar::need_level(w, s)),
                name: slot.as_ref().map(|s| hotbar::name(w, s)).unwrap_or_default(),
                fighting: matches!(slot, Some(Slot::Attack)) && hotbar::engaged(w),
                slot,
                icon,
                left,
                total,
                press,
                flash,
            });
        }
    }
    // The pet bar's slots.
    let pet_views: Vec<View> = hotbar::pet_actions(w)
        .into_iter()
        .map(|s| {
            let (left, total) = hotbar::recast(w, &s);
            let why = hotbar::why(w, &s);
            View {
                why,
                count: None,
                mp: hotbar::mp_cost(w, &s),
                ring: why.is_none() && hotbar::in_range(w, &s) == Some(true),
                queued: false,
                armed: false,
                ws_ready: false,
                target: None,
                need: None,
                name: hotbar::name(w, &s),
                fighting: false,
                icon: icon(p, w, &s),
                slot: Some(s),
                left,
                total,
                press: 0.0,
                flash: 0.0,
            }
        })
        .collect();
    if chime && w.tl_chime {
        w.sounds.push(crate::audio::Ui::Cursor as u32);
    }
    w.tl_bar.puffs.retain(|(_, t)| now.duration_since(*t).as_secs_f32() < PUFF);
    let animating = views.iter().chain(pet_views.iter()).any(|v| v.left > 0.0 || v.press > 0.0 || v.flash > 0.0 || v.queued || v.ws_ready || v.armed)
        || !w.tl_bar.puffs.is_empty()
        || w.tl_bar.cross_set.is_some();
    let labels = w.tl_labels;
    let m = p.m;
    let ctx = p.ctx.clone();
    let dragging = w.tl_bar.drag;
    let w_copy = w.tl_bar.drag_copy;
    let book_open = w.tl_bar.book.is_some();
    let locked = w.tl_bar.locked;
    let shared = w.tl_bar.shared_on;
    let job = hotbar::job_name(w.tl_bar.job);
    let pages = w.tl_bar.page;
    let looks = w.tl_bar.looks;
    let sets: [u8; BARS] = std::array::from_fn(|b| hotbar::key_set(&looks, b));
    let z = w.tl_bar.scale.clamp(0.75, 1.5);
    let _ = z;
    let (sz, gap) = slot_size(w);
    let (name_w, strip_h) = (NAME_W, STRIP);
    let fighting = hotbar::engaged(w);
    let layout_open = w.tl_bar.layout;
    // Fading while not fighting: back up under the pointer, while dragging, or with a menu open.
    let hovering = ctx.input(|i| i.pointer.latest_pos()).is_some_and(|at| {
        let rects = m.rects.borrow();
        BAR_IDS.iter().chain(["tl_petbar"].iter()).any(|id| rects.get(*id).is_some_and(|r| r.expand(8.0).contains(at)))
    });
    let awake = !w.tl_bar.fade
        || fighting
        || hovering
        || dragging.is_some()
        || w.tl_bar.carry.is_some()
        || w.tl_bar.menu.is_some()
        || layout_open;
    let fade = ctx.animate_value_with_time(Id::unique("tl hotbar fade"), if awake { 1.0 } else { 0.35 }, 0.35);
    let alpha = w.tl_bar.opacity.clamp(0.3, 1.0) * fade;
    let mut mouse = Mouse::default();
    let mut slot_rects: Vec<(usize, usize, Rect)> = Vec::new();
    let mut bars = Rect::NOTHING;
    for b in 0..BARS {
        let look = looks[b];
        let cols = (look.cols as usize).clamp(3, 12);
        let rows = SLOTS.div_ceil(cols);
        // Hidden (or out of a fight, for a combat-only bar): not drawn; bar 1 keeps its strip.
        let hidden = look.show == 2 || (look.show == 1 && !fighting);
        if hidden && b > 0 {
            continue;
        }
        let id = BAR_IDS[b];
        let width = name_w + cols as f32 * sz + (cols - 1) as f32 * gap;
        let height = if hidden { 0.0 } else { rows as f32 * sz + (rows - 1) as f32 * gap };
        // Locked, the bars stand where they are: a panel at a fixed point is not dragged.
        let fixed = locked.then(|| {
            let h = m.rects.borrow().get(id).map_or(0.0, |r| r.height());
            p.resolve(id, &p.place(id), h)
        });
        // The panel's inner width: bar 1's strip needs room for its buttons.
        let inner = if b == 0 { width.max(STRIP_W) } else { width };
        let shown = p.show(id, Show { size: vec2(inner + super::MARGIN.sum().x, 0.0), frame: "none", at: fixed, ..Default::default() }, |ui, _| {
            ui.multiply_opacity(alpha);
            // No spacing between the strip and the slots: the panel's height is exactly `bar_height`.
            ui.spacing_mut().item_spacing.y = 0.0;
            // Bar 1's strip (padlock, Actions, layout) under its slots, at the stack's foot.
            let (area, area_resp) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
            let strip = if b == 0 {
                if !hidden {
                    ui.add_space(STRIP_GAP);
                }
                Some(ui.allocate_exact_size(vec2(inner, strip_h), Sense::hover()).0)
            } else {
                None
            };
            let painter = ui.painter().clone();
            // A soft backdrop the slots sit on; its edge (the panel's margin) moves the bar.
            let mut back = m.theme.color("surface");
            back = Color32::from_rgba_unmultiplied(back.r(), back.g(), back.b(), 170);
            let edge = m.theme.color("bronze");
            let body = strip.map_or(area, |s| if hidden { s } else { area.union(s) });
            painter.rect(body.expand(8.0), CornerRadius::same(10), back, Stroke::new(1.0, edge.gamma_multiply(0.6)), egui::StrokeKind::Outside);
            if let Some(strip) = strip {
                // The padlock: locks the bars (nothing added, moved or cleared; they stay where they are).
                let lk = Rect::from_min_size(strip.min + vec2(2.0, 0.0), vec2(STRIP - 4.0, STRIP - 4.0));
                let lresp = ui.interact(lk, Id::unique("tl hotbar lock"), Sense::click());
                mouse.lock = lresp.clicked();
                if lresp.hovered() {
                    mouse.lock_tip = Some(lk.center_top());
                    painter.rect_filled(lk, CornerRadius::same(5), Color32::from_white_alpha(18));
                }
                if let Some(sp) = m.art.sprite(if locked { "icons/lock" } else { "icons/unlock" }) {
                    let tint = if locked { m.theme.color("tp") } else { m.theme.color("muted") };
                    painter.image(sp.tex, lk.shrink(2.0), sp.uv, tint);
                }
                // The Actions button: opens the Actions window (the books to drag actions from).
                let br = Rect::from_min_size(strip.min + vec2(NAME_W - 4.0, 0.0), vec2(96.0, STRIP - 4.0));
                let bresp = ui.interact(br, Id::unique("tl hotbar actions"), Sense::click());
                mouse.book = bresp.clicked();
                if bresp.hovered() {
                    mouse.book_tip = Some(br.center_top());
                }
                let fill = m.theme.color("focus").gamma_multiply(if bresp.hovered() || book_open { 0.4 } else { 0.22 });
                painter.rect_filled(br, CornerRadius::same(5), fill);
                if let Some(sp) = m.art.sprite("icons/abilities") {
                    let ir = Rect::from_min_size(br.min + vec2(6.0, 2.0), vec2(STRIP - 8.0, STRIP - 8.0));
                    painter.image(sp.tex, ir, sp.uv, m.theme.color("gold"));
                }
                painter.text(pos2(br.min.x + STRIP + 4.0, br.center().y), Align2::LEFT_CENTER, "Actions", FontId::proportional(12.5), m.theme.color("text"));
                // The layout button: shapes, pages, showing, size, opacity and fading.
                let gr = Rect::from_min_size(pos2(br.max.x + 6.0, br.min.y), vec2(STRIP - 4.0, STRIP - 4.0));
                let gresp = ui.interact(gr, Id::unique("tl hotbar layout"), Sense::click());
                mouse.layout = gresp.clicked();
                if gresp.hovered() || layout_open {
                    painter.rect_filled(gr, CornerRadius::same(5), m.theme.color("focus").gamma_multiply(0.3));
                }
                if gresp.hovered() {
                    mouse.layout_tip = Some(gr.center_top());
                }
                if let Some(sp) = m.art.sprite("icons/settings") {
                    painter.image(sp.tex, gr.shrink(2.0), sp.uv, m.theme.color("gold"));
                }
            }
            if hidden {
                return;
            }
            // Shift + the wheel over a bar: its page.
            if area_resp.contains_pointer() || ui.rect_contains_pointer(area.expand(6.0)) {
                let (shift, d) = ui.input(|i| {
                    let d: f32 = i
                        .raw
                        .events
                        .iter()
                        .map(|e| match e {
                            egui::Event::MouseWheel { delta, .. } => delta.x + delta.y,
                            _ => 0.0,
                        })
                        .sum();
                    (i.modifiers.shift, d)
                });
                if shift && d.abs() > 0.0 {
                    mouse.wheel = Some((b, if d < 0.0 { 1 } else { -1 }));
                }
            }
            // The bar's label: its key and whose bar it is (ALL shared, else the main job), its page; a click
            // switches shared / this job's.
            let lr = Rect::from_min_size(pos2(area.min.x - 4.0, area.min.y), vec2(NAME_W, sz.min(area.height())));
            let lresp = ui.interact(lr, Id::unique(("tl hotbar label", b)), Sense::click());
            if lresp.clicked() {
                mouse.toggle = Some(b);
            }
            if lresp.hovered() {
                mouse.label = Some((b, lr.center_top()));
                painter.rect_filled(lr.shrink(2.0), CornerRadius::same(4), Color32::from_white_alpha(14));
            }
            let cy = lr.center().y;
            painter.text(pos2(lr.center().x, cy - 9.0), Align2::CENTER_CENTER, hotbar::BAR_NAMES[b].trim_start_matches("Bar "), FontId::proportional(11.5), m.theme.color("text"));
            let (tag, col) = if shared[b] { ("ALL", m.theme.color("tp")) } else { (job, m.theme.color("muted")) };
            painter.text(pos2(lr.center().x, cy + 4.0), Align2::CENTER_CENTER, tag, FontId::proportional(10.0), col);
            // The page: three dots, the shown one lit.
            for pg in 0..hotbar::PAGES {
                let c = pos2(lr.center().x - 6.0 + pg as f32 * 6.0, cy + 14.0);
                let on = pg == pages[b];
                painter.circle_filled(c, if on { 2.2 } else { 1.6 }, if on { m.theme.color("tp") } else { m.theme.color("muted").gamma_multiply(0.6) });
            }
            for k in 0..SLOTS {
                let (row, col) = (k / cols, k % cols);
                let r = Rect::from_min_size(
                    pos2(area.min.x + NAME_W + col as f32 * (sz + gap), area.min.y + row as f32 * (sz + gap)),
                    vec2(sz, sz),
                );
                slot_rects.push((b, k, r));
                let sense = if locked { Sense::click() } else { Sense::click_and_drag() };
                let resp = ui.interact(r, Id::unique(("tl hotbar slot", b, k)), sense);
                let v = &views[b * SLOTS + k];
                if resp.clicked() {
                    mouse.click = Some((b, k));
                }
                if resp.secondary_clicked() {
                    mouse.menu = Some((b, k));
                }
                if resp.drag_started() && v.slot.is_some() {
                    mouse.drag = Some((b, k));
                }
                if resp.hovered() && dragging.is_none() {
                    mouse.hover = Some((b, k, r.center_top()));
                }
                let lit = resp.hovered() || dragging.is_some_and(|_| resp.contains_pointer());
                let held = dragging == Some((b, k)) && !w_copy;
                slot(ui, m, &painter, r, v, &hotbar::label_of(sets[b], k), lit, held, labels == 0);
            }
        });
        if let Some(r) = shown {
            bars = bars.union(r);
        }
    }
    // The pet bar: our pet's commands while it is out (Beastmaster, Summoner, Puppetmaster).
    if !pet_views.is_empty() {
        let n = pet_views.len();
        let ps = (sz * 0.8).round();
        let width = n as f32 * ps + (n - 1) as f32 * gap;
        let fixed = locked.then(|| {
            let h = m.rects.borrow().get("tl_petbar").map_or(0.0, |r| r.height());
            p.resolve("tl_petbar", &p.place("tl_petbar"), h)
        });
        let shown = p.show("tl_petbar", Show { size: vec2(width + 44.0, 0.0), frame: "none", at: fixed, ..Default::default() }, |ui, _| {
            ui.multiply_opacity(alpha);
            ui.spacing_mut().item_spacing.y = 0.0;
            let (area, _) = ui.allocate_exact_size(vec2(width.max(60.0), ps + 14.0), Sense::hover());
            let painter = ui.painter().clone();
            let mut back = m.theme.color("surface");
            back = Color32::from_rgba_unmultiplied(back.r(), back.g(), back.b(), 170);
            painter.rect(area.expand(8.0), CornerRadius::same(10), back, Stroke::new(1.0, m.theme.color("bronze").gamma_multiply(0.6)), egui::StrokeKind::Outside);
            painter.text(area.left_top(), Align2::LEFT_TOP, "Pet", FontId::proportional(10.5), m.theme.color("muted"));
            for (k, v) in pet_views.iter().enumerate() {
                let r = Rect::from_min_size(pos2(area.min.x + k as f32 * (ps + gap), area.min.y + 14.0), vec2(ps, ps));
                let resp = ui.interact(r, Id::unique(("tl pet slot", k)), Sense::click_and_drag());
                if resp.clicked() {
                    mouse.pet_click = Some(k);
                }
                if resp.drag_started() && !locked {
                    mouse.pet_drag = Some(k);
                }
                if resp.hovered() {
                    mouse.pet_hover = Some((k, r.center_top()));
                }
                slot(ui, m, &painter, r, v, "", resp.hovered(), false, false);
            }
        });
        if let Some(r) = shown {
            bars = bars.union(r);
        }
    }
    if animating {
        ctx.request_repaint();
    }
    // A drag: its icon follows the pointer; let go on a slot it swaps, off the bars it clears the slot.
    if let Some(from) = mouse.drag {
        w.tl_bar.drag = Some(from);
        // Shift held as the drag starts: a copy (the slot stays).
        w.tl_bar.drag_copy = ctx.input(|i| i.modifiers.shift);
        w.tl_bar.menu = None;
    }
    if let Some(from) = w.tl_bar.drag {
        let (pointer, released, down) =
            ctx.input(|i| (i.pointer.latest_pos(), i.pointer.any_released(), i.pointer.primary_down()));
        if let Some(at) = pointer {
            let v = &views[from.0 * SLOTS + from.1];
            let gp = ctx.layer_painter(LayerId::new(Order::Tooltip, Id::unique("tl hotbar drag")));
            let r = Rect::from_center_size(at, vec2(S, S));
            if let Some((t, uv)) = v.icon {
                gp.image(t, r, uv, Color32::from_white_alpha(210));
            } else if let Some(s) = &v.slot {
                glyph(m, &gp, r, s);
            }
            if w.tl_bar.drag_copy {
                let c = Rect::from_center_size(r.right_top() + vec2(-4.0, 4.0), vec2(16.0, 16.0));
                gp.circle_filled(c.center(), 8.0, m.theme.color("tp"));
                gp.text(c.center(), Align2::CENTER_CENTER, "+", FontId::proportional(14.0), m.theme.color("charcoal"));
            }
        }
        if released || !down {
            let to =
                pointer.and_then(|at| slot_rects.iter().find(|(_, _, r)| r.contains(at)).map(|(b, k, _)| (*b, *k)));
            let copy = std::mem::take(&mut w.tl_bar.drag_copy);
            match to {
                Some(to) if copy => {
                    w.tl_bar.copy(from, to);
                    if from != to {
                        w.tl_bar.ready_at[to.0][to.1] = Some(Instant::now());
                        w.sounds.push(crate::audio::Ui::Select as u32);
                    }
                }
                Some(to) => w.tl_bar.swap(from, to),
                // Let go off the bars: cleared, in a puff (a copy just goes nowhere).
                None if !copy && pointer.is_some_and(|at| !bars.expand(12.0).contains(at)) => {
                    w.tl_bar.set(from.0, from.1, None);
                    if let Some(at) = pointer {
                        w.tl_bar.puffs.push((at, Instant::now()));
                    }
                    w.sounds.push(crate::audio::Ui::Cancel as u32);
                }
                None => {}
            }
            w.tl_bar.drag = None;
        }
    }
    if let Some((b, k)) = mouse.click
        && w.tl_bar.drag.is_none()
    {
        hotbar::press(w, b, k);
    }
    if let Some(b) = mouse.toggle {
        if locked {
            w.sounds.push(crate::audio::Ui::Error as u32);
        } else {
            w.tl_bar.toggle_shared(b);
            w.sounds.push(crate::audio::Ui::Select as u32);
        }
    }
    if mouse.layout {
        w.tl_bar.layout = !w.tl_bar.layout;
        w.tl_bar.layout_at = Some(Instant::now());
        w.sounds.push(crate::audio::Ui::Select as u32);
    }
    if let Some(at) = mouse.layout_tip {
        m.tip_at(&ctx, at, "Hotbar Layout", "Each bar's shape, page and when it shows; the bars' size, opacity and fading.");
    }
    if let Some((b, step)) = mouse.wheel {
        let n = (w.tl_bar.page[b] as i32 + step).rem_euclid(hotbar::PAGES as i32) as usize;
        w.tl_bar.set_page(b, n);
        w.sounds.push(crate::audio::Ui::Cursor as u32);
    }
    if let Some(k) = mouse.pet_click
        && let Some(s) = pet_views.get(k).and_then(|v| v.slot.clone())
    {
        hotbar::run(w, &s);
    }
    if let Some(k) = mouse.pet_drag
        && let Some(s) = pet_views.get(k).and_then(|v| v.slot.clone())
    {
        w.tl_bar.carry = Some(s);
    }
    if let Some((k, at)) = mouse.pet_hover
        && w.tl_bar.carry.is_none()
        && let Some(v) = pet_views.get(k)
    {
        let mut lines = Vec::new();
        if let Some(why) = v.why.filter(|&y| y != hotbar::Why::Recast) {
            lines.push(why.text().to_string());
        }
        lines.push("Click to use. Drag onto a hotbar slot to keep it there.".to_string());
        m.tip_at(&ctx, at, &v.name, &lines.join("\n"));
    }
    if mouse.lock {
        w.tl_bar.set_locked(!locked);
        w.sounds.push(crate::audio::Ui::Select as u32);
    }
    if let Some(at) = mouse.lock_tip {
        let (t, b) = if locked {
            ("Hotbars locked", "Nothing can be added, moved or cleared, and the bars stay put. Click to unlock.")
        } else {
            ("Hotbars unlocked", "Slots can be set, dragged and cleared, and the bars moved. Click to lock.")
        };
        m.tip_at(&ctx, at, t, b);
    }
    if let Some((b, at)) = mouse.label
        && w.tl_bar.drag.is_none()
    {
        let job = hotbar::job_name(w.tl_bar.job);
        let (title, body) = if w.tl_bar.shared_on[b] {
            (
                format!("{}: shared", hotbar::BAR_NAMES[b]),
                format!("The same slots on every job. Click to give {job} its own bar for this row."),
            )
        } else {
            (
                format!("{}: {job} only", hotbar::BAR_NAMES[b]),
                "Changes with your main job. Click to share this row across every job.".to_string(),
            )
        };
        m.tip_at(&ctx, at, &title, &body);
    }
    let mut opened = false;
    if let Some((b, k)) = mouse.menu {
        if locked {
            w.sounds.push(crate::audio::Ui::Error as u32);
        } else if w.tl_bar.menu.is_some_and(|(mb, mk, _)| (mb, mk) == (b, k)) {
            w.tl_bar.menu = None;
            w.tl_bar.menu_edit = None;
        } else {
            let r = slot_rects.iter().find(|(sb, sk, _)| (*sb, *sk) == (b, k)).map_or(Rect::NOTHING, |x| x.2);
            w.tl_bar.menu = Some((b, k, r.left_top() + vec2(0.0, -6.0)));
            w.tl_bar.menu_at = Some(Instant::now());
            w.tl_bar.menu_edit = None;
            w.tl_bar.menu_sub = 0;
            w.tl_bar.picking = None;
            opened = true;
            w.sounds.push(crate::audio::Ui::Select as u32);
        }
    }
    if let Some((b, k, at)) = mouse.hover
        && w.tl_bar.picking.is_none()
    {
        let v = &views[b * SLOTS + k];
        let key = hotbar::key_text_of(sets[b], k);
        let key = if key.is_empty() { "none (click only)".to_string() } else { key };
        let mut lines: Vec<String> = Vec::new();
        if v.slot.as_ref().is_some_and(|s| hotbar::two_hour(w, s)) && w.tl_bar.confirm_sp {
            lines.push("Two-hour: press twice to use.".into());
        }
        if let Some(why) = v.why.filter(|&y| y != hotbar::Why::Recast) {
            lines.push(why.text().to_string());
        }
        if v.queued {
            lines.push("Queued: runs as soon as you can act.".into());
        }
        if let Some(s) = &v.slot
            && hotbar::takes_target(s)
        {
            lines.push(format!("Target: {}", hotbar::target_label(v.target.as_deref())));
        }
        if let Some(mp) = v.mp {
            lines.push(format!("{mp} MP"));
        }
        if let Some(n) = v.need {
            lines.push(format!("Needs Lv. {n}."));
        }
        if let Some(Slot::Gear { pieces, .. }) = &v.slot {
            let missing = hotbar::gear_missing(w, pieces);
            for (sl, id) in pieces {
                let name = hotbar::item_name(w, *id).unwrap_or_else(|| format!("Item {id}"));
                let gone = if missing.contains(id) { "  (not with you)" } else { "" };
                lines.push(format!("{}: {name}{gone}", hotbar::GEAR_SLOTS[(*sl as usize).min(15)]));
            }
        }
        let how = if locked && v.slot.is_some() {
            format!("Key {key}. Click or press to use. (Locked: unlock to change it.)")
        } else if locked {
            "Locked: unlock the bars to set it.".to_string()
        } else if matches!(v.slot, Some(Slot::Attack)) {
            format!("Key {key}. Engages your target; while you fight, disengages. Right-click for options.")
        } else if v.slot.is_some() {
            format!("Key {key}. Click or press to use. Right-click for options. Drag to move, Shift+drag to copy.")
        } else {
            "Right-click to set what this slot does, or click a tile in Actions.".to_string()
        };
        lines.push(how);
        let title = if v.slot.is_some() { v.name.clone() } else { format!("Empty slot {key}") };
        m.tip_at(&ctx, at, &title, &lines.join("\n"));
    }
    if mouse.book {
        w.tl_bar.book = if w.tl_bar.book.is_some() {
            None
        } else {
            Some(super::super::timeless::book::BookState { job: w.tl_bar.job, ..Default::default() })
        };
        w.tl_bar.book_at = Some(Instant::now());
        w.sounds.push(crate::audio::Ui::Select as u32);
    }
    if let Some(at) = mouse.book_tip {
        m.tip_at(&ctx, at, "Actions", "Every ability, weapon skill, spell and action: drag one onto a slot.");
    }
    // Key labels on hover: the slot's whole key over it.
    if labels == 1
        && let Some((b, k, _)) = mouse.hover
        && sets[b] != hotbar::NO_KEYS
        && let Some((_, _, r)) = slot_rects.iter().find(|(sb, sk, _)| (*sb, *sk) == (b, k))
    {
        let gp = ctx.layer_painter(LayerId::new(Order::Foreground, Id::unique("tl hotbar key badge")));
        let text = hotbar::key_text_of(sets[b], k);
        let galley = gp.layout_no_wrap(text, FontId::proportional(11.5), m.theme.color("text"));
        let pill = Rect::from_center_size(pos2(r.center().x, r.min.y + 9.0), galley.size() + vec2(10.0, 4.0));
        gp.rect_filled(pill, CornerRadius::same(5), m.theme.color("charcoal").gamma_multiply(0.92));
        gp.rect_stroke(pill, CornerRadius::same(5), Stroke::new(1.0, m.theme.color("focus")), egui::StrokeKind::Inside);
        gp.galley(pill.center() - galley.size() / 2.0, galley, Color32::WHITE);
    }
    puffs(&ctx, m, &w.tl_bar.puffs);
    if let Some(set) = w.tl_bar.cross_set {
        cross(&ctx, m, &views, set, sz);
    }
    menu(p, w, opened);
    layout(p, w);
    picker(p, w);
    super::tlbook::draw(p, w);
    carry(p, w, &slot_rects);
}

/// Escape's order and the Misc Book's closing: when a game window (the one with the keys, or the map) opened or was
/// clicked, when ours were clicked; the Misc window a press opened; an Escape to send the game this frame.
fn track(p: &mut Panels, w: &mut World) {
    let now = Instant::now();
    let focus = super::focus_panel(w).or(w.map.is_some().then_some("map"));
    if focus != w.tl_bar.game_seen {
        if focus.is_some() {
            w.tl_bar.game_at = Some(now);
        }
        w.tl_bar.game_seen = focus;
    }
    match w.tl_bar.misc_open.clone() {
        Some((k, None, t)) if focus.is_some() => w.tl_bar.misc_open = Some((k, focus, t)),
        Some((_, None, t)) if t.elapsed().as_secs_f32() > 2.0 => w.tl_bar.misc_open = None,
        Some((_, Some(id), _)) if focus != Some(id) => w.tl_bar.misc_open = None,
        _ => {}
    }
    if w.tl_bar.esc_left > 0 {
        let ours = |f: &str| Some(f) == w.tl_bar.esc_target || f.starts_with("command");
        if focus.is_some_and(ours) {
            w.tl_bar.esc_now = true;
            w.tl_bar.esc_left -= 1;
        } else {
            w.tl_bar.esc_left = 0;
        }
    }
    let (press, at) = p.ctx.input(|i| (i.pointer.any_pressed(), i.pointer.interact_pos()));
    if press && let Some(at) = at {
        let rects = p.m.rects.borrow();
        let inside = |id: &str| rects.get(id).is_some_and(|r| r.contains(at));
        if inside("tl_actions") {
            w.tl_bar.book_at = Some(now);
        } else if inside("tl_picker") {
            w.tl_bar.picker_at = Some(now);
        } else if inside("tl_layout") {
            w.tl_bar.layout_at = Some(now);
        } else if !inside("tl_hotbar") && focus.is_some_and(|f| inside(f) || f == "map") {
            w.tl_bar.game_at = Some(now);
        }
    }
    // Wide Scan from a slot: a scan and its window, or the window folded when its list is up.
    if std::mem::take(&mut w.tl_bar.scan_press) {
        let folded = p.folded.get("widescan").copied().unwrap_or(false);
        if !folded && !w.scan.entries.is_empty() {
            p.folded.insert("widescan".into(), true);
        } else {
            p.folded.insert("widescan".into(), false);
            super::super::timeless::book::run_misc(w, "widescan");
        }
        *p.changed = true;
    }
}

/// The bars' default place: just right of the chat window, their bottom on its bottom (until moved). A save file from
/// before this default forgets the bars' saved place once.
fn place(p: &mut Panels, w: &mut World) {
    if std::mem::take(&mut w.tl_bar.reset_place) {
        for id in BAR_IDS.iter().chain(["tl_petbar"].iter()) {
            if p.layout.remove(*id).is_some() {
                *p.changed = true;
            }
        }
        w.tl_bar.save();
    }
    let shown: Vec<usize> = (0..BARS).filter(|&b| b == 0 || bar_shown(w, b)).collect();
    let s = p.ctx.content_rect();
    // As Panels::resolve reads a place back: a fraction of the screen's width and height.
    let frac = |at: Pos2| super::Place {
        x: (at.x / s.width()).clamp(0.0, 0.95),
        y: (at.y / s.height()).clamp(0.0, 0.95),
        ..Default::default()
    };
    let rect_of = |p: &Panels, id: &str| p.this.get(id).copied().or_else(|| p.m.rects.borrow().get(id).copied());
    // A dock saved between our bars (before they refused them) becomes a plain place where the bar stands.
    for id in BAR_IDS.iter().chain(["tl_petbar"].iter()) {
        let Some(pl) = p.layout.get(*id).cloned() else { continue };
        if !pl.dock.as_ref().is_some_and(|d| d.to.starts_with("tl_")) {
            continue;
        }
        let mut fixed = match rect_of(p, id) {
            Some(r) => frac(pos2(r.min.x.round(), r.min.y.round())),
            None => super::Place { x: pl.x, y: pl.y, ..Default::default() },
        };
        (fixed.w, fixed.h) = (pl.w, pl.h);
        p.layout.insert(id.to_string(), fixed);
        *p.changed = true;
    }
    // The default places, from the chat window alone (so each bar moves on its own): bar 1 (its strip at the foot)
    // with its bottom on the chat window's bottom just right of it, then the Ctrl bar over it, the Alt bar over that,
    // the pet bar on top; backdrops SNAP_GAP apart.
    let Some(chat) = rect_of(p, "chat") else { return };
    let m = super::MARGIN;
    // Panels overlap by their margins less the backdrops' 8 px, less the gap.
    let overlap = (m.top as f32 - 8.0) + (m.bottom as f32 - 8.0) - SNAP_GAP;
    let x = (chat.max.x + 12.0).round();
    let mut bottom = chat.max.y;
    for &b in &shown {
        let top = (bottom - bar_height(w, b)).round();
        if !p.layout.contains_key(BAR_IDS[b]) {
            p.retail.insert(BAR_IDS[b].into(), frac(pos2(x, top)));
        }
        bottom = top + overlap;
    }
    if !p.layout.contains_key("tl_petbar") {
        p.retail.insert("tl_petbar".into(), frac(pos2(x, (bottom - pet_height(w)).round())));
    }
}

/// Something carried from the Actions window or the Items list: its icon follows the pointer; let go on a slot it
/// goes there, anywhere else it is dropped.
fn carry(p: &Panels, w: &mut World, slots: &[(usize, usize, Rect)]) {
    let Some(s) = w.tl_bar.carry.clone() else { return };
    let ctx = p.ctx.clone();
    let (pointer, released, down) =
        ctx.input(|i| (i.pointer.latest_pos(), i.pointer.any_released(), i.pointer.primary_down()));
    if let Some(at) = pointer {
        let gp = ctx.layer_painter(LayerId::new(Order::Tooltip, Id::unique("tl hotbar carry")));
        let r = Rect::from_center_size(at, vec2(S, S));
        gp.rect_filled(r, CornerRadius::same(6), p.m.theme.color("charcoal"));
        match icon(p, w, &s) {
            Some((t, uv)) if plate(&s).is_some() => {
                gp.rect_filled(
                    r.shrink(3.0),
                    CornerRadius::same(4),
                    p.m.theme.color(plate(&s).unwrap_or("focus")).gamma_multiply(0.45),
                );
                gp.image(t, r.shrink(9.0), uv, Color32::WHITE);
            }
            Some((t, uv)) => {
                gp.image(t, r.shrink(3.0), uv, Color32::from_white_alpha(220));
            }
            None => glyph(p.m, &gp, r.shrink(3.0), &s),
        }
        // The slot it would land on.
        if let Some((_, _, sr)) = slots.iter().find(|(_, _, sr)| sr.contains(at)) {
            gp.rect_stroke(
                *sr,
                CornerRadius::same(6),
                Stroke::new(2.0, if w.tl_bar.locked { p.m.theme.color("hp") } else { p.m.theme.color("tp") }),
                egui::StrokeKind::Outside,
            );
        }
    }
    if released || !down {
        if let Some((b, k)) =
            pointer.and_then(|at| slots.iter().find(|(_, _, r)| r.contains(at)).map(|(b, k, _)| (*b, *k)))
        {
            if w.tl_bar.locked {
                w.sounds.push(crate::audio::Ui::Error as u32);
            } else {
                w.tl_bar.set(b, k, Some(s));
                w.sounds.push(crate::audio::Ui::Select as u32);
            }
        }
        w.tl_bar.carry = None;
    }
}

/// The plate colour of a slot drawn as the pack's glyph on a plate (Attack red, the Misc Book's blue).
pub(super) fn plate(s: &Slot) -> Option<&'static str> {
    match s {
        Slot::Attack => Some("hp"),
        Slot::Misc { .. } => Some("focus"),
        Slot::Gear { .. } => Some("tp"),
        _ => None,
    }
}

/// A slot's icon: the modern art (or the game's) for a spell or ability, the game's for an item; a command has none
/// (it gets a glyph).
pub(super) fn icon(p: &Panels, w: &World, s: &Slot) -> Option<(egui::TextureId, Rect)> {
    match s {
        Slot::Spell { id, .. } => p.m.row_icon(p.ctx, p.gu?, w, Kind::Magic, *id),
        Slot::Ability { id, .. } => {
            let i = hotbar::ability_index(w, *id)?;
            p.m.row_icon(p.ctx, p.gu?, w, Kind::Ability, i)
        }
        Slot::Item { id, .. } => p.m.item_icon(p.ctx, w.item_info.as_deref()?, *id).map(|t| (t, FULL_UV)),
        Slot::Command { .. } => None,
        Slot::Attack => {
            let s = p.m.art.sprite(if hotbar::engaged(w) { "icons/disengage" } else { "icons/attack" })?;
            Some((s.tex, s.uv))
        }
        Slot::Gear { .. } => {
            let s = p.m.art.sprite("icons/equipment")?;
            Some((s.tex, s.uv))
        }
        Slot::Misc { key } => {
            let m = super::super::timeless::book::misc(key)?;
            let s = p.m.art.sprite(&format!("icons/{}", m.icon))?;
            Some((s.tex, s.uv))
        }
    }
}

/// A command's stand-in icon: its verb on a plate.
pub(super) fn glyph(m: &super::Modern, painter: &egui::Painter, r: Rect, s: &Slot) {
    let Slot::Command { line } = s else { return };
    let verb: String = line
        .trim_start_matches('/')
        .split_whitespace()
        .next()
        .unwrap_or("")
        .chars()
        .take(5)
        .collect::<String>()
        .to_uppercase();
    let plate = m.theme.color("focus").gamma_multiply(0.35);
    painter.rect_filled(r.shrink(4.0), CornerRadius::same(5), plate);
    painter.text(
        r.center() - vec2(0.0, 7.0),
        Align2::CENTER_CENTER,
        "/",
        FontId::proportional(15.0),
        m.theme.color("gold"),
    );
    painter.text(
        r.center() + vec2(0.0, 8.0),
        Align2::CENTER_CENTER,
        verb,
        FontId::proportional(if line.len() > 4 { 10.0 } else { 11.5 }),
        m.theme.color("text"),
    );
}

/// One slot: frame, icon, dimming with its reason, recast hourglass and seconds, press shadow, ready glint, the edge
/// (hover, queued, fighting, in range), key hint, own-target tag, MP cost and count.
#[allow(clippy::too_many_arguments)]
fn slot(
    ui: &egui::Ui,
    m: &super::Modern,
    painter: &egui::Painter,
    r: Rect,
    v: &View,
    key: &str,
    lit: bool,
    held: bool,
    show_key: bool,
) {
    let t = ui.input(|i| i.time) as f32;
    let pulse = 0.5 + 0.5 * (t * 5.0).sin();
    let round = CornerRadius::same(6);
    painter.rect_filled(r, round, m.theme.color("charcoal"));
    let inner = r.shrink(3.0);
    let pic = painter.with_clip_rect(inner.intersect(ui.clip_rect()));
    if let Some(s) = &v.slot {
        let tint = if held { Color32::from_white_alpha(90) } else { Color32::WHITE };
        match v.icon {
            // Attack and the Misc Book's actions: the pack's glyph on a plate (Attack red, lit while fighting).
            Some((tex, uv)) if plate(s).is_some() => {
                let c = m.theme.color(plate(s).unwrap_or("focus"));
                pic.rect_filled(inner, CornerRadius::same(4), c.gamma_multiply(if v.fighting { 0.75 } else { 0.4 }));
                pic.image(tex, inner.shrink(6.0), uv, tint);
            }
            Some((tex, uv)) => {
                pic.image(tex, inner, uv, tint);
            }
            None => glyph(m, &pic, inner, s),
        }
        // A weapon skill that can go now: a soft gold glow inside.
        if v.ws_ready && v.left <= 0.0 {
            let gold = m.theme.color("tp");
            pic.rect_stroke(
                inner,
                CornerRadius::same(3),
                Stroke::new(3.0, gold.gamma_multiply(0.35 + 0.35 * pulse)),
                egui::StrokeKind::Inside,
            );
        }
        if v.left > 0.0 {
            hourglass(&pic, m, inner, v.left, v.total, t);
            let secs =
                if v.left >= 60.0 { format!("{}m", (v.left / 60.0).ceil()) } else { format!("{}", v.left.ceil()) };
            outlined(painter, r.max - vec2(4.0, 2.0), Align2::RIGHT_BOTTOM, &secs, 12.0, Color32::WHITE);
        } else if let Some(why) = v.why {
            let a = if why == hotbar::Why::Empty { 175 } else { 140 };
            pic.rect_filled(inner, CornerRadius::ZERO, Color32::from_black_alpha(a));
            let tag = why.tag();
            if !tag.is_empty() {
                let c = match why {
                    hotbar::Why::Mp => m.theme.color("mp"),
                    hotbar::Why::Tp => m.theme.color("tp"),
                    hotbar::Why::Range | hotbar::Why::Engage => m.theme.color("text"),
                    _ => m.theme.color("hp"),
                };
                let size = if tag.len() > 5 { 8.5 } else { 10.5 };
                outlined(painter, inner.center() + vec2(0.0, 1.0), Align2::CENTER_CENTER, tag, size, c);
            }
        }
        if v.press > 0.0 {
            pic.rect_filled(inner, CornerRadius::ZERO, Color32::from_black_alpha((170.0 * v.press) as u8));
        }
        // A two-hour armed by its first press: red, asking for the second.
        if v.armed {
            let red = m.theme.color("hp");
            pic.rect_filled(inner, CornerRadius::ZERO, red.gamma_multiply(0.25 + 0.2 * pulse));
            outlined(painter, inner.center(), Align2::CENTER_CENTER, "AGAIN", 10.0, Color32::WHITE);
        }
        // The ready glint: a band of light sweeping across the icon as the recast comes back.
        if v.flash > 0.0 {
            let k = 1.0 - v.flash;
            let span = inner.width() + inner.height();
            let x = inner.min.x - inner.height() + span * k;
            let band = 9.0;
            let a = Color32::from_white_alpha((150.0 * v.flash) as u8);
            pic.add(Shape::convex_polygon(
                vec![
                    pos2(x, inner.max.y),
                    pos2(x + band, inner.max.y),
                    pos2(x + band + inner.height(), inner.min.y),
                    pos2(x + inner.height(), inner.min.y),
                ],
                a,
                Stroke::NONE,
            ));
        }
    }
    // The bevel: a lit top edge and the frame.
    painter.line_segment(
        [pos2(r.min.x + 5.0, r.min.y + 1.5), pos2(r.max.x - 5.0, r.min.y + 1.5)],
        Stroke::new(1.0, Color32::from_white_alpha(40)),
    );
    let gold = m.theme.color("tp");
    let edge = if v.armed {
        Stroke::new(2.2, m.theme.color("hp").gamma_multiply(0.5 + 0.5 * pulse))
    } else if lit {
        Stroke::new(1.6, m.theme.color("focus"))
    } else if v.queued {
        Stroke::new(2.0, gold.gamma_multiply(0.45 + 0.55 * pulse))
    } else if v.fighting {
        Stroke::new(1.8, m.theme.color("hp"))
    } else if v.ring {
        Stroke::new(1.6, GREEN)
    } else if v.why == Some(hotbar::Why::Range) {
        Stroke::new(1.4, m.theme.color("hp").gamma_multiply(0.8))
    } else {
        Stroke::new(1.0, m.theme.color("bronze"))
    };
    painter.rect_stroke(r, round, edge, egui::StrokeKind::Inside);
    if v.flash > 0.0 {
        let grow = (1.0 - v.flash) * 6.0;
        painter.rect_stroke(
            r.expand(1.0 + grow),
            CornerRadius::same(7 + grow as u8),
            Stroke::new(2.0, Color32::from_rgba_unmultiplied(gold.r(), gold.g(), gold.b(), (230.0 * v.flash) as u8)),
            egui::StrokeKind::Outside,
        );
    }
    if show_key {
        outlined(painter, r.min + vec2(4.0, 2.0), Align2::LEFT_TOP, key, 11.0, m.theme.color("text"));
    }
    if let Some(Slot::Gear { name, .. }) = &v.slot {
        let short: String = name.chars().take(7).collect();
        outlined(painter, pos2(r.center().x, r.max.y - 2.0), Align2::CENTER_BOTTOM, &short, 9.5, m.theme.color("text"));
    }
    if let Some(tag) = v.target.as_deref() {
        outlined(painter, pos2(r.max.x - 4.0, r.min.y + 2.0), Align2::RIGHT_TOP, &target_short(tag), 9.5, gold);
    }
    if let Some(mp) = v.mp {
        outlined(painter, pos2(r.min.x + 4.0, r.max.y - 2.0), Align2::LEFT_BOTTOM, &mp.to_string(), 9.5, m.theme.color("mp"));
    }
    if let Some((n, low)) = v.count
        && v.left <= 0.0
    {
        let c = if n == 0 {
            m.theme.color("muted")
        } else if low {
            m.theme.color("hp")
        } else {
            m.theme.color("text")
        };
        outlined(painter, r.max - vec2(4.0, 2.0), Align2::RIGHT_BOTTOM, &n.to_string(), 11.0, c);
    }
}

/// A slot's own target as its corner tag ("ME", "ST", "P3").
fn target_short(tag: &str) -> String {
    match tag {
        "<t>" => "T".into(),
        "<me>" => "ME".into(),
        "<st>" => "ST".into(),
        "<stpc>" => "PC".into(),
        "<bt>" => "BT".into(),
        t if t.starts_with("<p") && t.len() == 4 => {
            format!("P{}", t[2..3].parse::<u8>().map_or(0, |n| n + 1))
        }
        _ => "*".into(),
    }
}

/// The controller's cross hotbar while LT or RT is held: slots 1..8 of the Ctrl bar (LT) or the Alt bar (RT) as two
/// diamonds, the d-pad's on the left and the face buttons' on the right, low in the middle of the screen.
fn cross(ctx: &egui::Context, m: &super::Modern, views: &[View], set: usize, sz: f32) {
    let bar = 1 + set;
    let screen = ctx.content_rect();
    let step = sz * 1.08;
    let center = pos2(screen.center().x, screen.max.y - screen.height() * 0.30);
    let gap = step * 2.4;
    egui::Area::new(Id::unique("tl cross hotbar")).order(Order::Foreground).fixed_pos(screen.min).interactable(false).show(
        ctx,
        |ui| {
            let painter = ui.painter().clone();
            let back = Rect::from_center_size(center, vec2(gap * 2.0 + step * 3.2, step * 3.6));
            let mut bg = m.theme.color("surface");
            bg = Color32::from_rgba_unmultiplied(bg.r(), bg.g(), bg.b(), 200);
            painter.rect(back, CornerRadius::same(14), bg, Stroke::new(1.0, m.theme.color("bronze")), egui::StrokeKind::Outside);
            let title = if set == 0 { "LT  -  Ctrl bar" } else { "RT  -  Alt bar" };
            outlined(&painter, pos2(back.center().x, back.min.y + 4.0), Align2::CENTER_TOP, title, 12.0, m.theme.color("gold"));
            const SPOTS: [(f32, f32); 4] = [(0.0, -1.0), (1.0, 0.0), (0.0, 1.0), (-1.0, 0.0)];
            const NAMES: [&str; 8] = ["Up", "Right", "Down", "Left", "Y", "B", "A", "X"];
            for k in 0..8 {
                let side = if k < 4 { -gap / 2.0 - step * 0.2 } else { gap / 2.0 + step * 0.2 };
                let (dx, dy) = SPOTS[k % 4];
                let c = center + vec2(side + dx * step, dy * step + step * 0.15);
                let r = Rect::from_center_size(c, vec2(sz, sz));
                let v = &views[bar * SLOTS + k];
                slot(ui, m, &painter, r, v, NAMES[k], false, false, true);
            }
        },
    );
}

/// Cleared slots' puffs: a ring and a few motes spreading and fading where the slot was let go.
fn puffs(ctx: &egui::Context, m: &super::Modern, puffs: &[(Pos2, Instant)]) {
    if puffs.is_empty() {
        return;
    }
    let gp = ctx.layer_painter(LayerId::new(Order::Tooltip, Id::unique("tl hotbar puffs")));
    let now = Instant::now();
    let smoke = m.theme.color("gold");
    for (at, t0) in puffs {
        let k = (now.duration_since(*t0).as_secs_f32() / PUFF).clamp(0.0, 1.0);
        let fade = 1.0 - k;
        gp.circle_stroke(*at, 6.0 + 18.0 * k, Stroke::new(2.0 * fade + 0.5, smoke.gamma_multiply(0.7 * fade)));
        for i in 0..7 {
            let a = i as f32 / 7.0 * std::f32::consts::TAU + 0.4;
            let d = 4.0 + 22.0 * k;
            let p = *at + vec2(a.cos(), a.sin()) * d - vec2(0.0, 6.0 * k);
            gp.circle_filled(p, 3.5 * fade + 1.0, smoke.gamma_multiply(0.55 * fade));
        }
    }
}

/// The recast inside an icon: the Timeless XI hourglass (the logo's glass and tiered caps) that starts the slot's
/// size and shrinks to nothing as the recast runs out, its sand draining from the top bulb into a pile below. The
/// icon underneath darkens more the longer is left. `left` of the whole `total`, `t` the clock (the falling grains).
fn hourglass(painter: &egui::Painter, m: &super::Modern, r: Rect, left: f32, total: f32, t: f32) {
    let rem = if total > 0.0 { (left / total).clamp(0.0, 1.0) } else { 0.0 };
    painter.rect_filled(r, CornerRadius::ZERO, Color32::from_black_alpha((64.0 + 100.0 * rem) as u8));
    // The logo's units: the glass from y -78 to 78, the caps out to 89, 104 wide.
    let s = r.height() * 0.94 / 178.0 * rem;
    if s * 178.0 < 3.0 {
        return;
    }
    let c = r.center();
    let at = |x: f32, y: f32| c + vec2(x * s, y * s);
    let wall = wall_points();
    let left_wall: Vec<Pos2> = wall.iter().map(|&(x, y)| at(x, y)).collect();
    let right_wall: Vec<Pos2> = wall.iter().map(|&(x, y)| at(-x, y)).collect();
    // The sand: the top bulb's from its level down to the neck, the pile under the neck, the grains between.
    let sand = m.theme.color("tp");
    let level = -74.0 * rem;
    let steps = 10;
    let ys: Vec<f32> = (0..=steps).map(|k| level + (-2.0 - level) * k as f32 / steps as f32).collect();
    let mut top: Vec<Pos2> = ys.iter().map(|&y| at(half_width(&wall, y).min(-1.5), y)).collect();
    top.extend(ys.iter().rev().map(|&y| at(-half_width(&wall, y).min(-1.5), y)));
    if rem > 0.02 {
        painter.add(Shape::convex_polygon(top, sand, Stroke::NONE));
    }
    let pile = 66.0 * (1.0 - rem);
    if pile > 0.5 {
        let base = -half_width(&wall, 76.0);
        painter.add(Shape::convex_polygon(
            vec![at(-base + 2.0, 76.0), at(0.0, 76.0 - pile - 6.0), at(base - 2.0, 76.0)],
            sand,
            Stroke::NONE,
        ));
    }
    if rem > 0.02 {
        // The stream through the neck: a solid thread (no repeating pattern, which can seem to run upward when frames
        // come slower than it moves), and two grains that each fall the whole way before coming round again.
        let (from, to) = (2.0, 76.0 - pile - 6.0);
        if to > from {
            let thread = Color32::from_rgba_unmultiplied(sand.r(), sand.g(), sand.b(), 170);
            painter.line_segment([at(0.0, from), at(0.0, to)], Stroke::new((1.6 * s).max(1.0), thread));
            let grain = m.theme.color("text");
            for k in 0..2 {
                let f = (t / 0.8 + k as f32 * 0.5).fract();
                let y = from + (to - from) * f;
                painter.circle_filled(at(0.0, y), (3.0 * s).max(1.0), grain.gamma_multiply(1.0 - 0.6 * f));
            }
        }
    }
    // The glass: a soft steel-blue glow under a fine pewter line, then the stepped caps.
    let glow = m.theme.color("focus");
    let pewter = m.theme.color("gold");
    let width = (4.0 * s).max(1.0);
    for wall in [&left_wall, &right_wall] {
        painter.add(Shape::line(wall.clone(), Stroke::new(width * 2.5, glow.gamma_multiply(0.3))));
        painter.add(Shape::line(wall.clone(), Stroke::new(width, pewter)));
    }
    let caps =
        [(-52.0, -83.0, 104.0, 6.0), (-44.0, -89.0, 88.0, 5.0), (-52.0, 77.0, 104.0, 6.0), (-44.0, 84.0, 88.0, 5.0)];
    for (x, y, w, h) in caps {
        painter.rect_filled(Rect::from_min_max(at(x, y), at(x + w, y + h)), CornerRadius::ZERO, pewter);
    }
}

/// The logo's left glass wall, top to bottom, as points (its two curves sampled); x is negative.
fn wall_points() -> Vec<(f32, f32)> {
    let cubic = |p: [(f32, f32); 4], t: f32| {
        let u = 1.0 - t;
        let w = [u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t];
        (p.iter().zip(w).map(|(q, k)| q.0 * k).sum::<f32>(), p.iter().zip(w).map(|(q, k)| q.1 * k).sum::<f32>())
    };
    let upper = [(-40.0, -78.0), (-40.0, -30.0), (-6.0, -14.0), (-3.0, 0.0)];
    let lower = [(-3.0, 0.0), (-6.0, 14.0), (-40.0, 30.0), (-40.0, 78.0)];
    let mut out: Vec<(f32, f32)> = (0..=12).map(|k| cubic(upper, k as f32 / 12.0)).collect();
    out.extend((1..=12).map(|k| cubic(lower, k as f32 / 12.0)));
    out
}

/// The left wall's x at height y (between the sampled points).
fn half_width(wall: &[(f32, f32)], y: f32) -> f32 {
    for p in wall.windows(2) {
        let ((x0, y0), (x1, y1)) = (p[0], p[1]);
        if y1 > y0 && (y0..=y1).contains(&y) {
            return x0 + (x1 - x0) * (y - y0) / (y1 - y0);
        }
    }
    -3.0
}

/// Text with a dark outline, readable over any icon.
pub(super) fn outlined(painter: &egui::Painter, at: Pos2, align: Align2, text: &str, size: f32, col: Color32) {
    let font = FontId::proportional(size);
    for d in [vec2(-1.0, 0.0), vec2(1.0, 0.0), vec2(0.0, -1.0), vec2(0.0, 1.0)] {
        painter.text(at + d, align, text, font.clone(), Color32::from_black_alpha(220));
    }
    painter.text(at, align, text, font, col);
}

/// What a click in a slot's menu asks for.
enum MenuAct {
    Change,
    EditStart,
    /// Name a new gear set from what we wear (1) or rename this one (2).
    GearStart(u8),
    GearUpdate,
    EditSave(String),
    EditCancel,
    Sub(u8),
    Target(Option<String>),
    Copy(u8),
    Clear,
    Close,
}

/// A slot's right-click menu above it: Change Action (the picker), Edit Command, its own Target, Copy to Job, Clear.
/// A click elsewhere or Escape closes it.
fn menu(p: &mut Panels, w: &mut World, opened: bool) {
    let Some((b, k, at)) = w.tl_bar.menu else { return };
    if w.tl_bar.locked {
        w.tl_bar.menu = None;
        w.tl_bar.menu_edit = None;
        return;
    }
    let slot = w.tl_bar.bars[b][k].clone();
    let m = p.m;
    let ctx = p.ctx.clone();
    let job = w.tl_bar.job;
    let shared = w.tl_bar.shared_on[b];
    let jobs: Vec<(u8, Option<String>)> = super::super::timeless::book::ability_jobs()
        .into_iter()
        .filter(|&j| j != job)
        .map(|j| (j, w.tl_bar.job_slot(j, b, k).map(|s| hotbar::name(w, s))))
        .collect();
    let name = slot.as_ref().map(|s| hotbar::name(w, s));
    let sub = w.tl_bar.menu_sub;
    let mode = w.tl_bar.menu_mode;
    let mut edit = w.tl_bar.menu_edit.clone();
    let wearing = !hotbar::current_gear(w).is_empty();
    let mut act = None;
    let text = m.theme.color("text");
    let muted = m.theme.color("muted");
    let shown = egui::Area::new(Id::unique("tl hotbar menu"))
        .order(Order::Foreground)
        .pivot(Align2::LEFT_BOTTOM)
        .fixed_pos(at)
        .show(&ctx, |ui| {
            egui::Frame::NONE
                .fill(m.theme.color("surface"))
                .stroke(Stroke::new(1.0, m.theme.color("bronze")))
                .corner_radius(CornerRadius::same(8))
                .inner_margin(egui::Margin::same(8))
                .show(ui, |ui| {
                    ui.set_width(220.0);
                    ui.spacing_mut().item_spacing.y = 3.0;
                    let title = name.clone().unwrap_or_else(|| "Empty slot".into());
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(title).color(m.theme.color("gold")).size(m.theme.ui));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(egui::RichText::new(hotbar::key_text(b, k)).color(muted).size(m.theme.small));
                        });
                    });
                    ui.separator();
                    let row = |ui: &mut egui::Ui, label: &str| {
                        ui.add_sized(
                            vec2(ui.available_width(), 22.0),
                            egui::Button::new(egui::RichText::new(label).color(text)).frame(false),
                        )
                        .clicked()
                    };
                    if row(ui, if slot.is_some() { "Change Action..." } else { "Choose an Action..." }) {
                        act = Some(MenuAct::Change);
                    }
                    // A line being typed: the command, a new gear set's name, or a set's new name.
                    if let Some(e) = edit.as_mut() {
                        let prompt = match mode {
                            1 => "Name the set (what you wear now):",
                            2 => "Rename the set:",
                            _ => "Command line:",
                        };
                        ui.label(egui::RichText::new(prompt).color(muted).size(m.theme.small));
                        let resp = ui.add(egui::TextEdit::singleline(e).desired_width(f32::INFINITY));
                        resp.request_focus();
                        let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
                        ui.horizontal(|ui| {
                            if ui.button("Save").clicked() || enter {
                                act = Some(MenuAct::EditSave(e.clone()));
                            }
                            if ui.button("Cancel").clicked() {
                                act = Some(MenuAct::EditCancel);
                            }
                        });
                        ui.separator();
                    } else if matches!(slot, Some(Slot::Command { .. })) && row(ui, "Edit Command...") {
                        act = Some(MenuAct::EditStart);
                    }
                    if edit.is_none() {
                        if let Some(Slot::Gear { .. }) = &slot {
                            if wearing && row(ui, "Update to Current Gear") {
                                act = Some(MenuAct::GearUpdate);
                            }
                            if row(ui, "Rename Set...") {
                                act = Some(MenuAct::GearStart(2));
                            }
                        }
                        if wearing && row(ui, "Save Current Gear Here...") {
                            act = Some(MenuAct::GearStart(1));
                        }
                    }
                    if let Some(s) = &slot
                        && hotbar::takes_target(s)
                    {
                        let cur = hotbar::target_tag(s).map(str::to_string);
                        let arrow = if sub == 1 { "v" } else { ">" };
                        if row(ui, &format!("Target: {}  {arrow}", hotbar::target_label(cur.as_deref()))) {
                            act = Some(MenuAct::Sub(if sub == 1 { 0 } else { 1 }));
                        }
                        if sub == 1 {
                            ui.indent("tl menu targets", |ui| {
                                if ui.selectable_label(cur.is_none(), "Automatic").clicked() {
                                    act = Some(MenuAct::Target(None));
                                }
                                for (tag, label) in hotbar::TARGETS {
                                    if ui.selectable_label(cur.as_deref() == Some(tag), format!("{label}  {tag}")).clicked() {
                                        act = Some(MenuAct::Target(Some(tag.to_string())));
                                    }
                                }
                            });
                        }
                    }
                    if slot.is_some() {
                        if shared {
                            ui.add_sized(
                                vec2(ui.available_width(), 22.0),
                                egui::Label::new(
                                    egui::RichText::new("Copy to Job: shared row, on every job").color(muted),
                                ),
                            );
                        } else {
                            let arrow = if sub == 2 { "v" } else { ">" };
                            if row(ui, &format!("Copy to Job  {arrow}")) {
                                act = Some(MenuAct::Sub(if sub == 2 { 0 } else { 2 }));
                            }
                            if sub == 2 {
                                ui.horizontal_wrapped(|ui| {
                                    ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
                                    for (j, there) in &jobs {
                                        let label = hotbar::job_name(*j);
                                        let resp = ui.add_sized(vec2(44.0, 22.0), egui::Button::new(label));
                                        let resp = match there {
                                            Some(n) => resp.on_hover_text(format!("Replaces {n} there.")),
                                            None => resp.on_hover_text("That slot is empty there."),
                                        };
                                        if resp.clicked() {
                                            act = Some(MenuAct::Copy(*j));
                                        }
                                    }
                                });
                            }
                        }
                        if row(ui, "Clear Slot") {
                            act = Some(MenuAct::Clear);
                        }
                    }
                });
        });
    // A press anywhere else closes it (not the right-click that opened it).
    let rect = shown.response.rect;
    if !opened && act.is_none() {
        let (press, pos) = ctx.input(|i| (i.pointer.any_pressed(), i.pointer.interact_pos()));
        if press && pos.is_some_and(|p| !rect.contains(p)) {
            act = Some(MenuAct::Close);
        }
    }
    if let Some(e) = edit
        && w.tl_bar.menu_edit.is_some()
    {
        w.tl_bar.menu_edit = Some(e);
    }
    let Some(act) = act else { return };
    let close = |w: &mut World| {
        w.tl_bar.menu = None;
        w.tl_bar.menu_edit = None;
        w.tl_bar.menu_sub = 0;
        w.tl_bar.menu_mode = 0;
    };
    match act {
        MenuAct::Change => {
            close(w);
            w.tl_bar.picking = Some((b, k));
            w.tl_bar.picker_at = Some(Instant::now());
            w.sounds.push(crate::audio::Ui::Select as u32);
        }
        MenuAct::EditStart => {
            if let Some(Slot::Command { line }) = &slot {
                w.tl_bar.menu_edit = Some(line.clone());
                w.tl_bar.menu_sub = 0;
                w.tl_bar.menu_mode = 0;
            }
            w.sounds.push(crate::audio::Ui::Cursor as u32);
        }
        MenuAct::GearStart(n) => {
            let name = match &slot {
                Some(Slot::Gear { name, .. }) if n == 2 => name.clone(),
                _ => String::new(),
            };
            w.tl_bar.menu_edit = Some(name);
            w.tl_bar.menu_sub = 0;
            w.tl_bar.menu_mode = n;
            w.sounds.push(crate::audio::Ui::Cursor as u32);
        }
        MenuAct::GearUpdate => {
            if let Some(Slot::Gear { name, .. }) = &slot {
                let pieces = hotbar::current_gear(w);
                w.tl_bar.set(b, k, Some(Slot::Gear { name: name.clone(), pieces }));
                w.tl_bar.ready_at[b][k] = Some(Instant::now());
            }
            close(w);
            w.sounds.push(crate::audio::Ui::Select as u32);
        }
        MenuAct::EditSave(text) => {
            let text = text.trim().to_string();
            match mode {
                // A new set: what we wear now, named (a name left empty: "Gear Set").
                1 => {
                    let name = if text.is_empty() { "Gear Set".to_string() } else { text };
                    let pieces = hotbar::current_gear(w);
                    w.tl_bar.set(b, k, Some(Slot::Gear { name, pieces }));
                    w.tl_bar.ready_at[b][k] = Some(Instant::now());
                }
                2 => {
                    if let Some(Slot::Gear { pieces, .. }) = &slot
                        && !text.is_empty()
                    {
                        w.tl_bar.set(b, k, Some(Slot::Gear { name: text, pieces: pieces.clone() }));
                    }
                }
                _ => w.tl_bar.set(b, k, (!text.is_empty()).then_some(Slot::Command { line: text })),
            }
            close(w);
            w.sounds.push(crate::audio::Ui::Select as u32);
        }
        MenuAct::EditCancel => {
            w.tl_bar.menu_edit = None;
            w.tl_bar.menu_mode = 0;
            w.sounds.push(crate::audio::Ui::Cancel as u32);
        }
        MenuAct::Sub(n) => {
            w.tl_bar.menu_sub = n;
            w.sounds.push(crate::audio::Ui::Cursor as u32);
        }
        MenuAct::Target(t) => {
            w.tl_bar.set_target(b, k, t);
            close(w);
            w.sounds.push(crate::audio::Ui::Select as u32);
        }
        MenuAct::Copy(j) => {
            if w.tl_bar.copy_to_job(b, k, j) {
                w.sounds.push(crate::audio::Ui::Select as u32);
            } else {
                w.sounds.push(crate::audio::Ui::Error as u32);
            }
            close(w);
        }
        MenuAct::Clear => {
            w.tl_bar.set(b, k, None);
            close(w);
            w.sounds.push(crate::audio::Ui::Cancel as u32);
        }
        MenuAct::Close => close(w),
    }
}

/// The layout window (the strip's gear): each bar's shape, page and when it shows; the bars' size, opacity and
/// fading while not fighting. Saved as it changes.
fn layout(p: &mut Panels, w: &mut World) {
    if !w.tl_bar.layout {
        return;
    }
    let m = p.m;
    let mut looks = w.tl_bar.looks;
    let mut pages = w.tl_bar.page;
    let mut scale = (w.tl_bar.scale * 100.0).round() as i32;
    let mut opacity = (w.tl_bar.opacity * 100.0).round() as i32;
    let mut fade = w.tl_bar.fade;
    let mut confirm_sp = w.tl_bar.confirm_sp;
    let mut cross_on = w.tl_bar.cross;
    let mut mouseover = w.tl_bar.mouseover;
    let mut close = false;
    let mut import = w.tl_bar.import_text.clone();
    let note = w.tl_bar.note.clone();
    let backups_open = w.tl_bar.backups_open;
    let backups = if backups_open { hotbar::backups(&w.name) } else { Vec::new() };
    let clear_armed = w.tl_bar.clear_armed.is_some_and(|t| t.elapsed().as_secs_f32() < 3.0);
    let locked = w.tl_bar.locked;
    let job = hotbar::job_name(w.tl_bar.job);
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
    enum Share {
        Export,
        ImportOpen,
        ImportApply(String),
        ImportCancel,
        Backups,
        Restore(std::path::PathBuf),
        Clear,
    }
    let mut share = None;
    p.panel("tl_layout", Some(Title { icon: Some("icons/settings"), text: "Hotbar Layout", note: None, name: false }), 380.0, |ui| {
        let muted = m.theme.color("muted");
        for b in 0..BARS {
            ui.label(egui::RichText::new(hotbar::BAR_NAMES[b]).color(m.theme.color("gold")).size(m.theme.ui));
            egui::Grid::new(("tl layout grid", b)).num_columns(2).spacing(vec2(10.0, 4.0)).show(ui, |ui| {
                ui.label(egui::RichText::new("Shape").color(muted));
                ui.horizontal(|ui| {
                    for (cols, label) in [(12u8, "12 x 1"), (6, "6 x 2"), (4, "4 x 3"), (3, "3 x 4")] {
                        if ui.selectable_label(looks[b].cols == cols, label).clicked() {
                            looks[b].cols = cols;
                        }
                    }
                });
                ui.end_row();
                ui.label(egui::RichText::new("Shows").color(muted));
                ui.horizontal(|ui| {
                    for (k, label) in ["Always", "In Combat", "Hidden"].iter().enumerate() {
                        if ui.selectable_label(looks[b].show == k as u8, *label).clicked() {
                            looks[b].show = k as u8;
                        }
                    }
                });
                ui.end_row();
                ui.label(egui::RichText::new("Page").color(muted));
                ui.horizontal(|ui| {
                    for k in 0..hotbar::PAGES {
                        if ui.selectable_label(pages[b] == k, format!("{}", k + 1)).clicked() {
                            pages[b] = k;
                        }
                    }
                });
                ui.end_row();
                ui.label(egui::RichText::new("Keys").color(muted));
                let cur = hotbar::key_set(&looks, b);
                egui::ComboBox::from_id_salt(("tl layout keys", b))
                    .selected_text(hotbar::KEY_SETS[cur as usize].0)
                    .show_ui(ui, |ui| {
                        for (k, (name, _, _)) in hotbar::KEY_SETS.iter().enumerate() {
                            if ui.selectable_label(cur == k as u8, *name).clicked() {
                                looks[b].keys = Some(k as u8);
                            }
                        }
                    });
                ui.end_row();
            });
            let set = hotbar::key_set(&looks, b);
            if set != hotbar::NO_KEYS && (0..b).any(|o| hotbar::key_set(&looks, o) == set) {
                ui.label(
                    egui::RichText::new("Another bar has these keys: only the first gets them.")
                        .color(m.theme.color("hp"))
                        .size(m.theme.small),
                );
            }
            ui.add_space(6.0);
        }
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Size").color(muted));
            ui.add(egui::Slider::new(&mut scale, 75..=150).suffix("%"));
        });
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Opacity").color(muted));
            ui.add(egui::Slider::new(&mut opacity, 30..=100).suffix("%"));
        });
        ui.checkbox(&mut fade, "Fade the bars while not fighting");
        ui.checkbox(&mut confirm_sp, "Two-hours need a double press");
        ui.checkbox(&mut cross_on, "Controller cross hotbar (hold LT: Ctrl bar, RT: Alt bar)");
        ui.checkbox(&mut mouseover, "Mouse-over casting on the party list");
        if (0..BARS).any(|b| matches!(hotbar::key_set(&looks, b), 4 | 5)) {
            ui.label(
                egui::RichText::new("F1 - F12 on a bar take over the game's party targeting keys while the chat line is closed.")
                    .color(muted)
                    .size(m.theme.small),
            );
        }
        ui.separator();
        // Sharing and backups.
        ui.horizontal(|ui| {
            if ui.button(format!("Export {job} Bars")).on_hover_text("Copies a code for every bar and page as this job has them.").clicked() {
                share = Some(Share::Export);
            }
            if import.is_none()
                && ui.add_enabled(!locked, egui::Button::new("Import...")).on_hover_text("Paste a code from Export.").clicked()
            {
                share = Some(Share::ImportOpen);
            }
        });
        if let Some(text) = import.as_mut() {
            ui.label(egui::RichText::new(format!("Paste a code: it replaces {job}'s bars (backed up first).")).color(muted).size(m.theme.small));
            let resp = ui.add(egui::TextEdit::singleline(text).desired_width(f32::INFINITY).hint_text("TLXI1..."));
            if !resp.has_focus() && text.is_empty() {
                resp.request_focus();
            }
            ui.horizontal(|ui| {
                if ui.button("Apply").clicked() {
                    share = Some(Share::ImportApply(text.clone()));
                }
                if ui.button("Cancel").clicked() {
                    share = Some(Share::ImportCancel);
                }
            });
        }
        ui.horizontal(|ui| {
            if ui.selectable_label(backups_open, "Backups...").clicked() {
                share = Some(Share::Backups);
            }
            let label = if clear_armed { "Click again to clear" } else { "Clear This Job's Bars" };
            let b = egui::Button::new(egui::RichText::new(label).color(if clear_armed { m.theme.color("hp") } else { m.theme.color("text") }));
            if ui.add_enabled(!locked, b).on_hover_text(format!("Empties {job}'s own bars, every page (shared bars stay). Backed up first.")).clicked() {
                share = Some(Share::Clear);
            }
        });
        if backups_open {
            if backups.is_empty() {
                ui.label(egui::RichText::new("No backups yet.").color(muted));
            }
            for (path, at) in &backups {
                let ago = now.saturating_sub(*at);
                let when = if ago < 90 {
                    "just now".to_string()
                } else if ago < 5400 {
                    format!("{} min ago", ago / 60)
                } else if ago < 172_800 {
                    format!("{} h ago", ago / 3600)
                } else {
                    format!("{} days ago", ago / 86_400)
                };
                ui.horizontal(|ui| {
                    ui.label(when);
                    if ui.button("Restore").clicked() {
                        share = Some(Share::Restore(path.clone()));
                    }
                });
            }
        }
        if let Some(n) = &note {
            ui.label(egui::RichText::new(n).color(m.theme.color("tp")).size(m.theme.small));
        }
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new("Pages: Shift + 1, 2, 3 for bar 1; Shift + the mouse wheel over any bar.")
                .color(muted)
                .size(m.theme.small),
        );
        ui.add_space(4.0);
        if ui.button("Close").clicked() {
            close = true;
        }
    });
    let h = &mut w.tl_bar;
    let (sc, op) = (scale as f32 / 100.0, opacity as f32 / 100.0);
    let changed = looks != h.looks
        || (sc - h.scale).abs() > 0.001
        || (op - h.opacity).abs() > 0.001
        || fade != h.fade
        || confirm_sp != h.confirm_sp
        || cross_on != h.cross
        || mouseover != h.mouseover;
    if h.import_text.is_some() {
        h.import_text = import;
    }
    for b in 0..BARS {
        if pages[b] != h.page[b] {
            h.set_page(b, pages[b]);
        }
    }
    if changed {
        h.looks = looks;
        h.scale = sc;
        h.opacity = op;
        h.fade = fade;
        h.confirm_sp = confirm_sp;
        h.cross = cross_on;
        h.mouseover = mouseover;
        h.save();
    }
    match share {
        Some(Share::Export) => {
            let code = h.export();
            p.ctx.copy_text(code);
            h.note = Some("Code copied: paste it anywhere to share it.".into());
            w.sounds.push(crate::audio::Ui::Select as u32);
        }
        Some(Share::ImportOpen) => {
            h.import_text = Some(String::new());
            h.note = None;
        }
        Some(Share::ImportApply(code)) => match h.import(&code) {
            Ok(n) => {
                h.import_text = None;
                h.note = Some(format!("Imported {n} slots (your old bars are in Backups)."));
                w.sounds.push(crate::audio::Ui::Select as u32);
            }
            Err(e) => {
                h.note = Some(e);
                w.sounds.push(crate::audio::Ui::Error as u32);
            }
        },
        Some(Share::ImportCancel) => h.import_text = None,
        Some(Share::Backups) => h.backups_open = !h.backups_open,
        Some(Share::Restore(path)) => {
            if h.restore(&path) {
                h.note = Some("Restored (what you had is a backup too).".into());
                w.sounds.push(crate::audio::Ui::Select as u32);
            } else {
                h.note = Some("That backup couldn't be read.".into());
                w.sounds.push(crate::audio::Ui::Error as u32);
            }
        }
        Some(Share::Clear) => {
            if clear_armed {
                h.clear_job();
                h.clear_armed = None;
                h.note = Some(format!("{job}'s bars cleared (they are in Backups)."));
                w.sounds.push(crate::audio::Ui::Cancel as u32);
            } else {
                h.clear_armed = Some(Instant::now());
                w.sounds.push(crate::audio::Ui::Cursor as u32);
            }
        }
        None => {}
    }
    let h = &mut w.tl_bar;
    if close {
        h.layout = false;
        h.import_text = None;
        w.sounds.push(crate::audio::Ui::Cancel as u32);
    }
}

/// The picker for a slot: tabs (Magic, Abilities, Weapon Skills, Items, Commands), the list, Clear and Close.
fn picker(p: &mut Panels, w: &mut World) {
    let Some((b, k)) = w.tl_bar.picking else { return };
    let tab = w.tl_bar.tab.min(TABS.len() - 1);
    let rows = hotbar::choices(w, tab);
    let icons: Vec<Option<(egui::TextureId, Rect)>> = rows.iter().map(|s| icon(p, w, s)).collect();
    let names: Vec<String> = rows.iter().map(|s| hotbar::name(w, s)).collect();
    let usable: Vec<bool> = rows.iter().map(|s| hotbar::usable(w, s)).collect();
    let current = w.tl_bar.bars[b][k].clone();
    let m = p.m;
    let title = format!("Slot {}", hotbar::label(b, k));
    let mut pick_tab = None;
    let mut chosen = None;
    let (mut clear, mut close) = (false, false);
    p.panel(
        "tl_picker",
        Some(Title { icon: None, text: &title, note: Some(hotbar::BAR_NAMES[b]), name: false }),
        380.0,
        |ui| {
            ui.horizontal_wrapped(|ui| {
                for (i, t) in TABS.iter().enumerate() {
                    if ui.selectable_label(i == tab, *t).clicked() {
                        pick_tab = Some(i);
                    }
                }
            });
            ui.add_space(4.0);
            egui::ScrollArea::vertical().max_height(330.0).auto_shrink([false, true]).show(ui, |ui| {
                if rows.is_empty() {
                    let empty = match tab {
                        0 => "No spells your jobs can cast now.",
                        1 => "No abilities for your jobs yet.",
                        2 => "No weapon skills yet.",
                        3 => "No usable items in your inventory or wardrobes.",
                        _ => "Type a command in the chat line first; it shows here.",
                    };
                    ui.label(egui::RichText::new(empty).color(m.theme.color("muted")));
                }
                for (i, s) in rows.iter().enumerate() {
                    let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 30.0), Sense::click());
                    let pt = ui.painter();
                    let on = current.as_ref() == Some(s);
                    if resp.hovered() || on {
                        let c = if on {
                            m.theme.color("focus").gamma_multiply(0.25)
                        } else {
                            Color32::from_white_alpha(14)
                        };
                        pt.rect_filled(r, CornerRadius::same(4), c);
                    }
                    let ir = Rect::from_min_size(r.min + vec2(4.0, 3.0), vec2(24.0, 24.0));
                    match icons[i] {
                        Some((t, uv)) => {
                            pt.image(t, ir, uv, Color32::WHITE);
                        }
                        None => glyph(m, pt, ir.expand(2.0), s),
                    }
                    let col = if usable[i] { m.theme.color("text") } else { m.theme.color("muted") };
                    pt.text(
                        pos2(ir.max.x + 10.0, r.center().y),
                        Align2::LEFT_CENTER,
                        &names[i],
                        FontId::proportional(m.theme.ui),
                        col,
                    );
                    if resp.clicked() {
                        chosen = Some(s.clone());
                    }
                }
            });
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if ui.add_enabled(current.is_some(), egui::Button::new("Clear Slot")).clicked() {
                    clear = true;
                }
                if ui.button("Close").clicked() {
                    close = true;
                }
            });
        },
    );
    if let Some(t) = pick_tab {
        w.tl_bar.tab = t;
        w.sounds.push(crate::audio::Ui::Cursor as u32);
    }
    if let Some(s) = chosen {
        w.tl_bar.set(b, k, Some(s));
        w.tl_bar.picking = None;
        w.sounds.push(crate::audio::Ui::Select as u32);
    } else if clear {
        w.tl_bar.set(b, k, None);
        w.tl_bar.picking = None;
        w.sounds.push(crate::audio::Ui::Cancel as u32);
    } else if close {
        w.tl_bar.picking = None;
        w.sounds.push(crate::audio::Ui::Cancel as u32);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bars_snap_within_reach() {
        let panel = |x: f32, y: f32| {
            // A panel whose backdrop (seen) starts at (x, y) and is 300 x 60.
            let m = super::super::MARGIN;
            let min = pos2(x - (m.left as f32 - 8.0), y - (m.top as f32 - 8.0));
            Rect::from_min_size(min, vec2(300.0 + m.left as f32 + m.right as f32 - 16.0, 60.0 + m.top as f32 + m.bottom as f32 - 16.0))
        };
        let other = seen(panel(100.0, 500.0));
        // Let go 9 px right and just over it with a 10 px gap: lines up on the left, sits SNAP_GAP over it.
        let d = snap_move(panel(109.0, 430.0), &[other]);
        assert_eq!(d, vec2(-9.0, 500.0 - SNAP_GAP - 490.0));
        // Too far on both axes: stays.
        assert_eq!(snap_move(panel(160.0, 300.0), &[other]), Vec2::ZERO);
    }
}
