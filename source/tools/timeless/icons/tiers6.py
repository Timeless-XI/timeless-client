#!/usr/bin/env python3
"""Elemental tier system: Fire I-IV / Firaga I-III, Blizzard I-IV / Blizzaga I-III.
One core design per element; tiers grow in size and intensity; -ga = three of the tier visual."""
import math, os, sys, random
sys.path.insert(0, os.path.dirname(__file__))
from paint import *
from lib import poly

OUT = os.path.join(os.path.dirname(__file__), "pilot6")

# ------------------------------------------------------------------ FIRE
FIRE_T = {1: dict(h=40, w=24, n=3, emb=8, glow=0.35),
          2: dict(h=56, w=32, n=4, emb=14, glow=0.5),
          3: dict(h=72, w=42, n=5, emb=22, glow=0.65),
          4: dict(h=90, w=58, n=7, emb=34, glow=0.85)}


def flame_path(cx, base, h, w, n, seed):
    rnd = random.Random(seed)
    xs = [cx - w * 0.42 + i * (w * 0.84) / (n - 1) for i in range(n)] if n > 1 else [cx]
    tips = []
    for i, x in enumerate(xs):
        c = 1 - abs((i - (n - 1) / 2) / ((n - 1) / 2 + 0.001)) if n > 1 else 1
        hh = h * (0.5 + 0.5 * c) * rnd.uniform(0.86, 1.0)
        lean = rnd.uniform(-0.12, 0.12) * w + (x - cx) * 0.25
        tips.append((x + lean, base - hh))
    tips[n // 2] = (cx + rnd.uniform(-0.05, 0.05) * w, base - h)
    d = f"M{cx - w/2:.1f} {base - h*0.12:.1f} "
    prev = (cx - w / 2, base - h * 0.12)
    for i, (tx, ty) in enumerate(tips):
        d += f"Q{(prev[0]+tx)/2 - w*0.06:.1f} {(prev[1]+ty)/2 + h*0.04:.1f} {tx:.1f} {ty:.1f} "
        if i < n - 1:
            nx = (tx + tips[i + 1][0]) / 2
            vy = base - h * rnd.uniform(0.28, 0.48) * (0.6 + 0.4 * (1 - abs(i + 0.5 - (n - 1) / 2) / max(1, n / 2)))
            d += f"Q{tx + w*0.05:.1f} {ty + h*0.22:.1f} {nx:.1f} {vy:.1f} "
            prev = (nx, vy)
    d += (f"Q{cx + w*0.52:.1f} {base - h*0.35:.1f} {cx + w/2:.1f} {base - h*0.12:.1f} "
          f"C{cx + w*0.55:.1f} {base + w*0.12:.1f} {cx + w*0.2:.1f} {base + w*0.22:.1f} {cx:.1f} {base + w*0.22:.1f} "
          f"C{cx - w*0.2:.1f} {base + w*0.22:.1f} {cx - w*0.55:.1f} {base + w*0.12:.1f} {cx - w/2:.1f} {base - h*0.12:.1f} Z")
    return d


def flame(cx, base, tier, sc=1.0, seed=1):
    t = FIRE_T[tier]
    h, w, n = t["h"] * sc, t["w"] * sc, t["n"]
    d = flame_path(cx, base, h, w, n, seed)
    gr = max(h, w) * 0.95
    out = [f'<circle cx="{cx}" cy="{base - h*0.45:.1f}" r="{gr:.1f}" fill="url(#fGlow)" opacity="{min(1, t["glow"] + 0.25):.2f}"/>',
           f'<g filter="url(#flameD)">',
           f'<path d="{d}" fill="url(#fOut)"/>']
    for k, f in ((0.78, "url(#fMid)"), (0.56, "url(#fIn)"), (0.32, "url(#fCore)")):
        out.append(f'<g transform="translate({cx} {base + w*0.15:.1f}) scale({k}) translate({-cx} {-(base + w*0.15):.1f})"><path d="{flame_path(cx, base, h, w, max(2, n-1), seed+int(k*10))}" fill="{f}"/></g>')
    out.append('</g>')
    out.append(blob(cx, base + w * 0.02, w * 0.09, w * 0.035, "#9ac8ff", 0.35, "b1"))
    out.append(f'<path d="{d}" fill="none" stroke="#ffd8a0" stroke-opacity="0.2" stroke-width="0.6" filter="url(#b06)"/>')
    return "".join(out)


FIRE_DEFS = (lg("fOut", [(0, "#ff7a3a", 0.0), (0.18, "#e8461c"), (0.7, "#c82a12"), (1, "#7a1408")])
             + lg("fMid", [(0, "#ffb050", 0.0), (0.22, "#ff8a2e"), (1, "#e85a1a")])
             + lg("fIn", [(0, "#fff0a0", 0.0), (0.28, "#ffd860"), (1, "#ffa032")])
             + lg("fCore", [(0, "#ffffff", 0.0), (0.35, "#fffbea"), (1, "#ffe6a0")])
             + rg("fGlow", [(0, "#ffb060", 0.75), (0.4, "#ff6a20", 0.35), (1, "#ff4a10", 0)])
             + rg("fBg", [(0, "#4a1408"), (0.55, "#1e0806"), (1, "#080303")], cy="70%"))


def fire_scene(tier, ga=False):
    inten = tier
    rnd = random.Random(4)
    sc = ['<rect width="128" height="128" fill="url(#fBg)"/>',
                    # far range: hazy, warm
          '<g filter="url(#organic)"><path d="M0 70 L10 62 L18 66 L28 54 L36 60 L46 50 L54 58 L62 52 L70 58 L80 50 L90 58 L100 52 L110 60 L120 56 L128 60 L128 92 L0 92 Z" fill="#5a2018" fill-opacity="0.6"/></g>',
          blob(40, 56, 40, 6, "#ff7a40", 0.18, "b3"),
          # volcano (right) with glowing crater and smoke plume
          blob(100, 22, 12, 16, "#3a2a28", 0.55, "b5"), blob(94, 12, 16, 10, "#2e2422", 0.45, "b5"), blob(106, 34, 9, 8, "#4a3430", 0.5, "b3"),
          '<g filter="url(#rock)"><path d="M70 96 L88 52 L96 44 L106 44 L112 52 L128 80 L128 100 Z" fill="#4a1c12"/></g>',
          '<path d="M70 96 L88 52 L96 44 L101 46 L94 70 L86 96 Z" fill="#8a3a22" fill-opacity="0.7" filter="url(#b06)"/>',
          '<path d="M88 52 L96 44 L106 44 L112 52" fill="none" stroke="#ff9a50" stroke-opacity="0.6" stroke-width="1.4" filter="url(#b06)"/>',
          blob(101, 45, 7, 2.6, "#ffb060", 0.9, "b1"),
          '<path d="M100 46 C99 52 102 58 98 66 C95 72 97 80 92 88 M104 46 C106 54 110 60 112 70 C114 78 118 84 120 92" fill="none" stroke="#ff6a20" stroke-opacity="0.7" stroke-width="2.4" filter="url(#b1)"/>',
          '<path d="M100 46 C99 52 102 58 98 66 C95 72 97 80 92 88 M104 46 C106 54 110 60 112 70 C114 78 118 84 120 92" fill="none" stroke="#ffd080" stroke-opacity="0.85" stroke-width="0.7"/>',
          # left ridge
          '<g filter="url(#rock)"><path d="M0 78 L8 70 L18 74 L26 64 L34 70 L44 66 L52 76 L58 96 L0 98 Z" fill="#3a140e"/></g>',
          '<path d="M0 78 L8 70 L12 80 L4 92 Z M18 74 L26 64 L30 78 L22 94 Z M34 70 L44 66 L46 80 L40 96 Z" fill="#6a2a18" fill-opacity="0.75" filter="url(#b06)"/>',
          '<path d="M0 78 L8 70 L18 74 L26 64 L34 70 L44 66 L52 76" fill="none" stroke="#ff8a4a" stroke-opacity="0.35" stroke-width="1.2" filter="url(#b06)"/>',
          # second far range + mist band between ranges
          '<g filter="url(#organic)"><path d="M0 76 L12 68 L22 72 L34 62 L42 68 L54 60 L66 66 L74 60 L86 70 L128 72 L128 94 L0 94 Z" fill="#46180f" fill-opacity="0.75"/></g>',
          blob(54, 74, 60, 4, "#c85a3a", 0.25, "b3"),
          # volcano strata, crags and secondary peak
          '<g filter="url(#rock)"><path d="M112 52 L118 46 L124 52 L128 58 L128 80 Z" fill="#3a160e"/></g>',
          '<path d="M86 60 C94 58 104 60 114 64 M82 70 C92 68 106 70 118 76 M78 80 C90 78 108 82 124 88" fill="none" stroke="#1a0806" stroke-opacity="0.65" stroke-width="0.7"/>',
          '<path d="M86 60.8 C94 58.8 104 60.8 114 64.8 M82 70.8 C92 68.8 106 70.8 118 76.8" fill="none" stroke="#ff9a60" stroke-opacity="0.18" stroke-width="0.5"/>',
          "".join(f'<path d="M{x} {y} l2 -2.6 l1.4 1.4 l2 -3 l2.2 4.2" fill="none" stroke="#1a0806" stroke-opacity="0.7" stroke-width="0.6"/><path d="M{x} {y-0.6} l2 -2.6 l1.4 1.4 l2 -3" fill="none" stroke="#ff9a60" stroke-opacity="0.3" stroke-width="0.4"/>' for x, y in ((88, 64), (98, 72), (106, 68), (84, 77), (112, 81))),
          blob(94, 94, 10, 2.4, "#ff8a30", 0.7, "b1"), '<ellipse cx="94" cy="94" rx="6" ry="1.2" fill="#ffd27a" fill-opacity="0.8"/>',
          # smoke plume detail: lit underside
          blob(98, 30, 7, 5, "#8a4a3a", 0.35, "b2"), blob(104, 20, 6, 4, "#6a3a30", 0.3, "b2"),
          # left ridge crags + strata
          '<path d="M6 82 C14 80 22 82 30 84 M10 90 C20 88 34 90 44 92" fill="none" stroke="#1a0806" stroke-opacity="0.6" stroke-width="0.6"/>',
          "".join(f'<path d="M{x} {y} l1.8 -2.4 l1.2 1.2 l1.8 -2.6 l2 3.8" fill="none" stroke="#1a0806" stroke-opacity="0.65" stroke-width="0.55"/>' for x, y in ((10, 80), (26, 75), (38, 78))),
          blob(64, 92, 64, 10, "#120604", 0.75, "b5"),
          # near rocks with rim light
          '<g filter="url(#rock)"><path d="M0 86 L8 80 L18 84 L24 94 L20 104 L0 106 Z" fill="#24100c"/><path d="M128 84 L118 78 L108 86 L104 98 L110 106 L128 106 Z" fill="#24100c"/></g>',
          '<path d="M0 86 L8 80 L18 84 L24 94 M128 84 L118 78 L108 86 L104 98" fill="none" stroke="#ff8a4a" stroke-opacity="0.45" stroke-width="1.2" filter="url(#b06)"/>',
          # extra rock detail: crevices, scree, far-range strata
          '<path d="M4 68 C14 66 24 68 34 66 M44 62 C54 61 64 63 72 62 M18 72 L20 78 M48 64 L46 70 M60 60 L62 66" fill="none" stroke="#1a0806" stroke-opacity="0.55" stroke-width="0.5"/>',
          '<path d="M92 58 L90 66 L93 74 M104 62 L106 72 L103 82 M116 70 L114 80" fill="none" stroke="#120604" stroke-opacity="0.7" stroke-width="0.7"/>',
          "".join(f'<ellipse cx="{x}" cy="{y}" rx="{r}" ry="{r*0.6:.1f}" fill="#1e0a06"/><path d="M{x-r} {y} A{r} {r*0.6:.1f} 0 0 1 {x+r} {y}" fill="none" stroke="#ff8a4a" stroke-opacity="0.25" stroke-width="0.4"/>' for x, y, r in ((22, 96, 2.2), (30, 98, 1.4), (100, 100, 2.4), (108, 98, 1.5), (16, 99, 1.2))),
          # push the whole landscape back into shadow
          '<rect x="0" y="0" width="128" height="106" fill="#060201" fill-opacity="0.32"/>',
          '<g filter="url(#organic)"><path d="M0 104 C30 100 96 102 128 100 L128 128 L0 128 Z" fill="#140806"/></g>',
          '<path d="M6 112 C18 110 28 116 42 113 M86 116 C98 112 108 118 122 114 M52 122 C60 120 70 124 78 121" fill="none" stroke="#ff7a30" stroke-opacity="0.55" stroke-width="0.9" filter="url(#b06)"/>',
          '<path d="M6 112 C18 110 28 116 42 113 M86 116 C98 112 108 118 122 114" fill="none" stroke="#ffd080" stroke-opacity="0.6" stroke-width="0.3"/>']
    sc = [x for x in sc if x]
    ring = ['<ellipse cx="64" cy="104" rx="44" ry="11" fill="#ff7a30" fill-opacity="0.28" filter="url(#b5)"/>',
            '<ellipse cx="64" cy="104" rx="38" ry="8" fill="none" stroke="#ff7a30" stroke-opacity="0.75" stroke-width="5" filter="url(#b3)"/>',
            '<ellipse cx="64" cy="104" rx="38" ry="8" fill="none" stroke="#ffb060" stroke-opacity="0.8" stroke-width="2.4" filter="url(#b1)"/>',
            '<ellipse cx="64" cy="104" rx="31" ry="6.4" fill="none" stroke="#ffb060" stroke-opacity="0.45" stroke-width="1.6" filter="url(#b1)"/>',
            '<ellipse cx="64" cy="104" rx="38" ry="8" fill="none" stroke="#ffd8a0" stroke-opacity="0.85" stroke-width="0.5"/>',
            '<ellipse cx="64" cy="104" rx="31" ry="6.4" fill="none" stroke="#ffd8a0" stroke-opacity="0.5" stroke-width="0.4"/>']
    for i in range(20):
        a = math.radians(i * 18)
        ring.append(f'<path d="M{64+31*math.cos(a):.1f} {104+6.4*math.sin(a):.1f} L{64+38*math.cos(a):.1f} {104+8*math.sin(a):.1f}" stroke="#ffd8a0" stroke-opacity="0.55" stroke-width="0.45"/>')
    return "".join(sc), "".join(ring)


def fire_icon(tier, ga=False):
    reset()
    sc, ring = fire_scene(tier, ga)
    b = [ring]
    t = FIRE_T[tier]
    if tier >= 4 and not ga:
        for i in range(16):
            a = math.radians(-90 + (i - 7.5) * 12)
            b.append(f'<path d="M64 70 L{64+math.cos(a)*62:.1f} {70+math.sin(a)*62:.1f}" stroke="#ffb060" stroke-opacity="0.18" stroke-width="3" filter="url(#b2)"/>')
        b.append(blob(64, 60, 50, 46, "#ffb060", 0.3, "b8"))
    if not ga:
        if tier >= 3:
            b.append(flame(64 - t["w"] * 0.62, 104, max(1, tier - 2), 0.8, seed=tier * 7 + 1))
            b.append(flame(64 + t["w"] * 0.62, 104, max(1, tier - 2), 0.8, seed=tier * 7 + 2))
        b.append(flame(64, 104, tier, 1.0, seed=tier * 7))
    else:
        s = 0.82
        b.append(flame(32, 102, tier, s * 0.86, seed=tier * 11 + 1))
        b.append(flame(96, 102, tier, s * 0.86, seed=tier * 11 + 2))
        b.append(flame(64, 106, tier, s, seed=tier * 11))
    rnd_n = t["emb"] + (10 if ga else 0)
    b.append(motes(tier * 13 + (5 if ga else 0), rnd_n, (16, 10, 112, 100), ["#ffd27a", "#ff9a4a", "#fff0c0"], (0.4, 1.1 + tier * 0.1)))
    name = (["", "Firaga", "Firaga II", "Firaga III"][tier] if ga else ["", "Fire", "Fire II", "Fire III", "Fire IV"][tier])
    return icon(f"Spell - {name}", sc, "".join(b), FIRE_DEFS, accent="#f0b080")


# ------------------------------------------------------------------ ICE (frost vortex)
ICE_T = {1: dict(R=22, rib=3, mote=14, glow=0.5),
         2: dict(R=27, rib=4, mote=22, glow=0.65),
         3: dict(R=33, rib=5, mote=32, glow=0.8),
         4: dict(R=41, rib=7, mote=48, glow=1.0)}

ICE_DEFS = (rg("iBg", [(0, "#123e6e"), (0.55, "#071c3a"), (1, "#020610")], cy="58%")
            + rg("coreI", [(0, "#ffffff", 1), (0.35, "#e0faff", 0.95), (0.7, "#7ae0ff", 0.55), (1, "#2aa8f0", 0)])
            + lg("ribG", [(0, "#ffffff", 0), (0.35, "#bff4ff", 0.75), (0.8, "#ffffff", 1), (1, "#ffffff", 1)], 0, 0, 1, 0)
            + lg("snowG", [(0, "#3a6e98"), (0.3, "#1a4268"), (1, "#081a30")]))


def snowflake(cx, cy, r, op=1.0, sw=1.0, glow=True):
    arms = []
    for k in range(6):
        a = k * 60
        arms.append(f'<g transform="rotate({a} {cx} {cy})"><path d="M{cx} {cy} L{cx} {cy-r:.1f} M{cx} {cy-r*0.55:.1f} l{-r*0.22:.1f} {-r*0.22:.1f} M{cx} {cy-r*0.55:.1f} l{r*0.22:.1f} {-r*0.22:.1f} '
                    f'M{cx} {cy-r*0.8:.1f} l{-r*0.13:.1f} {-r*0.13:.1f} M{cx} {cy-r*0.8:.1f} l{r*0.13:.1f} {-r*0.13:.1f}" stroke-linecap="round"/>'
                    f'<path d="M{cx} {cy-r*0.22:.1f} l{r*0.1:.1f} {-r*0.1:.1f} l{-r*0.1:.1f} {-r*0.1:.1f} l{-r*0.1:.1f} {r*0.1:.1f} Z"/></g>')
    body = "".join(arms)
    g1 = f'<g fill="none" stroke="#5fd0ff" stroke-width="{2.6*sw:.2f}" filter="url(#b2)">{body}</g>' if glow else ""
    return f'<g opacity="{op}">{g1}<g fill="#f4fdff" stroke="#f4fdff" stroke-width="{0.8*sw:.2f}">{body}</g></g>'


def ribbon(cx, cy, R, a0, sweep, width, op=1.0):
    """Tapered crescent of frost wind spiralling around (cx, cy)."""
    pts_o, pts_i = [], []
    n = 18
    for k in range(n + 1):
        t = k / n
        a = math.radians(a0 + sweep * t)
        r = R * (0.55 + 0.6 * t)
        w = width * math.sin(math.pi * t) ** 0.8
        pts_o.append((cx + math.cos(a) * (r + w / 2), cy + math.sin(a) * (r + w / 2) * 0.82))
        pts_i.append((cx + math.cos(a) * (r - w / 2), cy + math.sin(a) * (r - w / 2) * 0.82))
    d = "M" + " L".join(f"{x:.1f} {y:.1f}" for x, y in pts_o + pts_i[::-1]) + " Z"
    edge = "M" + " L".join(f"{x:.1f} {y:.1f}" for x, y in pts_o)
    return (f'<path d="{d}" fill="#5fd0ff" fill-opacity="{0.5*op:.2f}" filter="url(#b2)"/>'
            f'<path d="{d}" fill="#d8f8ff" fill-opacity="{0.55*op:.2f}" filter="url(#b06)"/>'
            f'<path d="{edge}" fill="none" stroke="#ffffff" stroke-opacity="{0.9*op:.2f}" stroke-width="0.6" stroke-linecap="round"/>')


def vortex(cx, cy, tier, sc=1.0, seed=1):
    t = ICE_T[tier]
    R = t["R"] * sc
    rnd = random.Random(seed)
    out = [blob(cx, cy, R * 1.6, R * 1.3, "#3ab8ff", 0.35 * t["glow"], "b8"),
           blob(cx, cy, R * 0.9, R * 0.8, "#9fe8ff", 0.45 * t["glow"], "b5")]
    # back ribbons (dimmer), front ribbons (brighter): a tilted vortex
    for k in range(t["rib"]):
        a0 = k * 360 / t["rib"] + rnd.uniform(-12, 12)
        out.append(ribbon(cx, cy, R, a0, 150, R * 0.32, 0.55 if k % 2 else 0.9))
    # fine inner spiral strands
    for k in range(t["rib"]):
        a0 = k * 360 / t["rib"] + 30
        pts_ = []
        for q in range(16):
            tt = q / 15
            a = math.radians(a0 + 200 * tt)
            r = R * (0.3 + 0.75 * tt)
            pts_.append((cx + math.cos(a) * r, cy + math.sin(a) * r * 0.82))
        out.append('<path d="M' + " L".join(f"{x:.1f} {y:.1f}" for x, y in pts_) + '" fill="none" stroke="#e8fbff" stroke-opacity="0.55" stroke-width="0.35" stroke-linecap="round"/>')
    # frost fringe ticks along the outer swirl
    for q in range(10 + tier * 4):
        a = rnd.uniform(0, 2 * math.pi); r = R * rnd.uniform(1.0, 1.2)
        x, y = cx + math.cos(a) * r, cy + math.sin(a) * r * 0.82
        out.append(f'<path d="M{x:.1f} {y:.1f} l{math.cos(a)*1.6:.1f} {math.sin(a)*1.3:.1f}" stroke="#ffffff" stroke-opacity="0.7" stroke-width="0.35"/>')
    # core: glowing orb with a snowflake inside
    out.append(f'<circle cx="{cx}" cy="{cy}" r="{R*0.62:.1f}" fill="url(#coreI)"/>')
    out.append(snowflake(cx, cy, R * 0.42, 1.0, 0.55 + 0.12 * tier))
    out.append(f'<circle cx="{cx}" cy="{cy}" r="{R*0.12:.1f}" fill="#ffffff"/>')
    out.append(f'<circle cx="{cx}" cy="{cy}" r="{R*0.66:.1f}" fill="none" stroke="#bff4ff" stroke-opacity="0.7" stroke-width="0.5"/>')
    out.append(f'<circle cx="{cx}" cy="{cy}" r="{R*0.72:.1f}" fill="none" stroke="#5fd0ff" stroke-opacity="0.35" stroke-width="1.6" filter="url(#b1)"/>')
    for q in range(2 + tier):
        a = rnd.uniform(0, 2 * math.pi); r = R * rnd.uniform(0.9, 1.5)
        out.append(sparkle(cx + math.cos(a) * r, cy + math.sin(a) * r * 0.82, rnd.uniform(1.6, 2.8) * sc, "#f4fdff", 0.9))
    # frost flecks orbiting
    for _ in range(t["mote"]):
        a = rnd.uniform(0, 2 * math.pi); r = R * rnd.uniform(0.7, 1.45)
        x, y = cx + math.cos(a) * r, cy + math.sin(a) * r * 0.82
        k = rnd.uniform(0.6, 1.5) * sc
        if rnd.random() < 0.4:
            out.append(f'<path d="M{x:.1f} {y-k*1.6:.1f} L{x+k*0.5:.1f} {y:.1f} L{x:.1f} {y+k*1.6:.1f} L{x-k*0.5:.1f} {y:.1f} Z" fill="#e8fbff" fill-opacity="0.9" transform="rotate({rnd.uniform(0,180):.0f} {x:.1f} {y:.1f})"/>')
        else:
            out.append(f'<circle cx="{x:.1f}" cy="{y:.1f}" r="{k*1.8:.2f}" fill="#5fd0ff" fill-opacity="0.25"/><circle cx="{x:.1f}" cy="{y:.1f}" r="{k*0.7:.2f}" fill="#ffffff"/>')
    return "".join(out)


def ice_icon(tier, ga=False):
    reset()
    t = ICE_T[tier]
    sc = ['<rect width="128" height="128" fill="url(#iBg)"/>', stars(tier * 3 + 1, 34, (6, 6, 122, 70), "#e8fbff"),
          # snowy hills + pine silhouettes
          '<g filter="url(#organic)"><path d="M0 90 C20 82 40 88 64 84 C88 80 108 86 128 82 L128 128 L0 128 Z" fill="#0e2a48"/></g>',
          "".join(f'<path d="M{x} {y} l{-w} {w*2.4:.1f} l{w*2} 0 Z" fill="#081a30"/><path d="M{x} {y} l{-w*0.6:.1f} {w*1.2:.1f} l{w*1.2:.1f} 0 Z" fill="#cfe6f4" fill-opacity="0.55"/>'
                  for x, y, w in ((10, 76, 4), (18, 80, 3), (112, 74, 4.4), (120, 80, 3), (100, 80, 2.6))),
          '<g filter="url(#organic)"><path d="M0 104 C26 100 56 104 84 101 C104 99 116 102 128 100 L128 128 L0 128 Z" fill="url(#snowG)"/></g>',
          '<g filter="url(#mottle)"><rect x="0" y="100" width="128" height="28" fill="#4a7a9a" fill-opacity="0.3"/></g>',
          motes(tier * 17 + 2, 14, (4, 104, 124, 124), ["#e8fbff", "#9fe8ff"], (0.25, 0.6), glow=False),
          blob(64, 20, 70, 6, "#3ac0a8", 0.08 + 0.03 * tier, "b5")]
    b = []
    if not ga:
        cy = 64
        if tier >= 3:
            b.append(snowflake(64, cy, 34 + (tier - 3) * 14, 0.25 + (tier - 3) * 0.3, 1.2 + (tier - 3) * 0.5))
        if tier >= 4:
            for i in range(12):
                y = 12 + i * 8
                b.append(f'<path d="M{-10 + (i*23)%50} {y} l{70 + (i*13)%30} {12}" stroke="#e8fbff" stroke-opacity="0.2" stroke-width="1.4" filter="url(#b1)"/>')
        b.append(blob(64, 106, t["R"] * 1.3, 4.5, "#7ae0ff", 0.45 * t["glow"], "b3"))
        b.append(f'<ellipse cx="64" cy="106" rx="{t["R"]*0.9:.1f}" ry="2.6" fill="none" stroke="#bff4ff" stroke-opacity="0.5" stroke-width="0.6"/>')
        b.append(blob(64, 112, t["R"] * 0.5, 2, "#e8fbff", 0.35 * t["glow"], "b2"))
        b.append(vortex(64, cy, tier, 1.0, seed=tier))
    else:
        s = 0.66
        b.append(vortex(32, 70, tier, s * 0.86, seed=tier * 5 + 1))
        b.append(vortex(96, 70, tier, s * 0.86, seed=tier * 5 + 2))
        b.append(vortex(64, 56, tier, s, seed=tier * 5))
    rnd = random.Random(tier * 9 + (3 if ga else 0))
    for _ in range(10 + tier * 6):
        x, y, r = rnd.uniform(8, 120), rnd.uniform(8, 100), rnd.uniform(0.4, 1.0)
        b.append(f'<circle cx="{x:.1f}" cy="{y:.1f}" r="{r:.2f}" fill="#ffffff" fill-opacity="{rnd.uniform(0.5,0.9):.2f}"/>')
    name = (["", "Blizzaga", "Blizzaga II", "Blizzaga III"][tier] if ga else ["", "Blizzard", "Blizzard II", "Blizzard III", "Blizzard IV"][tier])
    return icon(f"Spell - {name}", "".join(sc), "".join(b), ICE_DEFS, accent="#bff4ff")


if __name__ == "__main__":
    os.makedirs(OUT, exist_ok=True)
    for t in (1, 2, 3, 4):
        open(os.path.join(OUT, f"sp_fire{t}.svg"), "w").write(fire_icon(t))
        open(os.path.join(OUT, f"sp_blizzard{t}.svg"), "w").write(ice_icon(t))
    for t in (1, 2, 3):
        open(os.path.join(OUT, f"sp_firaga{t}.svg"), "w").write(fire_icon(t, ga=True))
        open(os.path.join(OUT, f"sp_blizzaga{t}.svg"), "w").write(ice_icon(t, ga=True))
    print("ok")
