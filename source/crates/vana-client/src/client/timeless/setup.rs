//! Timeless XI: the player's one download, `TimelessXI-Setup.exe`. It is this client itself: started under a name
//! with "setup" in it (or with VANA_SETUP set) it opens the Timeless setup wizard (client/modern/tlsetup.rs) instead
//! of the title, and
//!
//! 1. puts the client in its own folder (per user, no administrator: `%LOCALAPPDATA%\Programs\TimelessXI`) as
//!    `TimelessXI.exe`;
//! 2. gets the FINAL FANTASY XI game files, the quickest way there is: an install already on the computer (used where
//!    it is when it can be written to, else copied), Square Enix's installer already in Downloads, or Square Enix's
//!    free download (about 7.7 GB);
//! 3. brings the game files up to the version the Timeless XI server takes (client manifest `game_version`);
//! 4. points the client at the Timeless XI server in the Timeless interface, and adds the shortcuts and the Windows
//!    "Installed apps" entry (Uninstall runs `TimelessXI.exe uninstall`).
//!
//! Then Play starts the installed client at the title. A setup stopped part-way picks up where it was: downloads
//! resume, a finished game install isn't redone.

use crate::install::Progress;
use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

/// The installed client's file name.
pub const EXE: &str = "TimelessXI.exe";
/// The setup window's size (fixed).
pub const WINDOW: [u32; 2] = [1040, 660];
/// The Windows "Installed apps" entry's key (HKCU\...\Uninstall\<this>).
#[cfg_attr(not(windows), allow(dead_code))]
const UNINSTALL_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall\TimelessXI";
/// Square Enix's full client download (as client/firstrun.rs fetches it).
const CDN: &str = "https://gdl.square-enix.com/ffxi/download/us";
const VOLUMES: usize = 5;
const DOWNLOAD_BYTES: u64 = 7_734_210_712;
/// About what an installed, current FINAL FANTASY XI folder takes.
pub const GAME_BYTES: u64 = 17_000_000_000;
/// Left in the game folder while its files are being put in: a run stopped part-way is resumed, not trusted.
const MARKER: &str = ".timeless-installing";

fn volume(i: usize) -> String {
    if i == 1 { "FFXIFullSetup_US.part1.exe".into() } else { format!("FFXIFullSetup_US.part{i}.rar") }
}

/// Setup mode: this exe's name says setup (the website's TimelessXI-Setup.exe), or VANA_SETUP is set.
pub fn wanted() -> bool {
    static WANTED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *WANTED.get_or_init(|| {
        std::env::var_os("VANA_SETUP").is_some()
            || std::env::current_exe()
                .ok()
                .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().to_lowercase()))
                .is_some_and(|s| s.contains("setup"))
    })
}

/// Where the client goes unless the player picks another folder.
pub fn default_dir() -> PathBuf {
    if cfg!(windows) {
        dirs::data_local_dir().unwrap_or_else(|| PathBuf::from(r"C:\TimelessXI")).join("Programs").join("TimelessXI")
    } else {
        crate::config::data_dir().join("TimelessXI")
    }
}

/// Where the game files go in a client folder (the folder holding `FINAL FANTASY XI`).
pub fn game_dir_in(dir: &Path) -> PathBuf {
    dir.join("Game")
}

/// Where FINAL FANTASY XI's game files can come from.
#[derive(Clone, Debug, PartialEq)]
pub enum Source {
    /// Game files on this computer (the folder holding `FINAL FANTASY XI`); `writable`: used where they are, else
    /// copied into the client's folder.
    Installed { game_dir: PathBuf, writable: bool },
    /// Square Enix's installer, downloaded already: FFXIFullSetup_US.part1.exe with its four volumes, or a folder it
    /// was unpacked into.
    Installer(PathBuf),
    /// Square Enix's free download.
    Download,
}

impl Source {
    pub fn title(&self) -> &'static str {
        match self {
            Source::Installed { writable: true, .. } => "Use the FINAL FANTASY XI already on this computer",
            Source::Installed { .. } => "Copy the FINAL FANTASY XI already on this computer",
            Source::Installer(_) => "Install from Square Enix's installer in your Downloads",
            Source::Download => "Download FINAL FANTASY XI from Square Enix",
        }
    }

    pub fn detail(&self) -> String {
        match self {
            Source::Installed { game_dir, writable: true } => {
                format!("No download, ready in a moment  ·  {}", game_dir.join("FINAL FANTASY XI").display())
            }
            Source::Installed { game_dir, .. } => {
                format!("No download, copied for Timeless XI  ·  {}", game_dir.join("FINAL FANTASY XI").display())
            }
            Source::Installer(p) => format!("No download  ·  {}", p.display()),
            Source::Download => {
                "The official free client, 7.7 GB  ·  a paused download picks up where it stopped".into()
            }
        }
    }

    /// Bytes the folder it goes to needs, about.
    pub fn space(&self) -> u64 {
        match self {
            Source::Installed { writable: true, .. } => 0,
            Source::Installed { .. } => GAME_BYTES,
            Source::Installer(_) => GAME_BYTES + 3_000_000_000,
            Source::Download => GAME_BYTES + DOWNLOAD_BYTES + 3_000_000_000,
        }
    }
}

/// Game files in `game_dir` (the folder holding `FINAL FANTASY XI`) that are whole: not a run stopped part-way.
pub fn complete(game_dir: &Path) -> bool {
    !super::super::firstrun::needed(&game_dir.join("FINAL FANTASY XI")) && !game_dir.join(MARKER).is_file()
}

/// A folder holding a whole FINAL FANTASY XI can be written to (a test file made and removed).
fn writable(ffxi: &Path) -> bool {
    let probe = ffxi.join(".timeless-write-test");
    let ok = std::fs::write(&probe, b"").is_ok();
    let _ = std::fs::remove_file(&probe);
    ok
}

/// FINAL FANTASY XI installs this computer has: the PlayOnline registry entry, the usual folders, Steam's, and a
/// client's own (the config's and vana-client's).
fn installs() -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    #[cfg(windows)]
    {
        // HKLM\SOFTWARE\WOW6432Node\PlayOnlineUS\InstallFolder, 0001 = the FINAL FANTASY XI folder.
        for key in
            [r"HKLM\SOFTWARE\WOW6432Node\PlayOnlineUS\InstallFolder", r"HKLM\SOFTWARE\PlayOnlineUS\InstallFolder"]
        {
            if let Some(ffxi) = win::reg_query(key, "0001") {
                out.extend(Path::new(&ffxi).parent().map(Path::to_path_buf));
            }
        }
        for base in [r"C:\Program Files (x86)", r"C:\Program Files"] {
            out.push(Path::new(base).join(r"PlayOnline\SquareEnix"));
            out.push(Path::new(base).join(r"Steam\steamapps\common\FFXINA\SquareEnix"));
        }
    }
    out.push(crate::config::Config::load().game_dir);
    out.push(crate::config::data_dir().join("game"));
    let mut seen = Vec::new();
    out.retain(|p| {
        let k = p.to_string_lossy().to_lowercase();
        let new = !seen.contains(&k);
        seen.push(k);
        new && complete(p)
    });
    out
}

/// Square Enix's installer in Downloads: the five volumes whole, or the folder they were unpacked into.
fn installer() -> Option<PathBuf> {
    let dl = dirs::download_dir()?;
    let whole = (1..=VOLUMES).map(|i| std::fs::metadata(dl.join(volume(i))).map(|m| m.len())).sum::<Result<u64, _>>();
    if whole.is_ok_and(|n| n == DOWNLOAD_BYTES) {
        return Some(dl.join(volume(1)));
    }
    let unpacked = dl.join("FFXIFullSetup_US");
    unpacked.join("FINAL_FANTASY_XI").join("FINAL_FANTASY_XI.msi").is_file().then_some(unpacked)
}

/// Every source this computer offers, quickest first; Download is always last.
pub fn sources() -> Vec<Source> {
    // Only a client's own game files are used where they are: the Timeless patches go into the files, and a retail or
    // Steam install must stay as Square Enix left it, so those are copied.
    let own = [crate::config::Config::load().game_dir, crate::config::data_dir().join("game")];
    let mut out: Vec<Source> = installs()
        .into_iter()
        .map(|g| {
            let w = own.contains(&g) && writable(&g.join("FINAL FANTASY XI"));
            Source::Installed { game_dir: g, writable: w }
        })
        .collect();
    // Writable installs (no copy) first.
    out.sort_by_key(|s| !matches!(s, Source::Installed { writable: true, .. }));
    out.extend(installer().map(Source::Installer));
    out.push(Source::Download);
    out
}

/// A folder picked by hand: an install (`FINAL FANTASY XI` or a folder above it) or the installer's.
pub fn source_at(picked: &Path) -> Result<Source> {
    if picked.join("FINAL_FANTASY_XI").join("FINAL_FANTASY_XI.msi").is_file()
        || picked.join("FFXIFullSetup_US").join("FINAL_FANTASY_XI").join("FINAL_FANTASY_XI.msi").is_file()
    {
        return Ok(Source::Installer(picked.to_path_buf()));
    }
    if picked.join(volume(1)).is_file() {
        return Ok(Source::Installer(picked.join(volume(1))));
    }
    let game_dir = super::super::firstrun::existing(picked)?;
    // A folder picked by hand is copied too (see `sources`), unless it is a client's own.
    let own = [crate::config::Config::load().game_dir, crate::config::data_dir().join("game")];
    let w = own.contains(&game_dir) && writable(&game_dir.join("FINAL FANTASY XI"));
    Ok(Source::Installed { game_dir, writable: w })
}

/// Free bytes where `dir` (or the nearest folder above it that exists) is.
pub fn free_space(dir: &Path) -> Option<u64> {
    let at = dir.ancestors().find(|d| d.is_dir())?;
    #[cfg(windows)]
    {
        win::free_space(at)
    }
    #[cfg(not(windows))]
    {
        let _ = at;
        None
    }
}

/// What the player chose.
#[derive(Clone, Debug)]
#[cfg_attr(not(windows), allow(dead_code))]
pub struct Plan {
    pub dir: PathBuf,
    pub source: Source,
    pub desktop: bool,
    pub start_menu: bool,
}

impl Plan {
    /// The folder holding `FINAL FANTASY XI` the client will use.
    pub fn game_dir(&self) -> PathBuf {
        match &self.source {
            Source::Installed { game_dir, writable: true } => game_dir.clone(),
            _ => game_dir_in(&self.dir),
        }
    }

    pub fn exe(&self) -> PathBuf {
        self.dir.join(EXE)
    }
}

/// The install's steps, as the wizard lists them.
pub const STEPS: [&str; 4] = [
    "Installing the Timeless XI client",
    "Getting the game files",
    "Updating the game files",
    "Connecting you to Timeless XI",
];

/// The running install, shared with the wizard.
#[derive(Default)]
pub struct Work {
    pub step: AtomicUsize,
    pub progress: Arc<Progress>,
    /// Set when it ends: Ok(the manifest read, if any) or the error.
    pub end: Mutex<Option<Result<Option<super::manifest::Manifest>, String>>>,
}

fn set(p: &Progress, phase: impl Into<String>, done: u64, total: u64) {
    *p.phase.lock().unwrap() = phase.into();
    p.done.store(done, Ordering::Relaxed);
    p.total.store(total, Ordering::Relaxed);
}

/// Runs the install on a worker thread.
pub fn start(plan: Plan) -> Arc<Work> {
    let work = Arc::new(Work::default());
    let w = work.clone();
    std::thread::spawn(move || {
        let r = run(&plan, &w).map_err(|e| {
            log::warn!("setup: {e:#}");
            format!("{e:#}")
        });
        *w.end.lock().unwrap() = Some(r);
    });
    work
}

fn run(plan: &Plan, w: &Work) -> Result<Option<super::manifest::Manifest>> {
    let p = &w.progress;
    // 1. The client.
    w.step.store(0, Ordering::Relaxed);
    set(p, "Installing the Timeless XI client", 0, 0);
    std::fs::create_dir_all(&plan.dir).with_context(|| plan.dir.display().to_string())?;
    put_client(&plan.exe())?;
    // 2. The game files.
    w.step.store(1, Ordering::Relaxed);
    let game_dir = plan.game_dir();
    if !complete(&game_dir) {
        std::fs::create_dir_all(&game_dir).with_context(|| game_dir.display().to_string())?;
        std::fs::write(game_dir.join(MARKER), b"")?;
        match &plan.source {
            Source::Installed { game_dir: from, writable: false } => copy_game(from, &game_dir, p)?,
            Source::Installed { .. } => {}
            Source::Installer(src) => install_from(src, &game_dir, p)?,
            Source::Download => {
                let dl = game_dir.join(".download");
                download(&dl, p)?;
                install_from(&dl.join(volume(1)), &game_dir, p)?;
                set(p, "Cleaning up the download", 0, 0);
                std::fs::remove_dir_all(&dl).with_context(|| format!("removing {}", dl.display()))?;
            }
        }
        let _ = std::fs::remove_file(game_dir.join(MARKER));
    }
    // 3. Up to the server's version.
    w.step.store(2, Ordering::Relaxed);
    set(p, "Checking the game files", 0, 0);
    let manifest = super::manifest::fetch().map_err(|e| log::warn!("setup: client manifest: {e:#}")).ok();
    let cap = manifest.as_ref().map(|m| m.game_version.clone()).unwrap_or_default();
    update_game(&game_dir.join("FINAL FANTASY XI"), &cap, p)?;
    // 4. Shortcuts and the Installed apps entry (the config is the app's: the wizard writes it on the main thread).
    w.step.store(3, Ordering::Relaxed);
    set(p, "Adding Timeless XI to your computer", 0, 0);
    if let Err(e) = integrate(plan) {
        // Shortcuts are a convenience: the client is installed all the same.
        log::warn!("setup: shortcuts / Installed apps entry: {e:#}");
    }
    Ok(manifest)
}

/// This exe as the installed client (a newer one already there is kept; the same file, when setup runs from it).
fn put_client(dest: &Path) -> Result<()> {
    let me = std::env::current_exe().context("no executable path")?;
    if me.canonicalize().ok() == dest.canonicalize().ok() {
        return Ok(());
    }
    let part = dest.with_extension("part");
    std::fs::copy(&me, &part).with_context(|| format!("copying the client to {}", part.display()))?;
    // A client running from there (an update over a running game) can't be replaced; it is renamed aside first.
    if dest.is_file() {
        let old = dest.with_extension("old");
        let _ = std::fs::remove_file(&old);
        std::fs::rename(dest, &old).context("Timeless XI is running: close it, then try again")?;
    }
    std::fs::rename(&part, dest).with_context(|| dest.display().to_string())?;
    Ok(())
}

/// Square Enix's five volumes into `dl` (resumed: whole volumes are kept, a part file continues).
fn download(dl: &Path, p: &Progress) -> Result<()> {
    std::fs::create_dir_all(dl)?;
    let mut before = 0u64;
    for i in 1..=VOLUMES {
        let dest = dl.join(volume(i));
        if dest.is_file() {
            before += std::fs::metadata(&dest)?.len();
            continue;
        }
        let base = before;
        super::super::updater::download(&format!("{CDN}/{}", volume(i)), &dest, &|d, _| {
            set(
                p,
                format!("Downloading FINAL FANTASY XI from Square Enix (part {i} of {VOLUMES})"),
                base + d,
                DOWNLOAD_BYTES,
            )
        })?;
        before += std::fs::metadata(&dest)?.len();
    }
    Ok(())
}

/// The game files from Square Enix's installer (its first volume, or the folder it was unpacked into).
fn install_from(src: &Path, game_dir: &Path, p: &Arc<Progress>) -> Result<()> {
    let opts = crate::install::Options {
        source: src.to_path_buf(),
        staging: game_dir.join(".staging"),
        game_dir: game_dir.to_path_buf(),
        verify: true,
        keep_staging: false,
    };
    crate::install::install(&opts, p.clone())?;
    let _ = std::fs::remove_dir_all(game_dir.join(".staging"));
    Ok(())
}

/// What a copy of an install takes: `FINAL FANTASY XI`, and the PlayOnline Viewer's name dictionary the character
/// creator reads.
fn copy_list(from: &Path) -> Vec<(PathBuf, PathBuf)> {
    let mut files = Vec::new();
    for top in ["FINAL FANTASY XI", crate::poldic::DIC_DIR] {
        let root = from.join(top);
        let mut stack = vec![root.clone()];
        while let Some(dir) = stack.pop() {
            let Ok(rd) = std::fs::read_dir(&dir) else { continue };
            for e in rd.flatten() {
                let path = e.path();
                match e.file_type() {
                    Ok(t) if t.is_dir() => stack.push(path),
                    Ok(t) if t.is_file() => {
                        let rel = path.strip_prefix(from).unwrap_or(&path).to_path_buf();
                        files.push((path, rel));
                    }
                    _ => {}
                }
            }
        }
    }
    files
}

/// Copies an install's game files into `to` (a file already there at the same size is kept, so a stopped copy goes
/// on where it was).
fn copy_game(from: &Path, to: &Path, p: &Progress) -> Result<()> {
    let files = copy_list(from);
    if files.is_empty() {
        bail!("no game files in {}", from.display());
    }
    let size = |f: &Path| std::fs::metadata(f).map_or(0, |m| m.len());
    let total: u64 = files.iter().map(|(f, _)| size(f)).sum();
    let mut done = 0;
    for (i, (src, rel)) in files.iter().enumerate() {
        let dest = to.join(rel);
        let n = size(src);
        if size(&dest) != n || !dest.is_file() {
            if let Some(d) = dest.parent() {
                std::fs::create_dir_all(d)?;
            }
            let part = dest.with_extension("tlcopy");
            std::fs::copy(src, &part).with_context(|| format!("copying {}", src.display()))?;
            std::fs::rename(&part, &dest)?;
        }
        done += n;
        if i % 25 == 0 || i + 1 == files.len() {
            set(p, format!("Copying the game files ({} of {})", i + 1, files.len()), done, total);
        }
    }
    Ok(())
}

/// Brings the game files up to date, but never past `cap` (the server's game version; empty: no limit).
fn update_game(ffxi: &Path, cap: &str, p: &Progress) -> Result<()> {
    let plan = match crate::patch::plan(ffxi, true, false) {
        Ok(plan) => plan,
        Err(e) => {
            // The PlayOnline patch server down: the installed files play; the title's Update offers it later.
            log::warn!("setup: game update check: {e:#}");
            return Ok(());
        }
    };
    let (server, cap_key) = (crate::patch::version_key(&plan.server.version), crate::patch::version_key(cap));
    if cap_key.is_some() && server > cap_key {
        log::info!("setup: game version {} is past the server's {cap}; left as installed", plan.server.version);
        return Ok(());
    }
    let total: u64 = plan.todo.iter().map(|t| t.2).sum();
    let n = plan.todo.len();
    crate::patch::apply(ffxi, &plan, 4, &|f, b| set(p, format!("Updating the game files ({f} of {n})"), b, total))
}

/// The shortcuts and the Installed apps entry.
fn integrate(plan: &Plan) -> Result<()> {
    #[cfg(windows)]
    {
        let exe = plan.exe();
        if plan.desktop
            && let Some(d) = dirs::desktop_dir()
        {
            win::shortcut(&d.join("Timeless XI.lnk"), &exe)?;
        }
        if plan.start_menu {
            win::shortcut(&start_menu()?.join("Timeless XI.lnk"), &exe)?;
        }
        win::register(plan)?;
    }
    #[cfg(not(windows))]
    {
        let _ = plan;
    }
    Ok(())
}

#[cfg(windows)]
fn start_menu() -> Result<PathBuf> {
    Ok(dirs::config_dir().context("no AppData folder")?.join(r"Microsoft\Windows\Start Menu\Programs"))
}

/// The config a finished setup leaves: the Timeless XI server (the manifest's, else the default), the game files,
/// and the Timeless interface.
pub fn configure(cfg: &mut crate::config::Config, game_dir: &Path, m: Option<&super::manifest::Manifest>) {
    cfg.game_dir = game_dir.to_path_buf();
    cfg.installed = true;
    // A config on another server (a test server) is moved to Timeless XI: this is the Timeless XI install.
    cfg.server = super::manifest::DEFAULT_HOST.into();
    let server = m.and_then(|m| m.server.clone()).unwrap_or_default();
    super::manifest::apply_server(cfg, &server);
    cfg.modern_ui = true;
    cfg.timeless_ui = true;
    // The patched files' version (patch.ver), for the login.
    cfg.follow_applied_version();
}

/// Starts the installed client (from its folder) and leaves the setup.
pub fn launch(exe: &Path) -> Result<()> {
    let mut cmd = std::process::Command::new(exe);
    if let Some(d) = exe.parent() {
        cmd.current_dir(d);
    }
    cmd.spawn().with_context(|| format!("starting {}", exe.display()))?;
    Ok(())
}

/// `TimelessXI.exe uninstall` (Windows Installed apps > Uninstall): asks, then removes the shortcuts, the entry, the
/// client's folder and, if the player says so, the game files the setup put there. Settings and hotbars (beside
/// config.json) stay.
pub fn uninstall() -> Result<()> {
    use rfd::{MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
    let ask = |text: &str| {
        MessageDialog::new()
            .set_level(MessageLevel::Info)
            .set_title("Uninstall Timeless XI")
            .set_description(text)
            .set_buttons(MessageButtons::YesNo)
            .show()
            == MessageDialogResult::Yes
    };
    let exe = std::env::current_exe().context("no executable path")?;
    let dir = exe.parent().context("no folder")?.to_path_buf();
    if !ask("Remove Timeless XI from this computer?\n\nYour settings, hotbars and characters are kept.") {
        return Ok(());
    }
    let game = game_dir_in(&dir);
    let drop_game = game.is_dir()
        && ask(&format!(
            "Also delete the FINAL FANTASY XI game files in\n{}?\n\nKeep them (No) to reinstall later without downloading them again.",
            game.display()
        ));
    #[cfg(windows)]
    {
        if let Some(d) = dirs::desktop_dir() {
            let _ = std::fs::remove_file(d.join("Timeless XI.lnk"));
        }
        if let Ok(m) = start_menu() {
            let _ = std::fs::remove_file(m.join("Timeless XI.lnk"));
        }
        win::unregister();
    }
    // Everything in the folder but the running exe now; the folder itself once this process has gone.
    if let Ok(rd) = std::fs::read_dir(&dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p == exe || (p == game && !drop_game) {
                continue;
            }
            let _ = if p.is_dir() { std::fs::remove_dir_all(&p) } else { std::fs::remove_file(&p) };
        }
    }
    #[cfg(windows)]
    win::remove_after_exit(&exe, &dir, !drop_game && game.is_dir());
    Ok(())
}

#[cfg(windows)]
mod win {
    //! Windows: the registry through reg.exe, shortcuts through the shell's COM object (PowerShell), the free space.
    use anyhow::{Context, Result, bail};
    use std::os::windows::process::CommandExt;
    use std::path::Path;

    const NO_WINDOW: u32 = 0x0800_0000;

    fn run(program: &str, args: &[&str]) -> Result<std::process::Output> {
        let out = std::process::Command::new(program)
            .args(args)
            .creation_flags(NO_WINDOW)
            .output()
            .with_context(|| format!("running {program}"))?;
        Ok(out)
    }

    /// A string value (REG_SZ / REG_EXPAND_SZ) from `reg query`.
    pub fn reg_query(key: &str, value: &str) -> Option<String> {
        let out = run("reg", &["query", key, "/v", value]).ok().filter(|o| o.status.success())?;
        let text = String::from_utf8_lossy(&out.stdout);
        text.lines().find_map(|l| {
            let l = l.trim();
            let rest = l.strip_prefix(value)?.trim_start();
            let rest = rest.strip_prefix("REG_EXPAND_SZ").or_else(|| rest.strip_prefix("REG_SZ"))?;
            Some(rest.trim().to_string()).filter(|s| !s.is_empty())
        })
    }

    fn reg_set(name: &str, kind: &str, data: &str) -> Result<()> {
        let key = format!(r"HKCU\{}", super::UNINSTALL_KEY);
        let out = run("reg", &["add", &key, "/v", name, "/t", kind, "/d", data, "/f"])?;
        if !out.status.success() {
            bail!("reg add {name}: {}", String::from_utf8_lossy(&out.stderr).trim());
        }
        Ok(())
    }

    /// The Installed apps entry (per user: no administrator).
    pub fn register(plan: &super::Plan) -> Result<()> {
        let exe = plan.exe();
        let (exe_s, dir_s) = (exe.display().to_string(), plan.dir.display().to_string());
        let size_kb = std::fs::metadata(&exe).map_or(0, |m| m.len() / 1024)
            + if plan.game_dir().starts_with(&plan.dir) { super::GAME_BYTES / 1024 } else { 0 };
        reg_set("DisplayName", "REG_SZ", "Timeless XI")?;
        reg_set("DisplayVersion", "REG_SZ", env!("CARGO_PKG_VERSION"))?;
        reg_set("Publisher", "REG_SZ", "Timeless XI")?;
        reg_set("URLInfoAbout", "REG_SZ", "https://timelessxi.gg")?;
        reg_set("DisplayIcon", "REG_SZ", &format!("{exe_s},0"))?;
        reg_set("InstallLocation", "REG_SZ", &dir_s)?;
        reg_set("UninstallString", "REG_SZ", &format!("\"{exe_s}\" uninstall"))?;
        reg_set("NoModify", "REG_DWORD", "1")?;
        reg_set("NoRepair", "REG_DWORD", "1")?;
        reg_set("EstimatedSize", "REG_DWORD", &size_kb.min(u32::MAX as u64).to_string())?;
        Ok(())
    }

    pub fn unregister() {
        let key = format!(r"HKCU\{}", super::UNINSTALL_KEY);
        let _ = run("reg", &["delete", &key, "/f"]);
    }

    /// A PowerShell single-quoted string.
    fn ps(s: &str) -> String {
        format!("'{}'", s.replace('\'', "''"))
    }

    pub fn shortcut(lnk: &Path, exe: &Path) -> Result<()> {
        if let Some(d) = lnk.parent() {
            std::fs::create_dir_all(d)?;
        }
        let dir = exe.parent().map(|d| d.display().to_string()).unwrap_or_default();
        let script = format!(
            "$s = (New-Object -ComObject WScript.Shell).CreateShortcut({}); $s.TargetPath = {}; \
             $s.WorkingDirectory = {}; $s.IconLocation = {}; $s.Description = 'Timeless XI'; $s.Save()",
            ps(&lnk.display().to_string()),
            ps(&exe.display().to_string()),
            ps(&dir),
            ps(&format!("{},0", exe.display())),
        );
        let out =
            run("powershell", &["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", &script])?;
        if !out.status.success() || !lnk.is_file() {
            bail!("shortcut {}: {}", lnk.display(), String::from_utf8_lossy(&out.stderr).trim());
        }
        Ok(())
    }

    pub fn free_space(dir: &Path) -> Option<u64> {
        use std::os::windows::ffi::OsStrExt;
        let wide: Vec<u16> = dir.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
        let mut free = 0u64;
        // SAFETY: a NUL-terminated path; the out pointers are valid or null.
        let ok = unsafe {
            windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW(
                wide.as_ptr(),
                &mut free,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
        (ok != 0).then_some(free)
    }

    /// The running exe and its folder go once this process has ended (a running exe can't be deleted); `keep_game`:
    /// the folder stays for the kept game files.
    pub fn remove_after_exit(exe: &Path, dir: &Path, keep_game: bool) {
        let tail = if keep_game { String::new() } else { format!(" & rmdir /s /q \"{}\"", dir.display()) };
        let line = format!("ping 127.0.0.1 -n 3 > nul & del /f /q \"{}\"{tail}", exe.display());
        let _ = std::process::Command::new("cmd").args(["/c", &line]).creation_flags(NO_WINDOW).spawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_picked_install_or_installer_is_recognised() {
        let base = std::env::temp_dir().join(format!("tl-setup-{}", std::process::id()));
        let game = base.join("PlayOnline").join("SquareEnix");
        std::fs::create_dir_all(game.join("FINAL FANTASY XI")).unwrap();
        std::fs::write(game.join("FINAL FANTASY XI").join("VTABLE.DAT"), b"").unwrap();
        assert_eq!(source_at(&base).unwrap(), Source::Installed { game_dir: game.clone(), writable: false });
        let dl = base.join("dl");
        std::fs::create_dir_all(&dl).unwrap();
        std::fs::write(dl.join(volume(1)), b"").unwrap();
        assert_eq!(source_at(&dl).unwrap(), Source::Installer(dl.join(volume(1))));
        assert!(source_at(&base.join("nothing")).is_err());
        // A stopped copy isn't taken as whole.
        assert!(complete(&game));
        std::fs::write(game.join(MARKER), b"").unwrap();
        assert!(!complete(&game));
        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn a_copy_takes_the_game_and_the_name_dictionary_and_resumes() {
        let base = std::env::temp_dir().join(format!("tl-copy-{}", std::process::id()));
        let (from, to) = (base.join("from"), base.join("to"));
        std::fs::create_dir_all(from.join("FINAL FANTASY XI").join("ROM").join("1")).unwrap();
        std::fs::create_dir_all(from.join(crate::poldic::DIC_DIR)).unwrap();
        std::fs::write(from.join("FINAL FANTASY XI").join("VTABLE.DAT"), b"vtable").unwrap();
        std::fs::write(from.join("FINAL FANTASY XI").join("ROM").join("1").join("2.DAT"), b"dat").unwrap();
        std::fs::write(from.join(crate::poldic::DIC_DIR).join("entryz.dic"), b"dic").unwrap();
        std::fs::write(from.join("unrelated.txt"), b"x").unwrap();
        let p = Progress::default();
        copy_game(&from, &to, &p).unwrap();
        assert_eq!(std::fs::read(to.join("FINAL FANTASY XI").join("ROM").join("1").join("2.DAT")).unwrap(), b"dat");
        assert!(to.join(crate::poldic::DIC_DIR).join("entryz.dic").is_file());
        assert!(!to.join("unrelated.txt").exists());
        assert_eq!(p.total.load(Ordering::Relaxed), 12);
        copy_game(&from, &to, &p).unwrap();
        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn plan_uses_a_writable_install_in_place() {
        let dir = PathBuf::from("/x/TimelessXI");
        let inplace = Plan {
            dir: dir.clone(),
            source: Source::Installed { game_dir: "/g".into(), writable: true },
            desktop: true,
            start_menu: true,
        };
        assert_eq!(inplace.game_dir(), PathBuf::from("/g"));
        let copied = Plan { source: Source::Installed { game_dir: "/g".into(), writable: false }, ..inplace.clone() };
        assert_eq!(copied.game_dir(), dir.join("Game"));
        assert_eq!(Plan { source: Source::Download, ..inplace }.exe(), dir.join(EXE));
    }

    #[test]
    fn setup_configures_the_timeless_server() {
        let mut cfg = crate::config::Config { server: "127.0.0.1".into(), ..Default::default() };
        configure(&mut cfg, Path::new("/g"), None);
        assert_eq!(cfg.server, super::super::manifest::DEFAULT_HOST);
        assert!(cfg.timeless_ui && cfg.modern_ui && cfg.installed);
        assert_eq!(cfg.game_dir, PathBuf::from("/g"));
    }
}
