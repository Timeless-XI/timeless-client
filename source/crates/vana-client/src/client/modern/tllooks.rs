//! Timeless XI: the Appearance window (`/look`) for custom looks (timeless/looks.rs): RGB sliders for one- or
//! two-tone hair, eye colour, face markings, lip tint, and a grey-to-black nose slider. Every interface.

use super::super::World;
use super::super::timeless::looks::{self, BOTH, LEFT, MARKS, RIGHT, SIDED, Settings};
use egui::{Color32, Context, CornerRadius, Id, RichText, Stroke, vec2};

fn gold() -> Color32 {
    Color32::from_rgb(214, 225, 240)
}
fn text() -> Color32 {
    Color32::from_rgb(230, 236, 245)
}
fn muted() -> Color32 {
    Color32::from_rgb(152, 163, 184)
}

/// Quick picks under each colour.
const HAIR: [[u8; 3]; 10] = [
    [232, 226, 216],
    [24, 22, 26],
    [86, 52, 30],
    [150, 92, 48],
    [222, 186, 120],
    [168, 44, 36],
    [226, 120, 60],
    [60, 84, 150],
    [70, 150, 120],
    [170, 110, 190],
];
const EYES: [[u8; 3]; 8] = [
    [111, 143, 168],
    [74, 138, 208],
    [64, 176, 112],
    [216, 160, 56],
    [176, 92, 40],
    [190, 60, 60],
    [150, 100, 200],
    [200, 200, 210],
];
const LIPS: [[u8; 3]; 6] = [[154, 106, 90], [192, 112, 104], [168, 58, 58], [120, 70, 100], [96, 64, 52], [70, 50, 60]];
const MARK: [[u8; 3]; 6] = [[90, 52, 32], [30, 20, 18], [232, 232, 240], [150, 40, 36], [40, 60, 120], [200, 150, 60]];

fn c32(c: [u8; 3]) -> Color32 {
    Color32::from_rgb(c[0], c[1], c[2])
}

fn heading(ui: &mut egui::Ui, t: &str) {
    ui.add_space(6.0);
    ui.label(RichText::new(t).strong().color(gold()));
}

/// A colour as three 0..255 sliders beside a swatch, with quick picks under them.
fn rgb(ui: &mut egui::Ui, c: &mut [u8; 3], picks: &[[u8; 3]]) {
    ui.horizontal(|ui| {
        let (r, _) = ui.allocate_exact_size(vec2(40.0, 58.0), egui::Sense::hover());
        ui.painter().rect_filled(r, CornerRadius::same(4), c32(*c));
        ui.painter().rect_stroke(r, CornerRadius::same(4), Stroke::new(1.0, muted()), egui::StrokeKind::Inside);
        ui.vertical(|ui| {
            for (k, name) in ["R", "G", "B"].iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(*name).color(muted()).monospace());
                    ui.add(egui::Slider::new(&mut c[k], 0..=255).trailing_fill(true));
                });
            }
        });
    });
    ui.horizontal_wrapped(|ui| {
        ui.add_space(46.0);
        for p in picks {
            let (r, resp) = ui.allocate_exact_size(vec2(18.0, 18.0), egui::Sense::click());
            ui.painter().rect_filled(r, CornerRadius::same(3), c32(*p));
            let edge = if p == c { gold() } else { Color32::from_gray(60) };
            ui.painter().rect_stroke(r, CornerRadius::same(3), Stroke::new(1.0, edge), egui::StrokeKind::Inside);
            if resp.clicked() {
                *c = *p;
            }
        }
    });
}

impl super::Modern {
    pub fn tl_looks(&self, ctx: &Context, w: &mut World) {
        if !w.looks.open {
            return;
        }
        let mut s = w.looks.settings.clone();
        let before = s.clone();
        let fits = w.looks.fits;
        let mut open = true;
        egui::Window::new(RichText::new("Appearance").color(gold()))
            .id(Id::unique("tl looks"))
            .open(&mut open)
            .default_size(vec2(360.0, 620.0))
            .resizable(true)
            .collapsible(false)
            .frame(
                egui::Frame::new()
                    .fill(Color32::from_rgba_unmultiplied(17, 20, 29, 238))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(92, 107, 133)))
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(egui::Margin::same(12)),
            )
            .show(ctx, |ui| {
                ui.checkbox(&mut s.on, RichText::new("Use my custom look").color(text()));
                let face = w.looks.face;
                if fits {
                    ui.label(RichText::new(format!("Mithra face style {}", w.looks.style)).small().color(muted()));
                } else if face.is_some_and(|(race, _)| race == 7) {
                    ui.label(
                        RichText::new("Your face has no custom look yet. Try one on face 6A (only you see it):")
                            .color(muted()),
                    );
                    ui.checkbox(&mut s.try_on, RichText::new("Try on face 6A").color(text()));
                } else {
                    ui.label(RichText::new("Custom looks are made for Mithra faces for now.").color(muted()));
                }
                ui.label(RichText::new("Only you can see your custom look for now.").small().color(muted()));
                ui.separator();
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    ui.add_enabled_ui(s.on, |ui| body(ui, &mut s));
                });
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Reset to my face").on_hover_text("Your face's own colours and markings").clicked() {
                        s = face.and_then(|(_, f)| looks::seed(f, &s)).unwrap_or(Settings {
                            on: s.on,
                            try_on: s.try_on,
                            ..Settings::default()
                        });
                    }
                });
            });
        if !open {
            w.looks.open = false;
        }
        if s != before {
            w.looks.settings = s;
            w.looks.changed(&before);
        }
    }
}

fn body(ui: &mut egui::Ui, s: &mut Settings) {
    heading(ui, "Hair");
    rgb(ui, &mut s.hair, &HAIR);
    ui.checkbox(&mut s.two_tone, RichText::new("Two-tone (colour toward the ends)").color(text()));
    if s.two_tone {
        ui.label(RichText::new("Ends").color(muted()));
        rgb(ui, &mut s.tip, &HAIR);
        ui.horizontal(|ui| {
            ui.label(RichText::new("Starts").color(muted()));
            ui.add(egui::Slider::new(&mut s.tip_start, 0.15..=0.95).show_value(false));
        });
        ui.horizontal(|ui| {
            ui.label(RichText::new("Blend ").color(muted()));
            ui.add(egui::Slider::new(&mut s.tip_blend, 0.02..=0.45).show_value(false));
        });
    }
    heading(ui, "Eyes");
    rgb(ui, &mut s.eyes, &EYES);
    heading(ui, "Face markings");
    s.marks.resize(MARKS.len(), 0);
    ui.label(RichText::new("Left / Right = under your character's own left or right eye.").small().color(muted()));
    egui::Grid::new("tl looks marks").num_columns(2).spacing(vec2(10.0, 4.0)).show(ui, |ui| {
        for (k, name) in MARKS.iter().enumerate() {
            ui.label(RichText::new(*name).color(text()));
            ui.horizontal(|ui| {
                let v = &mut s.marks[k];
                if SIDED[k] {
                    for (val, label) in [(0, "Off"), (LEFT, "Left"), (BOTH, "Both"), (RIGHT, "Right")] {
                        ui.selectable_value(v, val, label);
                    }
                } else {
                    if *v != 0 {
                        *v = BOTH;
                    }
                    ui.selectable_value(v, 0, "Off");
                    ui.selectable_value(v, BOTH, "On");
                }
            });
            ui.end_row();
        }
    });
    if s.marks.iter().any(|m| *m != 0) {
        rgb(ui, &mut s.mark_colour, &MARK);
        ui.horizontal(|ui| {
            ui.label(RichText::new("Strength").color(muted()));
            ui.add(egui::Slider::new(&mut s.mark_strength, 0.2..=1.0).show_value(false));
        });
    }
    heading(ui, "Lips");
    rgb(ui, &mut s.lips, &LIPS);
    heading(ui, "Nose");
    ui.horizontal(|ui| {
        ui.label(RichText::new("Grey").color(muted()));
        ui.add(egui::Slider::new(&mut s.nose, 0.0..=1.0).show_value(false));
        ui.label(RichText::new("Black").color(muted()));
    });
}
