//! Timeless XI: the client manifest, `catalogue/client.json` in Timeless-XI/timeless-client (main): the newest
//! client, the oldest one still allowed to play, and the live server's address and ports. Read at every start.
//!
//! - **Required update.** A client older than `required` is stopped at the title: the login and the character menu
//!   give way to the Update Required card (client/modern/tlgate.rs) until it has updated itself to `latest` (signed
//!   release, client/updater.rs) and restarted. The last manifest read is kept beside config.json, so a client that
//!   can't reach GitHub still holds to the newest `required` it has seen; one that has never read it lets the player
//!   in (the login needs the network all the same).
//! - **Server information.** The Timeless XI server's host, ports and client version come from here, so moving the
//!   server needs no client release: a config pointed at the Timeless XI server (its host now or one it had, or no
//!   server at all) follows the manifest. A config pointed anywhere else (a test server, 127.0.0.1) is left alone.
//! - Never the retail game files: those are Square Enix's, and `game_version` only holds their updates back.
//!
//! `VANA_TIMELESS_MANIFEST`: another manifest (a URL, or a file path) for test runs.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Where the manifest is published (the repository's main branch, as the mod catalogue is).
pub const URL: &str = "https://raw.githubusercontent.com/Timeless-XI/timeless-client/main/catalogue/client.json";
/// The Timeless XI server a fresh install plays on (the manifest can move it).
pub const DEFAULT_HOST: &str = "play.timelessxi.gg";
/// The format this client reads; a newer one's unknown fields are ignored.
pub const FORMAT: u32 = 1;
/// How long the title waits for the manifest before letting the player in.
pub const CHECK_SECS: f32 = 10.0;

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Manifest {
    pub format: u32,
    /// The newest client release (its tag, with or without the leading v).
    pub latest: String,
    /// The oldest client allowed to play; empty: any.
    pub required: String,
    /// Shown on the Update Required card under its title (empty: the stock line).
    pub message: String,
    pub server: Option<Server>,
    /// The newest retail game version the server takes (the PlayOnline patch server's, e.g. "30261102_0"): game file
    /// updates past it are held back. Empty: no limit.
    pub game_version: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Server {
    pub name: String,
    pub host: String,
    /// Hosts the server had before: a config still pointed at one moves to `host`.
    pub previous: Vec<String>,
    pub auth_port: u16,
    pub data_port: u16,
    pub view_port: u16,
    pub profile_port: u16,
    /// The version string the server's login takes (LSB's CLIENT_VER); the config's is only ever raised to it.
    pub client_version: String,
}

impl Default for Server {
    fn default() -> Self {
        Self {
            name: "Timeless XI".into(),
            host: DEFAULT_HOST.into(),
            previous: Vec::new(),
            auth_port: 54231,
            data_port: 54230,
            view_port: 54001,
            profile_port: 51220,
            client_version: String::new(),
        }
    }
}

/// This client's version, as its release is tagged.
pub fn current() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

impl Manifest {
    /// This client is older than `required`.
    pub fn requires_update(&self, current: &str) -> bool {
        !self.required.trim().is_empty() && super::super::updater::newer(&self.required, current)
    }

    /// The release to update to: `latest`, or `required` when `latest` is older or missing.
    pub fn target(&self) -> String {
        let (l, r) = (self.latest.trim(), self.required.trim());
        if l.is_empty() || (!r.is_empty() && super::super::updater::newer(r, l)) { r.into() } else { l.into() }
    }
}

/// Whether `server` (a config's) is the Timeless XI server: its host, one it had, or the default.
pub fn is_timeless(server: &str, m: Option<&Server>) -> bool {
    let s = server.trim();
    let same = |h: &str| !h.trim().is_empty() && h.trim().eq_ignore_ascii_case(s);
    s.is_empty() || same(DEFAULT_HOST) || m.is_some_and(|m| same(&m.host) || m.previous.iter().any(|p| same(p)))
}

/// Points a config at the manifest's server when the config is the Timeless XI server's; whether anything changed.
pub fn apply_server(cfg: &mut crate::config::Config, m: &Server) -> bool {
    if m.host.trim().is_empty() || !is_timeless(&cfg.server, Some(m)) {
        return false;
    }
    let before = (cfg.server.clone(), cfg.auth_port, cfg.data_port, cfg.view_port, cfg.profile_port);
    cfg.server = m.host.trim().to_string();
    let port = |p: u16, old: u16| if p == 0 { old } else { p };
    cfg.auth_port = port(m.auth_port, cfg.auth_port);
    cfg.data_port = port(m.data_port, cfg.data_port);
    cfg.view_port = port(m.view_port, cfg.view_port);
    cfg.profile_port = port(m.profile_port, cfg.profile_port);
    let mut changed = before != (cfg.server.clone(), cfg.auth_port, cfg.data_port, cfg.view_port, cfg.profile_port);
    // Raised only: a newer version from patched game files stays.
    let (want, have) = (crate::patch::version_key(&m.client_version), crate::patch::version_key(&cfg.client_version));
    if want.is_some() && (have.is_none() || want > have) {
        cfg.client_version = m.client_version.clone();
        changed = true;
    }
    if changed {
        log::info!(
            "timeless: server {} ({}/{}/{}/{}, {})",
            cfg.server,
            cfg.auth_port,
            cfg.data_port,
            cfg.view_port,
            cfg.profile_port,
            cfg.client_version
        );
    }
    changed
}

/// The copy of the last manifest read, beside config.json.
fn cache_path() -> PathBuf {
    crate::config::config_path().with_file_name("timeless-client.json")
}

/// The last manifest read on this computer.
pub fn cached() -> Option<Manifest> {
    serde_json::from_str(&std::fs::read_to_string(cache_path()).ok()?).ok()
}

fn keep(m: &Manifest) {
    if crate::config::is_copy() {
        return;
    }
    let path = cache_path();
    let r = path
        .parent()
        .map_or(Ok(()), std::fs::create_dir_all)
        .and_then(|()| std::fs::write(&path, serde_json::to_string_pretty(m).unwrap_or_default()));
    if let Err(e) = r {
        log::warn!("timeless: keeping the manifest in {}: {e}", path.display());
    }
}

/// Reads the manifest (VANA_TIMELESS_MANIFEST, else [`URL`]).
pub fn fetch() -> Result<Manifest> {
    let src = std::env::var("VANA_TIMELESS_MANIFEST").unwrap_or_else(|_| URL.to_string());
    let text = if src.starts_with("http://") || src.starts_with("https://") {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .user_agent(concat!("vana-client/", env!("CARGO_PKG_VERSION")))
            .timeout_global(Some(Duration::from_secs_f32(CHECK_SECS - 1.0)))
            .build()
            .into();
        // A query the CDN hasn't seen, so a just-published manifest isn't served from its cache for minutes.
        let bust = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs() / 60);
        let url = format!("{src}{}t={bust}", if src.contains('?') { '&' } else { '?' });
        agent.get(&url).call().with_context(|| src.clone())?.body_mut().read_to_string()?
    } else {
        std::fs::read_to_string(&src).with_context(|| src.clone())?
    };
    let m: Manifest = serde_json::from_str(&text).with_context(|| format!("{src}: not a client manifest"))?;
    if m.format > FORMAT {
        log::info!("timeless: manifest format {} (this client reads {FORMAT}); reading what it knows", m.format);
    }
    Ok(m)
}

#[derive(Clone, Debug, PartialEq)]
pub enum Verdict {
    Checking,
    /// Play on. `Some`: the manifest (fresh or kept).
    Clear(Option<Manifest>),
    /// This client is too old: the manifest that says so.
    Required(Manifest),
}

/// What a manifest read (or its failure, with the kept copy) means for `current`.
pub fn judge(read: Result<Manifest, String>, kept: Option<Manifest>, current: &str) -> Verdict {
    match read {
        Ok(m) if m.requires_update(current) => Verdict::Required(m),
        Ok(m) => Verdict::Clear(Some(m)),
        Err(e) => {
            log::warn!("timeless: client manifest: {e}");
            match kept {
                Some(k) if k.requires_update(current) => Verdict::Required(k),
                k => Verdict::Clear(k),
            }
        }
    }
}

pub type Shared = Arc<Mutex<Verdict>>;

/// Reads the manifest in the background.
pub fn start() -> Shared {
    let s: Shared = Arc::new(Mutex::new(Verdict::Checking));
    let out = s.clone();
    std::thread::spawn(move || {
        let read = fetch().map_err(|e| format!("{e:#}"));
        if let Ok(m) = &read {
            keep(m);
        }
        let v = judge(read, cached(), current());
        match &v {
            Verdict::Required(m) => {
                log::info!("timeless: client {} is older than {}: update required", current(), m.required)
            }
            Verdict::Clear(Some(m)) => {
                log::info!("timeless: client {} ok (latest {}, required {})", current(), m.latest, m.required)
            }
            _ => log::info!("timeless: client {} (no manifest)", current()),
        }
        *out.lock().unwrap() = v;
    });
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(latest: &str, required: &str) -> Manifest {
        Manifest { format: 1, latest: latest.into(), required: required.into(), ..Default::default() }
    }

    #[test]
    fn required_versions() {
        assert!(m("1.0.15-timeless.1", "1.0.15-timeless.1").requires_update("1.0.14-timeless.3"));
        assert!(!m("1.0.15-timeless.1", "1.0.14-timeless.3").requires_update("1.0.14-timeless.3"));
        assert!(!m("1.0.15-timeless.1", "").requires_update("1.0.14-timeless.3"));
        assert!(m("v1.0.14-timeless.4", "v1.0.14-timeless.4").requires_update("1.0.14-timeless.3"));
        // A newer client than the manifest names (a test build) plays.
        assert!(!m("1.0.14-timeless.4", "1.0.14-timeless.4").requires_update("1.0.14-timeless.5"));
        assert_eq!(m("", "1.0.15-timeless.1").target(), "1.0.15-timeless.1");
        assert_eq!(m("1.0.15-timeless.2", "1.0.15-timeless.1").target(), "1.0.15-timeless.2");
    }

    #[test]
    fn a_failed_read_holds_to_the_kept_requirement() {
        let kept = m("1.0.15-timeless.1", "1.0.15-timeless.1");
        assert!(matches!(judge(Err("offline".into()), Some(kept.clone()), "1.0.14-timeless.3"), Verdict::Required(_)));
        assert_eq!(judge(Err("offline".into()), None, "1.0.14-timeless.3"), Verdict::Clear(None));
        assert!(matches!(judge(Ok(kept), None, "1.0.15-timeless.1"), Verdict::Clear(Some(_))));
    }

    #[test]
    fn server_follows_only_the_timeless_server() {
        let s = Server {
            host: "play2.timelessxi.gg".into(),
            previous: vec!["74.208.79.201".into()],
            auth_port: 54331,
            client_version: "30261102_0".into(),
            ..Default::default()
        };
        let mut cfg = crate::config::Config { server: "74.208.79.201".into(), ..Default::default() };
        assert!(apply_server(&mut cfg, &s));
        assert_eq!((cfg.server.as_str(), cfg.auth_port, cfg.data_port), ("play2.timelessxi.gg", 54331, 54230));
        assert_eq!(cfg.client_version, "30261102_0");
        assert!(!apply_server(&mut cfg, &s), "applying twice changes nothing");
        let mut local = crate::config::Config { server: "127.0.0.1".into(), ..Default::default() };
        assert!(!apply_server(&mut local, &s));
        assert_eq!(local.server, "127.0.0.1");
        // A newer client version from patched game files is kept.
        let mut newer = crate::config::Config {
            server: DEFAULT_HOST.into(),
            client_version: "30270101_0".into(),
            ..Default::default()
        };
        apply_server(&mut newer, &s);
        assert_eq!(newer.client_version, "30270101_0");
    }

    #[test]
    fn the_published_manifest_reads() {
        // The repository's catalogue/ (the client's source is in source/ beside it; a bare copy of the source skips).
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../catalogue/client.json");
        let Ok(text) = std::fs::read_to_string(&path) else { return };
        let m: Manifest = serde_json::from_str(&text).unwrap();
        assert_eq!(m.format, FORMAT);
        assert!(m.server.is_some_and(|s| !s.host.is_empty()));
        assert!(!m.latest.is_empty());
    }
}
