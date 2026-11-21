#!/usr/bin/env python3
"""Timeless XI: scenes taken out of assets/timeless/vignette_scenes.json after looking at every one's opening shots
(vignette-shot --scene k at 2 s and 6 s, 2026-11-20): a post, rock, trunk or low wall over the legs, or the bright
void past a zone's edge in frame. Run after vignette-scout writes a new table.

    python3 tools/timeless/vignette_curate.py crates/vana-client/assets/timeless/vignette_scenes.json
"""
import json
import sys

# zone: walk scene indices (as scouted) to drop, with why.
DROP = {
    "25": [0],  # Misareaux Coast: fence posts across the legs
    "121": [0],  # Sanctuary of Zi'Tah: a rock over the legs
    "124": [1],  # Yhoator Jungle: ruin pillars in front
    "236": [0],  # Port Bastok: a bollard over the legs
    "239": [0],  # Windurst Walls: a grass bank over the legs
    "241": [1],  # Windurst Woods: a trunk in front
    "244": [0],  # Upper Jeuno: the void past the terrace
    "246": [0],  # Port Jeuno: the void past the dock
    "248": [1],  # Selbina: a post in front
}


def main(path):
    table = json.load(open(path))
    for zone, drop in DROP.items():
        if zone in table:
            walk = table[zone]["walk"]
            table[zone]["walk"] = [s for i, s in enumerate(walk) if i not in drop]
    with open(path, "w") as f:
        json.dump(table, f, indent=1)
        f.write("\n")
    print(f"dropped {sum(len(v) for v in DROP.values())} scenes")


if __name__ == "__main__":
    main(sys.argv[1])
