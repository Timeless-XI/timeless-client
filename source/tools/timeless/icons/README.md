# Timeless View icons

The painted icons the Timeless interface shows on the hotbars, in the Actions window and in the spell / ability
lists (client: `src/client/timeless/icons.rs`).

- `final/svg/` — the masters (128 x 128 SVG, one per icon). **These are the source of truth.** The owner-approved
  pilots (Attack, Map, Inventory, Linkshell, Fishing, Excavation, Fire / Blizzard tiers) and the round-3/4 icons
  (Sword WS, Mining, Logging, Harvesting) were made by the older generators in this folder (`pilot.py`, `p2.py`..`p8.py`,
  `tiers6.py`) and copied in as SVG.
- `final/*.py` — the generators for the rest (built on `paint.py`, the painted toolkit):
  - `elements.py`: one core design per element, scaled by tier, three of them for -ga (Aero, Stone, Thunder, Water,
    Banish); `spellset.py`: every other spell family (Cure / Curaga tiers, Bio tiers, Holy, buffs, enfeebles, songs,
    ninjutsu, summons, Blue Magic, Trusts, element-coloured Enspells / Bar-spells / Spikes / Spirits / Carols /
    Threnodies); `weapons2.py`: one weapon skill icon per weapon type; `misc2.py`: the Misc Book; `jobs.py`: a job
    emblem per job (the fallback for job abilities), two-hour variants and the unique ability icons.
  - Run `python3 final/<module>.py` to rewrite its SVGs; `final/sheet.py out.png name ...` makes a contact sheet.
- Build: `python3 final/render.py <png dir>` (rsvg-convert), then
  `python3 ../icon_map.py <png dir>` — writes `assets/timeless/icons/map.tsv`, copies the used PNGs and regenerates
  `src/client/timeless/icon_files.rs`. Rebuild the client.

Mapping rules (`icon_map.py`): element nukes by tier (ancient magic = tier IV, elemental debuffs = tier I), families by
name, songs by type, Blue Magic by job, Trusts by id (896+), job abilities by name (unique / two-hour / blood pact →
avatar / BST Ready → Ready / rolls, maneuvers, Quick Draw), anything else → the job's emblem. Weapon skills by weapon type.
