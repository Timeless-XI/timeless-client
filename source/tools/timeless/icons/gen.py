#!/usr/bin/env python3
"""Timeless XI - Engage hotbar icon concepts (128x128 SVG)."""
import math, os

OUT = os.path.join(os.path.dirname(__file__), "out")

# Palette (Timeless theme.json + logo)
CHAR = "#0a0b12"; SURF = "#11141d"; BRONZE = "#5c6b85"; PALE = "#d6e1f0"
FOCUS = "#8fb2e6"; SAND = "#e3c98e"; SAND_D = "#a8864f"; TEXT = "#e6ecf5"


def defs_common():
    return f"""
  <radialGradient id="plate" cx="50%" cy="38%" r="75%">
    <stop offset="0" stop-color="#1d2433"/>
    <stop offset="0.55" stop-color="{SURF}"/>
    <stop offset="1" stop-color="#05060a"/>
  </radialGradient>
  <radialGradient id="aura" cx="50%" cy="50%" r="50%">
    <stop offset="0" stop-color="#3d5f94" stop-opacity="0.55"/>
    <stop offset="1" stop-color="#3d5f94" stop-opacity="0"/>
  </radialGradient>
  <linearGradient id="rim" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0" stop-color="#a9b6cc"/>
    <stop offset="0.45" stop-color="{BRONZE}"/>
    <stop offset="1" stop-color="#1e2431"/>
  </linearGradient>
  <linearGradient id="bladeL" x1="0" y1="0" x2="1" y2="0">
    <stop offset="0" stop-color="#8d9bb3"/>
    <stop offset="0.35" stop-color="#f3f6fb"/>
    <stop offset="1" stop-color="#c3cddd"/>
  </linearGradient>
  <linearGradient id="bladeR" x1="0" y1="0" x2="1" y2="0">
    <stop offset="0" stop-color="#7d8aa2"/>
    <stop offset="0.7" stop-color="#46516a"/>
    <stop offset="1" stop-color="#2a3142"/>
  </linearGradient>
  <linearGradient id="pewter" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0" stop-color="#eef2f8"/>
    <stop offset="0.35" stop-color="#a7b3c7"/>
    <stop offset="0.7" stop-color="{BRONZE}"/>
    <stop offset="1" stop-color="#232a38"/>
  </linearGradient>
  <linearGradient id="pewterH" x1="0" y1="0" x2="1" y2="0">
    <stop offset="0" stop-color="#3a4459"/>
    <stop offset="0.3" stop-color="#c9d3e2"/>
    <stop offset="0.55" stop-color="#eef2f8"/>
    <stop offset="1" stop-color="#4a556c"/>
  </linearGradient>
  <linearGradient id="grip" x1="0" y1="0" x2="1" y2="0">
    <stop offset="0" stop-color="#0d1019"/>
    <stop offset="0.45" stop-color="#2a3550"/>
    <stop offset="1" stop-color="#0b0e16"/>
  </linearGradient>
  <radialGradient id="gem" cx="38%" cy="32%" r="70%">
    <stop offset="0" stop-color="#ffffff"/>
    <stop offset="0.3" stop-color="#bcd3f5"/>
    <stop offset="0.7" stop-color="{FOCUS}"/>
    <stop offset="1" stop-color="#27406b"/>
  </radialGradient>
  <linearGradient id="sand" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0" stop-color="#f6e6bd"/>
    <stop offset="0.5" stop-color="{SAND}"/>
    <stop offset="1" stop-color="{SAND_D}"/>
  </linearGradient>
  <radialGradient id="flare" cx="50%" cy="50%" r="50%">
    <stop offset="0" stop-color="#ffffff" stop-opacity="1"/>
    <stop offset="0.25" stop-color="#cfe0fa" stop-opacity="0.85"/>
    <stop offset="0.6" stop-color="{FOCUS}" stop-opacity="0.3"/>
    <stop offset="1" stop-color="{FOCUS}" stop-opacity="0"/>
  </radialGradient>
  <linearGradient id="lacquer" x1="0" y1="0" x2="1" y2="0">
    <stop offset="0" stop-color="#0c1220"/>
    <stop offset="0.35" stop-color="#34466b"/>
    <stop offset="0.5" stop-color="#5b729c"/>
    <stop offset="0.7" stop-color="#22304d"/>
    <stop offset="1" stop-color="#0a0f1a"/>
  </linearGradient>
  <filter id="shadow" x="-30%" y="-30%" width="160%" height="160%">
    <feDropShadow dx="0" dy="1.6" stdDeviation="1.4" flood-color="#000" flood-opacity="0.75"/>
  </filter>
  <filter id="soft" x="-50%" y="-50%" width="200%" height="200%">
    <feGaussianBlur stdDeviation="2.2"/>
  </filter>
  <filter id="softS" x="-50%" y="-50%" width="200%" height="200%">
    <feGaussianBlur stdDeviation="0.8"/>
  </filter>
  <clipPath id="plateClip"><rect x="3" y="3" width="122" height="122" rx="11"/></clipPath>
"""


def plate_back():
    return """
  <rect x="3" y="3" width="122" height="122" rx="11" fill="url(#plate)"/>
  <g clip-path="url(#plateClip)">
    <ellipse cx="64" cy="58" rx="54" ry="50" fill="url(#aura)"/>
    <g opacity="0.07" stroke="#d6e1f0" stroke-width="0.5" fill="none">
      <circle cx="64" cy="64" r="52"/>
      <circle cx="64" cy="64" r="47"/>
    </g>
  </g>
"""


def plate_front():
    # bevel rim, inner hairline, logo-style diamonds on the top/bottom hairlines
    return f"""
  <rect x="3" y="3" width="122" height="122" rx="11" fill="none" stroke="#000" stroke-opacity="0.6" stroke-width="3"/>
  <rect x="3.5" y="3.5" width="121" height="121" rx="10.5" fill="none" stroke="url(#rim)" stroke-width="1.6"/>
  <rect x="7.5" y="7.5" width="113" height="113" rx="7.5" fill="none" stroke="{PALE}" stroke-opacity="0.1" stroke-width="0.6"/>
  <g fill="none" stroke="{PALE}" stroke-opacity="0.35" stroke-width="0.6">
    <path d="M44 7.5 H58 M70 7.5 H84"/>
    <path d="M44 120.5 H58 M70 120.5 H84"/>
  </g>
  <g fill="{PALE}" fill-opacity="0.8">
    <path d="M64 5.3 L66.2 7.5 L64 9.7 L61.8 7.5 Z"/>
    <path d="M64 118.3 L66.2 120.5 L64 122.7 L61.8 120.5 Z"/>
  </g>
  <path d="M8 18 Q8 8 18 8 H110 Q120 8 120 18" fill="none" stroke="#fff" stroke-opacity="0.06" stroke-width="1"/>
"""


def sword(L=74, w=9.6, grip=15, gw=30, pommel=4.2, fuller=True, gleam=0.55):
    """Sword in local coords: guard centred at 0,0, tip at (0,-L)."""
    ht = w / 2 * 0.78; hb = w / 2; tl = w * 2.1
    s = []
    # blade halves
    s.append(f'<path d="M0 {-L} L{-ht:.2f} {-L+tl:.2f} L{-hb:.2f} -2.5 L0 -2.5 Z" fill="url(#bladeL)"/>')
    s.append(f'<path d="M0 {-L} L{ht:.2f} {-L+tl:.2f} L{hb:.2f} -2.5 L0 -2.5 Z" fill="url(#bladeR)"/>')
    # secondary bevel (edge grind) on light side
    s.append(f'<path d="M0 {-L} L{-ht:.2f} {-L+tl:.2f} L{-hb:.2f} -2.5 L{-hb+1.1:.2f} -2.5 L{-ht+1.0:.2f} {-L+tl+0.8:.2f} Z" fill="#ffffff" fill-opacity="0.28"/>')
    # outline
    s.append(f'<path d="M0 {-L} L{-ht:.2f} {-L+tl:.2f} L{-hb:.2f} -2.5 L{hb:.2f} -2.5 L{ht:.2f} {-L+tl:.2f} Z" fill="none" stroke="#141823" stroke-width="0.55" stroke-linejoin="round"/>')
    if fuller:
        fy0 = -L + tl + 5
        s.append(f'<path d="M0 {fy0:.2f} L0 -7" stroke="#1d2433" stroke-width="1.5" stroke-linecap="round"/>')
        s.append(f'<path d="M0.75 {fy0+1:.2f} L0.75 -7.5" stroke="#ffffff" stroke-opacity="0.35" stroke-width="0.45" stroke-linecap="round"/>')
    # edge gleam
    s.append(f'<path d="M{-ht+0.3:.2f} {-L+tl:.2f} L{-hb+0.35:.2f} -4" stroke="#ffffff" stroke-opacity="{gleam}" stroke-width="0.5"/>')
    # ricasso block
    s.append(f'<rect x="{-hb-0.3:.2f}" y="-5.2" width="{w+0.6:.2f}" height="3.2" rx="0.6" fill="url(#pewterH)" stroke="#141823" stroke-width="0.4"/>')
    # guard: tapered bar with finials and a centre gem
    g = gw / 2
    s.append(f'<path d="M{-g} 0 Q{-g/2} -3.4 0 -3.6 Q{g/2} -3.4 {g} 0 Q{g/2} 3.4 0 3.6 Q{-g/2} 3.4 {-g} 0 Z" fill="url(#pewter)" stroke="#141823" stroke-width="0.5"/>')
    s.append(f'<path d="M{-g+2} -0.6 Q{-g/2} -2.6 0 -2.8 Q{g/2} -2.6 {g-2} -0.6" fill="none" stroke="#ffffff" stroke-opacity="0.5" stroke-width="0.45"/>')
    for sx in (-1, 1):
        s.append(f'<circle cx="{sx*(g+0.6)}" cy="0" r="2.1" fill="url(#pewter)" stroke="#141823" stroke-width="0.45"/>')
        s.append(f'<circle cx="{sx*(g+0.6)-0.5}" cy="-0.7" r="0.6" fill="#fff" fill-opacity="0.7"/>')
    s.append('<path d="M0 -4.6 L3.4 0 L0 4.6 L-3.4 0 Z" fill="url(#pewter)" stroke="#141823" stroke-width="0.45"/>')
    s.append('<path d="M0 -2.6 L1.9 0 L0 2.6 L-1.9 0 Z" fill="url(#gem)"/>')
    # grip with diagonal wrap
    gy = 3.4
    s.append(f'<rect x="-2.5" y="{gy}" width="5" height="{grip}" rx="1.4" fill="url(#grip)" stroke="#06080d" stroke-width="0.45"/>')
    n = int(grip / 2.2)
    for i in range(n):
        y = gy + 0.8 + i * (grip - 1.6) / n
        s.append(f'<path d="M-2.4 {y:.2f} L2.4 {y+1.6:.2f}" stroke="{BRONZE}" stroke-opacity="0.75" stroke-width="0.5"/>')
    s.append(f'<rect x="-3" y="{gy-0.4}" width="6" height="1.6" rx="0.5" fill="url(#pewterH)"/>')
    s.append(f'<rect x="-3" y="{gy+grip-1.1}" width="6" height="1.6" rx="0.5" fill="url(#pewterH)"/>')
    # pommel: faceted disc with gem
    py = gy + grip + pommel - 0.2
    s.append(f'<circle cx="0" cy="{py:.2f}" r="{pommel}" fill="url(#pewter)" stroke="#141823" stroke-width="0.5"/>')
    s.append(f'<circle cx="0" cy="{py:.2f}" r="{pommel*0.62:.2f}" fill="none" stroke="#141823" stroke-opacity="0.6" stroke-width="0.4"/>')
    s.append(f'<circle cx="0" cy="{py:.2f}" r="{pommel*0.45:.2f}" fill="url(#gem)"/>')
    return "\n    ".join(s)


GLASS = "M-40 -78 C-40 -30 -6 -14 -3 0 C-6 14 -40 30 -40 78 L40 78 C40 30 6 14 3 0 C6 -14 40 -30 40 -78 Z"


def hourglass(top_level=-34, pile_h=26, wall=3.2, stream=True, pid="hg"):
    """Logo hourglass in its own coords (+-52 x, +-88 y)."""
    s = [f'<clipPath id="{pid}c"><path d="{GLASS}"/></clipPath>']
    s.append(f'<path d="{GLASS}" fill="#8fb2e6" fill-opacity="0.10"/>')
    s.append(f'<g clip-path="url(#{pid}c)">')
    s.append(f'<path d="M-46 {top_level} Q0 {top_level+9} 46 {top_level} L46 2 L-46 2 Z" fill="url(#sand)"/>')
    s.append(f'<path d="M-46 80 L-46 {78-pile_h*0.3:.1f} Q-18 {78-pile_h*0.45:.1f} 0 {78-pile_h} Q18 {78-pile_h*0.45:.1f} 46 {78-pile_h*0.3:.1f} L46 80 Z" fill="url(#sand)"/>')
    if stream:
        s.append(f'<path d="M0 0 L0 {78-pile_h}" stroke="{SAND}" stroke-width="2.2"/>')
    # glass sheen
    s.append('<path d="M-30 -70 C-30 -40 -14 -24 -8 -12" fill="none" stroke="#fff" stroke-opacity="0.35" stroke-width="3"/>')
    s.append('<path d="M-30 70 C-30 46 -16 30 -9 18" fill="none" stroke="#fff" stroke-opacity="0.18" stroke-width="2.4"/>')
    s.append('</g>')
    # walls
    s.append(f'<path d="M-40 -78 C-40 -30 -6 -14 -3 0 C-6 14 -40 30 -40 78" fill="none" stroke="url(#pewter)" stroke-width="{wall}"/>')
    s.append(f'<path d="M40 -78 C40 -30 6 -14 3 0 C6 14 40 30 40 78" fill="none" stroke="url(#pewter)" stroke-width="{wall}"/>')
    # tiered caps (main bar +-52, inner bar +-44)
    for sy in (-1, 1):
        y1 = sy * 80; y2 = sy * 84.5
        s.append(f'<rect x="-52" y="{y1-4.5 if sy<0 else y1-0.5}" width="104" height="5" rx="1.2" fill="url(#pewter)" stroke="#141823" stroke-width="1"/>')
        s.append(f'<rect x="-44" y="{y2-2.5 if sy<0 else y2-1.5}" width="88" height="4" rx="1" fill="url(#pewter)" stroke="#141823" stroke-width="1"/>')
    return "\n    ".join(s)


def star(cx, cy, r, ratio=0.16, rot=0, fill="#fff", op=1.0):
    pts = []
    for i in range(8):
        a = math.radians(rot + i * 45)
        rr = r if i % 2 == 0 else r * ratio
        pts.append(f"{cx + rr*math.sin(a):.2f},{cy - rr*math.cos(a):.2f}")
    return f'<polygon points="{" ".join(pts)}" fill="{fill}" fill-opacity="{op}"/>'


def wrap(body, title):
    return f"""<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 128 128" width="128" height="128">
  <title>{title}</title>
  <defs>{defs_common()}
  </defs>
{plate_back()}
  <g clip-path="url(#plateClip)">
{body}
  </g>
{plate_front()}
</svg>
"""


# ---------- 1. Crossed Steel ----------
def crossed_steel():
    sw = sword(L=84, gw=26, grip=13, pommel=3.8)
    hgs = hourglass(top_level=-36, pile_h=30, wall=7, pid="m1")
    return wrap(f"""
  <g filter="url(#shadow)">
    <g transform="translate(64 66) rotate(-42) translate(0 22)">
    {sw}
    </g>
    <g transform="translate(64 66) rotate(42) translate(0 22)">
    {sw}
    </g>
  </g>
  <ellipse cx="64" cy="62" rx="22" ry="22" fill="url(#aura)"/>
  <g filter="url(#shadow)">
    <circle cx="64" cy="62" r="14" fill="#0b0e16" stroke="url(#pewter)" stroke-width="2.4"/>
    <circle cx="64" cy="62" r="11.4" fill="none" stroke="{PALE}" stroke-opacity="0.25" stroke-width="0.5"/>
    <g transform="translate(64 62) scale(0.106)">
    {hgs}
    </g>
  </g>""", "Engage - Crossed Steel")


# ---------- 2. Hourglass Blade ----------
def hourglass_blade():
    hg = hourglass(top_level=-26, pile_h=34, wall=4, stream=False, pid="h2")
    sw = sword(L=78, w=10, gw=34, grip=11, pommel=3.6)
    motes = "".join(
        f'<circle cx="{x}" cy="{y}" r="{r}" fill="{SAND}" fill-opacity="{o}"/>'
        for x, y, r, o in [(58.5, 70, 0.7, .9), (69.4, 75, 0.6, .8), (57.8, 80, 0.5, .7), (70.2, 66, 0.5, .6), (59, 88, 0.6, .8)])
    return wrap(f"""
  <g transform="translate(64 59) scale(0.43)" opacity="0.98">
    {hg}
  </g>
  {motes}
  <g filter="url(#shadow)" transform="translate(64 94)">
    {sw}
  </g>
  <g transform="translate(64 16)">
    <circle r="7" fill="url(#flare)"/>
    {star(0, 0, 7.5, 0.13)}
    {star(0, 0, 3.6, 0.2, rot=45, op=0.8)}
  </g>""", "Engage - Hourglass Blade")


# ---------- 3. Starfall Clash ----------
def starfall_clash():
    th = 31
    d = 50 * math.sin(math.radians(th))
    sw = sword(L=76, gw=28, grip=13, pommel=3.8)
    CY = 48
    sparks = []
    import random
    random.seed(7)
    for i in range(14):
        a = random.uniform(0, 360); r0 = random.uniform(7, 10); r1 = r0 + random.uniform(3, 9)
        ax, ay = math.sin(math.radians(a)), -math.cos(math.radians(a))
        col = SAND if i % 3 == 0 else "#cfe0fa"
        sparks.append(f'<path d="M{64+ax*r0:.2f} {CY+ay*r0:.2f} L{64+ax*r1:.2f} {CY+ay*r1:.2f}" stroke="{col}" stroke-opacity="{random.uniform(.45,.9):.2f}" stroke-width="{random.uniform(.4,.8):.2f}" stroke-linecap="round"/>')
    for i in range(9):
        a = random.uniform(0, 360); rr = random.uniform(12, 22)
        sparks.append(f'<circle cx="{64+math.sin(math.radians(a))*rr:.2f}" cy="{CY-math.cos(math.radians(a))*rr:.2f}" r="{random.uniform(.35,.8):.2f}" fill="{SAND if i%2 else "#dbe7fb"}" fill-opacity="{random.uniform(.5,.95):.2f}"/>')
    return wrap(f"""
  <g filter="url(#shadow)">
    <g transform="translate({64-d:.2f} {CY+50*math.cos(math.radians(th)):.2f}) rotate({th})">
    {sw}
    </g>
    <g transform="translate({64+d:.2f} {CY+50*math.cos(math.radians(th)):.2f}) rotate({-th})">
    {sw}
    </g>
  </g>
  <circle cx="64" cy="{CY}" r="22" fill="url(#flare)" opacity="0.55"/>
  <g filter="url(#softS)" opacity="0.9">{star(64, CY, 17, 0.09)}</g>
  {star(64, CY, 15, 0.08)}
  {star(64, CY, 7, 0.14, rot=45, op=0.85)}
  <circle cx="64" cy="{CY}" r="2.2" fill="#fff"/>
  {"".join(sparks)}""", "Engage - Starfall Clash")


# ---------- 4. Unsheathed ----------
def unsheathed():
    # axis: guard at (82,46), blade toward down-left (rotate 225)
    sw = sword(L=80, w=8.6, gw=28, grip=15, gleam=0.9)
    sc_a, sc_b = -26, -88   # scabbard spans these local y values
    W = 6.6
    scab = f"""
    <path d="M{-W} {sc_a} L{W} {sc_a} L{W*0.86:.2f} {sc_b+8} Q0 {sc_b-2} {-W*0.86:.2f} {sc_b+8} Z" fill="url(#lacquer)" stroke="#05070c" stroke-width="0.6"/>
    <path d="M{-W+1.6:.2f} {sc_a-2} L{-W*0.86+1.4:.2f} {sc_b+9}" stroke="{FOCUS}" stroke-opacity="0.35" stroke-width="0.6"/>
    <rect x="{-W-0.8:.2f}" y="{sc_a-6.5}" width="{2*W+1.6:.2f}" height="7" rx="1.2" fill="url(#pewterH)" stroke="#141823" stroke-width="0.5"/>
    <path d="M{-W-0.8:.2f} {sc_a-3} H{W+0.8:.2f}" stroke="#141823" stroke-opacity="0.6" stroke-width="0.4"/>
    <rect x="{-W-0.4:.2f}" y="{(sc_a+sc_b)/2-2:.1f}" width="{2*W+0.8:.2f}" height="4" rx="1" fill="url(#pewterH)" stroke="#141823" stroke-width="0.45"/>
    <path d="M0 {(sc_a+sc_b)/2-1.2:.1f} L1.4 {(sc_a+sc_b)/2:.1f} L0 {(sc_a+sc_b)/2+1.2:.1f} L-1.4 {(sc_a+sc_b)/2:.1f} Z" fill="url(#gem)"/>
    <path d="M{-W*0.86-0.6:.2f} {sc_b+12} L{W*0.86+0.6:.2f} {sc_b+12} L{W*0.5:.2f} {sc_b+3} Q0 {sc_b-3.5} {-W*0.5:.2f} {sc_b+3} Z" fill="url(#pewter)" stroke="#141823" stroke-width="0.5"/>"""
    return wrap(f"""
  <path d="M30 98 L80 48" stroke="{FOCUS}" stroke-opacity="0.18" stroke-width="10" filter="url(#soft)"/>
  <g filter="url(#shadow)" transform="translate(82 46) rotate(225)">
    {sw}
    {scab}
  </g>
  <g>
    <circle cx="70.5" cy="57.5" r="6" fill="url(#flare)"/>
    {star(70.5, 57.5, 8.5, 0.11, rot=0)}
    {star(70.5, 57.5, 3.5, 0.22, rot=45, op=0.8)}
  </g>
  <g fill="{SAND}">
    <circle cx="61" cy="57" r="0.7" fill-opacity="0.8"/><circle cx="66" cy="50" r="0.5" fill-opacity="0.6"/>
    <circle cx="75" cy="64" r="0.6" fill-opacity="0.7"/><circle cx="58" cy="64" r="0.45" fill-opacity="0.5"/>
    <circle cx="79" cy="56" r="0.4" fill-opacity="0.6"/>
  </g>""", "Engage - Unsheathed")


# ---------- 5. Dial Edge ----------
def dial_edge():
    R = 40
    ticks = []
    for i in range(60):
        a = math.radians(i * 6)
        sx, sy = math.sin(a), -math.cos(a)
        if i % 5:
            r0, r1, w, o = R - 4.2, R - 2.4, 0.4, 0.45
            ticks.append(f'<path d="M{64+sx*r0:.2f} {64+sy*r0:.2f} L{64+sx*r1:.2f} {64+sy*r1:.2f}" stroke="{PALE}" stroke-opacity="{o}" stroke-width="{w}"/>')
        elif i % 15:
            r0, r1 = R - 6, R - 2.2
            ticks.append(f'<path d="M{64+sx*r0:.2f} {64+sy*r0:.2f} L{64+sx*r1:.2f} {64+sy*r1:.2f}" stroke="{PALE}" stroke-opacity="0.85" stroke-width="0.9"/>')
        else:
            cx, cy = 64 + sx * (R - 4.6), 64 + sy * (R - 4.6)
            ticks.append(f'<path d="M{cx+sx*2.6:.2f} {cy+sy*2.6:.2f} L{cx-sy*1.5:.2f} {cy+sx*1.5:.2f} L{cx-sx*2.6:.2f} {cy-sy*2.6:.2f} L{cx+sy*1.5:.2f} {cy-sx*1.5:.2f} Z" fill="url(#pewter)" stroke="#141823" stroke-width="0.3"/>')
    # gold arc of elapsed time 12 -> ~4 o'clock
    a0, a1 = 0, 125
    ra = R - 9
    p0 = (64 + ra * math.sin(math.radians(a0)), 64 - ra * math.cos(math.radians(a0)))
    p1 = (64 + ra * math.sin(math.radians(a1)), 64 - ra * math.cos(math.radians(a1)))
    arc = f'<path d="M{p0[0]:.2f} {p0[1]:.2f} A{ra} {ra} 0 0 1 {p1[0]:.2f} {p1[1]:.2f}" fill="none" stroke="url(#sand)" stroke-width="1.6" stroke-linecap="round"/>'
    arcg = f'<path d="M{p0[0]:.2f} {p0[1]:.2f} A{ra} {ra} 0 0 1 {p1[0]:.2f} {p1[1]:.2f}" fill="none" stroke="{SAND}" stroke-opacity="0.5" stroke-width="4" filter="url(#softS)"/>'
    sw = sword(L=86, w=9, gw=30, grip=15)
    return wrap(f"""
  <circle cx="64" cy="64" r="{R+3}" fill="#000" fill-opacity="0.35" filter="url(#soft)"/>
  <circle cx="64" cy="64" r="{R}" fill="#0b0e16"/>
  <circle cx="64" cy="64" r="{R-1.6}" fill="none" stroke="url(#pewter)" stroke-width="3.2"/>
  <circle cx="64" cy="64" r="{R+0.4}" fill="none" stroke="#141823" stroke-width="0.6"/>
  <circle cx="64" cy="64" r="{R-3.4}" fill="none" stroke="#141823" stroke-width="0.5"/>
  <circle cx="64" cy="64" r="{R-11.5}" fill="none" stroke="{PALE}" stroke-opacity="0.12" stroke-width="0.5"/>
  {"".join(ticks)}
  {arcg}{arc}
  <circle cx="{p1[0]:.2f}" cy="{p1[1]:.2f}" r="1.5" fill="#f6e6bd"/>
  <g filter="url(#shadow)" transform="translate(36 92) rotate(45)">
    {sw}
  </g>
  <g>
    <circle cx="96.8" cy="31.2" r="5" fill="url(#flare)"/>
    {star(96.8, 31.2, 6.5, 0.12)}
  </g>""", "Engage - Dial Edge")


ICONS = [
    ("engage_1_crossed_steel", crossed_steel),
    ("engage_2_hourglass_blade", hourglass_blade),
    ("engage_3_starfall_clash", starfall_clash),
    ("engage_4_unsheathed", unsheathed),
    ("engage_5_dial_edge", dial_edge),
]

if __name__ == "__main__":
    os.makedirs(OUT, exist_ok=True)
    for name, fn in ICONS:
        with open(os.path.join(OUT, name + ".svg"), "w") as f:
            f.write(fn())
    print("ok")
