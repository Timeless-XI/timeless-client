"""Tiny numpy software renderer for the Elder fish mockups.

Meshes carry per-vertex position, normal, uv and a part id per triangle. Rendering writes a G-buffer
(depth, normal, world pos, uv, part) and shading is done per pixel afterwards ("deferred texturing"),
so the procedural textures are evaluated at full image resolution.
Two layers: opaque, and a single nearest translucent layer (fins, glass) composited on top."""
import numpy as np
from scipy.ndimage import gaussian_filter


# ----------------------------------------------------------------------------- mesh

# Export settings: LOD scales every mesh resolution; SEAM gives lofts a duplicated seam column (no wrap) so a
# texture can be baked in uv space without a wrap-around strip.
LOD = 1.0
SEAM = False


def _n(n, lo=3):
    return max(lo, int(round(n * LOD)))

class Mesh:
    def __init__(self):
        self.P = []; self.N = []; self.UV = []; self.T = []; self.part = []; self.layer = []

    def add(self, P, N, UV, T, part, layer=0):
        base = sum(len(p) for p in self.P)
        self.P.append(np.asarray(P, float)); self.N.append(np.asarray(N, float)); self.UV.append(np.asarray(UV, float))
        T = np.asarray(T, int) + base
        self.T.append(T); self.part.append(np.full(len(T), part)); self.layer.append(np.full(len(T), layer))
        return self

    def build(self):
        P = np.concatenate(self.P); N = np.concatenate(self.N); UV = np.concatenate(self.UV)
        T = np.concatenate(self.T); part = np.concatenate(self.part); layer = np.concatenate(self.layer)
        return P, N, UV, T, part, layer


def normalize(v):
    n = np.linalg.norm(v, axis=-1, keepdims=True)
    return v / np.maximum(n, 1e-9)


def grid_tris(nu, nv, wrap_v=False):
    T = []
    vv = nv if wrap_v else nv - 1
    for i in range(nu - 1):
        for j in range(vv):
            a = i * nv + j; b = i * nv + (j + 1) % nv; c = (i + 1) * nv + j; d = (i + 1) * nv + (j + 1) % nv
            T.append((a, c, b)); T.append((b, c, d))
    return np.array(T)


def grid_normals(P, nu, nv, wrap_v=False):
    G = P.reshape(nu, nv, 3)
    du = np.gradient(G, axis=0)
    if wrap_v:
        dv = (np.roll(G, -1, axis=1) - np.roll(G, 1, axis=1)) / 2
    else:
        dv = np.gradient(G, axis=1)
    return normalize(np.cross(du, dv)).reshape(-1, 3)


def loft(spine, frame_up, ht, hb, wd, nu=140, nv=56, shape=2.0, uv_scale=1.0):
    """Body of revolution-ish along a spine. spine(u)->(nu,3), frame_up(u)->(nu,3) unit up vectors,
    ht/hb/wd(u) half height top / bottom / half width. u in [0,1], 0 = tail, 1 = snout. v around 0..1."""
    nu, nv = _n(nu, 8), _n(nv, 8)
    u = np.linspace(0, 1, nu)
    S = spine(u); up = normalize(frame_up(u))
    tang = normalize(np.gradient(S, axis=0))
    side = normalize(np.cross(tang, up)); up = normalize(np.cross(side, tang))
    th = np.linspace(0, 2 * np.pi, nv, endpoint=SEAM)
    c, s = np.cos(th), np.sin(th)
    sc = np.sign(c) * np.abs(c) ** (2 / shape); ss = np.sign(s) * np.abs(s) ** (2 / shape)
    H = np.where(c[None, :] > 0, ht(u)[:, None], hb(u)[:, None])
    Wd = wd(u)[:, None]
    P = S[:, None, :] + up[:, None, :] * (H * sc[None, :])[..., None] + side[:, None, :] * (Wd * ss[None, :])[..., None]
    P = P.reshape(-1, 3)
    UV = np.stack(np.meshgrid(u, th / (2 * np.pi), indexing='ij'), -1).reshape(-1, 2)
    T = grid_tris(nu, nv, wrap_v=not SEAM)
    N = grid_normals(P, nu, nv, wrap_v=not SEAM)
    # make normals point outward
    centre = np.repeat(S, nv, axis=0)
    flip = np.sum(N * (P - centre), axis=1) < 0
    N[flip] *= -1
    return P, N, UV, T


def sheet(root, edge, ns=40, nt=16, ruffle=0.0, ruffle_freq=6, bulge=None):
    """Fin membrane between root curve root(s) and edge curve edge(s), s,t in [0,1]. uv = (s, t)."""
    ns, nt = _n(ns, 4), _n(nt, 3)
    s = np.linspace(0, 1, ns); t = np.linspace(0, 1, nt)
    R = root(s); E = edge(s)
    P = R[:, None, :] * (1 - t[None, :, None]) + E[:, None, :] * t[None, :, None]
    if ruffle:
        nrm = normalize(np.cross(np.gradient(R, axis=0) + 1e-9, E - R))
        P = P + nrm[:, None, :] * (ruffle * np.sin(s * ruffle_freq * np.pi)[:, None] * t[None, :] ** 1.2)[..., None]
    if bulge is not None:
        P = P + bulge(s[:, None], t[None, :])
    P = P.reshape(-1, 3)
    UV = np.stack(np.meshgrid(s, t, indexing='ij'), -1).reshape(-1, 2)
    T = grid_tris(ns, nt)
    N = grid_normals(P, ns, nt)
    return P, N, UV, T


def tube(curve, radius, n=40, nv=12):
    n, nv = _n(n, 3), _n(nv, 4)
    u = np.linspace(0, 1, n)
    C = curve(u)
    tang = normalize(np.gradient(C, axis=0))
    ref = np.where(np.abs(tang[:, 1:2]) > 0.9, np.array([[1, 0, 0]]), np.array([[0, 1, 0]]))
    a = normalize(np.cross(tang, ref)); b = normalize(np.cross(tang, a))
    th = np.linspace(0, 2 * np.pi, nv, endpoint=False)
    r = radius(u) if callable(radius) else np.full(n, radius)
    off = a[:, None, :] * np.cos(th)[None, :, None] + b[:, None, :] * np.sin(th)[None, :, None]
    P = (C[:, None, :] + off * r[:, None, None]).reshape(-1, 3)
    N = off.reshape(-1, 3)
    UV = np.stack(np.meshgrid(u, th / (2 * np.pi), indexing='ij'), -1).reshape(-1, 2)
    return P, N, UV, grid_tris(n, nv, wrap_v=True)


def sphere(c, r, n=24, scale=(1, 1, 1)):
    n = _n(n, 6)
    th = np.linspace(0, np.pi, n); ph = np.linspace(0, 2 * np.pi, n * 2, endpoint=False)
    T_, F_ = np.meshgrid(th, ph, indexing='ij')
    N = np.stack([np.sin(T_) * np.cos(F_), np.cos(T_), np.sin(T_) * np.sin(F_)], -1).reshape(-1, 3)
    P = np.asarray(c) + N * r * np.asarray(scale)
    N = normalize(N / np.asarray(scale))
    UV = np.stack([T_ / np.pi, F_ / (2 * np.pi)], -1).reshape(-1, 2)
    return P, N, UV, grid_tris(n, n * 2, wrap_v=True)


def flat_poly(pts, nrm):
    """Convex-ish polygon fan (for blades, crystal faces)."""
    pts = np.asarray(pts, float); c = pts.mean(0)
    P = np.vstack([c, pts]); n = len(pts)
    T = np.array([(0, 1 + i, 1 + (i + 1) % n) for i in range(n)])
    N = np.tile(normalize(np.asarray(nrm, float)), (len(P), 1))
    d = P - c; UV = np.stack([d[:, 0], d[:, 1]], -1)
    return P, N, UV, T


def prism(base_c, axis, r, h, sides=6, taper=0.15):
    """Crystal: hexagonal prism with a pointed cap."""
    axis = normalize(np.asarray(axis, float)); ref = np.array([1.0, 0, 0]) if abs(axis[0]) < 0.9 else np.array([0, 0, 1.0])
    a = normalize(np.cross(axis, ref)); b = np.cross(axis, a)
    th = np.linspace(0, 2 * np.pi, sides, endpoint=False)
    ring = lambda rr, hh: np.asarray(base_c) + axis * hh + (np.cos(th)[:, None] * a + np.sin(th)[:, None] * b) * rr
    r0 = ring(r, 0); r1 = ring(r * (1 - taper), h * 0.72); tip = np.asarray(base_c) + axis * h
    P = []; N = []; T = []; UV = []
    for i in range(sides):
        j = (i + 1) % sides
        quad = [r0[i], r0[j], r1[j], r1[i]]
        nrm = normalize(np.cross(r0[j] - r0[i], r1[i] - r0[i]))
        k = len(P); P += quad; N += [nrm] * 4; UV += [(0, 0), (1, 0), (1, 1), (0, 1)]
        T += [(k, k + 1, k + 2), (k, k + 2, k + 3)]
        tri = [r1[i], r1[j], tip]; nrm2 = normalize(np.cross(r1[j] - r1[i], tip - r1[i]))
        k = len(P); P += tri; N += [nrm2] * 3; UV += [(0, 1), (1, 1), (0.5, 1.4)]; T += [(k, k + 1, k + 2)]
    return np.array(P), np.array(N), np.array(UV, float), np.array(T)


# ----------------------------------------------------------------------------- camera / raster

def look_at(eye, target, up=(0, 1, 0)):
    eye = np.asarray(eye, float); f = normalize(np.asarray(target, float) - eye)
    r = normalize(np.cross(f, up)); u = np.cross(r, f)
    return eye, np.stack([r, u, -f])


def raster(P, N, UV, T, part, W, H, eye, R, fov=28.0, ortho=None, zbuf_limit=None):
    """Returns G-buffer dict. Perspective-correct attributes."""
    V = (P - eye) @ R.T  # camera space, looking down -z
    z = -V[:, 2]
    if ortho:
        sx = V[:, 0] / ortho * (W / 2) + W / 2; sy = -V[:, 1] / ortho * (W / 2) + H / 2
        iw = np.ones_like(z)
    else:
        f = 1 / np.tan(np.radians(fov) / 2)
        sx = V[:, 0] / z * f * (H / 2) + W / 2; sy = -V[:, 1] / z * f * (H / 2) + H / 2
        iw = 1 / z
    depth = np.full((H, W), np.inf); g_n = np.zeros((H, W, 3)); g_p = np.zeros((H, W, 3))
    g_uv = np.zeros((H, W, 2)); g_part = np.full((H, W), -1, int)
    attrs = np.concatenate([N, P, UV], 1)
    A = attrs * iw[:, None]
    for k in range(len(T)):
        i0, i1, i2 = T[k]
        x0, y0, x1, y1, x2, y2 = sx[i0], sy[i0], sx[i1], sy[i1], sx[i2], sy[i2]
        if min(z[i0], z[i1], z[i2]) <= 0.01:
            continue
        xmin = max(int(np.floor(min(x0, x1, x2))), 0); xmax = min(int(np.ceil(max(x0, x1, x2))), W - 1)
        ymin = max(int(np.floor(min(y0, y1, y2))), 0); ymax = min(int(np.ceil(max(y0, y1, y2))), H - 1)
        if xmin > xmax or ymin > ymax:
            continue
        den = (y1 - y2) * (x0 - x2) + (x2 - x1) * (y0 - y2)
        if abs(den) < 1e-12:
            continue
        xs = np.arange(xmin, xmax + 1) + 0.5; ys = np.arange(ymin, ymax + 1) + 0.5
        X, Y = np.meshgrid(xs, ys)
        b0 = ((y1 - y2) * (X - x2) + (x2 - x1) * (Y - y2)) / den
        b1 = ((y2 - y0) * (X - x2) + (x0 - x2) * (Y - y2)) / den
        b2 = 1 - b0 - b1
        m = (b0 >= -1e-6) & (b1 >= -1e-6) & (b2 >= -1e-6)
        if not m.any():
            continue
        w = b0 * iw[i0] + b1 * iw[i1] + b2 * iw[i2]
        zz = 1 / w if not ortho else (b0 * z[i0] + b1 * z[i1] + b2 * z[i2])
        sub = depth[ymin:ymax + 1, xmin:xmax + 1]
        m &= zz < sub
        if zbuf_limit is not None:
            m &= zz < zbuf_limit[ymin:ymax + 1, xmin:xmax + 1] - 1e-4
        if not m.any():
            continue
        att = (b0[..., None] * A[i0] + b1[..., None] * A[i1] + b2[..., None] * A[i2]) / w[..., None]
        sub[m] = zz[m]
        yy, xx = np.nonzero(m)
        yy += ymin; xx += xmin
        g_n[yy, xx] = att[m][:, 0:3]; g_p[yy, xx] = att[m][:, 3:6]; g_uv[yy, xx] = att[m][:, 6:8]
        g_part[yy, xx] = part[k]
    g_n = normalize(g_n)
    return dict(depth=depth, n=g_n, p=g_p, uv=g_uv, part=g_part)


# ----------------------------------------------------------------------------- noise helpers

def hash2(x, y):
    h = np.sin(x * 127.1 + y * 311.7) * 43758.5453
    return h - np.floor(h)


def vnoise(x, y):
    xi, yi = np.floor(x), np.floor(y); xf, yf = x - xi, y - yi
    u = xf * xf * (3 - 2 * xf); v = yf * yf * (3 - 2 * yf)
    a = hash2(xi, yi); b = hash2(xi + 1, yi); c = hash2(xi, yi + 1); d = hash2(xi + 1, yi + 1)
    return a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v


def fbm(x, y, oct=4):
    s = 0; a = 0.5; f = 1.0
    for _ in range(oct):
        s += a * vnoise(x * f, y * f); f *= 2.02; a *= 0.5
    return s


def scales(u, v, nu, nv, edge=0.18):
    """Overlapping scale rows: returns (height 0..1, edge mask) for a scale pattern in uv space."""
    x = u * nu; y = v * nv + (np.floor(x) % 2) * 0.5
    fx = x - np.floor(x); fy = y - np.floor(y)
    # each scale: circle-ish lobe opening toward the tail (u decreasing)
    d = np.sqrt(((fx - 0.15) * 1.0) ** 2 + ((fy - 0.5) * 1.15) ** 2)
    hgt = np.clip(1 - d / 0.95, 0, 1)
    rim = np.clip(1 - np.abs(d - 0.82) / edge, 0, 1)
    return hgt, rim


def mix(a, b, t):
    t = np.asarray(t, float)
    if t.ndim >= 1:
        t = t[..., None]
    return a * (1 - t) + b * t


def C(hexs):
    hexs = hexs.lstrip('#')
    return np.array([int(hexs[i:i + 2], 16) / 255 for i in (0, 2, 4)])


# ----------------------------------------------------------------------------- shading

def shade(G, material, eye, lights=None, env_up=(0.30, 0.36, 0.48), env_down=(0.16, 0.12, 0.10), bump_strength=1.0):
    """material(part, uv, p, n) -> dict(albedo HxWx3, gloss, spec, emit HxWx3, alpha, height)"""
    H, W = G['depth'].shape
    mask = G['part'] >= 0
    alb = np.zeros((H, W, 3)); emit = np.zeros((H, W, 3)); spec = np.zeros((H, W)); gloss = np.full((H, W), 20.0)
    alpha = np.ones((H, W)); hgt = np.zeros((H, W)); sss = np.zeros((H, W))
    for part in np.unique(G['part'][mask]):
        m = G['part'] == part
        r = material(int(part), G['uv'][m], G['p'][m], G['n'][m])
        alb[m] = r['albedo']; spec[m] = r.get('spec', 0.3); gloss[m] = r.get('gloss', 24)
        if 'emit' in r: emit[m] = r['emit']
        if 'alpha' in r: alpha[m] = r['alpha']
        if 'height' in r: hgt[m] = r['height']
        if 'sss' in r: sss[m] = r['sss']
    n = G['n'].copy()
    # bump from the height map (screen-space gradient), only inside the same part
    if bump_strength:
        hs = gaussian_filter(hgt, 0.6)
        gy, gx = np.gradient(hs)
        same = mask
        # camera right / up approximated from the view matrix rows
        n = normalize(n - bump_strength * (gx[..., None] * CAM['r'] - gy[..., None] * CAM['u']) * same[..., None])
    V = normalize(eye - G['p'])
    if lights is None:
        lights = [((-0.55, 0.75, 0.55), (1.05, 0.95, 0.82)), ((0.7, 0.1, 0.6), (0.25, 0.32, 0.45)), ((0.2, 0.3, -1.0), (0.55, 0.65, 0.85))]
    col = np.zeros((H, W, 3))
    ndl_up = n[..., 1:2] * 0.5 + 0.5
    amb = np.asarray(env_down) * (1 - ndl_up) + np.asarray(env_up) * ndl_up
    col += alb * amb
    for d, c in lights:
        L = normalize(np.asarray(d, float))
        ndl = np.clip(np.sum(n * L, -1), 0, 1)
        wrap = np.clip((np.sum(n * L, -1) + 0.35) / 1.35, 0, 1)
        diff = ndl * (1 - sss) + wrap * sss
        Hh = normalize(L + V)
        sp = np.clip(np.sum(n * Hh, -1), 0, 1) ** gloss * spec
        col += alb * diff[..., None] * np.asarray(c) + sp[..., None] * np.asarray(c)
    fres = (1 - np.clip(np.sum(n * V, -1), 0, 1)) ** 3
    col += fres[..., None] * np.array([0.25, 0.32, 0.45]) * 0.6
    col += emit
    return col, alpha, mask, emit


CAM = {'r': np.array([1.0, 0, 0]), 'u': np.array([0, 1.0, 0])}


def render(mesh_parts, material, W=900, H=600, eye=(0.2, 0.15, 2.4), target=(0, 0, 0), fov=26, ortho=None,
           bg=None, bloom=0.0, ss=1, lights=None, env_up=(0.30, 0.36, 0.48), env_down=(0.16, 0.12, 0.10)):
    """Returns RGBA float image. Supersample by ss."""
    P, N, UV, T, part, layer = mesh_parts
    W2, H2 = W * ss, H * ss
    e, R = look_at(eye, target)
    CAM['r'] = R[0]; CAM['u'] = R[1]
    op = layer == 0
    G0 = raster(P, N, UV, T[op], part[op], W2, H2, e, R, fov, ortho)
    col0, a0, m0, em0 = shade(G0, material, e, lights, env_up, env_down)
    out = np.zeros((H2, W2, 4))
    if bg is not None:
        out[..., :3] = bg(H2, W2); out[..., 3] = 1
    out[m0, :3] = col0[m0]; out[m0, 3] = 1
    emit_total = np.where(m0[..., None], em0, 0)
    if (~op).any():
        G1 = raster(P, N, UV, T[~op], part[~op], W2, H2, e, R, fov, ortho, zbuf_limit=G0['depth'])
        col1, a1, m1, em1 = shade(G1, material, e, lights, env_up, env_down)
        a1 = np.where(m1, a1, 0)
        out[..., :3] = out[..., :3] * (1 - a1[..., None]) + col1 * a1[..., None]
        out[..., 3] = out[..., 3] * (1 - a1) + a1
        emit_total = emit_total * (1 - a1[..., None]) + em1 * a1[..., None]
    if bloom:
        glow = np.stack([gaussian_filter(emit_total[..., c], 6 * ss) + gaussian_filter(emit_total[..., c], 18 * ss) for c in range(3)], -1)
        out[..., :3] += glow * bloom
        out[..., 3] = np.maximum(out[..., 3], np.clip(glow.max(-1) * bloom, 0, 1))
    if ss > 1:
        out = out.reshape(H, ss, W, ss, 4)
        a = out[..., 3:4]
        rgb = (out[..., :3] * a).mean((1, 3)); aa = a.mean((1, 3))
        out = np.concatenate([rgb / np.maximum(aa, 1e-6), aa], -1)
    return out


def tonemap(img):
    rgb = img[..., :3]
    rgb = 1 - np.exp(-rgb * 1.15)
    rgb = np.clip(rgb, 0, 1) ** (1 / 1.15)
    return np.concatenate([rgb, img[..., 3:4]], -1)


def save(img, path):
    from PIL import Image
    t = tonemap(img)
    Image.fromarray((np.clip(t, 0, 1) * 255 + 0.5).astype(np.uint8), 'RGBA').save(path)


def cellnoise(x, y, jitter=0.8):
    """F1 distance to scattered feature points (round spots)."""
    xi, yi = np.floor(x), np.floor(y)
    best = np.full(np.shape(x), 9.0)
    for dx in (-1, 0, 1):
        for dy in (-1, 0, 1):
            cx, cy = xi + dx, yi + dy
            px = cx + 0.5 + (hash2(cx, cy) - 0.5) * jitter
            py = cy + 0.5 + (hash2(cx + 17.3, cy - 9.1) - 0.5) * jitter
            best = np.minimum(best, np.hypot(x - px, y - py))
    return best
