"""Misc Book / Commands icons in the painted style."""
from prims import *  # noqa
from weapons2 import W_archery
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

ICONS = {}


def reg(name, title, pal, seed=1, hills=True):
    def deco(fn):
        ICONS[name] = (title, pal, fn, seed, hills)
        return fn
    return deco


def reticle(cx, cy, r, col="#ff6060", lite="#ffd0d0", w=2.0):
    out = [f'<circle cx="{cx}" cy="{cy}" r="{r}" fill="none" stroke="{col}" stroke-width="{w*1.8:.1f}" stroke-opacity="0.5" filter="url(#b1)"/>',
           f'<circle cx="{cx}" cy="{cy}" r="{r}" fill="none" stroke="{lite}" stroke-width="{w*0.5:.1f}"/>']
    for a in (0, 90, 180, 270):
        x0, y0 = cx + (r - 6) * math.cos(math.radians(a)), cy + (r - 6) * math.sin(math.radians(a))
        x1, y1 = cx + (r + 8) * math.cos(math.radians(a)), cy + (r + 8) * math.sin(math.radians(a))
        out.append(f'<path d="M{x0:.1f} {y0:.1f} L{x1:.1f} {y1:.1f}" stroke="{col}" stroke-width="{w*1.6:.1f}" stroke-linecap="round"/><path d="M{x0:.1f} {y0:.1f} L{x1:.1f} {y1:.1f}" stroke="{lite}" stroke-width="{w*0.5:.1f}" stroke-linecap="round"/>')
    out.append(f'<circle cx="{cx}" cy="{cy}" r="1.8" fill="{lite}"/>')
    return "".join(out)


def figure(cx, base, h, col="#120f1c", rim="#ffb060"):
    """A standing adventurer silhouette (head, cloak, shoulders) with a rim of light."""
    s = h / 60
    d = (f"M{cx} {base-60*s:.1f} C{cx+8*s:.1f} {base-60*s:.1f} {cx+9*s:.1f} {base-50*s:.1f} {cx+6*s:.1f} {base-46*s:.1f} C{cx+16*s:.1f} {base-44*s:.1f} {cx+18*s:.1f} {base-30*s:.1f} {cx+16*s:.1f} {base-18*s:.1f} "
         f"L{cx+12*s:.1f} {base:.1f} L{cx-12*s:.1f} {base:.1f} L{cx-16*s:.1f} {base-18*s:.1f} C{cx-18*s:.1f} {base-30*s:.1f} {cx-16*s:.1f} {base-44*s:.1f} {cx-6*s:.1f} {base-46*s:.1f} "
         f"C{cx-9*s:.1f} {base-50*s:.1f} {cx-8*s:.1f} {base-60*s:.1f} {cx} {base-60*s:.1f} Z")
    return (f'<path d="{d}" fill="{rim}" fill-opacity="0.6" transform="translate(0.8 -0.4)"/><path d="{d}" fill="#5a6a9a" fill-opacity="0.3" transform="translate(-0.6 -0.3)"/>'
            f'<path d="{d}" fill="{col}"/>')


@reg("misc_disengage", "Disengage", "night", 2)
def disengage():
    sheath = "M-5 -2 L5 -2 L4.4 -54 L0 -58 L-4.4 -54 Z"
    return (ring("night", op=0.6) + halo(64, 60, 40, "#8fb2e6", 0.4)
            + '<g transform="translate(64 40) rotate(180)">' + paint(sheath, "leather", hl=(-2, -30, 2, 12)) + paint("M-5.4 -40 L5.4 -40 L5.4 -36 L-5.4 -36 Z", "brass") + paint("M-5.4 -10 L5.4 -10 L5.4 -6 L-5.4 -6 Z", "brass") + "</g>"
            + sword(64, 40, 70, 0, 7).split('<g transform="translate(64 40) rotate(0)">')[0] + '<g transform="translate(64 40)">' + paint("M-13 -1.2 Q0 -3.4 13 -1.2 L13 2.2 Q0 0.6 -13 2.2 Z", "brass") + paint("M-2.2 -2 L2.2 -2 L2 -16 L-2 -16 Z", "leather", rim=False) + paint(circle_d(0, -18, 3.4), "brass") + gem(0, 0.5, 2.5) + "</g>"
            + f'<path d="M30 92 Q64 80 98 92" fill="none" stroke="#8fb2e6" stroke-opacity="0.5" stroke-width="1.2"/>')


@reg("misc_ranged", "Ranged Attack", "wind", 3)
def ranged():
    return (ring("wind", op=0.6) + halo(64, 60, 40, "#8ff0c0", 0.35) + f'<g transform="translate(70 62) rotate(-30) scale(1.15)">{W_archery()}</g>'
            + "".join(f'<path d="M{20+k*6} {84-k*4} l14 -8" stroke="#e8fff2" stroke-opacity="0.4" stroke-width="0.8"/>' for k in range(4)))


@reg("misc_check", "Check", "dusk", 4)
def check():
    lens = circle_d(56, 52, 22)
    return (ring("dusk", op=0.6) + figure(56, 92, 46, rim="#cfe0ff")
            + f'<g transform="rotate(40 56 52)">' + paint("M53 74 L59 74 L59 104 L53 104 Z", "wood", hl=(55, 88, 1.4, 8)) + paint("M52 70 L60 70 L60 76 L52 76 Z", "brass") + "</g>"
            + f'<path d="{lens}" fill="#cfe0ff" fill-opacity="0.18"/>' + paint(circle_d(56, 52, 22) + " " + circle_d(56, 52, 18.6), "brass", hl=(44, 38, 6, 3))
            + blob(48, 44, 7, 4, "#ffffff", 0.45, "b1", -30) + sparkle(46, 42, 4))


@reg("misc_assist", "Assist", "blood", 5)
def assist():
    return (ring("blood", op=0.6) + figure(36, 96, 40, rim="#8fb2e6") + reticle(80, 52, 20)
            + f'<path d="M44 66 Q58 50 66 52" fill="none" stroke="#ffd0d0" stroke-width="2" stroke-dasharray="3 2"/><path d="M62 47 L68 52 L61 56 Z" fill="#ffd0d0"/>')


@reg("misc_lockon", "Lock On", "blood", 6)
def lockon():
    out = [ring("blood", op=0.6), figure(64, 92, 52, rim="#ff9090")]
    for sx, sy in ((-1, -1), (1, -1), (-1, 1), (1, 1)):
        x, y = 64 + sx * 26, 62 + sy * 30
        d = f"M{x - sx*0:.1f} {y - sy*12:.1f} L{x:.1f} {y:.1f} L{x - sx*12:.1f} {y:.1f}"
        out.append(f'<path d="{d}" fill="none" stroke="#ff6060" stroke-width="4" stroke-opacity="0.5" filter="url(#b1)"/><path d="{d}" fill="none" stroke="#ffd0d0" stroke-width="1.6"/>')
    out.append(paint("M57 24 L71 24 L71 30 L57 30 Z" if False else "", "brass") if False else "")
    out.append(f'<path d="M64 14 L58 22 L70 22 Z" fill="#ff6060"/><path d="M64 14 L58 22 L70 22 Z" fill="none" stroke="#ffd0d0" stroke-width="0.6"/>')
    return "".join(out)


@reg("misc_follow", "Follow", "dusk", 7)
def follow():
    out = [ring("dusk", op=0.5), figure(90, 70, 40, rim="#cfe0ff")]
    for i, (x, y, a) in enumerate(((24, 104, -60), (36, 98, -50), (48, 94, -60), (60, 88, -50), (72, 84, -60))):
        d = f"M{x-2.6} {y} C{x-3.2} {y-5} {x-1} {y-7.6} {x+0.8} {y-7} C{x+2.6} {y-6.2} {x+3.2} {y-3} {x+2} {y} C{x+1.2} {y+2} {x-2} {y+2} {x-2.6} {y} Z"
        out.append(f'<g transform="rotate({a+90} {x} {y})" opacity="{0.35+i*0.13:.2f}"><path d="{d}" fill="#cfe0ff"/></g>')
    return "".join(out)


@reg("misc_rest", "Rest", "night", 8)
def rest():
    from p8 import campfire
    return (campfire(48, 98, 0.9) + f'<g>{paint("M74 92 C74 86 108 86 112 92 C112 98 74 98 74 92 Z", "cloth_blue", hl=(90, 88, 10, 2))}{paint(ell_d(110, 92, 5, 4.6), "cloth_red")}</g>'
            + "".join(f'<text x="{x}" y="{y}" font-family="Georgia,serif" font-style="italic" font-size="{s}" fill="#e6ecf5" fill-opacity="0.8">z</text>' for x, y, s in ((92, 74, 12), (102, 62, 9), (110, 52, 7))))


@reg("misc_sit", "Sit", "night", 9)
def sit():
    from p8 import sitter
    return halo(64, 80, 40, "#ffb060", 0.25) + sitter(56, 100, 42, 1, "hug", "hood", 120) + blob(64, 102, 30, 3, "#000", 0.5, "b2")


@reg("misc_widescan", "Wide Scan", "azure", 10)
def widescan():
    g = uid("sweep")
    out = [f'<defs><linearGradient id="{g}" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#40e0ff" stop-opacity="0"/><stop offset="1" stop-color="#40e0ff" stop-opacity="0.7"/></linearGradient></defs>',
           halo(64, 62, 46, "#40e0ff", 0.3)]
    for r in (40, 30, 20, 10):
        out.append(f'<circle cx="64" cy="62" r="{r}" fill="none" stroke="#40e0ff" stroke-opacity="0.45" stroke-width="0.8"/>')
    out.append('<path d="M24 62 L104 62 M64 22 L64 102" stroke="#40e0ff" stroke-opacity="0.3" stroke-width="0.6"/>')
    out.append(f'<path d="M64 62 L104 62 A40 40 0 0 0 92 33.7 Z" fill="url(#{g})" transform="rotate(-20 64 62)"/>')
    for x, y, c in ((84, 46, "#ff6060"), (46, 76, "#ffd060"), (80, 82, "#60ff90"), (50, 44, "#ff6060")):
        out.append(f'<circle cx="{x}" cy="{y}" r="2.4" fill="{c}"/><circle cx="{x}" cy="{y}" r="5" fill="{c}" fill-opacity="0.3" filter="url(#b1)"/>')
    out.append(paint(circle_d(64, 62, 44) + " " + circle_d(64, 62, 40.5), "brass", rim=False))
    return "".join(out)


@reg("misc_region", "Region Info", "gold", 11)
def region():
    out = [halo(64, 60, 46, "#ffd27a", 0.35), paint(circle_d(64, 60, 34), "parch", hl=(54, 48, 10, 6))]
    out.append(paint(circle_d(64, 60, 34) + " " + circle_d(64, 60, 30), "brass", rim=False))
    for a, L, c in ((0, 28, "#a83a32"), (180, 28, "#e6ecf5"), (90, 20, "#3a2a14"), (270, 20, "#3a2a14"), (45, 14, "#5a4020"), (135, 14, "#5a4020"), (225, 14, "#5a4020"), (315, 14, "#5a4020")):
        t = math.radians(a - 90)
        x, y = 64 + L * math.cos(t), 60 + L * math.sin(t)
        px, py = -math.sin(t) * 3.4, math.cos(t) * 3.4
        out.append(f'<path d="M{64+px:.1f} {60+py:.1f} L{x:.1f} {y:.1f} L{64-px:.1f} {60-py:.1f} Z" fill="{c}"/><path d="M64 60 L{x:.1f} {y:.1f}" stroke="#fff4dc" stroke-opacity="0.5" stroke-width="0.4"/>')
    out.append(paint(circle_d(64, 60, 3), "brass") + '<text x="64" y="23" text-anchor="middle" font-family="Georgia,serif" font-size="7" fill="#3a2a14">N</text>')
    return "".join(out)


@reg("misc_status", "Status", "night", 12, False)
def status():
    out = [halo(64, 60, 46, "#8fb2e6", 0.35)]
    book = "M30 30 L98 30 L98 96 L30 96 Z"
    out.append(cast_shadow(book, 3, 4) + paint(book, "cloth_blue", hl=(50, 44, 12, 10)))
    out.append(paint("M36 34 L94 34 L94 92 L36 92 Z", "parch", rim=False))
    out.append(figure(52, 76, 32, "#2a2018", "#ffffff"))
    for k, (c, w) in enumerate((("#e0606c", 22), ("#5f9be6", 18), ("#e3c98e", 14))):
        y = 48 + k * 9
        out.append(f'<rect x="66" y="{y}" width="24" height="4" rx="1" fill="#3a2a14" fill-opacity="0.35"/><rect x="66" y="{y}" width="{w}" height="4" rx="1" fill="{c}"/>')
    out.append(paint("M30 30 L36 30 L36 96 L30 96 Z", "brass", rim=False))
    return "".join(out)


@reg("misc_equipment", "Equipment", "gold", 13)
def equipment():
    helm = "M44 40 C44 24 84 24 84 40 L84 58 L74 58 L74 48 L54 48 L54 58 L44 58 Z"
    chest = "M38 64 L52 58 L76 58 L90 64 L92 96 L36 96 Z"
    return (halo(64, 64, 46, "#ffd27a", 0.35) + cast_shadow(chest, 3, 4) + paint(chest, "steel", hl=(54, 70, 8, 8), extra='<path d="M64 60 L64 96" stroke="#2a303c" stroke-width="0.8"/>')
            + paint("M38 64 L52 58 L54 66 L40 72 Z", "brass") + paint("M90 64 L76 58 L74 66 L88 72 Z", "brass")
            + cast_shadow(helm, 2, 3) + paint(helm, "steel", hl=(56, 32, 8, 4)) + paint("M62 22 L66 22 L66 40 L62 40 Z", "brass")
            + f'<path d="M54 48 L74 48" stroke="#11141b" stroke-width="1.6"/>' + gem(64, 78, 4))


@reg("misc_party", "Party", "dusk", 14)
def party():
    return (halo(64, 70, 46, "#ffd27a", 0.3) + figure(36, 104, 40, rim="#cfe0ff") + figure(92, 104, 40, rim="#cfe0ff") + figure(64, 100, 52, rim="#ffd27a")
            + f'<path d="M30 52 Q64 30 98 52" fill="none" stroke="#ffd27a" stroke-opacity="0.5" stroke-width="1" stroke-dasharray="2 2"/>')


@reg("misc_trade", "Trade", "gold", 15)
def trade():
    out = [ring("gold", op=0.5), halo(64, 60, 40, "#ffd27a", 0.4)]
    for x, y in ((50, 82), (58, 76), (46, 72), (54, 66)):
        out.append(paint(ell_d(x, y, 10, 4), "brass", hl=(x - 3, y - 1.6, 4, 1)) + f'<ellipse cx="{x}" cy="{y}" rx="7" ry="2.6" fill="none" stroke="#7a5a1a" stroke-width="0.4"/>')
    out.append(f'<g transform="rotate(-14 84 56)">' + paint("M70 40 L98 40 L98 72 L70 72 Z", "parch", hl=(80, 48, 5, 5)) + paint("M70 40 L98 40 L98 44 L70 44 Z", "cloth_red", rim=False) + "</g>")
    out.append(f'<path d="M30 40 C40 30 56 30 64 36 M64 36 l-6 -2 M64 36 l-3 5" fill="none" stroke="#fff2d0" stroke-width="1.4" stroke-linecap="round"/>'
               f'<path d="M98 92 C88 102 72 102 64 96 M64 96 l6 2 M64 96 l3 -5" fill="none" stroke="#fff2d0" stroke-width="1.4" stroke-linecap="round"/>')
    return "".join(out)


@reg("misc_search", "Search", "azure", 16, False)
def search():
    out = [halo(64, 60, 46, "#40e0ff", 0.3)]
    for k in range(4):
        y = 30 + k * 16
        out.append(paint(f"M28 {y} L96 {y} L96 {y+11} L28 {y+11} Z", "parch", rim=False) + f'<circle cx="35" cy="{y+5.5}" r="3" fill="#5a6a9a"/><path d="M42 {y+5.5} L{70 + (k%2)*12} {y+5.5}" stroke="#5a4020" stroke-width="1"/>')
    out.append(f'<g transform="rotate(40 74 62)">' + paint("M71 84 L77 84 L77 108 L71 108 Z", "wood") + "</g>")
    out.append(f'<circle cx="74" cy="62" r="18" fill="#d8faff" fill-opacity="0.2"/>' + paint(circle_d(74, 62, 18) + " " + circle_d(74, 62, 15), "brass") + blob(68, 56, 5, 3, "#ffffff", 0.5, "b1", -30))
    return "".join(out)


@reg("misc_synthesis", "Synthesis", "fire", 17)
def synthesis():
    anvil = "M30 78 L98 78 C98 84 88 86 80 86 L76 96 L88 100 L40 100 L52 96 L48 86 C40 86 30 84 30 78 Z"
    out = [ring("fire", op=0.6), cast_shadow(anvil, 3, 3), paint(anvil, "iron", hl=(50, 80, 14, 2))]
    out.append(halo(64, 48, 30, "#ffb060", 0.6))
    d = "M64 22 L76 46 L64 72 L52 46 Z"
    out.append(paint(d, dict(base="#ff8a40", lo="#4a1004", hi="#fff0c0", rim="#ffd080", outline="#2a0802", tex="mottle"), hl=(60, 40, 4, 8)) + f'<path d="M64 22 L64 72 M52 46 L76 46" stroke="#fff0c0" stroke-opacity="0.6" stroke-width="0.6"/>' + sparkle(64, 34, 6))
    out.append(motes(17, 16, (30, 20, 98, 80), ["#ffd27a", "#ff9a4a", "#ffffff"], (0.4, 1.0)))
    return "".join(out)


@reg("misc_missions", "Missions", "blood", 18, False)
def missions():
    env = "M26 36 L102 36 L102 92 L26 92 Z"
    out = [halo(64, 62, 46, "#ff7070", 0.3), cast_shadow(env, 3, 4), paint(env, "parch", hl=(46, 50, 14, 8))]
    out.append(f'<path d="M26 36 L64 66 L102 36" fill="none" stroke="#8a7048" stroke-width="1.2"/><path d="M26 92 L54 62 M102 92 L74 62" stroke="#8a7048" stroke-opacity="0.6" stroke-width="0.8"/>')
    out.append(paint(circle_d(64, 66, 11), dict(base="#b03a32", lo="#3a0a08", hi="#ff9a80", rim="#ffc0b0", outline="#1a0404", tex="mottle"), hl=(60, 62, 4, 3)))
    out.append(f'<path d="M64 59 L66 64 L71 64 L67 67 L69 72 L64 69 L59 72 L61 67 L57 64 L62 64 Z" fill="#5a0e0a" fill-opacity="0.8"/>')
    out.append(paint("M56 76 L60 76 L58 96 L52 92 Z", "cloth_red", rim=False) + paint("M68 76 L72 76 L76 92 L70 96 Z", "cloth_red", rim=False))
    return "".join(out)


@reg("misc_quests", "Quests", "gold", 19)
def quests():
    return (halo(64, 60, 44, "#ffd27a", 0.4) + scroll(64, 62, 44, 46, "parch", None, -6, True)
            + '<g transform="rotate(-6 64 62)">' + paint("M94 34 L102 34 L100 62 L96 62 Z", "cloth_red") + paint(circle_d(98, 70, 4), "cloth_red") + "</g>")


@reg("misc_keyitems", "Key Items", "gold", 20)
def keyitems():
    bow = circle_d(42, 52, 14) + " " + circle_d(42, 52, 8)
    shaft = "M54 49 L104 49 L104 55 L54 55 Z"
    teeth = "M88 55 L94 55 L94 66 L88 66 Z M98 55 L104 55 L104 62 L98 62 Z"
    return (ring("gold", op=0.5) + halo(64, 56, 44, "#ffd27a", 0.45) + f'<g transform="rotate(-24 64 56)">' + cast_shadow(shaft, 2, 3) + paint(bow, "brass", hl=(36, 44, 5, 3)) + paint(shaft, "brass", hl=(80, 50, 12, 1))
            + paint(teeth, "brass", rim=False) + gem(42, 52, 5.4, ("#40c0e0", "#06202a", "#d0f8ff")) + "</g>" + sparkle(40, 44, 5))


@reg("misc_fishing", "Fishing", "water", 21)
def _fish():
    return ""


def build():
    from base import make
    for k, (title, pal, fn, seed, hills) in ICONS.items():
        if k == "misc_fishing":
            continue
        make(k, title, pal, fn, seed=seed, hills=hills)


if __name__ == "__main__":
    build()
    print("ok")
