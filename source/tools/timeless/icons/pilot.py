#!/usr/bin/env python3
"""Painterly pilot icons."""
import math, os, sys, random
sys.path.insert(0, os.path.dirname(__file__))
from paint import *

OUT = os.path.join(os.path.dirname(__file__), "pilot")


def m(preset, **kw):
    d = dict(preset)
    d.update(kw)
    return d


# =========================== FISHING ===========================
def fishing():
    defs = (lg("skyF", [(0, "#141a33"), (0.45, "#3b3456"), (0.72, "#a3627a"), (0.86, "#f0a46a")])
            + lg("waterF", [(0, "#3a3a5c"), (0.25, "#1d2440"), (1, "#070a14")])
            + lg("fishBack", [(0, "#2c5560"), (0.45, "#7aa898"), (1, "#e2e8d4")], 0, 0, 0, 1)
            + lg("rodG", [(0, "#c48a52"), (1, "#5a3418")], 0, 0, 1, 0))
    sc = ['<rect x="0" y="0" width="128" height="80" fill="url(#skyF)"/>',
          stars(31, 30, (8, 8, 120, 40)),
          blob(70, 74, 50, 8, "#ffc08a", 0.55, "b5"),
          # far hills + treeline
          '<g filter="url(#organic)"><path d="M0 70 C14 62 24 64 34 60 C44 56 52 62 62 60 C76 57 84 50 96 54 C108 58 116 56 128 60 L128 80 L0 80 Z" fill="#2c2440"/>'
          '<path d="M0 74 C10 70 20 72 30 69 L34 64 L36 69 C46 67 58 70 70 68 L74 62 L77 68 C90 67 104 70 128 68 L128 80 L0 80 Z" fill="#1c1830"/></g>',
          '<rect x="0" y="78" width="128" height="50" fill="url(#waterF)"/>',
          # sun reflection on water
          blob(70, 82, 34, 2.4, "#ffb67a", 0.6, "b2"), blob(68, 88, 22, 1.6, "#ff9a6a", 0.45, "b1"), blob(72, 94, 14, 1.2, "#ff9a6a", 0.3, "b1"),
          ''.join(f'<path d="M{x} {y} h{w}" stroke="#c8b8e8" stroke-opacity="{o}" stroke-width="0.5"/>' for x, y, w, o in
                  ((14, 100, 18, .25), (40, 108, 12, .2), (88, 104, 20, .22), (20, 116, 26, .15), (96, 116, 14, .15)))]
    b = []
    # rod (bent), with cork grip, reel
    rod = "M8 122 C14 96 24 60 46 22 L47.2 22.8 C27 61 18 97 13 123 Z"
    b.append(cast_shadow(rod, 3, 3, 0.4))
    b.append(form(rod, "url(#rodG)", "#2a160a", hi="#e8b880", hl=(30, 60, 2, 14, 25), rim="#ffb27a", tex="wood", off=(1, 0.5), ao=0.3, outline="#1a0c05", ow=0.45))
    def bez(t, p=((8, 122), (14, 96), (24, 60), (46, 22))):
        return tuple((1-t)**3*p[0][i] + 3*(1-t)**2*t*p[1][i] + 3*(1-t)*t*t*p[2][i] + t**3*p[3][i] for i in (0, 1))
    for t in (0.45, 0.65, 0.85):
        x, y = bez(t)
        b.append(f'<circle cx="{x+1.4:.1f}" cy="{y+0.6:.1f}" r="1.0" fill="none" stroke="#d8d0c0" stroke-opacity="0.9" stroke-width="0.45"/>')
    grip = "M7.5 108 C9 101 10.5 96 12 92 L16.5 93.5 C15 98 13.8 103 12.5 109.5 Z"
    b.append(form(grip, "#b88a5a", "#4a2e16", hi="#f0d0a0", hl=(11, 98, 1.6, 4), tex="mottle", off=(1, 1), ao=0.3, ow=0.4))
    b.append(form("M12 104 m-6 0 a6 6 0 1 0 12 0 a6 6 0 1 0 -12 0 Z", "#b8893a", "#3a2408", hi="#ffe2a0", hl=(10, 101, 2.2, 1.6), rim="#ffd080", tex="hammered", off=(1.2, 1.4), ow=0.5))
    b.append('<circle cx="12" cy="104" r="2.2" fill="#2a1a08"/><circle cx="12" cy="104" r="1" fill="#e8c880"/>')
    # line
    b.append('<path d="M46.4 22.4 C52 34 56 42 60.5 50.5" fill="none" stroke="#f4ead8" stroke-opacity="0.85" stroke-width="0.4"/>')
    # splash under fish
    b.append(blob(90, 82, 16, 3, "#c8d8ff", 0.35, "b2"))
    b.append('<ellipse cx="92" cy="83" rx="17" ry="3.2" fill="none" stroke="#dce8ff" stroke-opacity="0.55" stroke-width="0.8"/>')
    b.append('<ellipse cx="92" cy="84" rx="26" ry="4.6" fill="none" stroke="#dce8ff" stroke-opacity="0.22" stroke-width="0.6"/>')
    for x, h, w, o in ((78, 8, 1.6, .55), (83, 13, 2.0, .7), (88, 10, 1.8, .6), (96, 12, 2.0, .7), (101, 9, 1.6, .55), (106, 6, 1.4, .45)):
        b.append(f'<path d="M{x-w} 83 Q{x} {83-h*0.4:.1f} {x+0.4} {83-h} Q{x+0.2} {83-h*0.4:.1f} {x+w} 83 Z" fill="#e6eeff" fill-opacity="{o*0.8:.2f}" filter="url(#b06)"/>')
    # fish (rainbow trout) leaping
    body = "M-22 0 C-18 -8 -4 -11.5 8 -8.5 C14 -6.5 18 -3 21 -1 L21 1 C18 3 14 6 8 8 C-4 11 -18 8 -22 0 Z"
    tail = "M19 0 L32 -10 C29.5 -3.5 29.5 3.5 32 10 Z"
    dors = "M-6 -10.2 C-2 -16.5 6 -15 10 -8"
    fins = ""
    fish = []
    fish.append(form(tail, "#7fa59a", "#2a4048", hi="#d8e8d0", hl=(26, -2, 2, 4), rim="#ffb27a", ao=0.25, ow=0.45, outline="#14242a"))
    fish.append(form(dors + " Z", "#6f9a8c", "#2a4048", ao=0.2, ow=0.4, outline="#14242a"))
    fish.append(form(body, "url(#fishBack)", "#20363e", hi="#fbf6ea", hl=(-6, 4.5, 13, 3.4), rim="#ffb07a", rim_op=0.9,
                     ao=0.4, ow=0.55, outline="#10202a",
                     extra='<path d="M-20 1 C-10 -1 4 -1 20 0 C4 3 -10 3 -20 1 Z" fill="#e8848e" fill-opacity="0.75" filter="url(#b06)"/>'
                     + "".join(f'<circle cx="{x}" cy="{y}" r="0.55" fill="#1a2a2a" fill-opacity="0.75"/>' for x, y in
                               ((-8, -6), (-3, -7), (2, -6.5), (7, -5.5), (11, -4), (-5, -4), (4, -3.6), (14, -2.6), (16, 1.6), (9, 4)))))
    fish.append('<path d="M-9 5 C-7.5 9.5 -4 12.5 -0.5 13 C-2.5 10 -4.5 7.5 -5.6 5.2 Z" fill="#e0a07a" fill-opacity="0.7" stroke="#5a3020" stroke-opacity="0.6" stroke-width="0.3"/>')
    fish.append('<path d="M-8 6 L-2 12.2 M-7 5.6 L-3.2 10.4" stroke="#7a4028" stroke-opacity="0.45" stroke-width="0.25"/>')
    fish.append('<circle cx="-15.6" cy="-2.4" r="1.55" fill="#c8a858"/><circle cx="-15.8" cy="-2.4" r="1.05" fill="#0c0806"/><circle cx="-16.2" cy="-2.9" r="0.32" fill="#fff" fill-opacity="0.8"/>')
    fish.append('<path d="M-22 0 C-20 1.4 -18.4 1.8 -17 1.6" fill="none" stroke="#10202a" stroke-width="0.5"/>')
    fish.append('<path d="M-11 -7 C-11.5 -3 -11 1 -10 4" fill="none" stroke="#10202a" stroke-opacity="0.6" stroke-width="0.45"/>')
    b.append(f'<g transform="translate(80 66) rotate(42)">{cast_shadow(body, 2, 3, 0.35)}{"".join(fish)}</g>')
    # droplets
    rnd = random.Random(5)
    for _ in range(14):
        x, y = rnd.uniform(66, 112), rnd.uniform(56, 80)
        r = rnd.uniform(0.6, 1.5)
        b.append(f'<circle cx="{x:.1f}" cy="{y:.1f}" r="{r:.2f}" fill="#e0ecff" fill-opacity="0.85"/><circle cx="{x-r*0.3:.1f}" cy="{y-r*0.3:.1f}" r="{r*0.35:.2f}" fill="#fff"/>')
    return icon("Fishing", "".join(sc), "".join(b), defs, accent="#f0c8a0")



# =========================== MINING ===========================
def crystal_shard(cx, cy, h, w, ang, cols=("#d8f6ff", "#6fc8e8", "#3a8ab0", "#1e4a6a")):
    t = f'transform="rotate({ang} {cx} {cy})"'
    return (f'<g {t}><polygon points="{cx},{cy-h} {cx-w},{cy-h*0.55} {cx-w*0.8},{cy} {cx},{cy+w*0.4}" fill="{cols[1]}"/>'
            f'<polygon points="{cx},{cy-h} {cx+w},{cy-h*0.55} {cx+w*0.8},{cy} {cx},{cy+w*0.4}" fill="{cols[2]}"/>'
            f'<polygon points="{cx},{cy-h} {cx-w},{cy-h*0.55} {cx},{cy-h*0.45}" fill="{cols[0]}"/>'
            f'<polygon points="{cx},{cy-h} {cx+w},{cy-h*0.55} {cx},{cy-h*0.45}" fill="{cols[1]}"/>'
            f'<path d="M{cx} {cy-h} L{cx-w} {cy-h*0.55} L{cx-w*0.8} {cy} L{cx} {cy+w*0.4} L{cx+w*0.8} {cy} L{cx+w} {cy-h*0.55} Z" fill="none" stroke="{cols[3]}" stroke-width="0.4"/>'
            f'<path d="M{cx-w*0.6} {cy-h*0.5} L{cx-w*0.5} {cy-w*0.2}" stroke="#fff" stroke-opacity="0.7" stroke-width="0.4"/></g>')


def mining():
    defs = rg("caveG", [(0, "#d98a48"), (0.25, "#6a3a24"), (0.6, "#26161a"), (1, "#0a0608")], cx="20%", cy="22%", r="85%")
    sc = ['<rect width="128" height="128" fill="url(#caveG)"/>',
          '<g filter="url(#rock)"><path d="M0 0 L40 0 C34 14 42 30 30 46 C20 60 26 80 10 96 L0 100 Z" fill="#4a3428"/>'
          '<path d="M128 0 L92 0 C98 20 88 34 100 52 C110 66 104 86 120 100 L128 104 Z" fill="#2c2024"/>'
          '<path d="M30 0 L100 0 C92 8 76 12 64 10 C50 8 40 6 30 0 Z" fill="#3a2a26"/></g>',
          blob(20, 24, 22, 20, "#ffb060", 0.35, "b8"),
          # lantern hanging
          '<path d="M22 0 L22 14" stroke="#2a1a10" stroke-width="0.8"/>',
          form("M17 15 L27 15 L28 30 L16 30 Z", "#3a2a14", "#120a04", ao=0.2, ow=0.5, outline="#0a0604"),
          '<rect x="18.5" y="17" width="7" height="11" rx="1" fill="#ffd080"/>', blob(22, 22.5, 4, 6, "#fff2c0", 0.9, "b1"),
          blob(22, 22, 12, 12, "#ffb050", 0.45, "b5"),
          form("M15 13 L29 13 L27 16 L17 16 Z", "#b8893a", "#4a2e0c", tex="hammered", ow=0.4, ao=0.2),
          form("M15 29 L29 29 L28 32 L16 32 Z", "#b8893a", "#4a2e0c", tex="hammered", ow=0.4, ao=0.2)]
    b = []
    boulder = "M4 128 L6 100 C10 86 22 74 40 70 C56 67 72 74 80 88 C86 100 86 114 92 128 Z"
    b.append(form(boulder, "#7a6a62", "#221a18", hi="#d8a878", hl=(30, 80, 18, 8, -15), rim="#ffb070", rim_op=0.5, tex="rock", off=(3, 4), ao=0.5, outline="#120c0a"))
    # ore vein glow + crystals
    b.append(blob(44, 92, 22, 12, "#6fd8ff", 0.45, "b5", -20))
    b.append('<path d="M18 108 C28 98 38 96 46 88 C54 80 64 80 72 76" fill="none" stroke="#9fe8ff" stroke-opacity="0.55" stroke-width="2" filter="url(#b06)"/>')
    for cx, cy, h, w, a in ((38, 96, 14, 4, -20), (46, 92, 18, 5, 8), (54, 90, 11, 3.4, 28), (30, 102, 9, 3, -35), (62, 86, 8, 2.8, 40)):
        b.append(crystal_shard(cx, cy, h, w, a))
    for x, y in ((24, 112), (60, 100), (70, 94), (36, 116)):
        b.append(f'<circle cx="{x}" cy="{y}" r="0.9" fill="#ffd27a"/>')
    # pickaxe
    pick = []
    haft = "M-2.6 2 L2.6 2 L3.2 72 L-3.2 72 Z"
    pick.append(cast_shadow(haft, 3, 3, 0.45))
    pick.append(form(haft, "#9a6438", "#3a2212", hi="#e0aa70", hl=(-1, 30, 1.4, 18), rim="#ffb070", tex="wood", off=(1.2, 0.2), ao=0.35, outline="#1e1008", ow=0.5))
    pick.append('<path d="M-3 56 L3 56 L3.2 72 L-3.2 72 Z" fill="#3a2010" fill-opacity="0.6"/>')
    for y in (58, 61, 64, 67, 70):
        pick.append(f'<path d="M-3.1 {y} L3.1 {y+1.6}" stroke="#c89060" stroke-opacity="0.5" stroke-width="0.5"/>')
    head = "M-36 10 C-26 -1 -12 -6.5 0 -6.5 C10 -6.5 22 -4 30 2.5 L28 6 C20 1.5 10 0.4 0 1 C-11 0.6 -23 4.4 -33 12.5 Z"
    pick.append(cast_shadow(head, 3, 3, 0.5))
    pick.append(form(head, "#6e7078", "#18181e", hi="#e8c8a0", hl=(-10, -4, 10, 1.6), rim="#ff9a50", rim_op=0.75, tex="hammered", off=(1, 1.8), ao=0.4, outline="#0a0a0e", ow=0.55))
    pick.append(form("M-5 -9 L5 -9 L5.6 5 L-5.6 5 Z", "#5a5c64", "#141418", hi="#c8b8a0", hl=(-2, -6, 2, 1.4), tex="hammered", off=(1, 1), ow=0.5, ao=0.3))
    pick.append('<circle cx="0" cy="-2" r="1.1" fill="#2a2a30" stroke="#a8a090" stroke-width="0.4"/>')
    b.append(f'<g transform="translate(70 46) rotate(-34)">{"".join(pick)}</g>')
    # sparks + chips
    rnd = random.Random(8)
    for i in range(12):
        a = math.radians(rnd.uniform(-170, -10)); L = rnd.uniform(5, 14); x0, y0 = 42, 71
        b.append(f'<path d="M{x0+math.cos(a)*3:.1f} {y0+math.sin(a)*3:.1f} L{x0+math.cos(a)*L:.1f} {y0+math.sin(a)*L:.1f}" stroke="#ffd27a" stroke-width="{rnd.uniform(0.4,0.9):.2f}" stroke-linecap="round"/>')
    b.append(blob(42, 71, 7, 5, "#fff0b0", 0.9, "b2"))
    b.append(sparkle(42, 71, 4.5, "#fff6d0"))
    for x, y, r in ((30, 60, 1.6), (52, 58, 1.2), (24, 70, 1.0), (58, 64, 0.9)):
        b.append(f'<polygon points="{x-r},{y} {x},{y-r} {x+r*1.2},{y+r*0.3} {x},{y+r}" fill="#6a5a50" stroke="#1a1210" stroke-width="0.3"/>')
    return icon("Mining", "".join(sc), "".join(b), defs, accent="#f0c090")


# =========================== LOGGING ===========================
def logging():
    defs = (lg("forestG", [(0, "#0c1e14"), (0.5, "#24402a"), (0.8, "#3a4a22"), (1, "#1a1408")])
            + lg("shaft", [(0, "#fff2b8", 0.0), (0.5, "#fff2b8", 0.28), (1, "#fff2b8", 0.0)], 0, 0, 1, 0)
            + rg("ringsG", [(0, "#f0d098"), (0.6, "#d8a868"), (0.9, "#b07a40"), (1, "#7a4a22")]))
    rnd = random.Random(3)
    trunks = []
    for x, w, c, o in ((8, 10, "#16241a", 1), (30, 6, "#1e3022", 0.9), (96, 12, "#14201a", 1), (116, 8, "#1a2a1e", 1), (78, 5, "#2a3a26", 0.8), (50, 4, "#2e4028", 0.7)):
        trunks.append(f'<rect x="{x}" y="0" width="{w}" height="100" fill="{c}" fill-opacity="{o}" filter="url(#b1)"/>')
    sc = ['<rect width="128" height="128" fill="url(#forestG)"/>', "".join(trunks),
          '<g filter="url(#b3)"><polygon points="40,0 60,0 100,96 70,96" fill="url(#shaft)"/><polygon points="66,0 76,0 112,90 94,90" fill="url(#shaft)"/></g>',
          motes(12, 22, (6, 4, 122, 50), ["#d8f0a0", "#f8e8a0", "#a8d080"], (0.6, 1.8)),
          '<g filter="url(#organic)"><path d="M0 104 C20 96 40 100 60 98 C80 96 100 100 128 96 L128 128 L0 128 Z" fill="#1c2412"/></g>',
          '<g filter="url(#organic)">' + "".join(f'<path d="M{x} 104 l2 -{h} l2 {h}" fill="#3a5a26"/>' for x, h in ((10, 8), (18, 6), (94, 9), (104, 7), (114, 10), (122, 6))) + "</g>"]
    b = []
    stump = "M26 80 C24 96 22 108 14 116 C30 114 34 118 44 116 C56 120 72 120 84 116 C94 118 100 114 114 116 C106 108 104 96 102 80 Z"
    b.append(cast_shadow(stump, 3, 4, 0.5))
    b.append(form(stump, "#6a4a30", "#1e120a", hi="#c89060", hl=(40, 92, 8, 14), rim="#f0c070", tex="wood", off=(2, 0), ao=0.5, outline="#140a04"))
    for x in (34, 46, 60, 74, 88, 96):
        b.append(f'<path d="M{x} 86 C{x-1} 96 {x+1} 104 {x-1} 114" fill="none" stroke="#1a0e06" stroke-opacity="0.55" stroke-width="0.8"/>')
    b.append('<path d="M30 98 C34 96 40 100 38 104 C34 106 30 104 30 98 Z" fill="#5a7a2a" fill-opacity="0.8" filter="url(#b06)"/>')
    # top face with rings
    b.append('<ellipse cx="64" cy="80" rx="38" ry="10.5" fill="#3a2410"/>')
    b.append('<ellipse cx="64" cy="79.2" rx="37" ry="9.8" fill="url(#ringsG)"/>')
    for k in range(1, 7):
        b.append(f'<ellipse cx="{64+k*0.3:.1f}" cy="79.4" rx="{37*k/7:.1f}" ry="{9.8*k/7:.1f}" fill="none" stroke="#8a5a2a" stroke-opacity="0.55" stroke-width="0.45"/>')
    b.append('<path d="M64 79 L44 84 M64 79 L90 76" stroke="#5a3418" stroke-opacity="0.6" stroke-width="0.5"/>')
    b.append(blob(54, 77, 14, 3, "#fff2c8", 0.4, "b2"))
    # hatchet: bearded axe biting into the top face, haft up-right
    ax = []
    haft = "M-2.4 0 L2.4 0 L2.8 -60 L-1.6 -60 Z"
    ax.append(cast_shadow(haft, 3, 2, 0.4))
    ax.append(form(haft, "#a87040", "#3a2010", hi="#f0c080", hl=(-0.5, -30, 1.2, 16), rim="#ffd080", tex="wood", off=(1.2, 0), ao=0.3, ow=0.5))
    ax.append('<path d="M-1.6 -60 L2.8 -60 L3 -55 C1 -53 -1 -55 -1.8 -56 Z" fill="#2a1408"/>')
    head = "M-4 -7 L5 -7 C10 -6 16 -9 22 -14 C24 -6 25 4 23 14 C17 8 11 6 5 6 L-4 6 Z"
    cid = uid("axc")
    ax.append(f'<clipPath id="{cid}"><path d="M-40 -40 L40 -40 L40 9 L-40 2 Z"/></clipPath>')
    ax.append(f'<g clip-path="url(#{cid})">' + cast_shadow(head, 2, 2, 0.45)
              + form(head, "#7c7e86", "#1c1c22", hi="#f4e0b8", hl=(10, -3, 7, 2.2), rim="#ffcc80", rim_op=0.75, tex="hammered", off=(1, 1.4), ao=0.35, outline="#0c0c10")
              + '<path d="M22 -14 C24 -6 25 4 23 14" fill="none" stroke="#fff4dc" stroke-opacity="0.8" stroke-width="0.7"/>'
              + '<path d="M5 -7 C10 -6 16 -9 22 -14" fill="none" stroke="#fff4dc" stroke-opacity="0.35" stroke-width="0.4"/></g>')
    ax.append(form("M-5 -9 L4 -9 L4 8 L-5 8 Z", "#5e6068", "#141418", hi="#c8b8a0", hl=(-2, -6, 1.6, 2), ow=0.45, ao=0.3, tex="hammered"))
    ax.append('<path d="M4 2.6 L24.5 4.4" stroke="#140a04" stroke-width="1.8" stroke-linecap="round"/><path d="M4 3.8 L24.5 5.6" stroke="#f0d098" stroke-opacity="0.45" stroke-width="0.5"/>')
    b.append(f'<g transform="translate(58 74) rotate(38)">{"".join(ax)}</g>')
    # chips
    rnd = random.Random(14)
    for _ in range(8):
        x, y = rnd.uniform(74, 104), rnd.uniform(58, 78)
        a = rnd.uniform(0, 180)
        b.append(f'<rect x="{x:.1f}" y="{y:.1f}" width="{rnd.uniform(2,4):.1f}" height="1.3" rx="0.4" transform="rotate({a:.0f} {x:.1f} {y:.1f})" fill="#e8c890" stroke="#6a4020" stroke-width="0.25"/>')
    return icon("Logging", "".join(sc), "".join(b), defs, accent="#e8d8a0")


# =========================== HARVESTING ===========================
def wheat_head(x, y, ang, L=14, col="#e8b850"):
    out = []
    for i in range(7):
        t = i / 6
        for sx in (-1, 1):
            cx = sx * 1.6
            cy = -t * L
            out.append(f'<ellipse cx="{cx}" cy="{cy:.1f}" rx="1.5" ry="2.6" transform="rotate({sx*24} {cx} {cy:.1f})" fill="{col}" stroke="#7a5418" stroke-width="0.3"/>')
            out.append(f'<ellipse cx="{cx-0.4}" cy="{cy-0.6:.1f}" rx="0.5" ry="1.1" transform="rotate({sx*24} {cx} {cy:.1f})" fill="#fff0c0" fill-opacity="0.7"/>')
    out.append(f'<ellipse cx="0" cy="{-L-2.6}" rx="1.2" ry="2.4" fill="{col}" stroke="#7a5418" stroke-width="0.3"/>')
    for i in range(6):
        out.append(f'<path d="M{(-1)**i*1.4} {-i*2.4:.1f} L{(-1)**i*5} {-i*2.4-9:.1f}" stroke="#f0d088" stroke-opacity="0.6" stroke-width="0.25"/>')
    return f'<g transform="translate({x} {y}) rotate({ang})">' + "".join(out) + "</g>"


def harvesting():
    defs = (lg("duskH", [(0, "#4a5a8a"), (0.35, "#c8907a"), (0.6, "#f6c27a"), (0.75, "#f8dca0")])
            + lg("fieldH", [(0, "#c89040"), (1, "#5a3a14")]))
    sc = ['<rect width="128" height="80" fill="url(#duskH)"/>', blob(92, 66, 22, 22, "#fff4d0", 0.9, "b5"), blob(92, 66, 10, 10, "#ffffff", 0.95, "b2"),
          blob(30, 36, 26, 5, "#ffe0c0", 0.45, "b3"), blob(84, 26, 30, 4, "#ffd8c8", 0.35, "b3"),
          '<g filter="url(#organic)"><path d="M0 70 C30 66 70 70 128 66 L128 128 L0 128 Z" fill="url(#fieldH)"/></g>',
          '<g opacity="0.7" filter="url(#b06)">' + "".join(f'<path d="M{x} {72+ (x%7)} l{(x%5)-2} -6" stroke="#f0c060" stroke-width="0.6"/>' for x in range(0, 128, 3)) + "</g>"]
    b = []
    # sheaf
    stalks = []
    for i, a in enumerate((-26, -18, -10, -3, 4, 11, 18, 25, 32)):
        x0, y0 = 76 + (i - 4) * 0.8, 116
        L = 62 - abs(a) * 0.4
        ra = math.radians(a)
        x1, y1 = x0 + math.sin(ra) * L, y0 - math.cos(ra) * L
        stalks.append(f'<path d="M{x0:.1f} {y0} Q{x0+math.sin(ra)*L*0.4:.1f} {y0-L*0.55:.1f} {x1:.1f} {y1:.1f}" stroke="#a87830" stroke-width="1.3" fill="none"/>'
                      f'<path d="M{x0-0.4:.1f} {y0} Q{x0+math.sin(ra)*L*0.4-0.4:.1f} {y0-L*0.55:.1f} {x1-0.4:.1f} {y1:.1f}" stroke="#f0d088" stroke-opacity="0.6" stroke-width="0.4" fill="none"/>')
        stalks.append(wheat_head(x1, y1, a, 13, "#e4b048" if i % 2 else "#f0c860"))
    b.append(cast_shadow("M60 116 C66 80 86 80 92 116 Z", 4, 2, 0.4))
    b.append("".join(stalks))
    # twine
    b.append(form("M66 90 C72 92 82 92 88 90 L88 95 C82 97 72 97 66 95 Z", "#c89a5a", "#4a2e12", hi="#f8e0a8", hl=(74, 91, 5, 1), tex="mottle", ow=0.4, ao=0.3))
    b.append('<path d="M70 92 L72 96 M76 93 L78 97 M82 93 L84 96" stroke="#5a3a14" stroke-width="0.4"/>')
    # sickle
    blade = "M38 92 C22 74 26 44 50 34 C62 30 74 34 80 42 C70 38 58 38 50 44 C36 54 36 74 44 88 Z"
    b.append(cast_shadow(blade, 3, 3, 0.4))
    b.append(form(blade, "#8a8c94", "#202028", hi="#ffe8c0", hl=(34, 62, 2.6, 14, 20), rim="#ffc070", rim_op=0.8, tex="hammered", off=(1.2, 1.2), ao=0.35, outline="#0e0e14"))
    b.append('<path d="M50 44 C36 54 36 74 44 88" fill="none" stroke="#fff0d0" stroke-opacity="0.75" stroke-width="0.6"/>')
    handle = "M36.5 90 L45 87 L36 116 L28.6 114 Z"
    b.append(form(handle, "#9a6438", "#3a2212", hi="#e8b070", hl=(34, 100, 1.6, 8, 18), rim="#ffc070", tex="wood", off=(1, 0), ao=0.3))
    b.append(form("M35 88 L46 84.6 L47 88.6 L36 92 Z", "#c09040", "#4a2e0c", hi="#ffe6a8", hl=(40, 87, 3, 1), tex="hammered", ow=0.4, ao=0.2))
    b.append(motes(19, 10, (14, 20, 120, 70), ["#fff4d0", "#ffe0a0"], (0.4, 1.0)))
    return icon("Harvesting", "".join(sc), "".join(b), defs, accent="#f8e0b0")


# =========================== EXCAVATION ===========================
def excavation():
    defs = lg("skyE", [(0, "#6a8ab0"), (1, "#c8d4d8")])
    layers = [(0, 26, "#4a2e1a"), (26, 50, "#5e3e22"), (50, 72, "#74502a"), (72, 96, "#5a3626"), (96, 128, "#3a2c28")]
    sc = ['<rect width="128" height="22" fill="url(#skyE)"/>']
    for y0, y1, c in layers:
        sc.append(f'<g filter="url(#organic)"><path d="M0 {y0+16} C30 {y0+12} 70 {y0+20} 128 {y0+14} L128 {y1+18} L0 {y1+18} Z" fill="{c}"/></g>')
    sc.append('<g filter="url(#rock)"><rect x="0" y="20" width="128" height="108" fill="#8a6a4a" fill-opacity="0.25"/></g>')
    sc.append('<g filter="url(#organic)">' + "".join(f'<path d="M{x} 22 l1.4 -{h} l1.4 {h}" fill="#4a7a2a"/>' for x, h in ((4, 5), (12, 7), (22, 4), (34, 6), (90, 6), (100, 4), (110, 7), (120, 5))) + "</g>")
    sc.append('<path d="M10 30 C14 40 10 50 16 58 M110 28 C106 40 114 46 108 56" fill="none" stroke="#2a1a10" stroke-opacity="0.7" stroke-width="0.8"/>')
    rnd = random.Random(6)
    for _ in range(7):
        x, y, r = rnd.uniform(6, 122), rnd.uniform(34, 124), rnd.uniform(1.2, 2.8)
        sc.append(form(f"M{x-r:.1f} {y:.1f} C{x-r:.1f} {y-r*0.9:.1f} {x+r:.1f} {y-r:.1f} {x+r:.1f} {y:.1f} C{x+r:.1f} {y+r*0.8:.1f} {x-r:.1f} {y+r*0.9:.1f} {x-r:.1f} {y:.1f} Z",
                       "#a8988a", "#3a302a", hi="#e8dccc", hl=(x - r * 0.3, y - r * 0.4, r * 0.4, r * 0.3), ao=0.3, ow=0.3, outline="#1a120c", off=(0.6, 0.8), blur="b06"))
    # ammonite fossil
    sc.append('<g transform="translate(96 104)">' + form("M-9 0 a9 8 0 1 0 18 0 a9 8 0 1 0 -18 0 Z", "#c8b090", "#4a3a2a", hi="#f4e8d0", hl=(-3, -3, 3, 2), ao=0.35, tex="rock", ow=0.4)
              + '<path d="M0 0 m0 0 a1.5 1.5 0 1 1 1.6 1.4 a3.2 3 0 1 1 -3.6 -3 a5.2 4.8 0 1 1 6.2 5 a7.4 6.6 0 1 1 -9.2 -7" fill="none" stroke="#5a4430" stroke-width="0.6"/></g>')
    b = []
    # relic: ancient gold-green medallion half buried
    relic = "M24 96 a14 14 0 1 1 28 0 Z"
    b.append(blob(38, 92, 20, 13, "#ffe090", 0.6, "b5"))
    b.append(form(relic, "#b89848", "#3a2c10", hi="#fff0b8", hl=(32, 86, 5, 3), rim="#a8e0b8", rim_op=0.6, tex="hammered", off=(1, 1.4), ao=0.4))
    b.append('<path d="M28.5 96 a9.5 9.5 0 1 1 19 0" fill="none" stroke="#5a7a50" stroke-opacity="0.8" stroke-width="1.6"/>')
    b.append('<path d="M38 86 L42 92 L38 96 L34 92 Z" fill="#3a8a6a" stroke="#1a3a2a" stroke-width="0.4"/><path d="M38 86 L34 92 L38 91 Z" fill="#a8f0c8" fill-opacity="0.7"/>')
    b.append('<path d="M20 96 C30 94 46 94 56 96 C50 100 28 100 20 96 Z" fill="#5a3a22"/>')
    b.append(sparkle(46, 84, 4, "#fff4c8"))
    # spade
    shaft = "M98 18 L102 20.4 L74 66 L70.4 63.6 Z"
    b.append(cast_shadow(shaft, 3, 3, 0.45))
    b.append(form(shaft, "#a87040", "#3a2010", hi="#f0c080", hl=(88, 38, 1.4, 14, 31), rim="#ffd8a0", tex="wood", off=(1, 0.4), ao=0.3, ow=0.5))
    grip = "M94 10 C96 6 104 6 108 10 L110 20 L104 22 L102 14 C101 12 98 12 97 14 L98 22 L92 20 Z"
    b.append(form(grip, "#a87040", "#3a2010", hi="#f0c080", hl=(100, 9, 4, 1.4), tex="wood", ow=0.5, ao=0.3))
    blade = "M66 60 L78 68 C78 76 74 86 64 92 C58 92 54 88 52 82 C56 74 60 66 66 60 Z"
    b.append(cast_shadow(blade, 3, 3, 0.5))
    b.append(form(blade, "#82848c", "#1e1e24", hi="#f0e0c0", hl=(62, 70, 4, 6, 30), rim="#c8d8ff", rim_op=0.6, tex="hammered", off=(1.2, 1.6), ao=0.4))
    b.append('<path d="M54 86 C58 90 62 91 64 92 L66 90 C62 88 58 86 56 82 Z" fill="#6a4a2e" fill-opacity="0.85"/>')
    # flying dirt clods
    for x, y, r in ((50, 52, 1.6), (44, 58, 1.1), (58, 48, 1.2), (40, 50, 0.9)):
        b.append(f'<circle cx="{x}" cy="{y}" r="{r}" fill="#6a4a2e" stroke="#2a1a10" stroke-width="0.3"/>')
    return icon("Excavation", "".join(sc), "".join(b), defs, accent="#e8d0a8")



# =========================== shared painted parts ===========================
from gen2 import logo_hourglass
HG_DEFS = """
  <linearGradient id="hfMetal" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#8e98aa"/><stop offset="0.45" stop-color="#f1f3f7"/><stop offset="1" stop-color="#7d8597"/></linearGradient>
  <linearGradient id="oGlass" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#dfe4ec"/><stop offset="0.5" stop-color="#aeb6c4" stop-opacity="0.45"/><stop offset="1" stop-color="#dfe4ec"/></linearGradient>
  <radialGradient id="hfBack"><stop offset="0" stop-color="#6f93c8" stop-opacity="0.45"/><stop offset="0.45" stop-color="#3d5f94" stop-opacity="0.2"/><stop offset="1" stop-color="#3d5f94" stop-opacity="0"/></radialGradient>
  <radialGradient id="oCyan"><stop offset="0" stop-color="#e8fbff" stop-opacity="0.85"/><stop offset="0.35" stop-color="#7fd8f0" stop-opacity="0.35"/><stop offset="1" stop-color="#5fb8e0" stop-opacity="0"/></radialGradient>
  <linearGradient id="r13Sand" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#eef3fb" stop-opacity="0.95"/><stop offset="0.35" stop-color="#bcd0ea" stop-opacity="0.88"/><stop offset="0.75" stop-color="#7f9cc8" stop-opacity="0.85"/><stop offset="1" stop-color="#4f6a96" stop-opacity="0.85"/></linearGradient>
  <linearGradient id="oWispV" x1="0" y1="1" x2="0" y2="0"><stop offset="0" stop-color="#c3a2f0" stop-opacity="0.8"/><stop offset="1" stop-color="#9a86e0" stop-opacity="0"/></linearGradient>
"""


def gem(cx, cy, r, c1="#3a6ad0", c2="#0c1a4a", c3="#a8d0ff"):
    return (f'<path d="M{cx} {cy-r} L{cx+r*0.85:.2f} {cy} L{cx} {cy+r} L{cx-r*0.85:.2f} {cy} Z" fill="{c1}"/>'
            f'<path d="M{cx} {cy} L{cx+r*0.85:.2f} {cy} L{cx} {cy+r} Z" fill="{c2}"/>'
            f'<path d="M{cx} {cy-r} L{cx-r*0.85:.2f} {cy} L{cx} {cy} Z" fill="{c3}" fill-opacity="0.8"/>'
            f'<circle cx="{cx-r*0.25:.2f}" cy="{cy-r*0.35:.2f}" r="{r*0.18:.2f}" fill="#fff" fill-opacity="0.85"/>'
            f'<path d="M{cx} {cy-r} L{cx+r*0.85:.2f} {cy} L{cx} {cy+r} L{cx-r*0.85:.2f} {cy} Z" fill="none" stroke="#0a0a14" stroke-width="0.35"/>')


def sword_painted(L=88, gemcol=("#3a6ad0", "#0c1a4a", "#a8d0ff")):
    s = []
    blade = f"M0 {-L} L-3.4 {-L+13} L-4.5 -3 L4.5 -3 L3.4 {-L+13} Z"
    s.append(form(blade, "#b4bcc8", "#2a303c", hi="#f4ece0", hl=(-2, -L * 0.45, 1.6, L * 0.3), rim="#8ab0e0", rim_op=0.6,
                  tex="mottle", off=(1.6, 0.6), ao=0.35, outline="#10131a", ow=0.5,
                  extra=f'<path d="M0 {-L+3} L0 -6" stroke="#20242e" stroke-opacity="0.55" stroke-width="1.1" filter="url(#b06)"/>'
                        f'<path d="M-0.7 {-L+4} L-0.7 -6" stroke="#ffffff" stroke-opacity="0.45" stroke-width="0.5" filter="url(#b06)"/>'
                        f'<path d="M0 {-L} L3.4 {-L+13} L4.5 -3 L0 -3 Z" fill="#141a26" fill-opacity="0.42"/>'
                        f'<path d="M0 {-L} L-3.4 {-L+13} L-4.5 -3 L-2.2 -3 L-1.7 {-L+14} Z" fill="#fff8ee" fill-opacity="0.22"/>'))
    guard = "M-12 -2.4 C-7 -3 -4 -3 0 -3.2 C4 -3 7 -3 12 -2.4 L14 0 L12 2.4 C7 1.9 4 2.2 0 2.2 C-4 2.2 -7 1.9 -12 2.4 L-14 0 Z"
    s.append(form(guard, "#c49440", "#3e2608", hi="#ffe8a8", hl=(-6, -1.4, 5, 1), rim="#ffd890", tex="hammered", off=(0.6, 1.2), ao=0.35, outline="#1e1206", ow=0.5))
    s.append(form("M0 -8 L3.6 -2.6 L3.6 2.2 L0 4.2 L-3.6 2.2 L-3.6 -2.6 Z", "#c49440", "#3e2608", hi="#ffe8a8", hl=(-1.6, -4, 1.4, 2), tex="hammered", off=(0.8, 1), ao=0.3, ow=0.45, outline="#1e1206"))
    s.append(gem(0, -0.8, 1.9, *gemcol))
    grip = "M-2.4 4.6 L2.4 4.6 L2.1 18.4 L-2.1 18.4 Z"
    s.append(form(grip, "#6a3c22", "#1e0e06", hi="#b87a4a", hl=(-1, 10, 0.9, 4), tex="mottle", off=(0.8, 0), ao=0.3, ow=0.4, outline="#120804",
                  extra="".join(f'<path d="M-2.4 {5.4+i*1.6:.1f} L2.4 {6.6+i*1.6:.1f}" stroke="#120804" stroke-opacity="0.6" stroke-width="0.4"/>' for i in range(8))))
    s.append(form("M0 18 C2.8 18.4 3.8 21 3 23.4 C2.2 25 1 25.6 0 26.8 C-1 25.6 -2.2 25 -3 23.4 C-3.8 21 -2.8 18.4 0 18 Z", "#c49440", "#3e2608", hi="#ffe8a8", hl=(-1.2, 20.6, 1, 1.6), tex="hammered", off=(0.6, 0.8), ao=0.3, ow=0.45, outline="#1e1206"))
    return "".join(s)


def medallion_painted(cx=64, cy=61, r=15.5):
    ring = f"M{cx-r} {cy} a{r} {r} 0 1 0 {2*r} 0 a{r} {r} 0 1 0 {-2*r} 0 Z M{cx-r+2.6} {cy} a{r-2.6} {r-2.6} 0 1 1 {2*r-5.2} 0 a{r-2.6} {r-2.6} 0 1 1 {-2*r+5.2} 0 Z"
    sc = (r - 3.4) * 2 / 196
    return (f'<circle cx="{cx+1.5}" cy="{cy+2.5}" r="{r+0.5}" fill="#000" fill-opacity="0.55" filter="url(#cast)"/>'
            f'<circle cx="{cx}" cy="{cy}" r="{r-2.4}" fill="#10141e"/>'
            + blob(cx, cy - 2, r - 5, r - 5, "#3d5f94", 0.55, "b3")
            + f'<g transform="translate({cx} {cy}) scale({sc:.4f})">{logo_hourglass(w=6.0)}</g>'
            + f'<g fill-rule="evenodd">{form(ring, "#a8aab0", "#2a2c34", hi="#fff4e0", hl=(cx-6, cy-11, 6, 2.4, -25), rim="#9ab8e8", tex="hammered", off=(0.8, 1.2), ao=0.35, outline="#0e1016", ow=0.5)}</g>')


# =========================== ATTACK ===========================
def attack():
    defs = lg("nightA", [(0, "#070b1c"), (0.5, "#16203c"), (1, "#2c3456")]) + HG_DEFS
    clouds = []
    for x, y, rx, ry, o in ((22, 92, 30, 8, .55), (100, 96, 34, 9, .5), (64, 108, 50, 10, .6), (30, 30, 24, 5, .35), (104, 24, 22, 4, .3)):
        clouds.append(blob(x, y, rx, ry, "#3a4870", o, "b3"))
        clouds.append(blob(x - rx * 0.2, y - ry * 0.5, rx * 0.7, ry * 0.4, "#8a9ac8", o * 0.55, "b2"))
    sc = ['<rect width="128" height="128" fill="url(#nightA)"/>', stars(41, 60, (6, 6, 122, 80)),
          blob(64, 58, 30, 30, "#c8d8ff", 0.35, "b8"), blob(64, 58, 16, 16, "#e8f0ff", 0.4, "b5"), "".join(clouds),
          '<g filter="url(#organic)" fill="#0a0e1c"><path d="M0 118 L0 108 L8 108 L8 100 L12 96 L16 100 L16 108 L26 108 L26 104 L34 104 L34 112 L94 112 L94 102 L98 98 L102 102 L102 106 L112 106 L112 96 L116 92 L120 96 L120 108 L128 108 L128 128 L0 128 Z"/></g>',
          '<rect x="10" y="102" width="1.4" height="2" fill="#ffc870" fill-opacity="0.8"/><rect x="114.2" y="100" width="1.4" height="2" fill="#ffc870" fill-opacity="0.8"/>']
    sw = sword_painted(88)
    b = [f'<g transform="translate(64 64) rotate(-40) translate(0 28)">{cast_shadow("M-4 -80 L4 -80 L4 26 L-4 26 Z", 3, 3, 0.35)}{sw}</g>',
         f'<g transform="translate(64 64) rotate(40) translate(0 28)">{cast_shadow("M-4 -80 L4 -80 L4 26 L-4 26 Z", 3, 3, 0.35)}{sw}</g>',
         medallion_painted()]
    return icon("Attack", "".join(sc), "".join(b), defs, accent="#d6e1f0")


# =========================== SWORD WS ===========================
def ws_sword():
    defs = (lg("emberS", [(0, "#12060a"), (0.55, "#3a1410"), (1, "#8a3a18")])
            + lg("arcG", [(0, "#fff6e0", 0), (0.35, "#ffd08a", 0.55), (0.75, "#fff8e8", 1), (1, "#ffffff", 1)], 0, 1, 1, 0))
    sc = ['<rect width="128" height="128" fill="url(#emberS)"/>',
          blob(30, 100, 40, 14, "#1a0806", 0.8, "b5"), blob(100, 104, 36, 12, "#200a08", 0.7, "b5"),
          blob(80, 30, 40, 12, "#2a1014", 0.6, "b5"), blob(20, 40, 28, 10, "#3a1812", 0.5, "b5"),
          motes(55, 26, (6, 30, 122, 124), ["#ffb060", "#ff8a40", "#ffd08a"], (0.4, 1.1))]
    arc = "M20 96 C24 52 56 22 104 20 C70 30 42 56 32 98 Z"
    b = [f'<path d="{arc}" fill="#ff9a50" fill-opacity="0.45" filter="url(#b5)"/>',
         f'<path d="{arc}" fill="url(#arcG)" filter="url(#b06)"/>',
         '<path d="M26 92 C32 56 58 30 98 22" fill="none" stroke="#ffffff" stroke-opacity="0.9" stroke-width="0.8" filter="url(#b06)"/>',
         "".join(f'<path d="M{x} {y} l{dx} {dy}" stroke="#ffd8a0" stroke-opacity="{o}" stroke-width="0.6" stroke-linecap="round"/>' for x, y, dx, dy, o in
                 ((40, 40, -8, 6, .6), (56, 30, -9, 4, .5), (30, 60, -6, 8, .5), (74, 24, -8, 2, .45)))]
    b.append(f'<g transform="translate(70 66) rotate(45) translate(0 26)">{cast_shadow("M-4 -80 L4 -80 L4 26 L-4 26 Z", 3, 3, 0.4)}{sword_painted(86, ("#d04040", "#4a0a0a", "#ffb0a0"))}</g>')
    b.append(sparkle(100, 21, 5, "#fff6e0"))
    return icon("Weapon Skill - Sword", "".join(sc), "".join(b), defs, accent="#f0c090")


# =========================== FIRE ===========================
def fire():
    defs = (rg("fireBg", [(0, "#4a1408"), (0.55, "#1e0806"), (1, "#080303")], cy="62%")
            + lg("fl1", [(0, "#ff6a2a", 0.0), (0.25, "#e8401a"), (1, "#8a1a0a")])
            + lg("fl2", [(0, "#ffb050", 0.0), (0.3, "#ff8a30"), (1, "#e05018")])
            + lg("fl3", [(0, "#fff0a0", 0.0), (0.35, "#ffd060"), (1, "#ffa030")])
            + lg("fl4", [(0, "#ffffff", 0.0), (0.4, "#fffbe8"), (1, "#ffe8a0")]))
    sc = ['<rect width="128" height="128" fill="url(#fireBg)"/>',
          blob(40, 40, 22, 10, "#3a2a26", 0.5, "b5", -30), blob(88, 30, 24, 9, "#2e2220", 0.5, "b5", 25), blob(64, 20, 30, 7, "#2a2020", 0.45, "b5"),
          blob(64, 92, 40, 16, "#ff7a2a", 0.4, "b8")]
    # casting circle
    ring = []
    ring.append('<ellipse cx="64" cy="98" rx="34" ry="9" fill="none" stroke="#ff9a4a" stroke-opacity="0.45" stroke-width="2.4" filter="url(#b1)"/>')
    ring.append('<ellipse cx="64" cy="98" rx="34" ry="9" fill="none" stroke="#ffd8a0" stroke-opacity="0.85" stroke-width="0.5"/>')
    ring.append('<ellipse cx="64" cy="98" rx="27" ry="7" fill="none" stroke="#ffd8a0" stroke-opacity="0.5" stroke-width="0.4"/>')
    for i in range(18):
        a = math.radians(i * 20)
        ring.append(f'<path d="M{64+27*math.cos(a):.1f} {98+7*math.sin(a):.1f} L{64+34*math.cos(a):.1f} {98+9*math.sin(a):.1f}" stroke="#ffd8a0" stroke-opacity="0.55" stroke-width="0.45"/>')
    f = ("M64 18 C68 30 82 38 84 54 C85 62 83 70 79 77 C85 72 88 66 88 59 C95 70 92 86 81 94 C75 99 69 100 64 100 "
         "C56 100 48 98 43 92 C35 84 35 69 41 60 C41 66 43 72 47 76 C43 64 46 52 54 45 C59 40 62 30 64 18 Z")
    flames = [f'<g filter="url(#flameD)"><path d="{f}" fill="url(#fl1)"/>',
              f'<g transform="translate(64 100) scale(0.78) translate(-64 -100)"><path d="{f}" fill="url(#fl2)"/></g>',
              f'<g transform="translate(64 100) scale(0.55) translate(-64 -100)"><path d="{f}" fill="url(#fl3)"/></g>',
              f'<g transform="translate(64 100) scale(0.32) translate(-64 -100)"><path d="{f}" fill="url(#fl4)"/></g></g>']
    b = ["".join(ring), blob(64, 74, 22, 26, "#ff8a3a", 0.45, "b5"), "".join(flames),
         motes(77, 22, (24, 10, 104, 90), ["#ffd27a", "#ff9a4a", "#fff0c0"], (0.4, 1.2))]
    return icon("Spell - Fire", "".join(sc), "".join(b), defs, accent="#f0b080")


# =========================== CURE ===========================
def cure():
    defs = (lg("meadow", [(0, "#06121a"), (0.55, "#0f2a2a"), (1, "#1a3a28")])
            + lg("petal", [(0, "#ffffff"), (0.55, "#f4e8f0"), (1, "#d8a8c8")])
            + lg("petalB", [(0, "#f8eef4"), (1, "#b888a8")])
            + lg("beam", [(0, "#e8fff0", 0), (0.6, "#e8fff0", 0.35), (1, "#e8fff0", 0)], 0, 0, 1, 0))
    sc = ['<rect width="128" height="128" fill="url(#meadow)"/>', stars(61, 30, (6, 6, 122, 50), "#e8fff4"),
          '<g filter="url(#b2)"><polygon points="54,0 74,0 72,84 56,84" fill="url(#beam)"/></g>',
          '<g filter="url(#organic)"><path d="M0 98 C20 92 44 96 64 94 C84 92 106 96 128 92 L128 128 L0 128 Z" fill="#0a1e18"/></g>',
          '<g filter="url(#organic)">' + "".join(f'<path d="M{x} 100 q{(x%5)-2} -{h/2} {(x%3)-1} -{h}" fill="none" stroke="#2a5a3a" stroke-width="0.9"/>' for x, h in
                                               ((6, 12), (12, 16), (18, 10), (100, 14), (108, 18), (116, 12), (122, 16), (30, 8), (92, 9))) + "</g>",
          motes(66, 14, (8, 30, 120, 96), ["#d8ff9a", "#f0ffb0"], (0.5, 1.1))]
    b = [blob(64, 72, 30, 18, "#c8ffd8", 0.35, "b8")]
    # lily pad + water ring
    b.append('<ellipse cx="64" cy="94" rx="34" ry="7" fill="#c8ffd8" fill-opacity="0.15" filter="url(#b1)"/>')
    b.append(form("M34 92 C34 86 50 84 64 84 C80 84 94 86 94 92 C94 98 80 100 64 100 C48 100 34 98 34 92 Z M64 92 L84 86 L82 90 Z",
                  "#3a7a4a", "#0e2a18", hi="#a8e0a0", hl=(50, 88, 12, 2), tex="mottle", off=(0.8, 1.2), ao=0.35, outline="#06140c"))
    # lotus petals (back to front)
    petal = "M0 0 C-7 -8 -7.5 -22 0 -32 C7.5 -22 7 -8 0 0 Z"
    for a, L, fill in ((-62, 0.78, "url(#petalB)"), (62, 0.78, "url(#petalB)"), (-34, 0.92, "url(#petalB)"), (34, 0.92, "url(#petalB)"),
                       (0, 1.0, "url(#petal)"), (-18, 0.86, "url(#petal)"), (18, 0.86, "url(#petal)")):
        b.append(f'<g transform="translate(64 88) rotate({a}) scale({L})">'
                 + form(petal, fill, "#8a5a7a", hi="#ffffff", hl=(-2, -18, 2.4, 8), rim="#c8ffd8", rim_op=0.7, off=(1.2, 1.6), ao=0.3, outline="#5a3a52", ow=0.45,
                        extra='<path d="M0 -3 L0 -28" stroke="#d8a8c8" stroke-opacity="0.6" stroke-width="0.5"/>')
                 + "</g>")
    b.append(blob(64, 74, 9, 7, "#fff8d0", 0.95, "b2"))
    b.append('<circle cx="64" cy="74" r="2.6" fill="#ffffff"/>')
    b.append(sparkle(64, 46, 5.5, "#f4fff8"))
    b.append(sparkle(50, 56, 2.4, "#f4fff8", 0.8))
    b.append(sparkle(80, 52, 2.8, "#f4fff8", 0.8))
    return icon("Spell - Cure", "".join(sc), "".join(b), defs, accent="#d8f5e0")


# =========================== ITEMS ===========================
def items():
    defs = (rg("candle", [(0, "#ffc070", 0.55), (1, "#ffc070", 0)], cx="20%", cy="15%", r="70%")
            + lg("planks", [(0, "#5a3a24"), (1, "#2a180e")])
            + lg("potion", [(0, "#ff7a8a"), (0.5, "#c8202e"), (1, "#5a0810")], 0, 0, 1, 0)
            + lg("glassG", [(0, "#ffffff", 0.35), (1, "#a8c8ff", 0.1)], 0, 0, 1, 0))
    planks = ['<rect width="128" height="128" fill="url(#planks)"/>']
    for y in (0, 26, 52, 78, 104):
        planks.append(f'<g filter="url(#woodH)"><rect x="0" y="{y}" width="128" height="25" fill="#6a4428"/></g>')
        planks.append(f'<rect x="0" y="{y+24.4}" width="128" height="1.6" fill="#140a04" fill-opacity="0.8"/>')
    planks.append('<rect width="128" height="128" fill="url(#candle)"/>')
    sc = "".join(planks)
    b = []
    # scroll peeking (behind)
    b.append(form("M74 24 L82 22 L94 60 L86 62 Z", "#e8d6aa", "#8a7048", hi="#fff6e0", hl=(80, 30, 2, 8, -18), tex="paperT", off=(1, 0.5), ao=0.3, outline="#4a3418", ow=0.45))
    b.append(form("M72 22 a5 3 -18 1 0 11 -3.4 a5 3 -18 1 0 -11 3.4 Z", "#d8c49a", "#7a6040", hi="#fff6e0", hl=(76, 20, 3, 1), ow=0.45, ao=0.2))
    b.append('<path d="M80 40 L88 38 L89 42 L81 44 Z" fill="#a8202e"/>')
    # potion (behind satchel flap)
    bottle = "M44 50 C38 52 34 58 34 64 C34 72 40 76 48 76 C56 76 62 72 62 64 C62 58 58 52 52 50 L52 40 L44 40 Z"
    b.append(cast_shadow(bottle, 3, 3, 0.5))
    b.append(form(bottle, "url(#potion)", "#3a0408", hi="#ffd0d8", hl=(40, 58, 3, 6, 20), rim="#ffb0a0", rim_op=0.7, off=(1, 1), ao=0.3, outline="#1e0204", ow=0.5,
                  extra='<path d="M36 60 C36 56 40 52 44 51 L44 54 C41 55 38 58 38 62 Z" fill="#fff" fill-opacity="0.35"/>'))
    b.append(form("M43 34 L53 34 L52.6 41 L43.4 41 Z", "#b88a5a", "#4a2e16", hi="#f0d0a0", hl=(46, 36, 1.6, 1.4), tex="mottle", ow=0.45, ao=0.2))
    b.append('<path d="M43 42 L53 42" stroke="#e8c870" stroke-width="1.2"/>')
    # satchel
    body = "M22 66 C22 60 26 58 32 58 L96 58 C102 58 106 60 106 66 L104 110 C104 114 100 116 96 116 L32 116 C28 116 24 114 24 110 Z"
    b.append(cast_shadow(body, 4, 5, 0.6))
    b.append(form(body, "#8a5230", "#2a1408", hi="#d89a62", hl=(40, 92, 14, 10), rim="#ffb070", tex="mottle", off=(2.4, 3), ao=0.5, outline="#1a0c05",
                  extra='<path d="M29 70 L28 108 M99 70 L98 108" stroke="#e8c090" stroke-opacity="0.45" stroke-width="0.6" stroke-dasharray="1.6 1.4"/>'))
    flap = "M20 64 C20 58 26 56 32 56 L96 56 C102 56 108 58 108 64 L105 84 C96 94 32 94 23 84 Z"
    b.append(form(flap, "#9a5c36", "#2e160a", hi="#e8a870", hl=(44, 64, 18, 4), rim="#ffc080", tex="mottle", off=(1.6, 2.4), ao=0.45, outline="#1a0c05",
                  extra='<path d="M25 62 C25 60 28 60 32 60 L96 60 C100 60 103 60 103 62 L101 82 C92 90 36 90 27 82 Z" fill="none" stroke="#f0c898" stroke-opacity="0.45" stroke-width="0.6" stroke-dasharray="1.6 1.4"/>'))
    b.append(form("M59 80 L69 80 L69 102 L59 102 Z", "#6a3a1e", "#1e0e06", hl=None, tex="mottle", ow=0.45, ao=0.3))
    buckle = "M55 82 L73 82 L73 96 L55 96 Z M58 85 L70 85 L70 93 L58 93 Z"
    b.append(f'<g fill-rule="evenodd">{form(buckle, "#c49440", "#3e2608", hi="#ffe8a8", hl=(60, 83, 6, 1), rim="#ffd890", tex="hammered", off=(0.6, 1), ao=0.3, outline="#1e1206", ow=0.5)}</g>')
    b.append('<path d="M64 82.6 L64 95.4" stroke="#c49440" stroke-width="1.4"/><path d="M63.4 82.6 L63.4 95.4" stroke="#ffe8a8" stroke-opacity="0.7" stroke-width="0.4"/>')
    return icon("Items", sc, "".join(b), defs, accent="#f0d0a0")


# =========================== MAP ===========================
def mapi():
    defs = (rg("candleM", [(0, "#ffc070", 0.5), (1, "#ffc070", 0)], cx="85%", cy="10%", r="70%")
            + lg("tableM", [(0, "#4a2e1c"), (1, "#1e120a")]))
    sc = ['<rect width="128" height="128" fill="url(#tableM)"/>', '<g filter="url(#wood)"><rect width="128" height="128" fill="#5a3820"/></g>',
          '<rect width="128" height="128" fill="url(#candleM)"/>']
    b = []
    sheet = "M18 30 C40 26 88 34 110 28 L112 98 C88 104 40 96 16 102 Z"
    b.append(cast_shadow(sheet, 4, 5, 0.6))
    b.append(form(sheet, "#e2cc9c", "#8a6e44", hi="#fff6dc", hl=(70, 50, 30, 18), tex="paperT", off=(3, 4), ao=0.5, outline="#4a3418", blur="b3"))
    # sea + land painted
    cid = uid("mc")
    land = ('<path d="M24 40 C34 36 44 44 50 40 C58 36 62 46 70 44 C76 42 78 52 72 58 C66 66 74 72 66 80 C58 88 48 82 40 88 C32 92 26 84 24 76 Z" fill="#8a9a5a"/>'
            '<path d="M80 36 C90 34 102 38 106 46 C108 54 98 56 96 64 C94 70 100 80 92 86 C86 90 80 82 82 74 C84 64 76 58 78 50 Z" fill="#9aa060"/>')
    b.append(f'<clipPath id="{cid}"><path d="{sheet}"/></clipPath><g clip-path="url(#{cid})">'
             '<rect x="0" y="0" width="128" height="128" fill="#6a9ab0" fill-opacity="0.35"/>'
             f'<g filter="url(#organic)">{land}</g>'
             '<g filter="url(#organic)"><path d="M34 54 C40 50 46 56 52 52 M86 48 C92 46 96 52 100 50" fill="none" stroke="#5a6a3a" stroke-width="1.2"/></g>'
             + "".join(f'<path d="M{x-4} {y+3} L{x} {y-3} L{x+4} {y+3}" fill="#7a6a4a" stroke="#4a3418" stroke-width="0.5"/>' for x, y in ((40, 64), (48, 70), (56, 62), (94, 72)))
             + "".join(f'<circle cx="{x}" cy="{y}" r="1.6" fill="#4a6a2a"/>' for x, y in ((32, 50), (36, 46), (60, 52), (100, 58), (88, 80)))
             + '<path d="M30 80 C40 72 50 76 58 66 C64 58 74 62 84 56 C88 54 92 60 96 54" fill="none" stroke="#a8202e" stroke-width="1.1" stroke-dasharray="2.2 1.6"/>'
             '<path d="M93 50 L99 56 M99 50 L93 56" stroke="#a8202e" stroke-width="1.8" stroke-linecap="round"/>'
             '<rect x="0" y="0" width="128" height="128" filter="url(#paperT)" fill="#e2cc9c" fill-opacity="0.25"/></g>')
    # curled edges
    b.append(form("M14 26 C38 22 88 30 112 24 L112 32 C88 38 40 30 16 34 Z", "#d8c094", "#7a5e38", hi="#fff6dc", hl=(60, 26, 30, 1.4), tex="paperT", ow=0.45, ao=0.3, outline="#4a3418"))
    # compass
    cmp = "M80 86 a16 16 0 1 0 32 0 a16 16 0 1 0 -32 0 Z"
    b.append(cast_shadow(cmp, 3, 4, 0.6))
    b.append(form(cmp, "#c49440", "#3e2608", hi="#ffe8a8", hl=(90, 76, 6, 3, -20), rim="#ffd890", tex="hammered", off=(1, 1.4), ao=0.35, outline="#1e1206"))
    b.append('<circle cx="96" cy="86" r="12.4" fill="#f0e6cc"/><circle cx="96" cy="86" r="12.4" fill="none" stroke="#3e2608" stroke-width="0.5"/>')
    for i in range(16):
        a = math.radians(i * 22.5)
        r0 = 10 if i % 4 else 8.4
        b.append(f'<path d="M{96+math.cos(a)*r0:.1f} {86+math.sin(a)*r0:.1f} L{96+math.cos(a)*11.6:.1f} {86+math.sin(a)*11.6:.1f}" stroke="#4a3418" stroke-width="0.4"/>')
    b.append('<g transform="rotate(28 96 86)"><path d="M96 75 L98.2 86 L93.8 86 Z" fill="#b02a2a"/><path d="M96 97 L98.2 86 L93.8 86 Z" fill="#2a3a5a"/></g>')
    b.append('<circle cx="96" cy="86" r="1.3" fill="#c49440" stroke="#3e2608" stroke-width="0.3"/>')
    b.append(blob(90, 80, 5, 3, "#ffffff", 0.35, "b2"))
    return icon("Map", "".join(sc), "".join(b), defs, accent="#f0d0a0")


# =========================== LINKSHELL ===========================
def linkshell():
    defs = (lg("beachSky", [(0, "#0a1230"), (0.6, "#24346a"), (1, "#5a6a9a")])
            + lg("sea", [(0, "#2a4a7a"), (1, "#0e1a3a")])
            + lg("sand", [(0, "#6a6070"), (1, "#3a3040")])
            + lg("shellG", [(0, "#fff4f0"), (0.6, "#f0c8c8"), (1, "#c8889a")]))
    sc = ['<rect width="128" height="60" fill="url(#beachSky)"/>', stars(81, 40, (6, 6, 122, 50)),
          blob(100, 22, 9, 9, "#f4f6ff", 0.95, "b1"), blob(100, 22, 22, 22, "#c8d4ff", 0.35, "b5"),
          '<rect x="0" y="56" width="128" height="24" fill="url(#sea)"/>',
          blob(100, 62, 6, 1.2, "#dce4ff", 0.6, "b1"), blob(98, 68, 10, 1.2, "#dce4ff", 0.35, "b1"),
          '<g filter="url(#organic)"><path d="M0 76 C20 72 40 78 64 74 C88 70 104 76 128 72 L128 128 L0 128 Z" fill="url(#sand)"/>'
          '<path d="M0 76 C20 72 40 78 64 74 C88 70 104 76 128 72 L128 79 C104 82 88 76 64 80 C40 84 20 78 0 82 Z" fill="#e8f0ff" fill-opacity="0.7"/></g>',
          '<g filter="url(#mottle)"><rect x="0" y="80" width="128" height="48" fill="#5a5060" fill-opacity="0.5"/></g>',
          blob(64, 104, 36, 4, "#c8d4ff", 0.2, "b2")]
    b = []
    hinge = (64, 100)
    fan = []
    n, R = 11, 34
    for i in range(n):
        a0 = math.radians(-162 + i * (144 / n)); a1 = math.radians(-162 + (i + 1) * (144 / n)); am = (a0 + a1) / 2
        p0 = (hinge[0] + R * math.cos(a0), hinge[1] + R * math.sin(a0) * 0.95)
        p1 = (hinge[0] + R * math.cos(a1), hinge[1] + R * math.sin(a1) * 0.95)
        pm = (hinge[0] + (R + 3) * math.cos(am), hinge[1] + (R + 3) * math.sin(am) * 0.95)
        d = f"M{hinge[0]} {hinge[1]} L{p0[0]:.1f} {p0[1]:.1f} Q{pm[0]:.1f} {pm[1]:.1f} {p1[0]:.1f} {p1[1]:.1f} Z"
        fan.append(form(d, "url(#shellG)", "#8a5468", hi="#ffffff", hl=(pm[0] * 0.6 + 64 * 0.4, pm[1] * 0.6 + 100 * 0.4, 1.6, 5, math.degrees(am) + 90),
                        rim="#c8d4ff", rim_op=0.5, off=(0.6, 1.2), ao=0.25, outline="#5a3446", ow=0.4, blur="b1"))
    full = "M30 100 C30 70 98 70 98 100 Z"
    b.append(cast_shadow(full, 3, 3, 0.55))
    b.append("".join(fan))
    for k in (0.4, 0.65, 0.88):
        b.append(f'<path d="M{64-R*k:.1f} {100-R*k*0.3:.1f} Q64 {100-R*k*1.3:.1f} {64+R*k:.1f} {100-R*k*0.3:.1f}" fill="none" stroke="#8a5468" stroke-opacity="0.35" stroke-width="0.5"/>')
    b.append(form("M52 100 L64 94 L76 100 L72 106 L56 106 Z", "#f0c8c8", "#7a4458", hi="#fff", hl=(60, 98, 4, 1.2), ow=0.4, ao=0.3, outline="#5a3446"))
    # pearl
    b.append(blob(64, 84, 16, 16, "#e8dcff", 0.55, "b5"))
    b.append('<circle cx="65.5" cy="88" r="9" fill="#000" fill-opacity="0.45" filter="url(#cast)"/>')
    pearl = "M55 84 a9 9 0 1 0 18 0 a9 9 0 1 0 -18 0 Z"
    b.append(form(pearl, "#e8e2f4", "#6a5a8a", hi="#ffffff", hl=(60.5, 80, 3.4, 2.4), rim="#ffd8e8", rim_op=0.8, off=(1.8, 2.4), ao=0.35, outline="#3a2e52", ow=0.45, blur="b2"))
    b.append(sparkle(59, 78.5, 3.2, "#ffffff"))
    # starfish + bubbles
    st = []
    for i in range(5):
        a = math.radians(-90 + i * 72); a2 = math.radians(-90 + i * 72 + 36)
        st.append(f"{100+math.cos(a)*7:.1f},{108+math.sin(a)*7:.1f} {100+math.cos(a2)*2.8:.1f},{108+math.sin(a2)*2.8:.1f}")
    b.append(form("M" + " L".join(p.replace(",", " ") for p in " ".join(st).split(" ")) + " Z", "#e8805a", "#6a2414", hi="#ffd0a8", hl=(98, 105, 2, 1.4), tex="mottle", ow=0.4, ao=0.3, off=(0.6, 0.8), blur="b06"))
    b.append(motes(91, 8, (22, 90, 50, 118), ["#e8f0ff"], (0.5, 1.0), glow=False))
    return icon("Linkshell", "".join(sc), "".join(b), defs, accent="#d8e0ff")


ICONS = [("helm_fishing", fishing), ("helm_mining", mining), ("helm_logging", logging), ("helm_harvesting", harvesting), ("helm_excavation", excavation), ("misc_attack", attack), ("ws_sword", ws_sword), ("sp_fire", fire), ("sp_cure", cure), ("misc_items", items), ("misc_map", mapi), ("misc_linkshell", linkshell)]

if __name__ == "__main__":
    os.makedirs(OUT, exist_ok=True)
    want = sys.argv[1:]
    for name, fn in ICONS:
        if want and name not in want:
            continue
        reset()
        open(os.path.join(OUT, name + ".svg"), "w").write(fn())
    print("ok")
