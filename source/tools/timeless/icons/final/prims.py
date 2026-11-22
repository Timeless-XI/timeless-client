"""Painted primitives shared by the full set."""
from base import *  # noqa
from elements import poly, spiral_pts, leaf, droplet, rock, bolt_pts, bolt_draw  # noqa

MAT = {
    "steel": dict(base="#9aa4b4", lo="#2a303c", hi="#eae6dc", rim="#86a8d8", outline="#11141b", tex="mottle"),
    "iron": dict(base="#6e7078", lo="#1c1c22", hi="#c8c0b0", rim="#e08a48", outline="#0e0e12", tex="hammered"),
    "wood": dict(base="#9a6438", lo="#3a2212", hi="#d8a26a", rim="#f0b070", outline="#1e1008", tex="wood"),
    "leather": dict(base="#8a5230", lo="#2e170a", hi="#c88c58", rim="#e8a070", outline="#1a0c05", tex="mottle"),
    "brass": dict(base="#c09040", lo="#4a2e0c", hi="#ffe6a8", rim="#ffd080", outline="#24160a", tex="mottle"),
    "parch": dict(base="#dcc79a", lo="#8a7048", hi="#fff4dc", rim="#ffe0a8", outline="#4a3418", tex="paperT"),
    "cloth_red": dict(base="#a83a32", lo="#3a0e0c", hi="#f08a70", rim="#ffb090", outline="#1e0606", tex="mottle"),
    "cloth_blue": dict(base="#3a5a9a", lo="#0e1a3a", hi="#9ab8f0", rim="#a8c8ff", outline="#080e1e", tex="mottle"),
    "stone": dict(base="#8a8478", lo="#2a2822", hi="#e0dccc", rim="#c8c0a8", outline="#14120e", tex="rock"),
    "bone": dict(base="#d8cdb0", lo="#6a5e44", hi="#fffbe8", rim="#fff0c8", outline="#2a2214", tex="mottle"),
    "silver": dict(base="#c8d0dc", lo="#4a5262", hi="#ffffff", rim="#a8c8ff", outline="#1a1e26", tex="mottle"),
}


def paint(d, mat, hl=None, rim=True, off=(1.6, 2.0), extra="", tf="", blur="b1", ao=0.4, ow=0.5):
    m = MAT[mat] if isinstance(mat, str) else mat
    return form(d, m["base"], m["lo"], hi=m["hi"], hl=hl, rim=m["rim"] if rim else None, off=off, blur=blur,
                outline=m["outline"], tex=m["tex"], ao=ao, extra=extra, tf=tf, ow=ow)


def circle_d(cx, cy, r):
    return f"M{cx-r:.2f} {cy:.2f} A{r:.2f} {r:.2f} 0 1 0 {cx+r:.2f} {cy:.2f} A{r:.2f} {r:.2f} 0 1 0 {cx-r:.2f} {cy:.2f} Z"


def ell_d(cx, cy, rx, ry):
    return f"M{cx-rx:.2f} {cy:.2f} A{rx:.2f} {ry:.2f} 0 1 0 {cx+rx:.2f} {cy:.2f} A{rx:.2f} {ry:.2f} 0 1 0 {cx-rx:.2f} {cy:.2f} Z"


def gem(cx, cy, r, col=("#3a6ad0", "#0c1a4a", "#a8d0ff")):
    c1, c2, c3 = col
    d = f"M{cx} {cy-r:.2f} L{cx+r:.2f} {cy:.2f} L{cx} {cy+r:.2f} L{cx-r:.2f} {cy:.2f} Z"
    return (f'<path d="{d}" fill="{c2}"/><path d="M{cx} {cy-r:.2f} L{cx+r:.2f} {cy:.2f} L{cx} {cy:.2f} Z" fill="{c1}"/>'
            f'<path d="M{cx} {cy-r:.2f} L{cx-r:.2f} {cy:.2f} L{cx} {cy:.2f} Z" fill="{c3}" fill-opacity="0.8"/>'
            f'<path d="{d}" fill="none" stroke="#0a0a12" stroke-width="0.4"/>' + sparkle(cx - r * 0.3, cy - r * 0.35, r * 0.7))


def shield_d(cx, cy, w, h):
    return (f"M{cx-w:.1f} {cy-h*0.62:.1f} Q{cx} {cy-h*0.8:.1f} {cx+w:.1f} {cy-h*0.62:.1f} L{cx+w*0.96:.1f} {cy+h*0.05:.1f} "
            f"Q{cx+w*0.8:.1f} {cy+h*0.6:.1f} {cx} {cy+h:.1f} Q{cx-w*0.8:.1f} {cy+h*0.6:.1f} {cx-w*0.96:.1f} {cy+h*0.05:.1f} Z")


def shield(cx, cy, w, h, face="cloth_blue", trim="brass", emblem=""):
    d = shield_d(cx, cy, w, h)
    inner = shield_d(cx, cy + h * 0.04, w * 0.8, h * 0.82)
    return (cast_shadow(d, 2, 3, 0.5) + paint(d, trim, hl=(cx - w * 0.4, cy - h * 0.4, w * 0.4, h * 0.2))
            + paint(inner, face, hl=(cx - w * 0.3, cy - h * 0.3, w * 0.5, h * 0.3), rim=False) + emblem
            + "".join(f'<circle cx="{x:.1f}" cy="{y:.1f}" r="1.1" fill="#ffe6a8"/><circle cx="{x:.1f}" cy="{y:.1f}" r="1.1" fill="none" stroke="#3a240a" stroke-width="0.3"/>'
                      for x, y in ((cx - w * 0.82, cy - h * 0.52), (cx + w * 0.82, cy - h * 0.52), (cx, cy + h * 0.86))))


def blade_d(L, w, tip=0.18):
    """Blade along -y from the guard at 0: width w, length L."""
    return f"M{-w/2:.2f} 0 L{-w/2:.2f} {-L*(1-tip):.2f} L0 {-L:.2f} L{w/2:.2f} {-L*(1-tip):.2f} L{w/2:.2f} 0 Z"


def sword(cx, cy, L=70, ang=-35, w=7, gemcol=("#3a6ad0", "#0c1a4a", "#a8d0ff"), glow=None):
    b = blade_d(L * 0.72, w)
    fuller = f'<path d="M0 -3 L0 {-L*0.6:.1f}" stroke="#5a6478" stroke-width="{w*0.18:.2f}" stroke-linecap="round"/>'
    edge = f'<path d="M{w/2:.2f} 0 L{w/2:.2f} {-L*0.72*0.82:.1f} L0 {-L*0.72:.1f}" fill="none" stroke="#f4f8ff" stroke-opacity="0.75" stroke-width="0.5"/>'
    guard = f"M{-w*1.9:.1f} -1.2 Q0 -3.4 {w*1.9:.1f} -1.2 L{w*1.9:.1f} 2.2 Q0 0.6 {-w*1.9:.1f} 2.2 Z"
    grip = f"M{-w*0.32:.1f} 2 L{w*0.32:.1f} 2 L{w*0.28:.1f} {L*0.2:.1f} L{-w*0.28:.1f} {L*0.2:.1f} Z"
    out = [f'<g transform="translate({cx} {cy}) rotate({ang})">']
    if glow:
        out.append(f'<path d="{b}" fill="none" stroke="{glow}" stroke-opacity="0.75" stroke-width="5" filter="url(#b2)"/>')
    out += [paint(b, "steel", hl=(-w * 0.2, -L * 0.4, w * 0.25, L * 0.25), extra=fuller), edge,
            paint(grip, "leather", rim=False) + "".join(f'<path d="M{-w*0.3:.1f} {y:.1f} L{w*0.3:.1f} {y+1.6:.1f}" stroke="#2a1408" stroke-width="0.5"/>' for y in [4 + k * 2.6 for k in range(int(L * 0.06))]),
            paint(guard, "brass", hl=(-w, -1, w * 0.8, 0.8)),
            paint(circle_d(0, L * 0.2 + 2.4, w * 0.48), "brass"), gem(0, 0.5, w * 0.36, gemcol), "</g>"]
    return "".join(out)


def scroll(cx, cy, w, h, mat="parch", seal=None, ang=0, lines=True):
    body = f"M{cx-w/2:.1f} {cy-h/2:.1f} L{cx+w/2:.1f} {cy-h/2:.1f} L{cx+w/2:.1f} {cy+h/2:.1f} L{cx-w/2:.1f} {cy+h/2:.1f} Z"
    out = [f'<g transform="rotate({ang} {cx} {cy})">', cast_shadow(body, 2, 3, 0.45), paint(body, mat, hl=(cx - w * 0.2, cy - h * 0.2, w * 0.3, h * 0.2))]
    if lines:
        for k in range(4):
            y = cy - h * 0.3 + k * h * 0.18
            out.append(f'<path d="M{cx-w*0.32:.1f} {y:.1f} L{cx+w*(0.3 - 0.1*(k%2)):.1f} {y:.1f}" stroke="#5a4020" stroke-opacity="0.6" stroke-width="0.7"/>')
    for yy in (cy - h / 2, cy + h / 2):
        rd = f"M{cx-w/2-2:.1f} {yy-2.6:.1f} L{cx+w/2+2:.1f} {yy-2.6:.1f} A2.6 2.6 0 0 1 {cx+w/2+2:.1f} {yy+2.6:.1f} L{cx-w/2-2:.1f} {yy+2.6:.1f} A2.6 2.6 0 0 1 {cx-w/2-2:.1f} {yy-2.6:.1f} Z"
        out.append(paint(rd, mat, hl=(cx, yy - 1.2, w * 0.4, 0.8)))
        out.append(f'<ellipse cx="{cx+w/2+2:.1f}" cy="{yy:.1f}" rx="1.2" ry="2.4" fill="#8a7048" stroke="#4a3418" stroke-width="0.3"/>')
    if seal:
        out.append(paint(circle_d(cx, cy + h * 0.32, 3.6), dict(base=seal, lo="#2a0606", hi="#ffb0a0", rim="#ffd0c0", outline="#1a0404", tex="mottle")))
    out.append("</g>")
    return "".join(out)


def orb(cx, cy, r, c0, c1, c2, glow=None):
    g = uid("orb")
    out = [f'<defs>{rg(g, [(0, c0), (0.5, c1), (1, c2)], cx="38%", cy="35%", r="70%")}</defs>']
    if glow:
        out.append(halo(cx, cy, r * 2.2, glow, 0.5))
    out += [f'<circle cx="{cx}" cy="{cy}" r="{r}" fill="url(#{g})"/>', f'<circle cx="{cx}" cy="{cy}" r="{r}" fill="none" stroke="{c0}" stroke-opacity="0.6" stroke-width="0.6"/>',
            blob(cx - r * 0.35, cy - r * 0.4, r * 0.3, r * 0.18, "#ffffff", 0.8, "b1", -30)]
    return "".join(out)


def feather(cx, cy, L, ang, col="#f4f6fa", vane="#c8d0e0"):
    w = L * 0.22
    d = f"M0 0 C{w:.1f} {-L*0.2:.1f} {w*0.9:.1f} {-L*0.75:.1f} 0 {-L:.1f} C{-w*0.7:.1f} {-L*0.75:.1f} {-w:.1f} {-L*0.25:.1f} 0 0 Z"
    barbs = "".join(f'<path d="M0 {-L*t:.1f} L{w*0.8:.1f} {-L*t-L*0.08:.1f} M0 {-L*t:.1f} L{-w*0.75:.1f} {-L*t-L*0.07:.1f}" stroke="{vane}" stroke-opacity="0.7" stroke-width="0.35"/>' for t in (0.2, 0.32, 0.44, 0.56, 0.68, 0.8))
    return (f'<g transform="translate({cx} {cy}) rotate({ang})">'
            + form(d, col, "#7a8498", hi="#ffffff", hl=(-w * 0.3, -L * 0.6, w * 0.4, L * 0.2), ao=0.3, ow=0.4, outline="#3a4050", off=(0.8, 1.2), blur="b06", extra=barbs)
            + f'<path d="M0 {L*0.12:.1f} L0 {-L*0.96:.1f}" stroke="#ffffff" stroke-width="0.6"/></g>')


def wing(cx, cy, sx, s=1.0, col="#f4f6fa"):
    out = []
    for i, (a, L) in enumerate(((20, 26), (40, 31), (60, 33), (80, 30), (100, 24))):
        out.append(feather(cx, cy, L * s, sx * a, col))
    return "".join(reversed(out))


def hourglass(cx, cy, h, sand="#e3c98e", glass="#cfe0ff", cap="brass"):
    w = h * 0.36
    caps = "".join(paint(f"M{cx-w*1.2:.1f} {y-2.2:.1f} L{cx+w*1.2:.1f} {y-2.2:.1f} L{cx+w*1.2:.1f} {y+2.2:.1f} L{cx-w*1.2:.1f} {y+2.2:.1f} Z", cap) for y in (cy - h / 2, cy + h / 2))
    gl = f"M{cx-w:.1f} {cy-h/2+2:.1f} C{cx-w:.1f} {cy-h*0.12:.1f} {cx-2:.1f} {cy-3:.1f} {cx-1.2:.1f} {cy:.1f} C{cx-2:.1f} {cy+3:.1f} {cx-w:.1f} {cy+h*0.12:.1f} {cx-w:.1f} {cy+h/2-2:.1f} L{cx+w:.1f} {cy+h/2-2:.1f} C{cx+w:.1f} {cy+h*0.12:.1f} {cx+2:.1f} {cy+3:.1f} {cx+1.2:.1f} {cy:.1f} C{cx+2:.1f} {cy-3:.1f} {cx+w:.1f} {cy-h*0.12:.1f} {cx+w:.1f} {cy-h/2+2:.1f} Z"
    sandd = f"M{cx-w*0.92:.1f} {cy+h/2-2:.1f} Q{cx} {cy+h*0.12:.1f} {cx+w*0.92:.1f} {cy+h/2-2:.1f} Z M{cx-w*0.55:.1f} {cy-h*0.22:.1f} Q{cx} {cy-h*0.16:.1f} {cx+w*0.55:.1f} {cy-h*0.22:.1f} L{cx} {cy-1:.1f} Z"
    return (f'<path d="{gl}" fill="{glass}" fill-opacity="0.22"/><path d="{sandd}" fill="{sand}"/>'
            f'<path d="M{cx} {cy:.1f} L{cx} {cy+h*0.36:.1f}" stroke="{sand}" stroke-width="0.8"/>'
            f'<path d="{gl}" fill="none" stroke="{glass}" stroke-opacity="0.85" stroke-width="1"/>'
            f'<path d="M{cx-w*0.7:.1f} {cy-h*0.38:.1f} C{cx-w*0.7:.1f} {cy-h*0.2:.1f} {cx-w*0.3:.1f} {cy-h*0.1:.1f} {cx-2:.1f} {cy-2:.1f}" fill="none" stroke="#ffffff" stroke-opacity="0.6" stroke-width="0.8"/>'
            + caps
            + "".join(paint(f"M{x-1.2:.1f} {cy-h/2+2:.1f} L{x+1.2:.1f} {cy-h/2+2:.1f} L{x+1.2:.1f} {cy+h/2-2:.1f} L{x-1.2:.1f} {cy+h/2-2:.1f} Z", cap, rim=False) for x in (cx - w * 1.05, cx + w * 1.05)))


def moon(cx, cy, r, col="#e8ecff", inner=0.78, off=0.42):
    m = uid("mn")
    return (f'<defs><mask id="{m}"><rect width="128" height="128" fill="#fff"/><circle cx="{cx+r*off:.1f}" cy="{cy-r*0.2:.1f}" r="{r*inner:.1f}" fill="#000"/></mask></defs>'
            + halo(cx, cy, r * 2, col, 0.4)
            + f'<g mask="url(#{m})">' + orb(cx, cy, r, "#ffffff", col, "#8a90b8") + '</g>')


def notes(seed, n, box, col):
    rnd = random.Random(seed)
    out = []
    for _ in range(n):
        x, y = rnd.uniform(box[0], box[2]), rnd.uniform(box[1], box[3])
        s = rnd.uniform(0.7, 1.2)
        out.append(f'<g transform="translate({x:.1f} {y:.1f}) scale({s:.2f}) rotate({rnd.uniform(-15,15):.0f})" opacity="0.9">'
                   f'<ellipse cx="0" cy="0" rx="2.2" ry="1.6" transform="rotate(-25)" fill="{col}"/><path d="M1.8 -0.6 L1.8 -9 Q4.5 -7.5 5 -5" fill="none" stroke="{col}" stroke-width="0.8"/></g>')
    return "".join(out)


def lyre(cx, cy, s=1.0, glow="#ffd27a"):
    arm = lambda sx: f"M{cx+sx*4*s:.1f} {cy+16*s:.1f} C{cx+sx*16*s:.1f} {cy+12*s:.1f} {cx+sx*20*s:.1f} {cy-4*s:.1f} {cx+sx*13*s:.1f} {cy-14*s:.1f} C{cx+sx*10*s:.1f} {cy-20*s:.1f} {cx+sx*14*s:.1f} {cy-24*s:.1f} {cx+sx*17*s:.1f} {cy-24*s:.1f} L{cx+sx*16*s:.1f} {cy-20*s:.1f} C{cx+sx*16*s:.1f} {cy-14*s:.1f} {cx+sx*24*s:.1f} {cy-2*s:.1f} {cx+sx*11*s:.1f} {cy+20*s:.1f} Z"
    out = [halo(cx, cy, 36 * s, glow, 0.35), paint(arm(-1), "brass", hl=(cx - 14 * s, cy - 6 * s, 3 * s, 8 * s)), paint(arm(1), "brass", hl=(cx + 12 * s, cy - 6 * s, 3 * s, 8 * s))]
    bar = f"M{cx-17*s:.1f} {cy-17*s:.1f} L{cx+17*s:.1f} {cy-17*s:.1f} L{cx+17*s:.1f} {cy-13.5*s:.1f} L{cx-17*s:.1f} {cy-13.5*s:.1f} Z"
    base = f"M{cx-12*s:.1f} {cy+16*s:.1f} L{cx+12*s:.1f} {cy+16*s:.1f} L{cx+9*s:.1f} {cy+23*s:.1f} L{cx-9*s:.1f} {cy+23*s:.1f} Z"
    out.append(paint(bar, "wood", hl=(cx, cy - 16 * s, 12 * s, 1)))
    for k in range(5):
        x = cx + (k - 2) * 3.4 * s
        out.append(f'<path d="M{x:.1f} {cy-13.5*s:.1f} L{x:.1f} {cy+16*s:.1f}" stroke="#fff4dc" stroke-opacity="0.85" stroke-width="0.45"/>')
    out.append(paint(base, "wood", hl=(cx, cy + 18 * s, 8 * s, 1.5)))
    out.append(gem(cx, cy + 19.5 * s, 2.4 * s, ("#ff7070", "#4a0a0a", "#ffc0c0")))
    return "".join(out)


def eye(cx, cy, w, iris="#40c0e0", lid="#0c0a12"):
    d = f"M{cx-w:.1f} {cy:.1f} Q{cx} {cy-w*0.62:.1f} {cx+w:.1f} {cy:.1f} Q{cx} {cy+w*0.62:.1f} {cx-w:.1f} {cy:.1f} Z"
    c = uid("ec")
    return (f'<defs><clipPath id="{c}"><path d="{d}"/></clipPath></defs><path d="{d}" fill="#e8ecf2"/>'
            f'<g clip-path="url(#{c})">' + orb(cx, cy, w * 0.42, "#ffffff", iris, "#06202a") + f'<circle cx="{cx}" cy="{cy}" r="{w*0.16:.1f}" fill="#020406"/>'
            + f'<path d="{d}" fill="none" stroke="#000" stroke-opacity="0.5" stroke-width="3" filter="url(#b1)"/></g>'
            f'<path d="{d}" fill="none" stroke="{lid}" stroke-width="1.4"/>' + sparkle(cx - w * 0.14, cy - w * 0.14, w * 0.18))


def chain(x0, y0, x1, y1, n, r=2.4, mat="iron"):
    out = []
    for i in range(n):
        t = (i + 0.5) / n
        x, y = x0 + (x1 - x0) * t, y0 + (y1 - y0) * t
        ang = math.degrees(math.atan2(y1 - y0, x1 - x0)) + (0 if i % 2 else 90)
        d = ell_d(0, 0, r * 1.4, r * 0.8) + " " + ell_d(0, 0, r * 0.8, r * 0.3)
        out.append(f'<g transform="translate({x:.1f} {y:.1f}) rotate({ang:.0f})"><path d="{d}" fill-rule="evenodd" fill="#6e7078"/><path d="{ell_d(0,0,r*1.4,r*0.8)}" fill="none" stroke="#0e0e12" stroke-width="0.4"/>'
                   f'<path d="M{-r*1.1:.1f} {-r*0.4:.1f} Q0 {-r*0.9:.1f} {r*1.1:.1f} {-r*0.4:.1f}" fill="none" stroke="#e8e0d0" stroke-opacity="0.7" stroke-width="0.5"/></g>')
    return "".join(out)


def wisps(cx, cy, seed, n, R, col, lite, sweep=200, w=2.0):
    rnd = random.Random(seed)
    out = []
    for i in range(n):
        a0 = rnd.uniform(0, 360)
        pts = spiral_pts(cx, cy, R * rnd.uniform(0.8, 1.2), a0, sweep, 0.8, 0.7, 20)
        out.append(f'<path d="{poly(pts)}" fill="none" stroke="{col}" stroke-opacity="0.65" stroke-width="{w*1.8:.1f}" stroke-linecap="round" filter="url(#b1)"/>'
                   f'<path d="{poly(pts)}" fill="none" stroke="{lite}" stroke-opacity="0.8" stroke-width="{w*0.5:.1f}" stroke-linecap="round"/>')
    return "".join(out)


ELC = {  # element colours: glow, light, gem triple
    "fire": ("#ff7a30", "#ffd8a0", ("#ff6a3a", "#5a0e04", "#ffc0a0")),
    "ice": ("#7ae0ff", "#e8fbff", ("#7ad8ff", "#0a2a4a", "#d8f6ff")),
    "wind": ("#6ae0a0", "#e8fff2", ("#5ad890", "#0a3a20", "#c8ffe0")),
    "earth": ("#e0a040", "#ffe6b0", ("#d8a040", "#3a2208", "#ffe0a0")),
    "thunder": ("#a890ff", "#f4f0ff", ("#b090ff", "#1c0e4a", "#e8e0ff")),
    "water": ("#4ab0f0", "#e0f6ff", ("#3a90f0", "#06204a", "#c0e8ff")),
    "light": ("#ffe6a0", "#fffaf0", ("#fff0b0", "#6a5010", "#ffffff")),
    "dark": ("#a060c0", "#f0e0ff", ("#9050c0", "#1a0628", "#e0c0ff")),
}
EL_PAL = {"fire": "fire", "ice": "ice", "wind": "wind", "earth": "earth", "thunder": "thunder", "water": "water", "light": "light", "dark": "dark"}
