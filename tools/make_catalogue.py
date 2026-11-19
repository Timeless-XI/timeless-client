#!/usr/bin/env python3
"""Writes catalogue/mods.json (and catalogue/dats.json): the mod list the TimelessXI client reads (Settings > Mods, Download Standard Mods).

It holds:
  - vana-client's standard mods, from the client source the build uses (its catalogue/mods.json). Their files are
    copied into standard/ here (checked against the hashes upstream lists) and served from this repository, so
    nothing a player downloads comes from upstream's GitHub;
  - TimelessXI's own mods, every folder in mods/ with a mod.toml, served from this repository's main branch.
    A mod here with the same id as a standard mod replaces it. They are "required": the TimelessXI client installs
    and updates them at every start, and switches them on the first time. A mod id listed in catalogue/optional.txt
    (one per line) is only offered in Settings > Mods instead.

Run it after changing anything in mods/ (raise the mod's version in its mod.toml, so clients see the update), or
after building the client from a newer upstream, then commit mods/, standard/ and catalogue/ together and push:

    py -3 tools/make_catalogue.py ..\\src        (Windows; the client source build-timeless.bat checked out)
    python3 tools/make_catalogue.py ../src      (Linux / Git Bash)

Files are hashed exactly as they are on disk; .gitattributes keeps git from changing line endings, so GitHub serves
the same bytes.
"""

import hashlib
import json
import pathlib
import re
import shutil
import sys
import tomllib

REPO = "Timeless-XI/timeless-client"
BRANCH = "main"
# Where players download the mods' files from: change this one line to move them (e.g. to the website).
BASE = f"https://raw.githubusercontent.com/{REPO}/{BRANCH}/"
UPSTREAM = "lenonk/vana-client"

here = pathlib.Path(__file__).resolve().parent.parent


def sha256(p: pathlib.Path) -> str:
    return hashlib.sha256(p.read_bytes()).hexdigest()


def upstream_entries(src: pathlib.Path) -> list:
    """The standard mods, their files copied from `src` into standard/<id>/ and listed from BASE."""
    listed = json.loads((src / "catalogue" / "mods.json").read_text(encoding="utf-8"))
    master = f"https://raw.githubusercontent.com/{UPSTREAM}/master/"
    std = here / "standard"
    if std.exists():
        shutil.rmtree(std)
    for e in listed["mods"]:
        if not re.fullmatch(r"[A-Za-z0-9_-]+", e["id"]):
            sys.exit(f"standard mod id {e['id']!r} is not a folder name")
        for f in e["files"]:
            if not f["url"].startswith(master):
                sys.exit(f"{e['id']}: {f['url']} is not one of upstream's files")
            source = src / f["url"][len(master):]
            if sha256(source) != f["sha256"]:
                sys.exit(f"{source} is not the file upstream's list names (run against the source the client builds from)")
            rel = pathlib.PurePosixPath(f["path"])
            if rel.is_absolute() or ".." in rel.parts:
                sys.exit(f"{e['id']}: bad path {f['path']!r}")
            dest = std / e["id"] / rel
            dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(source, dest)
            f["url"] = f"{BASE}standard/{e['id']}/{rel.as_posix()}"
    return listed["mods"]


def own_entry(d: pathlib.Path, optional: set) -> dict:
    m = tomllib.loads((d / "mod.toml").read_text(encoding="utf-8"))
    files = []
    for p in sorted(d.rglob("*")):
        rel = p.relative_to(d).as_posix()
        if p.is_dir() or any(part.startswith(".") for part in p.relative_to(d).parts) or rel.split("/")[0] == "tests":
            continue
        files.append({
            "path": rel,
            "url": f"{BASE}mods/{d.name}/{rel}",
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
        "required": d.name not in optional,
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
    opt_file = here / "catalogue" / "optional.txt"
    optional = {l.strip() for l in opt_file.read_text(encoding="utf-8").splitlines() if l.strip()} if opt_file.is_file() else set()
    own = [own_entry(d, optional) for d in sorted((here / "mods").iterdir()) if (d / "mod.toml").is_file()]
    ids = {e["id"] for e in own}
    mods = [e for e in upstream_entries(src) if e["id"] not in ids] + own
    mods.sort(key=lambda e: e["id"])
    out = here / "catalogue" / "mods.json"
    out.write_text(json.dumps({"format": 1, "mods": mods}, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
    req = sorted(e["id"] for e in own if e["required"])
    # The DAT packs' list (optional texture packs) is upstream's as it is: the packs are large downloads hosted by
    # upstream; keeping the list here means an entry can be dropped or replaced without a client release.
    shutil.copyfile(src / "catalogue" / "dats.json", here / "catalogue" / "dats.json")
    print(f"{out}: {len(mods)} mods ({len(own)} TimelessXI: {', '.join(sorted(ids))}; required: {', '.join(req) or 'none'})")


if __name__ == "__main__":
    main()
