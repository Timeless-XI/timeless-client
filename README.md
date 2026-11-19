# TimelessXI client

The TimelessXI client: [vana-client](https://github.com/lenonk/vana-client) built for the TimelessXI server. This
repository holds its source, its releases and the mod list players' clients download.

- **`source/`:** the client's source, vana-client with TimelessXI's changes (the Timeless interface, the Timeless
  hotbars and Actions window, Timeless Fishing ...). `source/TIMELESS.md` lists every change and where it lives; each
  is marked with a `Timeless XI:` comment for upstream merges. Build on Windows with `source\build_timeless.bat`.

- **Releases:** `vana-client-windows-x86_64.exe` and its `.sig`. The client checks this repository's latest release
  at start, and installs a newer one only when its signature matches TimelessXI's release key built into the client.
- **`catalogue/mods.json`:** the mod list (Settings > Mods > Download Standard Mods): vana-client's standard mods,
  pinned to the source commit the client is built from, plus TimelessXI's own mods in `mods/`.
- **`mods/`:** TimelessXI's mods. After changing one, raise its `version` in `mod.toml`, run
  `tools/make_catalogue.py source`, and commit `mods/` and `catalogue/` together.

vana-client is MIT licensed (Copyright (c) 2026 Lenon Kitchens); see `LICENSE-vana-client`.
