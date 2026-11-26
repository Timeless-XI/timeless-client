import numpy as np, swim, importlib, tank2 as T
SIZE = {'barbel': (0.70, 0.11), 'oreeater': (0.6, 0.08), 'whiskerwise': (0.69, 0.07), 'admiral': (0.67, 0.11),
        'moonfin': (0.62, 0.09), 'rimecoil': (0.5, 0.07), 'herald': (0.96, 0.07), 'siren': (0.6, 0.1)}   # length, radius
X, Y1, Z = 1.5, 1.3, 0.95
HG_R, HG_TOP, HG_Z = 0.25, 0.9, -0.62

def segdist(p1, q1, p2, q2):
    d1, d2, r = q1 - p1, q2 - p2, p1 - p2
    a, e, f = d1 @ d1, d2 @ d2, d2 @ r; c = d1 @ r; b = d1 @ d2; den = a * e - b * b
    s = np.clip((b * f - c * e) / den, 0, 1) if den > 1e-12 else 0.0
    t = (b * s + f) / e
    if t < 0: t = 0; s = np.clip(-c / a, 0, 1)
    elif t > 1: t = 1; s = np.clip((b - c) / a, 0, 1)
    return np.linalg.norm((p1 + d1 * s) - (p2 + d2 * t))

P = {nm: swim.params(spec, swim.samples(spec, 1.0, 0.0), -0.3) for nm, spec in swim.PATHS.items()}
SIZE = {k: (v[0] * 1.1, v[1] * 1.1) for k, v in SIZE.items()}
def caps(nm, t):
    M = swim.matrix(P[nm], t); L, r = SIZE[nm]
    a = M @ [L / 2, 0, 0, 1]; b = M @ [-L / 2, 0, 0, 1]
    tank = lambda v: np.array([v[0], -v[1], -v[2]])
    return tank(a), tank(b), r
bad = {}
names = list(P)
for t in np.arange(0, 600, 0.25):
    cs = {nm: caps(nm, t) for nm in names}
    for i, a in enumerate(names):
        p, q, r = cs[a]
        for e in (p, q):
            if abs(e[0]) > X - r or e[2] > Z or e[2] < -Z + r or e[1] < 0.08 + r or e[1] > Y1 - r: bad[(a, 'wall')] = bad.get((a, 'wall'), 0) + 1
        # hourglass: a vertical segment at x=0,z=0 from 0 to HG_TOP
        pts = p[None] + (q - p)[None] * np.linspace(0, 1, 9)[:, None]
        dh = np.hypot(pts[:, 0], pts[:, 2] - HG_Z) - HG_R
        d = np.where(pts[:, 1] <= HG_TOP, dh, np.hypot(np.maximum(dh, 0), pts[:, 1] - HG_TOP))
        if d.min() < r: bad[(a, 'hourglass')] = bad.get((a, 'hourglass'), 0) + 1
        for b in names[i + 1:]:
            p2, q2, r2 = cs[b]
            if segdist(p, q, p2, q2) < r + r2: bad[(a, b)] = bad.get((a, b), 0) + 1
for k, v in sorted(bad.items(), key=lambda kv: -kv[1]): print(k, f'{v / 4:.1f} s of 600')
print('done')
