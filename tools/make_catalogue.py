#!/usr/bin/env python3
"""Writes catalogue/mods.json: the mod list the TimelessXI client reads (Settings > Mods, Download Standard Mods).

It holds:
  - vana-client's standard mods, from the client source the build uses (its catalogue/mods.json), each file pinned to
    that source's commit, so the list never drifts from what upstream has on master;
  - TimelessXI's own mods, every folder in mods/ with a mod.toml, served from this repository's main branch.
    A mod here with the same id as a standard mod replaces it.

Run it after changing anything in mods/ (raise the mod's version in its mod.toml, so clients see the update), then
commit mods/ and catalogue/mods.json together and push:

    py -3 tools/make_catalogue.py ..\\src        (Windows; the client source build-timeless.bat checked out)
    python3 tools/make_catalogue.py ../src      (Linux / Git Bash)

Files are hashed exactly as they are on disk; .gitattributes keeps git from changing line endings, so GitHub serves
the same bytes.
"""

import hashlib
import json
import pathlib
import subprocess
import sys
import tomllib

REPO = "Timeless-XI/timeless-client"
BRANCH = "main"
UPSTREAM = "lenonk/vana-client"

here = pathlib.Path(__file__).resolve().parent.parent


def sha256(p: pathlib.Path) -> str:
    return hashlib.sha256(p.read_bytes()).hexdigest()


def upstream_entries(src: pathlib.Path) -> list:
    commit = subprocess.run(["git", "-C", str(src), "rev-parse", "HEAD"], capture_output=True, text=True, check=True).stdout.strip()
    listed = json.loads((src / "catalogue" / "mods.json").read_text(encoding="utf-8"))
    master = f"https://raw.githubusercontent.com/{UPSTREAM}/master/"
    pinned = f"https://raw.githubusercontent.com/{UPSTREAM}/{commit}/"
    for e in listed["mods"]:
        for f in e["files"]:
            if f["url"].startswith(master):
                f["url"] = pinned + f["url"][len(master):]
    return listed["mods"]


def own_entry(d: pathlib.Path) -> dict:
    m = tomllib.loads((d / "mod.toml").read_text(encoding="utf-8"))
    files = []
    for p in sorted(d.rglob("*")):
        rel = p.relative_to(d).as_posix()
        if p.is_dir() or any(part.startswith(".") for part in p.relative_to(d).parts) or rel.split("/")[0] == "tests":
            continue
        files.append({
            "path": rel,
            "url": f"https://raw.githubusercontent.com/{REPO}/{BRANCH}/mods/{d.name}/{rel}",
            "sha256": sha256(p),
            "size": p.stat().st_size,
        })
    e = {
        "id": d.name,
        "name": m["name"],
        "kind": "lua",
        "version": m["version"],
        "authors": m.get("authors", []),
        "description": m.get("description", ""),
        "files": files,
    }
    if m.get("commands"):
        e["commands"] = m["commands"]
    return e


def main() -> None:
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    src = pathlib.Path(sys.argv[1]).resolve()
    if not (src / "catalogue" / "mods.json").is_file():
        sys.exit(f"{src} is not a vana-client source folder (no catalogue/mods.json)")
    own = [own_entry(d) for d in sorted((here / "mods").iterdir()) if (d / "mod.toml").is_file()]
    ids = {e["id"] for e in own}
    mods = [e for e in upstream_entries(src) if e["id"] not in ids] + own
    mods.sort(key=lambda e: e["id"])
    out = here / "catalogue" / "mods.json"
    out.write_text(json.dumps({"format": 1, "mods": mods}, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
    print(f"{out}: {len(mods)} mods ({len(own)} TimelessXI: {', '.join(sorted(ids))})")


if __name__ == "__main__":
    main()
