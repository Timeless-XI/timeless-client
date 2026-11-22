#!/usr/bin/env python3
"""Timeless icon library: themed Starlit plates (round 3a lessons) + material gradients."""
import math, random

E = "#090b12"  # hairline edge colour

# ---- colour themes: sky (centre, mid, edge), ring, centre glow, star tint ----
THEMES = {
    # combat / misc actions (the chosen 3a)
    "night":   dict(sky=("#18203a", "#0c1120", "#04050a"), ring="#8fb2e6", glow="#3d5f94", tint="#cfe0f5"),
    # menu / utility actions
    "slate":   dict(sky=("#132a30", "#09161b", "#030608"), ring="#86cfd0", glow="#2f6f73", tint="#cfeef0"),
    # weapon skills
    "ember":   dict(sky=("#3a1a14", "#1a0b0a", "#060303"), ring="#e8a46c", glow="#8a3a28", tint="#ffe2c4"),
    # job abilities
    "violet":  dict(sky=("#2a1f45", "#140f24", "#05040a"), ring="#b9a0ec", glow="#5b3f94", tint="#e6dcff"),
    # elements
    "fire":    dict(sky=("#40180f", "#1d0906", "#070302"), ring="#ff9a5a", glow="#a83a1a", tint="#ffd9bf"),
    "ice":     dict(sky=("#163444", "#0a1922", "#03070b"), ring="#bfefff", glow="#3f8fb0", tint="#e8fbff"),
    "wind":    dict(sky=("#11321f", "#08170f", "#020604"), ring="#8fe0a6", glow="#2f7a4a", tint="#dcffe6"),
    "earth":   dict(sky=("#352a10", "#181207", "#060402"), ring="#e2c27a", glow="#7a5a20", tint="#fff0cc"),
    "thunder": dict(sky=("#2e1644", "#150a20", "#05020a"), ring="#e08cff", glow="#7a2fa8", tint="#f6dcff"),
    "water":   dict(sky=("#0f2448", "#081025", "#02040b"), ring="#6fb0ff", glow="#1f4fa0", tint="#d6e8ff"),
    "light":   dict(sky=("#312d22", "#16140f", "#050504"), ring="#fff1c2", glow="#a89a60", tint="#fffbe8"),
    "dark":    dict(sky=("#20132b", "#0c0812", "#020103"), ring="#a982d6", glow="#3f2266", tint="#e2d2f6"),
    # schools
    "song":    dict(sky=("#3a1a2c", "#1a0b14", "#060205"), ring="#f0a8cc", glow="#8a3a64", tint="#ffe0ef"),
    "aether":  dict(sky=("#10293b", "#08141f", "#020508"), ring="#7fd8f0", glow="#1f6f8a", tint="#e0f8ff"),
    "azure":   dict(sky=("#0e2a3e", "#06141f", "#010407"), ring="#5fd0ff", glow="#0f5f8f", tint="#d8f4ff"),
    "dusk":    dict(sky=("#232235", "#11111c", "#040409"), ring="#c0b6d8", glow="#4a466e", tint="#e8e4f4"),
    "muted":   dict(sky=("#141822", "#0a0c12", "#030407"), ring="#6f7a90", glow="#2a3142", tint="#aab2c2"),
}

DEFS = """
  <linearGradient id="lit" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#9ea8ba"/><stop offset="0.5" stop-color="#f4f6fa"/><stop offset="1" stop-color="#c4ccd9"/></linearGradient>
  <linearGradient id="shd" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#2b3242"/><stop offset="0.5" stop-color="#5a6479"/><stop offset="1" stop-color="#363e50"/></linearGradient>
  <linearGradient id="litV" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#f4f6fa"/><stop offset="1" stop-color="#aeb7c7"/></linearGradient>
  <linearGradient id="shdV" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#5a6479"/><stop offset="1" stop-color="#262c3a"/></linearGradient>
  <linearGradient id="midV" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#b8c1cf"/><stop offset="1" stop-color="#7b8599"/></linearGradient>
  <linearGradient id="bInL" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#ffffff"/><stop offset="0.35" stop-color="#e6ebf2"/><stop offset="0.7" stop-color="#c2cad8"/><stop offset="1" stop-color="#e9edf3"/></linearGradient>
  <linearGradient id="bOutL" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#c9d0dc"/><stop offset="0.6" stop-color="#8f99ac"/><stop offset="1" stop-color="#b3bccb"/></linearGradient>
  <linearGradient id="bInR" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#8f99ac"/><stop offset="0.6" stop-color="#5c667b"/><stop offset="1" stop-color="#7b8599"/></linearGradient>
  <linearGradient id="bOutR" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#4f586c"/><stop offset="0.6" stop-color="#222837"/><stop offset="1" stop-color="#3a4254"/></linearGradient>
  <linearGradient id="goldL" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#fff3d1"/><stop offset="0.5" stop-color="#e3c98e"/><stop offset="1" stop-color="#c4a466"/></linearGradient>
  <linearGradient id="goldD" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#a4844c"/><stop offset="1" stop-color="#5a4222"/></linearGradient>
  <linearGradient id="woodL" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#a27c58"/><stop offset="1" stop-color="#6e4f35"/></linearGradient>
  <linearGradient id="woodD" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#4e3626"/><stop offset="1" stop-color="#2a1c13"/></linearGradient>
  <linearGradient id="brownL" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#8a6848"/><stop offset="1" stop-color="#5c4230"/></linearGradient>
  <linearGradient id="brownD" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#46301f"/><stop offset="1" stop-color="#24170e"/></linearGradient>
  <linearGradient id="parL" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#f2e7cc"/><stop offset="1" stop-color="#d4c194"/></linearGradient>
  <linearGradient id="parD" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#bba878"/><stop offset="1" stop-color="#8a7650"/></linearGradient>
  <linearGradient id="clothL" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#4a6aa4"/><stop offset="1" stop-color="#2c4270"/></linearGradient>
  <linearGradient id="clothD" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#22345a"/><stop offset="1" stop-color="#121c34"/></linearGradient>
  <linearGradient id="leather" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#0b0d14"/><stop offset="0.4" stop-color="#2b3347"/><stop offset="0.6" stop-color="#232a3b"/><stop offset="1" stop-color="#07090e"/></linearGradient>
  <linearGradient id="waxL" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#d0505a"/><stop offset="1" stop-color="#8e2630"/></linearGradient>
  <radialGradient id="gem" cx="38%" cy="32%" r="70%"><stop offset="0" stop-color="#ffffff"/><stop offset="0.3" stop-color="#bcd3f5"/><stop offset="0.7" stop-color="#8fb2e6"/><stop offset="1" stop-color="#27406b"/></radialGradient>
  <radialGradient id="pearl" cx="36%" cy="32%" r="70%"><stop offset="0" stop-color="#ffffff"/><stop offset="0.35" stop-color="#eef0fa"/><stop offset="0.75" stop-color="#c9c2e4"/><stop offset="1" stop-color="#7f78a6"/></radialGradient>
  <radialGradient id="lens" cx="40%" cy="35%" r="70%"><stop offset="0" stop-color="#cfe0f5" stop-opacity="0.45"/><stop offset="0.6" stop-color="#5f86bf" stop-opacity="0.28"/><stop offset="1" stop-color="#1e2c44" stop-opacity="0.5"/></radialGradient>
  <radialGradient id="hot" cx="50%" cy="50%" r="50%"><stop offset="0" stop-color="#ffffff"/><stop offset="0.3" stop-color="#ffffff" stop-opacity="0.8"/><stop offset="1" stop-color="#ffffff" stop-opacity="0"/></radialGradient>
  <linearGradient id="hfMetal" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#8e98aa"/><stop offset="0.45" stop-color="#f1f3f7"/><stop offset="1" stop-color="#7d8597"/></linearGradient>
  <linearGradient id="oGlass" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#dfe4ec"/><stop offset="0.5" stop-color="#aeb6c4" stop-opacity="0.45"/><stop offset="1" stop-color="#dfe4ec"/></linearGradient>
  <linearGradient id="hfDisc" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#2a303d"/><stop offset="1" stop-color="#0d1016"/></linearGradient>
  <radialGradient id="hfBack"><stop offset="0" stop-color="#6f93c8" stop-opacity="0.45"/><stop offset="0.45" stop-color="#3d5f94" stop-opacity="0.2"/><stop offset="1" stop-color="#3d5f94" stop-opacity="0"/></radialGradient>
  <radialGradient id="oCyan"><stop offset="0" stop-color="#e8fbff" stop-opacity="0.85"/><stop offset="0.35" stop-color="#7fd8f0" stop-opacity="0.35"/><stop offset="1" stop-color="#5fb8e0" stop-opacity="0"/></radialGradient>
  <linearGradient id="r13Sand" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#eef3fb" stop-opacity="0.95"/><stop offset="0.35" stop-color="#bcd0ea" stop-opacity="0.88"/><stop offset="0.75" stop-color="#7f9cc8" stop-opacity="0.85"/><stop offset="1" stop-color="#4f6a96" stop-opacity="0.85"/></linearGradient>
  <linearGradient id="oWispV" x1="0" y1="1" x2="0" y2="0"><stop offset="0" stop-color="#c3a2f0" stop-opacity="0.8"/><stop offset="1" stop-color="#9a86e0" stop-opacity="0"/></linearGradient>
  <linearGradient id="rim" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#a9b6cc"/><stop offset="0.45" stop-color="#5c6b85"/><stop offset="1" stop-color="#1e2431"/></linearGradient>
  <filter id="shadow2" x="-30%" y="-30%" width="160%" height="160%"><feDropShadow dx="0" dy="1" stdDeviation="0.9" flood-color="#000" flood-opacity="0.65"/></filter>
  <filter id="ringBlur" x="-30%" y="-30%" width="160%" height="160%"><feGaussianBlur stdDeviation="2.6"/></filter>
  <filter id="soft" x="-50%" y="-50%" width="200%" height="200%"><feGaussianBlur stdDeviation="2.2"/></filter>
  <filter id="softS" x="-50%" y="-50%" width="200%" height="200%"><feGaussianBlur stdDeviation="0.8"/></filter>
  <filter id="bloom" x="-60%" y="-60%" width="220%" height="220%"><feGaussianBlur stdDeviation="3.5"/></filter>
  <clipPath id="plateClip"><rect x="3" y="3" width="122" height="122" rx="11"/></clipPath>
"""


def theme_defs(t):
    th = THEMES[t]
    s0, s1, s2 = th["sky"]
    r, g = th["ring"], th["glow"]
    return f"""
  <radialGradient id="sky" cx="50%" cy="42%" r="78%"><stop offset="0" stop-color="{s0}"/><stop offset="0.55" stop-color="{s1}"/><stop offset="1" stop-color="{s2}"/></radialGradient>
  <radialGradient id="ringAura" cx="50%" cy="50%" r="50%"><stop offset="0.72" stop-color="{r}" stop-opacity="0"/><stop offset="0.86" stop-color="{r}" stop-opacity="0.16"/><stop offset="1" stop-color="{r}" stop-opacity="0"/></radialGradient>
  <radialGradient id="centerGlow" cx="50%" cy="48%" r="50%"><stop offset="0" stop-color="{g}" stop-opacity="0.5"/><stop offset="0.6" stop-color="{g}" stop-opacity="0.12"/><stop offset="1" stop-color="{g}" stop-opacity="0"/></radialGradient>
  <radialGradient id="tglow" cx="50%" cy="50%" r="50%"><stop offset="0" stop-color="{r}" stop-opacity="0.75"/><stop offset="0.4" stop-color="{r}" stop-opacity="0.25"/><stop offset="1" stop-color="{r}" stop-opacity="0"/></radialGradient>
"""


def stars(seed, n, tint, sizes=(0.22, 0.62)):
    rnd = random.Random(seed)
    cols = ["#ffffff", tint, "#e8eefc"]
    out = []
    for _ in range(n):
        x, y = rnd.uniform(6, 122), rnd.uniform(6, 122)
        r = rnd.uniform(*sizes) if rnd.random() > 0.08 else rnd.uniform(0.6, 0.85)
        out.append(f'<circle cx="{x:.2f}" cy="{y:.2f}" r="{r:.2f}" fill="{cols[rnd.randrange(3)]}" fill-opacity="{rnd.uniform(0.25, 0.9):.2f}"/>')
    return "".join(out)


def glint(x, y, r, o=0.9, col="#ffffff"):
    return (f'<g opacity="{o}"><path d="M{x} {y-r} L{x+r*0.12:.2f} {y} L{x} {y+r} L{x-r*0.12:.2f} {y} Z '
            f'M{x-r} {y} L{x} {y+r*0.12:.2f} L{x+r} {y} L{x} {y-r*0.12:.2f} Z" fill="{col}"/>'
            f'<circle cx="{x}" cy="{y}" r="{r*0.35:.2f}" fill="{col}" fill-opacity="0.4"/></g>')


def plate_bg(t, seed):
    th = THEMES[t]
    rnd = random.Random(seed * 7 + 3)
    gl = []
    corners = [(24, 26), (104, 98), (100, 22), (22, 100), (106, 60), (20, 62)]
    rnd.shuffle(corners)
    for (x, y), r, o in zip(corners[:3], (2.6, 2.0, 1.6), (0.75, 0.55, 0.6)):
        gl.append(glint(x + rnd.uniform(-3, 3), y + rnd.uniform(-3, 3), r, o))
    r, R = th["ring"], 46
    return ('<rect x="3" y="3" width="122" height="122" fill="url(#sky)"/>'
            + stars(seed, 70, th["tint"]) + "".join(gl)
            + '<ellipse cx="64" cy="62" rx="40" ry="40" fill="url(#centerGlow)"/>'
            + f'<circle cx="64" cy="62" r="{R+6}" fill="url(#ringAura)"/>'
            + f'<circle cx="64" cy="62" r="{R}" fill="none" stroke="{r}" stroke-opacity="0.32" stroke-width="2.4" filter="url(#ringBlur)"/>'
            + f'<circle cx="64" cy="62" r="{R}" fill="none" stroke="{th["tint"]}" stroke-opacity="0.28" stroke-width="0.35"/>')


def plate_front():
    pale = "#d6e1f0"
    return f"""
  <rect x="3" y="3" width="122" height="122" rx="11" fill="none" stroke="#000" stroke-opacity="0.6" stroke-width="3"/>
  <rect x="3.5" y="3.5" width="121" height="121" rx="10.5" fill="none" stroke="url(#rim)" stroke-width="1.6"/>
  <rect x="7.5" y="7.5" width="113" height="113" rx="7.5" fill="none" stroke="{pale}" stroke-opacity="0.1" stroke-width="0.6"/>
  <g fill="none" stroke="{pale}" stroke-opacity="0.35" stroke-width="0.6"><path d="M44 7.5 H58 M70 7.5 H84"/><path d="M44 120.5 H58 M70 120.5 H84"/></g>
  <g fill="{pale}" fill-opacity="0.8"><path d="M64 5.3 L66.2 7.5 L64 9.7 L61.8 7.5 Z"/><path d="M64 118.3 L66.2 120.5 L64 122.7 L61.8 120.5 Z"/></g>
  <path d="M8 18 Q8 8 18 8 H110 Q120 8 120 18" fill="none" stroke="#fff" stroke-opacity="0.06" stroke-width="1"/>"""


def icon_svg(title, theme, body, seed=1, extra_defs=""):
    return f"""<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 128 128" width="128" height="128" shape-rendering="geometricPrecision">
  <title>{title}</title>
  <defs>{DEFS}{theme_defs(theme)}{extra_defs}
  </defs>
  <g clip-path="url(#plateClip)">
  {plate_bg(theme, seed)}
  {body}
  </g>
{plate_front()}
</svg>
"""


# ---------- helpers ----------
def pts(points):
    return " ".join(f"{x:.2f},{y:.2f}" for x, y in points)


def poly(points, fill, extra=""):
    return f'<polygon points="{pts(points)}" fill="{fill}" {extra}/>'


def line(d, col="#ffffff", op=0.8, w=0.28, extra=""):
    return f'<path d="{d}" fill="none" stroke="{col}" stroke-opacity="{op}" stroke-width="{w}" {extra}/>'


def edge(d, w=0.28):
    return f'<path d="{d}" fill="none" stroke="{E}" stroke-width="{w}" stroke-linejoin="miter"/>'


def g(content, tf="", shadow=True, extra=""):
    f = ' filter="url(#shadow2)"' if shadow else ""
    t = f' transform="{tf}"' if tf else ""
    return f'<g{t}{f} {extra}>{content}</g>'


def tglow(x, y, r, op=1.0):
    return f'<circle cx="{x}" cy="{y}" r="{r}" fill="url(#tglow)" opacity="{op}"/>'


def crystal(cx, cy, h_up, h_dn, hw, cols=("#80bde7", "#5588c1", "#68a0d2", "#3f6fa8"), edge_op=0.6, sw=0.3):
    """Logo-style elongated crystal: four faces, white edges."""
    t, b = cy - h_up, cy + h_dn
    m = hw * 0.35
    return (poly([(cx, t), (cx - hw, cy), (cx, cy + m)], cols[0])
            + poly([(cx, t), (cx + hw, cy), (cx, cy + m)], cols[1])
            + poly([(cx, b), (cx - hw, cy), (cx, cy + m)], cols[2])
            + poly([(cx, b), (cx + hw, cy), (cx, cy + m)], cols[3])
            + f'<path d="M{cx} {t} L{cx-hw} {cy} L{cx} {b} L{cx+hw} {cy} Z M{cx-hw} {cy} L{cx} {cy+m:.2f} L{cx+hw} {cy} M{cx} {t} L{cx} {cy+m:.2f}" '
              f'fill="none" stroke="#fff" stroke-opacity="{edge_op}" stroke-width="{sw}" stroke-linejoin="round"/>')
