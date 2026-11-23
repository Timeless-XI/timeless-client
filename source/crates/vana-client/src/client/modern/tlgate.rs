//! Timeless XI: the title's update gate (client/timeless/manifest.rs). While the client manifest is read (at most
//! manifest::CHECK_SECS) a small card says so; a client older than the manifest's `required` gets the Update Required
//! card in place of the login and the character menu, with Update Now (the signed release, then a restart) and Exit
//! Game. Nothing else on the title answers until it has updated.

use super::super::timeless::manifest::{self, Manifest, Server, Verdict};
use super::tlsetup::{
    ALERT, FOCUS, Look, MUTED, SILVER, TEXT, body, button, bytes, card, font, para, sand_bar, serif, spaced,
};
use egui::{Context, Id, Order, Rect, pos2, vec2};
use std::sync::{Arc, Mutex};
use std::time::Instant;

enum Job {
    Running(u64, u64),
    Done,
    Failed(String),
}

/// What the app does after a frame of the gate.
#[derive(PartialEq, Debug)]
pub enum GateOut {
    None,
    Exit,
    /// The new client is in place: start it and close this one.
    Restart,
}

pub struct Gate {
    verdict: manifest::Shared,
    started: Instant,
    job: Option<Arc<Mutex<Job>>>,
    /// The manifest's server, handed to the app once.
    server_given: bool,
}

impl Gate {
    /// Starts reading the manifest.
    pub fn new() -> Self {
        Self { verdict: manifest::start(), started: Instant::now(), job: None, server_given: false }
    }

    /// No gate (test drives, the setup).
    pub fn off() -> Self {
        Self {
            verdict: Arc::new(Mutex::new(Verdict::Clear(None))),
            started: Instant::now(),
            job: None,
            server_given: true,
        }
    }

    fn manifest(&self) -> Option<Manifest> {
        match &*self.verdict.lock().unwrap() {
            Verdict::Required(m) => Some(m.clone()),
            Verdict::Clear(m) => m.clone(),
            Verdict::Checking => None,
        }
    }

    /// The manifest's Timeless XI server, the first frame it is known (the app points its config at it).
    pub fn take_server(&mut self) -> Option<Server> {
        if self.server_given {
            return None;
        }
        let checking = matches!(*self.verdict.lock().unwrap(), Verdict::Checking);
        if checking {
            return None;
        }
        self.server_given = true;
        self.manifest().and_then(|m| m.server)
    }

    fn checking(&self) -> bool {
        matches!(*self.verdict.lock().unwrap(), Verdict::Checking)
            && self.started.elapsed().as_secs_f32() < manifest::CHECK_SECS
    }

    fn required(&self) -> Option<Manifest> {
        match &*self.verdict.lock().unwrap() {
            Verdict::Required(m) => Some(m.clone()),
            _ => None,
        }
    }

    /// The title is held: the manifest still being read, or an update required.
    pub fn holds(&self) -> bool {
        self.checking() || self.required().is_some()
    }

    fn update(&mut self, m: &Manifest) {
        let job = Arc::new(Mutex::new(Job::Running(0, 0)));
        self.job = Some(job.clone());
        let r = super::super::updater::release_for(&m.target());
        std::thread::spawn(move || {
            let j = job.clone();
            let out = super::super::updater::install_release(&r, &move |d, t| *j.lock().unwrap() = Job::Running(d, t));
            *job.lock().unwrap() = match out {
                Ok(()) => {
                    log::info!("timeless: updated to {}", r.tag);
                    Job::Done
                }
                Err(e) => {
                    log::warn!("timeless: update to {}: {e:#}", r.tag);
                    Job::Failed(format!("{e:#}"))
                }
            };
        });
    }

    /// A frame of the gate over the title (call only while `holds`).
    pub fn ui(&mut self, ctx: &Context) -> GateOut {
        let screen = ctx.content_rect();
        let t = ctx.input(|i| i.time);
        let mut out = GateOut::None;
        let Some(m) = self.required() else {
            // Still reading the manifest: a quiet card where the login goes.
            let r = Rect::from_center_size(
                pos2(screen.center().x, screen.min.y + screen.height() * 0.72),
                vec2(360.0, 86.0),
            );
            egui::Area::new(Id::unique("tl gate checking")).order(Order::Foreground).fixed_pos(r.min).show(ctx, |ui| {
                ui.set_min_size(r.size());
                let p = ui.painter();
                card(p, r);
                spaced(
                    p,
                    pos2(r.center().x, r.min.y + 20.0),
                    "CHECKING FOR UPDATES",
                    font(ctx, body(12.5)),
                    SILVER,
                    2.5,
                    true,
                );
                sand_bar(
                    p,
                    Rect::from_min_size(pos2(r.min.x + 40.0, r.min.y + 52.0), vec2(r.width() - 80.0, 8.0)),
                    None,
                    t,
                );
            });
            ctx.request_repaint();
            return out;
        };
        let w = (screen.width() * 0.46).clamp(460.0, 620.0);
        let r = Rect::from_center_size(pos2(screen.center().x, screen.min.y + screen.height() * 0.72), vec2(w, 250.0));
        let job = self.job.as_ref().map(|j| match &*j.lock().unwrap() {
            Job::Running(d, t) => Job::Running(*d, *t),
            Job::Done => Job::Done,
            Job::Failed(e) => Job::Failed(e.clone()),
        });
        let mut start = false;
        egui::Area::new(Id::unique("tl gate")).order(Order::Foreground).fixed_pos(r.min).show(ctx, |ui| {
            ui.set_min_size(r.size());
            card(ui.painter(), r);
            let inner = r.shrink2(vec2(34.0, 26.0));
            let target = m.target();
            let p = ui.painter();
            spaced(p, pos2(inner.center().x, inner.min.y), &format!("TIMELESS XI {}", target.trim_start_matches('v').to_uppercase()), font(ctx, body(11.5)), FOCUS, 3.0, true);
            let title = match &job {
                Some(Job::Done) => "Updated",
                Some(Job::Running(..)) => "Updating Timeless XI",
                _ => "An update is required",
            };
            p.text(pos2(inner.center().x, inner.min.y + 22.0), egui::Align2::CENTER_TOP, title, font(ctx, serif(31.0)), SILVER);
            super::borders::fade_rule(p, inner.min.x, inner.max.x, inner.min.y + 66.0, super::tlsetup::PEWTER.gamma_multiply(0.7));
            let body_y = inner.min.y + 80.0;
            match &job {
                Some(Job::Running(d, total)) => {
                    let line = if *total > 0 { format!("Downloading  ·  {} of {}", bytes(*d), bytes(*total)) } else { "Downloading…".into() };
                    p.text(pos2(inner.min.x, body_y), egui::Align2::LEFT_TOP, line, font(ctx, body(14.5)), TEXT);
                    sand_bar(p, Rect::from_min_size(pos2(inner.min.x, body_y + 30.0), vec2(inner.width(), 12.0)), (*total > 0).then(|| *d as f32 / *total as f32), t);
                    ctx.request_repaint();
                }
                Some(Job::Done) => {
                    para(p, pos2(inner.min.x, body_y), "The new client is in place. Timeless XI restarts now.", font(ctx, body(15.0)), TEXT, inner.width());
                    out = GateOut::Restart;
                }
                Some(Job::Failed(e)) => {
                    let h = para(p, pos2(inner.min.x, body_y), "The update didn't finish. Check your connection and try again.", font(ctx, body(15.0)), TEXT, inner.width());
                    para(p, pos2(inner.min.x, body_y + h + 6.0), e, font(ctx, body(12.5)), ALERT, inner.width());
                }
                None => {
                    let text = if m.message.trim().is_empty() {
                        "A new version of the Timeless XI client is out, and it's needed to play. Updating takes a moment \
                         and keeps your settings, hotbars and characters."
                    } else {
                        m.message.trim()
                    };
                    let h = para(p, pos2(inner.min.x, body_y), text, font(ctx, body(15.0)), TEXT, inner.width());
                    p.text(pos2(inner.min.x, body_y + h + 6.0), egui::Align2::LEFT_TOP, format!("You have {}.", manifest::current()), font(ctx, body(12.5)), MUTED);
                }
            }
            let running = matches!(job, Some(Job::Running(..)) | Some(Job::Done));
            let (bh, bw) = (38.0, 180.0);
            let y = inner.max.y - bh;
            let upd = egui::Rect::from_min_size(pos2(inner.max.x - bw, y), vec2(bw, bh));
            let exit = egui::Rect::from_min_size(pos2(inner.max.x - bw - 14.0 - 150.0, y), vec2(150.0, bh));
            let label = if matches!(job, Some(Job::Failed(_))) { "Try Again" } else { "Update Now" };
            let enter = ctx.input(|i| i.key_pressed(egui::Key::Enter));
            if button(ui, upd, "gate update", label, Look::Primary, !running) || (enter && !running) {
                start = true;
            }
            if button(ui, exit, "gate exit", "Exit Game", Look::Secondary, !running) {
                out = GateOut::Exit;
            }
        });
        if start {
            self.update(&m);
        }
        out
    }
}

/// `modern-shot --setup gate|gate-updating|gate-checking`: the gate with a sample manifest.
pub fn mock(what: &str) -> Gate {
    let m = Manifest {
        format: 1,
        latest: "1.0.15-timeless.1".into(),
        required: "1.0.15-timeless.1".into(),
        ..Default::default()
    };
    let v = if what == "gate-checking" { Verdict::Checking } else { Verdict::Required(m) };
    let job = (what == "gate-updating").then(|| Arc::new(Mutex::new(Job::Running(31_400_000, 73_000_000))));
    Gate { verdict: Arc::new(Mutex::new(v)), started: Instant::now(), job, server_given: true }
}
