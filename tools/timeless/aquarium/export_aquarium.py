"""Export the Timeless Aquarium and the eight Elder fish as game-ready meshes + baked textures for the Timeless
client, in one bundle (assets/timeless/aquarium/aquarium.bin).

Furniture space (what the client places): origin = centre of the 3 x 2 footprint on the floor, -y up, metres-ish
world units (a furniture cell is 0.6). Tank space (tank.py) is turned 180 degrees about x and scaled to fit.

Bundle: b"TLAQ", u32 version 1, u32 entries; entry = u8 kind (0 mesh, 1 png, 2 raw), u16 name length, name,
u32 length, bytes.
Mesh: u32 models; model = u16 texture-name length, texture name, u16 blending, u8 variant, u32 vertices, u32 indices,
vertices (pos f32x3, normal f32x3, colour u32 BGRA (0x80 = 1.0), uv f32x2), indices u16 (triangle list).
Variant: 0 always, 1 tank lit (1+ fish), 2 tank dark (no fish), 3 lamp off, 4 lamp dim (1-7), 5 lamp full (8).
"""
import io, struct, sys, importlib
import numpy as np
from PIL import Image
from scipy.ndimage import distance_transform_edt
import r3d
from r3d import Mesh, normalize, C, mix

OUT = sys.argv[1] if len(sys.argv) > 1 else '/home/claude/elder/aquarium.bin'
FISH = ['barbel', 'oreeater', 'whiskerwise', 'admiral', 'moonfin', 'rimecoil', 'herald', 'siren']  # bit order
K = 1.8 / 3.0          # tank width with its widest bars (3.0) -> 3 cells
FLOOR = -0.68          # stand bottom in tank space


# ----------------------------------------------------------------------------- baking

def bake(P, N, UV, T, material, part, W, H):
    """Rasterise the triangles in uv space; evaluate the material per texel with the interpolated position/normal.
    Returns (rgba uint8 HxWx4, normalised uv per vertex)."""
    lo = UV.min(0); hi = UV.max(0); span = np.maximum(hi - lo, 1e-6)
    tuv = (UV - lo) / span
    px = tuv[:, 0] * (W - 1); py = tuv[:, 1] * (H - 1)
    gp = np.zeros((H, W, 3)); gn = np.zeros((H, W, 3)); guv = np.zeros((H, W, 2)); cov = np.zeros((H, W), bool)
    for a, b, c in T:
        xs = (px[a], px[b], px[c]); ys = (py[a], py[b], py[c])
        x0, x1 = int(np.floor(min(xs))), int(np.ceil(max(xs))); y0, y1 = int(np.floor(min(ys))), int(np.ceil(max(ys)))
        x0, y0 = max(x0, 0), max(y0, 0); x1, y1 = min(x1, W - 1), min(y1, H - 1)
        if x0 > x1 or y0 > y1:
            continue
        den = (ys[1] - ys[2]) * (xs[0] - xs[2]) + (xs[2] - xs[1]) * (ys[0] - ys[2])
        if abs(den) < 1e-9:
            continue
        X, Y = np.meshgrid(np.arange(x0, x1 + 1), np.arange(y0, y1 + 1))
        w0 = ((ys[1] - ys[2]) * (X - xs[2]) + (xs[2] - xs[1]) * (Y - ys[2])) / den
        w1 = ((ys[2] - ys[0]) * (X - xs[2]) + (xs[0] - xs[2]) * (Y - ys[2])) / den
        w2 = 1 - w0 - w1
        m = (w0 >= -0.02) & (w1 >= -0.02) & (w2 >= -0.02)
        if not m.any():
            continue
        yy, xx = Y[m], X[m]
        ww = np.stack([w0[m], w1[m], w2[m]], 1)
        gp[yy, xx] = ww @ P[[a, b, c]]; gn[yy, xx] = ww @ N[[a, b, c]]; guv[yy, xx] = ww @ UV[[a, b, c]]
        cov[yy, xx] = True
    if not cov.any():
        return np.zeros((H, W, 4), np.uint8) + 128, tuv
    # fill empty texels from the nearest covered one (no dark seams when sampling at edges)
    idx = distance_transform_edt(~cov, return_distances=False, return_indices=True)
    gp, gn, guv = gp[idx[0], idx[1]], gn[idx[0], idx[1]], guv[idx[0], idx[1]]
    r3d.CAM['r'] = np.array([1.0, 0, 0]); r3d.CAM['u'] = np.array([0, 1.0, 0])
    r = material(part, guv.reshape(-1, 2), gp.reshape(-1, 3), normalize(gn.reshape(-1, 3)))
    n = H * W
    alb = np.asarray(r['albedo'], float).reshape(n, 3)
    emit = np.asarray(r.get('emit', np.zeros((n, 3))), float).reshape(n, 3)
    alpha = np.broadcast_to(np.asarray(r.get('alpha', 1.0), float), (n,))
    # a little of the bump as baked shading, so scales and plates still read without the per-pixel bump
    hgt = np.broadcast_to(np.asarray(r.get('height', 0.0), float), (n,)).reshape(H, W)
    gy, gx = np.gradient(hgt)
    shade = np.clip(1.0 - 0.9 * gy + 0.45 * gx, 0.75, 1.15).reshape(n, 1)
    rgb = np.clip(alb * shade + emit * 0.8, 0, 1)
    rgba = np.concatenate([rgb, np.clip(alpha, 0, 1)[:, None]], 1).reshape(H, W, 4)
    return (rgba * 255 + 0.5).astype(np.uint8), tuv


def png(rgba):
    b = io.BytesIO(); Image.fromarray(rgba, 'RGBA').save(b, 'PNG', optimize=True); return b.getvalue()


# ----------------------------------------------------------------------------- lighting baked into vertex colours

KEY = normalize(np.array([-0.3, 1.0, 0.45])); FILL = normalize(np.array([0.8, 0.2, 0.6]))
WATER = C('#2a7a96')


def vcolour(p, n, inside, emissive, two_sided):
    d1 = n @ KEY; d2 = n @ FILL
    if two_sided:
        d1, d2 = np.abs(d1), np.abs(d2)
    light = 0.5 + 0.55 * np.clip(d1, 0, 1) + 0.2 * np.clip(d2, 0, 1) + 0.12 * (n[:, 1] * 0.5 + 0.5)
    col = np.repeat(light[:, None], 3, 1) * np.array([1.0, 0.98, 0.94])
    if inside:
        depth = np.clip(0.45 - p[:, 2], 0, 1) + np.clip(1.1 - p[:, 1], 0, 2) * 0.3
        f = np.clip(1 - np.exp(-depth * 0.9), 0, 0.6)[:, None]
        col = col * (1 - f) + (col * WATER * 1.6) * f
    if emissive:
        col = np.full_like(col, 1.0)
    col = np.clip(col, 0, 1.99) * 128
    bgra = (np.clip(col[:, 2], 0, 255).astype(np.uint32) | (np.clip(col[:, 1], 0, 255).astype(np.uint32) << 8)
            | (np.clip(col[:, 0], 0, 255).astype(np.uint32) << 16) | (np.uint32(0x80) << 24))
    return bgra


def to_furniture(P, N):
    P2 = np.stack([P[:, 0] * K, -(P[:, 1] - FLOOR) * K, -P[:, 2] * K], 1)
    N2 = np.stack([N[:, 0], -N[:, 1], -N[:, 2]], 1)
    return P2.astype(np.float32), N2.astype(np.float32)


def model_bytes(tex, blending, variant, P, N, colours, uv, T):
    name = tex.encode()
    out = struct.pack('<H', len(name)) + name + struct.pack('<HBII', blending, variant, len(P), T.size)
    v = np.zeros(len(P), dtype=[('p', '<f4', 3), ('n', '<f4', 3), ('c', '<u4'), ('uv', '<f4', 2)])
    v['p'] = P; v['n'] = N; v['c'] = colours; v['uv'] = uv
    return out + v.tobytes() + T.astype('<u2').tobytes()


def split(P, N, UV, T, part, layer):
    """Per part: compact vertex arrays and local triangle indices."""
    for pid in np.unique(part):
        tm = part == pid
        tris = T[tm]
        used = np.unique(tris)
        remap = -np.ones(len(P), int); remap[used] = np.arange(len(used))
        yield int(pid), int(layer[tm][0]), P[used], N[used], UV[used], remap[tris]


# ----------------------------------------------------------------------------- export

TEX = {}  # name -> png bytes


def export_fish(bit, name):
    import tank
    mod = importlib.import_module('fish_' + name)
    (P, N, UV, T, part, layer), _ = mod.build()
    s, yaw, pitch, t = tank.PLACE[name]
    Rm = tank.roty(yaw) @ tank.rotz(pitch)
    lo, hi = P.min(0), P.max(0); c = (lo + hi) / 2
    models = []; tris = 0
    for pid, lay, p, n, uv, tri in split(P, N, UV, T, part, layer):
        big = pid in (0,)  # body
        W, H = (256, 128) if big else (128, 64) if lay == 1 else (64, 64)
        rgba, tuv = bake(p, n, uv, tri, mod.material, pid, W, H)
        tex = f'tlaq{bit}_{pid}'
        TEX[tex] = png(rgba)
        pt = (p - c) @ Rm.T * s + np.asarray(t); nt = n @ Rm.T
        emissive = pid in (6,)  # GLOW
        col = vcolour(pt, nt, True, emissive, lay == 1)
        fp, fn = to_furniture(pt, nt)
        blend = 0x2000 | (0x8000 if (rgba[..., 3] < 250).any() else 0)
        models.append(model_bytes(tex, blend, 0, fp, fn, col, tuv.astype(np.float32), tri))
        tris += len(tri)
    print(f'fish{bit} {name}: {len(models)} models, {tris} triangles', flush=True)
    return struct.pack('<I', len(models)) + b''.join(models)


def export_tank():
    import tank
    m = Mesh(); tank.build_tank(m)
    P, N, UV, T, part, layer = m.build()
    models = []; tris = 0
    variants = {tank.SAND: [(1, 1.0), (2, 0.0)], tank.BACK: [(1, 1.0), (2, 0.0)], tank.LAMP: [(3, 0.0), (4, 0.3), (5, 1.0)]}
    for pid, lay, p, n, uv, tri in split(P, N, UV, T, part, layer):
        for variant, level in variants.get(pid, [(0, 1.0)]):
            if pid == tank.LAMP:
                tank.STATE.update(crystal=level, lit=1.0)
            else:
                tank.STATE.update(lit=level, crystal=1.0)
            W, H = (256, 128) if pid in (tank.SAND, tank.BACK) else (128, 128) if pid in (tank.SILVER, tank.STAND) else (64, 64)
            rgba, tuv = bake(p, n, uv, tri, tank.tank_material, pid, W, H)
            tex = f'tlaqT_{pid - 1000}_{variant}'
            TEX[tex] = png(rgba)
            inside = pid not in (tank.SILVER, tank.STAND, tank.LAMP)
            col = vcolour(p, n, inside, pid == tank.LAMP and level > 0.5, lay == 1 or pid == tank.WEED)
            if pid == tank.LAMP and level <= 0.5:
                col = vcolour(p, n, False, False, False)
            fp, fn = to_furniture(p, n)
            blend = 0x2000 | (0x8000 if (rgba[..., 3] < 250).any() else 0)
            models.append(model_bytes(tex, blend, variant, fp, fn, col, tuv.astype(np.float32), tri))
            tris += len(tri)
    # No front glass pane: a blended pane in front of the blended fins and weed could hide them, depending on draw
    # order; the frame and the water tint read as a tank without it.
    print(f'tank: {len(models)} models, {tris} triangles', flush=True)
    return struct.pack('<I', len(models)) + b''.join(models)


def icon_raw(path):
    """The item DAT icon layout dat/items.rs reads: 0x11 header bytes, BITMAPINFOHEADER (8 bpp, 32 x 32,
    bottom-up), 256-colour BGRA palette with alpha 0..0x80, pixels."""
    im = Image.open(path).convert('RGBA')
    q = im.quantize(colors=255, method=Image.Quantize.FASTOCTREE)
    pal = q.getpalette()[:255 * 3]
    rgba = np.asarray(im); idx = np.asarray(q).copy()
    # transparent texels -> index 255 (alpha 0)
    idx[rgba[..., 3] < 40] = 255
    palette = bytearray(1024)
    for i in range(255):
        r, g, b = pal[3 * i:3 * i + 3]
        # mean alpha of the texels using this index, scaled to the game's 0..0x80
        a = rgba[..., 3][idx == i]
        al = int(round((a.mean() if a.size else 255) / 255 * 0x80))
        palette[4 * i:4 * i + 4] = bytes((b, g, r, al))
    palette[1020:1024] = bytes((0, 0, 0, 0))
    header = bytearray(0x11); header[0] = 0x91; header[1:11] = b'tlaquarium'  # flag byte + 16-byte name
    bih = struct.pack('<IiiHHIIiiII', 40, 32, 32, 1, 8, 0, 1024, 0, 0, 256, 256)
    pixels = np.flipud(idx).astype(np.uint8).tobytes()
    return bytes(header) + bih + bytes(palette) + pixels


def bundle():
    entries = []
    r3d.SEAM = True
    r3d.LOD = 0.5
    for bit, name in enumerate(FISH):
        entries.append((0, f'fish{bit}', export_fish(bit, name)))
    r3d.LOD = 0.7
    entries.append((0, 'tank', export_tank()))
    for k, v in TEX.items():
        entries.append((1, k, v))
    entries.append((2, 'icon32', icon_raw('/home/claude/elder/out/aquarium_icon32.png')))
    out = b'TLAQ' + struct.pack('<II', 1, len(entries))
    for kind, name, data in entries:
        nb = name.encode()
        out += struct.pack('<BH', kind, len(nb)) + nb + struct.pack('<I', len(data)) + data
    open(OUT, 'wb').write(out)
    print(OUT, len(out) // 1024, 'KB,', len(TEX), 'textures')


if __name__ == '__main__':
    bundle()
