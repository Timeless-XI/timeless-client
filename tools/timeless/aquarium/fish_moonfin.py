from fishlib import *


def build():
    m = Mesh()
    b = Body(0.85,
             ht=[(0, .018), (.2, .045), (.5, .092), (.75, .088), (.92, .06), (1, .02)],
             hb=[(0, .018), (.3, .06), (.6, .085), (.85, .07), (1, .02)],
             wd=[(0, .008), (.4, .036), (.75, .04), (.93, .03), (1, .01)])
    b.mesh(m)
    veil = lambda s: 0.05 + 0.2 * np.sin(np.pi * (0.15 + 0.85 * s)) ** 0.6 * (1 - 0.3 * s)
    fin_along(m, b, 0.22, 0.78, lambda s: veil(1 - s), 'top', rake=0.9, ruffle=0.018, freq=6)
    fin_along(m, b, 0.18, 0.62, lambda s: 0.04 + 0.17 * np.sin(np.pi * s) ** 0.5, 'bottom', rake=0.95, ruffle=0.016, freq=5)
    tail_fin(m, b, lambda s: (0.85 + 0.6 * np.sin(np.pi * s) ** 0.8 - 0.25 * np.abs(2 * s - 1), (0.5 - s) * 2 * (0.55 + 0.25 * np.sin(np.pi * s))), 0.36, ruffle=0.02)
    for z in (1, -1):
        side_fin(m, b, 0.78, 0.2, 0.05, angle=30, droop=-15, zsign=z, yk=-0.3, ruffle=0.01, shape='long')
        side_fin(m, b, 0.55, 0.16, 0.04, angle=20, droop=-50, zsign=z, yk=-0.95, ruffle=0.01, shape='long')
        eye(m, b, 0.89, 0.3, 0.019, z)
    return m.build(), b


def material(part, uv, p, n):
    N = len(uv)
    if part == BODY:
        u, v, d = body_dirs(uv)
        hgt, rim = scales(u, v, 60, 36, edge=0.2)
        irid = 0.5 + 0.5 * np.sin(u * 18 + d * 4)
        base = mix(C('#c8c4e0'), C('#eef0ff'), irid)
        base = mix(base, C('#6a6aa0'), np.clip(d - 0.4, 0, 1) * 1.2)
        # crescent moons along the flank, waxing from tail to head
        cu = (u - 0.18) * 7; k = np.floor(cu); f = cu - k
        cx = (f - 0.5) * 0.14; cy = d * 0.08
        r = np.sqrt(cx ** 2 + cy ** 2); r2 = np.sqrt((cx - 0.02 * (1 - k / 5)) ** 2 + cy ** 2)
        cres = (r < 0.03) & (r2 > 0.026) & (k >= 0) & (k <= 4) & (np.abs(d) < 0.5)
        emit = np.zeros((N, 3)); emit[cres] = C('#9fd0ff') * 1.4
        base[cres] = C('#e8f4ff')
        return dict(albedo=base, spec=1.2, gloss=90, height=hgt * 0.5, emit=emit, sss=0.4)
    if part == FIN:
        ray, t = fin_rays(uv, 22)
        base = mix(C('#9a8ad0'), C('#e4dcff'), t)
        emit = np.outer(np.clip((t - 0.82) * 6, 0, 1), C('#8fc2ff')) * 0.9
        return dict(albedo=base * (0.85 + 0.15 * ray[:, None]), spec=0.8, gloss=60, alpha=0.62 - 0.25 * t + 0.15 * ray, sss=0.9, emit=emit)
    if part == EYE:
        return eye_mat(uv, C('#5aa0e0'), C('#dfefff'))
    return dict(albedo=np.tile([1, 0, 1.0], (N, 1)))
