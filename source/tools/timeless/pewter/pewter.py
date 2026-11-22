"""Timeless View · Pewter Hairline: the full menu / misc icon set (one glyph per pack icon id)."""
import os
import math, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from glyphs import G as G0, P, circ, star, sword, rot

def rr(x, y, w, h, r):
    return (f"M{x+r:.2f} {y:.2f} L{x+w-r:.2f} {y:.2f} Q{x+w:.2f} {y:.2f} {x+w:.2f} {y+r:.2f} L{x+w:.2f} {y+h-r:.2f} "
            f"Q{x+w:.2f} {y+h:.2f} {x+w-r:.2f} {y+h:.2f} L{x+r:.2f} {y+h:.2f} Q{x:.2f} {y+h:.2f} {x:.2f} {y+h-r:.2f} "
            f"L{x:.2f} {y+r:.2f} Q{x:.2f} {y:.2f} {x+r:.2f} {y:.2f} Z")
def ring(cx, cy, ro, ri): return circ(cx, cy, ro) + " " + circ(cx, cy, ri)
def ell(cx, cy, rx, ry): return f"M{cx-rx:.2f} {cy:.2f} A{rx} {ry} 0 1 0 {cx+rx:.2f} {cy:.2f} A{rx} {ry} 0 1 0 {cx-rx:.2f} {cy:.2f} Z"
def L(*pts): return "M" + " L".join(f"{x:.2f} {y:.2f}" for x, y in pts)
def S(d): return ("shape", d)
def Ln(d): return ("line", d)
def D(d): return ("dot", d)
def Dash(d): return ("dash", d)
def grp(elems, s=1.0, tx=0.0, ty=0.0, r=0.0, rc=(32, 32)):
    t = f"translate({tx:.2f} {ty:.2f}) scale({s})"
    if r: t = f"rotate({r} {rc[0]} {rc[1]}) " + t
    return ("g", t, elems, s)
def head(x, y, ang):  # arrowhead with tip at (x,y) pointing along ang (deg)
    a = math.radians(ang)
    pts = [(x, y), (x - 9 * math.cos(a) + 5.5 * math.sin(a), y - 9 * math.sin(a) - 5.5 * math.cos(a)),
           (x - 9 * math.cos(a) - 5.5 * math.sin(a), y - 9 * math.sin(a) + 5.5 * math.cos(a))]
    return S(P(pts))
def arc(cx, cy, r, a0, a1):
    p0 = (cx + r * math.cos(math.radians(a0)), cy + r * math.sin(math.radians(a0)))
    p1 = (cx + r * math.cos(math.radians(a1)), cy + r * math.sin(math.radians(a1)))
    large = 1 if (a1 - a0) % 360 > 180 else 0
    return f"M{p0[0]:.2f} {p0[1]:.2f} A{r} {r} 0 {large} 1 {p1[0]:.2f} {p1[1]:.2f}", p1
def bust(cx, top, s=1.0):
    h = 7.5 * s
    return [S(f"M{cx-13*s:.2f} {top+38*s:.2f} C{cx-13*s:.2f} {top+27*s:.2f} {cx-7.5*s:.2f} {top+21.5*s:.2f} {cx:.2f} {top+21.5*s:.2f} "
              f"C{cx+7.5*s:.2f} {top+21.5*s:.2f} {cx+13*s:.2f} {top+27*s:.2f} {cx+13*s:.2f} {top+38*s:.2f} Z"),
            S(circ(cx, top + h + 3 * s, h))]
EYE = [S("M6 32 C14 20 22 15 32 15 C42 15 50 20 58 32 C50 44 42 49 32 49 C22 49 14 44 6 32 Z"), S(circ(32, 32, 9.5)), D(circ(32, 32, 4)), D(circ(35.5, 28.5, 1.6))]
def hourglass(cx, cy, w, h, sand=True):
    g = (f"M{cx-w:.2f} {cy-h:.2f} C{cx-w:.2f} {cy-h*0.4:.2f} {cx-1.4:.2f} {cy-h*0.18:.2f} {cx-0.7:.2f} {cy:.2f} C{cx-1.4:.2f} {cy+h*0.18:.2f} {cx-w:.2f} {cy+h*0.4:.2f} {cx-w:.2f} {cy+h:.2f} "
         f"M{cx+w:.2f} {cy-h:.2f} C{cx+w:.2f} {cy-h*0.4:.2f} {cx+1.4:.2f} {cy-h*0.18:.2f} {cx+0.7:.2f} {cy:.2f} C{cx+1.4:.2f} {cy+h*0.18:.2f} {cx+w:.2f} {cy+h*0.4:.2f} {cx+w:.2f} {cy+h:.2f}")
    cw = w + 3.5
    out = [Ln(g), S(rr(cx - cw, cy - h - 4.5, 2 * cw, 4.5, 1.2)), S(rr(cx - cw, cy + h, 2 * cw, 4.5, 1.2))]
    if sand:
        out.append(D(f"M{cx-w*0.72:.2f} {cy+h-0.5:.2f} C{cx-w*0.4:.2f} {cy+h*0.62:.2f} {cx+w*0.4:.2f} {cy+h*0.62:.2f} {cx+w*0.72:.2f} {cy+h-0.5:.2f} Z"))
        out.append(D(rr(cx - 0.8, cy + 1, 1.6, h * 0.55, 0.8)))
    return out
SHIELD = "M32 7 L53 13.5 C53 34 46.5 48 32 57.5 C17.5 48 11 34 11 13.5 Z"
SHIELD_IN = "M32 12.5 L47.5 17.4 C47.5 33 42.5 44 32 51.4 C21.5 44 16.5 33 16.5 17.4 Z"
def crosshair(cx=32, cy=32, r=20):
    return [Ln(circ(cx, cy, r)), Ln(circ(cx, cy, r * 0.45)),
            Ln(f"M{cx} {cy-r-6} L{cx} {cy-r+6} M{cx} {cy+r-6} L{cx} {cy+r+6} M{cx-r-6} {cy} L{cx-r+6} {cy} M{cx+r-6} {cy} L{cx+r+6} {cy}"),
            D(circ(cx, cy, 2.4))]
BANNER = [Ln("M18 5 L18 59"), D(circ(18, 5, 2.6)), S(rr(14, 9, 38, 4, 1.5)), S("M20 13 L50 13 L50 40 L35 49 L20 40 Z"), Ln("M24 17 L46 17 L46 37.5 L35 44.5 L24 37.5 Z"), S(star(35, 28.5, 8, 2.4, 4))]
BOOT = [S("M20 6 L37 6 L37 37 C43 39 53 41 55 47 L55 54 L18 54 L18 40 Z"), Ln("M19.5 14 L37 14 M18.5 49.5 L55 49.5"), Ln("M37 37 L30 37")]
FLAME = [S("M32 5 C38 15 50 22 48 38 C47 50 40 58 32 58 C24 58 16 51 16 40 C16 30 22 26 24 17 C28 23 28 28 30 30 C32 22 30 13 32 5 Z"),
         S("M32 54 C26.5 54 23.5 49.5 24.3 44.5 C25 39.5 29 37.5 30.3 33 C33.5 38 40 40.5 39.4 47 C39 51 36 54 32 54 Z")]
DROP = "M32 6 C40 20 50 30 50 41 C50 51 42 58 32 58 C22 58 14 51 14 41 C14 30 24 20 32 6 Z"
TOME = G0["Magic"][:3]
CHEST = [S(rr(10, 30, 44, 26, 3)), Ln("M10 40 L54 40"), S(rr(28, 36, 8, 9, 1.5))]
FLAG = [Ln("M18 6 L18 58"), D(circ(18, 6, 2.6)), S("M20 9 C28 5 36 13 47 9 L47 32 C36 36 28 28 20 32 Z"), Ln("M10 58 L26 58")]
def chev(dirn, cx=32, cy=32, s=1.0):
    a = {"r": 0, "l": 180, "u": -90, "d": 90}[dirn]
    pts = [(-5 * s, -11 * s), (6 * s, 0), (-5 * s, 11 * s)]
    pts = rot(pts, math.radians(a), cx, cy)
    return Ln(L(*pts))
def arrow(x1, y1, x2, y2):
    ang = math.degrees(math.atan2(y2 - y1, x2 - x1)); a = math.radians(ang)
    return [Ln(L((x1, y1), (x2 - 6 * math.cos(a), y2 - 6 * math.sin(a)))), head(x2, y2, ang)]
def print_(cx, cy, r=0):
    return grp([S(ell(32, 26, 7, 12)), S(circ(32, 44, 5))], 1.0, 0, 0, r, (32, 32)) if False else \
        ("g", f"translate({cx-32} {cy-32}) rotate({r} 32 32)", [S(ell(32, 26, 6.5, 11.5)), S(circ(32, 44.5, 5))], 1.0)

I = {}
for k, v in {"items": "Items", "magic": "Magic", "abilities": "Abilities", "equipment": "Equipment", "party": "Party", "linkshell": "Linkshell",
             "map": "Map", "config": "Config", "trade": "Trade", "attack": "Attack", "check": "Check", "chat": "Chat", "key": "Key Items",
             "quest_log": "Journal", "mog_house": "Mog House", "companion": "Companion"}.items():
    I[k] = G0[v]
I["job"] = BANNER
I["subjob"] = [grp(BANNER, 0.7, -2, -2), grp(BANNER, 0.7, 20, 18)]
I["status"] = [S(rr(8, 11, 48, 42, 3.5))] + bust(23, 15, 0.82) + [Ln("M37 24 L50 24 M37 31 L50 31 M37 38 L46 38")]
I["system"] = [S("M32 4 L47 30 L32 60 L17 30 Z"), Ln("M32 4 L28.5 31 L32 60 M17 30 L28.5 31 L47 30"), S(star(52, 12, 6, 1.8, 4)), D(circ(12, 50, 1.6))]
I["find"] = G0["Check"] + [S(star(27, 27, 6, 1.8, 4))]
I["sword"] = sword(32, 33, 40)
I["shield"] = [S(ring(32, 32, 25, 0.01)), Ln(circ(32, 32, 18.5)), S(circ(32, 32, 6.5))] + [D(circ(32 + 22 * math.cos(math.radians(a)), 32 + 22 * math.sin(math.radians(a)), 1.6)) for a in range(0, 360, 45)]
I["fire"] = FLAME
arm = "M32 32 C36 30 43 28 45.5 19 C47 13 43 8.5 37.5 9.5"
I["ice"] = [grp([Ln(arm)], 1.0, 0, 0, a) for a in (0, 120, 240)] + [S(star(32, 32, 6, 2, 6)), S(star(50, 44, 4.5, 1.4, 4)), S(star(14, 40, 4.5, 1.4, 4)), S(star(30, 10, 4, 1.2, 4))]
I["wind"] = [Ln("M6 24 C20 24 34 23 41 17 C45.5 13 43.5 6.5 38.5 7 C34.5 7.5 34 12.5 37.5 14"),
             Ln("M6 34 L46 34 C54 34 58.5 40 55.5 46 C52.5 51 46 50 45.5 45.5"), Ln("M12 44 C21 44 28 46 30 53")]
I["earth"] = [S("M10 52 L19 24 L32 13 L46 21 L55 52 Z"), Ln("M19 24 L30 31 L32 13 M30 31 L46 21 M30 31 L36 52 M21 42 L30 31"), Ln("M5 56 L59 56")]
I["lightning"] = [S("M37 4 L15 36 L30 36 L23 60 L49 24 L34 24 L43 4 Z")]
I["water"] = [S(DROP), Ln("M21.5 42 C21.5 36 24.5 31.5 28 28.5"), Ln("M10 59 C18 56 46 56 54 59")]
I["light"] = [S(circ(32, 32, 11))] + [Ln(L((32 + 16 * math.cos(math.radians(a)), 32 + 16 * math.sin(math.radians(a))),
                                         (32 + (27 if a % 90 == 0 else 22) * math.cos(math.radians(a)), 32 + (27 if a % 90 == 0 else 22) * math.sin(math.radians(a))))) for a in range(0, 360, 45)] + [Ln(circ(32, 32, 6.5))]
I["dark"] = [S("M40 7 C26 9 15 20 15 34 C15 48 27 58 42 56 C32 52 26 44 26 33 C26 22 32 13 40 7 Z"), S(star(46, 22, 5, 1.5, 4)), S(star(52, 40, 3.5, 1.1, 4)), D(circ(38, 50, 1.4))]
I["healing"] = [S("M30 47 C21 41 13 35 9 24 C19 26 27 34 32 47 Z"), S("M34 47 C43 41 51 35 55 24 C45 26 37 34 32 47 Z"),
                S("M32 12 C38.5 22 38.5 36 32 47 C25.5 36 25.5 22 32 12 Z"), Ln("M12 50 C22 54.5 42 54.5 52 50")]
I["cure"] = [S(circ(32, 32, 24)), S(P([(28, 17), (36, 17), (36, 28), (47, 28), (47, 36), (36, 36), (36, 47), (28, 47), (28, 36), (17, 36), (17, 28), (28, 28)])), S(star(51, 13, 5.5, 1.6, 4))]
UP = P([(32, 18), (41, 29), (35.5, 29), (35.5, 42), (28.5, 42), (28.5, 29), (23, 29)])
DOWN = P([(32, 44), (41, 33), (35.5, 33), (35.5, 20), (28.5, 20), (28.5, 33), (23, 33)])
I["enhancing"] = [S(SHIELD), S(UP)]
I["enfeebling"] = [S(SHIELD), S(DOWN)]
I["elemental"] = [Ln(circ(32, 32, 19))] + [S(circ(32 + 19 * math.cos(math.radians(a)), 32 + 19 * math.sin(math.radians(a)), 6)) for a in (-90, 0, 90, 180)] + [S(star(32, 32, 10, 3, 4, -45))]
tri1 = [(32 + 20 * math.cos(math.radians(a)), 32 + 20 * math.sin(math.radians(a))) for a in (-90, 30, 150)]
tri2 = [(32 + 20 * math.cos(math.radians(a)), 32 + 20 * math.sin(math.radians(a))) for a in (90, 210, 330)]
I["summoning"] = [Ln(circ(32, 32, 26)), Ln(circ(32, 32, 20)), Ln(L(*tri1, tri1[0])), Ln(L(*tri2, tri2[0])), D(circ(32, 32, 3))] + [D(circ(32 + 23 * math.cos(math.radians(a)), 32 + 23 * math.sin(math.radians(a)), 1.4)) for a in range(0, 360, 60)]
I["geomancy"] = [Ln("M32 43 L32 47"), S("M23 47 L41 47 L45 57 L19 57 Z"), S(circ(32, 25, 18)), Ln(circ(32, 25, 12)), Ln("M23 34 L41 16"), S(star(32, 25, 5, 1.6, 4))]
I["ninjutsu"] = [S(star(32, 32, 27, 7.5, 4, -45) + " " + circ(32, 32, 4))]
I["songs"] = [S("M20 55 C11 41 11 22 19 10 C21 8 24.5 9.5 23 13 C18.5 24 19.5 40 25 51 Z"), S("M44 55 C53 41 53 22 45 10 C43 8 39.5 9.5 41 13 C45.5 24 44.5 40 39 51 Z"),
              S(rr(17, 13, 30, 4.5, 1.5)), Ln("M27.5 17.5 L27.5 50 M32 17.5 L32 50 M36.5 17.5 L36.5 50"), S(rr(20, 50, 24, 6.5, 2))]
I["food"] = [Ln("M24 27 C20 21 28 17 24 11 M32 27 C28 21 36 17 32 9 M40 27 C36 21 44 17 40 11"), S("M26 53 L38 53 L41 58 L23 58 Z"),
             S("M9 33 L55 33 C55 45 45 54 32 54 C19 54 9 45 9 33 Z"), Ln("M13 38 L51 38")]
I["medicine"] = [S("M27 17 L27 24 C16 28 12 35.5 12 42 C12 52 21 58 32 58 C43 58 52 52 52 42 C52 35.5 48 28 37 24 L37 17 Z"), S(rr(24.5, 7, 15, 10, 2)),
                 Ln("M15 42 C21 38 28 46 34 42 C40 38 46 41 49 42"), D(circ(26, 49, 2)), D(circ(36, 51, 1.4))]
def ingot(x, y): return [S(P([(x, y + 11), (x + 5, y), (x + 22, y), (x + 27, y + 11)])), Ln(L((x + 5, y), (x + 7, y + 4), (x + 20, y + 4), (x + 22, y)))]
I["materials"] = ingot(5, 42) + ingot(32, 42) + ingot(18.5, 29)
I["quest"] = [S(P([(32, 5), (58, 32), (32, 59), (6, 32)])), Ln(L((32, 11), (52, 32), (32, 53), (12, 32), (32, 11))), S(rr(29, 19, 6, 19, 3)), S(circ(32, 44.5, 3.4))]
def coin_stack(x, ys):
    out = []
    for y in ys:
        out += [S(f"M{x-14} {y} L{x-14} {y+3.5} A14 5 0 0 0 {x+14} {y+3.5} L{x+14} {y} A14 5 0 0 0 {x-14} {y} Z"), Ln(ell(x, y, 14, 5))]
    return out
I["currency"] = coin_stack(25, (50, 43.5, 37)) + [S(circ(44, 22, 13)), Ln(circ(44, 22, 9)), S(star(44, 22, 5, 1.6, 4))]
I["head"] = [S("M16 19 C16 12 22 7 32 7 C42 7 48 12 48 19 L48 50 C48 54 44.5 57 40 57 L24 57 C19.5 57 16 54 16 50 Z"), S(rr(20, 26, 24, 4.5, 2.2)),
             Ln("M32 7.5 L32 23 M32 33 L32 56"), D(circ(25, 42, 1.4)), D(circ(25, 47, 1.4)), D(circ(39, 42, 1.4)), D(circ(39, 47, 1.4))]
I["body"] = [S("M20 9 L26 13 C29 15.5 35 15.5 38 13 L44 9 L55 17 L51 30 L46.5 28 L46.5 54 C38 58 26 58 17.5 54 L17.5 28 L13 30 L9 17 Z"),
             Ln("M32 16 L32 54 M22 30 C26 34 30 34 32 32 M42 30 C38 34 34 34 32 32 M19 46 C26 49 38 49 45 46")]
I["hands"] = [S("M22 57 L22 41 C18 37 15.5 31 15.5 25 L15.5 15 C15.5 11.5 20 11.5 20 15 L20 26 L22 26 L22 10.5 C22 7 26.5 7 26.5 10.5 L26.5 25 L28.3 25 L28.3 8.5 C28.3 5 32.8 5 32.8 8.5 L32.8 25 L34.6 25 L34.6 10.5 C34.6 7 39 7 39 10.5 L39 31 L43 27 C45.5 24.5 49.5 27 47 31 L41 43 L41 57 Z"), Ln("M22 49 L41 49")]
I["legs"] = [S("M18 7 L46 7 L46 30 L42.5 57 L34 57 L33 23 L31 23 L30 57 L21.5 57 L18 30 Z"), Ln("M18 13.5 L46 13.5 M20 37 L29.8 37 M34.2 37 L44.5 37")]
I["feet"] = BOOT
I["ring"] = [S(ring(32, 40, 17, 12.5)), S(P([(25, 21), (32, 11), (39, 21), (32, 27)])), Ln("M25 21 L39 21 M32 11 L32 27")]
I["ear"] = [Ln("M28 5 C38 4 39 15 32 17"), Ln(circ(32, 20.5, 3.4)), S("M32 26 C40 36 42.5 44 38.5 50.5 C36 54.5 28 54.5 25.5 50.5 C21.5 44 24 36 32 26 Z"), Ln("M32 30 L32 52 M26 44 L38 44")]
I["neck"] = [Ln("M9 8 C13 31 51 31 55 8"), D(circ(15, 19, 1.3)), D(circ(21, 25, 1.3)), D(circ(43, 25, 1.3)), D(circ(49, 19, 1.3)),
             Ln("M32 30 L32 35"), S(P([(32, 35), (43, 46.5), (32, 59), (21, 46.5)])), Ln(L((32, 40), (38, 46.5), (32, 53.5), (26, 46.5), (32, 40)))]
I["waist"] = [S(rr(4, 25, 56, 14, 3)), S(rr(21, 19.5, 22, 25, 3) + " " + rr(26, 24.5, 12, 15, 1.5)), Ln("M24 32 L41 32"), D(circ(49, 32, 1.6)), D(circ(55, 32, 1.6))]
I["back"] = [S("M18 8 L46 8 C46 8 52 30 56.5 54 C46 50 40 56 32 52 C24 56 18 50 7.5 54 C12 30 18 8 18 8 Z"), Ln("M26 13 C26 30 24 42 21.5 52 M38 13 C38 30 40 42 42.5 52"), S(circ(32, 10, 3.6))]
I["bow"] = [S("M15 5 C42 14 42 50 15 59 L19 59 C47 50 47 14 19 5 Z"), Ln("M16.5 6.5 L16.5 57.5")] + arrow(7, 32, 58, 32) + [Ln("M10 32 L6 27 M10 32 L6 37 M14 32 L10 27 M14 32 L10 37")]
I["ammo"] = [Ln("M26 23 L21 6 M32 23 L32 4 M38 23 L43 6"), head(21, 5, -105), head(32, 3, -90), head(43, 5, -75), S("M20 22 L44 22 L47 58 L17 58 Z"), Ln("M18.5 30 L45.5 30 M18 50 L46 50")]
I["poison"] = [S("M21 7 L43 7 C43 21 39 33 32 44 C25 33 21 21 21 7 Z"), Ln("M27 12 C27 20 29 28 32 35"), S("M32 47 C35 51 37 53.5 37 55.5 C37 58.5 34.7 60 32 60 C29.3 60 27 58.5 27 55.5 C27 53.5 29 51 32 47 Z")]
WING = "M33 36 C26 27 15 21 4 23 C10 27 12 31 12.5 35 C16.5 33 21 33 23 35 C19 37 16.5 39.5 15.5 43.5 C21 41.5 27 39.5 33 40 Z"
I["haste"] = [grp(BOOT, 0.72, 17, 12), S(WING), Ln("M3 48 L13 48 M6 54 L16 54")]
I["protect"] = [S(SHIELD), Ln(SHIELD_IN), S(P([(29, 16), (35, 16), (35, 27), (45, 27), (45, 33), (35, 33), (35, 48), (29, 48), (29, 33), (19, 33), (19, 27), (29, 27)]))]
I["sleep"] = [S("M30 14 C19 16 11 25 11 36 C11 47 20 55 32 54 C24 50 20 43 20 35 C20 26 24 19 30 14 Z"), Ln("M33 11 L46 11 L33 25 L46 25"), Ln("M44 32 L53 32 L44 42 L53 42")]
I["silence"] = [I["chat"][0], Ln("M10 56 L54 8")]
a1, p1 = arc(32, 32, 24, 200, 330); a2, p2 = arc(32, 32, 24, 20, 150)
I["refresh"] = [Ln(a1), head(p1[0], p1[1], 330 + 90), Ln(a2), head(p2[0], p2[1], 150 + 90), S("M32 21 C36 27 40 31 40 36 C40 41 36.5 44 32 44 C27.5 44 24 41 24 36 C24 31 28 27 32 21 Z")]
I["weaponskill"] = sword(34, 34, 45) + [Ln("M7 42 C10 22 26 10 46 8"), Ln("M12 52 C15 36 25 26 38 21"), S(star(50, 8, 5, 1.5, 4))]
I["claimed"] = FLAG + [S(star(33, 20, 5, 1.6, 4))]
I["claim"] = FLAG
I["sort"] = [S(rr(7, 11, 30, 6, 2)), S(rr(7, 25, 22, 6, 2)), S(rr(7, 39, 14, 6, 2))] + arrow(48, 8, 48, 56)
I["stack"] = [S(P([(32, y - 10), (55, y), (32, y + 10), (9, y)])) for y in (42, 32, 22)]
I["split"] = [Ln("M32 59 L32 36 C32 26 22 24 16 15"), Ln("M32 36 C32 26 42 24 48 15"), head(13, 9.5, -122), head(51, 9.5, -58)]
I["deposit"] = CHEST + arrow(32, 3, 32, 28)
I["withdraw"] = CHEST + arrow(32, 27, 32, 2)
I["mog_safe"] = [S(rr(9, 9, 46, 46, 4)), Ln(rr(14, 14, 36, 36, 2)), S(circ(32, 32, 9)), Ln("M32 26 L32 29 M38 32 L35 32 M32 38 L32 35 M26 32 L29 32"), D(circ(32, 32, 2)), S(rr(6, 20, 4, 8, 1)), S(rr(6, 36, 4, 8, 1))]
I["mog_locker"] = [S(rr(13, 5, 38, 54, 3)), Ln("M32 5.5 L32 58.5 M17 11 L28 11 M17 15 L28 15 M36 11 L47 11 M36 15 L47 15"), D(circ(28.5, 34, 1.8)), D(circ(35.5, 34, 1.8))]
I["mog_satchel"] = [Ln("M14 23 C14 7 50 7 50 23"), S(rr(8, 24, 48, 31, 6)), S("M8 31 C8 26 12 22 17 22 L47 22 C52 22 56 26 56 31 L56 39 L8 39 Z"), S(rr(28, 34, 8, 9, 1.5))]
I["mog_sack"] = [S("M20 19 C8 29 6 45 12 52.5 C18 58.5 46 58.5 52 52.5 C58 45 56 29 44 19 Z"), S("M24 19 C21 12 26 6 32 8 C38 6 43 12 40 19 Z"), S(rr(19, 17, 26, 5, 2)), Ln("M20 44 L28 44 L28 50 L20 50 Z")]
I["mog_case"] = [S("M24 21 L24 12 C24 9 40 9 40 12 L40 21 L36 21 L36 14.5 L28 14.5 L28 21 Z"), S(rr(7, 20, 50, 35, 4)), Ln("M18 20.5 L18 54.5 M46 20.5 L46 54.5"), S(rr(28.5, 29, 7, 6, 1.2))]
I["wardrobe"] = [Ln("M32 18 C32 13 38 11.5 38 7.5 C38 3 30 3 30 7"), S("M32 18 L8 40 C5.5 42.5 7 45 10 45 L54 45 C57 45 58.5 42.5 56 40 Z"), Ln("M13 41 L51 41")]
I["equip_set"] = sword(32, 30, -38) + [grp([S(SHIELD), Ln(SHIELD_IN)], 0.62, 18, 22)]
SHACKLE = "M20 28 L20 20 C20 6 44 6 44 20 L44 28 L39 28 L39 20 C39 12 25 12 25 20 L25 28 Z"
LOCKBODY = [S(rr(13, 28, 38, 29, 4)), S(circ(32, 39.5, 3.6)), S(P([(30.5, 41), (33.5, 41), (34.5, 50), (29.5, 50)]))]
I["lock"] = [S(SHACKLE)] + LOCKBODY
I["unlock"] = [S("M30 20 C30 6 54 6 54 20 L54 26 L49 26 L49 20 C49 12 35 12 35 20 L35 28 L30 28 Z")] + LOCKBODY
PIN = [Ln("M32 38 L32 59"), S("M22 7 L42 7 L38 13 L38 27 C44 29 48 33 48 38 L16 38 C16 33 20 29 26 27 L26 13 Z")]
I["pin"] = PIN
I["unpin"] = PIN + [Ln("M10 56 L54 8")]
I["tell"] = [S(rr(7, 15, 50, 35, 3)), Ln("M8 17 L32 35 L56 17 M8 49 L26 32 M56 49 L38 32"), S(circ(32, 36, 4))]
I["linkshell_chat"] = [grp([I["chat"][0]], 0.62, -1, -2), grp(G0["Linkshell"], 0.62, 24, 24)]
I["emote"] = [S("M12 13 C22 9 42 9 52 13 C54 30 48 48 32 56 C16 48 10 30 12 13 Z"), S(ell(23.5, 27, 5.5, 3.2)), S(ell(40.5, 27, 5.5, 3.2)),
              Ln("M21.5 39 C27.5 46 36.5 46 42.5 39 M16 19.5 C20 17.5 26 17.5 29 20 M35 20 C38 17.5 44 17.5 48 19.5")]
I["keybind"] = [S(rr(4, 17, 56, 31, 4)), Ln("M10 25 L14 25 M19 25 L23 25 M28 25 L32 25 M37 25 L41 25 M46 25 L54 25 M10 32 L16 32 M21 32 L25 32 M30 32 L34 32 M39 32 L43 32 M48 32 L54 32 M19 40.5 L45 40.5")]
I["timer"] = [S(rr(28, 6, 8, 6, 1.5)), Ln("M32 12 L32 15"), S(circ(32, 37, 22)), Ln(circ(32, 37, 17.5)), Ln("M32 37 L32 25 M32 37 L40.5 41.5"), D(circ(32, 37, 2.4))] + \
             [D(circ(32 + 14.5 * math.cos(math.radians(a)), 37 + 14.5 * math.sin(math.radians(a)), 1.2)) for a in (0, 90, 180, 270)] + [Ln("M48 16 L52 12")]
I["cooldown"] = hourglass(32, 32, 13, 21)
ra, rp = arc(32, 32, 27, -50, 235)
I["recast"] = hourglass(32, 32, 8.5, 13.5) + [Ln(ra), head(rp[0], rp[1], 235 + 90)]
I["range"] = [Ln("M12 52 m-10 0 a10 10 0 0 1 10 -10"), Ln("M12 52 m-22 0 a22 22 0 0 1 22 -22"), D(circ(12, 52, 3))] + arrow(14, 50, 54, 10) + [D(circ(54, 10, 0.1))]
I["range"] = [Ln("M2 52 A10 10 0 0 1 12 42"), Ln("M12 30 A22 22 0 0 1 34 52"), Ln("M12 18 A34 34 0 0 1 46 52"), D(circ(12, 52, 3.4))] + arrow(15, 49, 55, 9)
I["line_of_sight"] = [grp(EYE, 0.62, -2, 12)] + [Ln(f"M{x} 32 L{x+4} 32") for x in (40, 47)] + [S(circ(56, 32, 4))]
I["aggro"] = [S(f"M{x} 6 Q{x+15} 30 {x+12} 59 Q{x+7.5} 31 {x} 6 Z") for x in (10, 23, 36)]
I["party_target"] = crosshair(32, 32, 22)[:3] + [grp(bust(32, 12, 1.0), 0.62, 12.2, 12.5)]
I["assist"] = [grp(crosshair(32, 32, 20), 0.68, 19, 0)] + bust(19, 23, 0.82)
I["follow"] = [print_(22, 40, -12), print_(42, 22, 12)]
I["mount"] = [S(rr(26, 3, 12, 11, 2.5)), S("M32 14 C44 14 52 30 52 48 L52 55 L12 55 L12 48 C12 30 20 14 32 14 Z M32 21.5 C40 21.5 45 33 45 47 L19 47 C19 33 24 21.5 32 21.5 Z"), Ln("M28 8.5 L36 8.5")]
I["sneak"] = [print_(26, 34, -14)] + [D(circ(x, y, 1.8)) for x, y in ((44, 22), (50, 14), (47, 33))]
I["invisible"] = [Dash("M19 50 C19 39 24.5 33.5 32 33.5 C39.5 33.5 45 39 45 50 Z"), Dash(circ(32, 22, 9)), S(star(51, 12, 5, 1.5, 4))]
I["raise"] = [S(WING), grp([S(WING)], 1.0, 64, 0) if False else ("g", "translate(64 0) scale(-1 1)", [S(WING)], 1.0), S(star(32, 17, 10, 2.8, 4)), Ln("M32 44 L32 56")]
I["rest"] = [S(P([(10, 51), (50, 59), (52, 54.5), (12, 46.5)])), S(P([(54, 51), (14, 59), (12, 54.5), (52, 46.5)])), grp(FLAME, 0.62, 12.2, 3)]
I["craft"] = [S("M9 21 L46 21 C52 21 56 25 58 29 C50 29 46.5 31 44.5 35 L40 35 L40 43 L48.5 52 L15.5 52 L24 43 L24 35 L20 35 C13.5 33 9 28 9 21 Z"), Ln("M9 25 L44 25"), S(star(50, 10, 6, 1.8, 4)), Ln("M40 8 L44 12")]
I["auction"] = [S(rr(7, 48, 30, 8, 2)), ("g", "rotate(-38 29 18)", [S(rr(15, 10, 28, 15, 3)), Ln("M21 10.5 L21 24.5 M37 10.5 L37 24.5")], 1.0), S(P([(33, 25), (37, 22), (56, 47), (52, 50)]))]
I["delivery"] = [S(P([(9, 21), (32, 11), (55, 21), (55, 46), (32, 56), (9, 46)])), Ln("M9 21 L32 31 L55 21 M32 31 L32 56"), Ln("M20.5 16 L43.5 26 L43.5 37")]
I["settings"] = [Ln("M8 17 L56 17 M8 32 L56 32 M8 47 L56 47"), S(circ(21, 17, 5.5)), S(circ(43, 32, 5.5)), S(circ(28, 47, 5.5))]
I["notification"] = [Ln(circ(32, 6.5, 2.6)), S("M32 9 C42 9 46 17 46 27 L46 38 L52 46 L12 46 L18 38 L18 27 C18 17 22 9 32 9 Z"), S(circ(32, 51, 4.5)), Ln("M22 38 L42 38")]
I["warning"] = [S("M32 5 C33.5 5 34.5 6 35.5 7.5 L58 50 C59.5 53 58 56 54.5 56 L9.5 56 C6 56 4.5 53 6 50 L28.5 7.5 C29.5 6 30.5 5 32 5 Z"), S(rr(29, 20, 6, 20, 3)), S(circ(32, 47, 3.4))]
I["success"] = [S(circ(32, 32, 25)), Ln("M20 33 L28.5 41.5 L45 24")]
I["failure"] = [S(circ(32, 32, 25)), Ln("M22 22 L42 42 M42 22 L22 42")]
I["help"] = [S(circ(32, 32, 25)), Ln("M24 25 C24 17.5 29 14.5 33 14.5 C38.5 14.5 41.5 18.5 41.5 23 C41.5 29 34 30 32 35 L32 39"), D(circ(32, 46, 2.8))]
I["filter"] = [S(P([(7, 10), (57, 10), (38, 33), (38, 55), (26, 48), (26, 33)])), Ln("M12 15 L52 15")]
I["favorites"] = [S(star(32, 34, 27, 11.5, 5, -90))]
rca, rcp = arc(32, 34, 25, 205, 345)
I["recent"] = [S(circ(32, 34, 19)), Ln("M32 34 L32 23 M32 34 L40 38"), D(circ(32, 34, 2.2)), Ln(rca), head(*rcp, 345 + 90)]
I["recent"] = [S(circ(32, 34, 19)), Ln("M32 34 L32 23 M32 34 L40 38"), D(circ(32, 34, 2.2)), Ln(rca), head(rcp[0], rcp[1], 345 + 90)]
I["macro"] = [S(rr(5, 14, 54, 36, 3))] + [Ln(rr(9 + c * 9.6, 18 + r * 15, 7, 12, 1.5)) for r in range(2) for c in range(5)]
I["party_invite"] = bust(24, 12, 1.0) + [S(P([(45, 10), (51, 10), (51, 19), (60, 19), (60, 25), (51, 25), (51, 34), (45, 34), (45, 25), (36, 25), (36, 19), (45, 19)]))]
I["trade_request"] = arrow(8, 22, 56, 22) + arrow(56, 42, 8, 42)
I["transfer"] = arrow(22, 56, 22, 8) + arrow(42, 8, 42, 56)
I["inspect"] = EYE
I["target_nearest"] = crosshair(32, 32, 20)
I["target_next"] = [grp(crosshair(32, 32, 20), 0.7, 2, 9.6), chev("r", 52, 32, 1.1), chev("r", 59, 32, 1.1)]
I["target_previous"] = [grp(crosshair(32, 32, 20), 0.7, 17.2, 9.6), chev("l", 12, 32, 1.1), chev("l", 5, 32, 1.1)]
I["cancel"] = [S(P(rot([(-4, -24), (4, -24), (4, 24), (-4, 24)], math.radians(45), 32, 32))), S(P(rot([(-4, -24), (4, -24), (4, 24), (-4, 24)], math.radians(-45), 32, 32)))]
I["confirm"] = [S(P([(8, 34), (14, 28), (25, 39), (50, 13), (56, 19), (25, 51)]))]
I["cursor"] = [S("M18 6 L18 51 L28.5 41 L37 59 L43.5 56 L35 38.5 L49.5 38.5 Z")]
I["scroll"] = [S(rr(17, 6, 30, 52, 15)), Ln("M17.5 25 L46.5 25 M32 6.5 L32 25"), S(rr(29.5, 11, 5, 10, 2.5))]
I["drag_drop"] = [Ln("M32 12 L32 52 M12 32 L52 32"), head(32, 5, -90), head(32, 59, 90), head(5, 32, 180), head(59, 32, 0), S(circ(32, 32, 4.5))]
ga, gp = arc(32, 32, 26, 200, 330); gb, gq = arc(32, 32, 26, 20, 150)
I["gear_swap"] = [Ln(ga), head(gp[0], gp[1], 420), Ln(gb), head(gq[0], gq[1], 240), grp([S(SHIELD), Ln(SHIELD_IN)], 0.58, 13.4, 14)]
I["cast_interrupt"] = [grp(TOME, 0.85, 4.8, 8)] + [S("M47 3 L36 22 L44 22 L35 40 L54 17 L45 17 L52 3 Z")]
I["magic_burst"] = [S(star(32, 32, 29, 11, 8, -90)), S(circ(32, 32, 8)), S(star(32, 32, 5, 1.6, 4))]
LINK = [S(rr(-15, -7.5, 30, 15, 7.5) + " " + rr(-10, -3, 20, 6, 3))]
I["skillchain"] = [("g", "translate(23 41) rotate(-45)", LINK, 1.0), ("g", "translate(41 23) rotate(-45)", LINK, 1.0), S(star(32, 32, 7, 2, 4))]
I["enmity"] = EYE + [Ln("M9 15 L27 22.5 M55 15 L37 22.5")]
I["distance"] = [Ln("M14 36 L50 36"), head(6, 36, 180), head(58, 36, 0), Ln("M6 24 L6 48 M58 24 L58 48"), Ln("M20 36 L20 30 M26 36 L26 32 M32 36 L32 28 M38 36 L38 32 M44 36 L44 30")]
I["disengage"] = sword(32, 29, 180) + [S(rr(9, 54, 46, 5, 2))]
I["warning"] = I["warning"]

ALL = ["items", "magic", "abilities", "job", "equipment", "status", "party", "linkshell", "system", "map", "find", "config", "sword", "shield", "fire", "ice", "wind", "earth", "lightning", "water", "light", "dark", "healing", "enhancing", "enfeebling", "elemental", "summoning", "geomancy", "ninjutsu", "songs", "food", "medicine", "materials", "key", "quest", "currency", "head", "body", "hands", "legs", "feet", "ring", "ear", "neck", "waist", "back", "bow", "ammo", "attack", "poison", "haste", "protect", "sleep", "silence", "refresh", "cure", "weaponskill", "claimed", "trade", "sort", "stack", "split", "deposit", "withdraw", "mog_house", "mog_safe", "mog_locker", "mog_satchel", "mog_sack", "mog_case", "wardrobe", "equip_set", "lock", "unlock", "pin", "unpin", "chat", "tell", "linkshell_chat", "emote", "quest_log", "keybind", "timer", "cooldown", "recast", "range", "line_of_sight", "claim", "aggro", "party_target", "assist", "follow", "mount", "sneak", "invisible", "raise", "rest", "craft", "auction", "delivery", "settings", "notification", "warning", "success", "failure", "help", "filter", "favorites", "recent", "macro", "subjob", "party_invite", "trade_request", "inspect", "target_nearest", "target_next", "target_previous", "cancel", "confirm", "transfer", "cursor", "scroll", "drag_drop", "gear_swap", "cast_interrupt", "magic_burst", "skillchain", "enmity", "distance", "check", "disengage", "companion"]
missing = [n for n in ALL if n not in I]
assert not missing, missing

# ---- render (Pewter Hairline treatment) ----
_n = [0]
def svg(name, px=64, design=30.0, glow=True, scale=0.9):
    """`design`: the on-screen size the strokes are weighted for (logical px)."""
    _n[0] += 1; i = _n[0]
    sw0 = max(1.7, 64 / design * 1.05)
    defs = (f'<linearGradient id="s{i}" gradientUnits="userSpaceOnUse" x1="0" y1="4" x2="0" y2="60"><stop offset="0" stop-color="#f4f6fa"/><stop offset=".55" stop-color="#b7c0cf"/><stop offset="1" stop-color="#7d879b"/></linearGradient>'
            f'<linearGradient id="e{i}" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#2b3346"/><stop offset="1" stop-color="#121621"/></linearGradient>'
            f'<radialGradient id="g{i}"><stop offset="0" stop-color="#3d5f94" stop-opacity=".55"/><stop offset="1" stop-color="#3d5f94" stop-opacity="0"/></radialGradient>')
    def els(elems, k):
        out = []
        for e in elems:
            if e[0] == "g":
                out.append(f'<g transform="{e[1]}">' + els(e[2], k / (e[3] if len(e) > 3 else 1.0)) + '</g>')
                continue
            kind, d = e; sw = sw0 * k
            if kind == "shape": out.append(f'<path d="{d}" fill="url(#e{i})" fill-rule="evenodd" stroke="url(#s{i})" stroke-width="{sw:.2f}" stroke-linejoin="round"/>')
            elif kind == "line": out.append(f'<path d="{d}" fill="none" stroke="url(#s{i})" stroke-width="{sw*0.8:.2f}" stroke-linecap="round" stroke-linejoin="round"/>')
            elif kind == "dash": out.append(f'<path d="{d}" fill="none" stroke="url(#s{i})" stroke-width="{sw*0.8:.2f}" stroke-linecap="round" stroke-dasharray="{sw*1.2:.2f} {sw*1.6:.2f}"/>')
            else: out.append(f'<path d="{d}" fill="url(#s{i})" fill-rule="evenodd"/>')
        return "".join(out)
    bg = f'<circle cx="32" cy="32" r="31" fill="url(#g{i})"/>' if glow else ''
    body = els(I[name], 1.0 / scale)
    return (f'<svg xmlns="http://www.w3.org/2000/svg" width="{px}" height="{px}" viewBox="0 0 64 64"><defs>{defs}</defs>{bg}'
            f'<g transform="translate(32 32) scale({scale}) translate(-32 -32)">{body}</g></svg>')
