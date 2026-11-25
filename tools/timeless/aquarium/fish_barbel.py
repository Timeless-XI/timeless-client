from fishlib import *


def build():
    m = Mesh()
    sp = lambda u: np.stack([(u - 0.5) * 1.0, -0.025 * np.clip((u - 0.82) / 0.18, 0, 1) ** 2, np.zeros_like(u)], -1)
    b = Body(1.0,
             ht=[(0, .028), (.08, .042), (.3, .108), (.52, .135), (.72, .122), (.88, .088), (.96, .055), (1, .016)],
             hb=[(0, .028), (.1, .038), (.35, .078), (.6, .092), (.84, .078), (.95, .05), (1, .014)],
             wd=[(0, .011), (.15, .028), (.5, .068), (.8, .064), (.94, .042), (1, .012)], spine=sp)
    b.mesh(m)
    fin_along(m, b, 0.47, 0.66, lambda s: 0.03 + 0.15 * (1 - s) ** 0.7, 'top', rake=0.45, ruffle=0.003)
    fin_along(m, b, 0.17, 0.28, lambda s: 0.02 + 0.07 * (1 - s) ** 0.8, 'bottom', rake=0.5)
    tail_fin(m, b, lambda s: (0.40 + 0.62 * np.abs(2 * s - 1) ** 1.5, (0.5 - s) * 2 * 0.64), 0.25)
    for z in (1, -1):
        side_fin(m, b, 0.80, 0.13, 0.06, angle=30, droop=-35, zsign=z, yk=-0.6)
        side_fin(m, b, 0.50, 0.10, 0.05, angle=20, droop=-55, zsign=z, yk=-0.95)
        eye(m, b, 0.905, 0.28, 0.017, z)
        # barbels: long pair at the mouth corners, short pair on the upper lip
        x0 = np.array([0.475, -0.052, 0.022 * z])
        whisker(m, [x0, x0 + [0.02, -0.04, 0.02 * z], x0 + [0.0, -0.085, 0.045 * z], x0 + [-0.04, -0.115, 0.06 * z]], 0.0045, 0.0012)
        x1 = np.array([0.495, -0.03, 0.012 * z])
        whisker(m, [x1, x1 + [0.03, -0.015, 0.01 * z], x1 + [0.045, -0.04, 0.02 * z]], 0.0035, 0.001)
    # the broken royal lance: shaft through the lower lip, steel head out the far side, a scrap of pennant
    whisker(m, [(0.33, -0.13, 0.25), (0.40, -0.095, 0.14), (0.47, -0.06, 0.03)], 0.014, 0.013, part=WOOD, n=20)
    whisker(m, [(0.47, -0.06, 0.03), (0.49, -0.061, 0.0)], 0.017, 0.017, part=BRASS, n=4)
    whisker(m, [(0.49, -0.061, 0.0), (0.53, -0.064, -0.06), (0.575, -0.068, -0.13)], lambda u: 0.022 * np.sin(np.pi * np.clip(u * 0.9 + 0.1, 0, 1)) ** 0.8 + 0.001, None, part=METAL, n=30) if False else None
    head = lambda u: np.stack([0.49 + 0.085 * u, -0.061 - 0.007 * u, -0.13 * u], -1)
    m.add(*tube(head, lambda u: 0.024 * np.sin(np.pi * (0.12 + 0.88 * u)) ** 0.7 * (1 - u) ** 0.15 + 0.0008, n=30, nv=4), METAL)
    pen = lambda s: np.stack([0.33 + 0.055 * s, -0.13 + 0.022 * s, 0.245 - 0.08 * s], -1)
    pen_e = lambda s: np.stack([0.30 + 0.05 * s, -0.25 + 0.07 * s + 0.015 * np.sin(s * 6), 0.235 - 0.07 * s], -1)
    m.add(*sheet(pen, pen_e, 14, 10, ruffle=0.01, ruffle_freq=3), CLOTH, layer=0)
    return m.build(), b


def material(part, uv, p, n):
    N = len(uv)
    if part == BODY:
        u, v, d = body_dirs(uv)
        hgt, rim = scales(u, v, 44, 30, edge=0.14)
        mott = fbm(u * 18, v * 9)
        top, mid, bel = C('#3d4422'), C('#a6863a'), C('#e3d3a0')
        t = np.clip((d + 0.25) / 1.0, 0, 1)
        base = np.where((d > 0)[:, None], mix(mid, top, np.clip(d * 1.6, 0, 1)), mix(mid, bel, np.clip(-d * 1.8, 0, 1)))
        base = base * (0.85 + 0.3 * mott[:, None])
        base = mix(base, C('#d8c27a'), rim * 0.22 * (d > -0.6))
        base = mix(base, C('#1f2412'), np.clip(d - 0.55, 0, 1) * 1.2)
        lat = (np.abs(d - 0.05) < 0.035) & (np.sin(u * 160) > 0.2) & (u > 0.12) & (u < 0.9)
        base[lat] *= 0.55
        mouth = (u > 0.965) & (np.abs(d + 0.35) < 0.12)
        base[mouth] *= 0.25
        gc = np.abs(u - 0.845 + 0.03 * (1 - d * d))
        base *= (1 - 0.45 * np.clip(1 - gc / 0.012, 0, 1) * (d > -0.75) * (d < 0.8))[:, None]
        return dict(albedo=base, spec=0.55, gloss=45, height=hgt * (u < 0.86) * (u > 0.06))
    if part == FIN:
        ray, t = fin_rays(uv, 16)
        base = mix(C('#c45a2a'), C('#7a2e1c'), ray * 0.6)
        base = mix(base, C('#e08a4a'), np.clip(t - 0.6, 0, 1))
        return dict(albedo=base, spec=0.4, gloss=30, alpha=0.88 - 0.3 * t, sss=0.6, height=ray * 0.3)
    if part == EYE:
        return eye_mat(uv, C('#c9a238'), C('#3a2a10'))
    if part == WHISK:
        return dict(albedo=np.tile(C('#b89a68'), (N, 1)), spec=0.3, gloss=20)
    if part == WOOD:
        g = 0.75 + 0.25 * np.sin(uv[:, 1] * 40 + fbm(uv[:, 0] * 8, uv[:, 1] * 3) * 6)
        return dict(albedo=C('#6b4426')[None] * g[:, None], spec=0.15, gloss=12)
    if part == METAL:
        rust = fbm(uv[:, 0] * 10, uv[:, 1] * 6) > 0.62
        alb = np.tile(C('#9aa3ad'), (N, 1)); alb[rust] = C('#7a4a2a')
        return dict(albedo=alb, spec=np.where(rust, 0.2, 1.3), gloss=70)
    if part == BRASS:
        return dict(albedo=np.tile(C('#b58a3a'), (N, 1)), spec=1.1, gloss=50)
    if part == CLOTH:
        s, t = uv[:, 0], uv[:, 1]
        alb = np.tile(C('#6a1408'), (N, 1)); alb[t > 0.82] = C('#a8862e')
        alb *= (0.8 + 0.2 * fbm(s * 6, t * 6))[:, None]
        return dict(albedo=alb, spec=0.1, gloss=8, sss=0.2)
    return dict(albedo=np.tile([1, 0, 1.0], (N, 1)))
