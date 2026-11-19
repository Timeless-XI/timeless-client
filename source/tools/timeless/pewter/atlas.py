import sys, json, subprocess, os, re
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import pewter, controls
from PIL import Image
OUT = sys.argv[1] if len(sys.argv) > 1 else "out"
for d in ("svg/icons", "svg/controls", "png", "asset"):
    os.makedirs(os.path.join(OUT, d), exist_ok=True)
items = []
for n in pewter.ALL:
    s = pewter.svg(n, 64, design=30)
    open(f"{OUT}/svg/icons/{n}.svg", "w").write(s); items.append((f"icons/{n}", f"{OUT}/svg/icons/{n}.svg", (64, 64)))
for k, s in controls.OUT.items():
    nm = k.split("/")[1]; open(f"{OUT}/svg/controls/{nm}.svg", "w").write(s)
    m = re.search(r'width="(\d+)" height="(\d+)"', s); w, h = int(m[1]), int(m[2])
    items.append((k, f"{OUT}/svg/controls/{nm}.svg", (w // 2, h // 2) if "viewBox=\"0 0 64 64\"" not in s else (24, 24)))
imgs = []
for k, path, logical in items:
    png = f"{OUT}/png/{k.replace('/', '__')}.png"
    subprocess.run(["rsvg-convert", path, "-o", png], check=True)
    imgs.append((k, Image.open(png).convert("RGBA"), logical))
# shelf pack, 1px gutter + edge extrusion avoided by 2px transparent padding
W = 1024; x = y = 0; shelf = 0; rects = {}
order = sorted(imgs, key=lambda t: -t[1].size[1])
for k, im, lg in order:
    w, h = im.size
    if x + w + 2 > W: x = 0; y += shelf + 2; shelf = 0
    rects[k] = (x + 1, y + 1, w, h, lg); x += w + 2; shelf = max(shelf, h)
H = y + shelf + 2
atlas = Image.new("RGBA", (W, H), (0, 0, 0, 0))
for k, im, lg in imgs:
    r = rects[k]; atlas.paste(im, (r[0], r[1]))
atlas.save(f"{OUT}/asset/pewter.png", optimize=True)
json.dump({"format": "timeless-pewter-v1", "note": "Timeless View, Pewter Hairline: menu / misc icons and controls (tools/timeless/pewter). rect = x, y, w, h in the atlas; size = logical px.",
           "items": {k: {"rect": list(r[:4]), "size": list(r[4])} for k, r in sorted(rects.items())}}, open(f"{OUT}/asset/pewter.json", "w"), indent=0)
print(W, H, len(rects), os.path.getsize(f"{OUT}/asset/pewter.png"))
