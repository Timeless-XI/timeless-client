"""Export the Timeless Aquarium v3 (tank3.py, swim.py) for the Timeless client: bundle version 3.

Container as v1/v2: b"TLAQ", u32 version, u32 entries; entry = u8 kind (0 mesh, 1 png, 2 raw), u16 name length,
name, u32 length, bytes.
Mesh (version 3) = one GROUP drawn as its own piece: u8 anim kind, u16 parameter count n, f32 x n, u32 models,
models as v1/v2. All in furniture space (-y up; origin = the middle of the 6 x 4 footprint on the floor).
  kind 0 none; 1 spin (pivot xyz, axis xyz, period s); 3 fall (direction xyz, spacing, period s);
  4 swim (the body), 5 swim (the tail fin): lap s, start (0-1), sway rad, beat Hz, joint x, wag rad, wag lag rad,
  samples n, then n x (x, y, z, yaw, pitch, roll) at even time steps of one lap (swim.py).
  6 slither (one segment of the eel): kind 4's parameters, then dt (s behind on the path), wave amplitude, wave
  phase, wave slope.
Groups: tank, crown, hgcrystal, grains, fish0..fish7 and their tails fish0t..fish7t; Rimecoil (bit 5) is
fish5s0..fish5s11 instead, its segments from tail to head.
Variants: 0 always; 1 lit (1+ fish) / 2 dark (none); 3 crown off (none) / 4 dim (1-7) / 5 full (all 8).
"""
import struct, sys
import numpy as np
import r3d
import export_aquarium as v1
from export_aquarium import bake, png, model_bytes, split, icon_raw
from r3d import normalize, C
import tank3 as T
import swim

OUT = sys.argv[1] if len(sys.argv) > 1 else '/home/claude/elder/aquarium3.bin'
TEX = {}
LIT_PARTS = {T.SKY, T.SKYSIDE, T.SAND, T.KELP, T.HGSAND, T.GRAIN, T.HGCRYSTAL, T.EMBLEM, T.HGGLASS, T.LAMP}
CROWN_PARTS = {T.CROWN, T.FINIAL}
FLAT = {T.SKY, T.SKYSIDE, T.LAMP}
GLOW = {T.CROWN, T.HGCRYSTAL, T.GRAIN, T.EMBLEM, T.FINIAL, T.LAMP}
INSIDE = {T.SAND, T.RUIN, T.KELP, T.HGGLASS, T.HGSAND, T.GRAIN, T.HGCRYSTAL, T.HGMETAL}
SIZES = {T.SKY: (2048, 1024), T.SKYSIDE: (256, 128), T.SAND: (512, 256), T.OBSID: (128, 128), T.HAIR: (128, 64),
         T.HGMETAL: (64, 64), T.GILT: (128, 128), T.PLAQUE: (1024, 36), T.COLUMN: (128, 64), T.LAMP: (16, 16),
         T.RUIN: (128, 128), T.KELP: (128, 64)}
K = None
FLOOR = T.FLOOR
WATER = C('#2a4a80')


def fit_scale():
    """K so the whole piece fits 6 x 4 cells (3.6 x 2.4), with a hair to spare."""
    P = T.group_tank().build()[0]
    w = P[:, 0].max() - P[:, 0].min(); d = P[:, 2].max() - P[:, 2].min()
    k = min(3.59 / w, 2.39 / d)
    return k, P


def to_f(P, N, floor=True):
    off = FLOOR if floor else 0.0
    P2 = np.stack([P[:, 0] * K, -(P[:, 1] - off) * K, -P[:, 2] * K], 1)
    N2 = np.stack([N[:, 0], -N[:, 1], -N[:, 2]], 1)
    return P2.astype(np.float32), N2.astype(np.float32)


def vcol(p, n, part, lit, two_sided, fish=False):
    if part in FLAT:
        return np.full(len(p), 0x80808080, np.uint32)
    if part in GLOW and lit > 0.5:
        return np.full(len(p), 0x80FFFFFF, np.uint32)
    up = n[:, 1]; key = n @ normalize(np.array([-0.3, 1.0, 0.45])); fill = n @ normalize(np.array([0.8, 0.2, 0.6]))
    if two_sided:
        up, key, fill = np.abs(up), np.abs(key), np.abs(fill)
    if fish:   # lit from above only, so it looks right whichever way the fish turns
        light = 0.62 + 0.45 * np.clip(up, 0, 1) + 0.1 * (up * 0.5 + 0.5)
    else:
        light = 0.5 + 0.55 * np.clip(key, 0, 1) + 0.2 * np.clip(fill, 0, 1) + 0.12 * (up * 0.5 + 0.5)
    col = np.repeat(light[:, None], 3, 1) * np.array([1.0, 0.98, 0.94])
    if part in INSIDE or fish:
        if fish:
            depth = np.full(len(p), 0.55)
        else:
            depth = np.clip(T.Z1 - 0.3 - p[:, 2], 0, 2) * 0.5 + np.clip(T.Y1 - p[:, 1], 0, 2) * 0.25
        f = np.clip(1 - np.exp(-depth * 0.9), 0, 0.55)[:, None]
        col = col * (1 - f) + (col * WATER * 1.6) * f
    col = np.clip(col, 0, 1.99) * 128
    return (np.clip(col[:, 2], 0, 255).astype(np.uint32) | (np.clip(col[:, 1], 0, 255).astype(np.uint32) << 8)
            | (np.clip(col[:, 0], 0, 255).astype(np.uint32) << 16) | (np.uint32(0x80) << 24))


def header(kind, prm):
    return struct.pack('<BH', kind, len(prm)) + struct.pack(f'<{len(prm)}f', *[float(x) for x in prm])


def anim_header(anim):
    if anim[0] == 'spin':
        c, ax, period = anim[1], anim[2], anim[3]
        pc, _ = to_f(np.asarray([c], float), np.zeros((1, 3)))
        a = np.asarray(ax, float); a = np.array([a[0], -a[1], -a[2]])
        return header(1, [*pc[0], *normalize(a), period])
    if anim[0] == 'fall':
        d, sp, period = anim[1:]
        d = np.asarray(d, float); d = np.array([d[0], -d[1], -d[2]])
        return header(3, [*d, sp * K, period])
    return header(0, [])


def tank_group(gname, mesh, anim):
    P, N, UV, Tr, part, layer = mesh
    models = []; tris = 0
    for pid, lay, p, n, uv, tri in split(P, N, UV, Tr, part, layer):
        if pid in CROWN_PARTS:
            variants = [(3, 0.0), (4, 0.3), (5, 1.0)]
        elif pid in LIT_PARTS:
            variants = [(1, 1.0), (2, 0.0)]
        else:
            variants = [(0, 1.0)]
        for variant, level in variants:
            if pid in CROWN_PARTS:
                T.STATE.update(lit=1.0, crystal=level)
            else:
                T.STATE.update(lit=level, crystal=1.0)
            W, H = SIZES.get(pid, (64, 64))
            rgba, tuv = bake(p, n, uv, tri, T.material, pid, W, H)
            tex = f'tlq{gname[:4]}{pid - 1000}_{variant}'
            TEX[tex] = png(rgba)
            glow_level = level
            col = vcol(p, n, pid, glow_level, lay == 1)
            fp, fn = to_f(p, n)
            blend = 0x2000 | (0x8000 if (rgba[..., 3] < 250).any() else 0)
            models.append(model_bytes(tex, blend, variant, fp, fn, col, tuv.astype(np.float32), tri))
            tris += len(tri)
    print(f'{gname}: {len(models)} models, {tris} triangles', flush=True)
    return anim_header(anim) + struct.pack('<I', len(models)) + b''.join(models)


def eel_entries(bit, name):
    segs, mat, xh, length = T.eel_segments(name)
    spec = swim.PATHS[name]
    smp = swim.samples(spec, K, FLOOR)
    out = []; prms = []
    for k, (mesh, xc) in enumerate(segs):
        prm = swim.eel_params(spec, smp, xc, xh, length, K)
        P, N, UV, Tr, part, layer = mesh
        models = []; tris = 0
        for pid, lay, p, n, uv, tri in split(P, N, UV, Tr, part, layer):
            W, H = (128, 64) if pid == 0 else (64, 32) if lay == 1 else (32, 32)
            rgba, tuv = bake(p + [xc, 0, 0], n, uv, tri, mat, pid, W, H)
            tex = f'tlqf{bit}s{k}_{pid}'
            TEX[tex] = png(rgba)
            col = vcol(p, n, pid, 1.0, lay == 1, fish=True)
            fp, fn = to_f(p, n, floor=False)
            blend = 0x2000 | (0x8000 if (rgba[..., 3] < 250).any() else 0)
            models.append(model_bytes(tex, blend, 0, fp, fn, col, tuv.astype(np.float32), tri))
            tris += len(tri)
        out.append((0, f'fish{bit}s{k}', header(6, prm) + struct.pack('<I', len(models)) + b''.join(models)))
        prms.append(prm)
    print(f'fish{bit} {name}: {len(segs)} segments', flush=True)
    return out, prms


def fish_entries(bit, name):
    if 'eel' in swim.PATHS[name]:
        return eel_entries(bit, name)
    body, tail, mat, jx = T.fish_mesh(name)
    spec = swim.PATHS[name]
    smp = swim.samples(spec, K, FLOOR)
    prm = swim.params(spec, smp, jx * K)
    out = []
    for tag, mesh, kind in (('', body, 4), ('t', tail, 5)):
        if mesh is None:
            continue
        P, N, UV, Tr, part, layer = mesh
        models = []; tris = 0
        for pid, lay, p, n, uv, tri in split(P, N, UV, Tr, part, layer):
            W, H = (256, 128) if pid == 0 else (128, 64) if lay == 1 else (64, 64)
            rgba, tuv = bake(p, n, uv, tri, mat, pid, W, H)
            tex = f'tlqf{bit}{tag}_{pid}'
            TEX[tex] = png(rgba)
            col = vcol(p, n, pid, 1.0, lay == 1, fish=True)
            if pid == 6:      # a glowing part (lanterns, lures)
                col = np.full(len(p), 0x80FFFFFF, np.uint32)
            fp, fn = to_f(p, n, floor=False)
            blend = 0x2000 | (0x8000 if (rgba[..., 3] < 250).any() else 0)
            models.append(model_bytes(tex, blend, 0, fp, fn, col, tuv.astype(np.float32), tri))
            tris += len(tri)
        print(f'fish{bit}{tag} {name}: {len(models)} models, {tris} triangles', flush=True)
        out.append((0, f'fish{bit}{tag}', header(kind, prm) + struct.pack('<I', len(models)) + b''.join(models)))
    return out, prm


def bundle():
    global K
    K, P = fit_scale()
    v1.K = K; v1.FLOOR = FLOOR
    top = (T.CAP + 0.29 + 0.13 * 1.36 - FLOOR) * K
    print(f'K = {K:.4f}; footprint {np.ptp(P[:, 0]) * K:.3f} x {np.ptp(P[:, 2]) * K:.3f}; height {top:.3f}')
    entries = []
    r3d.SEAM = True
    r3d.LOD = 0.5
    prms = {}
    for bit, name in enumerate(T.FISH_ORDER):
        es, prm = fish_entries(bit, name)
        entries += es; prms[name] = prm
    r3d.LOD = 0.7
    for gname, fn in (('tank', None), ('crown', T.group_crown), ('hgcrystal', T.group_hgcrystal), ('grains', T.group_grains)):
        if fn is None:
            mesh, anim = T.group_tank().build(), ('none',)
        else:
            mm, anim = fn(); mesh = mm.build()
        entries.append((0, gname, tank_group(gname, mesh, anim)))
    for k, v in TEX.items():
        entries.append((1, k, v))
    entries.append((2, 'icon32', icon_raw('/home/claude/elder/out/aquarium_icon32.png')))
    out = b'TLAQ' + struct.pack('<II', 3, len(entries))
    for kind, name, data in entries:
        nb = name.encode()
        out += struct.pack('<BH', kind, len(nb)) + nb + struct.pack('<I', len(data)) + data
    open(OUT, 'wb').write(out)
    print(OUT, len(out) // 1024, 'KB,', len(TEX), 'textures')
    # a reference point for the client's test: fish 4's body and tail at t = 7.3 s
    for tail in (False, True):
        M = swim.matrix(prms['moonfin'], 7.3, tail)
        print('moonfin', 'tail' if tail else 'body', np.round(M, 5).tolist())
    for k in (0, 7):
        M = swim.matrix(prms['rimecoil'][k], 9.1, kind=6)
        print('rimecoil seg', k, (M @ [0.02, 0.01, 0.0, 1.0])[:3].round(6).tolist())


if __name__ == '__main__':
    bundle()
