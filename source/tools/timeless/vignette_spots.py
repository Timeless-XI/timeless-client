#!/usr/bin/env python3
"""Timeless XI: the Select screen vignettes' fallback spots (client/timeless/vignette.rs).

A character with no record of where they left off is shown near their zone's first Home Point: open street-level
ground in every town and most outdoor zones. Reads the server's data/enums/zone.yaml and data/zones/*/npcs.yaml and
writes assets/timeless/vignette_spots.json ({zone id: [x, y, z]}).

    python3 tools/timeless/vignette_spots.py <timeless-server> assets/timeless/vignette_spots.json
"""
import json
import os
import re
import sys


def main(server, out):
    ids = {}
    for line in open(os.path.join(server, "data/enums/zone.yaml"), encoding="utf-8"):
        m = re.match(r"^\s+([a-z0-9_]+):\s+(\d+)\s*$", line)
        if m:
            ids[m.group(1)] = int(m.group(2))
    spots = {}
    zones = os.path.join(server, "data/zones")
    for name in sorted(os.listdir(zones)):
        path = os.path.join(zones, name, "npcs.yaml")
        if name not in ids or not os.path.isfile(path):
            continue
        text = open(path, encoding="utf-8").read()
        # Each entry: "  <id>:" then indented keys; the first Home Point with a place.
        for block in re.split(r"\n  (?=\d+:)", text):
            if "display_name: 'Home Point" not in block:
                continue
            m = re.search(r"\n\s+at:\s+\[([-\d.]+),\s*([-\d.]+),\s*([-\d.]+)", block)
            if m:
                spots[str(ids[name])] = [round(float(m.group(i)), 3) for i in (1, 2, 3)]
                break
    with open(out, "w", encoding="utf-8") as f:
        json.dump(dict(sorted(spots.items(), key=lambda kv: int(kv[0]))), f, indent=1)
        f.write("\n")
    print(f"{len(spots)} zones with a Home Point -> {out}")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
