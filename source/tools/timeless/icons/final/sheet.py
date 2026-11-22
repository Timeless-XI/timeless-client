import sys, os, subprocess
from PIL import Image, ImageDraw
names = sys.argv[2:]
out = sys.argv[1]
S = 160
cols = 8
rows = (len(names) + cols - 1) // cols
c = Image.new("RGBA", (cols * S, rows * (S + 14)), (20, 20, 24, 255))
d = ImageDraw.Draw(c)
for i, n in enumerate(names):
    p = f"/tmp/sh_{n}.png"
    subprocess.run(["rsvg-convert", "-w", str(S - 8), f"svg/{n}.svg", "-o", p], check=True)
    x, y = (i % cols) * S, (i // cols) * (S + 14)
    c.paste(Image.open(p), (x + 4, y + 4))
    d.text((x + 6, y + S - 4), n[:24], fill=(200, 200, 210))
c.save(out)
