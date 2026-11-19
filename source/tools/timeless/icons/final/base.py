"""Shared scene + layout helpers for the full Timeless icon set (built on paint.py, the approved pilots' toolkit)."""
import math, random, os, sys
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
from paint import *  # noqa
from paint import uid, reset, lg, rg, blob, form, cast_shadow, sparkle, motes, stars, icon, FILTERS  # noqa

OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "svg")

PAL = {  # sky top/mid/bottom, far ridge, near ridge, ground, mist, accent, light
    "fire":    dict(sky=("#3a120c", "#1c0806", "#060201"), far="#5a2018", near="#2a0e0a", ground="#140806", mist="#c85a3a", acc="#ff9a4a", lite="#ffd8a0"),
    "ice":     dict(sky=("#123e6e", "#071c3a", "#020610"), far="#1a3a5e", near="#0e2a48", ground="#0a1e34", mist="#7ac8e8", acc="#9fe8ff", lite="#e8fbff"),
    "wind":    dict(sky=("#1e4a3a", "#0c2a20", "#030a08"), far="#2a5a44", near="#123428", ground="#0a1e16", mist="#7ae0b0", acc="#8ff0c0", lite="#e8fff2"),
    "earth":   dict(sky=("#4a3418", "#22180c", "#080503"), far="#5a4224", near="#2e2010", ground="#18100a", mist="#d0a060", acc="#f0b860", lite="#ffe6b0"),
    "thunder": dict(sky=("#2e1e5a", "#140c30", "#04020c"), far="#3a2a6a", near="#1c143a", ground="#0e0a20", mist="#a890ff", acc="#c8b0ff", lite="#f4f0ff"),
    "water":   dict(sky=("#0e3a5a", "#061e34", "#01070e"), far="#14425e", near="#0a2a40", ground="#061826", mist="#5ab0e0", acc="#6fd0ff", lite="#e0f6ff"),
    "light":   dict(sky=("#5a4a2a", "#2a2214", "#0a0805"), far="#6a5a38", near="#342a18", ground="#1a140a", mist="#ffe0a0", acc="#ffe6a0", lite="#fffaf0"),
    "dark":    dict(sky=("#2a1238", "#140820", "#050208"), far="#341a44", near="#1c0e26", ground="#0e0614", mist="#a060c0", acc="#c080ff", lite="#f0e0ff"),
    "heal":    dict(sky=("#1a4a3a", "#0c2a22", "#030a08"), far="#245a46", near="#123a2c", ground="#0a2018", mist="#9ff0c8", acc="#b8ffd8", lite="#f2fff8"),
    "dusk":    dict(sky=("#4e6294", "#2a3458", "#0c1020"), far="#3a4670", near="#1e2440", ground="#121628", mist="#9a8cb4", acc="#cfe0ff", lite="#f4f8ff"),
    "night":   dict(sky=("#24345c", "#121a34", "#04060e"), far="#2a3658", near="#161e36", ground="#0c1022", mist="#8fb2e6", acc="#8fb2e6", lite="#e6ecf5"),
    "gold":    dict(sky=("#4a3a20", "#241a0c", "#080603"), far="#5a4828", near="#2e2412", ground="#18120a", mist="#e8c070", acc="#ffd27a", lite="#fff2d0"),
    "blood":   dict(sky=("#4a1418", "#22080a", "#080203"), far="#5a1c20", near="#2a0c0e", ground="#160608", mist="#e06060", acc="#ff7070", lite="#ffe0e0"),
    "poison":  dict(sky=("#2e3a14", "#161c08", "#050702"), far="#3a4a1a", near="#1c240c", ground="#0e1206", mist="#a0d040", acc="#c0f060", lite="#f4ffd8"),
    "azure":   dict(sky=("#103a5a", "#081c30", "#02060c"), far="#164866", near="#0a2a40", ground="#061826", mist="#40c0e0", acc="#40e0ff", lite="#d8faff"),
}


def ridge(seed, y, amp, step, jag=0.5):
    r = random.Random(seed)
    pts, x = [], -4
    while x <= 132:
        pts.append((x, y + r.uniform(-amp, amp) * (1 if r.random() > jag else 0.4)))
        x += r.uniform(step * 0.6, step * 1.4)
    return pts


def scene(pal, seed=1, n_stars=34, hills=True):
    p = PAL[pal]
    gid = uid("sky")
    defs = rg(gid, [(0, p["sky"][0]), (0.55, p["sky"][1]), (1, p["sky"][2])], cy="40%", r="75%")
    s = [defs.join(['<defs>', '</defs>']), f'<rect width="128" height="128" fill="url(#{gid})"/>',
         stars(seed, n_stars, (6, 6, 122, 70), p["lite"])]
    if hills:
        far = ridge(seed * 7 + 1, 74, 8, 14)
        near = ridge(seed * 7 + 2, 88, 5, 18)
        fd = "M" + " L".join(f"{x:.1f} {y:.1f}" for x, y in far) + " L132 128 L-4 128 Z"
        nd = "M" + " L".join(f"{x:.1f} {y:.1f}" for x, y in near) + " L132 128 L-4 128 Z"
        s += [f'<g filter="url(#organic)"><path d="{fd}" fill="{p["far"]}" fill-opacity="0.85"/></g>',
              f'<path d="M' + " L".join(f"{x:.1f} {y:.1f}" for x, y in far) + f'" fill="none" stroke="{p["acc"]}" stroke-opacity="0.18" stroke-width="1" filter="url(#b06)"/>',
              blob(64, 82, 70, 5, p["mist"], 0.18, "b3"),
              f'<g filter="url(#rock)"><path d="{nd}" fill="{p["near"]}"/></g>',
              f'<path d="M' + " L".join(f"{x:.1f} {y:.1f}" for x, y in near) + f'" fill="none" stroke="{p["acc"]}" stroke-opacity="0.22" stroke-width="0.9" filter="url(#b06)"/>',
              '<rect x="0" y="0" width="128" height="106" fill="#000" fill-opacity="0.28"/>',
              f'<g filter="url(#organic)"><path d="M-4 104 C30 100 96 102 132 100 L132 132 L-4 132 Z" fill="{p["ground"]}"/></g>',
              f'<g filter="url(#mottle)"><path d="M-4 104 C30 100 96 102 132 100 L132 132 L-4 132 Z" fill="{p["near"]}" fill-opacity="0.45"/></g>']
    return "".join(s)


def halo(cx, cy, r, col, op=0.5):
    g = uid("halo")
    return (f'<defs>{rg(g, [(0, col, op), (0.5, col, op * 0.35), (1, col, 0)])}</defs>'
            f'<circle cx="{cx}" cy="{cy}" r="{r}" fill="url(#{g})"/>')


def ring(pal, cy=104, rx=38, ry=8, op=1.0):
    p = PAL[pal]
    g = uid("rg")
    return (f'<defs>{rg(g, [(0, p["acc"], 0.35 * op), (0.6, p["acc"], 0.12 * op), (1, p["acc"], 0)])}</defs>'
            f'<ellipse cx="64" cy="{cy}" rx="{rx+8}" ry="{ry+4}" fill="url(#{g})"/>'
            f'<ellipse cx="64" cy="{cy}" rx="{rx}" ry="{ry}" fill="none" stroke="{p["acc"]}" stroke-opacity="{0.55*op:.2f}" stroke-width="2.2" filter="url(#b1)"/>'
            f'<ellipse cx="64" cy="{cy}" rx="{rx}" ry="{ry}" fill="none" stroke="{p["lite"]}" stroke-opacity="{0.7*op:.2f}" stroke-width="0.5"/>'
            f'<ellipse cx="64" cy="{cy}" rx="{rx*0.82:.1f}" ry="{ry*0.8:.1f}" fill="none" stroke="{p["lite"]}" stroke-opacity="{0.35*op:.2f}" stroke-width="0.4"/>')


def sparks(pal, seed, n, box=(10, 10, 118, 100)):
    p = PAL[pal]
    return motes(seed, n, box, [p["lite"], p["acc"]], (0.4, 1.1))


def tiered(core, tier, ga, pal, seed=1, cy=62):
    """One core design scaled by tier; -ga = three of the tier visual."""
    out = []
    if not ga:
        sc = {1: 0.66, 2: 0.8, 3: 0.94, 4: 1.08, 5: 1.16}[tier]
        out.append(halo(64, cy, 30 + tier * 9, PAL[pal]["acc"], 0.25 + 0.08 * tier))
        out.append(core(64, cy, sc, tier, seed))
    else:
        sc = {1: 0.5, 2: 0.58, 3: 0.66}[tier]
        out.append(halo(64, cy, 52, PAL[pal]["acc"], 0.3 + 0.08 * tier))
        out.append(core(33, cy + 10, sc * 0.86, tier, seed + 1))
        out.append(core(95, cy + 10, sc * 0.86, tier, seed + 2))
        out.append(core(64, cy - 6, sc, tier, seed))
    out.append(sparks(pal, seed * 13 + tier + (7 if ga else 0), 8 + tier * 4 + (8 if ga else 0)))
    return "".join(out)


def save(name, svg):
    os.makedirs(OUT, exist_ok=True)
    open(os.path.join(OUT, name + ".svg"), "w").write(svg)


def make(name, title, pal, body, defs="", seed=1, hills=True, accent=None):
    reset()
    sc = scene(pal, seed, hills=hills)
    save(name, icon(title, sc, body() if callable(body) else body, defs, accent=accent or PAL[pal]["lite"]))
