# TimelessXI client

Releases and the mod list for the TimelessXI client: [vana-client](https://github.com/lenonk/vana-client) built for
the TimelessXI server. This repository holds no client source, only what players' clients download.

- **Releases:** `vana-client-windows-x86_64.exe` and its `.sig`. The client checks this repository's latest release
  at start, and installs a newer one only when its signature matches TimelessXI's release key built into the client.
- **`catalogue/mods.json`:** the mod list (Settings > Mods > Download Standard Mods): vana-client's standard mods,
  pinned to the source commit the client is built from, plus TimelessXI's own mods in `mods/`.
- **`mods/`:** TimelessXI's mods. After changing one, raise its `version` in `mod.toml`, run
  `tools/make_catalogue.py <client source folder>`, and commit `mods/` and `catalogue/` together.

vana-client is MIT licensed (Copyright (c) 2026 Lenon Kitchens); see `LICENSE-vana-client`.
