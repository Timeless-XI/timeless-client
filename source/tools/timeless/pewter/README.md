# Timeless View · Pewter Hairline sprites

These are the menu and misc icons and the controls the Timeless interface draws in place of the modern pack's art. They are engraved silver line glyphs over dark enamel, with a soft steel-blue glow. The owner approved the style on the "Timeless XI — Menus & misc icons" canvas (style A) on 2026-11-19.

- `glyphs.py`: the first glyphs approved on the canvas (Chat, Magic, Abilities, Companion, Items, Attack, Check, Trade, Equipment, Map, the Linkshell linkpearl, Party, Key Items, Journal, Mog House, Config).
- `pewter.py`: one glyph for every `icons/*` id in `assets/modern/manifest.json` (131), plus `icons/companion` (Timeless View's Trust command), and `svg(name)`, the Pewter treatment:
  - silver gradient strokes over enamel fill;
  - stroke weight set for about 30 px on screen.
- `controls.py`: the controls, each at the pack's logical size and nine-slice insets:
  - buttons, inputs, tabs and item slots;
  - checkboxes, arrows, close, search, move, drag and resize;
  - scroll thumbs.
- `atlas.py [out]`: renders every SVG with `rsvg-convert` and shelf-packs them into `asset/pewter.png` + `asset/pewter.json` (rect + logical size per id). Icons are 64 px cells (1×, little minification on screen); controls are 2×.
- `svg/`: the rendered masters.

Rebuild:

```
python3 atlas.py out
cp out/asset/pewter.png out/asset/pewter.json ../../../crates/vana-client/assets/timeless/pewter/
```

In the client (`client/modern/theme.rs`):
- `Art::sprite` returns a Pewter sprite while the Timeless interface is on (after a mod's ui.skin, before the pack's).
- A sprite the pack also has keeps the pack's logical size and insets.
