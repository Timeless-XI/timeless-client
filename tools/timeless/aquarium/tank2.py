"""The Timeless Aquarium, v2: a night-sky tank in the Timeless style.

- Charcoal-glass frame with the logo's stepped bars and engraved pewter hairlines; the hourglass emblem on the stand.
- Back wall: a dusk-to-night sky full of stars, faint lilac cloud, and the logo's own Lufaise landmark (the mesa with
  its window arch and ruins, landmark.png) as a night near-silhouette fading into mist at its base.
- The logo hourglass stands in the sand: stepped bars, glass, a spinning star crystal in the upper bulb, grains
  falling to the pile below.
- A spinning star crystal crowns the tank.
- Silver-blue sand that glimmers; ruins and kelp kept as dark silhouettes; the Elder fish drift in place.

Every part is a group with an animation (tank space, y up):
  ('none',), ('spin', pivot, axis, period_s), ('bob', pivot, amp, period_s, phase, yaw_deg), ('fall', dir, spacing, period_s)
"""
import os
import sys, importlib, numpy as np
from r3d import *
from r3d import _n
from fishlib import interp

OUT = '/home/claude/elder/out/'
X0, X1, Y0, Y1, Z0, Z1 = -1.3, 1.3, 0.0, 1.1, -0.45, 0.45
(OBSID, HAIR, SKY, SKYSIDE, SAND, RUIN, KELP, CROWN, SWIRL, MOTE, RING, HGGLASS, HGMETAL, HGSAND, HGCRYSTAL, GRAIN,
 EMBLEM) = range(1000, 1017)
STATE = {'lit': 1.0, 'crystal': 1.0}
VOID_C = np.array([0.0, 0.64, Z0 + 0.03]); VOID_R = 0.42
HG = dict(x=0.0, z=0.18, h=0.52, rb=0.13, rn=0.014)
rng = np.random.default_rng(31)


# ----------------------------------------------------------------------------- geometry helpers

def box(m, lo, hi, part, layer=0):
    lo, hi = np.asarray(lo, float), np.asarray(hi, float)
    for ax in range(3):
        for side in (0, 1):
            n = np.zeros(3); n[ax] = 1 if side else -1
            a, b = [i for i in range(3) if i != ax]
            c = np.zeros((4, 3)); val = hi[ax] if side else lo[ax]
            for k, (ia, ib) in enumerate(((0, 0), (1, 0), (1, 1), (0, 1))):
                c[k, ax] = val; c[k, a] = hi[a] if ia else lo[a]; c[k, b] = hi[b] if ib else lo[b]
            uv = np.array([[0, 0], [1, 0], [1, 1], [0, 1]], float)
            m.add(c, np.tile(n, (4, 1)), uv, [(0, 1, 2), (0, 2, 3)], part, layer)


def plane(m, f, nu, nv, part, layer=0, flip=False):
    u = np.linspace(0, 1, nu); v = np.linspace(0, 1, nv)
    U, V = np.meshgrid(u, v, indexing='ij')
    P = f(U, V).reshape(-1, 3)
    UV = np.stack([U, V], -1).reshape(-1, 2)
    T = grid_tris(nu, nv); N = grid_normals(P, nu, nv) * (-1 if flip else 1)
    m.add(P, N, UV, T, part, layer)


def lathe(m, prof, y0, y1, part, n=40, nr=36, cx=0.0, cz=0.0, layer=0):
    ys = np.linspace(y0, y1, n); th = np.linspace(0, 2 * np.pi, nr + 1)
    Y, TH = np.meshgrid(ys, th, indexing='ij')
    R = prof(Y)
    P = np.stack([cx + R * np.cos(TH), Y, cz + R * np.sin(TH)], -1).reshape(-1, 3)
    UV = np.stack([(Y - y0) / (y1 - y0), TH / (2 * np.pi)], -1).reshape(-1, 2)
    T = grid_tris(n, nr + 1); N = grid_normals(P, n, nr + 1)
    c = np.stack([np.full(len(P), cx), P[:, 1], np.full(len(P), cz)], 1)
    flipn = np.sum(N * (P - c), 1) < 0; N[flipn] *= -1
    m.add(P, N, UV, T, part, layer)


def octa(m, c, rx, ry, rz, part, rot=0.0):
    """A diamond: six points, eight flat faces."""
    a = np.radians(rot); ca, sa = np.cos(a), np.sin(a)
    pts = {'px': (rx * ca, 0, rx * sa), 'nx': (-rx * ca, 0, -rx * sa), 'pz': (-rz * sa, 0, rz * ca), 'nz': (rz * sa, 0, -rz * ca),
           'py': (0, ry, 0), 'ny': (0, -ry, 0)}
    c = np.asarray(c, float)
    faces = [('px', 'pz', 'py'), ('pz', 'nx', 'py'), ('nx', 'nz', 'py'), ('nz', 'px', 'py'),
             ('pz', 'px', 'ny'), ('nx', 'pz', 'ny'), ('nz', 'nx', 'ny'), ('px', 'nz', 'ny')]
    P = []; N = []; UV = []; T = []
    for f in faces:
        q = [c + np.array(pts[k]) for k in f]
        nrm = normalize(np.cross(q[1] - q[0], q[2] - q[0]))
        if np.dot(nrm, np.mean(q, 0) - c) < 0:
            nrm = -nrm
        k = len(P); P += q; N += [nrm] * 3; UV += [(0, 0), (1, 0), (0.5, 1)]; T.append((k, k + 1, k + 2))
    m.add(np.array(P), np.array(N), np.array(UV, float), np.array(T), part)


def star_crystal(m, c, s, part):
    """The logo's star crystal: a tall diamond with four side points, the four between turned 45 degrees."""
    octa(m, c, s * 0.5, s * 1.35, s * 0.5, part, 45)
    octa(m, c, s * 1.05, s * 0.32, s * 0.16, part, 0)
    octa(m, c, s * 0.16, s * 0.32, s * 1.05, part, 0)
    octa(m, c, s * 0.62, s * 0.22, s * 0.12, part, 45)
    octa(m, c, s * 0.12, s * 0.22, s * 0.62, part, 45)


def disc(m, c, r0, r1, part, n_r=12, n_t=72, layer=0, z_off=0.0):
    rs = np.linspace(r0, r1, n_r); th = np.linspace(0, 2 * np.pi, n_t + 1)
    R, TH = np.meshgrid(rs, th, indexing='ij')
    P = np.stack([c[0] + R * np.cos(TH), c[1] + R * np.sin(TH), np.full_like(R, c[2] + z_off)], -1).reshape(-1, 3)
    UV = np.stack([0.5 + R * np.cos(TH) / (2 * r1), 0.5 + R * np.sin(TH) / (2 * r1)], -1).reshape(-1, 2)
    N = np.tile([0, 0, 1.0], (len(P), 1))
    m.add(P, N, UV, grid_tris(n_r, n_t + 1), part, layer)


def quad(m, c, s, part, layer=1):
    c = np.asarray(c, float)
    P = c + np.array([[-s, -s, 0], [s, -s, 0], [s, s, 0], [-s, s, 0]])
    m.add(P, np.tile([0, 0, 1.0], (4, 1)), np.array([[0, 0], [1, 0], [1, 1], [0, 1]], float), [(0, 1, 2), (0, 2, 3)], part, layer)


def hairline_box(m, lo, hi, body=OBSID, inset=0.0):
    """A charcoal bar with pewter hairlines along its front and back top/bottom edges."""
    lo, hi = np.asarray(lo, float), np.asarray(hi, float)
    box(m, lo, hi, body)
    t = 0.007
    for z in (hi[2], lo[2]):
        dz = t if z == hi[2] else -t
        for y in (hi[1] - t * 1.6 - inset, lo[1] + inset):
            box(m, (lo[0] + 0.01, y, min(z, z + dz)), (hi[0] - 0.01, y + t, max(z, z + dz)), HAIR)


# ----------------------------------------------------------------------------- the groups

def group_tank():
    rng = np.random.default_rng(31)
    m = Mesh()
    # floor: silver-blue sand with a mound under the hourglass
    def sand(U, V):
        x = X0 + (X1 - X0) * U; z = Z0 + (Z1 - Z0) * V
        y = 0.015 + 0.05 * np.exp(-((x - HG['x']) ** 2 / 0.12 + (z - HG['z']) ** 2 / 0.05)) + 0.02 * np.exp(-((x + 0.7) ** 2) / 0.08) * V
        return np.stack([x, y, z], -1)
    plane(m, sand, 80, 30, SAND, flip=True)
    # night sky: back wall and the two side walls
    plane(m, lambda U, V: np.stack([X0 + (X1 - X0) * U, Y0 + (Y1 - Y0) * V, np.full_like(U, Z0)], -1), 6, 4, SKY)
    for x in (X0, X1):
        plane(m, lambda U, V, x=x: np.stack([np.full_like(U, x), Y0 + (Y1 - Y0) * V, Z0 + (Z1 - Z0) * U], -1), 3, 3, SKYSIDE)
    # frame: stepped bars, stand, corner posts
    hairline_box(m, (X0 - 0.12, -0.10, Z0 - 0.10), (X1 + 0.12, 0.0, Z1 + 0.10))
    hairline_box(m, (X0 - 0.20, -0.18, Z0 - 0.16), (X1 + 0.20, -0.10, Z1 + 0.16))
    box(m, (X0 - 0.06, -0.62, Z0 - 0.06), (X1 + 0.06, -0.18, Z1 + 0.06), OBSID)
    # engraved inset panel on the stand front
    for (a, b) in (((X0 + 0.08, -0.565), (X1 - 0.08, -0.558)), ((X0 + 0.08, -0.242), (X1 - 0.08, -0.235))):
        box(m, (a[0], a[1], Z1 + 0.06), (b[0], b[1], Z1 + 0.067), HAIR)
    for x in (X0 + 0.08, X1 - 0.087):
        box(m, (x, -0.565, Z1 + 0.06), (x + 0.007, -0.235, Z1 + 0.067), HAIR)
    hairline_box(m, (X0 - 0.20, -0.68, Z0 - 0.16), (X1 + 0.20, -0.62, Z1 + 0.16))
    hairline_box(m, (X0 - 0.12, Y1, Z0 - 0.10), (X1 + 0.12, Y1 + 0.08, Z1 + 0.10))
    hairline_box(m, (X0 - 0.03, Y1 + 0.08, Z0 - 0.04), (X1 + 0.03, Y1 + 0.13, Z1 + 0.04))
    for x in (X0 - 0.022, X1 + 0.022):
        for z in (Z0 - 0.022, Z1 + 0.022):
            box(m, (x - 0.018, 0, z - 0.018), (x + 0.018, Y1, z + 0.018), HAIR)
    # crown pedestal
    hairline_box(m, (-0.13, Y1 + 0.13, -0.09), (0.13, Y1 + 0.17, 0.09))
    box(m, (-0.07, Y1 + 0.17, -0.05), (0.07, Y1 + 0.2, 0.05), HAIR)
    # the hourglass emblem on the stand: two pewter triangles meeting at a crystal
    zf = Z1 + 0.061
    m.add(*flat_poly([(-0.085, -0.29, zf), (0.085, -0.29, zf), (0.0, -0.40, zf)], (0, 0, 1)), HAIR)
    m.add(*flat_poly([(-0.085, -0.51, zf), (0.0, -0.40, zf), (0.085, -0.51, zf)], (0, 0, 1)), HAIR)
    box(m, (-0.1, -0.29, zf - 0.002), (0.1, -0.282, zf + 0.004), HAIR)
    box(m, (-0.1, -0.518, zf - 0.002), (0.1, -0.51, zf + 0.004), HAIR)
    octa(m, (0, -0.40, zf + 0.008), 0.016, 0.022, 0.008, EMBLEM, 45)
    # ruins as dark silhouettes: a broken column, a stub, a fallen drum, an arch fragment; rocks
    for (x, z, h, r) in ((1.0, -0.32, 0.5, 0.075), (-1.05, -0.3, 0.3, 0.065), (0.62, -0.36, 0.16, 0.07)):
        m.add(*tube(lambda u, x=x, z=z, h=h: np.stack([np.full_like(u, x), 0.0 + h * u, np.full_like(u, z)], -1), r, 6, 18), RUIN)
    m.add(*tube(lambda u: np.stack([-0.45 + 0.22 * u, np.full_like(u, 0.065), -0.33 + 0.05 * u], -1), 0.06, 4, 18), RUIN)
    for _ in range(10):
        c = (rng.uniform(X0 + 0.1, X1 - 0.1), 0.02, rng.uniform(Z0 + 0.05, Z1 - 0.12))
        if abs(c[0] - HG['x']) < 0.25 and abs(c[2] - HG['z']) < 0.2:
            continue
        r = rng.uniform(0.03, 0.09)
        m.add(*sphere(c, 1.0, 9, scale=(r * 1.3, r * 0.6, r)), RUIN)
    # kelp silhouettes
    for k in range(8):
        x = rng.uniform(X0 + 0.12, X1 - 0.12)
        if abs(x) < 0.55:
            x = np.sign(x + 1e-6) * rng.uniform(0.6, 1.15)
        z = rng.uniform(Z0 + 0.05, Z0 + 0.3); h = rng.uniform(0.35, 0.85); ph = rng.uniform(0, 6)
        root = lambda s, x=x, z=z, h=h, ph=ph: np.stack([x + 0.04 * np.sin(s * 3 + ph) * s, 0.02 + h * s, np.full_like(s, z)], -1)
        edge = lambda s, x=x, z=z, h=h, ph=ph: np.stack([x + 0.045 + 0.04 * np.sin(s * 3 + ph) * s - 0.02 * s, 0.02 + h * s, np.full_like(s, z) + 0.01], -1)
        m.add(*sheet(root, edge, 24, 4, ruffle=0.01, ruffle_freq=5), KELP, layer=1)
    # the hourglass: stepped bars, glass, sand pile (crystal and grains are their own groups)
    x, z, h, rb, rn = HG['x'], HG['z'], HG['h'], HG['rb'], HG['rn']
    y0 = 0.07
    for (yb, hh, w) in ((y0 - 0.035, 0.02, 0.15), (y0 - 0.015, 0.015, 0.125)):
        hairline_box(m, (x - w, yb, z - w * 0.8), (x + w, yb + hh, z + w * 0.8), HGMETAL)
    for (yb, hh, w) in ((y0 + h, 0.015, 0.125), (y0 + h + 0.015, 0.02, 0.15)):
        hairline_box(m, (x - w, yb, z - w * 0.8), (x + w, yb + hh, z + w * 0.8), HGMETAL)
    prof = lambda Y: rn + (rb - rn) * np.abs(2 * (Y - y0) / h - 1) ** 0.85 * np.sqrt(np.clip(1 - np.abs(2 * (Y - y0) / h - 1) ** 10, 0.05, 1))
    lathe(m, prof, y0, y0 + h, HGGLASS, 36, 32, x, z, layer=1)
    pile = lambda Y: np.clip((rb * 0.9) * (1 - (Y - y0) / (h * 0.24)) ** 0.8, 0.002, None)
    lathe(m, pile, y0 + 0.004, y0 + h * 0.24, HGSAND, 12, 28, x, z)
    return m


def group_crown():
    m = Mesh(); c = np.array([0, Y1 + 0.33, 0])
    star_crystal(m, c, 0.1, CROWN)
    return m, ('spin', c, (0, 1, 0), 14.0)


def group_hgcrystal():
    m = Mesh(); y0 = 0.07
    c = np.array([HG['x'], y0 + HG['h'] * 0.73, HG['z']])
    star_crystal(m, c, 0.038, HGCRYSTAL)
    return m, ('spin', c, (0, 1, 0), 9.0)


def group_grains():
    m = Mesh(); y0 = 0.07; sp = 0.028
    top = y0 + HG['h'] * 0.5
    for k in range(int((HG['h'] * 0.27) / sp) + 1):
        m.add(*sphere((HG['x'], top - k * sp, HG['z']), 0.0055, 6), GRAIN)
    return m, ('fall', (0, -1, 0), sp, 0.5)


# ----------------------------------------------------------------------------- materials

_LM = None
LM_BOX = (0.08, 1.0, 0.22)    # u0, u1, v of the image's bottom edge; height keeps the image's aspect on the 2.6 x 1.1 wall


def landmark(u, v):
    """The logo's landmark (landmark.png, 2442 x 750, alpha), placed on the back wall: linear rgb, alpha."""
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
        return (img[y0, x0] * ((1 - fx) * (1 - fy))[..., None] if img.ndim == 3 else img[y0, x0] * (1 - fx) * (1 - fy)) + \
               (img[y0, x0 + 1] * (fx * (1 - fy))[..., None] if img.ndim == 3 else img[y0, x0 + 1] * fx * (1 - fy)) + \
               (img[y0 + 1, x0] * ((1 - fx) * fy)[..., None] if img.ndim == 3 else img[y0 + 1, x0] * (1 - fx) * fy) + \
               (img[y0 + 1, x0 + 1] * (fx * fy)[..., None] if img.ndim == 3 else img[y0 + 1, x0 + 1] * fx * fy)
    c = bil(rgb); al = bil(a) * ok
    # below the image: the mist it fades into keeps going, darker toward the sand
    below = (v < v0) & (u > u0) & (u < u1)
    al = np.where(below, 0.0, al)
    return c, al


def sky(uv, side=False):
    u, v = uv[:, 0], uv[:, 1]
    lit = STATE['lit']
    top, mid, low = C('#05071a'), C('#1c1840'), C('#3c3e6a')
    base = np.where((v > 0.45)[:, None], mix(mid, top, np.clip((v - 0.45) / 0.55, 0, 1)), mix(low, mid, np.clip(v / 0.45, 0, 1)))
    cloud = np.clip(fbm(u * 5 + 3, v * 9, 5) - 0.52, 0, 1) * 2.2 * np.exp(-((v - 0.42) / 0.16) ** 2)
    base = mix(base, C('#8a7fb8'), cloud * 0.35)
    emit = np.zeros((len(u), 3))
    for f, thr, b in ((90, 0.05, 1.0), (45, 0.07, 1.6), (22, 0.045, 2.6)):
        d = cellnoise(u * f * 2.4, v * f)
        star = np.clip((thr - d) / thr, 0, 1) ** 2 * (0.4 + 0.6 * hash2(np.floor(u * f * 2.4), np.floor(v * f)))
        star *= np.clip((v - 0.12) * 2.5, 0, 1)
        emit += np.outer(star * b * (0.25 + 0.75 * lit), C('#dfe8ff'))
    if not side:
        # the Lufaise landmark from the logo (the mesa, its window arch and the ruins), a night silhouette
        rgb, al = landmark(u, v)
        base = base * (1 - al)[:, None] + rgb * al[:, None]
        emit *= (1 - al)[:, None]
    return dict(albedo=base * 0.12, spec=0.0, emit=emit + base * 0.55 * (0.35 + 0.65 * lit))


def material(part, uv, p, n):
    N = len(uv); lit = STATE['lit']; cr = STATE['crystal']
    if part == SKY:
        return sky(uv)
    if part == SKYSIDE:
        r = sky(uv, side=True); r['emit'] = r['emit'] * 0.3; r['albedo'] = r['albedo'] * 0.4; return r
    if part == SAND:
        g = fbm(p[:, 0] * 28, p[:, 2] * 28)
        rip = 0.5 + 0.5 * np.sin(p[:, 0] * 50 + fbm(p[:, 0] * 3, p[:, 2] * 3) * 5)
        base = mix(C('#3e4658'), C('#7f8aa2'), g * 0.6 + rip * 0.25)
        sp = cellnoise(p[:, 0] * 140, p[:, 2] * 140)
        glint = np.clip((0.08 - sp) / 0.08, 0, 1) * (hash2(np.floor(p[:, 0] * 140), np.floor(p[:, 2] * 140)) > 0.6)
        emit = np.outer(glint * 1.4 * lit, C('#cfe0ff'))
        return dict(albedo=base, spec=0.25, gloss=30, height=rip * 0.5 + g, emit=emit)
    if part == OBSID:
        g = fbm(p[:, 0] * 6, p[:, 1] * 6 + p[:, 2] * 6)
        return dict(albedo=mix(C('#06070c'), C('#10131c'), g), spec=0.55, gloss=90)
    if part in (HAIR, HGMETAL):
        brush = fbm(p[:, 0] * 120, p[:, 1] * 4 + p[:, 2] * 4)
        return dict(albedo=mix(C('#7d8696'), C('#c9d0dc'), brush), spec=1.5, gloss=70)
    if part == RUIN:
        g = fbm(uv[:, 0] * 8, uv[:, 1] * 14)
        return dict(albedo=mix(C('#0c0f1a'), C('#1c2230'), g), spec=0.3, gloss=20, height=g)
    if part == KELP:
        t = uv[:, 1]
        base = mix(C('#03080c'), C('#0a1c22'), t * 0.6)
        edge = np.clip((np.abs(uv[:, 1] - 0.5) * 2 - 0.85) * 6, 0, 1)
        return dict(albedo=base, spec=0.3, gloss=20, alpha=0.92, sss=0.6, emit=np.outer(edge * 0.25 * lit, C('#4ab8c0')))
    if part in (CROWN, HGCRYSTAL, EMBLEM):
        f = fbm(p[:, 0] * 30, p[:, 1] * 30 + p[:, 2] * 30)
        glow = cr if part == CROWN else (lit if part == HGCRYSTAL else 0.6 * lit + 0.1)
        alb = mix(C('#3a4458'), mix(C('#a8d8ff'), C('#f0f8ff'), f), min(1, glow + 0.15))
        return dict(albedo=alb, spec=2.0, gloss=120, emit=np.tile(C('#5ab0ff'), (N, 1)) * (0.05 + 1.9 * glow) * (0.8 + 0.4 * f[:, None]))
    if part == RING:
        brush = fbm(uv[:, 0] * 60, uv[:, 1] * 60)
        return dict(albedo=mix(C('#6a7282'), C('#b8c0cc'), brush), spec=1.4, gloss=70)
    if part == SWIRL:
        x = (uv[:, 0] - 0.5) * 2; y = (uv[:, 1] - 0.5) * 2
        r = np.hypot(x, y); a = np.arctan2(y, x)
        arm = 0.5 + 0.5 * np.cos(2 * (a - 3.2 * np.log(r + 0.05)))
        dust = fbm(x * 6 + 10, y * 6, 5)
        core = np.exp(-(r / 0.16) ** 2)
        col = mix(C('#140c32'), C('#5a3a9a'), arm * dust)
        col = mix(col, C('#7fb8ff'), np.clip(arm ** 4 * dust * 1.3, 0, 1) * 0.7)
        col = mix(col, C('#e8f0ff'), core * 0.8)
        stars = np.clip((0.06 - cellnoise(x * 40, y * 40)) / 0.06, 0, 1) ** 2
        emit = (col * (0.5 + 1.1 * arm[:, None] * dust[:, None]) + np.outer(stars, C('#ffffff')) * 1.5) * (0.12 + 0.88 * lit)
        alpha = np.clip((1 - r) * 5, 0, 1) * 0.96
        hole = np.clip((r - 0.0) / 0.05, 0, 1)
        return dict(albedo=col * 0.3, spec=0.0, emit=emit, alpha=alpha * hole + (1 - hole) * alpha)
    if part == MOTE:
        x = (uv[:, 0] - 0.5) * 2; y = (uv[:, 1] - 0.5) * 2
        cross = np.exp(-np.abs(x) * 9) * np.exp(-np.abs(y) * 1.6) + np.exp(-np.abs(y) * 9) * np.exp(-np.abs(x) * 1.6)
        dot = np.exp(-(x * x + y * y) * 10)
        a = np.clip(cross * 0.8 + dot, 0, 1)
        return dict(albedo=np.tile(C('#ffffff'), (N, 1)), spec=0, emit=np.outer(a * 2.2 * (0.1 + 0.9 * lit), C('#dfeaff')), alpha=a)
    if part == HGGLASS:
        fres = 0.18 + 0.5 * (1 - np.abs(n[:, 2])) ** 2
        return dict(albedo=np.tile(C('#cfe6ff'), (N, 1)), spec=1.8, gloss=120, alpha=np.clip(fres * 0.8, 0.14, 0.55), emit=np.outer(fres * 0.25, C('#8fb8ff')))
    if part == HGSAND:
        g = fbm(uv[:, 0] * 20, uv[:, 1] * 20)
        return dict(albedo=mix(C('#7a88a8'), C('#c0cae0'), g), spec=0.4, gloss=40, emit=np.tile(C('#5a78b0'), (N, 1)) * 0.15 * lit)
    if part == GRAIN:
        return dict(albedo=np.tile(C('#e0ecff'), (N, 1)), spec=1, gloss=60, emit=np.tile(C('#9cc8ff'), (N, 1)) * 1.4 * lit)
    return dict(albedo=np.tile([1, 0, 1.0], (N, 1)))


# ----------------------------------------------------------------------------- fish places (clear of the hourglass)

PLACE = {  # name: (scale, yaw, pitch, position)
    'herald': (0.62, 8, 3, (-0.5, 0.86, -0.2)),
    'barbel': (0.62, 192, -4, (0.68, 0.8, 0.06)),
    'moonfin': (0.6, 18, 4, (-0.62, 0.5, 0.2)),
    'admiral': (0.55, 205, 0, (0.84, 0.25, -0.1)),
    'oreeater': (0.55, -22, 0, (-0.86, 0.16, 0.02)),
    'whiskerwise': (0.55, 168, 0, (0.45, 0.1, -0.14)),
    'rimecoil': (0.52, 185, 0, (0.08, 0.98, -0.27)),
    'siren': (0.52, 28, 0, (-0.98, 0.72, 0.18)),
}
FISH_ORDER = ['barbel', 'oreeater', 'whiskerwise', 'admiral', 'moonfin', 'rimecoil', 'herald', 'siren']


def roty(a):
    a = np.radians(a); c, s = np.cos(a), np.sin(a)
    return np.array([[c, 0, s], [0, 1, 0], [-s, 0, c]])


def rotz(a):
    a = np.radians(a); c, s = np.cos(a), np.sin(a)
    return np.array([[c, -s, 0], [s, c, 0], [0, 0, 1]])


def fish_group(name):
    mod = importlib.import_module('fish_' + name)
    (P, N, UV, T, part, layer), _ = mod.build()
    s, yaw, pitch, t = PLACE[name]
    R = roty(yaw) @ rotz(pitch)
    lo, hi = P.min(0), P.max(0); c = (lo + hi) / 2
    k = FISH_ORDER.index(name)
    anim = ('bob', np.asarray(t, float), 0.022, 9.0 + 1.7 * k, k * 0.9, 4.0)
    return (P, N, UV, T, part, layer), mod.material, (R, s, np.asarray(t, float), c), anim


def anim_matrix(anim, t):
    """4x4 in tank space for an animation at time t (s)."""
    M = np.eye(4)
    if anim[0] == 'spin':
        _, c, ax, period = anim
        ang = 2 * np.pi * t / period
        ax = normalize(np.asarray(ax, float)); K = np.array([[0, -ax[2], ax[1]], [ax[2], 0, -ax[0]], [-ax[1], ax[0], 0]])
        R = np.eye(3) + np.sin(ang) * K + (1 - np.cos(ang)) * K @ K
        M[:3, :3] = R; M[:3, 3] = np.asarray(c) - R @ np.asarray(c)
    elif anim[0] == 'bob':
        _, c, amp, period, ph, yaw = anim
        w = 2 * np.pi * t / period + ph
        R = roty(yaw * np.sin(w))
        M[:3, :3] = R; M[:3, 3] = np.asarray(c) - R @ np.asarray(c) + [amp * np.sin(w), 0.5 * amp * np.sin(2 * w), 0.3 * amp * np.cos(w)]
    elif anim[0] == 'fall':
        _, d, sp, period = anim
        M[:3, 3] = np.asarray(d, float) * sp * ((t / period) % 1.0)
    return M


def groups(names):
    """All groups for a fill: (name, mesh parts, material, transform-to-tank or None, anim)."""
    out = [('tank', group_tank().build(), material, None, ('none',))]
    for name, fn in (('crown', group_crown), ('hgcrystal', group_hgcrystal), ('grains', group_grains)):
        mm, anim = fn()
        out.append((name, mm.build(), material, None, anim))
    for nm in names:
        mesh, mat, xf, anim = fish_group(nm)
        out.append((nm, mesh, mat, xf, anim))
    return out


def scene(names, t=0.0):
    m = Mesh(); mats = {}; xfs = {}
    for gi, (gname, (P, N, UV, T, part, layer), mat, xf, anim) in enumerate(groups(names)):
        if xf is not None:
            R, s, tt, c = xf
            P = (P - c) @ R.T * s + tt; N = N @ R.T
        A = anim_matrix(anim, t)
        P = P @ A[:3, :3].T + A[:3, 3]; N = N @ A[:3, :3].T
        base = sum(len(x) for x in m.P)
        m.P.append(P); m.N.append(N); m.UV.append(UV); m.T.append(T + base)
        m.part.append(part + (2000 + 20 * gi if part.max() < 1000 else 0)); m.layer.append(layer)
        if part.max() < 1000:
            mats[gi] = (mat, xf, A)
    water = C('#14254a')

    def mat_all(part, uv, p, n):
        if part < 2000:
            r = material(part, uv, p, n)
            inside = part in (SAND, HGGLASS, HGSAND, GRAIN, HGCRYSTAL, HGMETAL)
        else:
            gi = (part - 2000) // 20; fmat, (R, s, tt, c), A = mats[gi]
            Ai = np.linalg.inv(A[:3, :3])
            pa = (p - A[:3, 3]) @ Ai.T; na = n @ Ai.T
            pl = ((pa - tt) / s) @ R + c; nl = na @ R
            r = fmat((part - 2000) % 20, uv, pl, nl)
            inside = True
        if inside:
            depth = np.clip(Z1 - p[:, 2], 0, 2) + np.clip(Y1 - p[:, 1], 0, 2) * 0.3
            f = 1 - np.exp(-depth * 0.8)
            r['albedo'] = mix(np.asarray(r['albedo'], float), water, f * 0.5)
            r['emit'] = np.asarray(r.get('emit', np.zeros((len(uv), 3))), float) + np.outer(f, water) * 0.2
        return r
    return m.build(), mat_all


def shoot(names, tag, W=1600, H=1000, ss=2, eye=(1.0, 0.95, 4.1), target=(0, 0.42, 0), fov=34, t=0.0, bloom=1.0):
    mesh, mat = scene(names, t)
    bg = lambda h, w: (np.array([0.02, 0.022, 0.04]) + np.array([0.05, 0.05, 0.09]) * np.exp(-((np.linspace(-1, 1, w)[None, :] * 0.9) ** 2 + ((np.linspace(0, 1, h)[:, None] - 0.4) * 1.5) ** 2))[..., None])
    lights = [((-0.3, 1.0, 0.45), (0.85, 0.85, 0.95)), ((0.8, 0.2, 0.6), (0.2, 0.26, 0.42)), ((0.1, 0.4, -1.0), (0.4, 0.5, 0.8))]
    img = render(mesh, mat, W, H, eye=eye, target=target, fov=fov, bg=bg, bloom=bloom, ss=ss, lights=lights,
                 env_up=(0.2, 0.24, 0.36), env_down=(0.07, 0.07, 0.1))
    save(img, OUT + f'tank2_{tag}.png')


if __name__ == '__main__':
    tag = sys.argv[1]
    sets = {'empty': [], 'three': ['barbel', 'moonfin', 'siren'], 'full': FISH_ORDER, 'test': FISH_ORDER}
    STATE.update({'empty': dict(lit=0.0, crystal=0.0), 'three': dict(lit=1.0, crystal=0.3)}.get(tag, dict(lit=1.0, crystal=1.0)))
    if tag == 'test':
        shoot(sets['full'], 'test', 800, 500, 1)
    elif tag == 'close':
        shoot(FISH_ORDER, 'close', 1600, 1000, 2, eye=(0.15, 0.6, 2.1), target=(0.0, 0.48, 0), fov=40)
    else:
        shoot(sets[tag], tag)
