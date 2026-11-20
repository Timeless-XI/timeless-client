#!/usr/bin/env python3
"""Painted pilot round 2."""
import math, os, sys, random, re
sys.path.insert(0, os.path.dirname(__file__))
from paint import *

OUT = os.path.join(os.path.dirname(__file__), "pilot2")


def droplet(x, y, r, col="#cfe2ff"):
    """Lens-like water droplet: tinted body, dark lower rim, bright upper highlight."""
    return (f'<circle cx="{x:.1f}" cy="{y:.1f}" r="{r:.2f}" fill="{col}" fill-opacity="0.45"/>'
            f'<path d="M{x-r:.2f} {y:.1f} A{r:.2f} {r:.2f} 0 0 0 {x+r:.2f} {y:.1f}" fill="none" stroke="#1a2a4a" stroke-opacity="0.55" stroke-width="{r*0.35:.2f}"/>'
            f'<circle cx="{x-r*0.35:.2f}" cy="{y-r*0.35:.2f}" r="{r*0.32:.2f}" fill="#ffffff" fill-opacity="0.95"/>'
            f'<circle cx="{x:.1f}" cy="{y:.1f}" r="{r:.2f}" fill="none" stroke="#e8f2ff" stroke-opacity="0.5" stroke-width="0.25"/>')


# =========================== FISHING ===========================
def fishing():
    defs = (lg("skyF", [(0, "#141a33"), (0.45, "#3b3456"), (0.72, "#a3627a"), (0.86, "#f0a46a")])
            + lg("waterF", [(0, "#3a3a5c"), (0.25, "#1d2440"), (1, "#070a14")])
            + lg("trout", [(0, "#3c4a2e"), (0.3, "#6a7a4c"), (0.5, "#a8b0a0"), (0.62, "#d8dcd4"), (1, "#f4f2ea")])
            + lg("band", [(0, "#d87a8a", 0), (0.5, "#e07a8a", 0.85), (1, "#d87a8a", 0)])
            + lg("rodG", [(0, "#c48a52"), (1, "#5a3418")], 0, 0, 1, 0)
            + lg("sheet", [(0, "#ffffff", 0.85), (0.5, "#bcd4ff", 0.45), (1, "#7aa0e0", 0.1)]))
    sc = ['<rect x="0" y="0" width="128" height="80" fill="url(#skyF)"/>', stars(31, 30, (8, 8, 120, 40)),
          blob(70, 74, 50, 8, "#ffc08a", 0.55, "b5"),
          '<g filter="url(#organic)"><path d="M0 70 C14 62 24 64 34 60 C44 56 52 62 62 60 C76 57 84 50 96 54 C108 58 116 56 128 60 L128 80 L0 80 Z" fill="#2c2440"/>'
          '<path d="M0 74 C10 70 20 72 30 69 L34 64 L36 69 C46 67 58 70 70 68 L74 62 L77 68 C90 67 104 70 128 68 L128 80 L0 80 Z" fill="#1c1830"/></g>',
          '<rect x="0" y="78" width="128" height="50" fill="url(#waterF)"/>',
          blob(66, 82, 34, 2.4, "#ffb67a", 0.55, "b2"), blob(62, 89, 22, 1.6, "#ff9a6a", 0.4, "b1"), blob(68, 96, 14, 1.2, "#ff9a6a", 0.28, "b1"),
          ''.join(f'<path d="M{x} {y} h{w}" stroke="#c8b8e8" stroke-opacity="{o}" stroke-width="0.5"/>' for x, y, w, o in
                  ((14, 100, 18, .25), (40, 108, 12, .2), (20, 116, 26, .15)))]
    b = []
    # rod
    rod = "M8 122 C14 96 24 60 44 20 L45.2 20.8 C25 61 18 97 13 123 Z"
    b.append(cast_shadow(rod, 3, 3, 0.4))
    b.append(form(rod, "url(#rodG)", "#2a160a", hi="#e8b880", hl=(30, 60, 2, 14, 25), rim="#ffb27a", tex="wood", off=(1, 0.5), ao=0.3, outline="#1a0c05", ow=0.45))

    def bez(t, p=((8, 122), (14, 96), (24, 60), (44, 20))):
        return tuple((1-t)**3*p[0][i] + 3*(1-t)**2*t*p[1][i] + 3*(1-t)*t*t*p[2][i] + t**3*p[3][i] for i in (0, 1))
    for t in (0.45, 0.65, 0.85):
        x, y = bez(t)
        b.append(f'<circle cx="{x+1.4:.1f}" cy="{y+0.6:.1f}" r="1.0" fill="none" stroke="#d8d0c0" stroke-opacity="0.9" stroke-width="0.45"/>')
    b.append(form("M7.5 108 C9 101 10.5 96 12 92 L16.5 93.5 C15 98 13.8 103 12.5 109.5 Z", "#b88a5a", "#4a2e16", hi="#f0d0a0", hl=(11, 98, 1.6, 4), tex="mottle", off=(1, 1), ao=0.3, ow=0.4))
    b.append(form("M12 104 m-6 0 a6 6 0 1 0 12 0 a6 6 0 1 0 -12 0 Z", "#b8893a", "#3a2408", hi="#ffe2a0", hl=(10, 101, 2.2, 1.6), rim="#ffd080", tex="hammered", off=(1.2, 1.4), ow=0.5))
    b.append('<circle cx="12" cy="104" r="2.2" fill="#2a1a08"/><circle cx="12" cy="104" r="1" fill="#e8c880"/>')
    # line (taut) to the mouth
    b.append('<path d="M44.6 20.4 C50 30 54 38 57.6 46.2" fill="none" stroke="#f4ead8" stroke-opacity="0.85" stroke-width="0.4"/>')

    # ---- splash crown under the fish (behind) ----
    sp = []
    sp.append(blob(90, 84, 22, 5, "#e8f0ff", 0.5, "b2"))
    for x0, x1, h, o in ((72, 82, 10, .7), (79, 90, 16, .8), (91, 101, 14, .75), (98, 109, 9, .65)):
        sp.append(f'<path d="M{x0} 84 C{x0+1} {84-h*0.6} {x0+3} {84-h} {(x0+x1)/2} {84-h*1.05:.1f} C{x1-3} {84-h} {x1-1} {84-h*0.6} {x1} 84 '
                  f'C{x1-2} {84-h*0.45} {x0+2} {84-h*0.45} {x0} 84 Z" fill="url(#sheet)" fill-opacity="{o}"/>')
        sp.append(f'<path d="M{x0} 84 C{x0+1} {84-h*0.6} {x0+3} {84-h} {(x0+x1)/2} {84-h*1.05:.1f}" fill="none" stroke="#ffffff" stroke-opacity="0.75" stroke-width="0.45"/>')
    b.append("".join(sp))

    # ---- trout ----
    body = ("M-25 1 C-24 -3 -20 -7 -14 -8.5 C-4 -11 8 -10 16 -6.5 C20 -4.8 23 -3 25.5 -2.2 L25.5 2.2 "
            "C23 3 20 4.6 16 6 C8 9 -4 10 -14 8.5 C-20 7.5 -24 5 -25 1 Z")
    tail = "M24.5 -2 C28 -6 31 -10 34 -12.5 C33.4 -6 31.6 -2 32 0 C31.6 2 33.4 6 34 12.5 C31 10 28 6 24.5 2 Z"
    fins = {
        "dorsal": "M-6 -10.3 C-4 -15 3 -16.5 7.5 -15 C6 -13 5.4 -11.2 6.4 -9.7 Z",
        "adipose": "M15 -6.6 C16.6 -8.8 18.6 -8.6 19.2 -5.2 Z",
        "anal": "M13 6.6 C14.5 10.4 17 11.4 19.4 10.2 C18.6 8.2 18.4 6.2 18 4.6 Z",
        "pelvic": "M0 9.6 C1 13 3 14.2 5.4 13.8 C4.2 12 3.6 10.6 3.4 9.3 Z",
    }
    spots = "".join(f'<circle cx="{x}" cy="{y}" r="{r}" fill="#1c2010" fill-opacity="0.8"/>' for x, y, r in
                    ((-10, -6.8, .5), (-6, -8.2, .45), (-2, -7, .55), (2, -8.6, .45), (5, -6.4, .5), (9, -7.8, .45), (12, -5.6, .5),
                     (15, -4.2, .45), (18, -3.4, .4), (21, -1.8, .4), (-4, -4.6, .4), (7, -3.6, .4), (12, -2.2, .38), (17, 1.4, .38), (20, 2.2, .35)))
    fish = [cast_shadow(body, 2, 3, 0.35)]
    fish.append(form(tail, "#7a8458", "#2a301a", hi="#d0d0a8", hl=(29, -4, 2, 4), rim="#ffb27a", ao=0.25, ow=0.4, outline="#14180c",
                     extra="".join(f'<path d="M25 0 L{33.6} {y}" stroke="#2a301a" stroke-opacity="0.35" stroke-width="0.3"/>' for y in (-11, -7, -3.5, 3.5, 7, 11))
                     + "".join(f'<circle cx="{x}" cy="{y}" r="0.4" fill="#1c2010"/>' for x, y in ((29, -6), (31, -9), (30, 5), (32, 8), (28.6, 1)))))
    for k, d in fins.items():
        col = ("#8a9060", "#2a301a") if k in ("dorsal", "adipose") else ("#e0a890", "#5a2a20")
        fish.append(form(d, col[0], col[1], ao=0.2, ow=0.35, outline="#1a1a0c", off=(0.6, 0.8), blur="b06",
                         extra='<path d="' + d + '" fill="none" stroke="#fff" stroke-opacity="0.12" stroke-width="0.6"/>'))
    fish.append(form(body, "url(#trout)", "#1c2418", hi="#fffaf0", hl=(-4, -7.5, 14, 1.8), rim="#ffb07a", rim_op=0.85, ao=0.35, ow=0.5,
                     outline="#121a0e", tex="mottle", off=(1.6, 2.2),
                     extra=('<path d="M-20 0.6 C-8 -1.6 6 -1.4 25 0 C6 3.4 -8 3.6 -20 0.6 Z" fill="url(#band)" filter="url(#b06)"/>' + spots
                            + '<path d="M-15.5 -6.2 C-13 -2 -13 3 -15.2 7" fill="none" stroke="#2a3020" stroke-opacity="0.7" stroke-width="0.5"/>'
                            + '<path d="M-15 -6 C-12.6 -2 -12.6 3 -14.8 6.6" fill="none" stroke="#ffd8c8" stroke-opacity="0.35" stroke-width="0.4"/>'
                            + "".join(f'<path d="M{x} -6 q1.2 1.4 0 2.8 M{x} -1 q1.2 1.4 0 2.8" fill="none" stroke="#fff" stroke-opacity="0.07" stroke-width="0.35"/>' for x in range(-10, 22, 2)))))
    fish.append(form("M-13 3 C-12 7.2 -9 10 -5.6 10.8 C-7.4 8 -9 5.4 -10 3 Z", "#e0a890", "#5a2a20", ao=0.2, ow=0.35, outline="#2a140c", off=(0.6, 0.8), blur="b06"))
    fish.append('<circle cx="-19.4" cy="-2.6" r="1.4" fill="#c8a040"/><circle cx="-19.5" cy="-2.6" r="0.85" fill="#080604"/><circle cx="-19.9" cy="-3.1" r="0.3" fill="#fff" fill-opacity="0.85"/>')
    fish.append('<path d="M-25 1 C-23.4 2.6 -21.4 3 -19.6 2.4" fill="none" stroke="#1a140c" stroke-width="0.55"/>')
    fish.append('<path d="M-24.6 1.4 L-25.6 3.2" stroke="#c8c0b0" stroke-width="0.35"/>')
    # water streaming off the fish
    for x, y, r in ((22, 6, 0.9), (26, 9, 0.7), (29, 12, 0.6), (31, 15, 0.5), (8, 10.5, 0.7), (12, 13, 0.55)):
        fish.append(droplet(x, y, r))
    b.append(f'<g transform="translate(78 64) rotate(40)">{"".join(fish)}</g>')
    # front splash: foam + rings + droplets
    b.append(f'<g filter="url(#organic)"><ellipse cx="91" cy="85" rx="15" ry="2.6" fill="#f4f8ff" fill-opacity="0.75"/></g>')
    b.append('<ellipse cx="91" cy="85.5" rx="22" ry="3.8" fill="none" stroke="#e8f0ff" stroke-opacity="0.45" stroke-width="0.6"/>')
    b.append('<ellipse cx="91" cy="86" rx="31" ry="5.4" fill="none" stroke="#e8f0ff" stroke-opacity="0.2" stroke-width="0.5"/>')
    rnd = random.Random(5)
    for _ in range(16):
        a = math.radians(rnd.uniform(-170, -10))
        L = rnd.uniform(6, 20)
        x, y = 91 + math.cos(a) * L * 1.3, 84 + math.sin(a) * L
        b.append(droplet(x, y, rnd.uniform(0.5, 1.3)))
    return icon("Fishing", "".join(sc), "".join(b), defs, accent="#f0c8a0")


# =========================== MINING ===========================
def nugget(cx, cy, r, base, lo, hi, seed, rot=0, sides=7, spec=True):
    rnd = random.Random(seed)
    pts_ = []
    for i in range(sides):
        a = 2 * math.pi * i / sides + rnd.uniform(-0.25, 0.25)
        rr = r * rnd.uniform(0.7, 1.15)
        pts_.append((cx + math.cos(a) * rr, cy + math.sin(a) * rr * 0.85))
    d = "M" + " L".join(f"{x:.1f} {y:.1f}" for x, y in pts_) + " Z"
    s = form(d, base, lo, hi=hi, hl=(cx - r * 0.3, cy - r * 0.35, r * 0.45, r * 0.3), ao=0.4, ow=0.35, outline="#0e0a06",
             tex="hammered", off=(r * 0.18, r * 0.24), blur="b06")
    if spec:
        s += f'<circle cx="{cx-r*0.35:.1f}" cy="{cy-r*0.4:.1f}" r="{max(0.35, r*0.12):.2f}" fill="#fffbe8" fill-opacity="0.9"/>'
    return s


def mining():
    defs = (rg("caveG", [(0, "#d98a48"), (0.25, "#6a3a24"), (0.6, "#26161a"), (1, "#0a0608")], cx="20%", cy="22%", r="85%")
            + lg("quartz", [(0, "#f4f0ea"), (0.5, "#d8d2cc"), (1, "#9a948e")]))
    sc = ['<rect width="128" height="128" fill="url(#caveG)"/>',
          '<g filter="url(#rock)"><path d="M0 0 L40 0 C34 14 42 30 30 46 C20 60 26 80 10 96 L0 100 Z" fill="#4a3428"/>'
          '<path d="M128 0 L92 0 C98 20 88 34 100 52 C110 66 104 86 120 100 L128 104 Z" fill="#2c2024"/>'
          '<path d="M30 0 L100 0 C92 8 76 12 64 10 C50 8 40 6 30 0 Z" fill="#3a2a26"/></g>',
          blob(20, 24, 22, 20, "#ffb060", 0.35, "b8"),
          '<path d="M22 0 L22 14" stroke="#2a1a10" stroke-width="0.8"/>',
          form("M17 15 L27 15 L28 30 L16 30 Z", "#3a2a14", "#120a04", ao=0.2, ow=0.5, outline="#0a0604"),
          '<rect x="18.5" y="17" width="7" height="11" rx="1" fill="#ffd080"/>', blob(22, 22.5, 4, 6, "#fff2c0", 0.9, "b1"),
          blob(22, 22, 12, 12, "#ffb050", 0.45, "b5"),
          form("M15 13 L29 13 L27 16 L17 16 Z", "#b8893a", "#4a2e0c", tex="hammered", ow=0.4, ao=0.2),
          form("M15 29 L29 29 L28 32 L16 32 Z", "#b8893a", "#4a2e0c", tex="hammered", ow=0.4, ao=0.2)]
    b = []
    # jagged faceted rock: top plane lit by lantern, side planes shaded
    top = "M6 86 L20 70 L40 62 L62 64 L80 72 L88 84 L70 90 L44 92 L22 92 Z"
    left = "M6 86 L22 92 L44 92 L40 128 L2 128 Z"
    front = "M44 92 L70 90 L88 84 L96 128 L40 128 Z"
    right = "M88 84 L80 72 L96 80 L110 128 L96 128 Z"
    b.append(cast_shadow("M6 86 L20 70 L40 62 L62 64 L80 72 L96 80 L110 128 L2 128 Z", 4, 3, 0.55))
    b.append(form(right, "#3e3430", "#120e0e", ao=0.4, tex="rock", ow=0.5, outline="#0a0606", off=(1, 1)))
    b.append(form(front, "#5e504a", "#1a1414", hi="#a08070", hl=(66, 100, 12, 6), tex="rock", ao=0.45, ow=0.5, outline="#0a0606"))
    b.append(form(left, "#7a665a", "#241a16", hi="#d0a078", hl=(24, 100, 10, 8), rim="#ffb070", rim_op=0.35, tex="rock", ao=0.45, ow=0.5, outline="#0a0606"))
    b.append(form(top, "#9a8274", "#3a2c26", hi="#f0c090", hl=(36, 72, 16, 6, -10), rim="#ffc080", rim_op=0.4, tex="rock", ao=0.4, ow=0.5, outline="#0a0606"))
    # fracture lines
    for d in ("M20 70 L28 80 L26 92", "M62 64 L58 76 L66 90", "M44 92 L48 108 L44 122", "M70 90 L76 104", "M22 92 L16 110"):
        b.append(f'<path d="{d}" fill="none" stroke="#120c0a" stroke-opacity="0.7" stroke-width="0.6"/>')
        b.append(f'<path d="{d}" fill="none" stroke="#e8c0a0" stroke-opacity="0.18" stroke-width="0.4" transform="translate(-0.6 -0.4)"/>')
    # ---- realistic ore: jagged quartz host with native gold, plus a hematite pocket ----
    def jag(cx, cy, rx, ry, n, seed, rough=0.28):
        rnd = random.Random(seed)
        pts_ = []
        for i in range(n):
            a = 2 * math.pi * i / n
            k = 1 + rnd.uniform(-rough, rough)
            pts_.append((cx + math.cos(a) * rx * k, cy + math.sin(a) * ry * k))
        return "M" + " L".join(f"{x:.1f} {y:.1f}" for x, y in pts_) + " Z"
    q1 = jag(46, 80, 22, 9, 26, 31)
    q2 = jag(72, 92, 10, 7, 16, 32)
    for q in (q1, q2):
        b.append(f'<path d="{q}" fill="#000" fill-opacity="0.5" filter="url(#b1)" transform="translate(0.8 1.2)"/>')
        b.append(f'<g filter="url(#organic)">' + form(q, "url(#quartz)", "#5a5450", hi="#ffffff", hl=(40, 76, 10, 2.4), ao=0.5, ow=0.35, outline="#2a2420", tex="rock", off=(1, 1.4), blur="b1") + "</g>")
    # crystal faces glinting in the quartz
    for x, y in ((34, 78), (44, 75), (56, 79), (70, 90), (40, 84)):
        b.append(f'<polygon points="{x},{y} {x+2.4},{y-1.6} {x+3.2},{y+0.6} {x+0.8},{y+1.6}" fill="#ffffff" fill-opacity="0.55"/>')
    # native gold: branching veins + nuggets + flecks
    gid = uid("gold")
    defs_extra = ""
    veins = ["M28 82 C34 80 38 82 44 79 C48 77 52 80 58 78 C62 77 64 80 68 79",
             "M44 79 C45 76 47 74 49 73", "M58 78 C59 82 62 84 63 87", "M66 90 C70 88 74 92 78 91"]
    def bezpts(p, n):
        out = []
        for i in range(n):
            t = i / (n - 1)
            out.append(tuple((1-t)**3*p[0][j] + 3*(1-t)**2*t*p[1][j] + 3*(1-t)*t*t*p[2][j] + t**3*p[3][j] for j in (0, 1)))
        return out
    curves = [((28, 82), (36, 79), (44, 81), (52, 78.5)), ((52, 78.5), (58, 77), (64, 81), (68, 79)),
              ((44, 80), (45, 77), (47, 75), (49, 73.5)), ((58, 78.5), (60, 82), (61.5, 84), (63, 87)), ((66, 90), (70, 88), (74, 92), (78, 91))]
    rnd = random.Random(77)
    k = 0
    for c in curves:
        for (x, y) in bezpts(c, 9):
            k += 1
            r = rnd.uniform(0.7, 1.8)
            b.append(nugget(x + rnd.uniform(-0.8, 0.8), y + rnd.uniform(-0.8, 0.8), r, "#d4a032", "#4a2e08", "#fff2b8", 100 + k, sides=6, spec=rnd.random() < 0.4))
    # cloudy grey fractures inside the quartz
    for d in ("M30 79 L36 84 L42 82", "M48 74 L52 80 L50 86", "M60 76 L64 84", "M68 88 L72 94"):
        b.append(f'<path d="{d}" fill="none" stroke="#7a746e" stroke-opacity="0.55" stroke-width="0.5"/>')
    b.append(blob(50, 84, 10, 2.4, "#8a847e", 0.35, "b1"))
    for cx, cy, r, sd in ((38, 81, 2.2, 1), (53, 78.5, 2.8, 3), (73, 91, 2.0, 5)):
        b.append(nugget(cx, cy, r, "#d8a83a", "#4a3008", "#fff0b0", sd))
    rnd = random.Random(44)
    for _ in range(26):
        x, y = rnd.uniform(26, 80), rnd.uniform(74, 96)
        b.append(f'<circle cx="{x:.1f}" cy="{y:.1f}" r="{rnd.uniform(0.25,0.6):.2f}" fill="#ffd66a" fill-opacity="{rnd.uniform(0.6,1):.2f}"/>')
    for x, y, r in ((52, 77, 2.6), (38, 79.5, 1.8), (73, 89.5, 1.6)):
        b.append(sparkle(x, y, r, "#fff4c8", 0.9))
    # hematite / iron pocket on the left face: rusty staining + botryoidal metallic bumps
    b.append(blob(22, 108, 12, 9, "#8a3418", 0.55, "b2"))
    b.append(blob(60, 112, 10, 7, "#7a3014", 0.45, "b2"))
    rnd = random.Random(9)
    for cx0, cy0, n in ((22, 108, 14), (60, 112, 10)):
        for _ in range(n):
            x, y, r = cx0 + rnd.uniform(-7, 7), cy0 + rnd.uniform(-4, 4), rnd.uniform(0.9, 2.0)
            b.append(f'<circle cx="{x:.1f}" cy="{y+0.6:.1f}" r="{r:.2f}" fill="#0a0c10"/>'
                     f'<circle cx="{x:.1f}" cy="{y:.1f}" r="{r:.2f}" fill="#34363a"/>'
                     f'<circle cx="{x-r*0.25:.1f}" cy="{y-r*0.3:.1f}" r="{r*0.55:.2f}" fill="#8a8c90" fill-opacity="0.7" filter="url(#b06)"/>'
                     f'<circle cx="{x-r*0.35:.1f}" cy="{y-r*0.4:.1f}" r="{r*0.18:.2f}" fill="#e8eef8"/>'
                     f'<path d="M{x-r:.1f} {y:.1f} A{r:.2f} {r:.2f} 0 0 0 {x+r:.1f} {y:.1f}" fill="none" stroke="#c87a48" stroke-opacity="0.45" stroke-width="0.4"/>')
    # pickaxe (more detail)
    pick = []
    haft = "M-2.6 2 L2.6 2 L3.2 72 L-3.2 72 Z"
    pick.append(cast_shadow(haft, 3, 3, 0.45))
    pick.append(form(haft, "#9a6438", "#3a2212", hi="#e0aa70", hl=(-1, 30, 1.4, 18), rim="#ffb070", tex="wood", off=(1.2, 0.2), ao=0.35, outline="#1e1008", ow=0.5))
    pick.append(form("M-3.1 54 L3.1 54 L3.3 72 L-3.3 72 Z", "#4a2a18", "#140804", hi="#a86a40", hl=(-1, 60, 1, 6), tex="mottle", ow=0.4, ao=0.3,
                     extra="".join(f'<path d="M-3.2 {y} L3.2 {y+1.8}" stroke="#0a0402" stroke-opacity="0.6" stroke-width="0.45"/>' for y in (55, 58, 61, 64, 67, 70))))
    head = "M-36 10 C-26 -1 -12 -6.5 0 -6.5 C10 -6.5 22 -4 30 2.5 L28 6 C20 1.5 10 0.4 0 1 C-11 0.6 -23 4.4 -33 12.5 Z"
    pick.append(cast_shadow(head, 3, 3, 0.5))
    pick.append(form(head, "#6e7078", "#18181e", hi="#e8c8a0", hl=(-10, -4, 10, 1.6), rim="#ff9a50", rim_op=0.75, tex="hammered", off=(1, 1.8), ao=0.4, outline="#0a0a0e", ow=0.55,
                     extra='<path d="M-36 10 L-30 6.6 L-28.6 9 Z" fill="#c8ccd4" fill-opacity="0.8"/><path d="M30 2.5 L25 0.6 L24.4 3 Z" fill="#c8ccd4" fill-opacity="0.7"/>'))
    pick.append(form("M-5 -9 L5 -9 L5.6 5 L-5.6 5 Z", "#5a5c64", "#141418", hi="#c8b8a0", hl=(-2, -6, 2, 1.4), tex="hammered", off=(1, 1), ow=0.5, ao=0.3))
    pick.append('<path d="M-1.6 -9 L1.6 -9 L1.2 -6 L-1.2 -6 Z" fill="#7a5030"/><circle cx="0" cy="-2" r="1.1" fill="#2a2a30" stroke="#a8a090" stroke-width="0.4"/>')
    b.append(f'<g transform="translate(74 40) rotate(-34)">{"".join(pick)}</g>')
    # impact: sparks + rock chips at the strike point
    x0, y0 = 46, 63
    rnd = random.Random(8)
    for i in range(12):
        a = math.radians(rnd.uniform(-175, -5)); L = rnd.uniform(5, 15)
        b.append(f'<path d="M{x0+math.cos(a)*3:.1f} {y0+math.sin(a)*3:.1f} L{x0+math.cos(a)*L:.1f} {y0+math.sin(a)*L:.1f}" stroke="#ffd27a" stroke-width="{rnd.uniform(0.4,0.9):.2f}" stroke-linecap="round"/>')
    b.append(blob(x0, y0, 6, 4, "#fff0b0", 0.85, "b2"))
    b.append(sparkle(x0, y0, 4, "#fff6d0"))
    for x, y, r in ((34, 54, 1.6), (56, 52, 1.2), (28, 62, 1.0), (60, 58, 0.9)):
        b.append(f'<polygon points="{x-r},{y} {x},{y-r} {x+r*1.2},{y+r*0.3} {x},{y+r}" fill="#8a7468" stroke="#1a1210" stroke-width="0.3"/>')
    return icon("Mining", "".join(sc), "".join(b), defs, accent="#f0c090")


# =========================== LOGGING ===========================
def logging():
    defs = (lg("forestG", [(0, "#0c1e14"), (0.5, "#24402a"), (0.8, "#3a4a22"), (1, "#1a1408")])
            + lg("shaft", [(0, "#fff2b8", 0.0), (0.5, "#fff2b8", 0.26), (1, "#fff2b8", 0.0)], 0, 0, 1, 0)
            + rg("ringsG", [(0, "#f2d6a0"), (0.6, "#dcae70"), (0.9, "#b8824a"), (1, "#8a5428")])
            + lg("bevel", [(0, "#f4f0ea"), (1, "#a8acb4")], 0, 0, 1, 0))
    trunks = "".join(f'<rect x="{x}" y="0" width="{w}" height="80" fill="{c}" fill-opacity="{o}" filter="url(#b1)"/>' for x, w, c, o in
                     ((6, 10, "#16241a", 1), (28, 6, "#1e3022", 0.9), (98, 12, "#14201a", 1), (118, 8, "#1a2a1e", 1), (80, 5, "#2a3a26", 0.8)))
    sc = ['<rect width="128" height="128" fill="url(#forestG)"/>', trunks,
          '<g filter="url(#b3)"><polygon points="40,0 60,0 100,80 70,80" fill="url(#shaft)"/></g>',
          motes(12, 16, (6, 4, 122, 44), ["#d8f0a0", "#f8e8a0", "#a8d080"], (0.6, 1.6))]
    b = []
    # big chopping stump, framed: top face spans the width
    stump = "M14 70 C12 92 10 108 4 128 L124 128 C118 108 116 92 114 70 Z"
    b.append(form(stump, "#6a4a30", "#1e120a", hi="#c89060", hl=(34, 96, 10, 18), rim="#f0c070", tex="wood", off=(3, 0), ao=0.5, outline="#140a04"))
    for x in (24, 38, 54, 72, 88, 104):
        b.append(f'<path d="M{x} 78 C{x-1} 92 {x+1} 108 {x-2} 128" fill="none" stroke="#120a04" stroke-opacity="0.6" stroke-width="0.9"/>')
        b.append(f'<path d="M{x+1.2} 80 C{x} 94 {x+2} 108 {x-0.6} 126" fill="none" stroke="#d8a878" stroke-opacity="0.15" stroke-width="0.5"/>')
    b.append('<g filter="url(#organic)"><path d="M14 92 C20 88 30 94 26 102 C20 106 14 102 14 92 Z M104 84 C110 82 116 88 114 94 C108 96 104 92 104 84 Z" fill="#5a7a2a" fill-opacity="0.85"/></g>')
    b.append('<ellipse cx="64" cy="70" rx="51" ry="14" fill="#2a180a"/>')
    b.append('<ellipse cx="64" cy="69" rx="50" ry="13.2" fill="url(#ringsG)"/>')
    b.append('<g filter="url(#mottle)"><ellipse cx="64" cy="69" rx="50" ry="13.2" fill="#e0b880" fill-opacity="0.35"/></g>')
    for k in range(1, 10):
        b.append(f'<ellipse cx="{62+k*0.3:.1f}" cy="69.4" rx="{50*k/10:.1f}" ry="{13.2*k/10:.1f}" fill="none" stroke="#8a5a2a" stroke-opacity="0.5" stroke-width="0.4"/>')
    b.append('<path d="M60 69 L30 76 M60 69 L22 66" stroke="#4a2810" stroke-opacity="0.65" stroke-width="0.6"/>')
    b.append(blob(48, 66, 18, 4, "#fff2c8", 0.35, "b2"))
    # axe: detailed bearded head biting into the top face, haft up to the top-right
    ax = []
    haft = "M-2.6 0 C-2.8 -20 -2 -40 -3.4 -62 L1.8 -62 C1.6 -40 2.6 -20 2.6 0 Z"
    ax.append(cast_shadow(haft, 4, 2, 0.4))
    ax.append(form(haft, "#b07a46", "#3a2010", hi="#f4cc90", hl=(-1, -30, 1.2, 18), rim="#ffd080", tex="wood", off=(1.3, 0), ao=0.3, ow=0.5, outline="#1e1008"))
    ax.append(form("M-3.6 -62 L2 -62 L2.6 -55 C0.6 -54 -2.6 -55 -3.6 -57 Z", "#5a3418", "#1a0a04", ow=0.45, ao=0.25))
    ax.append('<path d="M-3.4 -48 L1.9 -46.6 M-3.3 -45 L1.9 -43.6" stroke="#2a1408" stroke-width="0.9"/>')
    head = "M-5 -8 L5 -8 C11 -7 18 -10 25 -17 C28 -8 29 4 26 16 C19 9 12 6.4 5 6.4 L-5 6.4 L-6.6 -0.8 Z"
    cid = uid("axc")
    ax.append(f'<clipPath id="{cid}"><path d="M-40 -60 L50 -60 L50 9.5 L-40 3 Z"/></clipPath>')
    edgeband = "M20 -14 C24 -6 25 4 22.4 12.6 L26 16 C29 4 28 -8 25 -17 Z"
    ax.append(f'<g clip-path="url(#{cid})">' + cast_shadow(head, 3, 2, 0.5)
              + form(head, "#6e7078", "#18181e", hi="#e8d4b0", hl=(10, -2, 7, 2.4), rim="#ffcc80", rim_op=0.6, tex="hammered", off=(1.2, 1.6), ao=0.4, outline="#0c0c10",
                     extra='<path d="M6 -7 C11 -6 17 -9 22 -14" fill="none" stroke="#2a2c32" stroke-opacity="0.6" stroke-width="0.6"/>'
                           '<path d="M6 5.4 C12 5.6 18 8 23 12.6" fill="none" stroke="#2a2c32" stroke-opacity="0.6" stroke-width="0.6"/>'
                           + "".join(f'<circle cx="{x}" cy="{y}" r="0.45" fill="#2a2c32" fill-opacity="0.7"/>' for x, y in ((10, -2), (12, -1), (14, -2), (12, 1.2))))
              + form(edgeband, "url(#bevel)", "#5a5e66", hi="#ffffff", hl=(24, -4, 1, 6), ao=0.2, ow=0.4, outline="#1a1c22", off=(0.6, 0.6), blur="b06")
              + "</g>")
    ax.append(form("M-6.6 -10 L5.4 -10 L5.4 8 L-6.6 8 Z", "#5a5c64", "#141418", hi="#c8b8a0", hl=(-3, -7, 2, 2), ow=0.45, ao=0.3, tex="hammered"))
    ax.append('<path d="M-3.4 -10 L2.4 -10 L2 -12 L-3 -12 Z" fill="#8a5a30"/><path d="M-2 -12 L1 -12 L0.4 -13.4 L-1.4 -13.4 Z" fill="#7a8088"/>')
    ax.append('<path d="M5 3.4 L26 5.6" stroke="#140a04" stroke-width="2.2" stroke-linecap="round"/><path d="M5 4.8 L26 7" stroke="#f0d098" stroke-opacity="0.5" stroke-width="0.6"/>')
    b.append(blob(70, 74, 16, 4, "#1a0c04", 0.55, "b2", 20))
    b.append(f'<g transform="translate(58 63) rotate(40) scale(1.25)">{"".join(ax)}</g>')
    b.append('<path d="M60.5 69.8 C64 71.8 70 74.6 74.8 77.8" fill="none" stroke="#f4dcae" stroke-opacity="0.65" stroke-width="0.7"/>')
    # split crack + chips
    b.append('<path d="M74 78 L82 82 L84 86" fill="none" stroke="#2a1408" stroke-width="0.9"/>')
    rnd = random.Random(14)
    for _ in range(9):
        x, y = rnd.uniform(78, 112), rnd.uniform(40, 64)
        b.append(f'<rect x="{x:.1f}" y="{y:.1f}" width="{rnd.uniform(2.4,4.4):.1f}" height="1.4" rx="0.4" transform="rotate({rnd.uniform(0,180):.0f} {x:.1f} {y:.1f})" fill="#ecd09a" stroke="#6a4020" stroke-width="0.25"/>')
    return icon("Logging", "".join(sc), "".join(b), defs, accent="#e8d8a0")



# =========================== HARVESTING (herbs, no wheat) ===========================
def flax(x, y, r, rot=0):
    pet = []
    for i in range(5):
        a = rot + i * 72
        pet.append(f'<ellipse cx="{x}" cy="{y-r*0.62:.2f}" rx="{r*0.42:.2f}" ry="{r*0.62:.2f}" transform="rotate({a} {x} {y})" fill="#6a8ae0" stroke="#1e2a6a" stroke-width="0.25"/>'
                   f'<ellipse cx="{x}" cy="{y-r*0.7:.2f}" rx="{r*0.16:.2f}" ry="{r*0.4:.2f}" transform="rotate({a} {x} {y})" fill="#b8c8ff" fill-opacity="0.6"/>')
    return "".join(pet) + f'<circle cx="{x}" cy="{y}" r="{r*0.22:.2f}" fill="#f0d060"/>'


def harvesting():
    defs = (lg("duskH", [(0, "#3a4a7a"), (0.4, "#b8848a"), (0.7, "#f0b878"), (0.85, "#f6d8a0")])
            + lg("meadowH", [(0, "#5a6a2a"), (1, "#1e2410")])
            + lg("bladeG", [(0, "#a8d878"), (1, "#3a6a24")], 0, 0, 1, 0))
    sc = ['<rect width="128" height="86" fill="url(#duskH)"/>', blob(96, 70, 20, 20, "#fff4d0", 0.85, "b5"), blob(96, 70, 8, 8, "#ffffff", 0.95, "b2"),
          blob(34, 34, 28, 5, "#ffe0c8", 0.4, "b3"),
          '<g filter="url(#organic)"><path d="M0 74 C24 70 52 74 80 70 C100 68 114 72 128 70 L128 128 L0 128 Z" fill="url(#meadowH)"/></g>',
          '<g opacity="0.85">' + "".join(f'<path d="M{x} {78+(x*7)%9} q{(x%5)-2} -{6+(x%4)} {(x%7)-3} -{10+(x%6)}" fill="none" stroke="#4a5a22" stroke-width="0.8"/>' for x in range(0, 128, 4)) + "</g>",
          "".join(f'<circle cx="{x}" cy="{y}" r="1.1" fill="#7a9ae8" fill-opacity="0.75"/>' for x, y in ((10, 82), (24, 86), (110, 80), (118, 88), (100, 92)))]
    b = []
    # herb bundle lying diagonally: grass blades, flax stems, wild onions
    bundle = []
    rnd = random.Random(21)
    for i in range(13):
        a = rnd.uniform(-8, 8)
        L = rnd.uniform(48, 64)
        w = rnd.uniform(1.2, 2.2)
        x0 = rnd.uniform(-3, 3)
        bundle.append(f'<g transform="rotate({a:.1f})"><path d="M{x0-w:.1f} 0 C{x0-w:.1f} {-L*0.5:.1f} {x0+1:.1f} {-L*0.85:.1f} {x0+2:.1f} {-L:.1f} C{x0+w:.1f} {-L*0.6:.1f} {x0+w:.1f} {-L*0.3:.1f} {x0+w:.1f} 0 Z" fill="url(#bladeG)" stroke="#1e3a10" stroke-width="0.3"/>'
                      f'<path d="M{x0:.1f} -2 C{x0:.1f} {-L*0.5:.1f} {x0+1:.1f} {-L*0.8:.1f} {x0+1.8:.1f} {-L+2:.1f}" fill="none" stroke="#e0ffc0" stroke-opacity="0.45" stroke-width="0.35"/></g>')
    for a, L in ((-14, 50), (-4, 58), (8, 54), (16, 46)):
        ra = math.radians(a)
        x1, y1 = math.sin(ra) * L, -math.cos(ra) * L
        bundle.append(f'<path d="M0 0 Q{x1*0.3:.1f} {y1*0.6:.1f} {x1:.1f} {y1:.1f}" fill="none" stroke="#3a6a2a" stroke-width="0.7"/>')
        bundle.append(flax(x1, y1, 3.6, a))
    # wild onions
    for x, y, rot in ((-6, 6, -20), (6, 9, 15)):
        bundle.append(f'<g transform="translate({x} {y}) rotate({rot})">'
                      + form("M0 -4 C4 -4 6 0 5 4 C4 7 2 8 0 10 C-2 8 -4 7 -5 4 C-6 0 -4 -4 0 -4 Z", "#f2ece0", "#8a7a60", hi="#ffffff", hl=(-1.6, 0, 1.6, 3), ao=0.3, ow=0.35, outline="#4a3a24", off=(0.8, 1), blur="b06",
                             extra='<path d="M-2 -2 C-2.4 2 -1.6 6 0 9 M2 -2 C2.4 2 1.6 6 0 9" fill="none" stroke="#c8b8a0" stroke-width="0.3"/>')
                      + '<path d="M0 10 l-1.4 2.4 M0 10 l0 2.8 M0 10 l1.4 2.4" stroke="#c8b8a0" stroke-width="0.35"/></g>')
    # twine wrap
    bundle.append(form("M-8 -12 C-3 -10 3 -10 8 -12 L8 -6 C3 -4 -3 -4 -8 -6 Z", "#c89a5a", "#4a2e12", hi="#f8e0a8", hl=(-2, -11, 4, 1), tex="mottle", ow=0.4, ao=0.3,
                       extra="".join(f'<path d="M{x} -12 L{x+2} -5" stroke="#5a3a14" stroke-width="0.4"/>' for x in (-6, -2, 2, 6))))
    b.append(f'<g transform="translate(42 104) rotate(38)">{cast_shadow("M-8 0 L-6 -60 L8 -60 L8 0 Z", 3, 3, 0.4)}{"".join(bundle)}</g>')
    # sickle in front, handle bottom-right
    blade = "M98 104 C116 84 112 54 88 44 C76 40 62 44 56 52 C66 48 78 48 86 54 C100 64 100 84 92 100 Z"
    b.append(cast_shadow(blade, 3, 3, 0.45))
    b.append(form(blade, "#8a8c94", "#202028", hi="#ffe8c0", hl=(104, 72, 2.6, 14, -20), rim="#ffc070", rim_op=0.8, tex="hammered", off=(-1.2, 1.2), ao=0.35, outline="#0e0e14",
                  extra='<path d="M86 54 C100 64 100 84 92 100" fill="none" stroke="#fff0d0" stroke-opacity="0.8" stroke-width="0.7"/>'
                        + "".join(f'<path d="M{x} {y} l1.4 1.2" stroke="#2a2a30" stroke-width="0.45"/>' for x, y in ((90, 58), (94, 64), (96, 72), (96, 80), (94, 88), (92, 96)))))
    handle = "M91.5 102 L100 105 L106 127 L98 127 Z"
    b.append(form(handle, "#9a6438", "#3a2212", hi="#e8b070", hl=(99, 112, 1.6, 8, -15), rim="#ffc070", tex="wood", off=(1, 0), ao=0.3))
    b.append(form("M90 99 L101 102.6 L100 107 L89 103.4 Z", "#c09040", "#4a2e0c", hi="#ffe6a8", hl=(95, 101, 3, 1), tex="hammered", ow=0.4, ao=0.2))
    b.append(motes(19, 10, (14, 20, 120, 70), ["#fff4d0", "#ffe0a0"], (0.4, 1.0)))
    return icon("Harvesting", "".join(sc), "".join(b), defs, accent="#f8e0b0")


# =========================== EXCAVATION (bones at night) ===========================
def excavation():
    defs = (lg("nightE", [(0, "#070b1e"), (0.6, "#1e2244"), (1, "#3a2e44")])
            + lg("earthE", [(0, "#4e3a2c"), (1, "#140c08")])
            + lg("boneG", [(0, "#fbf4e2"), (0.6, "#e2d4b2"), (1, "#b09c74")]))
    sc = ['<rect width="128" height="128" fill="url(#nightE)"/>', stars(57, 50, (6, 6, 122, 60)),
          blob(96, 22, 7, 7, "#f4f2ff", 0.95, "b1"), blob(96, 22, 20, 20, "#c8d0ff", 0.35, "b5"),
          '<g filter="url(#rock)"><path d="M0 34 L14 30 L22 40 L30 38 L36 54 L40 72 L0 80 Z" fill="#6a4434"/>'
          '<path d="M128 40 L114 36 L106 46 L98 44 L92 60 L90 76 L128 84 Z" fill="#5a3a2e"/></g>',
          blob(20, 46, 10, 18, "#b8c8ff", 0.12, "b3"),
          '<g filter="url(#organic)"><path d="M0 74 C30 68 60 72 90 70 C108 69 118 72 128 72 L128 128 L0 128 Z" fill="url(#earthE)"/></g>',
          '<g filter="url(#mottle)"><rect x="0" y="70" width="128" height="58" fill="#3a2a20" fill-opacity="0.6"/></g>',
          '<path d="M10 96 L22 92 L30 98 M70 112 L82 108 L88 116 L100 112 M40 120 L52 116 M104 92 L114 96" fill="none" stroke="#1a0e08" stroke-opacity="0.7" stroke-width="0.8"/>',
          '<path d="M10 95.4 L22 91.4 L30 97.4 M70 111.4 L82 107.4 L88 115.4" fill="none" stroke="#c8a888" stroke-opacity="0.25" stroke-width="0.4"/>']
    b = []
    # dug hollow
    b.append('<ellipse cx="62" cy="90" rx="48" ry="16" fill="#0a0604" fill-opacity="0.85" filter="url(#b3)"/>')
    b.append('<path d="M16 84 C30 76 96 74 112 84" fill="none" stroke="#a8b0d8" stroke-opacity="0.35" stroke-width="1" filter="url(#b06)"/>')
    # giant femur, half buried
    femur = ("M-30 -3 C-33 -9 -39 -7 -38 -1.5 C-41 2 -39 8 -33.5 6.5 C-30.5 8.5 -27.5 6.5 -26.4 3.2 L20 2.6 "
             "C22 6.6 27 9.4 32 7.6 C37.6 5.4 37.6 -2.4 33.4 -4.8 C29.8 -6.8 25.4 -5.6 22.6 -2.8 L-26.4 -2.8 Z")
    fem = [cast_shadow(femur, 2, 3, 0.55),
           form(femur, "url(#boneG)", "#5a4a30", hi="#fffaea", hl=(-4, -2.4, 18, 1.2), rim="#c8d0ff", rim_op=0.55, tex="mottle", off=(1.2, 1.6), ao=0.4, outline="#2a2014", ow=0.5,
                extra='<path d="M-10 -1 L-6 0.4 M6 -1.2 L10 0" stroke="#6a5438" stroke-opacity="0.6" stroke-width="0.4"/>'
                      + blob(32, 2, 4, 3, "#8a6a3a", 0.45, "b1") + blob(-34, 2, 4, 3, "#8a6a3a", 0.45, "b1"))]
    b.append(blob(64, 82, 34, 8, "#c8d0ff", 0.18, "b3", -22))
    b.append(f'<g transform="translate(64 82) rotate(-22) scale(1.18)">{"".join(fem)}</g>')
    # fang / claw
    fang = "M0 0 C6 -10 8 -22 4 -34 C11 -22 13 -8 8.6 2.4 Z"
    b.append(f'<g transform="translate(26 96) rotate(-18)">' + cast_shadow(fang, 2, 2, 0.5)
             + form(fang, "url(#boneG)", "#5a4a30", hi="#fffaea", hl=(6, -14, 1.2, 8), rim="#c8d0ff", rim_op=0.5, tex="mottle", off=(1, 1), ao=0.35, outline="#2a2014", ow=0.45)
             + "</g>")
    # turtle-shell fragment
    shell = "M88 102 C90 94 100 90 110 92 C118 94 120 102 116 108 C108 112 94 112 88 102 Z"
    b.append(cast_shadow(shell, 2, 2, 0.5))
    b.append(form(shell, "#6a5a3a", "#1e1810", hi="#c8b080", hl=(100, 96, 8, 2.4), rim="#c8d0ff", rim_op=0.4, tex="mottle", off=(1, 1.4), ao=0.4, outline="#14100a",
                  extra='<path d="M96 96 L100 102 L108 101 L112 95 M100 102 L98 109 M108 101 L110 108" fill="none" stroke="#2a2014" stroke-width="0.7"/>'))
    # earth lip covering the buried ends
    b.append('<g filter="url(#organic)"><path d="M0 104 C20 100 30 104 40 102 C52 100 60 104 70 102 L72 128 L0 128 Z" fill="#4a3424"/>'
             '<path d="M84 110 C96 106 112 110 128 106 L128 128 L80 128 Z" fill="#3e2c20"/></g>')
    b.append(motes(66, 10, (20, 60, 110, 110), ["#e8dcc0"], (0.4, 0.9), glow=False))
    b.append(sparkle(78, 74, 3.4, "#f4f2ff"))
    b.append(sparkle(30, 66, 2.2, "#f4f2ff", 0.8))
    return icon("Excavation", "".join(sc), "".join(b), defs, accent="#d8d0f0")


# =========================== SWORD WS (charged blade planted in the ground) ===========================
from pilot import sword_painted


def ws_sword():
    defs = (lg("emberS", [(0, "#0e0608"), (0.55, "#2e1210"), (1, "#5a2410")])
            + lg("auraG", [(0, "#fff6d8", 0), (0.5, "#ffd890", 0.55), (1, "#fff6d8", 0.95)]))
    sc = ['<rect width="128" height="128" fill="url(#emberS)"/>',
          blob(30, 40, 30, 10, "#2a1014", 0.6, "b5"), blob(96, 30, 30, 9, "#2a1210", 0.5, "b5"),
          '<g filter="url(#organic)"><path d="M0 92 C30 88 90 90 128 88 L128 128 L0 128 Z" fill="#1a0c08"/></g>',
          '<g filter="url(#rock)"><rect x="0" y="88" width="128" height="40" fill="#5a3a2a" fill-opacity="0.5"/></g>']
    b = []
    # glowing ground cracks radiating from the impact
    cracks = ["M64 98 L50 102 L40 100 L28 106 L16 104", "M64 98 L78 103 L90 101 L104 108 L116 106", "M64 98 L60 110 L64 120 L58 128",
              "M64 98 L72 112 L70 124", "M64 98 L46 94 L34 95", "M64 98 L84 94 L96 95"]
    for c in cracks:
        b.append(f'<path d="{c}" fill="none" stroke="#ff9a40" stroke-opacity="0.6" stroke-width="2.6" filter="url(#b1)"/>')
        b.append(f'<path d="{c}" fill="none" stroke="#fff0b0" stroke-width="0.8"/>')
    b.append('<ellipse cx="64" cy="98" rx="40" ry="8" fill="none" stroke="#ffd890" stroke-opacity="0.55" stroke-width="1.6" filter="url(#b1)"/>')
    b.append('<ellipse cx="64" cy="98" rx="26" ry="5" fill="#ffb060" fill-opacity="0.35" filter="url(#b3)"/>')
    # aura rising around the blade
    b.append('<path d="M50 98 C46 70 56 46 64 22 C72 46 82 70 78 98 Z" fill="#ffc870" fill-opacity="0.35" filter="url(#b5)"/>')
    for x, h, o in ((56, 52, .55), (72, 48, .5), (62, 66, .6), (68, 40, .45)):
        b.append(f'<path d="M{x-2} 98 C{x-4} {98-h*0.5} {x+2} {98-h*0.8} {x} {98-h} C{x+3} {98-h*0.7} {x+3} {98-h*0.4} {x+2} 98 Z" fill="url(#auraG)" fill-opacity="{o}" filter="url(#b06)"/>')
    # sword point-down, planted
    b.append(f'<g transform="translate(64 26) rotate(180)">{cast_shadow("M-4 -80 L4 -80 L4 26 L-4 26 Z", -3, -3, 0.35)}{sword_painted(80, ("#d04040", "#4a0a0a", "#ffb0a0"))}</g>')
    b.append('<ellipse cx="64" cy="98" rx="5" ry="1.6" fill="#140804"/>')
    b.append(motes(55, 26, (20, 20, 108, 96), ["#ffd27a", "#fff0c0", "#ffb060"], (0.4, 1.1)))
    b.append(sparkle(64, 98, 5, "#fff6d8"))
    return icon("Weapon Skill - Sword", "".join(sc), "".join(b), defs, accent="#f0c090")


# =========================== INVENTORY (cleaned) ===========================
def inventory():
    defs = (rg("candleI", [(0, "#ffc070", 0.45), (1, "#ffc070", 0)], cx="22%", cy="18%", r="80%")
            + lg("planksI", [(0, "#4a2e1c"), (1, "#1e120a")])
            + lg("potionI", [(0, "#ff7a8a"), (0.5, "#c8202e"), (1, "#5a0810")], 0, 0, 1, 0))
    sc = ['<rect width="128" height="128" fill="url(#planksI)"/>']
    for y in (0, 32, 64, 96):
        sc.append(f'<g filter="url(#woodH)"><rect x="0" y="{y}" width="128" height="31" fill="#5a3a22"/></g>')
        sc.append(f'<rect x="0" y="{y+30.6}" width="128" height="1.4" fill="#120804" fill-opacity="0.85"/>')
    sc.append('<rect width="128" height="128" fill="url(#candleI)"/>')
    b = []
    bottle = "M76 40 C71 42 68 47 68 52 C68 59 73 62 80 62 C87 62 92 59 92 52 C92 47 89 42 84 40 L84 32 L76 32 Z"
    b.append(cast_shadow(bottle, 3, 3, 0.45))
    b.append(form(bottle, "url(#potionI)", "#3a0408", hi="#ffd0d8", hl=(73, 48, 2.4, 4.6, 20), rim="#ffb0a0", rim_op=0.6, off=(1, 1), ao=0.3, outline="#1e0204", ow=0.5))
    b.append(form("M75 27 L85 27 L84.6 33 L75.4 33 Z", "#b88a5a", "#4a2e16", hi="#f0d0a0", hl=(78, 29, 1.6, 1.2), tex="mottle", ow=0.45, ao=0.2))
    body = "M24 62 C24 56 28 54 34 54 L94 54 C100 54 104 56 104 62 L102 104 C102 108 98 110 94 110 L34 110 C30 110 26 108 26 104 Z"
    b.append(cast_shadow(body, 4, 5, 0.6))
    b.append(form(body, "#8a5230", "#2a1408", hi="#d89a62", hl=(42, 90, 14, 9), rim="#ffb070", rim_op=0.45, tex="mottle", off=(2.4, 3), ao=0.5, outline="#1a0c05",
                  extra='<path d="M31 66 L30 102 M97 66 L96 102 M34 105 L94 105" fill="none" stroke="#e8c090" stroke-opacity="0.4" stroke-width="0.55" stroke-dasharray="1.6 1.4"/>'))
    flap = "M22 60 C22 54 28 52 34 52 L94 52 C100 52 106 54 106 60 L103 78 C94 87 34 87 25 78 Z"
    b.append(cast_shadow(flap, 0, 3, 0.45))
    b.append(form(flap, "#9a5c36", "#2e160a", hi="#e8a870", hl=(46, 60, 18, 3.4), rim="#ffc080", rim_op=0.45, tex="mottle", off=(1.6, 2.4), ao=0.45, outline="#1a0c05",
                  extra='<path d="M27 58 C27 56 30 56 34 56 L94 56 C98 56 101 56 101 58 L99 76 C90 83 38 83 29 76 Z" fill="none" stroke="#f0c898" stroke-opacity="0.4" stroke-width="0.55" stroke-dasharray="1.6 1.4"/>'))
    b.append(form("M59.5 76 L68.5 76 L68.5 96 L59.5 96 Z", "#6a3a1e", "#1e0e06", tex="mottle", ow=0.45, ao=0.3))
    buckle = "M56 78 L72 78 L72 91 L56 91 Z M58.8 80.8 L69.2 80.8 L69.2 88.2 L58.8 88.2 Z"
    b.append(f'<g fill-rule="evenodd">{form(buckle, "#c49440", "#3e2608", hi="#ffe8a8", hl=(61, 79, 5, 1), rim="#ffd890", tex="hammered", off=(0.6, 1), ao=0.3, outline="#1e1206", ow=0.5)}</g>')
    b.append('<path d="M64 78.6 L64 90.4" stroke="#c49440" stroke-width="1.3"/>')
    # strap loops
    for x in (34, 94):
        b.append(form(f"M{x-3} 50 C{x-3} 42 {x+3} 42 {x+3} 50 L{x+1.6} 52 C{x+1.6} 46 {x-1.6} 46 {x-1.6} 52 Z", "#8a5230", "#2a1408", tex="mottle", ow=0.4, ao=0.3))
    return icon("Inventory", "".join(sc), "".join(b), defs, accent="#f0d0a0")


# =========================== LINKSHELL (the shipped Timeless pearl) ===========================
def linkshell():
    src = open("/mnt/user-data/uploads/v10_border/linkpearl_item_master.svg").read()
    src = re.sub(r"<metadata>.*?</metadata>", "", src, flags=re.S)
    inner = re.search(r"<svg[^>]*>(.*)</svg>", src, re.S).group(1)
    inner = re.sub(r'id="([^"]+)"', r'id="ls_\1"', inner)
    inner = re.sub(r'url\(#([^)]+)\)', r'url(#ls_\1)', inner)
    inner = re.sub(r'href="#([^"]+)"', r'href="#ls_\1"', inner)
    defs = lg("lsSky", [(0, "#2c3c64"), (0.45, "#5a6e9c"), (0.75, "#9a90b8"), (1, "#c0a8b8")]) + rg("lsAura", [(0, "#cfe0ff", 0.75), (0.5, "#8fb2e6", 0.35), (1, "#8fb2e6", 0)])
    sc = ['<rect width="128" height="128" fill="url(#lsSky)"/>', stars(91, 40, (6, 6, 122, 60)),
          '<g filter="url(#organic)"><path d="M0 100 C24 94 44 98 64 96 C86 94 104 98 128 94 L128 128 L0 128 Z" fill="#4a4666"/></g>',
          blob(64, 40, 50, 6, "#e8d8f0", 0.25, "b5"),
          '<ellipse cx="58" cy="60" rx="50" ry="48" fill="url(#lsAura)"/>']
    b = [f'<ellipse cx="62" cy="106" rx="34" ry="5" fill="#000" fill-opacity="0.45" filter="url(#b3)"/>',
         f'<svg x="14" y="12" width="100" height="100" viewBox="0 0 32 32">{inner}</svg>']
    return icon("Linkshell", "".join(sc), "".join(b), defs, accent="#d6e1f0")


ICONS = {"helm_fishing": fishing, "helm_mining": mining, "helm_logging": logging, "helm_harvesting": harvesting, "helm_excavation": excavation, "ws_sword": ws_sword, "misc_inventory": inventory, "misc_linkshell": linkshell}

if __name__ == "__main__":
    os.makedirs(OUT, exist_ok=True)
    want = sys.argv[1:]
    for name, fn in ICONS.items():
        if want and name not in want:
            continue
        reset()
        open(os.path.join(OUT, name + ".svg"), "w").write(fn())
    print("ok")
