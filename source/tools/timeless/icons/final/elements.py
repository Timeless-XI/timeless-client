"""Element cores (one design per element, scaled by tier) and the tiered spell icons."""
from base import *  # noqa


def spiral_pts(cx, cy, R, a0, sweep, squash, k=0.55, n=28):
    pts = []
    for i in range(n + 1):
        t = i / n
        r = R * (1 - k * t)
        a = math.radians(a0 + sweep * t)
        pts.append((cx + r * math.cos(a), cy + r * math.sin(a) * squash))
    return pts


def poly(pts):
    return "M" + " L".join(f"{x:.2f} {y:.2f}" for x, y in pts)


def leaf(x, y, L, ang, col="#5aa040"):
    a = math.radians(ang)
    ex, ey = x + L * math.cos(a), y + L * math.sin(a)
    nx, ny = -math.sin(a) * L * 0.32, math.cos(a) * L * 0.32
    mx, my = (x + ex) / 2, (y + ey) / 2
    d = f"M{x:.2f} {y:.2f} Q{mx+nx:.2f} {my+ny:.2f} {ex:.2f} {ey:.2f} Q{mx-nx:.2f} {my-ny:.2f} {x:.2f} {y:.2f} Z"
    return (f'<path d="{d}" fill="{col}"/><path d="{d}" fill="none" stroke="#0e2a10" stroke-width="0.3"/>'
            f'<path d="M{x:.2f} {y:.2f} L{ex:.2f} {ey:.2f}" stroke="#d8f8b0" stroke-opacity="0.6" stroke-width="0.3"/>')


# ---------------------------------------------------------------- WIND
def aero_core(cx, cy, sc, tier, seed):
    R = 34 * sc
    rnd = random.Random(seed)
    out = [halo(cx, cy, R * 1.1, "#8ff0c0", 0.35)]
    n = 3 + tier
    for i in range(n):
        a0 = i * 360 / n + rnd.uniform(-12, 12)
        pts = spiral_pts(cx, cy, R * rnd.uniform(0.86, 1.05), a0, 250, 0.62, 0.62)
        d = poly(pts)
        w = (2.6 + tier * 0.5) * sc * 1.6
        out.append(f'<path d="{d}" fill="none" stroke="#0e3a28" stroke-opacity="0.55" stroke-width="{w*1.5:.2f}" stroke-linecap="round" filter="url(#b1)"/>')
        out.append(f'<path d="{d}" fill="none" stroke="#3aa878" stroke-opacity="0.85" stroke-width="{w:.2f}" stroke-linecap="round"/>')
        out.append(f'<path d="{d}" fill="none" stroke="#a8f8d0" stroke-opacity="0.9" stroke-width="{w*0.45:.2f}" stroke-linecap="round" transform="translate(-0.4 -0.6)"/>')
        out.append(f'<path d="{poly(pts[:10])}" fill="none" stroke="#f0fff8" stroke-opacity="0.8" stroke-width="{w*0.18:.2f}" stroke-linecap="round" transform="translate(-0.5 -0.8)"/>')
        # leaves caught in the gust
        for k in (8, 17):
            x, y = pts[k]
            out.append(leaf(x, y, 4.2 * sc + tier * 0.4, a0 + 250 * k / 28 + 90, rnd.choice(["#6ab048", "#8ac858", "#a0b040"])))
    out.append(blob(cx, cy, R * 0.22, R * 0.14, "#e8fff2", 0.9, "b2"))
    out.append(blob(cx, cy, R * 0.1, R * 0.07, "#ffffff", 1, "b06"))
    if tier >= 3:
        out.append(f'<ellipse cx="{cx}" cy="{cy+R*0.95:.1f}" rx="{R*0.8:.1f}" ry="{R*0.14:.1f}" fill="none" stroke="#a8f8d0" stroke-opacity="0.4" stroke-width="1.2" filter="url(#b06)"/>')
    return "".join(out)


# ---------------------------------------------------------------- EARTH
def rock(cx, cy, r, seed, rot=0, sides=7, base="#8a6a48", lo="#2a1c10", hi="#e8c890"):
    rnd = random.Random(seed)
    pts = []
    for i in range(sides):
        a = math.radians(rot + i * 360 / sides + rnd.uniform(-14, 14))
        rr = r * rnd.uniform(0.72, 1.08)
        pts.append((cx + rr * math.cos(a), cy + rr * math.sin(a) * 0.86))
    d = poly(pts) + " Z"
    facet = f'<path d="M{pts[0][0]:.1f} {pts[0][1]:.1f} L{cx-r*0.1:.1f} {cy-r*0.15:.1f} L{pts[3][0]:.1f} {pts[3][1]:.1f}" fill="none" stroke="#1a1008" stroke-opacity="0.55" stroke-width="0.5"/>'
    return form(d, base, lo, hi=hi, hl=(cx - r * 0.3, cy - r * 0.4, r * 0.5, r * 0.28), ao=0.45, ow=0.45, outline="#120a04",
                off=(r * 0.12, r * 0.16), blur="b1", tex="rock", extra=facet)


def stone_core(cx, cy, sc, tier, seed):
    R = 38 * sc
    rnd = random.Random(seed)
    out = [halo(cx, cy, R * 1.2, "#f0b860", 0.4)]
    # glowing earth heart with cracks
    out.append(blob(cx, cy, R * 0.42, R * 0.36, "#ffb050", 0.85, "b3"))
    out.append(rock(cx, cy, R * 0.42, seed, rnd.uniform(0, 60), 8, "#6a4a2c", "#1e1208", "#ffcf80"))
    for k in range(3 + tier):
        a = math.radians(rnd.uniform(0, 360))
        x2, y2 = cx + R * 0.36 * math.cos(a), cy + R * 0.32 * math.sin(a)
        mx, my = (cx + x2) / 2 + rnd.uniform(-2, 2), (cy + y2) / 2 + rnd.uniform(-2, 2)
        out.append(f'<path d="M{cx:.1f} {cy:.1f} L{mx:.1f} {my:.1f} L{x2:.1f} {y2:.1f}" fill="none" stroke="#ff9a30" stroke-width="{1.1*sc+0.3:.2f}" filter="url(#b06)"/>'
                   f'<path d="M{cx:.1f} {cy:.1f} L{mx:.1f} {my:.1f} L{x2:.1f} {y2:.1f}" fill="none" stroke="#fff0b0" stroke-width="{0.4*sc+0.2:.2f}"/>')
    # orbiting boulders, back half first
    n = 4 + tier
    orb = []
    for i in range(n):
        a = math.radians(i * 360 / n + rnd.uniform(-15, 15))
        rr = R * rnd.uniform(0.85, 1.05)
        x, y = cx + rr * math.cos(a), cy + rr * math.sin(a) * 0.5
        orb.append((math.sin(a), x, y, R * rnd.uniform(0.2, 0.32), rnd.randint(0, 999)))
    for s_, x, y, r, sd in sorted(orb):
        out.append(rock(x, y, r, sd, sd % 60))
        out.append(f'<path d="M{x-r*1.6:.1f} {y+r*0.2:.1f} Q{x-r*0.8:.1f} {y+r*0.9:.1f} {x:.1f} {y+r*0.9:.1f}" fill="none" stroke="#e8c890" stroke-opacity="0.25" stroke-width="0.6"/>')
    out.append(f'<ellipse cx="{cx}" cy="{cy}" rx="{R*1.05:.1f}" ry="{R*0.52:.1f}" fill="none" stroke="#f0c070" stroke-opacity="0.25" stroke-width="0.8" stroke-dasharray="2 3"/>')
    out.append(motes(seed + 3, 6 + tier * 3, (cx - R, cy - R * 0.6, cx + R, cy + R * 0.8), ["#c8a070", "#8a6a48"], (0.4, 0.9), glow=False))
    return "".join(out)


# ---------------------------------------------------------------- THUNDER
def bolt_pts(x0, y0, x1, y1, n, jag, rnd):
    pts = [(x0, y0)]
    for i in range(1, n):
        t = i / n
        pts.append((x0 + (x1 - x0) * t + rnd.uniform(-jag, jag), y0 + (y1 - y0) * t + rnd.uniform(-jag * 0.3, jag * 0.3)))
    pts.append((x1, y1))
    return pts


def bolt_draw(pts, w, sc):
    d = poly(pts)
    return (f'<path d="{d}" fill="none" stroke="#7a5aff" stroke-opacity="0.55" stroke-width="{w*3.2:.2f}" stroke-linejoin="round" filter="url(#b2)"/>'
            f'<path d="{d}" fill="none" stroke="#c8b0ff" stroke-width="{w*1.4:.2f}" stroke-linejoin="round" filter="url(#b06)"/>'
            f'<path d="{d}" fill="none" stroke="#ffffff" stroke-width="{w*0.6:.2f}" stroke-linejoin="round"/>')


def thunder_core(cx, cy, sc, tier, seed):
    rnd = random.Random(seed)
    top, bot = cy - 44 * sc, cy + 36 * sc
    out = [halo(cx, bot, 30 * sc, "#c8b0ff", 0.55), halo(cx, top + 8, 26 * sc, "#a890ff", 0.35)]
    # storm cloud knot at the top
    for i in range(5):
        out.append(blob(cx + (i - 2) * 9 * sc, top + rnd.uniform(-3, 2) * sc, 10 * sc, 6 * sc, rnd.choice(["#2a2050", "#3a2e6a", "#1c1640"]), 0.9, "b2"))
    out.append(blob(cx, top + 4 * sc, 16 * sc, 3 * sc, "#c8b0ff", 0.5, "b1"))
    w = (1.4 + tier * 0.45) * sc * 1.3
    main = bolt_pts(cx + rnd.uniform(-4, 4) * sc, top + 4 * sc, cx, bot, 7, 8 * sc, rnd)
    out.append(bolt_draw(main, w, sc))
    for b in range(tier + 1):
        k = rnd.randint(1, 5)
        x0, y0 = main[k]
        side = rnd.choice([-1, 1])
        br = bolt_pts(x0, y0, x0 + side * rnd.uniform(10, 20) * sc, y0 + rnd.uniform(10, 22) * sc, 4, 4 * sc, rnd)
        out.append(bolt_draw(br, w * 0.5, sc))
    # strike flare
    out.append(blob(cx, bot, 12 * sc, 4 * sc, "#e8e0ff", 0.85, "b2"))
    out.append(sparkle(cx, bot - 1, 7 * sc + tier, "#ffffff"))
    for i in range(6 + tier * 2):
        a = math.radians(rnd.uniform(180, 360))
        r1, r2 = 6 * sc, rnd.uniform(10, 18) * sc
        out.append(f'<path d="M{cx+r1*math.cos(a):.1f} {bot+r1*math.sin(a)*0.5:.1f} L{cx+r2*math.cos(a):.1f} {bot+r2*math.sin(a)*0.5:.1f}" stroke="#e8e0ff" stroke-opacity="0.7" stroke-width="0.5"/>')
    return "".join(out)


# ---------------------------------------------------------------- WATER
def droplet(x, y, r, col="#9fe0ff"):
    d = f"M{x:.2f} {y-r*1.6:.2f} C{x+r*0.4:.2f} {y-r*0.7:.2f} {x+r:.2f} {y-r*0.2:.2f} {x+r:.2f} {y+r*0.3:.2f} A{r:.2f} {r:.2f} 0 0 1 {x-r:.2f} {y+r*0.3:.2f} C{x-r:.2f} {y-r*0.2:.2f} {x-r*0.4:.2f} {y-r*0.7:.2f} {x:.2f} {y-r*1.6:.2f} Z"
    return (f'<path d="{d}" fill="{col}" fill-opacity="0.75"/><path d="{d}" fill="none" stroke="#e8fbff" stroke-opacity="0.7" stroke-width="0.35"/>'
            f'<ellipse cx="{x-r*0.35:.2f}" cy="{y-r*0.1:.2f}" rx="{r*0.22:.2f}" ry="{r*0.35:.2f}" fill="#ffffff" fill-opacity="0.85"/>')


def water_core(cx, cy, sc, tier, seed):
    R = 22 * sc
    rnd = random.Random(seed)
    g = uid("wo")
    out = [f'<defs>{rg(g, [(0, "#bff0ff"), (0.35, "#3ab0e8"), (0.8, "#0c5a9a"), (1, "#062a50")], cx="38%", cy="35%", r="70%")}</defs>',
           halo(cx, cy, R * 1.7, "#6fd0ff", 0.4)]
    # orbiting water ribbon behind
    out.append(f'<ellipse cx="{cx}" cy="{cy}" rx="{R*1.45:.1f}" ry="{R*0.42:.1f}" fill="none" stroke="#1a6aa8" stroke-opacity="0.6" stroke-width="{2.2*sc+tier*0.3:.2f}" transform="rotate(-14 {cx} {cy})"/>')
    out.append(f'<circle cx="{cx}" cy="{cy}" r="{R:.1f}" fill="url(#{g})"/>')
    # inner currents
    for i in range(2 + tier):
        a0 = rnd.uniform(0, 360)
        pts = spiral_pts(cx, cy, R * rnd.uniform(0.55, 0.85), a0, 160, 0.75, 0.3, 16)
        out.append(f'<path d="{poly(pts)}" fill="none" stroke="#d8f8ff" stroke-opacity="0.45" stroke-width="{0.9*sc+0.2:.2f}" stroke-linecap="round"/>')
    out.append(f'<circle cx="{cx}" cy="{cy}" r="{R:.1f}" fill="none" stroke="#e0f6ff" stroke-opacity="0.65" stroke-width="0.7"/>')
    out.append(f'<path d="M{cx-R*0.75:.1f} {cy+R*0.35:.1f} A{R:.1f} {R:.1f} 0 0 0 {cx+R*0.6:.1f} {cy+R*0.78:.1f}" fill="none" stroke="#8ff0ff" stroke-opacity="0.5" stroke-width="{1.4*sc:.2f}" filter="url(#b06)"/>')
    out.append(blob(cx - R * 0.38, cy - R * 0.42, R * 0.3, R * 0.18, "#ffffff", 0.85, "b1", -30))
    # front half of the ribbon + droplets flung off it
    fr = f'M{cx-R*1.45:.1f} {cy} A{R*1.45:.1f} {R*0.42:.1f} 0 0 0 {cx+R*1.45:.1f} {cy}'
    out.append(f'<path d="{fr}" fill="none" stroke="#3ab0e8" stroke-opacity="0.85" stroke-width="{2.2*sc+tier*0.3:.2f}" transform="rotate(-14 {cx} {cy})"/>')
    out.append(f'<path d="{fr}" fill="none" stroke="#e0f6ff" stroke-opacity="0.8" stroke-width="{0.6*sc+0.2:.2f}" transform="rotate(-14 {cx} {cy}) translate(0 -0.6)"/>')
    for i in range(3 + tier * 2):
        a = math.radians(rnd.uniform(0, 360))
        rr = R * rnd.uniform(1.3, 1.9)
        out.append(droplet(cx + rr * math.cos(a), cy + rr * math.sin(a) * 0.7, rnd.uniform(1.1, 2.0) * (0.7 + sc * 0.4)))
    return "".join(out)


# ---------------------------------------------------------------- LIGHT
def light_core(cx, cy, sc, tier, seed):
    R = 30 * sc
    rnd = random.Random(seed)
    out = [halo(cx, cy, R * 1.6, "#fff0c0", 0.55)]
    for i in range(12 + tier * 4):
        a = math.radians(i * 360 / (12 + tier * 4) + rnd.uniform(-4, 4))
        L = R * rnd.uniform(0.9, 1.5)
        out.append(f'<path d="M{cx} {cy} L{cx+L*math.cos(a):.1f} {cy+L*math.sin(a):.1f}" stroke="#ffe6a0" stroke-opacity="0.35" stroke-width="{1.6*sc+0.4:.2f}" filter="url(#b1)"/>')
    # four-point star
    s = R * 0.95
    d = (f"M{cx} {cy-s:.1f} Q{cx+s*0.12:.1f} {cy-s*0.12:.1f} {cx+s:.1f} {cy} Q{cx+s*0.12:.1f} {cy+s*0.12:.1f} {cx} {cy+s:.1f} "
         f"Q{cx-s*0.12:.1f} {cy+s*0.12:.1f} {cx-s:.1f} {cy} Q{cx-s*0.12:.1f} {cy-s*0.12:.1f} {cx} {cy-s:.1f} Z")
    out.append(f'<path d="{d}" fill="#ffd27a" fill-opacity="0.6" filter="url(#b1)"/><path d="{d}" fill="#fff6dc"/>')
    s2 = s * 0.5
    out.append(f'<path d="{d}" fill="#ffffff" transform="translate({cx} {cy}) rotate(45) scale(0.48) translate({-cx} {-cy})" opacity="0.85"/>')
    out.append(f'<circle cx="{cx}" cy="{cy}" r="{R*0.62:.1f}" fill="none" stroke="#ffe6a0" stroke-opacity="0.6" stroke-width="{0.8+tier*0.2:.2f}"/>')
    if tier >= 2:
        out.append(f'<circle cx="{cx}" cy="{cy}" r="{R*0.8:.1f}" fill="none" stroke="#fff6dc" stroke-opacity="0.35" stroke-width="0.5" stroke-dasharray="1.2 2.4"/>')
    out.append(blob(cx, cy, R * 0.2, R * 0.2, "#ffffff", 1, "b1"))
    return "".join(out)


# ---------------------------------------------------------------- DARK
def dark_core(cx, cy, sc, tier, seed):
    R = 22 * sc
    rnd = random.Random(seed)
    g = uid("do")
    out = [f'<defs>{rg(g, [(0, "#000000"), (0.6, "#14061e"), (0.85, "#4a1a6a"), (1, "#c080ff")])}</defs>',
           halo(cx, cy, R * 1.9, "#a060c0", 0.45)]
    for i in range(5 + tier * 2):
        a0 = rnd.uniform(0, 360)
        pts = spiral_pts(cx, cy, R * rnd.uniform(1.4, 1.9), a0, -120, 0.9, 0.45, 14)
        out.append(f'<path d="{poly(pts)}" fill="none" stroke="#6a2a8a" stroke-opacity="0.6" stroke-width="{2.4*sc:.2f}" stroke-linecap="round" filter="url(#b1)"/>'
                   f'<path d="{poly(pts)}" fill="none" stroke="#d8a0ff" stroke-opacity="0.55" stroke-width="{0.6*sc+0.2:.2f}" stroke-linecap="round"/>')
    out.append(f'<circle cx="{cx}" cy="{cy}" r="{R:.1f}" fill="url(#{g})"/>')
    out.append(f'<circle cx="{cx}" cy="{cy}" r="{R:.1f}" fill="none" stroke="#e0b0ff" stroke-opacity="0.7" stroke-width="0.6"/>')
    out.append(blob(cx + R * 0.3, cy - R * 0.35, R * 0.18, R * 0.1, "#d8a0ff", 0.5, "b1", 30))
    return "".join(out)


CORES = {"fire": None, "ice": None, "wind": aero_core, "earth": stone_core, "thunder": thunder_core, "water": water_core,
         "light": light_core, "dark": dark_core}
SPELLS = {"wind": ("sp_aero", "Aero", "Aeroga"), "earth": ("sp_stone", "Stone", "Stonega"), "thunder": ("sp_thunder", "Thunder", "Thundaga"),
          "water": ("sp_water", "Water", "Waterga"), "light": ("sp_banish", "Banish", "Banishga")}
ROMAN = ["", "", " II", " III", " IV", " V"]


def build():
    for el, (key, one, ga) in SPELLS.items():
        core = CORES[el]
        for t in (1, 2, 3, 4):
            make(f"{key}{t}", f"Spell - {one}{ROMAN[t]}", el, lambda: ring(el) + tiered(core, t, False, el, seed=t), seed=t)
        for t in (1, 2, 3):
            make(f"{key}ga{t}", f"Spell - {ga}{ROMAN[t]}", el, lambda: ring(el) + tiered(core, t, True, el, seed=t + 10), seed=t + 4)


if __name__ == "__main__":
    build()
    print("ok")
