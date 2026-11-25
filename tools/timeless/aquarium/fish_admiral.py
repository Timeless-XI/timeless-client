from fishlib import *


def build():
    m = Mesh()
    b = Body(1.0,
             ht=[(0, .032), (.15, .07), (.42, .165), (.68, .18), (.86, .15), (.96, .1), (1, .055)],
             hb=[(0, .032), (.38, .125), (.75, .14), (.92, .11), (1, .06)],
             wd=[(0, .014), (.45, .088), (.82, .088), (.95, .065), (1, .035)], shape=2.2)
    b.mesh(m)
    m.add(*sphere((0.455, -0.055, 0), 1.0, 18, scale=(0.065, 0.045, 0.06)), JAW)
    # spiny dorsal: tall notched spines, like the brim of a tricorne
    fin_along(m, b, 0.48, 0.80, lambda s: 0.05 + 0.07 * (0.6 + 0.4 * np.abs(np.sin(s * 9.5 * np.pi))) * (1 - 0.4 * s), 'top', rake=0.15, ruffle=0.003, freq=9)
    fin_along(m, b, 0.24, 0.49, lambda s: 0.06 + 0.06 * np.sin(np.pi * s) ** 0.5, 'top', rake=0.4, ruffle=0.004, part=FIN2)
    fin_along(m, b, 0.20, 0.38, lambda s: 0.05 + 0.06 * np.sin(np.pi * s) ** 0.6, 'bottom', rake=0.35, part=FIN2)
    tail_fin(m, b, lambda s: (0.56 + 0.18 * np.sin(np.pi * s), (0.5 - s) * 2 * 0.62), 0.22, part=FIN2)
    for z in (1, -1):
        side_fin(m, b, 0.74, 0.15, 0.10, angle=35, droop=-25, zsign=z, yk=-0.2, part=FIN2)
        side_fin(m, b, 0.6, 0.10, 0.06, angle=25, droop=-60, zsign=z, yk=-0.95, part=FIN2)
        eye(m, b, 0.885, 0.42, 0.017, z)
    # the cutlass: blade lodged in the jaw, curving up and back over the near cheek; guard and grip out the far side
    back = lambda s: np.stack([0.43 - 0.17 * s - 0.05 * s * s, -0.01 + 0.33 * s + 0.02 * s * s, 0.10 + 0.07 * s], -1)
    edge = lambda s: np.stack([0.43 - 0.17 * s - 0.05 * s * s + 0.045 * (1 - s) ** 0.35, -0.01 + 0.33 * s + 0.02 * s * s + 0.012 * (1 - s), 0.10 + 0.07 * s], -1)
    m.add(*sheet(back, edge, 50, 4), METAL)
    m.add(*sheet(lambda s: back(s) + [0, 0, -0.004], lambda s: edge(s) + [0, 0, -0.004], 50, 4), METAL)
    whisker(m, [(0.445, -0.035, 0.11), (0.47, -0.04, -0.02), (0.49, -0.045, -0.10)], 0.006, 0.006, part=METAL, n=10)
    whisker(m, [(0.49, -0.07, -0.10), (0.50, -0.02, -0.105), (0.49, 0.03, -0.10)], 0.006, 0.006, part=BRASS, n=10)
    whisker(m, [(0.49, -0.045, -0.105), (0.52, -0.05, -0.16), (0.545, -0.055, -0.21)], 0.011, 0.011, part=WOOD, n=10)
    m.add(*sphere((0.548, -0.056, -0.215), 0.014, 10), BRASS)
    return m.build(), b


def tatter(uv):
    s, t = uv[:, 0], uv[:, 1]
    torn = fbm(s * 14, 3.0, 3)
    return np.clip((0.9 + 0.25 * torn - t) * 8, 0, 1)


def material(part, uv, p, n):
    N = len(uv)
    if part == BODY:
        u, v, d = body_dirs(uv)
        hgt, rim = scales(u, v, 52, 34, edge=0.2)
        base = mix(C('#a21c1c'), C('#4a0a0e'), np.clip(d * 1.1, 0, 1))
        base = mix(base, C('#e08a72'), np.clip(-d - 0.3, 0, 1) * 1.2)
        gs = cellnoise(u * 48, v * 30)
        gold = np.clip((0.2 - gs) * 14, 0, 1) * (d > -0.5)
        base = mix(base, C('#e8b84a'), gold * 0.9)
        # old scars
        sc = (np.abs(fbm(u * 6, v * 3) - 0.5) < 0.006) & (d > -0.2) & (u < 0.8)
        base[sc] = C('#e0b0a0')
        mouth = (u > 0.965) & (np.abs(d + 0.25) < 0.14)
        base[mouth] *= 0.25
        return dict(albedo=base, spec=0.6, gloss=40, height=hgt * (u < 0.85))
    if part == JAW:
        return dict(albedo=np.tile(C('#7a1418'), (N, 1)), spec=0.5, gloss=40)
    if part in (FIN, FIN2):
        ray, t = fin_rays(uv, 11 if part == FIN else 18)
        base = mix(C('#8a1416'), C('#3a0608'), ray * 0.5)
        base = mix(base, C('#150405'), np.clip((t - 0.72) * 4, 0, 1))  # dark trailing edge
        a = (0.95 - 0.2 * t) * tatter(uv)
        if part == FIN:
            spine = ray > 0.55
            base[spine] = C('#8a5a3a')
            a = np.maximum(a * (0.55 + 0.45 * np.clip(1 - t, 0, 1)), spine * (t < 1.0) * 0.95)
        return dict(albedo=base, spec=0.4, gloss=30, alpha=a, sss=0.5, height=ray * 0.4)
    if part == EYE:
        return eye_mat(uv, C('#d8a028'), C('#4a1a08'))
    if part == METAL:
        nicks = fbm(uv[:, 0] * 30, uv[:, 1] * 2) > 0.66
        alb = np.tile(C('#b8c0c8'), (N, 1)); alb[nicks] = C('#6a5a4a')
        return dict(albedo=alb, spec=np.where(nicks, 0.3, 1.8), gloss=90)
    if part == BRASS:
        return dict(albedo=np.tile(C('#c09040'), (N, 1)), spec=1.3, gloss=60)
    if part == WOOD:
        wrap = 0.6 + 0.4 * (np.sin(uv[:, 0] * 60 + uv[:, 1] * 6.28) > 0)
        return dict(albedo=C('#3a2216')[None] * wrap[:, None], spec=0.2, gloss=15)
    return dict(albedo=np.tile([1, 0, 1.0], (N, 1)))
