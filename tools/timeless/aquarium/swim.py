"""Swimming paths for the Elder fish in the Timeless Aquarium (v3).

Every fish follows its own closed loop through the tank (tank space, y up), at a gently changing speed, rising and
sinking a little, turning to face its way, pitching as it climbs and banking into its turns. Its body sways and its
tail fin beats.

The client gets each loop as samples in FURNITURE space (-y up, scaled by K, z flipped): per sample x, y, z, yaw,
pitch, roll (radians), in glam's terms: the fish model faces +x and

    M(t) = T(pos) * Ry(yaw) * Rz(pitch) * Rx(roll) * Ry(sway)           (the body)
    M_tail(t) = M(t) * T(joint) * Ry(wag) * T(-joint)                    (the tail fin, joint = (jx, 0, 0))

with sway = A_s sin(2 pi f t), wag = A_w sin(2 pi f t - lag). `matrix()` is the same maths in numpy, for previews
and checks; the client's `Anim::local` (aquarium.rs) must match it.
"""
import numpy as np

N_SAMPLES = 256


def superellipse(u, A, R, n=4.0):
    c, s = np.cos(2 * np.pi * u), np.sin(2 * np.pi * u)
    return A * np.sign(c) * np.abs(c) ** (2 / n), R * np.sign(s) * np.abs(s) ** (2 / n)


def eight(u, A, R):
    return A * np.sin(2 * np.pi * u), R * np.sin(4 * np.pi * u)


# name: dict(shape, centre (x, y, z), A, R, lap seconds, start (0-1), direction, rise amplitude, rises per lap,
#            speed wobble, beat Hz, sway deg, wag deg)
PATHS = {
    # four layers, two fish each, never touching: the lowest pair each loop its own half of the tank, the next pair
    # each traces a figure of eight in its own half, the two upper pairs share one loop each, half a lap apart
    # (same lap time and speed changes, so the gap never closes). The hourglass stands at the back, clear of all.
    'oreeater':    dict(shape='loop',  c=(-0.74, 0.27, 0.16), A=0.38, R=0.48, lap=22, start=0.0,  dir=1,  rise=0.025, k=2, wob=0.3, beat=0.9, sway=4, wag=20),
    'whiskerwise': dict(shape='loop',  c=(0.74, 0.27, 0.16),  A=0.38, R=0.48, lap=25, start=0.4,  dir=-1, rise=0.025, k=3, wob=0.3, beat=0.8, sway=5, wag=18),
    'siren':       dict(shape='eight', c=(-0.72, 0.55, 0.26), A=0.38, R=0.36, lap=26, start=0.2,  dir=1,  rise=0.03,  k=1, wob=0.35, beat=0.7, sway=3, wag=14),
    'admiral':     dict(shape='eight', c=(0.72, 0.55, 0.26),  A=0.38, R=0.36, lap=24, start=0.7,  dir=-1, rise=0.02,  k=1, wob=0.3, beat=1.0, sway=4, wag=20),
    'moonfin':     dict(shape='loop',  c=(0.0, 0.83, 0.27),    A=1.0, R=0.45, lap=34, start=0.0,  dir=1,  rise=0.03,  k=2, wob=0.25, beat=0.6, sway=4, wag=16),
    'barbel':      dict(shape='loop',  c=(0.0, 0.83, 0.27),    A=1.0, R=0.45, lap=34, start=0.5,  dir=1,  rise=0.02,  k=3, wob=0.25, beat=1.1, sway=4, wag=22),
    'herald':      dict(shape='loop',  c=(0.0, 1.1, 0.05),  A=1.0, R=0.58, lap=40, start=0.25, dir=-1, rise=0.025, k=2, wob=0.2, beat=0.7, sway=3, wag=14),
    'rimecoil':    dict(shape='loop',  c=(0.0, 1.1, 0.05),  A=1.0, R=0.58, lap=40, start=0.75, dir=-1, rise=0.025, k=3, wob=0.2, beat=0.5, sway=6, wag=0),
}


def lap_points(spec, n=2048):
    """Tank-space points of one lap at uniform arc length, then eased by the speed wobble."""
    u = np.arange(n) / n
    if spec['shape'] == 'loop':
        x, z = superellipse(u, spec['A'], spec['R'])
    else:
        x, z = eight(u, spec['A'], spec['R'])
    if spec['dir'] < 0:
        x = x[::-1]; z = z[::-1]
    cx, cy, cz = spec['c']
    P = np.stack([cx + x, np.full(n, cy), cz + z], 1)
    seg = np.linalg.norm(np.roll(P, -1, 0) - P, axis=1)
    s = np.concatenate([[0], np.cumsum(seg)])[:-1] / seg.sum()
    # arc-length parameter with a speed wobble: s' = s + w sin(2 pi m s) / (2 pi m)
    m = 2
    return P, s, seg.sum()


def samples(spec, K, floor, n=N_SAMPLES):
    """Furniture-space samples (n x 6): x, y, z, yaw, pitch, roll."""
    P, s, length = lap_points(spec)
    # eased arc parameter: time fraction tau -> arc fraction a(tau) (monotone while wob < 1)
    tau = np.arange(n) / n
    m = 2
    a = tau + spec['wob'] * np.sin(2 * np.pi * m * tau) / (2 * np.pi * m)
    a = np.mod(a, 1.0)
    idx = np.interp(a, np.concatenate([s, [1.0]]), np.arange(len(s) + 1))
    i0 = np.floor(idx).astype(int) % len(P); i1 = (i0 + 1) % len(P); f = (idx - np.floor(idx))[:, None]
    Q = P[i0] * (1 - f) + P[i1] * f
    Q[:, 1] += spec['rise'] * np.sin(2 * np.pi * spec['k'] * tau + spec['start'] * 7)
    # furniture space
    F = np.stack([Q[:, 0] * K, -(Q[:, 1] - floor) * K, -Q[:, 2] * K], 1)
    # direction of travel and its turning (central differences, closed loop)
    d = (np.roll(F, -1, 0) - np.roll(F, 1, 0)) / 2
    fwd = d / np.linalg.norm(d, axis=1, keepdims=True)
    yaw = np.arctan2(-fwd[:, 2], fwd[:, 0])
    pitch = np.arcsin(np.clip(fwd[:, 1], -1, 1)) * 0.8
    acc = np.roll(F, -1, 0) - 2 * F + np.roll(F, 1, 0)
    # the lateral pull in the fish's own frame (its +z side), from the turn
    side = np.stack([-np.sin(yaw), np.zeros(n), np.cos(yaw)], 1)          # Ry(yaw) * (0, 0, 1)
    seglen = np.linalg.norm(d, axis=1)
    lat = np.sum(acc * side, 1) / np.maximum(seglen ** 2, 1e-9)            # curvature, signed
    lat = np.convolve(np.concatenate([lat[-8:], lat, lat[:8]]), np.ones(9) / 9, 'same')[8:-8]
    roll = -np.clip(lat * 0.12, -0.38, 0.38)
    yaw = np.unwrap(yaw)
    return np.column_stack([F, yaw, pitch, roll]).astype(np.float32)


def ry(a):
    c, s = np.cos(a), np.sin(a); return np.array([[c, 0, s, 0], [0, 1, 0, 0], [-s, 0, c, 0], [0, 0, 0, 1.0]])


def rz(a):
    c, s = np.cos(a), np.sin(a); return np.array([[c, -s, 0, 0], [s, c, 0, 0], [0, 0, 1, 0], [0, 0, 0, 1.0]])


def rx(a):
    c, s = np.cos(a), np.sin(a); return np.array([[1, 0, 0, 0], [0, c, -s, 0], [0, s, c, 0], [0, 0, 0, 1.0]])


def tr(v):
    M = np.eye(4); M[:3, 3] = v; return M


def params(spec, smp, joint_x):
    """The client's parameter list for anim kind 4."""
    return [float(spec['lap']), float(spec['start']), np.radians(spec['sway']), float(spec['beat']), float(joint_x),
            np.radians(spec['wag']), 1.2, float(len(smp))] + smp.reshape(-1).astype(float).tolist()


def matrix(p, t, tail=False):
    """Furniture-space 4x4 at t for a kind-4 parameter list (the client's Anim::local)."""
    period, phase, sway_a, beat, jx, wag_a, lag, n = p[:8]
    n = int(n); S = np.asarray(p[8:8 + 6 * n], float).reshape(n, 6)
    x = ((t / period + phase) % 1.0) * n
    i0 = int(np.floor(x)) % n; i1 = (i0 + 1) % n; f = x - np.floor(x)
    a, b = S[i0], S[i1].copy()
    b[3] = a[3] + ((b[3] - a[3] + np.pi) % (2 * np.pi) - np.pi)
    v = a * (1 - f) + b * f
    w = 2 * np.pi * beat * t
    M = tr(v[:3]) @ ry(v[3]) @ rz(v[4]) @ rx(v[5]) @ ry(sway_a * np.sin(w))
    if tail and wag_a != 0:
        M = M @ tr((jx, 0, 0)) @ ry(wag_a * np.sin(w - lag)) @ tr((-jx, 0, 0))
    return M
