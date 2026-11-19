#!/usr/bin/env python3
"""Round 2: Crossed Steel refinements (FFXI-style swords, true logo hourglass)."""
import math, os, sys
sys.path.insert(0, os.path.dirname(__file__))
from gen import defs_common, plate_back, plate_front, OUT

EXTRA = """
  <linearGradient id="fInL" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#ffffff"/><stop offset="0.45" stop-color="#e3e8f0"/><stop offset="0.8" stop-color="#b4bdcc"/><stop offset="1" stop-color="#dfe4ec"/></linearGradient>
  <linearGradient id="fOutL" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#d5dae3"/><stop offset="0.5" stop-color="#9aa3b5"/><stop offset="1" stop-color="#c3cad6"/></linearGradient>
  <linearGradient id="fInR" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#aab2c2"/><stop offset="0.5" stop-color="#6f7a90"/><stop offset="1" stop-color="#8e98aa"/></linearGradient>
  <linearGradient id="fOutR" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#6b7589"/><stop offset="0.55" stop-color="#2e3545"/><stop offset="1" stop-color="#4a556c"/></linearGradient>
  <linearGradient id="hfMetal" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#8e98aa"/><stop offset="0.45" stop-color="#f1f3f7"/><stop offset="1" stop-color="#7d8597"/></linearGradient>
  <linearGradient id="oGlass" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#dfe4ec"/><stop offset="0.5" stop-color="#aeb6c4" stop-opacity="0.45"/><stop offset="1" stop-color="#dfe4ec"/></linearGradient>
  <linearGradient id="hfDisc" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#2a303d"/><stop offset="1" stop-color="#0d1016"/></linearGradient>
  <radialGradient id="hfBack"><stop offset="0" stop-color="#6f93c8" stop-opacity="0.45"/><stop offset="0.45" stop-color="#3d5f94" stop-opacity="0.2"/><stop offset="1" stop-color="#3d5f94" stop-opacity="0"/></radialGradient>
  <radialGradient id="oCyan"><stop offset="0" stop-color="#e8fbff" stop-opacity="0.85"/><stop offset="0.35" stop-color="#7fd8f0" stop-opacity="0.35"/><stop offset="1" stop-color="#5fb8e0" stop-opacity="0"/></radialGradient>
  <linearGradient id="r13Sand" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#eef3fb" stop-opacity="0.95"/><stop offset="0.35" stop-color="#bcd0ea" stop-opacity="0.88"/><stop offset="0.75" stop-color="#7f9cc8" stop-opacity="0.85"/><stop offset="1" stop-color="#4f6a96" stop-opacity="0.85"/></linearGradient>
  <linearGradient id="oWispV" x1="0" y1="1" x2="0" y2="0"><stop offset="0" stop-color="#c3a2f0" stop-opacity="0.8"/><stop offset="1" stop-color="#9a86e0" stop-opacity="0"/></linearGradient>
  <linearGradient id="leather" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#0b0d14"/><stop offset="0.4" stop-color="#2b3347"/><stop offset="0.6" stop-color="#232a3b"/><stop offset="1" stop-color="#07090e"/></linearGradient>
"""
K = "#141823"


def blade(L, hb, hs, pt):
    """Four-facet (diamond section) blade, guard at y=0, tip at -L."""
    y0, ys = -2.2, -L + pt
    b1, b2 = hb * 0.5, hs * 0.48
    s = [
        f'<path d="M{-hb} {y0} L{-hs} {ys} L0 {-L} L{-b2} {ys} L{-b1} {y0} Z" fill="url(#fOutL)"/>',
        f'<path d="M{-b1} {y0} L{-b2} {ys} L0 {-L} L0 {y0} Z" fill="url(#fInL)"/>',
        f'<path d="M{b1} {y0} L{b2} {ys} L0 {-L} L0 {y0} Z" fill="url(#fInR)"/>',
        f'<path d="M{hb} {y0} L{hs} {ys} L0 {-L} L{b2} {ys} L{b1} {y0} Z" fill="url(#fOutR)"/>',
        # ridge + bevel lines
        f'<path d="M0 {-L+1.5} L0 {y0}" stroke="#ffffff" stroke-opacity="0.55" stroke-width="0.35"/>',
        f'<path d="M{-b1} {y0} L{-b2} {ys} L0 {-L} M{b1} {y0} L{b2} {ys} L0 {-L}" fill="none" stroke="{K}" stroke-opacity="0.35" stroke-width="0.3"/>',
        # sharpened edge highlight on lit side
        f'<path d="M{-hb+0.25} {y0-1} L{-hs+0.25} {ys} L0 {-L+0.6}" fill="none" stroke="#ffffff" stroke-opacity="0.75" stroke-width="0.4"/>',
        # sky reflection band across the blade
        f'<path d="M{-hb} {-L*0.42} L{hb} {-L*0.42-3} L{hb} {-L*0.42+1.2} L{-hb} {-L*0.42+4.2} Z" fill="#ffffff" fill-opacity="0.12"/>',
        f'<path d="M{-hb} {y0} L{-hs} {ys} L0 {-L} L{hs} {ys} L{hb} {y0} Z" fill="none" stroke="{K}" stroke-width="0.5" stroke-linejoin="round"/>',
        # ricasso
        f'<path d="M{-hb-0.2} -1.5 L{hb+0.2} -1.5 L{hb} -5.6 L{-hb} -5.6 Z" fill="url(#pewterH)" stroke="{K}" stroke-width="0.4"/>',
    ]
    return s


def gem(cx, cy, r):
    return (f'<path d="M{cx} {cy-r} L{cx+r*0.8} {cy} L{cx} {cy+r} L{cx-r*0.8} {cy} Z" fill="url(#gem)" stroke="{K}" stroke-width="0.35"/>'
            f'<path d="M{cx} {cy-r} L{cx-r*0.8} {cy} L{cx} {cy}" fill="#ffffff" fill-opacity="0.35"/>')


def grip(y, n, top_w=3.0, bot_w=2.5):
    s = [f'<path d="M{-top_w} {y} L{top_w} {y} L{bot_w} {y+n} L{-bot_w} {y+n} Z" fill="url(#leather)" stroke="#05070b" stroke-width="0.4"/>']
    steps = int(n / 2.0)
    for i in range(steps):
        yy = y + 0.6 + i * (n - 1.2) / steps
        w = top_w - (top_w - bot_w) * (yy - y) / n
        s.append(f'<path d="M{-w+0.2:.2f} {yy:.2f} Q0 {yy+1.6:.2f} {w-0.2:.2f} {yy+0.4:.2f}" fill="none" stroke="#5c6b85" stroke-opacity="0.7" stroke-width="0.45"/>')
        s.append(f'<path d="M{-w+0.2:.2f} {yy+0.5:.2f} Q0 {yy+2.1:.2f} {w-0.2:.2f} {yy+0.9:.2f}" fill="none" stroke="#000" stroke-opacity="0.5" stroke-width="0.35"/>')
    s.append(f'<rect x="{-top_w-0.5}" y="{y-0.6}" width="{2*top_w+1}" height="1.8" rx="0.5" fill="url(#pewterH)" stroke="{K}" stroke-width="0.3"/>')
    s.append(f'<rect x="{-bot_w-0.5}" y="{y+n-1.2}" width="{2*bot_w+1}" height="1.8" rx="0.5" fill="url(#pewterH)" stroke="{K}" stroke-width="0.3"/>')
    return s


def sword_knight(L=82):
    """San d'Orian-style arming sword: down-swept spade quillons, langet + gem, pear pommel."""
    s = blade(L, 5.4, 4.3, 12)
    q = ("M-3.5 -2.4 C-8 -2.8 -12 -2.2 -14.6 -0.6 L-17 -0.8 L-16.4 2.8 L-14.2 2.4 "
         "C-11.5 1.6 -8 2.2 -3.5 2.4 Z")
    for sx in (1, -1):
        s.append(f'<path d="{q}" transform="scale({sx} 1)" fill="url(#pewter)" stroke="{K}" stroke-width="0.5" stroke-linejoin="round"/>')
        s.append(f'<path d="M-4 -1.9 C-8 -2 -11.5 -1.6 -14.6 0" transform="scale({sx} 1)" fill="none" stroke="#fff" stroke-opacity="0.55" stroke-width="0.4"/>')
    s.append(f'<path d="M0 -11 L4 -3 L4 2.8 L0 4.6 L-4 2.8 L-4 -3 Z" fill="url(#pewter)" stroke="{K}" stroke-width="0.5" stroke-linejoin="round"/>')
    s.append(f'<path d="M0 -9.6 L2.8 -3.2 L-2.8 -3.2 Z" fill="#ffffff" fill-opacity="0.18"/>')
    s.append(gem(0, -0.2, 2.4))
    s += grip(5.0, 13)
    # pear pommel
    s.append(f'<path d="M0 18.2 C3.6 18.6 4.6 22.4 3.4 24.6 C2.4 26.4 1 27 0 28.4 C-1 27 -2.4 26.4 -3.4 24.6 C-4.6 22.4 -3.6 18.6 0 18.2 Z" fill="url(#pewter)" stroke="{K}" stroke-width="0.5"/>')
    s.append(f'<path d="M0 18.6 L0 28" stroke="{K}" stroke-opacity="0.35" stroke-width="0.35"/>')
    s.append(f'<ellipse cx="-1.4" cy="21.2" rx="0.9" ry="1.6" fill="#fff" fill-opacity="0.55"/>')
    return s


def sword_broad(L=80):
    """Bastokan-style broadsword: wide blade, straight block guard, wheel pommel."""
    s = blade(L, 6.6, 5.6, 13)
    s.append(f'<path d="M-13 -2.6 L13 -2.6 L15.2 -3.8 L16 -0.9 L16 0.9 L15.2 3.8 L13 2.6 L-13 2.6 L-15.2 3.8 L-16 0.9 L-16 -0.9 L-15.2 -3.8 Z" fill="url(#pewter)" stroke="{K}" stroke-width="0.5" stroke-linejoin="round"/>')
    s.append(f'<path d="M-13 -1.7 H13" stroke="#fff" stroke-opacity="0.5" stroke-width="0.45"/>')
    for x in (-10, -7, 7, 10):
        s.append(f'<circle cx="{x}" cy="0" r="0.75" fill="#2a3142" stroke="#fff" stroke-opacity="0.3" stroke-width="0.25"/>')
    s.append(f'<rect x="-4.4" y="-3.8" width="8.8" height="7.6" rx="1.1" fill="url(#pewter)" stroke="{K}" stroke-width="0.5"/>')
    s.append(gem(0, 0, 2.6))
    s += grip(4.4, 13, 3.1, 2.8)
    s.append(f'<circle cx="0" cy="22.6" r="4.6" fill="url(#pewter)" stroke="{K}" stroke-width="0.5"/>')
    s.append(f'<circle cx="0" cy="22.6" r="2.9" fill="url(#hfDisc)" stroke="#fff" stroke-opacity="0.35" stroke-width="0.4"/>')
    s.append(gem(0, 22.6, 1.8))
    s.append(f'<rect x="-1.2" y="26.8" width="2.4" height="2" rx="0.6" fill="url(#pewterH)"/>')
    return s


def sword_elegant(L=86):
    """Elvaan-style knightly blade: slender, up-swept quillons with finials, faceted ball pommel."""
    s = blade(L, 4.4, 3.4, 11)
    q = "M-3 -1.4 C-7 -1.2 -10 -2.6 -11.8 -5.6 L-10.6 -6.4 C-8.8 -4 -6.6 -3.2 -3 -3.4 Z"
    for sx in (1, -1):
        s.append(f'<path d="{q}" transform="scale({sx} 1)" fill="url(#pewter)" stroke="{K}" stroke-width="0.45" stroke-linejoin="round"/>')
        s.append(f'<circle cx="{sx*11.6}" cy="-6.6" r="1.5" fill="url(#pewter)" stroke="{K}" stroke-width="0.4"/>')
        s.append(f'<circle cx="{sx*11.6-0.4}" cy="-7.0" r="0.5" fill="#fff" fill-opacity="0.7"/>')
    s.append(f'<path d="M-5 -1 C-5 -3.2 5 -3.2 5 -1 C5 2.4 2 3 0 4.8 C-2 3 -5 2.4 -5 -1 Z" fill="url(#pewter)" stroke="{K}" stroke-width="0.5"/>')
    s.append(gem(0, 0.2, 2.1))
    s += grip(5.0, 14, 2.4, 2.1)
    s.append(f'<path d="M0 19.6 L3.4 21.6 L3.4 24.8 L0 26.8 L-3.4 24.8 L-3.4 21.6 Z" fill="url(#pewter)" stroke="{K}" stroke-width="0.45" stroke-linejoin="round"/>')
    s.append(f'<path d="M0 19.6 L0 23.2 L-3.4 21.6 M0 23.2 L3.4 21.6 M0 23.2 L0 26.8" fill="none" stroke="{K}" stroke-opacity="0.4" stroke-width="0.3"/>')
    s.append(f'<path d="M0 19.6 L-3.4 21.6 L0 23.2 Z" fill="#fff" fill-opacity="0.35"/>')
    s.append(f'<circle cx="0" cy="28" r="1.1" fill="url(#pewter)"/>')
    return s


GLASS = "M-40 -78 C-40 -30 -6 -14 -3 0 C-6 14 -40 30 -40 78 L40 78 C40 30 6 14 3 0 C6 -14 40 -30 40 -78 Z"


def logo_hourglass(w=7.0):
    """The hero-logo hourglass, logo units (stroke weights thickened for icon scale)."""
    crys = []
    k = 1.5
    top, bot, hw = -23.5 * k, 16 * k, 5.2 * k
    crys.append(f'<g transform="translate(0 -46)">')
    crys.append(f'<circle r="34" fill="url(#oCyan)"/>')
    crys.append(f'<polygon points="0,{top} {-hw},0 0,{hw*0.35} " fill="#80bde7" fill-opacity="0.95"/>')
    crys.append(f'<polygon points="0,{top} {hw},0 0,{hw*0.35}" fill="#5588c1" fill-opacity="0.95"/>')
    crys.append(f'<polygon points="0,{bot} {-hw},0 0,{hw*0.35}" fill="#68a0d2" fill-opacity="0.95"/>')
    crys.append(f'<polygon points="0,{bot} {hw},0 0,{hw*0.35}" fill="#3f6fa8" fill-opacity="0.95"/>')
    crys.append(f'<path d="M0 {top} L{-hw} 0 L0 {bot} L{hw} 0 Z M{-hw} 0 L0 {hw*0.35} L{hw} 0 M0 {top} L0 {hw*0.35}" fill="none" stroke="#fff" stroke-opacity="0.55" stroke-width="1.6" stroke-linejoin="round"/>')
    crys.append('</g>')
    s = [f'<clipPath id="lgc"><path d="{GLASS}"/></clipPath>',
         '<ellipse rx="70" ry="96" fill="url(#hfBack)"/>',
         f'<path d="{GLASS}" fill="#bcd3ee" fill-opacity="0.07"/>',
         '<g clip-path="url(#lgc)">',
         '<path d="M-14 -14 C-18 -32 -8 -44 -13 -60" fill="none" stroke="url(#oWispV)" stroke-width="5" stroke-opacity="0.6"/>',
         '<path d="M12 -12 C16 -30 8 -42 12 -58" fill="none" stroke="url(#oWispV)" stroke-width="4" stroke-opacity="0.45"/>',
         *crys,
         '<path d="M0 0 L0 54" stroke="#e6d3a3" stroke-opacity="0.85" stroke-width="2.4"/>',
         '<path d="M-46 80 L-46 71 Q-24 70 -12 60 Q-4 52 0 51 Q4 52 12 60 Q24 70 46 71 L46 80 Z" fill="url(#r13Sand)"/>',
         '<path d="M-26 68 Q-12 62 -4 54" fill="none" stroke="#fff" stroke-opacity="0.5" stroke-width="1.6"/>',
         '<circle cx="-1.5" cy="20" r="2" fill="#e6d3a3"/><circle cx="1.2" cy="34" r="1.8" fill="#9fbbe0"/>',
         '<path d="M-31 -70 C-31 -40 -16 -26 -10 -16" fill="none" stroke="#fff" stroke-opacity="0.4" stroke-width="3" stroke-linecap="round"/>',
         '</g>',
         f'<path d="M-40 -78 C-40 -30 -6 -14 -3 0 C-6 14 -40 30 -40 78" fill="none" stroke="url(#oGlass)" stroke-width="{w}"/>',
         f'<path d="M40 -78 C40 -30 6 -14 3 0 C6 14 40 30 40 78" fill="none" stroke="url(#oGlass)" stroke-width="{w}"/>']
    for sy in (1, -1):
        s.append(f'<g transform="scale(1 {sy})">'
                 f'<line x1="-54" y1="-80" x2="54" y2="-80" stroke="url(#hfMetal)" stroke-width="{w*1.7:.1f}" stroke-linecap="round"/>'
                 f'<line x1="-46" y1="-88" x2="46" y2="-88" stroke="#dfe4ec" stroke-opacity="0.85" stroke-width="{w*1.1:.1f}" stroke-linecap="round"/>'
                 f'<line x1="-32" y1="-95" x2="32" y2="-95" stroke="#dfe4ec" stroke-opacity="0.65" stroke-width="{w*0.85:.1f}" stroke-linecap="round"/>'
                 '</g>')
    return "\n    ".join(s)


def icon(sword_fn, title, ang=40, off=22, med_r=15.5):
    sw = "\n    ".join(sword_fn())
    hg = logo_hourglass()
    sc = (med_r - 3.2) * 2 / 196
    return f"""<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 128 128" width="128" height="128">
  <title>{title}</title>
  <defs>{defs_common()}{EXTRA}
  </defs>
{plate_back()}
  <g clip-path="url(#plateClip)">
  <g filter="url(#shadow)">
    <g transform="translate(64 64) rotate({-ang}) translate(0 {off})">
    {sw}
    </g>
    <g transform="translate(64 64) rotate({ang}) translate(0 {off})">
    {sw}
    </g>
  </g>
  <circle cx="64" cy="61" r="{med_r+9}" fill="url(#hfBack)"/>
  <g filter="url(#shadow)">
    <circle cx="64" cy="61" r="{med_r}" fill="url(#hfDisc)"/>
    <circle cx="64" cy="61" r="{med_r-0.6}" fill="none" stroke="url(#hfMetal)" stroke-width="1.5"/>
    <circle cx="64" cy="61" r="{med_r-2.4}" fill="none" stroke="#dfe4ec" stroke-opacity="0.22" stroke-width="0.45"/>
    <circle cx="64" cy="61" r="{med_r+0.4}" fill="none" stroke="#05070b" stroke-width="0.7"/>
    <g transform="translate(64 61) scale({sc:.4f})">
    {hg}
    </g>
  </g>
  </g>
{plate_front()}
</svg>
"""


VARS = [
    ("engage_1a_knight", lambda: sword_knight(86), "Engage 1a - Knight's Arming Sword", 40, 27),
    ("engage_1b_broad", lambda: sword_broad(84), "Engage 1b - Broadsword", 40, 27),
    ("engage_1c_elegant", lambda: sword_elegant(90), "Engage 1c - Knightly Blade", 38, 28),
]
if __name__ == "__main__":
    for name, fn, title, ang, off in VARS:
        open(os.path.join(OUT, name + ".svg"), "w").write(icon(fn, title, ang, off))
    print("ok")
