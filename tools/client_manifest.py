#!/usr/bin/env python3
"""Edits catalogue/client.json, the Timeless XI client manifest every client reads at start
(source/crates/vana-client/src/client/timeless/manifest.rs).

  latest     the newest client release (its tag). Update Now installs it.
  required   the oldest client allowed to play. Older clients are held at the title, behind the Update Required card,
             until they update. Raise it only once the `latest` release is published, signed, with its .sig.
  server     the live server's host and ports. Clients pointed at the Timeless XI server follow it, so the server can
             move without a client release (keep the old host in "previous" so clients find their way).
  message    an optional line on the Update Required card.

Examples:
    python3 tools/client_manifest.py --latest 1.0.14-timeless.4                 # offer it, nobody forced
    python3 tools/client_manifest.py --latest 1.0.14-timeless.4 --required 1.0.14-timeless.4   # everyone must update
    python3 tools/client_manifest.py --host play.timelessxi.gg --keep-previous 74.208.79.201
    python3 tools/client_manifest.py --check                                     # the release assets are really there

Then commit catalogue/client.json and push to main. Clients read it at their next start (GitHub's CDN can take a few
minutes to serve the new copy).
"""

import argparse
import json
import pathlib
import sys
import urllib.request

REPO = "Timeless-XI/timeless-client"
ASSET = "vana-client-windows-x86_64.exe"
SETUP = "TimelessXI-Setup.exe"
path = pathlib.Path(__file__).resolve().parent.parent / "catalogue" / "client.json"


def tag(v: str) -> str:
    return v if v.startswith("v") else "v" + v


def exists(url: str) -> bool:
    try:
        req = urllib.request.Request(url, method="HEAD")
        with urllib.request.urlopen(req, timeout=20) as r:
            return r.status == 200
    except Exception:
        return False


def main() -> None:
    a = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    a.add_argument("--latest")
    a.add_argument("--required")
    a.add_argument("--message")
    a.add_argument("--host")
    a.add_argument("--keep-previous", action="append", default=[])
    a.add_argument("--game-version")
    a.add_argument("--check", action="store_true", help="check that the latest release's assets can be downloaded")
    args = a.parse_args()
    m = json.loads(path.read_text(encoding="utf-8"))
    if args.latest:
        m["latest"] = args.latest.lstrip("v")
    if args.required is not None:
        m["required"] = args.required.lstrip("v")
    if args.message is not None:
        m["message"] = args.message
    if args.game_version:
        m["game_version"] = args.game_version
    if args.host:
        s = m.setdefault("server", {})
        if s.get("host") and s["host"] != args.host:
            s.setdefault("previous", [])
            if s["host"] not in s["previous"]:
                s["previous"].append(s["host"])
        s["host"] = args.host
    for p in args.keep_previous:
        prev = m.setdefault("server", {}).setdefault("previous", [])
        if p not in prev:
            prev.append(p)
    if args.check or args.required:
        base = f"https://github.com/{REPO}/releases/download/{tag(m['latest'])}"
        missing = [n for n in (ASSET, ASSET + ".sig", SETUP) if not exists(f"{base}/{n}")]
        if missing:
            sys.exit(f"release {tag(m['latest'])} lacks {', '.join(missing)}: publish it before requiring it")
        print(f"release {tag(m['latest'])}: assets present")
    path.write_text(json.dumps(m, indent=2) + "\n", encoding="utf-8")
    print(path.read_text(encoding="utf-8"))


if __name__ == "__main__":
    main()
