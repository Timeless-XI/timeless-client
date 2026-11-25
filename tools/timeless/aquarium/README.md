# Timeless Aquarium and the Elder fish: generators

The current aquarium is **v3** (`tank3.py`, `swim.py`, `export_aquarium3.py`): the 6 x 4 show piece with swimming fish. The v2 files (`tank2.py`, `export_aquarium2.py`) stay as the base it builds on (shared geometry helpers and part ids).

Builds `source/crates/vana-client/assets/timeless/aquarium/aquarium.bin`, which the client reads in
`client/timeless/aquarium.rs`. Python 3 with numpy, scipy and Pillow. Every model is built in code. No DATs are
read, and no outside art is used.

| File | What it is |
| --- | --- |
| `r3d.py` | Mesh builders (loft, fin sheet, tube, sphere, prism) and a small numpy software renderer for mock-ups. `LOD` scales mesh resolution; `SEAM` gives lofts a seam column for baking. |
| `fishlib.py` | Fish parts: body, fins, eyes, whiskers, part ids. |
| `fish_<name>.py` | The eight Elder fish: `build()` (mesh) and `material()` (procedural textures). |
| `landmark.png` | The logo's Lufaise landmark (the mesa, its window arch and ruins, cut from the logo's background), placed on the back wall. |
| `tank2.py` | The Timeless Aquarium (v2): the frame, night sky with the landmark, hourglass and crown, the fish places, the animations (`anim_matrix`), and mock-up renders (`python3 tank2.py full\|empty\|three\|close`). |
| `export_aquarium2.py` | Bakes every part's texture in uv space, bakes the lighting into vertex colours, converts to furniture space (-y up, origin = the middle of the 3 x 2 footprint on the floor, scale `K`), and writes the bundle (version 2; the format is described at the top of the file and in `aquarium.rs`). |
| `export_aquarium.py` | Helpers the v2 exporter uses: bake, PNG, model packing, item icon record. Its own `bundle()` is the older v1 format, no longer read. |
| `make_aq_icon2.py` | The item icon: 32 x 32 (written into the bundle as an item-DAT icon record) and 128 px. |
| `render_all.py` | Per-fish mock-up renders and icons (the Elder fish canvas). |
| `gif.py` | An animated preview of the full tank. |

Rebuild (v3):

```
python3 swimcheck.py               # the fish paths: no fish touching another, the walls or the hourglass
python3 make_aq_icon3.py           # writes out/aquarium_icon32.png (set OUT at the top of the scripts)
python3 export_aquarium3.py <path to assets/timeless/aquarium/aquarium.bin>
cargo test -p vana-client aquarium   # also checks the client's swim maths against swim.py (Moonfin at 7.3 s)
python3 tank3.py full|empty|three|close|top [t]    # mock-up renders; gif3.py for the swimming loop
```

v3 files:

| File | What it is |
| --- | --- |
| `tank3.py` | The 6 x 4 show piece: gilt-trimmed charcoal cabinet, fluted columns, cornice with the "ELDERS OF THE DEEP" frieze and dentils, finials, sunburst crest, lamp hood, scrollwork stand on bun feet, night sky with the logo's landmark and light shafts, the hourglass at the back; the fish split into body and tail fin. |
| `swim.py` | Each fish's loop (four layers, two fish each, never touching), sampled for the client (anim kinds 4 / 5), and the same matrix maths as the client's `Anim::local`. |
| `swimcheck.py` | Simulates ten minutes of swimming and reports any fish touching another, a wall or the hourglass. |
| `export_aquarium3.py` | Bundle version 3: u16 parameter count per group (the swim samples), fish exported centred, facing +x. |
| `make_aq_icon3.py`, `gif3.py` | The item icon and the swimming preview. |


Groups, and how they move:

- `tank`: still.
- `crown`: spins about its vertical axis every 14 s.
- `hgcrystal`: spins every 9 s.
- `grains`: fall one spacing every 0.5 s.
- `fish0`..`fish7`: each drifts in a small figure-eight, with its own period and phase.

Variants:

- Lit (1+ fish) or dark (none).
- The crown is off with no fish, dim with 1 to 7, and full with all 8.

Fish bit order: 0 Ser Barbel the Unbowed, 1 The Ore-Eater, 2 Whiskerwise, 3 The Crimson Admiral, 4 Moonfin, 5 Rimecoil,
6 Herald of Tavnazia, 7 Siren's Lantern. This is the same order as timeless-server's `modules/timeless_aquarium`.
