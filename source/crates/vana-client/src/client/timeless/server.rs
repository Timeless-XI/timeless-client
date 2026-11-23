//! TimelessXI (not vana-client): what the standalone TimelessXI client brings up to date at every start, before the
//! game files and the Lua mods are read.
//!
//! - **The Timeless client patches** (Records of Eminence, the linkshell icon, custom items, item adjustments, key
//!   items): small byte regions written into the game's own files, plus a few files of Timeless's own. The set is the
//!   one the Timeless Launcher installs, published at timelessxi.gg/patches/latest.json (timeless-tools
//!   ClientPatch/publish_patches.py), and applied by the launcher's rules (TimelessLauncher/src/Patcher.cs): a
//!   region is written only where the game's bytes are still Square Enix's (its oldSha1), a region already patched
//!   (newSha1) is left, anything else is a different game version and is left alone. The original bytes are kept, so
//!   a newer patch set can put them back first. Only Timeless's bytes are downloaded, never Square Enix's files.
//! - **The required mods** of the mod list (catalogue.rs, `"required": true`): installed or updated, and switched on
//!   when first installed.
//!
//! Nothing here stops the client: without the network the last downloaded patch set is used, and every failure is
//! logged and left for the next start.

use super::super::catalogue;
use crate::config::{Config, ModRef};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicU32;
use std::sync::mpsc::channel;
use std::time::Duration;

/// Where the Timeless Launcher, and this client, read which patch set the server runs.
pub const PATCH_FEED: &str = "https://timelessxi.gg/patches/latest.json";
/// The most of a patch set file taken (the set is about 1 MB).
const MAX_PATCH_FILE: u64 = 64 << 20;
/// How long a start waits for the required mods before it goes on without them (the patches always finish first).
const MODS_WAIT: Duration = Duration::from_secs(45);

/// An agent that gives up: a start must never hang on a server that doesn't answer.
fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .user_agent(concat!("vana-client/", env!("CARGO_PKG_VERSION")))
        .timeout_global(Some(Duration::from_secs(20)))
        .build()
        .into()
}

fn get(url: &str) -> Result<Vec<u8>> {
    Ok(agent()
        .get(url)
        .header("Cache-Control", "no-cache")
        .call()
        .with_context(|| url.to_string())?
        .body_mut()
        .with_config()
        .limit(MAX_PATCH_FILE)
        .read_to_vec()?)
}

/// The mod list's text, through the agent that gives up.
fn get_text(url: &str) -> Result<String> {
    Ok(String::from_utf8(get(url)?)?)
}

fn sha1(b: &[u8]) -> String {
    hex::encode(ring::digest::digest(&ring::digest::SHA1_FOR_LEGACY_USE_ONLY, b))
}

// ---------------------------------------------------------------- the patch set

#[derive(Deserialize)]
struct Feed {
    version: String,
    json: Asset,
    bin: Asset,
    /// The HD faces pack (too large for the main set): its own manifest and gzipped data, downloaded in the
    /// background and written in at the next start. Older clients and the launcher ignore it.
    #[serde(default)]
    hd: Option<Pack>,
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Debug)]
struct Pack {
    version: String,
    json: Asset,
    bin: Asset,
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Debug)]
struct Asset {
    url: String,
    sha1: String,
}

#[derive(Deserialize)]
pub struct PatchSet {
    pub version: String,
    #[serde(rename = "blobSize")]
    blob_size: u64,
    files: Vec<PatchFile>,
    #[serde(skip)]
    blob: Vec<u8>,
}

#[derive(Deserialize)]
struct PatchFile {
    path: String,
    /// "edit": regions of a game file; "new": a file of Timeless's own, stored whole.
    kind: String,
    #[serde(default, rename = "newSha1")]
    new_sha1: String,
    #[serde(default)]
    data: u64,
    #[serde(default)]
    length: u64,
    #[serde(default)]
    regions: Vec<Region>,
}

#[derive(Deserialize)]
struct Region {
    offset: u64,
    length: usize,
    #[serde(rename = "oldSha1")]
    old_sha1: String,
    #[serde(rename = "newSha1")]
    new_sha1: String,
    data: u64,
}

impl PatchSet {
    /// A set's manifest and its bytes, checked against each other.
    pub fn parse(json: &[u8], blob: Vec<u8>) -> Result<Self> {
        let mut ps: PatchSet = serde_json::from_slice(json).context("reading the Timeless patch set")?;
        if ps.blob_size != blob.len() as u64 {
            bail!("the Timeless patch data is {} bytes, not {}", blob.len(), ps.blob_size);
        }
        for f in &ps.files {
            if !safe_rel(&f.path) {
                bail!("the Timeless patch set names {:?}, outside the game folder", f.path);
            }
            let end = |data: u64, len: u64| data.checked_add(len).is_none_or(|e| e > blob.len() as u64);
            if (f.kind == "new" && end(f.data, f.length)) || f.regions.iter().any(|r| end(r.data, r.length as u64)) {
                bail!("the Timeless patch set's {} points past its data", f.path);
            }
        }
        ps.blob = blob;
        Ok(ps)
    }

    fn bytes(&self, data: u64, len: u64) -> &[u8] {
        &self.blob[data as usize..(data + len) as usize]
    }
}

/// A game file's path in the set: relative, forward slashes, no "..".
fn safe_rel(p: &str) -> bool {
    !p.is_empty()
        && !p.starts_with('/')
        && !p.contains(['\\', ':'])
        && p.split('/').all(|part| !part.is_empty() && part != "." && part != "..")
}

/// A version as a folder name (latest.json is trusted no further than that).
fn safe_version(v: &str) -> bool {
    !v.is_empty() && v.len() <= 64 && v.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_')
}

/// The patch sets downloaded, the version installed and the original bytes: beside the config.
fn dir() -> PathBuf {
    crate::config::data_dir().join("timeless-patches")
}

const MANIFEST: &str = "timeless_patches.json";
const BLOB: &str = "timeless_patches.bin";

/// The set `version` as downloaded before.
fn cached(version: &str) -> Result<PatchSet> {
    let d = dir().join(version);
    let json = std::fs::read(d.join(MANIFEST)).with_context(|| format!("{}", d.join(MANIFEST).display()))?;
    PatchSet::parse(&json, std::fs::read(d.join(BLOB))?)
}

/// The newest set: the server's (downloaded when it is new), else the last one downloaded.
pub fn latest() -> Result<PatchSet> {
    match fetch() {
        Ok(ps) => Ok(ps),
        Err(e) => {
            let last = std::fs::read_to_string(dir().join("latest.txt")).map(|s| s.trim().to_string());
            match last {
                Ok(v) if safe_version(&v) => {
                    log::info!("timeless patches: {e:#}; using {v}, downloaded before");
                    cached(&v)
                }
                _ => Err(e),
            }
        }
    }
}

fn fetch() -> Result<PatchSet> {
    let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
    let feed: Feed =
        serde_json::from_slice(&get(&format!("{PATCH_FEED}?t={stamp}"))?).context("reading latest.json")?;
    if !safe_version(&feed.version) {
        bail!("latest.json names version {:?}", feed.version);
    }
    let d = dir().join(&feed.version);
    let have =
        |name: &str, want: &str| std::fs::read(d.join(name)).ok().filter(|b| sha1(b) == want.to_ascii_lowercase());
    let json = match have(MANIFEST, &feed.json.sha1) {
        Some(b) => b,
        None => download(&feed.json, &d.join(MANIFEST))?,
    };
    let blob = match have(BLOB, &feed.bin.sha1) {
        Some(b) => b,
        None => download(&feed.bin, &d.join(BLOB))?,
    };
    let ps = PatchSet::parse(&json, blob)?;
    let _ = std::fs::write(dir().join("latest.txt"), &feed.version);
    // The HD pack the feed names now (none: the one installed comes out at this start).
    let want = hd_dir().join(HD_WANT);
    match feed.hd.filter(|p| safe_version(&p.version)) {
        Some(p) => {
            let _ = std::fs::create_dir_all(hd_dir());
            let _ = serde_json::to_vec(&p).map(|b| std::fs::write(&want, b));
        }
        None => {
            let _ = std::fs::remove_file(&want);
        }
    }
    Ok(ps)
}

// ---------------------------------------------------------------- the HD faces pack

const HD_WANT: &str = "wanted.json";
const HD_BLOB: &str = "timeless_hd.bin.gz";
/// The pack's data, gzipped and unpacked.
const MAX_HD_FILE: u64 = 192 << 20;
const MAX_HD_DATA: u64 = 512 << 20;

/// The HD pack's downloads and its own installed state (apart from the main set's).
fn hd_dir() -> PathBuf {
    dir().join("hd")
}

fn hd_wanted() -> Option<Pack> {
    serde_json::from_slice(&std::fs::read(hd_dir().join(HD_WANT)).ok()?).ok()
}

/// Pack `p` as downloaded before, unpacked; None when it isn't all there.
fn hd_cached(p: &Pack) -> Option<Result<PatchSet>> {
    let d = hd_dir().join(&p.version);
    let have =
        |name: &str, want: &str| std::fs::read(d.join(name)).ok().filter(|b| sha1(b) == want.to_ascii_lowercase());
    let json = have(MANIFEST, &p.json.sha1)?;
    let gz = have(HD_BLOB, &p.bin.sha1)?;
    Some((|| {
        let mut blob = Vec::new();
        flate2::read::GzDecoder::new(&gz[..])
            .take(MAX_HD_DATA)
            .read_to_end(&mut blob)
            .context("unpacking the HD faces")?;
        PatchSet::parse(&json, blob)
    })())
}

/// Downloads pack `p` (no hurry: a slow line takes its time, and the next start writes it in).
fn hd_download(p: &Pack) -> Result<()> {
    hd_download_with(p, &|_, _| {})
}

/// `hd_download`, telling `on` the bytes read of the pack's data and its size.
fn hd_download_with(p: &Pack, on: &dyn Fn(u64, u64)) -> Result<()> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .user_agent(concat!("vana-client/", env!("CARGO_PKG_VERSION")))
        .timeout_connect(Some(Duration::from_secs(20)))
        .timeout_global(Some(Duration::from_secs(30 * 60)))
        .build()
        .into();
    let d = hd_dir().join(&p.version);
    for (a, name, limit) in [(&p.json, MANIFEST, MAX_PATCH_FILE), (&p.bin, HD_BLOB, MAX_HD_FILE)] {
        if std::fs::read(d.join(name)).is_ok_and(|b| sha1(&b) == a.sha1.to_ascii_lowercase()) {
            continue;
        }
        let mut res = agent.get(&a.url).call().with_context(|| a.url.clone())?;
        let size = res.body().content_length().unwrap_or(0);
        let mut r = res.body_mut().with_config().limit(limit).reader();
        let (mut b, mut buf) = (Vec::with_capacity(size as usize), vec![0u8; 1 << 16]);
        loop {
            let n = r.read(&mut buf)?;
            if n == 0 {
                break;
            }
            b.extend_from_slice(&buf[..n]);
            if name == HD_BLOB {
                on(b.len() as u64, size);
            }
        }
        if sha1(&b) != a.sha1.to_ascii_lowercase() {
            bail!("{} is not the file latest.json names (its SHA-1 differs)", a.url);
        }
        std::fs::create_dir_all(&d)?;
        let part = d.join(format!("{name}.part"));
        std::fs::write(&part, &b)?;
        std::fs::rename(&part, d.join(name))?;
    }
    // Older packs' downloads go (their originals stay in the state until the new pack replaces them).
    if let Ok(rd) = std::fs::read_dir(hd_dir()) {
        for e in rd.flatten() {
            if e.path().is_dir() && e.file_name() != std::ffi::OsStr::new(&p.version) {
                let _ = std::fs::remove_dir_all(e.path());
            }
        }
    }
    Ok(())
}

/// The setup wizard: the main set and the HD faces pack written into the new game now (downloaded with progress), so
/// the first start already has them. `on`: (what, bytes done, total). A failure leaves it to the client's first start.
pub fn install_all(ffxi: &Path, on: &dyn Fn(&str, u64, u64)) -> Result<()> {
    on("Adding the Timeless patches", 0, 0);
    let ps = latest()?;
    let rep = apply(&ps, ffxi, &dir())?;
    log::info!("setup: timeless patches {}: {} written", ps.version, rep.written);
    let Some(p) = hd_wanted() else { return Ok(()) };
    if hd_cached(&p).is_none() {
        hd_download_with(&p, &|d, t| on("Downloading the HD faces", d, t))?;
    }
    on("Adding the HD faces", 0, 0);
    let ps = hd_cached(&p).context("the HD faces did not download")??;
    let rep = apply(&ps, ffxi, &hd_dir().join("state"))?;
    log::info!("setup: HD faces {}: {} written", ps.version, rep.written);
    Ok(())
}

/// At start, after the main set: the HD pack written in when it is downloaded, else downloaded in the background for
/// the next start; taken out when the feed no longer names one.
fn hd_start(ffxi: &Path) {
    let state = hd_dir().join("state");
    let Some(p) = hd_wanted() else {
        if installed_path(&state).is_file() {
            match restore(ffxi, &state) {
                Ok(()) => log::info!("timeless HD faces: taken out (the feed names none)"),
                Err(e) => log::warn!("timeless HD faces: taking out: {e:#}"),
            }
        }
        return;
    };
    match hd_cached(&p) {
        Some(Ok(ps)) => match apply(&ps, ffxi, &state) {
            Ok(rep) => log::info!(
                "timeless HD faces {}: {} written, {} already there, {} left",
                ps.version,
                rep.written,
                rep.done,
                rep.foreign
            ),
            Err(e) => log::warn!("timeless HD faces: {e:#}"),
        },
        Some(Err(e)) => log::warn!("timeless HD faces: {e:#}"),
        None => {
            std::thread::spawn(move || match hd_download(&p) {
                Ok(()) => log::info!("timeless HD faces {}: downloaded; written in at the next start", p.version),
                Err(e) => log::warn!("timeless HD faces: {e:#}"),
            });
        }
    }
}

fn download(a: &Asset, dest: &Path) -> Result<Vec<u8>> {
    let b = get(&a.url)?;
    if sha1(&b) != a.sha1.to_ascii_lowercase() {
        bail!("{} is not the file latest.json names (its SHA-1 differs)", a.url);
    }
    std::fs::create_dir_all(dest.parent().unwrap_or(Path::new(".")))?;
    std::fs::write(dest, &b)?;
    Ok(b)
}

// ---------------------------------------------------------------- applying

#[derive(Debug, Default, PartialEq)]
pub struct Report {
    /// Regions (and whole files) written now.
    pub written: usize,
    /// Already patched.
    pub done: usize,
    /// Neither Square Enix's bytes nor ours: another game version, or a file another tool changed.
    pub foreign: usize,
}

/// Where the original bytes go: one record per region written, (path, offset, length, our bytes' SHA-1, the
/// original bytes), so they are put back only where our bytes still are.
fn backup_path(state: &Path) -> PathBuf {
    state.join("originals.bin")
}

fn added_path(state: &Path) -> PathBuf {
    state.join("added.txt")
}

fn installed_path(state: &Path) -> PathBuf {
    state.join("installed.txt")
}

fn read_at(f: &mut std::fs::File, offset: u64, len: usize) -> Result<Vec<u8>> {
    let mut b = vec![0; len];
    f.seek(SeekFrom::Start(offset))?;
    f.read_exact(&mut b)?;
    Ok(b)
}

/// Writes set `ps` into the game at `ffxi` (the "FINAL FANTASY XI" folder), keeping what it needs in `state`. A set
/// other than the one installed first has the old one's bytes put back.
pub fn apply(ps: &PatchSet, ffxi: &Path, state: &Path) -> Result<Report> {
    if !ffxi.join("ROM").is_dir() {
        bail!("no game at {}", ffxi.display());
    }
    std::fs::create_dir_all(state)?;
    let installed = std::fs::read_to_string(installed_path(state)).ok().map(|s| s.trim().to_string());
    if installed.as_deref().is_some_and(|v| v != ps.version) {
        restore(ffxi, state).context("putting back the older Timeless patch set")?;
    }
    let mut rep = Report::default();
    let mut originals = std::fs::OpenOptions::new().create(true).append(true).open(backup_path(state))?;
    for f in &ps.files {
        let path = ffxi.join(&f.path);
        if f.kind == "new" {
            if std::fs::read(&path).is_ok_and(|b| sha1(&b) == f.new_sha1) {
                rep.done += 1;
                continue;
            }
            std::fs::create_dir_all(path.parent().unwrap_or(ffxi))?;
            std::fs::write(&path, ps.bytes(f.data, f.length)).with_context(|| path.display().to_string())?;
            writeln!(std::fs::OpenOptions::new().create(true).append(true).open(added_path(state))?, "{}", f.path)?;
            rep.written += 1;
            continue;
        }
        if f.kind != "edit" {
            continue;
        }
        let Ok(mut file) = std::fs::OpenOptions::new().read(true).write(true).open(&path) else {
            if path.is_file() {
                bail!("{} can't be written (is the game folder read-only?)", path.display());
            }
            rep.foreign += f.regions.len();
            continue;
        };
        let len = file.metadata()?.len();
        for r in &f.regions {
            if r.offset + r.length as u64 > len {
                rep.foreign += 1;
                continue;
            }
            let cur = read_at(&mut file, r.offset, r.length)?;
            let h = sha1(&cur);
            if h == r.new_sha1 {
                rep.done += 1;
            } else if h == r.old_sha1 {
                let mut rec = Vec::with_capacity(f.path.len() + r.length + 54);
                rec.extend_from_slice(&(f.path.len() as u16).to_le_bytes());
                rec.extend_from_slice(f.path.as_bytes());
                rec.extend_from_slice(&r.offset.to_le_bytes());
                rec.extend_from_slice(&(r.length as u32).to_le_bytes());
                rec.extend_from_slice(r.new_sha1.as_bytes().get(..40).context("a region's newSha1 is too short")?);
                rec.extend_from_slice(&cur);
                originals.write_all(&rec)?;
                originals.flush()?;
                file.seek(SeekFrom::Start(r.offset))?;
                file.write_all(ps.bytes(r.data, r.length as u64))?;
                rep.written += 1;
            } else {
                rep.foreign += 1;
            }
        }
    }
    std::fs::write(installed_path(state), &ps.version)?;
    Ok(rep)
}

/// `n` bytes of `b` at `at`, moving `at` past them.
fn take<'a>(b: &'a [u8], at: &mut usize, n: usize) -> Result<&'a [u8]> {
    let s = b.get(*at..*at + n).context("the kept original bytes are cut short")?;
    *at += n;
    Ok(s)
}

/// Puts back the original bytes `apply` kept, where our bytes are still there, and removes the files it added.
pub fn restore(ffxi: &Path, state: &Path) -> Result<()> {
    if let Ok(b) = std::fs::read(backup_path(state)) {
        let mut at = 0usize;
        while at < b.len() {
            let n = u16::from_le_bytes(take(&b, &mut at, 2)?.try_into()?) as usize;
            let rel = String::from_utf8(take(&b, &mut at, n)?.to_vec())?;
            let offset = u64::from_le_bytes(take(&b, &mut at, 8)?.try_into()?);
            let len = u32::from_le_bytes(take(&b, &mut at, 4)?.try_into()?) as usize;
            let ours = String::from_utf8(take(&b, &mut at, 40)?.to_vec())?;
            let orig = take(&b, &mut at, len)?;
            if !safe_rel(&rel) {
                continue;
            }
            let Ok(mut file) = std::fs::OpenOptions::new().read(true).write(true).open(ffxi.join(&rel)) else {
                continue;
            };
            if read_at(&mut file, offset, len).is_ok_and(|cur| sha1(&cur) == ours) {
                file.seek(SeekFrom::Start(offset))?;
                file.write_all(orig)?;
            }
        }
        std::fs::remove_file(backup_path(state))?;
    }
    if let Ok(list) = std::fs::read_to_string(added_path(state)) {
        for rel in list.lines().map(str::trim).filter(|r| safe_rel(r)) {
            let _ = std::fs::remove_file(ffxi.join(rel));
        }
        std::fs::remove_file(added_path(state))?;
    }
    let _ = std::fs::remove_file(installed_path(state));
    Ok(())
}

/// The newest patch set written into the game at `ffxi`; the report, logged.
pub fn patch_game(ffxi: &Path) -> Result<Report> {
    let ps = latest()?;
    let rep = apply(&ps, ffxi, &dir())?;
    hd_start(ffxi);
    log::info!(
        "timeless patches {}: {} written, {} already there, {} left (another game version)",
        ps.version,
        rep.written,
        rep.done,
        rep.foreign
    );
    Ok(rep)
}

/// After a game update: the patch set downloaded last, written again where the update put Square Enix's bytes back.
pub fn repatch(ffxi: &Path) -> Result<Report> {
    let v = std::fs::read_to_string(dir().join("latest.txt")).context("no Timeless patch set downloaded yet")?;
    let v = v.trim();
    if !safe_version(v) {
        bail!("latest.txt names version {v:?}");
    }
    let rep = apply(&cached(v)?, ffxi, &dir())?;
    if let Some(Ok(ps)) = hd_wanted().as_ref().and_then(hd_cached) {
        apply(&ps, ffxi, &hd_dir().join("state"))?;
    }
    Ok(rep)
}

// ---------------------------------------------------------------- game updates

/// Where TimelessXI says which retail game version players may update to: catalogue/game.json in its releases
/// repository, `{"version": "30261102_0"}`. Raised by hand once a retail update has been tried on the test server.
pub fn game_json_url() -> String {
    format!("https://raw.githubusercontent.com/{}/main/catalogue/game.json", super::super::updater::RELEASE_REPO)
}

/// The newest retail game version TimelessXI allows.
pub fn allowed_game_version() -> Result<String> {
    let v: serde_json::Value = serde_json::from_str(&get_text(&game_json_url())?).context("reading game.json")?;
    let s = v.get("version").and_then(|x| x.as_str()).context("game.json names no version")?;
    if crate::patch::version_key(s).is_none() {
        bail!("game.json's version {s:?} is not a game version (like 30261102_0)");
    }
    Ok(s.to_string())
}

/// Whether a game update to retail's `latest` version may be offered: when TimelessXI allows that version, or when
/// it is no newer than the version this install already has (a repair, not an update). Not knowing what TimelessXI
/// allows (no network, a bad file) holds it back.
pub fn game_update_allowed(latest: &str, allowed: Option<&str>, applied: Option<&str>) -> bool {
    let Some(l) = crate::patch::version_key(latest) else { return false };
    let within = |v: Option<&str>| v.and_then(crate::patch::version_key).is_some_and(|k| l <= k);
    within(allowed) || within(applied)
}

// ---------------------------------------------------------------- the start

/// The required mods installed or brought up to date; the ids of those the list requires.
fn required_mods() -> Result<Vec<String>> {
    let root = crate::config::mods_dir();
    let list = catalogue::fetch_with(catalogue::List::Mods, get_text)?;
    let mut ids = Vec::new();
    for e in list.iter().filter(|e| e.required) {
        ids.push(e.id.clone());
        let have = catalogue::installed(&root, &e.id);
        if catalogue::offer(e, have.as_deref()).is_some() {
            match catalogue::install(e, &root, &AtomicU32::new(0)) {
                Ok(_) => log::info!("timeless: required mod {} {} installed", e.id, e.version),
                Err(err) => log::warn!("timeless: required mod {}: {err:#}", e.id),
            }
        }
    }
    Ok(ids)
}

/// At start, before the game files and the Lua mods are read: the patches written into the game (when it is
/// installed), then the required mods, each one there switched on in `cfg` the first time. True when `cfg` changed.
pub fn start(cfg: &mut Config) -> bool {
    let ffxi = cfg.ffxi_dir();
    if ffxi.join("VTABLE.DAT").is_file()
        && let Err(e) = patch_game(&ffxi)
    {
        log::warn!("timeless patches: {e:#}");
    }
    // The mods over the network may be slow: the start waits a while, then goes on (an install still running
    // finishes in its folder's work copy and is picked up next time).
    let (tx, rx) = channel();
    std::thread::spawn(move || {
        let _ = tx.send(required_mods());
    });
    let ids = match rx.recv_timeout(MODS_WAIT) {
        Ok(Ok(ids)) => ids,
        Ok(Err(e)) => {
            log::warn!("timeless: the mod list: {e:#}");
            return false;
        }
        Err(_) => {
            log::warn!("timeless: the required mods took too long; going on without waiting");
            return false;
        }
    };
    let root = crate::config::mods_dir();
    let mut changed = false;
    for id in ids {
        if root.join(&id).join("mod.toml").is_file() && !cfg.mods.iter().any(|m| m.name == id) {
            cfg.mods.insert(0, ModRef::new(id, true));
            changed = true;
        }
    }
    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The feed names the HD pack beside the main set; a feed without one still reads.
    #[test]
    fn feed_with_and_without_hd() {
        let asset = r#"{"url": "https://timelessxi.gg/x", "sha1": "ab", "size": 1}"#;
        let plain = format!(r#"{{"version": "1", "json": {asset}, "bin": {asset}}}"#);
        assert!(serde_json::from_str::<Feed>(&plain).unwrap().hd.is_none());
        let hd = format!(
            r#"{{"version": "1", "json": {asset}, "bin": {asset}, "hd": {{"version": "hd-1", "json": {asset}, "bin": {asset}}}}}"#
        );
        let f: Feed = serde_json::from_str(&hd).unwrap();
        assert_eq!(f.hd.unwrap().version, "hd-1");
    }

    /// A game folder with one game file, and a patch set of two regions in it plus a new file.
    fn game(tag: &str) -> (PathBuf, PathBuf, PatchSet) {
        let base = std::env::temp_dir().join(format!("vana-timeless-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let ffxi = base.join("FINAL FANTASY XI");
        std::fs::create_dir_all(ffxi.join("ROM/1")).unwrap();
        std::fs::write(ffxi.join("ROM/1/2.DAT"), b"0123456789abcdef").unwrap();
        let blob = b"XYZnew file".to_vec();
        let json = serde_json::json!({
            "version": "1", "blobSize": blob.len(), "groups": ["g"],
            "files": [
                {"path": "ROM/1/2.DAT", "kind": "edit", "regions": [
                    {"group": "g", "offset": 2, "length": 2, "oldSha1": sha1(b"23"), "newSha1": sha1(b"XY"), "data": 0},
                    {"group": "g", "offset": 10, "length": 1, "oldSha1": sha1(b"a"), "newSha1": sha1(b"Z"), "data": 2}]},
                {"path": "ROM/450/1.DAT", "kind": "new", "group": "g", "newSha1": sha1(b"new file"), "data": 3, "length": 8}
            ]
        });
        let ps = PatchSet::parse(json.to_string().as_bytes(), blob).unwrap();
        (ffxi, base.join("state"), ps)
    }

    /// Applied once it writes, a second time it finds them done, restored the file is Square Enix's again.
    #[test]
    fn applies_once_and_restores() {
        let (ffxi, state, ps) = game("apply");
        assert_eq!(apply(&ps, &ffxi, &state).unwrap(), Report { written: 3, done: 0, foreign: 0 });
        assert_eq!(std::fs::read(ffxi.join("ROM/1/2.DAT")).unwrap(), b"01XY456789Zbcdef");
        assert_eq!(std::fs::read(ffxi.join("ROM/450/1.DAT")).unwrap(), b"new file");
        assert_eq!(apply(&ps, &ffxi, &state).unwrap(), Report { written: 0, done: 3, foreign: 0 });
        restore(&ffxi, &state).unwrap();
        assert_eq!(std::fs::read(ffxi.join("ROM/1/2.DAT")).unwrap(), b"0123456789abcdef");
        assert!(!ffxi.join("ROM/450/1.DAT").exists());
        let _ = std::fs::remove_dir_all(ffxi.parent().unwrap());
    }

    /// Bytes that are neither theirs nor ours are left; a game update that put theirs back is patched again.
    #[test]
    fn leaves_other_versions_and_repatches() {
        let (ffxi, state, ps) = game("foreign");
        std::fs::write(ffxi.join("ROM/1/2.DAT"), b"01QQ456789abcdef").unwrap();
        assert_eq!(apply(&ps, &ffxi, &state).unwrap(), Report { written: 2, done: 0, foreign: 1 });
        std::fs::write(ffxi.join("ROM/1/2.DAT"), b"0123456789Zbcdef").unwrap();
        assert_eq!(apply(&ps, &ffxi, &state).unwrap(), Report { written: 1, done: 2, foreign: 0 });
        let _ = std::fs::remove_dir_all(ffxi.parent().unwrap());
    }

    /// A different set: the old one's bytes go back first, then the new one is written.
    #[test]
    fn a_new_version_replaces_the_old() {
        let (ffxi, state, ps) = game("version");
        apply(&ps, &ffxi, &state).unwrap();
        let blob = b"QQ".to_vec();
        let json = serde_json::json!({"version": "2", "blobSize": 2, "files": [{"path": "ROM/1/2.DAT", "kind": "edit",
            "regions": [{"offset": 4, "length": 2, "oldSha1": sha1(b"45"), "newSha1": sha1(b"QQ"), "data": 0}]}]});
        let ps2 = PatchSet::parse(json.to_string().as_bytes(), blob).unwrap();
        assert_eq!(apply(&ps2, &ffxi, &state).unwrap(), Report { written: 1, done: 0, foreign: 0 });
        assert_eq!(std::fs::read(ffxi.join("ROM/1/2.DAT")).unwrap(), b"0123QQ6789abcdef");
        assert!(!ffxi.join("ROM/450/1.DAT").exists());
        let _ = std::fs::remove_dir_all(ffxi.parent().unwrap());
    }

    /// Retail updates wait for TimelessXI's allowed version; repairs at the installed version don't.
    #[test]
    fn game_updates_wait_for_the_server() {
        let ok = |latest, allowed, applied| game_update_allowed(latest, allowed, applied);
        assert!(ok("30261102_0", Some("30261102_0"), Some("30261006_0")), "the allowed version");
        assert!(ok("30261006_0", Some("30261102_0"), Some("30261006_0")), "older than allowed");
        assert!(!ok("30261201_0", Some("30261102_0"), Some("30261102_0")), "newer than allowed: held");
        assert!(!ok("30261201_0", None, Some("30261102_0")), "unknown allowed version: held");
        assert!(ok("30261201_0", None, Some("30261201_0")), "a repair at the installed version");
        assert!(!ok("30261102_1", Some("30261102_0"), None), "a later build of the same day: held");
        assert!(!ok("garbage", Some("30261102_0"), None), "unreadable version: held");
    }

    /// Paths outside the game folder and data past the blob are refused.
    #[test]
    fn bad_sets_refused() {
        let j = |path: &str, data: u64| {
            serde_json::json!({"version": "1", "blobSize": 4, "files": [{"path": path, "kind": "new", "newSha1": "",
                "data": data, "length": 4}]})
            .to_string()
        };
        assert!(PatchSet::parse(j("ROM/1.DAT", 0).as_bytes(), vec![0; 4]).is_ok());
        assert!(PatchSet::parse(j("../x", 0).as_bytes(), vec![0; 4]).is_err());
        assert!(PatchSet::parse(j("C:/x", 0).as_bytes(), vec![0; 4]).is_err());
        assert!(PatchSet::parse(j("ROM/1.DAT", 1).as_bytes(), vec![0; 4]).is_err());
        assert!(PatchSet::parse(j("ROM/1.DAT", 0).as_bytes(), vec![0; 3]).is_err());
        assert!(safe_version("2026.11.12") && !safe_version("../x") && !safe_version(""));
    }
}
