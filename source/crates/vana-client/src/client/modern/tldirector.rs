//! Timeless XI: the Director window (timeless/director.rs): six tabs (Camera, Shots, Scene, Look, Cast, Capture) and
//! the frame's overlays (letterbox bars, framing guides, the countdown). Every interface; drawn even with the HUD
//! hidden, and tucked away for takes and photos.

use super::super::World;
use super::super::timeless::director::{
    self as dir, AIMS, BARS, EASES, FF_CHECKING, FF_MISSING, FF_OK, FLY, FOLLOW, FREE, GAME, Look, ME, MODES, ORBIT,
    TARGET, WEATHERS,
};
use egui::{Align2, Color32, Context, CornerRadius, FontId, Id, RichText, Stroke, vec2};

fn gold() -> Color32 {
    Color32::from_rgb(214, 225, 240)
}
fn text() -> Color32 {
    Color32::from_rgb(230, 236, 245)
}
fn muted() -> Color32 {
    Color32::from_rgb(152, 163, 184)
}
fn focus() -> Color32 {
    Color32::from_rgb(143, 178, 230)
}
fn sand() -> Color32 {
    Color32::from_rgb(227, 201, 142)
}
fn red() -> Color32 {
    Color32::from_rgb(224, 96, 108)
}

const TABS: [&str; 6] = ["Camera", "Shots", "Scene", "Look", "Cast", "Capture"];

fn heading(ui: &mut egui::Ui, t: &str) {
    ui.add_space(6.0);
    ui.label(RichText::new(t).strong().color(gold()));
}

fn hint(ui: &mut egui::Ui, t: &str) {
    ui.label(RichText::new(t).small().color(muted()));
}

fn row(ui: &mut egui::Ui, label: &str, add: impl FnOnce(&mut egui::Ui)) {
    ui.label(RichText::new(label).color(muted()));
    add(ui);
    ui.end_row();
}

fn check(ui: &mut egui::Ui, v: &mut bool, label: &str) -> egui::Response {
    ui.checkbox(v, RichText::new(label).color(text()))
}

fn hour_text(h: f32) -> String {
    let m = (h.rem_euclid(24.0) * 60.0).round() as u32 % 1440;
    format!("{:02}:{:02}", m / 60, m % 60)
}

/// A filled button in the focus blue (the main action of a row).
fn primary(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(label).strong().color(Color32::from_rgb(10, 11, 18)))
            .fill(focus())
            .corner_radius(CornerRadius::same(4)),
    )
}

fn rgb3(ui: &mut egui::Ui, v: &mut [f32; 3], range: std::ops::RangeInclusive<f32>) {
    ui.horizontal(|ui| {
        for (k, c) in
            [Color32::from_rgb(224, 110, 110), Color32::from_rgb(110, 200, 130), Color32::from_rgb(110, 150, 230)]
                .into_iter()
                .enumerate()
        {
            ui.label(RichText::new("■").color(c));
            ui.add(egui::DragValue::new(&mut v[k]).range(range.clone()).speed(0.002).fixed_decimals(3));
        }
    });
}

impl super::Modern {
    /// The Director window and its overlays.
    pub fn tl_director(&self, ctx: &Context, w: &mut World) {
        if !w.director.open {
            w.director.typing = false;
            return;
        }
        overlays(ctx, w);
        if !w.director.panel_shown() {
            w.director.typing = false;
            return;
        }
        let mut open = true;
        egui::Window::new(RichText::new("Director Mode").color(gold()))
            .id(Id::unique("tl director"))
            .open(&mut open)
            .default_pos([24.0, 80.0])
            .default_size(vec2(390.0, 560.0))
            .resizable(true)
            .collapsible(true)
            .frame(
                egui::Frame::new()
                    .fill(Color32::from_rgba_unmultiplied(17, 20, 29, 240))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(92, 107, 133)))
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(egui::Margin::same(12)),
            )
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for (i, t) in TABS.iter().enumerate() {
                        let on = w.director.tab == i as u8;
                        let label = RichText::new(*t).color(if on { gold() } else { muted() });
                        if ui.add(egui::Button::selectable(on, label)).clicked() {
                            w.director.tab = i as u8;
                        }
                    }
                });
                ui.separator();
                egui::ScrollArea::vertical()
                    .auto_shrink([false, true])
                    .max_height(ctx.content_rect().height() * 0.7)
                    .show(ui, |ui| match w.director.tab {
                        0 => camera_tab(ui, w),
                        1 => shots_tab(ui, w),
                        2 => scene_tab(ui, w),
                        3 => look_tab(ui, w),
                        4 => cast_tab(ui, w),
                        _ => capture_tab(ui, w),
                    });
                ui.separator();
                transport(ui, w);
                if let Some((n, _)) = &w.director.note {
                    ui.label(RichText::new(n).small().color(sand()));
                }
                hint(ui, "Ctrl+Shift: D window · H hide window · F fly · K add key · P play · R record · S photo");
            });
        w.director.typing = ctx.egui_wants_keyboard_input();
        if !open {
            dir::close(w);
        }
    }
}

/// The row under every tab: where the camera is, and play / record / photo.
fn transport(ui: &mut egui::Ui, w: &mut World) {
    if let Some(k) = dir::key_now(w) {
        ui.label(
            RichText::new(format!(
                "{} · zone {} · ({:.1}, {:.1}, {:.1}) · {:.0}° · {:.0} mm",
                MODES[w.director.mode as usize % MODES.len()],
                w.zone.map_or("-".to_string(), |z| z.to_string()),
                k.eye[0],
                -k.eye[1],
                k.eye[2],
                k.fov,
                dir::focal_mm(k.fov)
            ))
            .small()
            .monospace()
            .color(muted()),
        );
    }
    ui.horizontal(|ui| {
        let playing = w.director.playing.is_some() || w.director.countdown.is_some_and(|c| c.1 == dir::Cue::Play);
        if ui.button(if playing { "■ Stop" } else { "▶ Play shot" }).clicked() {
            dir::play_toggle(w);
        }
        let rec = w.director.recording.is_some();
        let label = if rec {
            RichText::new("■ Stop recording").color(red())
        } else {
            RichText::new("● Record").color(red())
        };
        if ui.button(label).clicked() {
            dir::record_toggle(w);
        }
        if ui.button("📷 Photo").clicked() {
            dir::photo(w);
        }
    });
}

// -----------------------------------------------------------------------------------------------------------------
// Camera

fn camera_tab(ui: &mut egui::Ui, w: &mut World) {
    heading(ui, "Camera");
    let mut mode = w.director.mode;
    ui.horizontal_wrapped(|ui| {
        for (i, m) in MODES.iter().enumerate() {
            ui.selectable_value(&mut mode, i as u8, RichText::new(*m).color(text()));
        }
    });
    if mode != w.director.mode {
        dir::set_mode(w, mode);
    }
    match w.director.mode {
        GAME => hint(ui, "The usual game camera. Pick Free fly, Orbit or Follow to take the camera."),
        FLY => hint(
            ui,
            "WASD fly · E / Page Up rise · Q / Page Down sink · Shift fast · Ctrl slow · drag to look · middle-drag pan · wheel speed · arrows turn",
        ),
        ORBIT => hint(ui, "Drag to circle and tilt · wheel closer / farther · middle-drag raises the centre."),
        _ => hint(
            ui,
            "Rides beside the subject as it moves. Drag to move the camera's side and height · wheel distance.",
        ),
    }
    let mut back_to_me = false;
    let mut orbit_point = false;
    let has_target = w.target.is_some();
    let d = &mut w.director;
    egui::Grid::new("tl dir lens").num_columns(2).spacing(vec2(10.0, 6.0)).show(ui, |ui| {
        row(ui, "Field of view", |ui| {
            ui.add(egui::Slider::new(&mut d.rig.fov, 8.0..=110.0).suffix("°").fixed_decimals(0));
        });
        let mm = dir::focal_mm(d.rig.fov);
        row(ui, "Lens", |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("{mm:.0} mm")).color(text()).monospace());
                for (lab, f) in
                    [("14", 14.0f32), ("24", 24.0), ("35", 35.0), ("50", 50.0), ("85", 85.0), ("135", 135.0)]
                {
                    if ui.small_button(lab).clicked() {
                        d.rig.fov = (2.0 * (12.0 / f).atan()).to_degrees();
                    }
                }
            });
        });
        row(ui, "Dutch roll", |ui| {
            ui.horizontal(|ui| {
                ui.add(egui::Slider::new(&mut d.rig.roll, -45.0..=45.0).suffix("°").fixed_decimals(1));
                if ui.small_button("Level").clicked() {
                    d.rig.roll = 0.0;
                }
            });
        });
        row(ui, "Aim lock", |ui| {
            egui::ComboBox::from_id_salt("tl dir aim").selected_text(AIMS[d.aim as usize % 3]).show_ui(ui, |ui| {
                for (i, a) in AIMS.iter().enumerate() {
                    ui.selectable_value(&mut d.aim, i as u8, *a);
                }
            });
        });
        row(ui, "Handheld shake", |ui| {
            ui.add(egui::Slider::new(&mut d.shake.amount, 0.0..=4.0).suffix("°").fixed_decimals(2));
        });
        row(ui, "Shake pace", |ui| {
            ui.add(egui::Slider::new(&mut d.shake.speed, 0.1..=3.0).fixed_decimals(2));
        });
    });
    hint(ui, "Aim lock keeps the free camera and played shots pointed at the subject: tracking shots with no effort.");
    match d.mode {
        FLY => {
            heading(ui, "Flying");
            egui::Grid::new("tl dir fly").num_columns(2).spacing(vec2(10.0, 6.0)).show(ui, |ui| {
                row(ui, "Speed", |ui| {
                    ui.add(
                        egui::Slider::new(&mut d.prefs.speed, 0.1..=200.0)
                            .logarithmic(true)
                            .suffix(" y/s")
                            .fixed_decimals(1),
                    );
                });
                row(ui, "Glide", |ui| {
                    ui.add(egui::Slider::new(&mut d.prefs.smoothing, 0.0..=0.95).fixed_decimals(2));
                });
                row(ui, "Look smoothing", |ui| {
                    ui.add(egui::Slider::new(&mut d.prefs.look_smoothing, 0.0..=0.9).fixed_decimals(2));
                });
            });
            check(ui, &mut d.prefs.level_flight, "Level flight (W / S stay at the same height)");
            check(ui, &mut d.prefs.hold_character, "My character stays put while I fly (off: WASD walks it too)");
            if ui.button("Back to my character").clicked() {
                back_to_me = true;
            }
        }
        ORBIT => {
            heading(ui, "Orbit");
            let o = &mut d.orbit;
            let before = o.subject;
            egui::Grid::new("tl dir orbit").num_columns(2).spacing(vec2(10.0, 6.0)).show(ui, |ui| {
                row(ui, "Around", |ui| {
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut o.subject, ME, "Me");
                        ui.selectable_value(&mut o.subject, TARGET, "My target");
                        ui.selectable_value(&mut o.subject, FREE, "A fixed point");
                    });
                });
                row(ui, "Distance", |ui| {
                    ui.add(
                        egui::Slider::new(&mut o.dist, 0.5..=200.0).logarithmic(true).suffix(" y").fixed_decimals(1),
                    );
                });
                row(ui, "Height", |ui| {
                    ui.add(egui::Slider::new(&mut o.height, -3.0..=30.0).suffix(" y").fixed_decimals(1));
                });
                row(ui, "Angle", |ui| {
                    ui.add(egui::Slider::new(&mut o.yaw, 0.0..=std::f32::consts::TAU).show_value(false));
                });
                row(ui, "Tilt", |ui| {
                    ui.add(egui::Slider::new(&mut o.pitch, -1.4..=1.5).show_value(false));
                });
                row(ui, "Auto-turn", |ui| {
                    ui.add(egui::Slider::new(&mut o.spin, -60.0..=60.0).suffix("°/s").fixed_decimals(0));
                });
            });
            orbit_point = o.subject == FREE && before != FREE;
            if o.subject == FREE {
                hint(ui, "A fixed point: the orbit circles what the camera was looking at when you switched to Orbit.");
            }
            if o.subject == TARGET && !has_target {
                hint(ui, "Target someone to orbit them.");
            }
        }
        FOLLOW => {
            heading(ui, "Follow");
            let f = &mut d.follow;
            egui::Grid::new("tl dir follow").num_columns(2).spacing(vec2(10.0, 6.0)).show(ui, |ui| {
                row(ui, "Follow", |ui| {
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut f.subject, ME, "Me");
                        ui.selectable_value(&mut f.subject, TARGET, "My target");
                    });
                });
                row(ui, "Side", |ui| {
                    ui.add(egui::Slider::new(&mut f.side, -15.0..=15.0).suffix(" y").fixed_decimals(1));
                });
                row(ui, "Height", |ui| {
                    ui.add(egui::Slider::new(&mut f.up, -2.0..=15.0).suffix(" y").fixed_decimals(1));
                });
                row(ui, "Behind", |ui| {
                    ui.add(egui::Slider::new(&mut f.back, -20.0..=40.0).suffix(" y").fixed_decimals(1));
                });
                row(ui, "Look ahead", |ui| {
                    ui.add(egui::Slider::new(&mut f.ahead, -5.0..=15.0).suffix(" y").fixed_decimals(1));
                });
                row(ui, "Catch-up", |ui| {
                    ui.add(egui::Slider::new(&mut f.ease, 0.3..=12.0).logarithmic(true).fixed_decimals(1));
                });
            });
            ui.horizontal_wrapped(|ui| {
                for (lab, s, u, b, a) in [
                    ("Over the shoulder", 0.7f32, 1.7f32, 2.2f32, 6.0f32),
                    ("Side tracking", 5.0, 1.2, 0.0, 0.5),
                    ("Leading (front)", 0.5, 1.4, -4.0, -1.0),
                    ("Drone", 0.0, 9.0, 8.0, 2.0),
                ] {
                    if ui.small_button(lab).clicked() {
                        (f.side, f.up, f.back, f.ahead) = (s, u, b, a);
                    }
                }
            });
        }
        _ => {}
    }
    if orbit_point {
        dir::orbit_here(w);
    }
    if back_to_me && let Some(c) = behind_me(w) {
        let fov = w.director.rig.fov;
        w.director.rig = dir::Rig::from_camera(&c);
        w.director.rig.fov = fov;
    }
    heading(ui, "Bookmarks");
    hint(ui, "Click a number to jump there · Set saves the camera's place in it.");
    egui::Grid::new("tl dir marks").num_columns(9).spacing(vec2(4.0, 4.0)).show(ui, |ui| {
        let zone = w.zone;
        for i in 0..9 {
            let m = w.director.prefs.bookmarks.get(i).copied().flatten();
            let here = m.is_some_and(|(z, _)| z == zone);
            let label = RichText::new(format!("{}", i + 1)).color(if here { gold() } else { muted() });
            let r = ui.add_enabled(here, egui::Button::new(label).min_size(vec2(30.0, 22.0)));
            if r.clicked()
                && let Some((_, k)) = m
            {
                dir::go_to(w, k);
            }
        }
        ui.end_row();
        for i in 0..9 {
            if ui.add(egui::Button::new(RichText::new("Set").small()).min_size(vec2(30.0, 18.0))).clicked() {
                dir::set_bookmark(w, i);
            }
        }
        ui.end_row();
    });
}

/// The game camera's place behind our character (the last frame the game camera drew is not kept once the director
/// has the view, so this is worked out from the character).
fn behind_me(w: &World) -> Option<crate::render::Camera> {
    let p = super::super::drawn_pos(w);
    let f = glam::Vec3::new(w.yaw.sin(), 0.0, w.yaw.cos());
    let look = p + glam::Vec3::NEG_Y * 1.2;
    let eye = look - f * 5.0 + glam::Vec3::NEG_Y * 1.0;
    Some(crate::render::Camera { eye, target: look, up: glam::Vec3::NEG_Y, fov_y: w.director.rig.fov.to_radians() })
}

// -----------------------------------------------------------------------------------------------------------------
// Shots

fn shots_tab(ui: &mut egui::Ui, w: &mut World) {
    heading(ui, "Shot");
    hint(ui, "Place the camera, add a key, move, add the next. Play glides through them on a smooth curve.");
    ui.horizontal(|ui| {
        if primary(ui, "+ Add key here").clicked() {
            dir::add_key(w);
        }
        let n = w.director.keys.len();
        if ui.add_enabled(n > 0, egui::Button::new("Clear all")).clicked() {
            w.director.keys.clear();
            w.director.selected = None;
            w.director.playing = None;
        }
        check(ui, &mut w.director.show_path, "Show path");
    });
    let d = &mut w.director;
    egui::Grid::new("tl dir play").num_columns(2).spacing(vec2(10.0, 6.0)).show(ui, |ui| {
        row(ui, "Easing", |ui| {
            egui::ComboBox::from_id_salt("tl dir ease").selected_text(EASES[d.ease as usize % 3]).show_ui(ui, |ui| {
                for (i, e) in EASES.iter().enumerate() {
                    ui.selectable_value(&mut d.ease, i as u8, *e);
                }
            });
        });
        row(ui, "Playback speed", |ui| {
            ui.add(egui::Slider::new(&mut d.play_speed, 0.1..=4.0).logarithmic(true).suffix("×").fixed_decimals(2));
        });
        row(ui, "Loop", |ui| {
            ui.checkbox(&mut d.looped, RichText::new("Back to the first key and around").color(text()));
        });
        row(ui, "Length", |ui| {
            ui.label(RichText::new(format!("{:.1} s at 1×", d.path_seconds())).color(text()));
        });
    });
    // The timeline scrubber.
    let total = w.director.path_seconds();
    if w.director.keys.len() >= 2 && total > 0.0 {
        let mut t = w.director.scrub.or(w.director.playing).unwrap_or(0.0);
        let r = ui.add(egui::Slider::new(&mut t, 0.0..=total).text("Scrub").fixed_decimals(1).suffix(" s"));
        if r.changed() || r.dragged() {
            // Scrubbing shows the frame at that time while the slider is held.
            w.director.playing = None;
            w.director.scrub = Some(t);
        }
        if w.director.scrub.is_some() && !r.dragged() {
            // Let go: the free camera stays on that frame.
            let k = dir::sample(&w.director.keys, t, w.director.ease, w.director.looped);
            w.director.scrub = None;
            if let Some(k) = k {
                dir::go_to(w, k);
            }
        }
    }
    heading(ui, "Keys");
    let mut action: Option<(usize, u8)> = None;
    let n = w.director.keys.len();
    if n == 0 {
        hint(ui, "No keys yet.");
    }
    egui::Grid::new("tl dir keys").num_columns(5).striped(true).spacing(vec2(6.0, 4.0)).show(ui, |ui| {
        if n > 0 {
            ui.label(RichText::new("#").small().color(muted()));
            ui.label(RichText::new("Hold").small().color(muted()));
            ui.label(RichText::new("Travel").small().color(muted()));
            ui.label(RichText::new("Lens").small().color(muted()));
            ui.label("");
            ui.end_row();
        }
        for i in 0..n {
            let sel = w.director.selected == Some(i);
            let label = RichText::new(format!("{}", i + 1)).color(if sel { gold() } else { text() });
            if ui.add(egui::Button::selectable(sel, label)).clicked() {
                w.director.selected = if sel { None } else { Some(i) };
            }
            let k = &mut w.director.keys[i];
            ui.add(egui::DragValue::new(&mut k.hold).range(0.0..=60.0).speed(0.05).suffix(" s").fixed_decimals(1));
            let last = i + 1 == n && !w.director.looped;
            ui.add_enabled(
                !last,
                egui::DragValue::new(&mut k.secs).range(0.1..=600.0).speed(0.05).suffix(" s").fixed_decimals(1),
            );
            ui.label(RichText::new(format!("{:.0} mm", dir::focal_mm(k.fov))).small().color(muted()));
            ui.horizontal(|ui| {
                if ui.small_button("Go").on_hover_text("Put the camera on this key").clicked() {
                    action = Some((i, 0));
                }
                if ui.small_button("Set").on_hover_text("Move this key to the camera").clicked() {
                    action = Some((i, 1));
                }
                if ui.add_enabled(i > 0, egui::Button::new("▲").small()).clicked() {
                    action = Some((i, 2));
                }
                if ui.add_enabled(i + 1 < n, egui::Button::new("▼").small()).clicked() {
                    action = Some((i, 3));
                }
                if ui.small_button("✕").on_hover_text("Delete this key").clicked() {
                    action = Some((i, 4));
                }
            });
            ui.end_row();
        }
    });
    if let Some((i, a)) = action {
        match a {
            0 => {
                let k = w.director.keys[i];
                dir::go_to(w, k);
                w.director.selected = Some(i);
            }
            1 => {
                if let Some(now) = dir::key_now(w) {
                    let k = &mut w.director.keys[i];
                    *k = dir::Key { secs: k.secs, hold: k.hold, ..now };
                    w.director.say(format!("Key {} moved to the camera.", i + 1));
                }
            }
            2 => {
                w.director.keys.swap(i, i - 1);
                w.director.selected = Some(i - 1);
            }
            3 => {
                w.director.keys.swap(i, i + 1);
                w.director.selected = Some(i + 1);
            }
            _ => {
                w.director.keys.remove(i);
                w.director.selected = None;
            }
        }
    }
    if n >= 2 {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Every key's travel").color(muted()));
            for s in [2.0f32, 4.0, 6.0, 10.0] {
                if ui.small_button(format!("{s:.0} s")).clicked() {
                    for k in &mut w.director.keys {
                        k.secs = s;
                    }
                }
            }
        });
    }
    heading(ui, "Save and load");
    ui.horizontal(|ui| {
        ui.label(RichText::new("Name").color(muted()));
        ui.add(egui::TextEdit::singleline(&mut w.director.shot_name).hint_text("Lufaise dawn").desired_width(170.0));
        if ui.add_enabled(!w.director.keys.is_empty(), egui::Button::new("Save")).clicked() {
            dir::save_shot(w);
        }
    });
    hint(ui, "A saved shot keeps its keys and the whole scene: time, weather, look, letterbox and who is hidden.");
    let mut load: Option<String> = None;
    let mut delete: Option<String> = None;
    if w.director.library.is_empty() {
        hint(ui, "No saved shots yet.");
    }
    egui::Grid::new("tl dir lib").num_columns(2).striped(true).show(ui, |ui| {
        for name in &w.director.library {
            ui.label(RichText::new(name).color(text()));
            ui.horizontal(|ui| {
                if ui.small_button("Load").clicked() {
                    load = Some(name.clone());
                }
                if ui.small_button("Delete").clicked() {
                    delete = Some(name.clone());
                }
            });
            ui.end_row();
        }
    });
    if let Some(n) = load {
        dir::load_shot(w, &n);
    }
    if let Some(n) = delete {
        dir::delete_shot(w, &n);
    }
    ui.horizontal(|ui| {
        if ui.small_button("Open shots folder").clicked() {
            let p = dir::shots_dir();
            let _ = std::fs::create_dir_all(&p);
            open_folder(&p);
        }
        if ui.small_button("Refresh").clicked() {
            w.director.refresh_library();
        }
    });
    heading(ui, "Title screen");
    hint(
        ui,
        "Send this shot to your title reel: the title screen plays it behind the logo, at the scene's hour and weather.",
    );
    if ui.add_enabled(w.director.keys.len() >= 2, egui::Button::new("Add to my title reel")).clicked() {
        dir::export_reel(w);
    }
}

fn open_folder(p: &std::path::Path) {
    #[cfg(windows)]
    let _ = std::process::Command::new("explorer").arg(p).spawn();
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(p).spawn();
    #[cfg(all(unix, not(target_os = "macos")))]
    let _ = std::process::Command::new("xdg-open").arg(p).spawn();
}

// -----------------------------------------------------------------------------------------------------------------
// Scene

fn scene_tab(ui: &mut egui::Ui, w: &mut World) {
    let server_hour =
        w.clock.map_or(12.0, |(t, at)| crate::dat::weather::vana_minute(t, at.elapsed().as_secs_f32()) / 60.0);
    let server_weather = w.weather;
    let d = &mut w.director;
    heading(ui, "Time of day");
    hint(ui, "Only your screen changes: the server's clock and everyone else's game are untouched.");
    let s = &mut d.prefs.scene;
    let was = (s.time_on, s.hour);
    check(ui, &mut s.time_on, "Set the hour");
    ui.add_enabled_ui(s.time_on, |ui| {
        ui.horizontal(|ui| {
            ui.add(egui::Slider::new(&mut s.hour, 0.0..=24.0).show_value(false));
            ui.label(
                RichText::new(hour_text(if s.lapse != 0.0 { d.lapse_hour } else { s.hour })).monospace().color(text()),
            );
        });
        ui.horizontal_wrapped(|ui| {
            for (lab, h) in [
                ("Dawn", 6.0f32),
                ("Morning", 9.0),
                ("Noon", 12.0),
                ("Golden", 17.0),
                ("Dusk", 18.5),
                ("Blue hour", 19.3),
                ("Night", 22.0),
                ("Midnight", 0.0),
            ] {
                if ui.small_button(lab).clicked() {
                    s.hour = h;
                }
            }
        });
        egui::Grid::new("tl dir lapse").num_columns(2).spacing(vec2(10.0, 6.0)).show(ui, |ui| {
            row(ui, "Time-lapse", |ui| {
                ui.add(egui::Slider::new(&mut s.lapse, -6.0..=6.0).suffix(" h/s").fixed_decimals(2));
            });
        });
        ui.horizontal(|ui| {
            for (lab, v) in [("Still", 0.0f32), ("Slow", 0.1), ("Day in 1 min", 0.4), ("Fast", 2.0)] {
                if ui.small_button(lab).clicked() {
                    s.lapse = v;
                }
            }
        });
    });
    if !s.time_on {
        hint(ui, &format!("Server time now {}", hour_text(server_hour)));
    }
    if (s.time_on, s.hour) != was {
        d.lapse_hour = s.hour;
    }
    check(ui, &mut s.moon_on, "Set the moon");
    ui.add_enabled_ui(s.moon_on, |ui| {
        ui.horizontal(|ui| {
            ui.add(egui::Slider::new(&mut s.moon, 0..=11).text("phase"));
        });
    });
    heading(ui, "Weather");
    check(ui, &mut s.weather_on, "Set the weather");
    ui.add_enabled_ui(s.weather_on, |ui| {
        egui::Grid::new("tl dir wx").num_columns(4).spacing(vec2(4.0, 4.0)).show(ui, |ui| {
            for (i, name) in WEATHERS.iter().enumerate() {
                let on = s.weather == i as u16;
                if ui
                    .add(egui::Button::selectable(
                        on,
                        RichText::new(*name).small().color(if on { gold() } else { text() }),
                    ))
                    .clicked()
                {
                    s.weather = i as u16;
                }
                if i % 4 == 3 {
                    ui.end_row();
                }
            }
        });
        ui.add(egui::Slider::new(&mut s.blend, 0.0..=30.0).text("change over").suffix(" s").fixed_decimals(0));
    });
    if !s.weather_on {
        hint(ui, &format!("Server weather now: {}", WEATHERS.get(server_weather as usize).copied().unwrap_or("?")));
    }
    hint(ui, "A zone shows only the weathers its sky has; others fall back to its nearest.");
    heading(ui, "Scenery");
    check(ui, &mut s.all_scenery, "Draw all of the zone's scenery");
    hint(
        ui,
        "For high or far cameras: the game hides scenery a player on the ground can't see, which leaves holes from the sky.",
    );
}

// -----------------------------------------------------------------------------------------------------------------
// Look

fn look_tab(ui: &mut egui::Ui, w: &mut World) {
    let d = &mut w.director;
    heading(ui, "Grade");
    let l = &mut d.prefs.scene.look;
    check(ui, &mut l.on, "Use a look");
    let presets = dir::look_presets();
    ui.horizontal_wrapped(|ui| {
        for (i, (name, p)) in presets.iter().enumerate() {
            let on = l.preset == i as u8;
            if ui
                .add(egui::Button::selectable(on, RichText::new(*name).color(if on { gold() } else { text() })))
                .clicked()
            {
                *l = Look { on: true, preset: i as u8, ..*p };
            }
        }
    });
    hint(
        ui,
        "Memory, Film and Dusk are the title screen's own looks. Every slider below can be tuned from any of them.",
    );
    ui.add_enabled_ui(l.on, |ui| {
        egui::Grid::new("tl dir grade").num_columns(2).spacing(vec2(10.0, 6.0)).show(ui, |ui| {
            row(ui, "Saturation", |ui| {
                ui.add(egui::Slider::new(&mut l.saturation, 0.0..=1.8).fixed_decimals(2));
            });
            row(ui, "Contrast", |ui| {
                ui.add(egui::Slider::new(&mut l.contrast, 0.6..=1.6).fixed_decimals(2));
            });
            row(ui, "Vignette", |ui| {
                ui.add(egui::Slider::new(&mut l.vignette, 0.0..=1.0).fixed_decimals(2));
            });
            row(ui, "Glow (halation)", |ui| {
                ui.add(egui::Slider::new(&mut l.halation, 0.0..=1.0).fixed_decimals(2));
            });
            row(ui, "Film grain", |ui| {
                ui.add(egui::Slider::new(&mut l.grain, 0.0..=0.15).fixed_decimals(3));
            });
            row(ui, "Soft edges", |ui| {
                ui.add(egui::Slider::new(&mut l.soft_edges, 0.0..=1.5).fixed_decimals(2));
            });
            row(ui, "Gate weave", |ui| {
                ui.add(egui::Slider::new(&mut l.weave, 0.0..=1.0).fixed_decimals(2));
            });
            row(ui, "Flicker", |ui| {
                ui.add(egui::Slider::new(&mut l.flicker, 0.0..=0.08).fixed_decimals(3));
            });
        });
        egui::CollapsingHeader::new(RichText::new("Colour").color(text())).id_salt("tl dir colour").show(ui, |ui| {
            egui::Grid::new("tl dir tints").num_columns(2).spacing(vec2(10.0, 6.0)).show(ui, |ui| {
                row(ui, "Lifted blacks", |ui| rgb3(ui, &mut l.lift, 0.0..=0.2));
                row(ui, "Shadow tint", |ui| rgb3(ui, &mut l.shadow_tint, -0.1..=0.1));
                row(ui, "Highlight tint", |ui| rgb3(ui, &mut l.high_tint, -0.1..=0.1));
                row(ui, "Glow colour", |ui| rgb3(ui, &mut l.halation_tint, 0.0..=1.0));
            });
            ui.horizontal(|ui| {
                if ui.small_button("Warm").clicked() {
                    (l.shadow_tint, l.high_tint) = ([0.01, 0.0, -0.02], [0.05, 0.025, -0.03]);
                }
                if ui.small_button("Cool").clicked() {
                    (l.shadow_tint, l.high_tint) = ([-0.02, 0.0, 0.04], [-0.01, 0.0, 0.03]);
                }
                if ui.small_button("Teal and orange").clicked() {
                    (l.shadow_tint, l.high_tint) = ([-0.03, 0.01, 0.04], [0.06, 0.02, -0.04]);
                }
                if ui.small_button("No tint").clicked() {
                    (l.shadow_tint, l.high_tint, l.lift) = ([0.0; 3], [0.0; 3], [0.0; 3]);
                }
            });
        });
    });
    heading(ui, "Frame");
    let s = &mut d.prefs.scene;
    egui::ComboBox::from_id_salt("tl dir bars").selected_text(BARS[s.bars as usize % BARS.len()].0).show_ui(ui, |ui| {
        for (i, (name, _)) in BARS.iter().enumerate() {
            ui.selectable_value(&mut s.bars, i as u8, *name);
        }
    });
    hint(ui, "Letterbox bars are part of the picture: they show in photos and videos.");
    heading(ui, "Framing guides");
    hint(ui, "For lining up shots only: never in a photo, a video or a playing shot.");
    ui.horizontal_wrapped(|ui| {
        check(ui, &mut d.prefs.guides_thirds, "Rule of thirds");
        check(ui, &mut d.prefs.guides_center, "Centre cross");
        check(ui, &mut d.prefs.guides_safe, "Title safe");
    });
}

// -----------------------------------------------------------------------------------------------------------------
// Cast

fn cast_tab(ui: &mut egui::Ui, w: &mut World) {
    heading(ui, "Who is on screen");
    hint(ui, "Hidden only on your screen.");
    let s = &mut w.director.prefs.scene;
    check(ui, &mut s.hide_hud, "Hide the game's HUD, windows and names (Scroll Lock does too)");
    check(ui, &mut s.hide_me, "Hide my character");
    check(ui, &mut s.hide_players, "Hide other players");
    ui.add_enabled_ui(s.hide_players, |ui| {
        ui.horizontal(|ui| {
            ui.add_space(20.0);
            check(ui, &mut s.keep_party, "…but keep my party");
        });
    });
    check(ui, &mut s.hide_companions, "Hide Companions");
    check(ui, &mut s.hide_npcs, "Hide NPCs");
    check(ui, &mut s.hide_mobs, "Hide monsters");
    ui.horizontal(|ui| {
        if ui.small_button("Empty world").clicked() {
            (s.hide_players, s.keep_party, s.hide_companions, s.hide_npcs, s.hide_mobs) =
                (true, false, true, true, true);
        }
        if ui.small_button("Just my party").clicked() {
            (s.hide_players, s.keep_party, s.hide_npcs, s.hide_mobs) = (true, true, true, false);
        }
        if ui.small_button("Show everyone").clicked() {
            (s.hide_me, s.hide_players, s.hide_companions, s.hide_npcs, s.hide_mobs) =
                (false, false, false, false, false);
        }
    });
    heading(ui, "Pose");
    hint(ui, "Emotes for your character; everyone nearby sees these as usual.");
    let mut send: Option<&str> = None;
    egui::Grid::new("tl dir emotes").num_columns(4).spacing(vec2(4.0, 4.0)).show(ui, |ui| {
        let list = [
            ("Wave", "/wave motion"),
            ("Bow", "/bow motion"),
            ("Salute", "/salute motion"),
            ("Kneel", "/kneel motion"),
            ("Cheer", "/cheer motion"),
            ("Clap", "/clap motion"),
            ("Point", "/point motion"),
            ("Think", "/think motion"),
            ("Joy", "/joy motion"),
            ("Panic", "/panic motion"),
            ("Dance 1", "/dance1 motion"),
            ("Dance 2", "/dance2 motion"),
            ("Dance 3", "/dance3 motion"),
            ("Dance 4", "/dance4 motion"),
            ("Sit", "/sit"),
            ("Rest", "/heal"),
        ];
        for (i, (lab, cmd)) in list.iter().enumerate() {
            if ui.add(egui::Button::new(RichText::new(*lab).small()).min_size(vec2(78.0, 20.0))).clicked() {
                send = Some(cmd);
            }
            if i % 4 == 3 {
                ui.end_row();
            }
        }
    });
    if let Some(cmd) = send {
        super::super::send_chat(w, cmd.to_string());
    }
}

// -----------------------------------------------------------------------------------------------------------------
// Capture

fn capture_tab(ui: &mut egui::Ui, w: &mut World) {
    use std::sync::atomic::Ordering;
    let ff = w.director.ffmpeg.load(Ordering::Relaxed);
    heading(ui, "Video");
    match ff {
        FF_OK => hint(ui, "Records the screen as you see it (letterbox and look included) to an MP4."),
        FF_CHECKING => hint(ui, "Checking for ffmpeg…"),
        FF_MISSING => {
            ui.label(
                RichText::new("Video needs ffmpeg, which isn't on this PC. Install it (winget install ffmpeg in a terminal), then restart Timeless XI. OBS works too.")
                    .small()
                    .color(sand()),
            );
        }
        _ => {}
    }
    ui.add_enabled_ui(ff != FF_MISSING, |ui| {
        ui.horizontal(|ui| {
            let rec = w.director.recording.is_some();
            if ui
                .button(if rec {
                    RichText::new("■ Stop recording").color(red())
                } else {
                    RichText::new("● Record").color(red())
                })
                .clicked()
            {
                dir::record_toggle(w);
            }
            if ui
                .add_enabled(!rec && w.director.keys.len() >= 2, egui::Button::new("● Film the shot"))
                .on_hover_text("Records while the shot plays once, then stops by itself.")
                .clicked()
            {
                dir::record_start(w, true);
            }
        });
    });
    let d = &mut w.director;
    egui::Grid::new("tl dir cap").num_columns(2).spacing(vec2(10.0, 6.0)).show(ui, |ui| {
        row(ui, "Frame rate", |ui| {
            ui.horizontal(|ui| {
                for f in [30u32, 60] {
                    ui.selectable_value(&mut d.prefs.fps, f, format!("{f} fps"));
                }
            });
        });
        row(ui, "Countdown", |ui| {
            ui.add(egui::Slider::new(&mut d.prefs.countdown, 0..=10).suffix(" s"));
        });
    });
    check(ui, &mut d.prefs.auto_hide, "Hide this window while a shot plays or records");
    if let Some((p, at, _)) = &d.recording {
        ui.label(
            RichText::new(format!("Recording {:.0} s → {}", at.elapsed().as_secs_f32(), p.display()))
                .small()
                .color(red()),
        );
    }
    heading(ui, "Photo");
    hint(ui, "A full-size PNG with this window and the HUD left out.");
    if ui.button("📷 Take photo").clicked() {
        dir::photo(w);
    }
    heading(ui, "Files");
    ui.horizontal(|ui| {
        if ui.small_button("Open videos folder").clicked() {
            let p = dir::video_dir();
            let _ = std::fs::create_dir_all(&p);
            open_folder(&p);
        }
        if ui.small_button("Open photos folder").clicked() {
            let p = dir::photo_dir();
            let _ = std::fs::create_dir_all(&p);
            open_folder(&p);
        }
    });
    hint(ui, "File names start with the shot's name from the Shots tab.");
}

// -----------------------------------------------------------------------------------------------------------------
// Overlays

/// Letterbox bars, framing guides and the countdown, behind every window.
fn overlays(ctx: &Context, w: &World) {
    let d = &w.director;
    let screen = ctx.content_rect();
    let p = ctx.layer_painter(egui::LayerId::new(egui::Order::Background, Id::unique("tl director overlay")));
    // The picture's frame: the letterbox's inside, or the whole screen.
    let ratio = BARS[d.prefs.scene.bars as usize % BARS.len()].1;
    let mut frame = screen;
    if ratio > 0.0 {
        let sr = screen.width() / screen.height().max(1.0);
        if ratio > sr {
            let h = screen.width() / ratio;
            let bar = ((screen.height() - h) * 0.5).max(0.0);
            p.rect_filled(egui::Rect::from_min_size(screen.min, vec2(screen.width(), bar)), 0.0, Color32::BLACK);
            p.rect_filled(
                egui::Rect::from_min_size(egui::pos2(screen.min.x, screen.max.y - bar), vec2(screen.width(), bar)),
                0.0,
                Color32::BLACK,
            );
            frame = egui::Rect::from_min_max(
                egui::pos2(screen.min.x, screen.min.y + bar),
                egui::pos2(screen.max.x, screen.max.y - bar),
            );
        } else {
            let wd = screen.height() * ratio;
            let bar = ((screen.width() - wd) * 0.5).max(0.0);
            p.rect_filled(egui::Rect::from_min_size(screen.min, vec2(bar, screen.height())), 0.0, Color32::BLACK);
            p.rect_filled(
                egui::Rect::from_min_size(egui::pos2(screen.max.x - bar, screen.min.y), vec2(bar, screen.height())),
                0.0,
                Color32::BLACK,
            );
            frame = egui::Rect::from_min_max(
                egui::pos2(screen.min.x + bar, screen.min.y),
                egui::pos2(screen.max.x - bar, screen.max.y),
            );
        }
    }
    if d.guides_shown() {
        let line = Stroke::new(1.0, Color32::from_rgba_unmultiplied(214, 225, 240, 90));
        if d.prefs.guides_thirds {
            for k in [1.0 / 3.0, 2.0 / 3.0] {
                let x = frame.min.x + frame.width() * k;
                let y = frame.min.y + frame.height() * k;
                p.line_segment([egui::pos2(x, frame.min.y), egui::pos2(x, frame.max.y)], line);
                p.line_segment([egui::pos2(frame.min.x, y), egui::pos2(frame.max.x, y)], line);
            }
        }
        if d.prefs.guides_center {
            let c = frame.center();
            p.line_segment([c - vec2(12.0, 0.0), c + vec2(12.0, 0.0)], line);
            p.line_segment([c - vec2(0.0, 12.0), c + vec2(0.0, 12.0)], line);
        }
        if d.prefs.guides_safe {
            let r = frame.shrink2(vec2(frame.width() * 0.1, frame.height() * 0.1));
            p.rect_stroke(r, 0.0, line, egui::StrokeKind::Inside);
        }
    }
    if let Some((left, _)) = d.countdown {
        let n = left.ceil().max(1.0) as u32;
        let c = frame.center();
        let a = (left.fract() * 255.0).clamp(60.0, 255.0) as u8;
        p.circle_filled(c, 54.0, Color32::from_rgba_unmultiplied(10, 11, 18, 170));
        p.circle_stroke(c, 54.0, Stroke::new(2.0, Color32::from_rgba_unmultiplied(143, 178, 230, a)));
        p.text(
            c,
            Align2::CENTER_CENTER,
            n.to_string(),
            FontId::proportional(56.0),
            Color32::from_rgba_unmultiplied(230, 236, 245, a),
        );
        let stop = match d.countdown.map(|c| c.1) {
            Some(dir::Cue::Play) => "Ctrl+Shift+P stops the shot",
            Some(dir::Cue::Record) => "Recording · Ctrl+Shift+R stops",
            _ => "Filming the shot · Ctrl+Shift+R stops early",
        };
        p.text(
            c + vec2(0.0, 78.0),
            Align2::CENTER_CENTER,
            format!("{stop} · Ctrl+Shift+H shows the window"),
            FontId::proportional(15.0),
            Color32::from_rgba_unmultiplied(230, 236, 245, 220),
        );
    }
}
