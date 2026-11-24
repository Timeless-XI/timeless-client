"""Export the Timeless Aquarium v2 (tank2.py) for the Timeless client: bundle version 2.

Same container as v1 (export_aquarium.py), but every mesh entry is a GROUP that the client draws as its own piece
and animates on its own:
  mesh = u8 anim kind, f32 x 8 params, then u32 models + models as v1.
  kind 0 none; 1 spin (pivot xyz, axis xyz, period s, 0); 2 bob (pivot xyz, amplitude, period s, phase, yaw deg, 0);
  3 fall (direction xyz, spacing, period s, 0, 0, 0). All in furniture space (-y up).
Groups: tank, crown, hgcrystal, grains, fish0..fish7.
Variants: 0 always; 1 lit (1+ fish) / 2 dark (none); 3 crown off (none) / 4 dim (1-7) / 5 full (all 8).
"""
import struct, sys
import numpy as np
import r3d
import export_aquarium as v1
from export_aquarium import bake, png, model_bytes, split, icon_raw, to_furniture, K, FLOOR
from r3d import normalize, C
import tank2 as T

OUT = sys.argv[1] if len(sys.argv) > 1 else '/home/claude/elder/aquarium.bin'
TEX = {}
LIT_PARTS = {T.SKY, T.SKYSIDE, T.SAND, T.KELP, T.HGSAND, T.GRAIN, T.SWIRL, T.MOTE, T.HGCRYSTAL, T.EMBLEM, T.HGGLASS}
FLAT = {T.SKY, T.SKYSIDE, T.SWIRL, T.MOTE}           # self-lit: no baked shading
GLOW = {T.CROWN, T.HGCRYSTAL, T.GRAIN, T.EMBLEM}     # drawn at 2x when lit
INSIDE = {T.SAND, T.RUIN, T.KELP, T.HGGLASS, T.HGSAND, T.GRAIN, T.HGCRYSTAL, T.HGMETAL}
SIZES = {T.SKY: (1024, 512), T.SKYSIDE: (128, 128), T.SAND: (256, 128), T.SWIRL: (256, 256), T.MOTE: (32, 32),
         T.OBSID: (128, 128), T.HAIR: (128, 64), T.RING: (128, 32), T.HGMETAL: (64, 64)}
v1.WATER = C('#2a4a80')


def vcol(p, n, part, lit, two_sided):
    if part in FLAT:
        return np.full(len(p), 0x80808080, np.uint32)
    if part in GLOW and lit > 0.5:
        return np.full(len(p), 0x80FFFFFF, np.uint32)
    return v1.vcolour(p, n, part in INSIDE, False, two_sided)


def anim_header(anim):
    kind = {'none': 0, 'spin': 1, 'bob': 2, 'fall': 3}[anim[0]]
    prm = [0.0] * 8
    if anim[0] == 'spin':
        c, ax, period = anim[1], anim[2], anim[3]
        pc, _ = to_furniture(np.asarray([c], float), np.zeros((1, 3)))
        a = np.asarray(ax, float); a = np.array([a[0], -a[1], -a[2]])
        prm = [*pc[0], *normalize(a), period, 0.0]
    elif anim[0] == 'bob':
        c, amp, period, ph, yaw = anim[1:]
        pc, _ = to_furniture(np.asarray([c], float), np.zeros((1, 3)))
        prm = [*pc[0], amp * K, period, ph, yaw, 0.0]
    elif anim[0] == 'fall':
        d, sp, period = anim[1:]
        d = np.asarray(d, float); d = np.array([d[0], -d[1], -d[2]])
        prm = [*d, sp * K, period, 0.0, 0.0, 0.0]
    return struct.pack('<B8f', kind, *[float(x) for x in prm])


def tank_group(gname, mesh, anim):
    P, N, UV, Tr, part, layer = mesh
    models = []; tris = 0
    for pid, lay, p, n, uv, tri in split(P, N, UV, Tr, part, layer):
        if pid == T.CROWN:
            variants = [(3, 0.0), (4, 0.3), (5, 1.0)]
        elif pid in LIT_PARTS:
            variants = [(1, 1.0), (2, 0.0)]
        else:
            variants = [(0, 1.0)]
        for variant, level in variants:
            if pid == T.CROWN:
                T.STATE.update(lit=1.0, crystal=level)
            else:
                T.STATE.update(lit=level, crystal=1.0)
            W, H = SIZES.get(pid, (64, 64))
            rgba, tuv = bake(p, n, uv, tri, T.material, pid, W, H)
            tex = f'tlq{gname[:4]}{pid - 1000}_{variant}'
            TEX[tex] = png(rgba)
            col = vcol(p, n, pid, level if pid != T.CROWN else level, lay == 1)
            fp, fn = to_furniture(p, n)
            blend = 0x2000 | (0x8000 if (rgba[..., 3] < 250).any() else 0)
            models.append(model_bytes(tex, blend, variant, fp, fn, col, tuv.astype(np.float32), tri))
            tris += len(tri)
    print(f'{gname}: {len(models)} models, {tris} triangles', flush=True)
    return anim_header(anim) + struct.pack('<I', len(models)) + b''.join(models)


def fish_group(bit, name):
    (P, N, UV, Tr, part, layer), mat, (Rm, s, t, c), anim = T.fish_group(name)
    models = []; tris = 0
    for pid, lay, p, n, uv, tri in split(P, N, UV, Tr, part, layer):
        W, H = (256, 128) if pid == 0 else (128, 64) if lay == 1 else (64, 64)
        rgba, tuv = bake(p, n, uv, tri, mat, pid, W, H)
        tex = f'tlqf{bit}_{pid}'
        TEX[tex] = png(rgba)
        pt = (p - c) @ Rm.T * s + t; nt = n @ Rm.T
        col = v1.vcolour(pt, nt, True, pid == 6, lay == 1)
        fp, fn = to_furniture(pt, nt)
        blend = 0x2000 | (0x8000 if (rgba[..., 3] < 250).any() else 0)
        models.append(model_bytes(tex, blend, 0, fp, fn, col, tuv.astype(np.float32), tri))
        tris += len(tri)
    print(f'fish{bit} {name}: {len(models)} models, {tris} triangles', flush=True)
    return anim_header(anim) + struct.pack('<I', len(models)) + b''.join(models)


def bundle():
    entries = []
    r3d.SEAM = True
    r3d.LOD = 0.5
    for bit, name in enumerate(T.FISH_ORDER):
        entries.append((0, f'fish{bit}', fish_group(bit, name)))
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
    out = b'TLAQ' + struct.pack('<II', 2, len(entries))
    for kind, name, data in entries:
        nb = name.encode()
        out += struct.pack('<BH', kind, len(nb)) + nb + struct.pack('<I', len(data)) + data
    open(OUT, 'wb').write(out)
    print(OUT, len(out) // 1024, 'KB,', len(TEX), 'textures')


if __name__ == '__main__':
    bundle()
