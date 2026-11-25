from fishlib import *

rng = np.random.default_rng(9)
BULB = {}


def build():
    m = Mesh()
    b = Body(0.8,
             ht=[(0, .03), (.2, .07), (.5, .17), (.78, .2), (.92, .17), (.98, .12), (1, .08)],
             hb=[(0, .03), (.4, .15), (.75, .19), (.92, .17), (1, .11)],
             wd=[(0, .014), (.4, .13), (.75, .16), (.92, .14), (1, .1)], shape=2.0)
    b.mesh(m)
    # huge underslung jaw and a fence of needle teeth
    m.add(*sphere((0.37, -0.075, 0), 1.0, 20, scale=(0.07, 0.07, 0.15)), JAW)
    for i, a in enumerate(np.linspace(-1.25, 1.25, 15)):
        x = 0.375 + 0.07 * np.cos(a); z = 0.14 * np.sin(a)
        h = 0.035 + 0.03 * (i % 3 == 1) + 0.01 * rng.random()
        m.add(*prism((x, -0.02, z), (0.5 * np.cos(a), 1, 0.3 * np.sin(a)), 0.006, h, sides=4, taper=0.5), TEETH)
    for i, a in enumerate(np.linspace(-1.1, 1.1, 11)):
        x = 0.385 + 0.04 * np.cos(a); z = 0.12 * np.sin(a)
        m.add(*prism((x, 0.03, z), (0.4 * np.cos(a), -1, 0.2 * np.sin(a)), 0.005, 0.03 + 0.01 * (i % 2), sides=4, taper=0.5), TEETH)
    # the illicium: a rod arching forward from the brow, ending in the singing lantern
    curve = [(0.28, 0.19, 0), (0.33, 0.33, 0.0), (0.45, 0.38, 0.0), (0.56, 0.31, 0.0), (0.6, 0.22, 0.0)]
    whisker(m, curve, 0.008, 0.004, part=WHISK, n=50)
    BULB['c'] = np.array([0.605, 0.195, 0.0])
    m.add(*sphere(BULB['c'], 0.03, 18, scale=(1, 1.15, 1)), GLOW)
    for k in range(5):
        a = k / 5 * 2 * np.pi
        tip = BULB['c'] + [0.03 * np.cos(a) + 0.01, -0.05 - 0.02 * (k % 2), 0.03 * np.sin(a)]
        whisker(m, [BULB['c'] + [0, -0.025, 0], tip], 0.002, 0.0006, part=GLOW, n=8)
    fin_along(m, b, 0.16, 0.3, lambda s: 0.02 + 0.05 * np.sin(np.pi * s), 'top', rake=0.4, part=FIN)
    fin_along(m, b, 0.16, 0.3, lambda s: 0.02 + 0.05 * np.sin(np.pi * s), 'bottom', rake=0.4, part=FIN)
    tail_fin(m, b, lambda s: (0.5 + 0.15 * np.sin(np.pi * s), (0.5 - s) * 2 * 0.5), 0.18)
    for z in (1, -1):
        side_fin(m, b, 0.55, 0.1, 0.09, angle=55, droop=-30, zsign=z, yk=-0.3)
        eye(m, b, 0.86, 0.62, 0.012, z, bulge=0.4)
    return m.build(), b


def material(part, uv, p, n):
    N = len(uv)
    if part == BODY:
        u, v, d = body_dirs(uv)
        blot = fbm(u * 14, v * 9, 4)
        warts = vnoise(u * 160, v * 90)
        base = mix(C('#2a1a3a'), C('#0c0812'), np.clip(blot * 1.6 - 0.4, 0, 1))
        base = mix(base, C('#4a3050'), np.clip(-d - 0.4, 0, 1))
        base[warts > 0.85] = C('#5a4468')
        # the lure lights the face from above
        dist = np.linalg.norm(p - BULB['c'], axis=1)
        emit = np.outer(np.clip(1 - dist / 0.35, 0, 1) ** 2, C('#2ad8c8')) * 0.35 * np.clip(np.sum(n * (BULB['c'] - p), 1) / np.maximum(dist, 1e-6), 0, 1)[:, None]
        mouth = (u > 0.93) & (np.abs(d + 0.1) < 0.25)
        base[mouth] *= 0.15
        return dict(albedo=base, spec=0.6, gloss=40, height=warts * 1.5 + blot, emit=emit)
    if part == JAW:
        warts = vnoise(uv[:, 0] * 60, uv[:, 1] * 60)
        dist = np.linalg.norm(p - BULB['c'], axis=1)
        emit = np.outer(np.clip(1 - dist / 0.4, 0, 1) ** 2, C('#2ad8c8')) * 0.3
        return dict(albedo=mix(C('#1e1228'), C('#3a2846'), warts), spec=0.5, gloss=40, height=warts, emit=emit)
    if part == TEETH:
        return dict(albedo=np.tile(C('#e8e4d8'), (N, 1)), spec=0.9, gloss=60, sss=0.5)
    if part == WHISK:
        return dict(albedo=np.tile(C('#3a2a48'), (N, 1)), spec=0.5, gloss=30)
    if part == GLOW:
        return dict(albedo=np.tile(C('#c8fff6'), (N, 1)), spec=1.5, gloss=100, emit=np.tile(C('#3af0dc'), (N, 1)) * 2.6)
    if part == FIN:
        ray, t = fin_rays(uv, 10)
        base = mix(C('#2a1838'), C('#0a060e'), ray * 0.6)
        return dict(albedo=base, spec=0.4, gloss=30, alpha=0.9 - 0.3 * t, sss=0.4, height=ray * 0.4)
    if part == EYE:
        return eye_mat(uv, C('#b8e848'), None)
    return dict(albedo=np.tile([1, 0, 1.0], (N, 1)))
