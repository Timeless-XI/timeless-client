from fishlib import *


def spine(u):
    # a loose S-coil, mostly in the horizontal plane with a rising head
    x = -0.55 + 1.1 * u
    y = 0.13 * np.sin(u * 2 * np.pi * 1.15 + 0.3) * (0.35 + 0.65 * (1 - u)) + 0.06 * np.clip(u - 0.8, 0, 1) * 5
    z = 0.06 * np.sin(u * 2 * np.pi * 0.7)
    return np.stack([x, y, z], -1)


def build():
    m = Mesh()
    b = Body(1.4,
             ht=[(0, .006), (.1, .022), (.4, .038), (.8, .042), (.93, .036), (1, .012)],
             hb=[(0, .006), (.1, .02), (.4, .034), (.85, .036), (.95, .026), (1, .01)],
             wd=[(0, .005), (.1, .016), (.5, .03), (.85, .032), (.95, .024), (1, .009)], spine=spine, nu=220, nv=40)
    b.mesh(m)
    frill = lambda s: 0.022 + 0.045 * np.sin(np.pi * np.clip(s * 1.05, 0, 1)) ** 0.5
    fin_along(m, b, 0.02, 0.78, frill, 'top', rake=0.4, ruffle=0.006, ns=120, freq=26)
    fin_along(m, b, 0.02, 0.55, lambda s: frill(s) * 0.8, 'bottom', rake=0.4, ruffle=0.006, ns=90, freq=20)
    for z in (1, -1):
        side_fin(m, b, 0.88, 0.05, 0.035, angle=50, droop=-10, zsign=z, yk=-0.1)
        eye(m, b, 0.955, 0.35, 0.0085, z)
    return m.build(), b


def material(part, uv, p, n):
    N = len(uv)
    if part == BODY:
        u, v, d = body_dirs(uv)
        frost = fbm(u * 90, v * 10, 5)
        base = mix(C('#eef6fc'), C('#6f9cc8'), np.clip(d * 0.8 + 0.25, 0, 1) * 0.85)
        base = base * (0.85 + 0.25 * frost[:, None])
        line = np.abs(np.abs(d) - 0.05) < 0.03
        glint = line & (np.sin(u * 260) > 0.3)
        emit = np.zeros((N, 3)); emit[glint] = C('#7fe0ff') * 1.6
        mouth = (u > 0.975) & (np.abs(d + 0.2) < 0.15)
        base[mouth] *= 0.35
        return dict(albedo=base, spec=1.3, gloss=110, height=frost * 0.7, emit=emit, sss=0.6)
    if part == FIN:
        ray, t = fin_rays(uv, 70)
        base = mix(C('#bfe6fb'), C('#f4fcff'), t)
        ice = (hash2(np.floor(uv[:, 0] * 120), np.floor(uv[:, 1] * 8)) > 0.9) & (t > 0.6)
        emit = np.zeros((N, 3)); emit[ice] = C('#b8f0ff') * 0.8
        return dict(albedo=base, spec=1.0, gloss=80, alpha=0.35 + 0.3 * ray - 0.15 * t, sss=1.0, emit=emit)
    if part == EYE:
        return eye_mat(uv, C('#38b8e8'), C('#e0f8ff'))
    return dict(albedo=np.tile([1, 0, 1.0], (N, 1)))
