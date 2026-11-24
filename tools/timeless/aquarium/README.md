# Timeless Aquarium and the Elder fish: generators

Builds `source/crates/vana-client/assets/timeless/aquarium/aquarium.bin`, which the client reads in
`client/timeless/aquarium.rs`. Python 3 with numpy, scipy and Pillow. Every model is built in code. No DATs are
read, and no outside art is used.

| File | What it is |
| --- | --- |
| `r3d.py` | Mesh builders (loft, fin sheet, tube, sphere, prism) and a small numpy software renderer for mock-ups. `LOD` scales mesh resolution; `SEAM` gives lofts a seam column for baking. |
| `fishlib.py` | Fish parts: body, fins, eyes, whiskers, part ids. |
| `fish_<name>.py` | The eight Elder fish: `build()` (mesh) and `material()` (procedural textures). |
| `tank2.py` | The Timeless Aquarium (v2): the frame, night sky, void, hourglass and crown, the fish places, the animations (`anim_matrix`), and mock-up renders (`python3 tank2.py full\|empty\|three\|close`). |
| `export_aquarium2.py` | Bakes every part's texture in uv space, bakes the lighting into vertex colours, converts to furniture space (-y up, origin = the middle of the 3 x 2 footprint on the floor, scale `K`), and writes the bundle (version 2; the format is described at the top of the file and in `aquarium.rs`). |
| `export_aquarium.py` | Helpers the v2 exporter uses: bake, PNG, model packing, item icon record. Its own `bundle()` is the older v1 format, no longer read. |
| `make_aq_icon2.py` | The item icon: 32 x 32 (written into the bundle as an item-DAT icon record) and 128 px. |
| `render_all.py` | Per-fish mock-up renders and icons (the Elder fish canvas). |
| `gif.py` | An animated preview of the full tank. |

Rebuild:

```
python3 make_aq_icon2.py           # writes out/aquarium_icon32.png (set OUT at the top of the scripts)
python3 export_aquarium2.py <path to assets/timeless/aquarium/aquarium.bin>
cargo test -p vana-client aquarium
```

Groups, and how they move:

- `tank`: still.
- `crown`: spins about its vertical axis every 14 s.
- `hgcrystal`: spins every 9 s.
- `grains`: fall one spacing every 0.5 s.
- `swirl`: turns every 150 s.
- `motes`: turn the other way, every 70 s.
- `fish0`..`fish7`: each drifts in a small figure-eight, with its own period and phase.

Variants:

- Lit (1+ fish) or dark (none).
- The crown is off with no fish, dim with 1 to 7, and full with all 8.

Fish bit order: 0 Ser Barbel the Unbowed, 1 The Ore-Eater, 2 Whiskerwise, 3 The Crimson Admiral, 4 Moonfin, 5 Rimecoil,
6 Herald of Tavnazia, 7 Siren's Lantern. This is the same order as timeless-server's `modules/timeless_aquarium`.
