# TimelessXI client

The TimelessXI client: [vana-client](https://github.com/lenonk/vana-client) built for the TimelessXI server. This
repository holds its source, its releases and the mod list players' clients download.

- **`source/`:** the client's source, vana-client with TimelessXI's changes (the Timeless interface, the Timeless
  hotbars and Actions window, Timeless Fishing ...). `source/TIMELESS.md` lists every change and where it lives; each
  is marked with a `Timeless XI:` comment for upstream merges. Build on Windows with `source\build_timeless.bat`.

- **Releases:** `vana-client-windows-x86_64.exe` and its `.sig`, plus **`TimelessXI-Setup.exe`**: the same exe under
  the setup name, the one file players download. It opens the Timeless setup wizard, which installs the client, gets
  the FINAL FANTASY XI game files (an install already on the computer, Square Enix's installer in Downloads, or Square
  Enix's free download), updates them and points the client at the Timeless XI server. The website links
  **https://github.com/Timeless-XI/timeless-client/releases/latest/download/TimelessXI-Setup.exe**, which is always
  the newest. A client installs a newer release only when its signature matches TimelessXI's release key built into
  the client.
- **`catalogue/client.json`:** the client manifest every client reads at start: `latest`, `required` (older clients
  are held at the title until they update), the live server's host and ports (clients on the Timeless XI server follow
  them, so the server can move without a client release) and `game_version`. Edit it with `tools/client_manifest.py`.

### Publishing a client release

Releases install only when signed with one of the two built-in release keys: Cynix's (`publish-timeless.sh` on his
PC) or the owner's (`C:\FFXILocal\keys\timeless-release.pem`, `source\release\sign_release.bat`). Either can
publish. Full steps: `source/TIMELESS.md`, "Releasing".
- **`catalogue/mods.json`:** the mod list (Settings > Mods > Download Standard Mods): vana-client's standard mods,
  pinned to the source commit the client is built from, plus TimelessXI's own mods in `mods/`.
- **`catalogue/game.json`:** the newest Square Enix game version players may update to (`{"version": "30261102_0"}`). The client holds back retail game updates past it. Try a new retail update on the test server first, then raise this and push.
- **`mods/`:** TimelessXI's mods. After changing one, raise its `version` in `mod.toml`, run
  `tools/make_catalogue.py source`, and commit `mods/` and `catalogue/` together.

vana-client is MIT licensed (Copyright (c) 2026 Lenon Kitchens); see `LICENSE-vana-client`.
