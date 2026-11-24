from fishlib import *

EYE_C = {}


def build():
    m = Mesh()
    b = Body(1.1,
             ht=[(0, .022), (.25, .07), (.55, .095), (.8, .085), (.92, .065), (.98, .045), (1, .025)],
             hb=[(0, .022), (.35, .068), (.75, .072), (.95, .05), (1, .028)],
             wd=[(0, .009), (.35, .055), (.75, .095), (.9, .105), (.97, .085), (1, .05)], shape=2.3)
    b.mesh(m)
    fin_along(m, b, 0.56, 0.66, lambda s: 0.02 + 0.11 * (1 - s) ** 0.5, 'top', rake=0.35, ruffle=0.003)
    fin_along(m, b, 0.18, 0.30, lambda s: 0.012 + 0.03 * np.sin(np.pi * s) ** 0.5, 'top', rake=0.2, ruffle=0.001, part=FIN2)  # adipose
    fin_along(m, b, 0.05, 0.42, lambda s: 0.02 + 0.06 * np.sin(np.pi * s) ** 0.4, 'bottom', rake=0.35, ruffle=0.005, freq=11)
    tail_fin(m, b, lambda s: (0.48 + 0.2 * np.sin(np.pi * s) + 0.1 * np.abs(2 * s - 1), (0.5 - s) * 2 * 0.6), 0.2)
    for z in (1, -1):
        side_fin(m, b, 0.82, 0.14, 0.06, angle=55, droop=-25, zsign=z, yk=-0.55)
        side_fin(m, b, 0.48, 0.09, 0.05, angle=35, droop=-55, zsign=z, yk=-0.95)
        S, T, up, side = b.frame(np.array([0.93]))
        e = S[0] + np.array([0, 0.045, 0.072 * z])
        eye(m, b, 0.93, 0.55, 0.012, z)
        EYE_C[z] = e
        # the long maxillary whiskers sweep back past the pectorals
        x0 = np.array([0.545, -0.005, 0.075 * z])
        whisker(m, [x0, x0 + [0.01, 0.015, 0.06 * z], x0 + [-0.08, 0.02, 0.15 * z], x0 + [-0.28, -0.02, 0.22 * z], x0 + [-0.5, -0.06, 0.24 * z], x0 + [-0.62, -0.1, 0.21 * z]], 0.007, 0.0012, n=70)
        for k, (dz, ln) in enumerate(((0.025, 0.09), (0.05, 0.07))):
            c0 = np.array([0.53, -0.04, dz * z])
            whisker(m, [c0, c0 + [0.01, -0.04, 0.01 * z], c0 + [-0.02, -ln, 0.02 * z]], 0.003, 0.0008)
    return m.build(), b


def material(part, uv, p, n):
    N = len(uv)
    if part == BODY:
        u, v, d = body_dirs(uv)
        mott = fbm(u * 26, v * 12, 5)
        spots = cellnoise(u * 46, v * 22 + fbm(u * 8, v * 4) * 0.8)
        base = mix(C('#8a6a34'), C('#3e2c16'), np.clip(d * 0.9 + 0.2, 0, 1))
        base = mix(base, C('#e6d6b0'), np.clip(-d - 0.35, 0, 1) * 1.4)
        base = base * (0.75 + 0.45 * mott[:, None])
        base *= (1 - 0.55 * np.clip((0.32 - spots) * 8, 0, 1) * (d > -0.4))[:, None]
        # the "spectacles": pale rings around each eye, like a scholar's reading glasses
        for z, e in EYE_C.items():
            dist = np.linalg.norm(p - e, axis=1)
            ring = np.clip(1 - np.abs(dist - 0.026) / 0.006, 0, 1)
            base = mix(base, C('#efe2b8'), ring * 0.9)
            bridge = (np.abs(p[:, 0] - 0.535) < 0.004) & (p[:, 1] > 0.03)
        base[bridge] = C('#e6d6a8')
        mouth = (u > 0.975) & (np.abs(d + 0.2) < 0.2)
        base[mouth] *= 0.3
        return dict(albedo=base, spec=0.9, gloss=60, height=mott * 0.8, sss=0.2)  # smooth slick skin
    if part in (FIN, FIN2):
        ray, t = fin_rays(uv, 12 if part == FIN else 2)
        base = mix(C('#5a4222'), C('#2c2010'), ray * 0.5)
        sp = vnoise(uv[:, 0] * 30, uv[:, 1] * 10) > 0.75
        base[sp] *= 0.5
        return dict(albedo=base, spec=0.4, gloss=30, alpha=0.9 - 0.3 * t, sss=0.5, height=ray * 0.3)
    if part == EYE:
        return eye_mat(uv, C('#8c7a2a'), C('#2a200c'))
    if part == WHISK:
        return dict(albedo=mix(C('#3a2a16'), C('#c9b07a'), uv[:, 0]), spec=0.6, gloss=40)
    return dict(albedo=np.tile([1, 0, 1.0], (N, 1)))
