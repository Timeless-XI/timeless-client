"""Every non-elemental spell family (one design each, shared by its tiers)."""
from prims import *  # noqa
from elements import aero_core, stone_core, thunder_core, water_core, light_core, dark_core

ICONS = {}


def reg(name, title, pal, seed=1):
    def deco(fn):
        ICONS[name] = (title, pal, fn, seed)
        return fn
    return deco


def ground(pal):
    return ring(pal, op=0.7)


# ------------------------------------------------------------ healing
def cure_core(cx, cy, sc, tier, seed):
    R = 26 * sc
    out = [halo(cx, cy, R * 1.6, "#b8ffd8", 0.55)]
    for k in range(4):
        a = k * 90 + 45
        p = f"M0 0 C{R*0.28:.1f} {-R*0.3:.1f} {R*0.24:.1f} {-R*0.85:.1f} 0 {-R:.1f} C{-R*0.24:.1f} {-R*0.85:.1f} {-R*0.28:.1f} {-R*0.3:.1f} 0 0 Z"
        out.append(f'<g transform="translate({cx} {cy}) rotate({a})"><path d="{p}" fill="#6ae0a0" fill-opacity="0.55" filter="url(#b1)"/><path d="{p}" fill="#d8ffe8" fill-opacity="0.85" transform="scale(0.7)"/></g>')
    for k in range(4):
        a = k * 90
        p = f"M0 0 C{R*0.22:.1f} {-R*0.3:.1f} {R*0.18:.1f} {-R*0.7:.1f} 0 {-R*0.82:.1f} C{-R*0.18:.1f} {-R*0.7:.1f} {-R*0.22:.1f} {-R*0.3:.1f} 0 0 Z"
        out.append(f'<g transform="translate({cx} {cy}) rotate({a})"><path d="{p}" fill="#f2fff8"/><path d="M0 0 L0 {-R*0.75:.1f}" stroke="#6ae0a0" stroke-opacity="0.6" stroke-width="0.5"/></g>')
    out.append(blob(cx, cy, R * 0.25, R * 0.25, "#ffffff", 1, "b1"))
    out.append(motes(seed + 5, 6 + tier * 2, (cx - R, cy - R * 1.4, cx + R, cy + R * 0.4), ["#d8ffe8", "#8ff0c0", "#ffffff"], (0.5, 1.2)))
    return "".join(out)


def build_tiers():
    from base import make
    for t in range(1, 6):
        make(f"sp_cure{t}", f"Spell - Cure{['','',' II',' III',' IV',' V'][t]}", "heal", lambda: ground("heal") + tiered(cure_core, t, False, "heal", seed=t), seed=t)
    for t in range(1, 4):
        make(f"sp_curaga{t}", "Spell - Curaga", "heal", lambda: ground("heal") + tiered(cure_core, t, True, "heal", seed=t + 9), seed=t + 3)
    for t in (1, 2, 3):
        make(f"sp_bio{t}", f"Spell - Bio{['','',' II',' III'][t]}", "poison", lambda: ground("poison") + tiered(dark_core, t + 1, False, "poison", seed=t), seed=t)
    make("sp_holy", "Spell - Holy", "light", lambda: ground("light") + tiered(light_core, 5, False, "light", seed=3) + wing(64, 66, -1, 0.9, "#fff6dc") + wing(64, 66, 1, 0.9, "#fff6dc") + light_core(64, 62, 0.7, 4, 3), seed=9)
    for el, core in (("wind", aero_core), ("earth", stone_core), ("thunder", thunder_core), ("water", water_core)):
        pass


@reg("sp_raise", "Spell - Raise", "light")
def raise_():
    out = [ground("light"), halo(64, 60, 50, "#fff0c0", 0.5),
           f'<path d="M48 104 L58 10 L70 10 L80 104 Z" fill="#fff6dc" fill-opacity="0.18" filter="url(#b3)"/>',
           feather(62, 98, 70, 16, "#fff6e0", "#e8d0a0"), sparkle(64, 34, 8), motes(11, 18, (40, 20, 88, 100), ["#fff6dc", "#ffe6a0"], (0.5, 1.2))]
    return "".join(out)


@reg("sp_reraise", "Spell - Reraise", "light", 2)
def reraise():
    return raise_() + f'<g opacity="0.95">{hourglass(90, 82, 26)}</g>'


@reg("sp_protect", "Spell - Protect", "night", 3)
def protect():
    return (ground("night") + halo(64, 60, 46, "#8fb2e6", 0.5)
            + f'<ellipse cx="64" cy="66" rx="44" ry="40" fill="none" stroke="#8fb2e6" stroke-opacity="0.35" stroke-width="2" filter="url(#b1)"/>'
            + shield(64, 60, 24, 34, "cloth_blue", "steel", gem(64, 58, 6, ("#8fb2e6", "#0c1a4a", "#e6f0ff"))))


@reg("sp_shell", "Spell - Shell", "dark", 4)
def shell():
    out = [ground("dark"), halo(64, 62, 46, "#c080ff", 0.45)]
    # scallop shell ward
    d = "M64 96 L28 50 Q30 26 64 22 Q98 26 100 50 Z"
    ribs = "".join(f'<path d="M64 94 L{64 + 34*math.sin(math.radians(a)):.1f} {54 - 30*math.cos(math.radians(a)):.1f}" stroke="#3a1a5a" stroke-opacity="0.55" stroke-width="0.8"/>' for a in range(-70, 71, 20))
    out.append(cast_shadow(d, 2, 3, 0.5) + paint(d, dict(base="#b090e0", lo="#2a1240", hi="#f4e8ff", rim="#e0c8ff", outline="#140820", tex="mottle"), hl=(52, 38, 14, 8), extra=ribs))
    out.append(f'<path d="M28 50 Q30 26 64 22 Q98 26 100 50" fill="none" stroke="#f0e0ff" stroke-opacity="0.7" stroke-width="0.8"/>')
    out.append(f'<circle cx="64" cy="60" r="44" fill="none" stroke="#c080ff" stroke-opacity="0.35" stroke-width="1.4" stroke-dasharray="3 3"/>')
    return "".join(out)


@reg("sp_haste", "Spell - Haste", "dusk", 5)
def haste():
    return ground("dusk") + halo(64, 58, 40, "#ffd27a", 0.45) + wing(58, 58, -1, 1.0) + wing(70, 58, 1, 1.0) + hourglass(64, 58, 40) + motes(5, 10, (20, 20, 108, 90), ["#ffe6a0", "#ffffff"], (0.4, 1.0))


@reg("sp_slow", "Spell - Slow", "blood", 6)
def slow():
    return ground("blood") + halo(64, 58, 40, "#ff7070", 0.35) + hourglass(64, 58, 44, "#b05050", "#ffb0b0", "iron") + chain(30, 44, 98, 72, 9) + chain(30, 72, 98, 44, 9)


@reg("sp_refresh", "Spell - Refresh", "water", 7)
def refresh():
    out = [ground("water"), halo(64, 60, 44, "#6fd0ff", 0.5), wisps(64, 62, 7, 4, 26, "#2a7ad0", "#c0e8ff", 260, 2.2)]
    d = "M64 30 C70 44 82 54 82 68 A18 18 0 0 1 46 68 C46 54 58 44 64 30 Z"
    out.append(paint(d, dict(base="#3a8ae8", lo="#062a5a", hi="#e0f6ff", rim="#9fe0ff", outline="#041a3a", tex="mottle"), hl=(58, 62, 6, 9)))
    out.append(sparkle(58, 60, 5) + motes(17, 14, (24, 20, 104, 100), ["#c0e8ff", "#ffffff"], (0.4, 1.0)))
    return "".join(out)


@reg("sp_regen", "Spell - Regen", "heal", 8)
def regen():
    out = [ground("heal"), halo(64, 62, 44, "#9ff0c8", 0.5)]
    # seedling: curved stem, two round leaves, a bud, light pooling at its roots
    out.append(blob(64, 94, 16, 3, "#b8ffd8", 0.6, "b2"))
    stem = "M63 96 C62 84 66 74 64 58 L66 58 C68 74 64 84 66 96 Z"
    out.append(paint(stem, dict(base="#4a9a48", lo="#0e2a10", hi="#c8f0a0", rim="#b8ffc8", outline="#082a0c", tex="mottle")))
    for sx, y, L in ((-1, 72, 26), (1, 64, 22)):
        d = f"M0 0 C{L*0.25:.1f} {-L*0.55:.1f} {L*0.85:.1f} {-L*0.6:.1f} {L:.1f} {-L*0.1:.1f} C{L*0.7:.1f} {L*0.3:.1f} {L*0.25:.1f} {L*0.25:.1f} 0 0 Z"
        out.append(f'<g transform="translate(65 {y}) scale({sx} 1) rotate(-12)">' + paint(d, dict(base="#62c060", lo="#0e3a14", hi="#e0ffc0", rim="#c8ffd0", outline="#082a0c", tex="mottle"), hl=(L * 0.45, -L * 0.25, L * 0.25, L * 0.1))
                   + f'<path d="M1 0 C{L*0.35:.1f} {-L*0.22:.1f} {L*0.65:.1f} {-L*0.25:.1f} {L*0.92:.1f} {-L*0.1:.1f}" fill="none" stroke="#e8ffd0" stroke-opacity="0.7" stroke-width="0.6"/></g>')
    out.append(paint(ell_d(65, 54, 4, 6), dict(base="#9ae070", lo="#1a4a10", hi="#f0ffd0", rim="#ffffff", outline="#0a2a08", tex="mottle")))
    out.append(droplet(46, 44, 2.6, "#c8ffe0") + droplet(84, 40, 2.2, "#c8ffe0") + droplet(72, 28, 1.8, "#c8ffe0"))
    out.append(motes(23, 14, (30, 16, 98, 96), ["#d8ffe8", "#ffffff"], (0.4, 1.0)))
    return "".join(out)


@reg("sp_cleanse", "Spell - Status cure", "heal", 9)
def cleanse():
    out = [ground("heal"), blob(64, 76, 34, 14, "#5a2a6a", 0.6, "b3"), blob(48, 80, 12, 8, "#3a1a4a", 0.6, "b2"), blob(82, 80, 14, 8, "#3a1a4a", 0.6, "b2"),
           f'<path d="M50 104 L58 8 L70 8 L78 104 Z" fill="#e8fff2" fill-opacity="0.25" filter="url(#b3)"/>',
           cure_core(64, 56, 0.85, 3, 4)]
    rnd = random.Random(4)
    for _ in range(10):
        x, y = rnd.uniform(34, 94), rnd.uniform(70, 96)
        out.append(blob(x, y, 2, 2, "#a060c0", 0.6, "b1"))
    return "".join(out)


@reg("sp_dia", "Spell - Dia", "light", 10)
def dia():
    return ground("light") + light_core(64, 60, 0.85, 2, 2) + "".join(
        f'<path d="M{64+16*math.cos(math.radians(a)):.1f} {60+16*math.sin(math.radians(a)):.1f} L{64+40*math.cos(math.radians(a)):.1f} {60+40*math.sin(math.radians(a)):.1f}" stroke="#e05040" stroke-opacity="0.7" stroke-width="1.6" filter="url(#b06)"/>' for a in range(0, 360, 45))


@reg("sp_flash", "Spell - Flash", "light", 11)
def flash():
    out = [ground("light"), halo(64, 58, 60, "#ffffff", 0.7)]
    for i in range(28):
        a = math.radians(i * 360 / 28)
        L = 52 if i % 2 else 36
        out.append(f'<path d="M64 58 L{64+L*math.cos(a):.1f} {58+L*math.sin(a):.1f}" stroke="#fffaf0" stroke-opacity="0.8" stroke-width="{1.6 if i%2 else 1:.1f}" filter="url(#b06)"/>')
    out.append(blob(64, 58, 16, 16, "#ffffff", 1, "b2") + sparkle(64, 58, 18))
    return "".join(out)


# ------------------------------------------------------------ enhancing
@reg("sp_sneak", "Spell - Sneak", "night", 12)
def sneak():
    out = [ground("night")]
    for i, (x, y, a) in enumerate(((44, 96, -12), (58, 80, 8), (54, 62, -10), (68, 46, 10), (64, 28, -8))):
        op = 0.3 + i * 0.15
        d = f"M{x-4} {y} C{x-5} {y-8} {x-2} {y-12} {x+1} {y-11} C{x+4} {y-10} {x+5} {y-5} {x+3} {y} C{x+2} {y+3} {x-3} {y+3} {x-4} {y} Z"
        out.append(f'<g transform="rotate({a} {x} {y})" opacity="{op:.2f}"><path d="{d}" fill="#8fb2e6" fill-opacity="0.5" filter="url(#b1)"/><path d="{d}" fill="none" stroke="#e6ecf5" stroke-width="0.7" stroke-dasharray="1.6 1.4"/></g>')
    return "".join(out) + motes(31, 10, (30, 20, 98, 100), ["#8fb2e6", "#e6ecf5"], (0.3, 0.8))


@reg("sp_invisible", "Spell - Invisible", "night", 13)
def invisible():
    out = [ground("night"), halo(64, 60, 40, "#8fb2e6", 0.35)]
    d = "M64 22 C76 22 80 34 78 44 C90 50 94 72 92 100 L36 100 C34 72 38 50 50 44 C48 34 52 22 64 22 Z"
    out.append(f'<path d="{d}" fill="#8fb2e6" fill-opacity="0.12"/><path d="{d}" fill="none" stroke="#cfe0ff" stroke-opacity="0.65" stroke-width="1" stroke-dasharray="2 2"/>')
    out.append(f'<path d="M50 44 Q64 50 78 44" fill="none" stroke="#cfe0ff" stroke-opacity="0.5" stroke-width="0.8"/>')
    out.append(motes(41, 30, (36, 22, 92, 100), ["#cfe0ff", "#ffffff"], (0.3, 0.9)))
    return "".join(out)


@reg("sp_deodorize", "Spell - Deodorize", "wind", 14)
def deodorize():
    out = [ground("wind")]
    for k in range(5):
        a = math.radians(k * 72 - 90)
        x, y = 64 + 9 * math.cos(a), 58 + 9 * math.sin(a)
        out.append(paint(circle_d(x, y, 8), dict(base="#e8a8d0", lo="#5a1a40", hi="#fff0f8", rim="#ffd0e8", outline="#2a0a1e", tex="mottle"), hl=(x - 2, y - 2, 3, 2)))
    out.append(paint(circle_d(64, 58, 5), "brass"))
    out.append(wisps(64, 60, 3, 3, 40, "#3aa878", "#e8fff2", 140, 1.8))
    return "".join(out)


@reg("sp_stoneskin", "Spell - Stoneskin", "earth", 15)
def stoneskin():
    out = [ground("earth"), halo(64, 60, 44, "#f0b860", 0.4)]
    rnd = random.Random(15)
    for row in range(5):
        for col in range(5):
            x, y = 34 + col * 15 + (row % 2) * 7, 32 + row * 13
            if math.hypot(x - 64, (y - 60) * 1.1) < 34:
                out.append(rock(x, y, 8, rnd.randint(0, 99), rnd.uniform(0, 60), 6, "#8a7a62", "#2a2014", "#f0dcb0"))
    return "".join(out)


@reg("sp_blink", "Spell - Blink", "dusk", 16)
def blink():
    out = [ground("dusk")]
    d = "M0 -26 C8 -26 10 -18 9 -12 C18 -8 20 8 18 26 L-18 26 C-20 8 -18 -8 -9 -12 C-10 -18 -8 -26 0 -26 Z"
    for i, (x, op) in enumerate(((40, 0.25), (52, 0.45), (76, 0.45), (88, 0.25))):
        out.append(f'<g transform="translate({x} 70) scale(0.8)" opacity="{op}"><path d="{d}" fill="#cfe0ff" fill-opacity="0.35"/><path d="{d}" fill="none" stroke="#e6ecf5" stroke-width="0.8"/></g>')
    out.append(f'<g transform="translate(64 66)"><path d="{d}" fill="#1a2040"/><path d="{d}" fill="none" stroke="#cfe0ff" stroke-width="1"/></g>')
    out.append(sparkle(64, 46, 6) + motes(16, 16, (24, 30, 104, 100), ["#cfe0ff", "#ffffff"], (0.3, 0.8)))
    return "".join(out)


@reg("sp_aquaveil", "Spell - Aquaveil", "water", 17)
def aquaveil():
    out = [ground("water"), halo(64, 60, 44, "#6fd0ff", 0.45)]
    for k in range(7):
        x = 30 + k * 11
        d = f"M{x} 22 C{x+6} 40 {x-6} 60 {x+2} 100"
        out.append(f'<path d="{d}" fill="none" stroke="#2a7ad0" stroke-opacity="0.6" stroke-width="5" filter="url(#b1)"/><path d="{d}" fill="none" stroke="#c0e8ff" stroke-opacity="0.8" stroke-width="1"/>')
    out.append(droplet(46, 48, 2.4) + droplet(80, 64, 2.2) + droplet(62, 84, 2))
    return "".join(out)


@reg("sp_phalanx", "Spell - Phalanx", "gold", 18)
def phalanx():
    out = [ground("gold"), halo(64, 60, 46, "#ffd27a", 0.4)]
    out.append(shield(40, 64, 16, 24, "cloth_red", "steel") + shield(88, 64, 16, 24, "cloth_red", "steel") + shield(64, 58, 19, 28, "cloth_blue", "brass", gem(64, 56, 5, ("#ffd27a", "#4a2e0c", "#fff2c0"))))
    return "".join(out)


@reg("sp_teleport", "Spell - Teleport", "azure", 19)
def teleport():
    out = [ground("azure"), halo(64, 58, 46, "#40e0ff", 0.5)]
    out.append(f'<ellipse cx="64" cy="94" rx="30" ry="7" fill="none" stroke="#40e0ff" stroke-width="2" filter="url(#b1)"/><ellipse cx="64" cy="94" rx="30" ry="7" fill="none" stroke="#d8faff" stroke-width="0.6"/>')
    out.append(f'<path d="M36 94 L42 16 L86 16 L92 94 Z" fill="#40e0ff" fill-opacity="0.16" filter="url(#b2)"/>')
    for k in range(6):
        a = math.radians(k * 60)
        out.append(f'<path d="M{64+26*math.cos(a):.1f} {94+6*math.sin(a):.1f} L{64+18*math.cos(a):.1f} {30+4*math.sin(a):.1f}" stroke="#d8faff" stroke-opacity="0.35" stroke-width="0.6"/>')
    d = "M64 30 L76 54 L64 80 L52 54 Z"
    out.append(paint(d, dict(base="#5ae0f0", lo="#06303a", hi="#e8ffff", rim="#a8f8ff", outline="#022028", tex="mottle"), hl=(60, 46, 4, 8)))
    out.append(f'<path d="M64 30 L64 80 M52 54 L76 54" stroke="#e8ffff" stroke-opacity="0.6" stroke-width="0.6"/>' + sparkle(64, 40, 6))
    out.append(motes(19, 20, (30, 14, 98, 96), ["#d8faff", "#40e0ff"], (0.4, 1.0)))
    return "".join(out)


@reg("sp_warp", "Spell - Warp", "dark", 20)
def warp():
    out = [ground("dark"), wisps(64, 60, 20, 5, 34, "#6a2a8a", "#e0c0ff", 300, 2.4)]
    out.append(orb(64, 60, 12, "#ffffff", "#c080ff", "#2a0a40", glow="#c080ff"))
    return "".join(out)


# ------------------------------------------------------------ enfeebling / black
@reg("sp_sleep", "Spell - Sleep", "night", 21)
def sleep():
    out = [ground("night"), moon(60, 56, 22, "#dfe6ff")]
    for x, y, s in ((88, 40, 1.0), (98, 28, 0.75), (104, 18, 0.55)):
        out.append(f'<text x="{x}" y="{y}" font-family="Georgia,serif" font-size="{14*s:.1f}" font-style="italic" fill="#e6ecf5" fill-opacity="0.85">z</text>')
    return "".join(out) + motes(21, 10, (20, 20, 108, 90), ["#cfe0ff"], (0.3, 0.8))


@reg("sp_poison", "Spell - Poison", "poison", 22)
def poison():
    out = [ground("poison"), halo(64, 64, 40, "#a0d040", 0.45)]
    d = "M64 26 C70 42 84 54 84 70 A20 20 0 0 1 44 70 C44 54 58 42 64 26 Z"
    out.append(paint(d, dict(base="#7ab030", lo="#1a2a06", hi="#e8ffb0", rim="#c0f060", outline="#0a1402", tex="mottle"), hl=(56, 64, 6, 9)))
    # skull mark
    out.append(f'<path d="M57 70 C57 63 71 63 71 70 C71 74 69 75 68 76 L68 79 L60 79 L60 76 C59 75 57 74 57 70 Z" fill="#14200a" fill-opacity="0.8"/>'
               f'<circle cx="61" cy="70" r="1.8" fill="#c0f060"/><circle cx="67" cy="70" r="1.8" fill="#c0f060"/>')
    for x, y, r in ((40, 92, 3), (88, 88, 2.4), (76, 96, 1.8), (50, 98, 1.6)):
        out.append(f'<circle cx="{x}" cy="{y}" r="{r}" fill="#a0d040" fill-opacity="0.5"/><circle cx="{x}" cy="{y}" r="{r}" fill="none" stroke="#e8ffb0" stroke-width="0.4"/>')
    return "".join(out)


@reg("sp_paralyze", "Spell - Paralyze", "gold", 23)
def paralyze():
    out = [ground("gold"), halo(64, 60, 44, "#ffd27a", 0.4)]
    for cx in (40, 88):
        out.append(paint(ell_d(cx, 62, 13, 9) + " " + ell_d(cx, 62, 9, 5.4), "iron", hl=(cx - 4, 56, 5, 2)))
    out.append(chain(53, 62, 75, 62, 5))
    rnd = random.Random(23)
    for k in range(7):
        x0, y0 = rnd.uniform(28, 100), rnd.uniform(40, 84)
        pts = bolt_pts(x0, y0, x0 + rnd.uniform(-16, 16), y0 + rnd.uniform(-14, 14), 4, 4, rnd)
        out.append(bolt_draw(pts, 0.6, 1).replace("#7a5aff", "#ffa020").replace("#c8b0ff", "#ffe080"))
    out.append(sparkle(64, 46, 7, "#fff6c0"))
    return "".join(out)


@reg("sp_silence", "Spell - Silence", "dusk", 24)
def silence():
    out = [ground("dusk"), halo(64, 58, 40, "#cfe0ff", 0.4)]
    bell = "M64 30 C76 30 80 42 80 56 C80 66 84 72 90 76 L38 76 C44 72 48 66 48 56 C48 42 52 30 64 30 Z"
    out.append(cast_shadow(bell, 2, 3, 0.5) + paint(bell, "brass", hl=(56, 46, 5, 10)) + paint(circle_d(64, 80, 5), "brass"))
    out.append(f'<circle cx="64" cy="58" r="34" fill="none" stroke="#e05050" stroke-width="5" stroke-opacity="0.85"/><path d="M40 34 L88 82" stroke="#e05050" stroke-width="5" stroke-opacity="0.85"/>'
               f'<circle cx="64" cy="58" r="34" fill="none" stroke="#ffd0d0" stroke-width="0.6"/>')
    return "".join(out)


@reg("sp_blind", "Spell - Blind", "dark", 25)
def blind():
    return ground("dark") + eye(64, 58, 28, "#8a60b0") + blob(64, 58, 30, 10, "#0a0412", 0.75, "b3") + wisps(64, 58, 25, 4, 30, "#2a0a3a", "#c080ff", 160, 2.4)


@reg("sp_gravity", "Spell - Gravity", "dark", 26)
def gravity():
    out = [ground("dark")]
    for k in range(4):
        out.append(f'<ellipse cx="64" cy="{88 + k*4}" rx="{40 - k*7}" ry="{8 - k*1.4:.1f}" fill="none" stroke="#c080ff" stroke-opacity="{0.6 - k*0.1:.2f}" stroke-width="1.2" filter="url(#b06)"/>')
    for x in (40, 52, 76, 88):
        out.append(f'<path d="M{x} 20 L{x} 70 M{x-4} 64 L{x} 72 L{x+4} 64" fill="none" stroke="#c080ff" stroke-opacity="0.6" stroke-width="1.4"/>')
    out.append(orb(64, 66, 14, "#e0c0ff", "#6a2a9a", "#100418", glow="#a060c0"))
    return "".join(out)


@reg("sp_bind", "Spell - Bind", "poison", 27)
def bind():
    out = [ground("poison"), f'<ellipse cx="64" cy="98" rx="30" ry="6" fill="none" stroke="#c0f060" stroke-width="1.6" filter="url(#b06)"/>']
    rnd = random.Random(27)
    for k in range(6):
        x0 = 40 + k * 9.6
        side = -1 if k < 3 else 1
        d = f"M{x0:.1f} 98 C{x0 - side*4:.1f} 80 {64 + side*6:.1f} 70 {64 - side*10:.1f} 56 C{64 - side*16:.1f} 48 {64 - side*6:.1f} 38 {64 + side*2:.1f} 40"
        out.append(f'<path d="{d}" fill="none" stroke="#1a2a06" stroke-width="5.2" stroke-linecap="round"/><path d="{d}" fill="none" stroke="#6a8a28" stroke-width="3.4" stroke-linecap="round"/>'
                   f'<path d="{d}" fill="none" stroke="#d8f0a0" stroke-opacity="0.55" stroke-width="0.8" stroke-linecap="round" transform="translate(-0.6 -0.6)"/>')
    for _ in range(14):
        x, y = rnd.uniform(40, 88), rnd.uniform(44, 94)
        out.append(f'<path d="M{x:.1f} {y:.1f} l{rnd.uniform(-3,3):.1f} -3.2 l1.6 2.6 Z" fill="#c0e070"/>')
    out.append(leaf(52, 52, 9, 200, "#7ab048") + leaf(78, 60, 8, -20, "#8ac858"))
    return "".join(out)


@reg("sp_dispel", "Spell - Dispel", "azure", 28)
def dispel():
    out = [ground("azure"), halo(64, 60, 40, "#40e0ff", 0.35)]
    for k in range(8):
        a0 = k * 45 + 6
        a1 = a0 + 33
        r = 30
        p0 = (64 + r * math.cos(math.radians(a0)), 60 + r * math.sin(math.radians(a0)))
        p1 = (64 + r * math.cos(math.radians(a1)), 60 + r * math.sin(math.radians(a1)))
        dx, dy = math.cos(math.radians(a0 + 16)) * 4, math.sin(math.radians(a0 + 16)) * 4
        out.append(f'<path d="M{p0[0]+dx:.1f} {p0[1]+dy:.1f} A{r} {r} 0 0 1 {p1[0]+dx:.1f} {p1[1]+dy:.1f}" fill="none" stroke="#40e0ff" stroke-width="3" filter="url(#b06)"/>'
                   f'<path d="M{p0[0]+dx:.1f} {p0[1]+dy:.1f} A{r} {r} 0 0 1 {p1[0]+dx:.1f} {p1[1]+dy:.1f}" fill="none" stroke="#d8faff" stroke-width="0.8"/>')
    out.append(sparkle(64, 60, 14) + motes(28, 18, (24, 20, 104, 100), ["#d8faff", "#40e0ff"], (0.4, 1.0)))
    return "".join(out)


@reg("sp_stun", "Spell - Stun", "thunder", 29)
def stun():
    out = [ground("thunder"), halo(64, 58, 44, "#c8b0ff", 0.5)]
    for k in range(5):
        a = math.radians(k * 72 - 90)
        out.append(sparkle(64 + 26 * math.cos(a), 58 + 10 * math.sin(a), 5, "#fff6dc"))
    out.append(thunder_core(64, 60, 0.72, 2, 9))
    return "".join(out)


def life_draw(col, lite, pal, seed):
    out = [ground(pal), wisps(64, 60, seed, 4, 36, col, lite, 220, 2.2), orb(64, 60, 12, "#ffffff", col, "#100408", glow=col)]
    return "".join(out)


@reg("sp_drain", "Spell - Drain", "blood", 30)
def drain():
    return life_draw("#e04050", "#ffd0d0", "blood", 30)


@reg("sp_aspir", "Spell - Aspir", "water", 31)
def aspir():
    return life_draw("#3a7af0", "#d0e8ff", "water", 31)


@reg("sp_absorb", "Spell - Absorb", "dark", 32)
def absorb():
    return life_draw("#a050e0", "#f0d8ff", "dark", 32)


@reg("sp_tractor", "Spell - Tractor", "dark", 33)
def tractor():
    out = [ground("dark"), f'<path d="M52 104 L58 14 L70 14 L76 104 Z" fill="#c080ff" fill-opacity="0.2" filter="url(#b2)"/>']
    out.append(orb(64, 70, 9, "#ffffff", "#c8d8ff", "#405080", glow="#cfe0ff"))
    out.append(chain(64, 20, 64, 60, 6))
    return "".join(out)


@reg("sp_elem_debuff", "Spell - Elemental debuff", "dark", 34)
def elem_debuff():
    return ground("dark") + dark_core(64, 60, 0.8, 2, 3)


# ------------------------------------------------------------ element-coloured families
def el_icon(el, fn):
    return lambda: fn(el)


def enspell(el):
    g, l, gc = ELC[el]
    return ground(EL_PAL[el]) + halo(64, 58, 44, g, 0.45) + sword(64, 98, 92, 0, 8, gc, glow=g) + wisps(64, 48, 3, 3, 18, g, l, 180, 1.6)


def bar(el):
    g, l, gc = ELC[el]
    return ground(EL_PAL[el]) + halo(64, 58, 44, g, 0.45) + shield(64, 60, 24, 34, "cloth_blue", "silver", orb(64, 56, 8, l, g, "#100808", glow=g))


def spikes(el):
    g, l, gc = ELC[el]
    out = [ground(EL_PAL[el]), halo(64, 60, 46, g, 0.45)]
    for k in range(16):
        a = math.radians(k * 22.5)
        x0, y0 = 64 + 20 * math.cos(a), 60 + 20 * math.sin(a)
        x1, y1 = 64 + 38 * math.cos(a), 60 + 38 * math.sin(a)
        px, py = -math.sin(a) * 3, math.cos(a) * 3
        out.append(f'<path d="M{x0+px:.1f} {y0+py:.1f} L{x1:.1f} {y1:.1f} L{x0-px:.1f} {y0-py:.1f} Z" fill="{g}" fill-opacity="0.85"/><path d="M{x0:.1f} {y0:.1f} L{x1:.1f} {y1:.1f}" stroke="{l}" stroke-width="0.6"/>')
    out.append(orb(64, 60, 18, l, g, "#080408"))
    return "".join(out)


def spirit(el):
    g, l, gc = ELC[el]
    return (ground(EL_PAL[el]) + halo(64, 58, 44, g, 0.55) + wisps(64, 62, 5, 4, 24, g, l, 260, 2.0)
            + orb(64, 58, 13, "#ffffff", g, "#100810") + f'<circle cx="59" cy="56" r="1.6" fill="#ffffff"/><circle cx="69" cy="56" r="1.6" fill="#ffffff"/>'
            + motes(7, 14, (30, 24, 98, 96), [l, g], (0.4, 1.0)))


def avatar(el, sym):
    g, l, gc = ELC[el]
    out = [ground(EL_PAL[el]), halo(64, 58, 48, g, 0.55)]
    # summoning circle + crest gem
    out.append(f'<circle cx="64" cy="58" r="34" fill="none" stroke="{g}" stroke-width="2" filter="url(#b06)"/><circle cx="64" cy="58" r="34" fill="none" stroke="{l}" stroke-width="0.5"/>'
               f'<circle cx="64" cy="58" r="28" fill="none" stroke="{l}" stroke-opacity="0.5" stroke-width="0.4" stroke-dasharray="1.4 2"/>')
    for k in range(6):
        a = math.radians(k * 60 - 90)
        out.append(f'<path d="M{64+28*math.cos(a):.1f} {58+28*math.sin(a):.1f} L{64+28*math.cos(a+2.094):.1f} {58+28*math.sin(a+2.094):.1f}" stroke="{l}" stroke-opacity="0.35" stroke-width="0.5"/>')
    out.append(paint(f"M64 30 L84 58 L64 86 L44 58 Z", "brass", hl=(58, 44, 4, 8)))
    out.append(gem(64, 58, 13, gc))
    out.append(sym)
    return "".join(out)


def carol(el):
    g, l, gc = ELC[el]
    return ground(EL_PAL[el]) + lyre(64, 58, 1.2, g) + notes(5, 5, (30, 20, 100, 60), l)


def threnody(el):
    g, l, gc = ELC[el]
    return ground("dark") + lyre(64, 58, 1.2, g) + notes(9, 5, (30, 20, 100, 60), g) + blob(64, 84, 30, 6, "#000", 0.4, "b2")


def nin_el(el):
    g, l, gc = ELC[el]
    return ground(EL_PAL[el]) + halo(64, 58, 44, g, 0.45) + scroll(64, 58, 30, 38, "parch", None, -12, False) + f'<g transform="rotate(-12 64 58)"><circle cx="64" cy="58" r="9" fill="none" stroke="{g}" stroke-width="2.4"/><circle cx="64" cy="58" r="4" fill="{g}"/></g>' + wisps(64, 58, 3, 3, 30, g, l, 140, 1.6)


def song(col, lite, emblem=""):
    return lambda: ground("night") + lyre(64, 60, 1.25, col) + notes(13, 6, (24, 18, 104, 56), lite) + emblem


def build():
    from base import make
    for k, (title, pal, fn, seed) in ICONS.items():
        make(k, title, pal, fn, seed=seed)
    els = ["fire", "ice", "wind", "earth", "thunder", "water", "light", "dark"]
    for el in els:
        p = EL_PAL[el]
        make(f"sp_en_{el}", "Spell - Enspell", p, el_icon(el, enspell), seed=3)
        make(f"sp_bar_{el}", "Spell - Bar-spell", p, el_icon(el, bar), seed=4)
        make(f"sp_spirit_{el}", "Spell - Spirit", p, el_icon(el, spirit), seed=5)
        make(f"sp_carol_{el}", "Song - Carol", p, el_icon(el, carol), seed=6)
        make(f"sp_threnody_{el}", "Song - Threnody", "dark", el_icon(el, threnody), seed=7)
        if el not in ("light", "dark"):
            make(f"sp_nin_{el}", "Ninjutsu", p, el_icon(el, nin_el), seed=8)
    for el in ("fire", "ice", "thunder", "dark"):
        make(f"sp_spikes_{el}", "Spell - Spikes", EL_PAL[el], el_icon(el, spikes), seed=9)
    # bar-status spells share one shield with a status orb
    make("sp_bar_status", "Spell - Bar-status", "night", lambda: bar("light"), seed=4)
    av = {"carbuncle": ("light", "ruby"), "ifrit": ("fire", ""), "shiva": ("ice", ""), "garuda": ("wind", ""), "titan": ("earth", ""),
          "ramuh": ("thunder", ""), "leviathan": ("water", ""), "fenrir": ("dark", "moon"), "diabolos": ("dark", "")}
    for name, (el, sym) in av.items():
        extra = moon(64, 26, 7) if sym == "moon" else (gem(64, 22, 5, ("#ff4060", "#4a0010", "#ffc0d0")) if sym == "ruby" else "")
        make(f"sp_avatar_{name}", f"Summon - {name.title()}", EL_PAL[el], lambda el=el, extra=extra: avatar(el, extra), seed=10)
    songs = {"minuet": ("#ff7a50", "#ffd8c0", sword(96, 40, 30, -30, 3)), "paeon": ("#6ae0a0", "#e8fff2", ""), "ballad": ("#4ab0f0", "#e0f6ff", ""),
             "march": ("#ffd27a", "#fff2d0", feather(98, 46, 22, 30)), "madrigal": ("#ffb060", "#fff0d0", ""), "mambo": ("#40e0c0", "#d8fff6", ""),
             "etude": ("#cfe0ff", "#ffffff", ""), "requiem": ("#a060c0", "#f0e0ff", ""), "elegy": ("#e05050", "#ffd0d0", hourglass(98, 44, 18)),
             "lullaby": ("#8fb2e6", "#e6ecf5", moon(98, 36, 8)), "finale": ("#ff5050", "#ffd0d0", ""), "hymnus": ("#ffe6a0", "#fffaf0", ""),
             "mazurka": ("#f0d040", "#fff8c0", ""), "prelude": ("#a0d040", "#f4ffd8", ""), "minne": ("#8fb2e6", "#e6ecf5", ""),
             "song": ("#d6e1f0", "#ffffff", "")}
    for k, (c, l, e) in songs.items():
        make(f"sp_song_{k}", f"Song - {k.title()}", "night", song(c, l, e), seed=11)
    nin = {
        "sp_utsusemi": ("Ninjutsu - Utsusemi", "night", lambda: ground("night") + "".join(f'<g opacity="{op}">{scroll(x, 60, 18, 30, "parch", None, a, False)}</g>' for x, op, a in ((44, 0.35, -18), (84, 0.35, 18))) + scroll(64, 58, 22, 36, "parch", "#a83a32", 0, False) + motes(3, 14, (24, 20, 104, 96), ["#cfe0ff"], (0.3, 0.8))),
        "sp_tonko": ("Ninjutsu - Tonko", "night", lambda: ground("night") + "".join(blob(x, y, r, r * 0.8, "#8a94b0", 0.55, "b3") for x, y, r in ((50, 70, 16), (78, 66, 18), (64, 52, 16), (64, 80, 14))) + leaf(70, 46, 12, -30, "#6ab048") + leaf(50, 60, 10, 200, "#8ac858")),
        "sp_nin_enfeeble": ("Ninjutsu - Enfeeble", "dark", lambda: ground("dark") + scroll(64, 58, 30, 38, "parch", "#6a2a8a", -10, False) + chain(34, 76, 94, 40, 10)),
        "sp_bluemagic": ("Blue Magic", "azure", lambda: ground("azure") + halo(64, 58, 44, "#40e0ff", 0.5) + "".join(paint(f"M{64+dx} {58+dy-8} Q{64+dx+8} {58+dy} {64+dx} {58+dy+8} Q{64+dx-8} {58+dy} {64+dx} {58+dy-8} Z", dict(base="#2a8ac0", lo="#04202e", hi="#c8f4ff", rim="#80e8ff", outline="#021018", tex="mottle")) for dx, dy in ((-16, -14), (0, -14), (16, -14), (-24, 0), (24, 0), (-16, 14), (0, 14), (16, 14))) + eye(64, 58, 16, "#40e0ff")),
        "sp_trust": ("Trust", "gold", lambda: ground("gold") + halo(64, 58, 44, "#ffd27a", 0.45) + f'<g transform="rotate(-8 64 58)">' + paint("M42 24 L86 24 L86 92 L42 92 Z", "parch", hl=(54, 40, 8, 14)) + paint("M46 28 L82 28 L82 88 L46 88 Z", dict(MAT["cloth_blue"], tex="mottle"), rim=False) + gem(64, 58, 12, ("#ffd27a", "#4a2e0c", "#fff2c0")) + '</g>'),
        "sp_ancient": ("Ancient Magic", "dark", lambda: ground("dark") + light_core(64, 58, 0.9, 4, 7).replace("#ffe6a0", "#ff8a50").replace("#fff6dc", "#ffe0c0") + dark_core(64, 58, 0.4, 1, 3)),
    }
    for k, (t, p, fn) in nin.items():
        make(k, t, p, fn, seed=12)
    build_tiers()


if __name__ == "__main__":
    build()
    print(len(os.listdir(OUT)))
