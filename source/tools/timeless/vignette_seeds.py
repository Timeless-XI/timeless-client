#!/usr/bin/env python3
"""Timeless XI: places players can reach in each zone, for `vana-client vignette-scout --seeds` (the Select screen's
scenes must be where a player can actually stand, not on a backdrop mountain): every mob spawn and NPC position in
the server's data/zones/*/mobs.yaml and npcs.yaml ([1, 1, 1] placeholders left out).

    python3 tools/timeless/vignette_seeds.py <timeless-server> seeds.json
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
    seeds = {}
    zones = os.path.join(server, "data/zones")
    for name in sorted(os.listdir(zones)):
        if name not in ids:
            continue
        pts = []
        for f in ("mobs.yaml", "npcs.yaml"):
            path = os.path.join(zones, name, f)
            if not os.path.isfile(path):
                continue
            for m in re.finditer(r"\bat:\s+\[([-\d.]+),\s*([-\d.]+),\s*([-\d.]+)", open(path, encoding="utf-8").read()):
                p = [round(float(m.group(i)), 2) for i in (1, 2, 3)]
                if p not in ([1.0, 1.0, 1.0], [0.0, 0.0, 0.0]):
                    pts.append(p)
        if pts:
            seeds[str(ids[name])] = pts
    json.dump(seeds, open(out, "w"))
    print(f"{len(seeds)} zones, {sum(len(v) for v in seeds.values())} places -> {out}")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
