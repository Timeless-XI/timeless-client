"""Shared fish-building helpers: a body loft along a spine plus fins, eyes, barbels and props."""
import numpy as np
from r3d import *

# part ids
TAIL_TAG = 0   # added to the tail fin's part id when set (tank3 splits the tail off to make it beat)
BODY, FIN, EYE, METAL, WOOD, CLOTH, GLOW, CRYSTAL, WHISK, FIN2, TEETH, BRASS, JAW = range(13)


def interp(pts):
    pts = np.asarray(pts, float)
    return lambda u: np.interp(u, pts[:, 0], pts[:, 1])


class Body:
    def __init__(self, length, ht, hb, wd, spine=None, up=None, shape=2.0, nu=150, nv=60):
        self.L = length
        cap = lambda f: (lambda u: f(u) * np.sqrt(np.clip(1 - ((np.asarray(u, float) - 0.97) / 0.03).clip(0, None) ** 2, 0, 1)) ** 0.8)
        self.ht, self.hb, self.wd = cap(interp(ht)), cap(interp(hb)), cap(interp(wd))
        self.spine = spine or (lambda u: np.stack([(u - 0.5) * length, np.zeros_like(u), np.zeros_like(u)], -1))
        self.up = up or (lambda u: np.tile([0.0, 1.0, 0.0], (len(np.atleast_1d(u)), 1)))
        self.shape = shape; self.nu = nu; self.nv = nv

    def frame(self, u):
        u = np.atleast_1d(np.asarray(u, float))
        eps = 1e-3
        S = self.spine(u)
        T = normalize(self.spine(np.clip(u + eps, 0, 1)) - self.spine(np.clip(u - eps, 0, 1)))
        up = normalize(self.up(u)); side = normalize(np.cross(T, up)); up = normalize(np.cross(side, T))
        return S, T, up, side

    def top(self, u, k=0.98):
        S, T, up, side = self.frame(u); return S + up * (self.ht(u) * k)[:, None]

    def bot(self, u, k=0.98):
        S, T, up, side = self.frame(u); return S - up * (self.hb(u) * k)[:, None]

    def flank(self, u, z=1, yk=0.0, k=0.95):
        S, T, up, side = self.frame(u)
        return S + side * (self.wd(u) * k * z)[:, None] + up * (yk * self.ht(u))[:, None]

    def mesh(self, m, part=BODY):
        P, N, UV, T = loft(self.spine, self.up, self.ht, self.hb, self.wd, self.nu, self.nv, self.shape)
        m.add(P, N, UV, T, part)


def fin_along(m, body, u0, u1, height, where='top', rake=0.3, part=FIN, ruffle=0.004, ns=50, nt=14, sidez=0.0, freq=7):
    """Fin rooted along the top (dorsal) or bottom (anal) line between u0 and u1.
    height(s) -> fin height; rake shifts the edge toward the tail by rake*height."""
    def root(s):
        u = u0 + (u1 - u0) * s
        return (body.top(u, 0.9) if where == 'top' else body.bot(u, 0.9)) + np.array([0, 0, sidez])

    def edge(s):
        u = u0 + (u1 - u0) * s
        S, T, up, side = body.frame(u)
        h = height(s)[:, None]
        sgn = 1 if where == 'top' else -1
        return root(s) + up * h * sgn - T * h * rake
    m.add(*sheet(root, edge, ns, nt, ruffle=ruffle, ruffle_freq=freq), part, layer=1)


def tail_fin(m, body, shape, size, part=FIN, ns=60, nt=18, ruffle=0.004, skew=0.0):
    """shape(s) -> (dx, dy) of the edge relative to the tail root, s from top tip (0) to bottom tip (1)."""
    S, T, up, side = body.frame(np.array([0.0]))
    base = S[0]; t = T[0]; u = up[0]
    h0 = max(body.ht(0.0), body.hb(0.0), 0.006)

    def root(s):
        y = (0.5 - s) * 2 * h0 * 0.9
        return base[None, :] + u[None, :] * y[:, None]

    def edge(s):
        dx, dy = shape(s)
        return base[None, :] - t[None, :] * (dx * size)[:, None] + u[None, :] * (dy * size + skew * size)[:, None]
    m.add(*sheet(root, edge, ns, nt, ruffle=ruffle, ruffle_freq=5), part + TAIL_TAG, layer=1)


def side_fin(m, body, u, length, width, angle=35, droop=-20, part=FIN, zsign=1, yk=-0.35, ruffle=0.003, shape='round'):
    """Pectoral / pelvic fin: rooted on the flank at body position u, swept back and out."""
    S, T, up, side = body.frame(np.array([u]))
    S, T, up, side = S[0], T[0], up[0], side[0]
    side = side * zsign
    root_c = S + side * body.wd(u) * 0.85 + up * yk * (body.hb(u) if yk < 0 else body.ht(u))
    a = np.radians(angle); d = np.radians(droop)
    out = normalize(-T * np.cos(a) + side * np.sin(a) + up * np.sin(d))
    perp = normalize(np.cross(out, side)) * np.sign(np.dot(np.cross(out, side), up) + 1e-9)

    def root(s):
        return root_c[None, :] + perp[None, :] * ((s - 0.5) * width * 0.35)[:, None]

    def edge(s):
        if shape == 'round':
            prof = np.sin(np.pi * np.clip(s, 0, 1)) ** 0.6
        else:
            prof = 0.5 + 0.5 * np.sin(np.pi * s)
        return root_c[None, :] + out[None, :] * (length * (0.55 + 0.45 * prof))[:, None] + perp[None, :] * ((s - 0.5) * width)[:, None]
    m.add(*sheet(root, edge, 30, 12, ruffle=ruffle, ruffle_freq=4), part, layer=1)


def eye(m, body, u, yk, r, zsign=1, part=EYE, bulge=0.6):
    S, T, up, side = body.frame(np.array([u]))
    c = S[0] + side[0] * zsign * body.wd(u) * (1 - r / max(body.wd(u), 1e-6) * (1 - bulge)) + up[0] * yk * body.ht(u)
    P, N, UV, Tr = sphere((0, 0, 0), r, 16)
    # rotate so the sphere's pole (+y) points out of the flank
    d = normalize(side[0] * zsign * 0.85 + T[0] * 0.25 + up[0] * 0.1)
    yv = np.array([0, 1.0, 0]); ax = np.cross(yv, d); s = np.linalg.norm(ax); cth = np.dot(yv, d)
    if s < 1e-6:
        Rm = np.eye(3) * (1 if cth > 0 else -1)
    else:
        ax /= s; K = np.array([[0, -ax[2], ax[1]], [ax[2], 0, -ax[0]], [-ax[1], ax[0], 0]])
        Rm = np.eye(3) + K * s + K @ K * (1 - cth)
    m.add(P @ Rm.T + c, N @ Rm.T, UV, Tr, part)


def whisker(m, pts, r0, r1, part=WHISK, n=40):
    pts = np.asarray(pts, float)
    t = np.linspace(0, 1, len(pts))

    def curve(u):
        return np.stack([np.interp(u, t, pts[:, i]) for i in range(3)], -1)
    # smooth the control polyline with a Catmull-Rom-ish resample
    m.add(*tube(curve, lambda u: r0 + (r1 - r0) * u, n=n, nv=8), part)


def body_dirs(material_uv):
    """uv -> (u along, v around), with handy masks: top (dorsal) and belly."""
    u, v = material_uv[:, 0], material_uv[:, 1]
    ang = v * 2 * np.pi
    dors = np.cos(ang)  # +1 top, -1 belly
    return u, v, dors


def fin_rays(uv, n_rays=14, softness=0.35):
    s, t = uv[:, 0], uv[:, 1]
    ray = 0.5 + 0.5 * np.cos(s * n_rays * 2 * np.pi)
    return ray ** 3, t


def eye_mat(uv, iris, ring=None, milky=False):
    th = uv[:, 0]  # 0 at the pole (pupil)
    n = len(th)
    alb = np.tile(np.array([0.85, 0.82, 0.7]), (n, 1))
    iris_m = th < 0.30
    alb[iris_m] = iris
    if ring is not None:
        alb[(th > 0.24) & (th < 0.30)] = ring
    pup = th < (0.13 if not milky else 0.0)
    alb[pup] = [0.02, 0.02, 0.03]
    if milky:
        alb = alb * 0.4 + np.array([0.75, 0.78, 0.8]) * 0.6
    return dict(albedo=alb, spec=1.4, gloss=90)
