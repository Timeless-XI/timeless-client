//! First start without game files (not retail: retail's installer was its program). The title and login are
//! drawn from the DATs, so until they exist a small window of ours asks to download the free retail client from
//! Square Enix, installs its game files, brings them up to date and deletes what it downloaded, or to point at game
//! files already on disk.

use crate::install::Progress;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

/// Square Enix's download of the full client, five RAR volumes (part1 is the self-extractor).
const CDN: &str = "https://gdl.square-enix.com/ffxi/download/us";
const VOLUMES: usize = 5;
/// 7,734,210,712 bytes, what the CDN's lengths add up to (for the estimate before the first response).
const TOTAL: u64 = 7_734_210_712;

fn volume(i: usize) -> String {
    if i == 1 { "FFXIFullSetup_US.part1.exe".into() } else { format!("FFXIFullSetup_US.part{i}.rar") }
}

/// The first-start window's size (fixed; the configured size applies once the game files are in). TimelessXI: wide
/// enough for the launcher's banner at its own size.
pub const WINDOW: [u32; 2] = [980, 700];

/// TimelessXI: the first-start window wears the Timeless Launcher's look (timeless-tools TimelessLauncher: Theme.cs,
/// Banner.cs): its banner, Timeless XI: Reforged over the misty island, with the hourglass animated from the
/// launcher's own frames, its colours, and the heading in its typeface (Cormorant, OFL; drawn into title.png, as the
/// client's fonts are replaced once the game's interface starts).
mod look {
    use egui::Color32;
    /// The banner's bottom edge, so the window runs on from it.
    pub const BG: Color32 = Color32::from_rgb(0x0B, 0x0D, 0x14);
    pub const CARD: Color32 = Color32::from_rgb(0x11, 0x14, 0x1D);
    pub const CARD2: Color32 = Color32::from_rgb(0x18, 0x1D, 0x28);
    pub const HOVER: Color32 = Color32::from_rgb(0x24, 0x2A, 0x36);
    pub const LINE: Color32 = Color32::from_rgb(0x28, 0x30, 0x40);
    pub const TEXT: Color32 = Color32::from_rgb(0xE6, 0xEC, 0xF5);
    pub const MUTED: Color32 = Color32::from_rgb(0x98, 0xA3, 0xB8);
    pub const ICE: Color32 = Color32::from_rgb(0xD6, 0xE1, 0xF0);
    pub const ACCENT: Color32 = Color32::from_rgb(0x8F, 0xB2, 0xE6);
    pub const DEEP: Color32 = Color32::from_rgb(0x3D, 0x5F, 0x94);
    pub const DEEP_HOVER: Color32 = Color32::from_rgb(0x4C, 0x72, 0xAD);
    pub const BAD: Color32 = Color32::from_rgb(0xE0, 0x7A, 0x70);

    pub const BANNER: &[u8] = include_bytes!("../../assets/timeless/banner.png");
    /// The banner's size in pixels.
    pub const BANNER_SIZE: egui::Vec2 = egui::vec2(980.0, 320.0);
    /// "TXHP", then count, fps, x, y, w, h (u32 LE; where the frames go on the banner), then each frame as a u32
    /// length and a PNG: the launcher's hourglass_frames.bin, cut to the part that moves.
    pub const HOURGLASS: &[u8] = include_bytes!("../../assets/timeless/hourglass.bin");
    /// The heading, drawn at twice the size it is shown.
    pub const TITLE: &[u8] = include_bytes!("../../assets/timeless/title.png");
}

/// A PNG as an egui image.
fn png_image(bytes: &[u8]) -> Option<egui::ColorImage> {
    let mut d = png::Decoder::new(std::io::Cursor::new(bytes));
    d.set_transformations(png::Transformations::EXPAND | png::Transformations::ALPHA);
    let mut r = d.read_info().ok()?;
    let mut buf = vec![0; r.output_buffer_size()?];
    let info = r.next_frame(&mut buf).ok()?;
    let (w, h) = (info.width as usize, info.height as usize);
    match info.color_type {
        png::ColorType::Rgba => Some(egui::ColorImage::from_rgba_unmultiplied([w, h], buf.get(..w * h * 4)?)),
        _ => None,
    }
}

/// The hourglass frames: frames per second, where they go on the banner, each frame's PNG.
fn hourglass(b: &'static [u8]) -> Option<(u32, egui::Rect, Vec<&'static [u8]>)> {
    let u32_at = |at: usize| -> Option<u32> { Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?)) };
    if b.get(..4)? != b"TXHP" {
        return None;
    }
    let (n, fps) = (u32_at(4)?, u32_at(8)?);
    let at = egui::Rect::from_min_size(
        egui::pos2(u32_at(12)? as f32, u32_at(16)? as f32),
        egui::vec2(u32_at(20)? as f32, u32_at(24)? as f32),
    );
    let mut pos = 28;
    let mut frames = Vec::with_capacity(n as usize);
    for _ in 0..n {
        let len = u32_at(pos)? as usize;
        frames.push(b.get(pos + 4..pos + 4 + len)?);
        pos += 4 + len;
    }
    Some((fps.max(1), at, frames))
}

/// The banner and the heading as textures, and the hourglass frame shown now.
struct Art {
    banner: egui::TextureHandle,
    title: Option<egui::TextureHandle>,
    fps: u32,
    at: egui::Rect,
    frames: Vec<&'static [u8]>,
    frame: Option<(usize, egui::TextureHandle)>,
}

impl Art {
    fn load(ctx: &egui::Context) -> Option<Self> {
        let banner = ctx.load_texture("timeless-banner", png_image(look::BANNER)?, egui::TextureOptions::LINEAR);
        let title = png_image(look::TITLE).map(|i| ctx.load_texture("timeless-title", i, egui::TextureOptions::LINEAR));
        let (fps, at, frames) = hourglass(look::HOURGLASS).unwrap_or((20, egui::Rect::NOTHING, Vec::new()));
        Some(Self { banner, title, fps, at, frames, frame: None })
    }

    /// The hourglass frame for this moment (decoded when it changes).
    fn frame(&mut self, ctx: &egui::Context) -> Option<egui::TextureId> {
        if self.frames.is_empty() {
            return None;
        }
        let n = ((ctx.input(|i| i.time) * self.fps as f64) as u64 % self.frames.len() as u64) as usize;
        match &mut self.frame {
            Some((i, _)) if *i == n => {}
            Some((i, tex)) => {
                if let Some(img) = png_image(self.frames[n]) {
                    tex.set(img, egui::TextureOptions::LINEAR);
                    *i = n;
                }
            }
            None => {
                let img = png_image(self.frames[n])?;
                self.frame = Some((n, ctx.load_texture("timeless-hourglass", img, egui::TextureOptions::LINEAR)));
            }
        }
        self.frame.as_ref().map(|(_, t)| t.id())
    }
}

/// The launcher's widget colours for everything in the card.
fn style(ui: &mut egui::Ui) {
    let v = ui.visuals_mut();
    v.override_text_color = Some(look::TEXT);
    v.selection.bg_fill = look::DEEP;
    v.selection.stroke = egui::Stroke::new(1.0, look::ACCENT);
    v.extreme_bg_color = look::CARD2;
    let w = &mut v.widgets;
    for (s, fill, line) in [
        (&mut w.inactive, look::CARD2, look::LINE),
        (&mut w.hovered, look::HOVER, look::ACCENT),
        (&mut w.active, look::DEEP, look::ACCENT),
    ] {
        s.weak_bg_fill = fill;
        s.bg_fill = fill;
        s.bg_stroke = egui::Stroke::new(1.0, line);
        s.fg_stroke = egui::Stroke::new(1.5, look::TEXT);
        s.corner_radius = egui::CornerRadius::same(6);
    }
}

/// The launcher's blue main button.
fn primary(ui: &mut egui::Ui, enabled: bool, text: &str) -> egui::Response {
    ui.scope(|ui| {
        let w = &mut ui.visuals_mut().widgets;
        w.inactive.weak_bg_fill = look::DEEP;
        w.hovered.weak_bg_fill = look::DEEP_HOVER;
        w.active.weak_bg_fill = look::DEEP_HOVER;
        w.inactive.bg_stroke = egui::Stroke::new(1.0, look::DEEP_HOVER);
        ui.add_enabled(enabled, egui::Button::new(egui::RichText::new(text).size(18.0).color(egui::Color32::WHITE)))
    })
    .inner
}

/// A plain button in the launcher's colours.
fn secondary(ui: &mut egui::Ui, enabled: bool, text: &str, size: f32) -> egui::Response {
    ui.add_enabled(enabled, egui::Button::new(egui::RichText::new(text).size(size)))
}

/// Left in the game folder while the first start runs: one stopped part-way resumes in this window.
const MARKER: &str = ".vana-installing";

/// No game files where the config says they are, or a first start that did not finish.
pub fn needed(ffxi_dir: &Path) -> bool {
    !ffxi_dir.join("VTABLE.DAT").is_file()
        || ffxi_dir.parent().is_some_and(|g| g.join(MARKER).is_file() || g.join(".download").is_dir())
}

/// The game folder (the one that holds `FINAL FANTASY XI`) for an install picked in the window: the
/// `FINAL FANTASY XI` folder itself, or a folder above it in the Windows layout (`PlayOnline/SquareEnix`).
pub fn existing(picked: &Path) -> Result<PathBuf> {
    let candidates = [
        picked.parent().filter(|_| picked.file_name().is_some_and(|n| n == "FINAL FANTASY XI")).map(Path::to_path_buf),
        Some(picked.to_path_buf()),
        Some(picked.join("SquareEnix")),
        Some(picked.join("PlayOnline").join("SquareEnix")),
    ];
    let found = candidates.into_iter().flatten().find(|g| g.join("FINAL FANTASY XI").join("VTABLE.DAT").is_file());
    let Some(game_dir) = found else {
        anyhow::bail!(
            "No game files in {}: pick the FINAL FANTASY XI folder (the one with VTABLE.DAT) or a folder above it.",
            picked.display()
        );
    };
    if needed(&game_dir.join("FINAL FANTASY XI")) {
        anyhow::bail!(
            "The install in {} did not finish: choose Download and install with that folder to resume it.",
            game_dir.display()
        );
    }
    Ok(game_dir)
}

/// What closing the client does to the step under way.
fn closing_note(phase: &str) -> &'static str {
    if phase.starts_with("Downloading") {
        "Closing the TimelessXI client stops this; the download resumes next time."
    } else if phase.starts_with("Unpacking") || phase.starts_with("Installing") {
        "Closing the TimelessXI client stops this; installing starts over next time (the download is kept)."
    } else if phase.starts_with("Updating") || phase.starts_with("Checking") {
        "Closing the TimelessXI client stops this; the update picks up where it left off next time."
    } else {
        ""
    }
}

enum Stage {
    Ask,
    Working,
    Done,
    Failed(String),
}

pub enum Action {
    None,
    Quit,
    /// The download starts in this folder: the config keeps it, so a resumed start finds it.
    Begin(PathBuf),
    /// Game files are in: load the title.
    Ready,
}

pub struct FirstRun {
    game_dir: PathBuf,
    stage: Arc<Mutex<Stage>>,
    progress: Arc<Progress>,
    /// The folder picker's answer (it runs on a separate thread so the window keeps drawing).
    picked: Arc<Mutex<Option<Option<PathBuf>>>>,
    /// The picker open, and what its answer is for.
    picking: Option<Pick>,
    /// The game files were found in place rather than downloaded.
    found: bool,
    /// The finish page's choices (Linux): an application menu entry, a desktop shortcut.
    menu: bool,
    shortcut: bool,
    note: String,
    /// TimelessXI: the banner and the hourglass (loaded on the first frame drawn; None if they wouldn't load).
    art: Option<Art>,
    art_loaded: bool,
}

#[derive(Clone, Copy)]
enum Pick {
    /// Where to download and install.
    Install,
    /// An install already on disk.
    Existing,
}

impl FirstRun {
    pub fn new(game_dir: PathBuf) -> Self {
        Self {
            game_dir,
            stage: Arc::new(Mutex::new(Stage::Ask)),
            progress: Arc::new(Progress::default()),
            picked: Default::default(),
            picking: None,
            found: false,
            menu: true,
            shortcut: false,
            note: String::new(),
            art: None,
            art_loaded: false,
        }
    }

    /// Where the game files go (the config default, or the folder picked).
    pub fn game_dir(&self) -> &Path {
        &self.game_dir
    }

    fn pick(&mut self, what: Pick) {
        self.picking = Some(what);
        let (start, out) = (self.game_dir.clone(), self.picked.clone());
        let title = match what {
            Pick::Install => "Install the game files in",
            Pick::Existing => "The FINAL FANTASY XI folder",
        };
        std::thread::spawn(move || {
            let mut dialog = rfd::FileDialog::new().set_title(title);
            if let Some(dir) = start.ancestors().find(|d| d.is_dir()) {
                dialog = dialog.set_directory(dir);
            }
            *out.lock().unwrap() = Some(dialog.pick_folder());
        });
    }

    fn start(&self) {
        *self.stage.lock().unwrap() = Stage::Working;
        let (game_dir, stage, progress) = (self.game_dir.clone(), self.stage.clone(), self.progress.clone());
        std::thread::spawn(move || {
            let r = run(&game_dir, &progress);
            *stage.lock().unwrap() = match r {
                Ok(()) => Stage::Done,
                Err(e) => {
                    log::warn!("first start: {e:#}");
                    Stage::Failed(format!("{e:#}"))
                }
            };
        });
    }

    /// The window (egui: no game art exists yet).
    pub fn ui(&mut self, ctx: &egui::Context) -> Action {
        let mut action = Action::None;
        let stage = match &*self.stage.lock().unwrap() {
            Stage::Ask => 0,
            Stage::Working => 1,
            Stage::Done => 2,
            Stage::Failed(_) => 3,
        };
        if stage == 2 && !super::desktop::supported() {
            return Action::Ready;
        }
        if let Some(answer) = self.picked.lock().unwrap().take()
            && let Some(what) = self.picking.take()
            && let Some(dir) = answer
        {
            self.note.clear();
            match what {
                Pick::Install => self.game_dir = dir,
                Pick::Existing => match existing(&dir) {
                    Ok(game_dir) => {
                        self.game_dir = game_dir;
                        self.found = true;
                        *self.stage.lock().unwrap() = Stage::Done;
                        if !super::desktop::supported() {
                            return Action::Ready;
                        }
                    }
                    Err(e) => self.note = format!("{e:#}"),
                },
            }
        }
        let failed = match &*self.stage.lock().unwrap() {
            Stage::Failed(e) => e.clone(),
            _ => String::new(),
        };
        if !self.art_loaded {
            self.art_loaded = true;
            self.art = Art::load(ctx);
        }
        let screen = ctx.content_rect();
        let painter = ctx.layer_painter(egui::LayerId::background());
        painter.rect_filled(screen, 0.0, look::BG);
        // The banner across the top, the hourglass animated on it; the card sits just over its foot.
        let mut card_top = 32.0;
        let mut title = None;
        if let Some(art) = self.art.as_mut() {
            let scale = screen.width() / look::BANNER_SIZE.x;
            let banner = egui::Rect::from_min_size(screen.min, look::BANNER_SIZE * scale);
            let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
            painter.image(art.banner.id(), banner, uv, egui::Color32::WHITE);
            if let Some(id) = art.frame(ctx) {
                let at = egui::Rect::from_min_size(banner.min + art.at.min.to_vec2() * scale, art.at.size() * scale);
                painter.image(id, at, uv, egui::Color32::WHITE);
            }
            card_top = banner.height() - 24.0 * scale;
            title = art.title.as_ref().map(|t| (t.id(), t.size_vec2() / 2.0));
            ctx.request_repaint_after(std::time::Duration::from_millis(1000 / art.fps.max(1) as u64));
        }
        egui::Area::new(egui::Id::new("first run")).anchor(egui::Align2::CENTER_TOP, [0.0, card_top]).show(ctx, |ui| {
            {
                egui::Frame::new()
                    .fill(look::CARD)
                    .stroke(egui::Stroke::new(1.0, look::LINE))
                    .corner_radius(10.0)
                    .inner_margin(28.0)
                    .show(ui, |ui| {
                        ui.set_max_width(640.0);
                        style(ui);
                        ui.style_mut().spacing.button_padding = egui::vec2(16.0, 8.0);
                        ui.style_mut().spacing.item_spacing = egui::vec2(10.0, 8.0);
                        ui.vertical(|ui| {
                            match title {
                                Some((id, size)) => {
                                    ui.add(egui::Image::new(egui::load::SizedTexture::new(id, size)));
                                }
                                None => {
                                    ui.label(egui::RichText::new("FINAL FANTASY XI GAME FILES").size(24.0).color(look::ICE));
                                }
                            }
                            // The launcher's thin accent rule under a heading.
                            let (rule, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), egui::Sense::hover());
                            ui.painter().rect_filled(rule, 0.0, egui::Color32::from_rgba_unmultiplied(0x8F, 0xB2, 0xE6, 70));
                            ui.add_space(8.0);
                            match stage {
                                0 => {
                                    ui.label(egui::RichText::new(
                                        "TimelessXI plays on the game files of the free FINAL FANTASY XI client. It downloads them from \
                                         Square Enix (about 7.7 GB) and installs them to",
                                    ).size(16.0));
                                    ui.label(egui::RichText::new(self.game_dir.display().to_string()).size(16.0).color(look::ACCENT));
                                    ui.label(egui::RichText::new(
                                        "The download is deleted once the game files are installed and up to date. About 25 GB of free \
                                         space is needed while it runs. Already have FINAL FANTASY XI? Use its game files instead.",
                                    ).size(14.0).color(look::MUTED));
                                    ui.add_space(4.0);
                                    let picking = self.picking.is_some();
                                    ui.horizontal(|ui| {
                                        if secondary(ui, !picking, "Change install folder...", 15.0).clicked() {
                                            self.pick(Pick::Install);
                                        }
                                        if secondary(ui, !picking, "Use existing game files...", 15.0).clicked() {
                                            self.pick(Pick::Existing);
                                        }
                                    });
                                    if !self.note.is_empty() {
                                        ui.label(egui::RichText::new(&self.note).color(look::BAD));
                                    }
                                    ui.add_space(12.0);
                                    ui.horizontal(|ui| {
                                        if primary(ui, !picking, "Download and install").clicked() {
                                            self.start();
                                            action = Action::Begin(self.game_dir.clone());
                                        }
                                        if secondary(ui, true, "Quit", 18.0).clicked() {
                                            action = Action::Quit;
                                        }
                                    });
                                }
                                2 => {
                                    if self.found {
                                        ui.label(egui::RichText::new("Using the game files in").size(16.0));
                                        ui.label(
                                            egui::RichText::new(self.game_dir.join("FINAL FANTASY XI").display().to_string())
                                                .size(16.0)
                                                .color(look::ACCENT),
                                        );
                                        ui.label(egui::RichText::new(
                                            "If they are behind the current version, the Update button on the title screen brings them up to date.",
                                        ).size(14.0).color(look::MUTED));
                                    } else {
                                        ui.label(egui::RichText::new("The game files are installed and up to date.").size(16.0));
                                    }
                                    ui.add_space(10.0);
                                    ui.checkbox(&mut self.menu, egui::RichText::new("Add TimelessXI to the application menu").size(15.0));
                                    ui.checkbox(&mut self.shortcut, egui::RichText::new("Add a shortcut to the desktop").size(15.0));
                                    if !self.note.is_empty() {
                                        ui.label(egui::RichText::new(&self.note).color(look::MUTED));
                                    }
                                    ui.add_space(12.0);
                                    if primary(ui, true, "Continue").clicked() {
                                        let r = super::desktop::install(self.menu).and_then(|()| if self.shortcut { super::desktop::desktop_shortcut() } else { Ok(()) });
                                        match r {
                                            Ok(()) => action = Action::Ready,
                                            Err(e) => self.note = format!("Could not add it: {e:#}"),
                                        }
                                    }
                                }
                                1 => {
                                    let p = &self.progress;
                                    let (done, total) = (p.done.load(Ordering::Relaxed), p.total.load(Ordering::Relaxed));
                                    ui.label(egui::RichText::new(p.phase.lock().unwrap().clone()).size(18.0).color(look::ICE));
                                    ui.add_space(6.0);
                                    ui.add(egui::ProgressBar::new(p.fraction()).desired_width(640.0).fill(look::DEEP_HOVER));
                                    if total > 0 {
                                        ui.label(
                                            egui::RichText::new(format!("{:.2} of {:.2} GB", done as f64 / 1e9, total as f64 / 1e9))
                                                .color(look::MUTED),
                                        );
                                    }
                                    ui.add_space(6.0);
                                    let note = closing_note(&p.phase.lock().unwrap());
                                    if !note.is_empty() {
                                        ui.label(egui::RichText::new(note).size(14.0).color(look::MUTED));
                                    }
                                    ctx.request_repaint_after(std::time::Duration::from_millis(250));
                                }
                                _ => {
                                    ui.label(egui::RichText::new("It did not finish:").size(16.0));
                                    ui.label(egui::RichText::new(&failed).color(look::BAD));
                                    ui.add_space(12.0);
                                    ui.horizontal(|ui| {
                                        if primary(ui, true, "Try again").clicked() {
                                            self.start();
                                        }
                                        if secondary(ui, true, "Quit", 18.0).clicked() {
                                            action = Action::Quit;
                                        }
                                    });
                                }
                            }
                        });
                    });
            }
        });
        action
    }
}

fn set(p: &Progress, phase: impl Into<String>, done: u64, total: u64) {
    *p.phase.lock().unwrap() = phase.into();
    p.done.store(done, Ordering::Relaxed);
    p.total.store(total, Ordering::Relaxed);
}

/// Download, install, update, clean up.
fn run(game_dir: &Path, p: &Arc<Progress>) -> Result<()> {
    let dl = game_dir.join(".download");
    std::fs::create_dir_all(&dl).with_context(|| dl.display().to_string())?;
    std::fs::write(game_dir.join(MARKER), b"")?;
    // Bytes already on disk count toward the bar, so a resumed download starts where it was.
    let mut before = 0u64;
    for i in 1..=VOLUMES {
        let dest = dl.join(volume(i));
        if dest.is_file() {
            before += std::fs::metadata(&dest)?.len();
            continue;
        }
        let base = before;
        super::updater::download(&format!("{CDN}/{}", volume(i)), &dest, &|d, _| {
            set(p, format!("Downloading from Square Enix (part {i} of {VOLUMES})"), base + d, TOTAL)
        })?;
        before += std::fs::metadata(&dest)?.len();
    }
    // A finished install from an earlier, stopped run is not unpacked again.
    let root = game_dir.join("FINAL FANTASY XI");
    let installed = root.join("VTABLE.DAT").is_file() && !dl.join(volume(1)).is_file();
    let opts = crate::install::Options {
        source: dl.join(volume(1)),
        staging: game_dir.join(".staging"),
        game_dir: game_dir.to_path_buf(),
        verify: true,
        keep_staging: false,
    };
    if !installed {
        crate::install::install(&opts, p.clone())?;
    }
    set(p, "Checking for game updates", 0, 0);
    let plan = crate::patch::plan(&root, true, false)?;
    let total: u64 = plan.todo.iter().map(|t| t.2).sum();
    let n = plan.todo.len();
    crate::patch::apply(&root, &plan, 4, &|f, b| set(p, format!("Updating game files ({f} of {n})"), b, total))?;
    set(p, "Cleaning up", 0, 0);
    std::fs::remove_dir_all(&dl).with_context(|| format!("removing {}", dl.display()))?;
    let _ = std::fs::remove_dir_all(game_dir.join(".staging"));
    std::fs::remove_file(game_dir.join(MARKER))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn existing_finds_the_game_folder_from_any_level() {
        let base = std::env::temp_dir().join(format!("vana-firstrun-{}", std::process::id()));
        let game = base.join("PlayOnline").join("SquareEnix");
        let ffxi = game.join("FINAL FANTASY XI");
        std::fs::create_dir_all(&ffxi).unwrap();
        assert!(existing(&base).is_err());
        std::fs::write(ffxi.join("VTABLE.DAT"), b"").unwrap();
        for picked in [&ffxi, &game, &base.join("PlayOnline"), &base] {
            assert_eq!(existing(picked).unwrap(), game, "picked {}", picked.display());
        }
        // An install vana-client stopped part-way is resumed, not used.
        std::fs::write(game.join(MARKER), b"").unwrap();
        assert!(existing(&ffxi).is_err());
        std::fs::remove_dir_all(&base).unwrap();
    }
}
