#!/usr/bin/env python3
"""Painterly toolkit for Timeless icons (pilot): textures, soft form shading, scenes, worn frame."""
import math, random

_uid = [0]


def uid(p="u"):
    _uid[0] += 1
    return f"{p}{_uid[0]}"


def reset():
    _uid[0] = 0


FILTERS = """
  <filter id="b06" x="-50%" y="-50%" width="200%" height="200%"><feGaussianBlur stdDeviation="0.6"/></filter>
  <filter id="b1" x="-50%" y="-50%" width="200%" height="200%"><feGaussianBlur stdDeviation="1"/></filter>
  <filter id="b2" x="-50%" y="-50%" width="200%" height="200%"><feGaussianBlur stdDeviation="2"/></filter>
  <filter id="b3" x="-60%" y="-60%" width="220%" height="220%"><feGaussianBlur stdDeviation="3.2"/></filter>
  <filter id="b5" x="-80%" y="-80%" width="260%" height="260%"><feGaussianBlur stdDeviation="5"/></filter>
  <filter id="b8" x="-100%" y="-100%" width="300%" height="300%"><feGaussianBlur stdDeviation="8"/></filter>
  <filter id="cast" x="-50%" y="-50%" width="200%" height="200%"><feGaussianBlur stdDeviation="2.4"/></filter>
  <filter id="mottle" x="-5%" y="-5%" width="110%" height="110%" color-interpolation-filters="sRGB">
    <feTurbulence type="fractalNoise" baseFrequency="0.55" numOctaves="3" seed="4" result="n"/>
    <feColorMatrix in="n" type="matrix" values="0.33 0.33 0.33 0 0  0.33 0.33 0.33 0 0  0.33 0.33 0.33 0 0  0 0 0 0 1" result="g"/>
    <feComponentTransfer in="g" result="gs"><feFuncR type="linear" slope="0.45" intercept="0.68"/><feFuncG type="linear" slope="0.45" intercept="0.68"/><feFuncB type="linear" slope="0.45" intercept="0.68"/></feComponentTransfer>
    <feComposite in="gs" in2="SourceGraphic" operator="in" result="gi"/>
    <feBlend in="SourceGraphic" in2="gi" mode="multiply"/>
  </filter>
  <filter id="wood" x="-5%" y="-5%" width="110%" height="110%" color-interpolation-filters="sRGB">
    <feTurbulence type="turbulence" baseFrequency="0.035 0.55" numOctaves="3" seed="9" result="n"/>
    <feColorMatrix in="n" type="matrix" values="0.33 0.33 0.33 0 0  0.33 0.33 0.33 0 0  0.33 0.33 0.33 0 0  0 0 0 0 1" result="g"/>
    <feComponentTransfer in="g" result="gs"><feFuncR type="linear" slope="-0.9" intercept="1.05"/><feFuncG type="linear" slope="-0.9" intercept="1.05"/><feFuncB type="linear" slope="-0.9" intercept="1.05"/></feComponentTransfer>
    <feComposite in="gs" in2="SourceGraphic" operator="in" result="gi"/>
    <feBlend in="SourceGraphic" in2="gi" mode="multiply"/>
  </filter>
  <filter id="woodH" x="-5%" y="-5%" width="110%" height="110%" color-interpolation-filters="sRGB">
    <feTurbulence type="turbulence" baseFrequency="0.55 0.035" numOctaves="3" seed="9" result="n"/>
    <feColorMatrix in="n" type="matrix" values="0.33 0.33 0.33 0 0  0.33 0.33 0.33 0 0  0.33 0.33 0.33 0 0  0 0 0 0 1" result="g"/>
    <feComponentTransfer in="g" result="gs"><feFuncR type="linear" slope="-0.9" intercept="1.05"/><feFuncG type="linear" slope="-0.9" intercept="1.05"/><feFuncB type="linear" slope="-0.9" intercept="1.05"/></feComponentTransfer>
    <feComposite in="gs" in2="SourceGraphic" operator="in" result="gi"/>
    <feBlend in="SourceGraphic" in2="gi" mode="multiply"/>
  </filter>
  <filter id="rock" x="-5%" y="-5%" width="110%" height="110%" color-interpolation-filters="sRGB">
    <feTurbulence type="fractalNoise" baseFrequency="0.09" numOctaves="4" seed="3" result="n"/>
    <feDiffuseLighting in="n" surfaceScale="3.2" diffuseConstant="1.15" lighting-color="#ffffff" result="l"><feDistantLight azimuth="225" elevation="48"/></feDiffuseLighting>
    <feComponentTransfer in="l" result="ls"><feFuncR type="linear" slope="0.9" intercept="0.25"/><feFuncG type="linear" slope="0.9" intercept="0.25"/><feFuncB type="linear" slope="0.9" intercept="0.25"/></feComponentTransfer>
    <feComposite in="ls" in2="SourceGraphic" operator="in" result="li"/>
    <feBlend in="SourceGraphic" in2="li" mode="multiply"/>
  </filter>
  <filter id="hammered" x="-5%" y="-5%" width="110%" height="110%" color-interpolation-filters="sRGB">
    <feTurbulence type="fractalNoise" baseFrequency="0.35" numOctaves="2" seed="11" result="n"/>
    <feDiffuseLighting in="n" surfaceScale="1.2" diffuseConstant="1.1" lighting-color="#ffffff" result="l"><feDistantLight azimuth="225" elevation="55"/></feDiffuseLighting>
    <feComponentTransfer in="l" result="ls"><feFuncR type="linear" slope="0.55" intercept="0.5"/><feFuncG type="linear" slope="0.55" intercept="0.5"/><feFuncB type="linear" slope="0.55" intercept="0.5"/></feComponentTransfer>
    <feComposite in="ls" in2="SourceGraphic" operator="in" result="li"/>
    <feBlend in="SourceGraphic" in2="li" mode="multiply"/>
  </filter>
  <filter id="organic" x="-10%" y="-10%" width="120%" height="120%">
    <feTurbulence type="fractalNoise" baseFrequency="0.06" numOctaves="2" seed="5" result="n"/>
    <feDisplacementMap in="SourceGraphic" in2="n" scale="5" xChannelSelector="R" yChannelSelector="G"/>
  </filter>
  <filter id="flameD" x="-20%" y="-20%" width="140%" height="140%">
    <feTurbulence type="fractalNoise" baseFrequency="0.045 0.09" numOctaves="3" seed="21" result="n"/>
    <feDisplacementMap in="SourceGraphic" in2="n" scale="11" xChannelSelector="R" yChannelSelector="G" result="d"/>
    <feGaussianBlur in="d" stdDeviation="0.5"/>
  </filter>
  <filter id="paperT" x="0" y="0" width="100%" height="100%" color-interpolation-filters="sRGB">
    <feTurbulence type="fractalNoise" baseFrequency="0.25" numOctaves="4" seed="2" result="n"/>
    <feColorMatrix in="n" type="matrix" values="0.33 0.33 0.33 0 0  0.33 0.33 0.33 0 0  0.33 0.33 0.33 0 0  0 0 0 0 1" result="g"/>
    <feComponentTransfer in="g" result="gs"><feFuncR type="linear" slope="0.5" intercept="0.68"/><feFuncG type="linear" slope="0.5" intercept="0.66"/><feFuncB type="linear" slope="0.5" intercept="0.6"/></feComponentTransfer>
    <feComposite in="gs" in2="SourceGraphic" operator="in" result="gi"/>
    <feBlend in="SourceGraphic" in2="gi" mode="multiply"/>
  </filter>
  <filter id="grainD" x="0" y="0" width="100%" height="100%">
    <feTurbulence type="fractalNoise" baseFrequency="1.1" numOctaves="2" seed="7"/>
    <feColorMatrix type="matrix" values="0 0 0 0 0  0 0 0 0 0  0 0 0 0 0  -2.2 0 0 0 1.25"/>
  </filter>
  <filter id="grainL" x="0" y="0" width="100%" height="100%">
    <feTurbulence type="fractalNoise" baseFrequency="1.3" numOctaves="1" seed="13"/>
    <feColorMatrix type="matrix" values="0 0 0 0 1  0 0 0 0 1  0 0 0 0 1  2.4 0 0 0 -1.45"/>
  </filter>
  <clipPath id="plateClip"><rect x="3" y="3" width="122" height="122" rx="11"/></clipPath>
  <radialGradient id="vign" cx="50%" cy="46%" r="72%"><stop offset="0.55" stop-color="#000" stop-opacity="0"/><stop offset="1" stop-color="#000" stop-opacity="0.72"/></radialGradient>
  <linearGradient id="rimP" x1="0" y1="0" x2="0.3" y2="1"><stop offset="0" stop-color="#d7d2c4"/><stop offset="0.35" stop-color="#8f8a80"/><stop offset="0.7" stop-color="#4e4a46"/><stop offset="1" stop-color="#262321"/></linearGradient>
  <radialGradient id="soft" cx="50%" cy="50%" r="50%"><stop offset="0" stop-color="#fff" stop-opacity="1"/><stop offset="1" stop-color="#fff" stop-opacity="0"/></radialGradient>
"""


def lg(id_, stops, x1=0, y1=0, x2=0, y2=1, units=""):
    u = f' gradientUnits="{units}"' if units else ""
    st = "".join(f'<stop offset="{o}" stop-color="{c}"' + (f' stop-opacity="{a}"' if a is not None else "") + "/>"
                 for o, c, a in [(s + (None,))[:3] if len(s) == 2 else s for s in stops])
    return f'<linearGradient id="{id_}" x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}"{u}>{st}</linearGradient>'


def rg(id_, stops, cx="50%", cy="50%", r="50%", fx=None, fy=None, units=""):
    u = f' gradientUnits="{units}"' if units else ""
    f = (f' fx="{fx}" fy="{fy}"' if fx is not None else "")
    st = "".join(f'<stop offset="{o}" stop-color="{c}"' + (f' stop-opacity="{a}"' if a is not None else "") + "/>"
                 for o, c, a in [(s + (None,))[:3] if len(s) == 2 else s for s in stops])
    return f'<radialGradient id="{id_}" cx="{cx}" cy="{cy}" r="{r}"{f}{u}>{st}</radialGradient>'


def blob(x, y, rx, ry, col, op=1.0, blur="b3", rot=0):
    t = f' transform="rotate({rot} {x} {y})"' if rot else ""
    return f'<ellipse cx="{x}" cy="{y}" rx="{rx}" ry="{ry}" fill="{col}" fill-opacity="{op}" filter="url(#{blur})"{t}/>'


def form(d, base, lo, hi=None, hl=None, rim=None, rim_op=0.55, off=(2.4, 3.0), blur="b2", outline="#140f0c",
         ow=0.55, tex=None, ao=0.45, extra="", tf=""):
    """Soft painted volume: dark base, offset light mass, highlight blob, bounce rim, AO edge, texture."""
    c = uid("c")
    dx, dy = off
    s = [f'<clipPath id="{c}"><path d="{d}" clip-rule="evenodd"/></clipPath>', f'<path d="{d}" fill="{lo}" fill-rule="evenodd"/>', f'<g clip-path="url(#{c})">',
         f'<path d="{d}" fill="{base}" fill-rule="evenodd" transform="translate({-dx} {-dy})" filter="url(#{blur})"/>']
    if hl:
        x, y, rx, ry = hl[:4]
        rot = hl[4] if len(hl) > 4 else 0
        s.append(blob(x, y, rx, ry, hi or "#fff", 0.75, "b3", rot))
    if rim:
        s.append(f'<path d="{d}" fill="none" stroke="{rim}" stroke-opacity="{rim_op}" stroke-width="1.6" transform="translate({-dx*0.45:.2f} {-dy*0.45:.2f})" filter="url(#b06)"/>')
    if ao:
        s.append(f'<path d="{d}" fill="none" stroke="#000" stroke-opacity="{ao}" stroke-width="2.4" filter="url(#b1)"/>')
    s.append(extra)
    s.append("</g>")
    if outline:
        s.append(f'<path d="{d}" fill="none" stroke="{outline}" stroke-opacity="0.85" stroke-width="{ow}" stroke-linejoin="round"/>')
    body = "".join(s)
    if tex:
        body = f'<g filter="url(#{tex})">{body}</g>'
    if tf:
        body = f'<g transform="{tf}">{body}</g>'
    return body


def cast_shadow(d, dx=3, dy=4, op=0.55, tf=""):
    t = f' transform="{tf} translate({dx} {dy})"' if tf else f' transform="translate({dx} {dy})"'
    return f'<path d="{d}" fill="#000" fill-opacity="{op}" filter="url(#cast)"{t}/>'


def sparkle(x, y, r, col="#fff6dc", op=0.95):
    return (f'<circle cx="{x}" cy="{y}" r="{r*1.8:.2f}" fill="{col}" fill-opacity="{op*0.35:.2f}" filter="url(#b1)"/>'
            f'<path d="M{x} {y-r} Q{x+r*0.12:.2f} {y-r*0.12:.2f} {x+r} {y} Q{x+r*0.12:.2f} {y+r*0.12:.2f} {x} {y+r} '
            f'Q{x-r*0.12:.2f} {y+r*0.12:.2f} {x-r} {y} Q{x-r*0.12:.2f} {y-r*0.12:.2f} {x} {y-r} Z" fill="{col}" fill-opacity="{op}"/>')


def motes(seed, n, box, cols, rr=(0.5, 1.3), glow=True):
    rnd = random.Random(seed)
    x0, y0, x1, y1 = box
    out = []
    for _ in range(n):
        x, y, r = rnd.uniform(x0, x1), rnd.uniform(y0, y1), rnd.uniform(*rr)
        c = rnd.choice(cols)
        o = rnd.uniform(0.45, 1.0)
        if glow:
            out.append(f'<circle cx="{x:.1f}" cy="{y:.1f}" r="{r*2.6:.2f}" fill="{c}" fill-opacity="{o*0.3:.2f}" filter="url(#b1)"/>')
        out.append(f'<circle cx="{x:.1f}" cy="{y:.1f}" r="{r:.2f}" fill="{c}" fill-opacity="{o:.2f}"/>')
    return "".join(out)


def stars(seed, n, box=(6, 6, 122, 60), col="#f2f0ff"):
    rnd = random.Random(seed)
    x0, y0, x1, y1 = box
    return "".join(f'<circle cx="{rnd.uniform(x0,x1):.1f}" cy="{rnd.uniform(y0,y1):.1f}" r="{rnd.uniform(0.2,0.6):.2f}" fill="{col}" fill-opacity="{rnd.uniform(0.3,0.9):.2f}"/>' for _ in range(n))


def frame(accent="#d6c9a8"):
    """Worn pewter frame with the logo's hairlines and diamonds, plus inner shadow and film grain."""
    return f"""
  <g clip-path="url(#plateClip)">
    <rect x="3" y="3" width="122" height="122" fill="url(#vign)"/>
    <rect x="3" y="3" width="122" height="122" rx="11" fill="none" stroke="#000" stroke-opacity="0.75" stroke-width="9" filter="url(#b3)"/>
    <rect x="3" y="3" width="122" height="122" filter="url(#grainD)" opacity="0.13"/>
    <rect x="3" y="3" width="122" height="122" filter="url(#grainL)" opacity="0.07"/>
  </g>
  <rect x="3" y="3" width="122" height="122" rx="11" fill="none" stroke="#050403" stroke-width="3.2"/>
  <g filter="url(#mottle)"><rect x="3.6" y="3.6" width="120.8" height="120.8" rx="10.4" fill="none" stroke="url(#rimP)" stroke-width="2.2"/></g>
  <rect x="6.2" y="6.2" width="115.6" height="115.6" rx="8" fill="none" stroke="#000" stroke-opacity="0.6" stroke-width="0.8"/>
  <g fill="none" stroke="{accent}" stroke-opacity="0.32" stroke-width="0.6"><path d="M46 8 H58 M70 8 H82"/><path d="M46 120 H58 M70 120 H82"/></g>
  <g fill="{accent}" fill-opacity="0.75"><path d="M64 5.4 L66.4 7.8 L64 10.2 L61.6 7.8 Z"/><path d="M64 117.8 L66.4 120.2 L64 122.6 L61.6 120.2 Z"/></g>"""


def icon(title, scene, body, defs="", accent="#d6c9a8"):
    reset_ok = True
    return f"""<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 128 128" width="128" height="128">
  <title>{title}</title>
  <defs>{FILTERS}{defs}
  </defs>
  <g clip-path="url(#plateClip)">
  {scene}
  {body}
  </g>
{frame(accent)}
</svg>
"""


# ---- material presets: (base, lo, hi, rim, outline, tex) ----
STEEL = dict(base="#9aa4b4", lo="#2a303c", hi="#eae6dc", rim="#86a8d8", outline="#11141b", tex="mottle")
IRON = dict(base="#6e7078", lo="#1c1c22", hi="#c8c0b0", rim="#e08a48", outline="#0e0e12", tex="hammered")
WOOD = dict(base="#9a6438", lo="#3a2212", hi="#d8a26a", rim="#f0b070", outline="#1e1008", tex="wood")
WOODH = dict(WOOD, tex="woodH")
LEATHER = dict(base="#8a5230", lo="#2e170a", hi="#c88c58", rim="#e8a070", outline="#1a0c05", tex="mottle")
BRASS = dict(base="#c09040", lo="#4a2e0c", hi="#ffe6a8", rim="#ffd080", outline="#24160a", tex="mottle")
PARCH = dict(base="#dcc79a", lo="#8a7048", hi="#fff4dc", rim="#ffe0a8", outline="#4a3418", tex="paperT")
