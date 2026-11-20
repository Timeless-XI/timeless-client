"""Job ability icons: one emblem per job (the fallback), two-hour variants, and unique art for the most-slotted abilities."""
from prims import *  # noqa
from weapons2 import W_dagger, W_greataxe, W_axe, W_scythe, W_polearm, W_katana, W_h2h, W_archery, W_marksmanship, W_staff, W_club, W_greatsword
from spellset import cure_core
from elements import light_core, dark_core, thunder_core
from misc2 import figure, reticle

JOBS = {1: "war", 2: "mnk", 3: "whm", 4: "blm", 5: "rdm", 6: "thf", 7: "pld", 8: "drk", 9: "bst", 10: "brd", 11: "rng", 12: "sam", 13: "nin",
        14: "drg", 15: "smn", 16: "blu", 17: "cor", 18: "pup"}


def placed(fn, x=64, y=62, rot=0, k=1.0, dy=0):
    return f'<g transform="translate({x} {y}) rotate({rot}) scale({k}) translate(0 {dy})">{fn()}</g>'


def die(cx, cy, s, n=5, rot=0):
    top = f"M{cx} {cy-s:.1f} L{cx+s*0.9:.1f} {cy-s*0.5:.1f} L{cx} {cy:.1f} L{cx-s*0.9:.1f} {cy-s*0.5:.1f} Z"
    left = f"M{cx-s*0.9:.1f} {cy-s*0.5:.1f} L{cx} {cy:.1f} L{cx} {cy+s:.1f} L{cx-s*0.9:.1f} {cy+s*0.5:.1f} Z"
    right = f"M{cx+s*0.9:.1f} {cy-s*0.5:.1f} L{cx} {cy:.1f} L{cx} {cy+s:.1f} L{cx+s*0.9:.1f} {cy+s*0.5:.1f} Z"
    bone = MAT["bone"]
    out = [f'<g transform="rotate({rot} {cx} {cy})">', paint(left, dict(bone, base="#b8ae94")), paint(right, dict(bone, base="#988e74")), paint(top, bone)]
    pip = lambda x, y: f'<ellipse cx="{x:.1f}" cy="{y:.1f}" rx="{s*0.1:.1f}" ry="{s*0.07:.1f}" fill="#a83a32"/>'
    out.append(pip(cx, cy - s * 0.5))
    out.append(pip(cx - s * 0.45, cy + s * 0.25) + pip(cx - s * 0.45, cy + s * 0.55) if n > 1 else "")
    out.append(pip(cx + s * 0.3, cy + s * 0.05) + pip(cx + s * 0.45, cy + s * 0.5) + pip(cx + s * 0.6, cy + s * 0.75))
    out.append("</g>")
    return "".join(out)


def cog(cx, cy, r, teeth=10, mat="brass"):
    pts = []
    for i in range(teeth * 2):
        a = math.radians(i * 180 / teeth)
        rr = r if i % 2 == 0 else r * 0.8
        a2 = math.radians((i + 1) * 180 / teeth)
        pts += [(cx + rr * math.cos(a), cy + rr * math.sin(a)), (cx + rr * math.cos(a2), cy + rr * math.sin(a2))]
    d = poly(pts) + " Z " + circle_d(cx, cy, r * 0.36)
    return paint(d, mat, hl=(cx - r * 0.3, cy - r * 0.4, r * 0.3, r * 0.2))


def shuriken(cx, cy, r, rot=0):
    pts = []
    for i in range(8):
        a = math.radians(rot + i * 45)
        rr = r if i % 2 == 0 else r * 0.32
        pts.append((cx + rr * math.cos(a), cy + rr * math.sin(a)))
    return paint(poly(pts) + " Z " + circle_d(cx, cy, r * 0.12), "steel", hl=(cx - r * 0.3, cy - r * 0.3, r * 0.3, r * 0.2))


def paw(cx, cy, s, col="#2a1a10"):
    out = [paint(ell_d(cx, cy + s * 0.4, s * 0.55, s * 0.45), dict(MAT["leather"], base="#6a4a30"))]
    for dx, dy in ((-0.6, -0.35), (-0.22, -0.7), (0.22, -0.7), (0.6, -0.35)):
        out.append(paint(ell_d(cx + dx * s, cy + dy * s, s * 0.2, s * 0.26), dict(MAT["leather"], base="#6a4a30")))
    return "".join(out)


def heart(cx, cy, s, col="#e0506c"):
    d = f"M{cx} {cy+s:.1f} C{cx-s*1.4:.1f} {cy:.1f} {cx-s:.1f} {cy-s:.1f} {cx} {cy-s*0.4:.1f} C{cx+s:.1f} {cy-s:.1f} {cx+s*1.4:.1f} {cy:.1f} {cx} {cy+s:.1f} Z"
    return paint(d, dict(base=col, lo="#3a0812", hi="#ffd0dc", rim="#ffb0c0", outline="#1a0408", tex="mottle"), hl=(cx - s * 0.4, cy - s * 0.4, s * 0.3, s * 0.2))


def waves(cx, cy, col, n=3, a0=-40, a1=40, r0=20):
    return "".join(f'<path d="M{cx+ (r0+k*9)*math.cos(math.radians(a0)):.1f} {cy+(r0+k*9)*math.sin(math.radians(a0)):.1f} A{r0+k*9} {r0+k*9} 0 0 1 {cx+(r0+k*9)*math.cos(math.radians(a1)):.1f} {cy+(r0+k*9)*math.sin(math.radians(a1)):.1f}" fill="none" stroke="{col}" stroke-opacity="{0.8-k*0.2:.2f}" stroke-width="{2.4-k*0.5:.1f}" stroke-linecap="round"/>' for k in range(n))


def aura(col, cx=64, cy=62, r=40):
    return halo(cx, cy, r, col, 0.6) + "".join(
        f'<path d="M{cx+ (r*0.5)*math.cos(math.radians(a)):.1f} {cy+(r*0.5)*math.sin(math.radians(a)):.1f} L{cx+r*math.cos(math.radians(a)):.1f} {cy+r*math.sin(math.radians(a)):.1f}" stroke="{col}" stroke-opacity="0.45" stroke-width="2" filter="url(#b1)"/>' for a in range(0, 360, 20))


# ---------------------------------------------------------------- job emblems
EMB = {
    "war": ("blood", lambda: placed(W_greataxe, 64, 66, 0, 0.75, 14)),
    "mnk": ("gold", lambda: placed(W_h2h, 64, 64, 0, 1.3, 10)),
    "whm": ("heal", lambda: cure_core(64, 60, 1.0, 3, 1)),
    "blm": ("dark", lambda: orb(64, 60, 18, "#ffb070", "#3a1050", "#05010a", glow="#ff7a30") + wisps(64, 60, 4, 3, 26, "#ff7a30", "#ffd8a0", 200, 1.6)),
    "rdm": ("blood", lambda: sword(64, 92, 84, 0, 5, ("#ff6060", "#4a0a0a", "#ffc0c0")) + feather(84, 40, 26, 30, "#f0f0f0")),
    "thf": ("dark", lambda: placed(W_dagger, 64, 62, 30, 1.3, 10)),
    "pld": ("night", lambda: shield(64, 60, 24, 34, "cloth_blue", "silver", gem(64, 56, 6, ("#ffffff", "#4a5262", "#ffffff")))),
    "drk": ("dark", lambda: placed(W_scythe, 70, 64, 10, 0.85, 20)),
    "bst": ("earth", lambda: paw(64, 60, 22)),
    "brd": ("night", lambda: lyre(64, 60, 1.3)),
    "rng": ("wind", lambda: placed(W_archery, 66, 62, -20, 1.1)),
    "sam": ("blood", lambda: placed(W_katana, 64, 64, 30, 1.0, 22)),
    "nin": ("night", lambda: shuriken(64, 60, 26, 10)),
    "drg": ("azure", lambda: placed(W_polearm, 64, 66, 0, 0.75, 20) + eye(64, 30, 10, "#ff6030")),
    "smn": ("gold", lambda: paint("M64 30 L84 58 L64 86 L44 58 Z", "brass") + gem(64, 58, 13, ("#ff4060", "#4a0010", "#ffc0d0"))),
    "blu": ("azure", lambda: eye(64, 60, 24, "#40e0ff")),
    "cor": ("gold", lambda: die(52, 62, 16, 5, -8) + die(80, 66, 12, 3, 12)),
    "pup": ("earth", lambda: cog(56, 56, 20) + cog(82, 76, 13, 8, "iron")),
}

TITLES = {"war": "Warrior", "mnk": "Monk", "whm": "White Mage", "blm": "Black Mage", "rdm": "Red Mage", "thf": "Thief", "pld": "Paladin",
          "drk": "Dark Knight", "bst": "Beastmaster", "brd": "Bard", "rng": "Ranger", "sam": "Samurai", "nin": "Ninja", "drg": "Dragoon",
          "smn": "Summoner", "blu": "Blue Mage", "cor": "Corsair", "pup": "Puppetmaster"}


def emblem_body(job, two_hour=False):
    pal, fn = EMB[job]
    out = [ring(pal, op=0.6)]
    if two_hour:
        out.append(aura("#ffd27a", 64, 60, 52))
    else:
        out.append(halo(64, 60, 44, PAL[pal]["acc"], 0.35))
    out.append(fn())
    if two_hour:
        out.append(f'<circle cx="64" cy="60" r="46" fill="none" stroke="#ffd27a" stroke-width="2.4" filter="url(#b1)"/><circle cx="64" cy="60" r="46" fill="none" stroke="#fff2d0" stroke-width="0.6"/>')
        out.append(motes(5, 20, (16, 10, 112, 100), ["#ffd27a", "#fff2d0"], (0.5, 1.2)))
    return "".join(out)


# ---------------------------------------------------------------- unique abilities
U = {
    "provoke": ("blood", lambda: figure(64, 98, 56, rim="#ff6060") + waves(64, 50, "#ff6060", 3, -60, 60, 18) + waves(64, 50, "#ff6060", 3, 120, 240, 18)),
    "berserk": ("blood", lambda: aura("#ff3020") + placed(W_greataxe, 64, 66, 20, 0.75, 14)),
    "defender": ("night", lambda: halo(64, 60, 44, "#8fb2e6", 0.6) + shield(64, 60, 26, 36, "cloth_blue", "iron", gem(64, 56, 6))),
    "warcry": ("gold", lambda: figure(48, 100, 50, rim="#ffd27a") + waves(56, 56, "#ffd27a", 4, -50, 50, 14)),
    "aggressor": ("blood", lambda: eye(64, 60, 26, "#ff4030") + aura("#ff3020", r=46)),
    "boost": ("fire", lambda: aura("#ff9a30") + placed(W_h2h, 64, 64, 0, 1.3, 10)),
    "focus": ("gold", lambda: eye(64, 60, 24, "#ffd27a") + reticle(64, 60, 32, "#ffd27a", "#fff2d0", 1.4)),
    "dodge": ("wind", lambda: "".join(f'<g opacity="{o}">{figure(x, 98, 54, rim="#8ff0c0")}</g>' for x, o in ((44, 0.25), (54, 0.5))) + figure(66, 98, 54, rim="#8ff0c0")),
    "chakra": ("gold", lambda: cure_core(64, 60, 1.05, 4, 2).replace("#6ae0a0", "#ffb060").replace("#d8ffe8", "#fff2d0").replace("#b8ffd8", "#ffd27a")),
    "divine_seal": ("light", lambda: light_core(64, 60, 0.9, 3, 1) + f'<circle cx="64" cy="60" r="36" fill="none" stroke="#ffe6a0" stroke-width="1.6"/><circle cx="64" cy="60" r="30" fill="none" stroke="#ffe6a0" stroke-width="0.6" stroke-dasharray="2 2"/>'),
    "elemental_seal": ("dark", lambda: f'<circle cx="64" cy="60" r="34" fill="none" stroke="#e6ecf5" stroke-opacity="0.5" stroke-width="1"/>' + "".join(orb(64 + 30 * math.cos(math.radians(a)), 60 + 30 * math.sin(math.radians(a)), 6, "#ffffff", ELC[e][0], "#100810") for a, e in zip(range(-90, 270, 45), ["fire", "ice", "wind", "earth", "thunder", "water", "light", "dark"])) + gem(64, 60, 9, ("#e6ecf5", "#2a2a3a", "#ffffff"))),
    "convert": ("dusk", lambda: orb(50, 60, 14, "#ffffff", "#e0606c", "#2a0408", glow="#e0606c") + orb(78, 60, 14, "#ffffff", "#5f9be6", "#04102a", glow="#5f9be6") + wisps(64, 60, 3, 2, 26, "#e6ecf5", "#ffffff", 180, 1.4)),
    "steal": ("dark", lambda: paint("M44 60 C44 44 84 44 84 60 C90 70 88 92 64 94 C40 92 38 70 44 60 Z", "leather", hl=(56, 64, 8, 8)) + paint("M52 52 L76 52 L74 46 L54 46 Z", "brass") + "".join(paint(ell_d(x, y, 6, 2.4), "brass") for x, y in ((90, 40), (96, 50), (86, 30)))),
    "sneak_attack": ("dark", lambda: figure(76, 100, 56, rim="#c080ff") + placed(W_dagger, 40, 60, 40, 1.2, 10)),
    "trick_attack": ("dark", lambda: figure(48, 100, 48, rim="#8fb2e6") + f'<g opacity="0.55">{figure(80, 98, 40, "#2a1a3a", "#c080ff")}</g>' + placed(W_dagger, 90, 46, 120, 0.9)),
    "flee": ("wind", lambda: figure(76, 100, 52, rim="#8ff0c0") + "".join(f'<path d="M{20+k*4} {40+k*14} l26 0" stroke="#e8fff2" stroke-opacity="0.6" stroke-width="1.2" stroke-linecap="round"/>' for k in range(5))),
    "hide": ("night", lambda: f'<g opacity="0.35">{figure(64, 98, 56, "#1a2040", "#8fb2e6")}</g>' + "".join(blob(x, y, 14, 10, "#2a3050", 0.6, "b3") for x, y in ((48, 84), (80, 80), (64, 70)))),
    "sentinel": ("night", lambda: shield(64, 62, 26, 36, "cloth_blue", "steel", eye(64, 58, 12, "#8fb2e6"))),
    "holy_circle": ("light", lambda: f'<ellipse cx="64" cy="84" rx="40" ry="10" fill="none" stroke="#ffe6a0" stroke-width="2.4" filter="url(#b06)"/>' + light_core(64, 54, 0.75, 2, 4)),
    "shield_bash": ("night", lambda: shield(58, 60, 22, 30, "cloth_blue", "iron") + "".join(f'<path d="M84 {48+k*8} l16 {-4+k*4}" stroke="#ffd27a" stroke-width="1.6" stroke-linecap="round"/>' for k in range(3)) + sparkle(86, 58, 10)),
    "cover": ("night", lambda: f'<g opacity="0.6">{figure(78, 100, 44, "#1a2040", "#8fb2e6")}</g>' + shield(52, 62, 24, 34, "cloth_blue", "silver")),
    "rampart": ("gold", lambda: halo(64, 60, 50, "#ffd27a", 0.4) + f'<path d="M20 70 Q64 10 108 70" fill="none" stroke="#ffd27a" stroke-width="3" filter="url(#b1)"/>' + shield(64, 66, 20, 28, "cloth_blue", "brass")),
    "arcane_circle": ("dark", lambda: f'<ellipse cx="64" cy="84" rx="40" ry="10" fill="none" stroke="#c080ff" stroke-width="2.4" filter="url(#b06)"/>' + dark_core(64, 54, 0.7, 2, 4)),
    "last_resort": ("blood", lambda: aura("#c02040") + placed(W_scythe, 70, 64, 10, 0.85, 20)),
    "souleater": ("blood", lambda: wisps(64, 60, 6, 4, 34, "#e04050", "#ffd0d0", 220, 2) + placed(W_scythe, 70, 64, 10, 0.8, 20)),
    "weapon_bash": ("dark", lambda: placed(W_scythe, 64, 64, -30, 0.8, 20) + sparkle(46, 46, 12)),
    "charm": ("blood", lambda: heart(64, 58, 22) + paw(92, 88, 8)),
    "reward": ("earth", lambda: paint("M40 70 C36 60 48 54 56 60 L80 52 C86 46 96 50 92 58 C98 62 94 72 86 70 L62 78 C58 86 44 84 44 76 C38 76 36 72 40 70 Z", "bone", hl=(64, 60, 10, 4))),
    "call_beast": ("earth", lambda: paint("M50 40 L78 40 L80 48 C90 54 92 70 88 88 C80 98 48 98 40 88 C36 70 38 54 48 48 Z", dict(MAT["stone"], base="#a88a60"), hl=(56, 56, 8, 10)) + paint("M52 34 L76 34 L76 42 L52 42 Z", "wood") + paw(64, 72, 10)),
    "fight": ("blood", lambda: "".join(f'<path d="M{40+k*12} 30 C{46+k*12} 54 {44+k*12} 76 {36+k*12} 96" fill="none" stroke="#ff5050" stroke-width="5" stroke-linecap="round" filter="url(#b1)"/><path d="M{40+k*12} 30 C{46+k*12} 54 {44+k*12} 76 {36+k*12} 96" fill="none" stroke="#ffd0d0" stroke-width="1.4" stroke-linecap="round"/>' for k in range(4))),
    "ready": ("earth", lambda: paw(64, 62, 24) + aura("#f0b860", r=44)),
    "scavenge": ("earth", lambda: "".join(placed(W_archery if False else (lambda: paint("M14 -0.8 L-30 -0.8 L-30 0.8 L14 0.8 Z", "wood", rim=False) + paint("M-30 -3 L-38 0 L-30 3 Z", "steel") + feather(17, 0, 9, 90, "#e05040")), x, y, r) for x, y, r in ((50, 84, -10), (70, 90, 20), (64, 80, 160), (80, 82, -40))) + figure(40, 70, 30)),
    "sharpshot": ("wind", lambda: reticle(64, 58, 30, "#8ff0c0", "#e8fff2") + placed(lambda: paint("M14 -0.8 L-40 -0.8 L-40 0.8 L14 0.8 Z", "wood", rim=False) + paint("M-40 -3 L-48 0 L-40 3 Z", "steel") + feather(17, 0, 9, 90, "#e05040"), 74, 70, 225, 1.0)),
    "barrage": ("wind", lambda: "".join(placed(lambda: paint("M14 -0.8 L-34 -0.8 L-34 0.8 L14 0.8 Z", "wood", rim=False) + paint("M-34 -3 L-42 0 L-34 3 Z", "steel") + feather(17, 0, 9, 90, "#e05040"), 70, 40 + k * 14, 200 + k * 3, 0.9) for k in range(4))),
    "camouflage": ("wind", lambda: f'<g opacity="0.45">{figure(64, 98, 56, "#123428", "#8ff0c0")}</g>' + "".join(leaf(x, y, 12, a, c) for x, y, a, c in ((44, 60, 200, "#4a8a38"), (80, 52, -20, "#6ab048"), (58, 80, 160, "#3a7a30"), (84, 76, 20, "#5a9a40"), (64, 40, -100, "#6ab048")))),
    "meditate": ("gold", lambda: halo(64, 60, 40, "#ffd27a", 0.5) + placed(W_katana, 64, 90, 90, 0.9) + cure_core(64, 52, 0.6, 2, 3).replace("#6ae0a0", "#ffb060").replace("#b8ffd8", "#ffd27a")),
    "third_eye": ("blood", lambda: eye(64, 58, 24, "#ff4060") + f'<path d="M30 26 L98 26" stroke="#ff4060" stroke-opacity="0.4" stroke-width="1"/>'),
    "hasso": ("blood", lambda: aura("#ff6040") + placed(W_katana, 64, 66, -20, 1.0, 26)),
    "seigan": ("night", lambda: halo(64, 60, 44, "#8fb2e6", 0.5) + placed(W_katana, 64, 60, 90, 1.0, 0) + shield(64, 76, 10, 14, "cloth_red", "brass")),
    "jump": ("azure", lambda: figure(64, 70, 46, rim="#40e0ff") + "".join(f'<path d="M{50+k*7} 100 l0 -18" stroke="#d8faff" stroke-opacity="0.5" stroke-width="1" stroke-linecap="round"/>' for k in range(5))),
    "high_jump": ("azure", lambda: figure(64, 56, 40, rim="#40e0ff") + "".join(f'<path d="M{50+k*7} 104 l0 -28" stroke="#d8faff" stroke-opacity="0.5" stroke-width="1" stroke-linecap="round"/>' for k in range(5)) + moon(96, 28, 8)),
    "super_jump": ("azure", lambda: f'<g opacity="0.85">{figure(64, 34, 26, rim="#40e0ff")}</g>' + "".join(f'<path d="M{48+k*8} 108 l0 -60" stroke="#d8faff" stroke-opacity="0.5" stroke-width="1.2" stroke-linecap="round"/>' for k in range(5))),
    "call_wyvern": ("azure", lambda: eye(64, 58, 22, "#ff6030") + wing(40, 58, -1, 0.8, "#6a8ab0") + wing(88, 58, 1, 0.8, "#6a8ab0")),
    "ancient_circle": ("azure", lambda: f'<ellipse cx="64" cy="84" rx="40" ry="10" fill="none" stroke="#40e0ff" stroke-width="2.4" filter="url(#b06)"/>' + eye(64, 54, 18, "#ff6030")),
    "release": ("gold", lambda: paint("M64 30 L84 58 L64 86 L44 58 Z", "brass") + gem(64, 58, 13, ("#e6ecf5", "#2a2a3a", "#ffffff")) + "".join(f'<path d="M{64+20*math.cos(math.radians(a)):.1f} {58+20*math.sin(math.radians(a)):.1f} L{64+38*math.cos(math.radians(a)):.1f} {58+38*math.sin(math.radians(a)):.1f}" stroke="#fff2d0" stroke-width="1.4" stroke-dasharray="3 2"/>' for a in range(0, 360, 45))),
    "assault": ("blood", lambda: paint("M64 30 L84 58 L64 86 L44 58 Z", "brass") + gem(64, 58, 13, ("#ff4060", "#4a0010", "#ffc0d0")) + reticle(64, 58, 30)),
    "retreat": ("night", lambda: paint("M64 30 L84 58 L64 86 L44 58 Z", "brass") + gem(64, 58, 13, ("#8fb2e6", "#0c1a4a", "#e6f0ff")) + f'<path d="M100 60 L84 60 M90 54 L84 60 L90 66" fill="none" stroke="#e6ecf5" stroke-width="2"/>'),
    "phantom_roll": ("gold", lambda: aura("#ffd27a", r=44) + die(64, 60, 22, 5, -6)),
    "double_up": ("gold", lambda: die(48, 56, 15, 5, -10) + die(80, 64, 15, 3, 10) + f'<text x="64" y="104" text-anchor="middle" font-family="Georgia,serif" font-size="12" fill="#fff2d0">x2</text>'),
    "quick_draw": ("gold", lambda: placed(W_marksmanship, 64, 64, -10, 0.95) + sparkle(16, 56, 9)),
    "random_deal": ("gold", lambda: "".join(f'<g transform="rotate({a} 64 90)">' + paint("M54 30 L74 30 L74 62 L54 62 Z", "parch", hl=(60, 38, 4, 6)) + f'<path d="M58 34 L70 34 L70 58 L58 58 Z" fill="none" stroke="#a83a32" stroke-width="0.8"/></g>' for a in (-24, 0, 24))),
    "maneuver": ("earth", lambda: cog(64, 60, 28)),
    "activate": ("earth", lambda: cog(56, 56, 20) + cog(82, 76, 13, 8, "iron") + sparkle(56, 56, 8)),
    "deploy": ("blood", lambda: cog(48, 70, 16) + reticle(84, 50, 16)),
    "repair": ("heal", lambda: cog(64, 62, 24, 10, "iron") + cure_core(64, 62, 0.45, 2, 5)),
    "overdrive": ("fire", lambda: aura("#ff9a30") + cog(64, 60, 26, 12, "iron")),
    "mug": ("dark", lambda: placed(W_dagger, 50, 64, -30, 1.1) + "".join(paint(ell_d(x, y, 6, 2.4), "brass") for x, y in ((84, 44), (90, 54), (80, 34)))),
    "feint": ("dark", lambda: f'<g opacity="0.45">{placed(W_dagger, 50, 60, 30, 1.2, 10)}</g>' + placed(W_dagger, 74, 60, -30, 1.2, 10)),
    "counterstance": ("gold", lambda: placed(W_h2h, 50, 64, -30, 1.1, 10) + placed(W_h2h, 78, 64, 30, 1.1, 10)),
    "mantra": ("gold", lambda: aura("#ffd27a") + heart(64, 60, 16, "#ffb060")),
    "chi_blast": ("gold", lambda: orb(64, 60, 18, "#ffffff", "#ffb060", "#5a2a04", glow="#ffd27a") + waves(64, 60, "#ffd27a", 3, -40, 40, 22)),
    "warding_circle": ("blood", lambda: f'<ellipse cx="64" cy="84" rx="40" ry="10" fill="none" stroke="#ff6040" stroke-width="2.4" filter="url(#b06)"/>' + placed(W_katana, 64, 56, 0, 0.9, 20)),
    "shadowbind": ("dark", lambda: chain(30, 90, 98, 40, 12) + placed(lambda: paint("M14 -0.8 L-34 -0.8 L-34 0.8 L14 0.8 Z", "wood", rim=False) + paint("M-34 -3 L-42 0 L-34 3 Z", "steel"), 64, 96, 90, 1.0)),
    "eagle_eye": ("wind", lambda: eye(64, 58, 26, "#ffd27a")),
    "spirit_link": ("azure", lambda: cure_core(64, 60, 0.8, 3, 7).replace("#6ae0a0", "#40e0ff").replace("#b8ffd8", "#40e0ff")),
    "familiar": ("earth", lambda: aura("#ffd27a") + paw(64, 62, 22)),
    "dismiss": ("night", lambda: paw(64, 62, 18) + f'<path d="M40 40 L88 88" stroke="#e05050" stroke-width="4"/>'),
    "pet_cmd": ("earth", lambda: paw(64, 62, 22)),
}
TWO_HOURS = {"mighty_strikes": "war", "hundred_fists": "mnk", "benediction": "whm", "manafont": "blm", "chainspell": "rdm", "perfect_dodge": "thf",
             "invincible": "pld", "blood_weapon": "drk", "familiar": "bst", "soul_voice": "brd", "eagle_eye_shot": "rng", "meikyo_shisui": "sam",
             "mijin_gakure": "nin", "spirit_surge": "drg", "astral_flow": "smn", "azure_lore": "blu", "wild_card": "cor", "overdrive": "pup"}


def build():
    from base import make
    for job in EMB:
        make(f"ja_job_{job}", f"Job Abilities - {TITLES[job]}", EMB[job][0], lambda job=job: emblem_body(job), seed=3)
        make(f"ja_2h_{job}", f"Two-hour - {TITLES[job]}", EMB[job][0], lambda job=job: emblem_body(job, True), seed=4)
    for k, (pal, fn) in U.items():
        make(f"ja_{k}", f"Ability - {k.replace('_', ' ').title()}", pal, lambda fn=fn, pal=pal: ring(pal, op=0.6) + fn(), seed=5)


if __name__ == "__main__":
    build()
    print("ok")
