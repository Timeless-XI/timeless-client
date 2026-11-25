from fishlib import *

LAMP = {}


def build():
    m = Mesh()
    sp = lambda u: np.stack([(u - 0.5) * 1.2, 0.012 * np.sin(u * np.pi), np.zeros_like(u)], -1)
    b = Body(1.2,
             ht=[(0, .016), (.12, .03), (.4, .08), (.65, .09), (.85, .075), (.93, .05), (.985, .02), (1, .006)],
             hb=[(0, .016), (.4, .07), (.7, .078), (.88, .055), (.95, .028), (1, .006)],
             wd=[(0, .01), (.4, .07), (.7, .075), (.88, .055), (.96, .025), (1, .006)], spine=sp, shape=2.6)
    b.mesh(m)
    # bony dorsal scutes as real ridges
    for u in np.linspace(0.35, 0.82, 11):
        t = b.top(np.array([u]), 0.96)[0]
        m.add(*prism(t, (0.25, 1, 0), 0.016, 0.03, sides=4, taper=0.5), BRASS)
    for z in (1, -1):
        for u in np.linspace(0.2, 0.86, 14):
            f = b.flank(np.array([u]), z, 0.15, 1.0)[0]
            m.add(*prism(f, (0.2, 0.1, z), 0.011, 0.016, sides=4, taper=0.6), BRASS)
    fin_along(m, b, 0.2, 0.3, lambda s: 0.02 + 0.06 * (1 - s) ** 0.6, 'top', rake=0.5)
    fin_along(m, b, 0.16, 0.26, lambda s: 0.02 + 0.05 * (1 - s) ** 0.6, 'bottom', rake=0.5)
    # heterocercal tail: long upper lobe
    tail_fin(m, b, lambda s: (0.35 + 0.95 * (1 - s) ** 2.2 + 0.25 * s ** 2, (0.5 - s) * 2 * 0.42 + 0.22 * (1 - s) ** 2), 0.26)
    for z in (1, -1):
        side_fin(m, b, 0.82, 0.14, 0.05, angle=45, droop=-30, zsign=z, yk=-0.7)
        side_fin(m, b, 0.45, 0.08, 0.04, angle=30, droop=-60, zsign=z, yk=-0.95)
        eye(m, b, 0.9, 0.35, 0.0105, z)
        for k in range(2):
            x0 = np.array([0.53 + 0.015 * k, -0.035, 0.012 * z * (1 + k)])
            whisker(m, [x0, x0 + [0.0, -0.03, 0.004 * z], x0 + [-0.01, -0.05, 0.006 * z]], 0.0028, 0.001)
    # the herald's light on the brow
    S, T, up, side = b.frame(np.array([0.86]))
    LAMP['c'] = S[0] + up[0] * b.ht(0.86) * 0.92
    m.add(*sphere(LAMP['c'], 0.016, 14, scale=(1.3, 0.7, 1)), GLOW)
    return m.build(), b


def material(part, uv, p, n):
    N = len(uv)
    if part == BODY:
        u, v, d = body_dirs(uv)
        skin = fbm(u * 80, v * 30, 5)
        base = mix(C('#2e5a60'), C('#0f2428'), np.clip(d, 0, 1))
        base = mix(base, C('#c8c6a8'), np.clip(-d - 0.35, 0, 1) * 1.4)
        base = base * (0.75 + 0.45 * skin[:, None])
        # lanterns along the lateral line, between the scutes
        lu = (u - 0.18) * 16; f = lu - np.floor(lu)
        lamp = (np.abs(np.abs(d) - 0.0) < 0.12) & (np.hypot((f - 0.5) * 0.075, d * 0.05) < 0.007) & (u > 0.18) & (u < 0.86)
        emit = np.zeros((N, 3)); emit[lamp] = C('#ffb84a') * 1.6
        base[lamp] = C('#ffd890')
        mouth = (u > 0.94) & (u < 0.955) & (d < -0.6)
        base[mouth] *= 0.3
        return dict(albedo=base, spec=0.7, gloss=50, height=skin * 1.2, emit=emit)
    if part == BRASS:
        w = fbm(uv[:, 0] * 8, uv[:, 1] * 8)
        return dict(albedo=mix(C('#7a6438'), C('#c8a868'), w), spec=0.9, gloss=40, height=w)
    if part == GLOW:
        return dict(albedo=np.tile(C('#ffe0a0'), (N, 1)), spec=1.5, gloss=100, emit=np.tile(C('#ffb040'), (N, 1)) * 2.2)
    if part == FIN:
        ray, t = fin_rays(uv, 14)
        base = mix(C('#2a4c52'), C('#0e1e22'), ray * 0.6)
        base = mix(base, C('#6a5a38'), np.clip(t - 0.7, 0, 1) * 2)
        return dict(albedo=base, spec=0.5, gloss=30, alpha=0.9 - 0.25 * t, sss=0.5, height=ray * 0.4)
    if part == EYE:
        return eye_mat(uv, C('#c88a2a'), None)
    if part == WHISK:
        return dict(albedo=np.tile(C('#b8a880'), (N, 1)), spec=0.3, gloss=20)
    return dict(albedo=np.tile([1, 0, 1.0], (N, 1)))
