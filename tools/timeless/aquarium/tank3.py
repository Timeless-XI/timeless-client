"""The Timeless Aquarium, v3: the Elder fish show piece.

Twice the width and depth of v2 (6 x 4 cells), and dressed up as a trophy:
- Charcoal glass and pewter with champagne-gilt accents: stepped cornice with a dentil row and the engraved frieze
  "ELDERS OF THE DEEP", fluted pewter corner columns with gilt capitals and bases, gilt finials with small glowing
  star crystals, a gilt sunburst crest behind the spinning star-crystal crown, gilt scrollwork either side of the
  hourglass emblem on the stand, bun feet.
- A lamp hood inside the cornice that lights the water once a fish is in.
- Back wall: a starry night with the logo's Lufaise landmark in silhouette, soft light shafts from above.
- The logo hourglass at the back (spinning star crystal, falling grains); ruins and kelp kept to the back so the
  fish have the open water.
- The Elder fish swim (swim.py): each its own loop, turning, banking, body swaying and tail beating.

Tank space: y up, x across, z toward the viewer; the interior is X0..X1, Y0..Y1, Z0..Z1.
"""
import os, sys, importlib
import numpy as np
from r3d import *
from r3d import _n
import fishlib
import swim
from tank2 import (box, plane, lathe, octa, star_crystal, quad, OBSID, HAIR, SKY, SKYSIDE, SAND, RUIN, KELP, CROWN,
                   HGGLASS, HGMETAL, HGSAND, HGCRYSTAL, GRAIN, EMBLEM, FISH_ORDER)

OUT = '/home/claude/elder/out/'
X0, X1, Y0, Y1, Z0, Z1 = -1.5, 1.5, 0.0, 1.3, -0.95, 0.95
GILT, PLAQUE, LAMP, COLUMN, FINIAL = range(1017, 1022)
STATE = {'lit': 1.0, 'crystal': 1.0}
HG = dict(x=0.0, z=-0.62, h=0.78, rb=0.19, rn=0.02)
STAND = -0.40          # stand bottom (above the base step)
FLOOR = -0.515         # the bottom of the feet
CAP = Y1 + 0.17        # top of the cornice
SCALE = {'barbel': 0.68, 'oreeater': 0.64, 'whiskerwise': 0.63, 'admiral': 0.64, 'moonfin': 0.68, 'rimecoil': 0.6,
         'herald': 0.7, 'siren': 0.62}
SCALE['rimecoil'] = 0.66      # the straight eel: about 0.92 long


# ----------------------------------------------------------------------------- helpers

def trim_box(m, lo, hi, body=OBSID, trim=GILT, t=0.008, lines='tb', sides=True):
    """A bar with thin trim lines along its front (and back / sides) top and bottom edges."""
    lo, hi = np.asarray(lo, float), np.asarray(hi, float)
    box(m, lo, hi, body)
    ys = ([hi[1] - t * 1.5] if 't' in lines else []) + ([lo[1] + t * 0.5] if 'b' in lines else [])
    for y in ys:
        box(m, (lo[0] - 0.002, y, hi[2]), (hi[0] + 0.002, y + t, hi[2] + 0.005), trim)
        if sides:
            box(m, (lo[0] - 0.002, y, lo[2] - 0.005), (hi[0] + 0.002, y + t, lo[2]), trim)
            for x, dx in ((lo[0], -0.005), (hi[0], 0.005)):
                box(m, (min(x, x + dx), y, lo[2]), (max(x, x + dx), y + t, hi[2]), trim)


def fluted_column(m, cx, cz, y0, y1, r, part=COLUMN, flutes=12, n=24, nr=72):
    ys = np.linspace(y0, y1, n); th = np.linspace(0, 2 * np.pi, nr + 1)
    Y, TH = np.meshgrid(ys, th, indexing='ij')
    R = r * (1 - 0.07 * np.clip(np.cos(flutes * TH), 0, 1) ** 0.6)
    P = np.stack([cx + R * np.cos(TH), Y, cz + R * np.sin(TH)], -1).reshape(-1, 3)
    UV = np.stack([(Y - y0) / (y1 - y0), TH / (2 * np.pi)], -1).reshape(-1, 2)
    T = grid_tris(n, nr + 1); N = grid_normals(P, n, nr + 1)
    c = np.stack([np.full(len(P), cx), P[:, 1], np.full(len(P), cz)], 1)
    flip = np.sum(N * (P - c), 1) < 0; N[flip] *= -1
    m.add(P, N, UV, T, part)


def ring(m, cx, cz, y0, y1, r0, r1, part=GILT):
    lathe(m, lambda Y: r0 + (r1 - r0) * (Y - y0) / max(y1 - y0, 1e-6), y0, y1, part, 4, 40, cx, cz)
    disc_y(m, cx, cz, y1, r1, part, up=True)
    disc_y(m, cx, cz, y0, r0, part, up=False)


def disc_y(m, cx, cz, y, r, part, up=True, n=40):
    th = np.linspace(0, 2 * np.pi, n + 1)[:-1]
    P = np.concatenate([[[cx, y, cz]], np.stack([cx + r * np.cos(th), np.full(n, y), cz + r * np.sin(th)], 1)])
    T = [(0, 1 + (k + 1) % n, 1 + k) if up else (0, 1 + k, 1 + (k + 1) % n) for k in range(n)]
    N = np.tile([0, 1.0 if up else -1.0, 0], (n + 1, 1))
    UV = np.concatenate([[[0.5, 0.5]], np.stack([0.5 + 0.5 * np.cos(th), 0.5 + 0.5 * np.sin(th)], 1)])
    m.add(P, N, UV, T, part)


def spire(m, cx, y0, cz, h, r, part=GILT):
    """A gilt finial: a ball and a slim spire."""
    m.add(*sphere((cx, y0 + r * 0.9, cz), 1.0, 12, scale=(r, r * 0.9, r)), part)
    lathe(m, lambda Y: r * 0.55 * (1 - (Y - y0 - r * 1.6) / (h - r * 1.6)) ** 1.4 + 0.002, y0 + r * 1.6, y0 + h, part, 10, 24, cx, cz)


def scroll(m, c, s, flip=1, part=GILT, z=0.0):
    """An S-scroll of gilt wire, flat on the front face at depth z (s = its half-length)."""
    th = np.linspace(0, 1, 120)
    # a spiral curling in at each end of a gentle S
    def curl(t, sgn):
        a = 4.2 * np.pi * t
        rr = s * 0.28 * np.exp(-0.26 * a)
        return np.stack([sgn * (s * 0.5 + rr * np.sin(a) - 0) , sgn * (rr * np.cos(a) - s * 0.28) ], -1)
    body = np.stack([(th - 0.5) * s, -s * 0.22 * np.sin(2 * np.pi * (th - 0.5))], -1)
    for pts in (body, curl(th, 1) * [1, 1], curl(th, -1)):
        P = np.column_stack([c[0] + flip * pts[:, 0], c[1] + pts[:, 1], np.full(len(pts), z)])
        f = lambda u, P=P: np.stack([np.interp(u, th, P[:, k]) for k in range(3)], -1)
        m.add(*tube(f, lambda u: 0.0055 * (1 - 0.5 * u) + 0.001, 40, 6), part)


# ----------------------------------------------------------------------------- the groups

def group_tank():
    rng = np.random.default_rng(31)
    m = Mesh()
    # sand with a mound under the hourglass and soft dunes
    def sand(U, V):
        x = X0 + (X1 - X0) * U; z = Z0 + (Z1 - Z0) * V
        y = (0.02 + 0.06 * np.exp(-((x - HG['x']) ** 2 / 0.16 + (z - HG['z']) ** 2 / 0.06))
             + 0.025 * np.sin(x * 2.1 + 1) * np.cos(z * 2.7) * 0.5 + 0.02 * np.exp(-((x + 1.0) ** 2) / 0.1) * (1 - V))
        return np.stack([x, y, z], -1)
    plane(m, sand, 110, 50, SAND, flip=True)
    # night sky: back wall and side walls, with dark backing outside so the tank is closed from any side
    plane(m, lambda U, V: np.stack([X0 + (X1 - X0) * U, Y0 + (Y1 - Y0) * V, np.full_like(U, Z0)], -1), 6, 4, SKY)
    for x in (X0, X1):
        plane(m, lambda U, V, x=x: np.stack([np.full_like(U, x), Y0 + (Y1 - Y0) * V, Z0 + (Z1 - Z0) * U], -1), 3, 3, SKYSIDE)
    box(m, (X0 - 0.06, 0, Z0 - 0.07), (X1 + 0.06, Y1, Z0 - 0.01), OBSID)
    for x0 in (X0 - 0.07, X1 + 0.01):
        box(m, (x0, 0, Z0 - 0.06), (x0 + 0.06, Y1, Z1), OBSID)

    # ---- the stand: base step on bun feet, the stand body with its panel, top step
    for x in (X0 - 0.06, X1 + 0.06):
        for z in (Z0 - 0.06, Z1 + 0.06):
            m.add(*sphere((x, FLOOR + 0.041, z), 1.0, 14, scale=(0.075, 0.04, 0.075)), GILT)
    trim_box(m, (X0 - 0.2, FLOOR + 0.05, Z0 - 0.2), (X1 + 0.2, STAND, Z1 + 0.2), OBSID, GILT, lines='t')
    box(m, (X0 - 0.06, STAND, Z0 - 0.06), (X1 + 0.06, -0.09, Z1 + 0.06), OBSID)
    trim_box(m, (X0 - 0.16, -0.09, Z0 - 0.15), (X1 + 0.16, 0.0, Z1 + 0.15), OBSID, GILT, lines='tb')
    zf = Z1 + 0.061
    # panel frame (gilt), corner rosettes
    for (a, b) in (((X0 + 0.1, STAND + 0.04), (X1 - 0.1, STAND + 0.048)), ((X0 + 0.1, -0.138), (X1 - 0.1, -0.13))):
        box(m, (a[0], a[1], zf - 0.001), (b[0], b[1], zf + 0.006), GILT)
    for x in (X0 + 0.1, X1 - 0.108):
        box(m, (x, STAND + 0.04, zf - 0.001), (x + 0.008, -0.13, zf + 0.006), GILT)
    for x in (X0 + 0.104, X1 - 0.104):
        for y in (STAND + 0.044, -0.134):
            octa(m, (x, y, zf + 0.01), 0.02, 0.02, 0.008, GILT, 45)
    # the hourglass emblem: two gilt triangles meeting at a crystal, bars above and below
    ec = -0.245; eh = 0.085
    m.add(*flat_poly([(-0.07, ec + eh, zf + 0.004), (0.07, ec + eh, zf + 0.004), (0.0, ec, zf + 0.004)], (0, 0, 1)), GILT)
    m.add(*flat_poly([(-0.07, ec - eh, zf + 0.004), (0.0, ec, zf + 0.004), (0.07, ec - eh, zf + 0.004)], (0, 0, 1)), GILT)
    for y in (ec + eh, ec - eh - 0.008):
        box(m, (-0.09, y, zf), (0.09, y + 0.008, zf + 0.008), GILT)
    octa(m, (0, ec, zf + 0.012), 0.016, 0.024, 0.009, EMBLEM, 45)
    # gilt scrollwork either side of the emblem
    for sgn in (1, -1):
        for k, (dx, sc) in enumerate(((0.32, 0.36), (0.85, 0.3))):
            scroll(m, (sgn * dx, ec), sc, flip=sgn, z=zf + 0.006)
        m.add(*tube(lambda u, s=sgn: np.stack([s * (0.13 + 1.05 * u), np.full_like(u, ec - 0.105), np.full_like(u, zf + 0.006)], -1), 0.004, 10, 6), GILT)
        m.add(*tube(lambda u, s=sgn: np.stack([s * (0.13 + 1.05 * u), np.full_like(u, ec + 0.105), np.full_like(u, zf + 0.006)], -1), 0.004, 10, 6), GILT)

    # ---- the tank frame: bottom rail, fluted corner columns, cornice with frieze and dentils, cap
    trim_box(m, (X0 - 0.1, -0.01, Z1 + 0.005), (X1 + 0.1, 0.07, Z1 + 0.09), OBSID, GILT, lines='t', sides=False)
    for x in (X0 - 0.055, X1 + 0.055):
        for z in (Z0 - 0.055, Z1 + 0.055):
            ring(m, x, z, 0.0, 0.03, 0.085, 0.085)
            ring(m, x, z, 0.03, 0.055, 0.072, 0.062)
            fluted_column(m, x, z, 0.055, Y1 - 0.07, 0.055)
            ring(m, x, z, Y1 - 0.07, Y1 - 0.05, 0.06, 0.07)
            ring(m, x, z, Y1 - 0.05, Y1 - 0.015, 0.075, 0.092)
    trim_box(m, (X0 - 0.13, Y1 - 0.015, Z0 - 0.13), (X1 + 0.13, Y1 + 0.105, Z1 + 0.13), OBSID, GILT, lines='tb')
    # the frieze: engraved letters across the front of the cornice
    fz = Z1 + 0.131
    P = np.array([[-1.2, Y1 + 0.004, fz], [1.2, Y1 + 0.004, fz], [1.2, Y1 + 0.086, fz], [-1.2, Y1 + 0.086, fz]])
    m.add(P, np.tile([0, 0, 1.0], (4, 1)), np.array([[0, 0], [1, 0], [1, 1], [0, 1]], float), [(0, 1, 2), (0, 2, 3)], PLAQUE)
    # dentils under the cornice, front and sides
    for x in np.arange(X0 - 0.08, X1 + 0.09, 0.075):
        box(m, (x - 0.018, Y1 - 0.045, Z1 + 0.1), (x + 0.018, Y1 - 0.015, Z1 + 0.128), GILT)
    for z in np.arange(Z0 - 0.08, Z1 + 0.09, 0.075):
        for x, d in ((X0 - 0.13, 1), (X1 + 0.13, -1)):
            box(m, (min(x, x + d * 0.028), Y1 - 0.045, z - 0.018), (max(x, x + d * 0.028), Y1 - 0.015, z + 0.018), GILT)
    trim_box(m, (X0 - 0.18, Y1 + 0.105, Z0 - 0.18), (X1 + 0.18, CAP, Z1 + 0.18), OBSID, GILT, lines='b')
    # the lamp hood: a glowing panel under the cornice, inside
    box(m, (X0 + 0.05, Y1 - 0.012, Z0 + 0.06), (X1 - 0.05, Y1, Z1 - 0.06), OBSID)
    for zc in (Z0 + 0.3, Z1 - 0.35):
        box(m, (X0 + 0.12, Y1 - 0.026, zc - 0.022), (X1 - 0.12, Y1 - 0.012, zc + 0.022), LAMP)
    # finials on the four corners
    for x in (X0 - 0.12, X1 + 0.12):
        for z in (Z0 - 0.12, Z1 + 0.12):
            ring(m, x, z, CAP, CAP + 0.025, 0.05, 0.04)
            spire(m, x, CAP + 0.025, z, 0.16, 0.032)
            octa(m, (x, CAP + 0.215, z), 0.022, 0.04, 0.022, FINIAL, 45)
    # the crest: a stepped gilt plinth and a sunburst fan behind the crown
    trim_box(m, (-0.36, CAP, -0.14), (0.36, CAP + 0.05, 0.14), OBSID, GILT, lines='t')
    trim_box(m, (-0.24, CAP + 0.05, -0.1), (0.24, CAP + 0.085, 0.1), OBSID, GILT, lines='t')
    cy = CAP + 0.29; zc = -0.02
    for k in range(17):
        a = np.pi * (k / 16)
        ln = 0.3 if k % 2 == 0 else 0.22
        d = np.array([np.cos(a), np.sin(a), 0]); p = np.array([-d[1], d[0], 0])
        base = np.array([0, cy, zc]) + d * 0.12
        tip = np.array([0, cy, zc]) + d * ln
        w = 0.022 if k % 2 == 0 else 0.014
        q = [base + p * w, base - p * w, tip]
        m.add(*flat_poly(q, (0, 0, 1)), GILT, layer=1)
    th = np.linspace(0, np.pi, 60)
    arc = lambda u: np.stack([0.125 * np.cos(np.pi * u), cy + 0.125 * np.sin(np.pi * u), np.full_like(u, zc + 0.004)], -1)
    m.add(*tube(arc, 0.008, 40, 8), GILT)
    m.add(*tube(lambda u: np.stack([-0.125 + 0.25 * u, np.full_like(u, CAP + 0.085), np.full_like(u, zc)], -1), 0.0, 2, 3), GILT) if False else None
    box(m, (-0.02, CAP + 0.085, zc - 0.02), (0.02, cy - 0.11, zc + 0.02), GILT)

    # ---- inside: ruins and kelp kept to the back strip, rocks low on the sand
    for (x, z, h, r) in ((1.12, -0.78, 0.62, 0.085), (-1.18, -0.74, 0.4, 0.075), (0.62, -0.82, 0.2, 0.08), (-0.55, -0.82, 0.13, 0.07)):
        fluted_column(m, x, z, 0.0, h, r, RUIN, flutes=10, n=10, nr=40)
        if h > 0.3:
            # a broken top: a slanted drum fragment
            m.add(*sphere((x + 0.02, h, z), 1.0, 10, scale=(r * 1.15, r * 0.35, r * 1.15)), RUIN)
    m.add(*tube(lambda u: np.stack([-0.95 + 0.3 * u, np.full_like(u, 0.075), -0.62 + 0.06 * u], -1), 0.07, 4, 18), RUIN)
    for _ in range(16):
        c = (rng.uniform(X0 + 0.12, X1 - 0.12), 0.03, rng.uniform(Z0 + 0.08, Z1 - 0.12))
        if abs(c[0] - HG['x']) < 0.32 and abs(c[2] - HG['z']) < 0.3:
            continue
        r = rng.uniform(0.03, 0.08)
        m.add(*sphere(c, 1.0, 9, scale=(r * 1.3, r * 0.55, r)), RUIN)
    for k in range(14):
        side = -1 if k % 2 else 1
        x = side * rng.uniform(0.45, 1.38)
        z = rng.uniform(Z0 + 0.06, Z0 + 0.38); h = rng.uniform(0.4, 0.92); ph = rng.uniform(0, 6)
        root = lambda s, x=x, z=z, h=h, ph=ph: np.stack([x + 0.05 * np.sin(s * 3 + ph) * s, 0.02 + h * s, np.full_like(s, z)], -1)
        edge = lambda s, x=x, z=z, h=h, ph=ph: np.stack([x + 0.055 + 0.05 * np.sin(s * 3 + ph) * s - 0.025 * s, 0.02 + h * s, np.full_like(s, z) + 0.012], -1)
        m.add(*sheet(root, edge, 26, 4, ruffle=0.012, ruffle_freq=5), KELP, layer=1)

    # ---- the hourglass: gilt-trimmed stepped bars, glass, the pile (crystal and grains are their own groups)
    x, z, h, rb, rn = HG['x'], HG['z'], HG['h'], HG['rb'], HG['rn']
    y0 = 0.1
    for (yb, hh, w) in ((y0 - 0.05, 0.028, 0.22), (y0 - 0.022, 0.022, 0.185)):
        trim_box(m, (x - w, yb, z - w * 0.8), (x + w, yb + hh, z + w * 0.8), HGMETAL, GILT, t=0.006)
    for (yb, hh, w) in ((y0 + h, 0.022, 0.185), (y0 + h + 0.022, 0.028, 0.22)):
        trim_box(m, (x - w, yb, z - w * 0.8), (x + w, yb + hh, z + w * 0.8), HGMETAL, GILT, t=0.006)
    for sx in (-1, 1):
        for sz in (-1, 1):
            m.add(*tube(lambda u, sx=sx, sz=sz: np.stack([np.full_like(u, x + sx * 0.16), y0 + h * u, np.full_like(u, z + sz * 0.125)], -1), 0.009, 12, 8), GILT)
    prof = lambda Y: rn + (rb - rn) * np.abs(2 * (Y - y0) / h - 1) ** 0.85 * np.sqrt(np.clip(1 - np.abs(2 * (Y - y0) / h - 1) ** 10, 0.05, 1))
    lathe(m, prof, y0, y0 + h, HGGLASS, 40, 36, x, z, layer=1)
    pile = lambda Y: np.clip((rb * 0.9) * (1 - (Y - y0) / (h * 0.24)) ** 0.8, 0.002, None)
    lathe(m, pile, y0 + 0.004, y0 + h * 0.24, HGSAND, 12, 30, x, z)
    return m


def group_crown():
    m = Mesh(); c = np.array([0, CAP + 0.29, -0.02])
    star_crystal(m, c, 0.13, CROWN)
    return m, ('spin', c, (0, 1, 0), 14.0)


def group_hgcrystal():
    m = Mesh(); y0 = 0.1
    c = np.array([HG['x'], y0 + HG['h'] * 0.73, HG['z']])
    star_crystal(m, c, 0.052, HGCRYSTAL)
    return m, ('spin', c, (0, 1, 0), 9.0)


def group_grains():
    m = Mesh(); y0 = 0.1; sp = 0.034
    top = y0 + HG['h'] * 0.5
    for k in range(int((HG['h'] * 0.27) / sp) + 1):
        m.add(*sphere((HG['x'], top - k * sp, HG['z']), 0.0075, 6), GRAIN)
    return m, ('fall', (0, -1, 0), sp, 0.5)


# ----------------------------------------------------------------------------- the fish: centred, facing +x; tail split off

def fish_mesh(name):
    """(body mesh, tail mesh or None, material, joint x): tank-space orientation, centred on the body, scaled."""
    fishlib.TAIL_TAG = 100
    mod = importlib.import_module('fish_' + name)
    importlib.reload(mod)
    (P, N, UV, T, part, layer), b = mod.build()
    fishlib.TAIL_TAG = 0
    s = SCALE[name]
    body = (part % 100) == fishlib.BODY
    used = np.unique(T[body])
    lo, hi = P[used].min(0), P[used].max(0)
    c = np.array([(lo[0] + hi[0]) / 2, (lo[1] + hi[1]) / 2, 0.0])
    P = (P - c) * s
    tail = part >= 100
    joint = (lo[0] - c[0]) * s
    def sub(mask):
        tris = T[mask]; used = np.unique(tris)
        remap = -np.ones(len(P), int); remap[used] = np.arange(len(used))
        return P[used], N[used], UV[used], remap[tris], part[mask] % 100, layer[mask]
    tail_mesh = sub(tail) if tail.any() and PATH_SPEC(name)['wag'] else None
    body_mesh = sub(~tail) if tail_mesh is not None else sub(np.ones(len(T), bool))
    return body_mesh, tail_mesh, mod.material, joint


def eel_segments(name='rimecoil'):
    """The straight eel cut across its length into segments: [(mesh centred on the segment, segment centre x)],
    the material, the head end x and the body length (tank units, already scaled)."""
    mod = importlib.import_module('fish_' + name)
    mod.STRAIGHT = True
    (P, N, UV, T, part, layer), b = mod.build()
    mod.STRAIGHT = False
    s = SCALE[name]
    body = part == fishlib.BODY
    used = np.unique(T[body]); lo, hi = P[used].min(0), P[used].max(0)
    c = np.array([(lo[0] + hi[0]) / 2, (lo[1] + hi[1]) / 2, 0.0])
    P = (P - c) * s
    xh, xt = (hi[0] - c[0]) * s, (lo[0] - c[0]) * s
    nseg = swim.PATHS[name]['eel']['segments']
    edges = np.linspace(xt, xh, nseg + 1)
    cx = P[T].mean(1)[:, 0]
    out = []
    for k in range(nseg):
        a, b2 = edges[k], edges[k + 1]
        ov = (b2 - a) * 0.12
        sel = np.where(body, (cx >= a - ov) & (cx < b2 + ov), (cx >= a) & (cx < b2))
        if k == 0:
            sel |= cx < a
        if k == nseg - 1:
            sel |= cx >= b2
        tris = T[sel]; u = np.unique(tris)
        remap = -np.ones(len(P), int); remap[u] = np.arange(len(u))
        xc = (a + b2) / 2
        out.append(((P[u] - [xc, 0, 0], N[u], UV[u], remap[tris], part[sel], layer[sel]), xc))
    return out, mod.material, xh, xh - xt


def PATH_SPEC(name):
    return swim.PATHS[name]


# ----------------------------------------------------------------------------- materials

_LM = None
LM_BOX = (0.04, 1.0, 0.24)


def landmark(u, v):
    global _LM
    if _LM is None:
        from PIL import Image
        im = np.asarray(Image.open(os.path.join(os.path.dirname(os.path.abspath(__file__)), 'landmark.png')).convert('RGBA'), float) / 255
        _LM = (im[..., :3] ** 2.2, im[..., 3])
    rgb, a = _LM
    h, w = a.shape
    u0, u1, v0 = LM_BOX
    hv = (u1 - u0) * (X1 - X0) * h / w / (Y1 - Y0)
    x = (u - u0) / (u1 - u0) * (w - 1)
    y = (1 - (v - v0) / hv) * (h - 1)
    ok = (x >= 0) & (x <= w - 1) & (y >= 0) & (y <= h - 1)
    xi = np.clip(x, 0, w - 1.001); yi = np.clip(y, 0, h - 1.001)
    x0 = xi.astype(int); y0 = yi.astype(int); fx = xi - x0; fy = yi - y0
    def bil(img):
        wts = [((1 - fx) * (1 - fy), 0, 0), (fx * (1 - fy), 0, 1), ((1 - fx) * fy, 1, 0), (fx * fy, 1, 1)]
        return sum((img[y0 + dy, x0 + dx] * (wt[..., None] if img.ndim == 3 else wt)) for wt, dy, dx in wts)
    c = bil(rgb); al = bil(a) * ok
    al = np.where((v < v0), 0.0, al)
    return c, al


def sky(uv, side=False):
    u, v = uv[:, 0], uv[:, 1]
    lit = STATE['lit']
    top, mid, low = C('#090d2a'), C('#262457'), C('#3d4479')
    base = np.where((v > 0.45)[:, None], mix(mid, top, np.clip((v - 0.45) / 0.55, 0, 1)), mix(low, mid, np.clip(v / 0.45, 0, 1)))
    cloud = np.clip(fbm(u * 5 + 3, v * 9, 5) - 0.5, 0, 1) * 2.2 * np.exp(-((v - 0.5) / 0.17) ** 2)
    base = mix(base, C('#a596d8'), cloud * 0.4)
    emit = np.zeros((len(u), 3))
    for f, thr, b in ((110, 0.05, 1.1), (55, 0.07, 1.8), (26, 0.045, 3.0)):
        d = cellnoise(u * f * 2.4, v * f)
        star = np.clip((thr - d) / thr, 0, 1) ** 2 * (0.4 + 0.6 * hash2(np.floor(u * f * 2.4), np.floor(v * f)))
        star *= np.clip((v - 0.15) * 2.5, 0, 1)
        emit += np.outer(star * b * (0.25 + 0.75 * lit), C('#e6eeff'))
    # soft shafts of light falling from the lamp
    shafts = sum(np.exp(-((u - c0 - 0.08 * (1 - v)) / w0) ** 2) * a0 for c0, w0, a0 in ((0.18, 0.03, 0.5), (0.33, 0.05, 0.35), (0.62, 0.035, 0.45), (0.8, 0.06, 0.3)))
    shafts = shafts * np.clip(v, 0, 1) ** 0.6
    if not side:
        rgb, al = landmark(u, v)
        base = base * (1 - al)[:, None] + rgb * 1.3 * al[:, None]
        emit *= (1 - al)[:, None]
    emit += np.outer(shafts * 0.35 * lit, C('#9fc0ff'))
    return dict(albedo=base * 0.12, spec=0.0, emit=emit + base * 0.7 * (0.3 + 0.7 * lit))


_PLAQUE = None


def plaque_mask(uv):
    global _PLAQUE
    if _PLAQUE is None:
        from PIL import Image, ImageDraw, ImageFont
        W, H = 2048, 72
        im = Image.new('L', (W, H), 0); d = ImageDraw.Draw(im)
        f = ImageFont.truetype('/usr/share/fonts/truetype/liberation/LiberationSerif-Regular.ttf', 54)
        text = 'E L D E R S    O F    T H E    D E E P'
        tw = d.textlength(text, font=f)
        d.text(((W - tw) / 2, 4), text, fill=255, font=f)
        for sx in (-1, 1):   # small diamonds and rules either side
            cx = W / 2 + sx * (tw / 2 + 60)
            d.polygon([(cx - 15, 36), (cx, 22), (cx + 15, 36), (cx, 50)], fill=255)
            x0, x1 = sorted((cx + sx * 26, cx + sx * 200))
            d.rectangle([x0, 35, x1, 37], fill=255)
        _PLAQUE = np.asarray(im, float) / 255
    H, W = _PLAQUE.shape
    x = np.clip(uv[:, 0] * (W - 1), 0, W - 1).astype(int); y = np.clip((1 - uv[:, 1]) * (H - 1), 0, H - 1).astype(int)
    return _PLAQUE[y, x]


def gilt(p, scale=90):
    brush = fbm(p[:, 0] * scale, p[:, 1] * 6 + p[:, 2] * 6)
    return mix(C('#8a7346'), C('#ecdcab'), brush)


def material(part, uv, p, n):
    N = len(uv); lit = STATE['lit']; cr = STATE['crystal']
    if part == SKY:
        return sky(uv)
    if part == SKYSIDE:
        r = sky(uv, side=True); fade = (0.25 + 0.5 * uv[:, 0:1]); r['emit'] = r['emit'] * 0.3 * fade; r['albedo'] = r['albedo'] * 0.4; return r
    if part == SAND:
        g = fbm(p[:, 0] * 22, p[:, 2] * 22)
        rip = 0.5 + 0.5 * np.sin(p[:, 0] * 40 + fbm(p[:, 0] * 3, p[:, 2] * 3) * 5)
        base = mix(C('#46506a'), C('#8e9ab4'), g * 0.6 + rip * 0.25)
        sp = cellnoise(p[:, 0] * 110, p[:, 2] * 110)
        glint = np.clip((0.08 - sp) / 0.08, 0, 1) * (hash2(np.floor(p[:, 0] * 110), np.floor(p[:, 2] * 110)) > 0.6)
        # caustics from the lamp
        ca = cellnoise(p[:, 0] * 7 + fbm(p[:, 2] * 2, p[:, 0] * 2) * 0.6, p[:, 2] * 7)
        caust = np.clip(1 - np.abs(ca - 0.32) / 0.06, 0, 1) ** 2
        emit = np.outer(glint * 1.4 * lit, C('#cfe0ff')) + np.outer(caust * 0.22 * lit, C('#a8c8ff'))
        return dict(albedo=base, spec=0.25, gloss=30, height=rip * 0.5 + g, emit=emit)
    if part == OBSID:
        g = fbm(p[:, 0] * 6, p[:, 1] * 6 + p[:, 2] * 6)
        return dict(albedo=mix(C('#07080e'), C('#141824'), g), spec=0.6, gloss=90)
    if part in (HAIR, HGMETAL, COLUMN):
        brush = fbm(p[:, 0] * 120, p[:, 1] * 4 + p[:, 2] * 4)
        return dict(albedo=mix(C('#7d8696'), C('#d0d7e2'), brush), spec=1.5, gloss=70)
    if part == GILT:
        return dict(albedo=gilt(p), spec=1.8, gloss=80, emit=np.tile(C('#3a2e14'), (N, 1)) * 0.25)
    if part == PLAQUE:
        t = plaque_mask(uv)[:, None]
        return dict(albedo=gilt(p, 40) * t + C('#0a0b12') * (1 - t), spec=1.2, gloss=70,
                    emit=np.tile(C('#6e5a2e'), (N, 1)) * t * 0.35)
    if part == LAMP:
        return dict(albedo=np.tile(C('#dfe9ff'), (N, 1)), spec=0, emit=np.tile(C('#cfe0ff'), (N, 1)) * (0.05 + 1.0 * lit))
    if part == RUIN:
        g = fbm(uv[:, 0] * 8, uv[:, 1] * 14)
        return dict(albedo=mix(C('#0e1220'), C('#222a3a'), g), spec=0.3, gloss=20, height=g)
    if part == KELP:
        t = uv[:, 1]
        base = mix(C('#04090e'), C('#0d2128'), t * 0.6)
        edge = np.clip((np.abs(uv[:, 1] - 0.5) * 2 - 0.85) * 6, 0, 1)
        return dict(albedo=base, spec=0.3, gloss=20, alpha=0.92, sss=0.6, emit=np.outer(edge * 0.3 * lit, C('#4ab8c0')))
    if part in (CROWN, HGCRYSTAL, EMBLEM, FINIAL):
        f = fbm(p[:, 0] * 30, p[:, 1] * 30 + p[:, 2] * 30)
        glow = cr if part == CROWN else (lit if part in (HGCRYSTAL, FINIAL) else 0.6 * lit + 0.1)
        if part == FINIAL:
            glow = 0.25 + 0.6 * cr
        alb = mix(C('#3a4458'), mix(C('#a8d8ff'), C('#f0f8ff'), f), min(1, glow + 0.15))
        return dict(albedo=alb, spec=2.0, gloss=120, emit=np.tile(C('#5ab0ff'), (N, 1)) * (0.05 + 1.9 * glow) * (0.8 + 0.4 * f[:, None]))
    if part == HGGLASS:
        fres = 0.18 + 0.5 * (1 - np.abs(n[:, 2])) ** 2
        return dict(albedo=np.tile(C('#cfe6ff'), (N, 1)), spec=1.8, gloss=120, alpha=np.clip(fres * 0.8, 0.14, 0.55), emit=np.outer(fres * 0.25, C('#8fb8ff')))
    if part == HGSAND:
        g = fbm(uv[:, 0] * 20, uv[:, 1] * 20)
        return dict(albedo=mix(C('#7a88a8'), C('#c0cae0'), g), spec=0.4, gloss=40, emit=np.tile(C('#5a78b0'), (N, 1)) * 0.15 * lit)
    if part == GRAIN:
        return dict(albedo=np.tile(C('#e0ecff'), (N, 1)), spec=1, gloss=60, emit=np.tile(C('#9cc8ff'), (N, 1)) * 1.4 * lit)
    return dict(albedo=np.tile([1, 0, 1.0], (N, 1)))


INSIDE = {SAND, RUIN, KELP, HGGLASS, HGSAND, GRAIN, HGCRYSTAL, HGMETAL}
WATER = C('#14254a')


# ----------------------------------------------------------------------------- previews (tank space)

K_PREVIEW = 1.0


def anim_matrix(anim, t):
    import tank2
    return tank2.anim_matrix(anim, t)


def fish_anims(name, K=1.0, floor=FLOOR):
    spec = swim.PATHS[name]
    smp = swim.samples(spec, K, floor)
    return smp


def scene(names, t=0.0):
    """All groups at time t, in tank space, with one material function."""
    m = Mesh(); mats = {}
    out = [(group_tank().build(), None, None)]
    for fn in (group_crown, group_hgcrystal, group_grains):
        mm, anim = fn()
        out.append((mm.build(), anim, None))
    for nm in names:
        if 'eel' in swim.PATHS[nm]:
            segs, fmat, xh, ln = eel_segments(nm)
            spec = swim.PATHS[nm]; smp = swim.samples(spec, 1.0, 0.0)
            for mesh, xc in segs:
                p = swim.eel_params(spec, smp, xc, xh, ln, 1.0)
                S = np.diag([1.0, -1, -1, 1])
                out.append((mesh, ('mat', S @ swim.matrix(p, t, kind=6) @ S), (fmat, nm)))
            continue
        body, tail, fmat, jx = fish_mesh(nm)
        p = swim.params(swim.PATHS[nm], swim.samples(swim.PATHS[nm], 1.0, 0.0), jx)
        for mesh, is_tail in ((body, False), (tail, True)):
            if mesh is None:
                continue
            M = swim.matrix(p, t, is_tail)            # furniture space with K = 1, floor = 0: (x, -y, -z)
            S = np.diag([1.0, -1, -1, 1])
            A = S @ M @ S                            # back to tank space
            out.append((mesh, ('mat', A), (fmat, nm)))
    for gi, (mesh, anim, fm) in enumerate(out):
        P, N, UV, T, part, layer = mesh
        if anim is None:
            A = np.eye(4)
        elif anim[0] == 'mat':
            A = anim[1]
        else:
            A = anim_matrix(anim, t)
        P2 = P @ A[:3, :3].T + A[:3, 3]; N2 = N @ A[:3, :3].T
        base = sum(len(x) for x in m.P)
        m.P.append(P2); m.N.append(N2); m.UV.append(UV); m.T.append(T + base)
        if fm is not None:
            m.part.append(part + 3000 + 20 * gi); mats[gi] = (fm[0], A)
        else:
            m.part.append(part)
        m.layer.append(layer)

    def mat_all(part, uv, p, n):
        if part < 3000:
            r = material(part, uv, p, n)
            inside = part in INSIDE
        else:
            gi = (part - 3000) // 20; fmat, A = mats[gi]
            Ai = np.linalg.inv(A[:3, :3])
            pl = (p - A[:3, 3]) @ Ai.T; nl = n @ Ai.T
            r = fmat((part - 3000) % 20, uv, pl, nl)
            inside = True
        if inside:
            depth = np.clip(Z1 - p[:, 2], 0, 2) * 0.6 + np.clip(Y1 - p[:, 1], 0, 2) * 0.25
            f = 1 - np.exp(-depth * 0.7)
            r['albedo'] = mix(np.asarray(r['albedo'], float), WATER, f * 0.45)
            r['emit'] = np.asarray(r.get('emit', np.zeros((len(uv), 3))), float) + np.outer(f, WATER) * 0.2
        return r
    return m.build(), mat_all


def shoot(names, tag, W=1600, H=1000, ss=2, eye=(1.6, 1.55, 6.3), target=(0, 0.55, 0), fov=36, t=0.0, bloom=1.0):
    mesh, mat = scene(names, t)
    bg = lambda h, w: (np.array([0.02, 0.022, 0.04]) + np.array([0.05, 0.05, 0.09]) * np.exp(-((np.linspace(-1, 1, w)[None, :] * 0.9) ** 2 + ((np.linspace(0, 1, h)[:, None] - 0.4) * 1.5) ** 2))[..., None])
    lights = [((-0.3, 1.0, 0.45), (0.85, 0.85, 0.95)), ((0.8, 0.2, 0.6), (0.22, 0.26, 0.4)), ((0.1, 0.4, -1.0), (0.4, 0.5, 0.8))]
    img = render(mesh, mat, W, H, eye=eye, target=target, fov=fov, bg=bg, bloom=bloom, ss=ss, lights=lights,
                 env_up=(0.2, 0.24, 0.36), env_down=(0.07, 0.07, 0.1))
    save(img, OUT + f'tank3_{tag}.png')


if __name__ == '__main__':
    tag = sys.argv[1]
    t = float(sys.argv[2]) if len(sys.argv) > 2 else 3.0
    sets = {'empty': [], 'three': ['barbel', 'moonfin', 'siren'], 'full': FISH_ORDER, 'test': FISH_ORDER}
    STATE.update({'empty': dict(lit=0.0, crystal=0.0), 'three': dict(lit=1.0, crystal=0.3)}.get(tag, dict(lit=1.0, crystal=1.0)))
    if tag == 'test':
        shoot(sets['full'], 'test', 800, 500, 1, t=t)
    elif tag == 'close':
        shoot(FISH_ORDER, 'close', 1600, 1000, 2, eye=(0.3, 0.85, 3.2), target=(0.0, 0.6, 0), fov=44, t=t)
    elif tag == 'top':
        shoot(FISH_ORDER, 'top', 1000, 700, 1, eye=(0.0, 5.5, 1.2), target=(0, 0.4, 0), fov=40, t=t)
    else:
        shoot(sets[tag], tag, t=t)
