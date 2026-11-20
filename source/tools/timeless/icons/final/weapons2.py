"""Weapon skill icons: one painted weapon per skill type, finishing a critical cross-slash (the approved Sword WS scene)."""
from prims import *  # noqa


def haft(y0, y1, w=3.2, mat="wood"):
    d = f"M{-w/2:.1f} {y0:.1f} L{w/2:.1f} {y0:.1f} L{w/2:.1f} {y1:.1f} L{-w/2:.1f} {y1:.1f} Z"
    return paint(d, mat, hl=(-w * 0.2, (y0 + y1) / 2, w * 0.2, abs(y1 - y0) * 0.3), off=(0.6, 0.8))


def grip_wrap(y0, y1, w, col="#2a1408"):
    return "".join(f'<path d="M{-w/2:.1f} {y:.1f} L{w/2:.1f} {y+1.5:.1f}" stroke="{col}" stroke-width="0.5"/>' for y in [y0 + k * 2.4 for k in range(int((y1 - y0) / 2.4))])


def W_dagger():
    b = blade_d(40, 8, 0.3)
    return (paint(b, "steel", hl=(-1.5, -22, 2, 10)) + f'<path d="M0 -2 L0 -34" stroke="#5a6478" stroke-width="1"/>'
            + paint("M-9 -1 L9 -1 L7 2.4 L-7 2.4 Z", "brass") + paint("M-2 2 L2 2 L2 16 L-2 16 Z", "leather", rim=False) + grip_wrap(3, 16, 4)
            + paint(circle_d(0, 17.5, 2.8), "brass"))


def W_sword():
    return sword(0, 0, 76, 0, 7.5)


def W_greatsword():
    b = blade_d(78, 14, 0.12)
    return (paint(b, "steel", hl=(-3, -40, 3, 20), extra='<path d="M0 -4 L0 -66" stroke="#5a6478" stroke-width="2.6"/><path d="M0 -4 L0 -66" stroke="#c8d0dc" stroke-opacity="0.5" stroke-width="0.6" transform="translate(-1 0)"/>')
            + f'<path d="M7 0 L7 -68.6 L0 -78" fill="none" stroke="#f4f8ff" stroke-opacity="0.7" stroke-width="0.6"/>'
            + paint("M-20 -2 Q0 -6 20 -2 L18 3 Q0 0 -18 3 Z", "iron") + paint("M-3 3 L3 3 L3 26 L-3 26 Z", "leather", rim=False) + grip_wrap(4, 26, 6)
            + paint(circle_d(0, 28.5, 4), "iron") + gem(0, 0.5, 3.2, ("#ff6a3a", "#5a0e04", "#ffc0a0")))


def W_axe():
    head = "M-2 -58 C-14 -64 -22 -56 -24 -44 C-22 -36 -14 -30 -2 -38 Z"
    return (haft(-64, 14, 4) + paint(head, "iron", hl=(-16, -52, 4, 4)) + f'<path d="M-24 -44 C-22 -36 -14 -30 -4 -36" fill="none" stroke="#f4f8ff" stroke-opacity="0.7" stroke-width="0.7"/>'
            + paint("M-3 -60 L3 -60 L3 -36 L-3 -36 Z", "iron") + paint(circle_d(0, 15, 2.6), "iron"))


def W_greataxe():
    head = lambda sx: f"M{sx*2} -66 C{sx*18} -76 {sx*28} -64 {sx*30} -48 C{sx*28} -34 {sx*18} -26 {sx*2} -36 Z"
    return (haft(-72, 22, 4.4) + paint(head(-1), "iron", hl=(-18, -58, 5, 5)) + paint(head(1), "iron", hl=(14, -58, 5, 5))
            + "".join(f'<path d="M{sx*30} -48 C{sx*28} -34 {sx*18} -26 {sx*4} -34" fill="none" stroke="#f4f8ff" stroke-opacity="0.7" stroke-width="0.7"/>' for sx in (-1, 1))
            + paint("M-3.4 -72 L0 -80 L3.4 -72 Z", "steel") + paint(circle_d(0, 23, 3), "iron"))


def W_scythe():
    blade = "M-2 -66 C-20 -70 -42 -60 -54 -40 C-40 -52 -22 -58 -2 -58 Z"
    return (haft(-70, 26, 3.6) + paint(blade, "steel", hl=(-24, -62, 8, 2)) + f'<path d="M-54 -40 C-40 -52 -22 -58 -4 -58" fill="none" stroke="#f4f8ff" stroke-opacity="0.8" stroke-width="0.7"/>'
            + paint("M-4 -72 L4 -72 L4 -56 L-4 -56 Z", "iron") + paint("M-5 -12 L5 -12 L5 -6 L-5 -6 Z", "brass"))


def W_polearm():
    tip = "M0 -96 L6 -74 L3 -64 L-3 -64 L-6 -74 Z"
    return (haft(-64, 26, 3.4) + paint(tip, "steel", hl=(-1.5, -82, 1.5, 8)) + f'<path d="M0 -96 L0 -66" stroke="#5a6478" stroke-width="0.8"/>'
            + paint("M-10 -66 L10 -66 L8 -62 L-8 -62 Z", "brass") + f'<path d="M-2 -62 C-8 -58 -10 -50 -6 -44 M2 -62 C8 -58 10 -50 6 -44" fill="none" stroke="#c83a2a" stroke-width="2" stroke-linecap="round"/>'
            + paint(circle_d(0, 27, 2.4), "brass"))


def W_katana(L=72):
    blade = f"M-2.6 0 C-3.6 {-L*0.4:.1f} -2 {-L*0.8:.1f} 3 {-L:.1f} C2.4 {-L*0.8:.1f} 3 {-L*0.4:.1f} 2.6 0 Z"
    hamon = f'<path d="M1.6 -2 C2 {-L*0.3:.1f} 1.2 {-L*0.6:.1f} 2.6 {-L*0.95:.1f}" fill="none" stroke="#f4f8ff" stroke-opacity="0.8" stroke-width="0.6" stroke-dasharray="2 1"/>'
    return (paint(blade, "silver", hl=(-1, -L * 0.5, 1.4, L * 0.3), extra=hamon)
            + paint(ell_d(0, 1, 6.4, 2.6), "brass") + paint(f"M-2.4 2.6 L2.4 2.6 L2.4 {L*0.32:.1f} L-2.4 {L*0.32:.1f} Z", "cloth_blue", rim=False)
            + "".join(f'<path d="M-2.4 {y:.1f} L2.4 {y+2:.1f} M2.4 {y:.1f} L-2.4 {y+2:.1f}" stroke="#d8dce8" stroke-width="0.5"/>' for y in [4 + k * 3 for k in range(int(L * 0.32 / 3) - 1)])
            + paint(f"M-2.6 {L*0.32:.1f} L2.6 {L*0.32:.1f} L2.6 {L*0.32+2.4:.1f} L-2.6 {L*0.32+2.4:.1f} Z", "brass"))


def W_greatkatana():
    return W_katana(92)


def W_club():
    head = "M0 -66 C10 -66 13 -58 13 -50 C13 -42 8 -38 0 -38 C-8 -38 -13 -42 -13 -50 C-13 -58 -10 -66 0 -66 Z"
    fl = "".join(f'<path d="M0 -52 L{13*math.cos(math.radians(a)):.1f} {-52+13*math.sin(math.radians(a)):.1f}" stroke="#1c1c22" stroke-width="0.8"/>' for a in range(0, 360, 45))
    spikes = "".join(paint(f"M{x-2.4:.1f} {y:.1f} L{x+2*math.cos(math.radians(a)):.1f}{'' } {y:.1f} Z".replace(" Z", f" L{x:.1f} {y+0.1:.1f} Z"), "steel", rim=False) for x, y, a in ())
    return (haft(-40, 24, 4) + paint(head, "iron", hl=(-5, -58, 4, 4), extra=fl)
            + "".join(paint(f"M{x-2.2:.1f} {y-1:.1f} L{x+(x>0 and 6 or -6):.1f} {y:.1f} L{x-2.2:.1f} {y+1:.1f} Z" if False else f"M{x:.1f} {y-2.4:.1f} L{x + 5*math.cos(math.radians(a)):.1f} {y + 5*math.sin(math.radians(a)):.1f} L{x:.1f} {y+2.4:.1f} Z", "steel", rim=False)
                      for x, y, a in ((12, -52, 0), (-12, -52, 180), (0, -66, 270), (8, -62, 315), (-8, -62, 225)))
            + paint(circle_d(0, 25, 2.8), "iron") + grip_wrap(8, 22, 4))


def W_staff():
    return (haft(-70, 34, 3.6) + f'<path d="M0 -70 C-10 -74 -12 -86 -2 -90 C8 -92 12 -82 6 -76" fill="none" stroke="#5a3a1a" stroke-width="3.4" stroke-linecap="round"/>'
            + orb(1, -80, 6, "#ffffff", "#7ad8ff", "#0a2a4a", glow="#7ad8ff") + paint("M-3 -64 L3 -64 L3 -60 L-3 -60 Z", "brass"))


def W_h2h():
    blade = "M0 -46 L8 -14 L-8 -14 Z"
    frame = "M-11 -14 L11 -14 L11 -10 L4 -10 L4 12 L11 12 L11 16 L-11 16 L-11 12 L-4 12 L-4 -10 L-11 -10 Z"
    return (paint(blade, "steel", hl=(-2, -26, 2, 8)) + f'<path d="M0 -46 L0 -16" stroke="#5a6478" stroke-width="0.8"/>'
            + paint(frame, "brass", hl=(-6, -12, 4, 1)) + paint("M-4 0 L4 0 L4 3 L-4 3 Z", "leather", rim=False))


def W_archery():
    limb = lambda sy: f"M-2 0 C-2 {sy*18} -10 {sy*34} -22 {sy*44} L-20 {sy*46} C-6 {sy*36} 2 {sy*18} 2 0 Z"
    return (paint(limb(-1), "wood", hl=(-6, -24, 2, 8)) + paint(limb(1), "wood", hl=(-6, 24, 2, 8))
            + '<path d="M-21 -45 L14 0 L-21 45" fill="none" stroke="#f0e8d0" stroke-width="0.6"/>'
            + paint("M-3 -6 L3 -6 L3 6 L-3 6 Z", "leather", rim=False)
            + paint("M14 -0.8 L-46 -0.8 L-46 0.8 L14 0.8 Z", "wood", rim=False) + paint("M-46 -3 L-56 0 L-46 3 Z", "steel")
            + feather(17, 0, 10, 90, "#e05040", "#ffb0a0"))


def W_marksmanship():
    stock = "M30 4 L54 2 C58 2 60 6 58 10 L44 14 C38 14 34 12 30 10 Z"
    barrel = "M-56 -2.4 L30 -2.4 L30 2.4 L-56 2.4 Z"
    return (paint(barrel, "iron", hl=(-20, -1.6, 20, 0.6)) + paint("M-6 -4 L32 -4 L32 6 L-6 6 Z", "wood", hl=(10, -2, 10, 1)) + paint(stock, "wood", hl=(46, 5, 6, 2))
            + paint("M18 6 L22 6 L20 12 Z", "brass") + paint(circle_d(16, 8, 3.4) + " " + circle_d(16, 8, 2), "brass")
            + paint("M-56 -3.6 L-52 -3.6 L-52 3.6 L-56 3.6 Z", "brass") + paint("M24 -6 L28 -6 L28 -3 L24 -3 Z", "iron"))


WS = {  # key: (title, weapon fn, rotation, scale, bg (inner, mid, outer), slash colour)
    "ws_dagger": ("Dagger", W_dagger, 135, 1.2, ("#3a1a4a", "#180a20", "#05020a"), "#c070ff"),
    "ws_greatsword": ("Great Sword", W_greatsword, 135, 0.95, ("#4a2a0a", "#1e1006", "#070402"), "#ff9a30"),
    "ws_axe": ("Axe", W_axe, 130, 1.0, ("#4a1a0a", "#200a06", "#070202"), "#ff6a2a"),
    "ws_greataxe": ("Great Axe", W_greataxe, 130, 0.85, ("#5a2a0a", "#241006", "#080402"), "#ff8a20"),
    "ws_scythe": ("Scythe", W_scythe, 140, 0.95, ("#2a0a2a", "#120412", "#040104"), "#e040a0"),
    "ws_polearm": ("Polearm", W_polearm, 135, 0.85, ("#0a2a4a", "#061428", "#010408"), "#40a0ff"),
    "ws_katana": ("Katana", W_katana, 135, 1.0, ("#2a0a10", "#14040a", "#050102"), "#ff4060"),
    "ws_greatkatana": ("Great Katana", W_greatkatana, 135, 0.85, ("#3a0a0a", "#180404", "#060101"), "#ff3030"),
    "ws_club": ("Club", W_club, 130, 1.05, ("#3a3a0a", "#181806", "#060602"), "#f0e040"),
    "ws_staff": ("Staff", W_staff, 140, 0.95, ("#0a3a3a", "#061a1a", "#010606"), "#40e0e0"),
    "ws_h2h": ("Hand-to-Hand", W_h2h, 150, 1.3, ("#4a2a1a", "#20100a", "#080403"), "#ffb060"),
    "ws_archery": ("Archery", W_archery, 225, 1.05, ("#1a3a1a", "#0a1a0a", "#020602"), "#80e060"),
    "ws_marksmanship": ("Marksmanship", W_marksmanship, 205, 1.0, ("#3a2a1a", "#1a120a", "#060403"), "#ffd060"),
}


def ws_icon(key):
    title, fn, rot, k, bg, sl = WS[key]
    reset()
    g, sg = uid("wsbg"), uid("slash")
    defs = (rg(g, [(0, bg[0]), (0.45, bg[1]), (1, bg[2])], cx="58%", cy="55%", r="75%")
            + lg(sg, [(0, sl, 0), (0.3, sl, 0.85), (0.65, "#ffe6c0", 1), (1, "#ffffff", 0.2)], 0, 0, 1, 1))
    sc = [f'<rect width="128" height="128" fill="url(#{g})"/>', halo(66, 64, 52, sl, 0.35), blob(30, 100, 30, 10, "#000", 0.6, "b5")]
    b = []
    rnd = random.Random(len(key))
    for _ in range(14):
        x, y = rnd.uniform(10, 90), rnd.uniform(10, 90)
        L = rnd.uniform(10, 26)
        b.append(f'<path d="M{x:.1f} {y:.1f} l{L*0.74:.1f} {L*0.67:.1f}" stroke="#ffe0c0" stroke-opacity="{rnd.uniform(0.12,0.35):.2f}" stroke-width="{rnd.uniform(0.4,1.0):.2f}" stroke-linecap="round"/>')
    b.append(f'<path d="M6 18 C40 28 80 56 122 112 C84 74 44 46 6 18 Z" fill="{sl}" fill-opacity="0.55" filter="url(#b5)"/>')
    b.append(f'<path d="M10 22 C40 30 76 54 118 108 C84 70 46 44 10 22 Z" fill="url(#{sg})"/>')
    b.append(f'<path d="M14 24 C44 34 78 58 114 102 C80 68 46 44 14 24 Z" fill="#ffffff" fill-opacity="0.9" filter="url(#b06)"/>')
    b.append(f'<path d="M112 20 C86 40 56 70 22 112 C52 74 82 44 112 20 Z" fill="url(#{sg})" fill-opacity="0.75"/>')
    cx, cy = 66, 63
    b.append(halo(cx, cy, 26, sl, 0.8) + halo(cx, cy, 14, "#ffffff", 0.9))
    for i in range(16):
        a = math.radians(i * 22.5 + 8)
        L, w = (28, 2.8) if i % 2 == 0 else (16, 1.8)
        ax_, ay = math.cos(a), math.sin(a)
        px, py = -ay * w, ax_ * w
        b.append(f'<path d="M{cx+px:.1f} {cy+py:.1f} L{cx+ax_*L:.1f} {cy+ay*L:.1f} L{cx-px:.1f} {cy-py:.1f} Z" fill="#fff4d0" fill-opacity="{0.85 if i%2==0 else 0.6}" filter="url(#b06)"/>')
    for _ in range(22):
        a = rnd.uniform(0, 2 * math.pi); r0 = rnd.uniform(10, 18); r1 = r0 + rnd.uniform(6, 20)
        b.append(f'<path d="M{cx+math.cos(a)*r0:.1f} {cy+math.sin(a)*r0:.1f} L{cx+math.cos(a)*r1:.1f} {cy+math.sin(a)*r1:.1f}" stroke="{rnd.choice(["#ffd27a","#ffffff",sl])}" stroke-width="{rnd.uniform(0.4,1.0):.2f}" stroke-linecap="round"/>')
    b.append(f'<g transform="translate(64 64) rotate({rot}) scale({k}) translate(0 18)">{fn()}</g>')
    b.append(sparkle(cx, cy, 8, "#ffffff"))
    return icon(f"Weapon Skill - {title}", "".join(sc), "".join(b), defs, accent="#f0d0b0")


def build():
    for k in WS:
        save(k, ws_icon(k))


if __name__ == "__main__":
    build()
    print("ok")
