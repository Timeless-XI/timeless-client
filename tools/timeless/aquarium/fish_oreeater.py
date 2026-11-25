from fishlib import *

rng = np.random.default_rng(4)


def build():
    m = Mesh()
    b = Body(0.92,
             ht=[(0, .03), (.1, .05), (.35, .12), (.6, .158), (.8, .168), (.92, .148), (.98, .1), (1, .06)],
             hb=[(0, .03), (.3, .085), (.7, .128), (.9, .125), (.97, .09), (1, .05)],
             wd=[(0, .013), (.4, .082), (.8, .102), (.95, .085), (1, .045)], shape=2.4)
    b.mesh(m)
    # heavy underslung grinder jaw
    m.add(*sphere((0.43, -0.082, 0), 1.0, 18, scale=(0.075, 0.042, 0.078)), JAW)
    for i, a in enumerate(np.linspace(-1.1, 1.1, 9)):
        x = 0.43 + 0.07 * np.cos(a); z = 0.075 * np.sin(a)
        m.add(*prism((x, -0.052, z), (0.3, 1, 0), 0.008, 0.034 + 0.012 * (i % 2), sides=4, taper=0.4), TEETH)
    # ore crystals growing out of the dorsal ridge (mythril blue-silver and copper)
    us = np.linspace(0.36, 0.84, 9)
    for i, u in enumerate(us):
        top = b.top(np.array([u]), 0.92)[0]
        for k in range(2 if i % 3 else 3):
            ax = (rng.normal(0, 0.35), 1, rng.normal(0, 0.45))
            h = 0.07 + 0.07 * rng.random() * (1 - abs(u - 0.62) * 1.2)
            m.add(*prism(top + (rng.normal(0, 0.012), -0.008, rng.normal(0, 0.02)), ax, 0.016 + 0.01 * rng.random(), h,
                         sides=6, taper=0.1), GLOW if (i + k) % 3 else CRYSTAL)
    fin_along(m, b, 0.12, 0.32, lambda s: 0.02 + 0.05 * np.sin(np.pi * s), 'top', rake=0.4, ruffle=0.002)
    fin_along(m, b, 0.12, 0.3, lambda s: 0.02 + 0.045 * np.sin(np.pi * s), 'bottom', rake=0.4)
    tail_fin(m, b, lambda s: (0.55 + 0.12 * np.sin(np.pi * s), (0.5 - s) * 2 * 0.55), 0.17)
    for z in (1, -1):
        side_fin(m, b, 0.74, 0.11, 0.08, angle=40, droop=-45, zsign=z, yk=-0.5)
        side_fin(m, b, 0.55, 0.07, 0.05, angle=25, droop=-65, zsign=z, yk=-0.95)
        eye(m, b, 0.9, 0.42, 0.011, z, bulge=0.3)
    return m.build(), b


def material(part, uv, p, n):
    N = len(uv)
    if part == BODY:
        u, v, d = body_dirs(uv)
        hgt, rim = scales(u, v, 14, 12, edge=0.22)  # big riveted plates
        cav = fbm(u * 30, v * 14, 5)
        base = mix(C('#3a3f45'), C('#1b1d22'), np.clip(d, 0, 1))
        base = mix(base, C('#7c7a72'), np.clip(-d - 0.2, 0, 1) * 0.8)
        plate = (d > -0.3)
        base = mix(base, C('#5d6670'), (hgt > 0.25) * plate * 0.55)
        base = base * (0.7 + 0.5 * cav[:, None])
        # ore flecks glinting in the hide
        fl = (cellnoise(u * 90, v * 50) < 0.12) & (hash2(np.floor(u * 90), np.floor(v * 50)) > 0.7) & (d > -0.4)
        base[fl] = C('#a9c6e6')
        mouth = (u > 0.97) & (np.abs(d + 0.45) < 0.15)
        base[mouth] *= 0.2
        sp = np.where(fl, 2.0, 0.25 + 0.9 * (hgt > 0.25) * plate)
        return dict(albedo=base, spec=sp, gloss=55, height=(hgt * 1.4 + cav * 0.6) * (u < 0.95))
    if part == JAW:
        cav = fbm(uv[:, 0] * 20, uv[:, 1] * 20)
        return dict(albedo=C('#4a4f56')[None] * (0.7 + 0.5 * cav[:, None]), spec=0.5, gloss=40, height=cav * 1.5)
    if part == TEETH:
        return dict(albedo=np.tile(C('#cfc6ae'), (N, 1)), spec=0.6, gloss=40)
    if part == CRYSTAL:
        f = fbm(uv[:, 0] * 5, uv[:, 1] * 5)
        return dict(albedo=mix(C('#b5642a'), C('#e09a4a'), f), spec=1.6, gloss=80, emit=np.tile(C('#3a1a08'), (N, 1)) * 0.6)
    if part == GLOW:
        f = fbm(uv[:, 0] * 5, uv[:, 1] * 5)
        return dict(albedo=mix(C('#7fa6d6'), C('#d8ecff'), f), spec=2.0, gloss=120, emit=np.tile(C('#4a86d8'), (N, 1)) * 0.8)
    if part == FIN:
        ray, t = fin_rays(uv, 9)
        base = mix(C('#2e3238'), C('#0f1013'), ray * 0.7)
        return dict(albedo=base, spec=0.5, gloss=30, alpha=0.95 - 0.25 * t, height=ray * 0.6)
    if part == EYE:
        return eye_mat(uv, C('#9aa3a8'), None, milky=True)
    return dict(albedo=np.tile([1, 0, 1.0], (N, 1)))
