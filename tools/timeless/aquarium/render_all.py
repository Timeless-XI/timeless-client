import sys, importlib, numpy as np
from PIL import Image, ImageFilter
from r3d import *

FISH = ['barbel', 'oreeater', 'whiskerwise', 'admiral', 'moonfin', 'rimecoil', 'herald', 'siren']
OUT = '/home/claude/elder/out/'


def bg_studio(H, W):
    y = np.linspace(0, 1, H)[:, None]; x = np.linspace(-1, 1, W)[None, :]
    base = np.array([0.035, 0.04, 0.062]) + np.array([0.05, 0.06, 0.09]) * np.exp(-((x * 0.8) ** 2 + ((y - 0.45) * 1.6) ** 2))[..., None]
    return base


def bounds(mesh):
    P = mesh[0]
    return P.min(0), P.max(0)


def transform(mesh, Rm, t=(0, 0, 0), s=1.0):
    P, N, UV, T, part, layer = mesh
    return (P @ Rm.T * s + np.asarray(t), N @ Rm.T, UV, T, part, layer)


def rotz(a):
    a = np.radians(a); c, s = np.cos(a), np.sin(a)
    return np.array([[c, -s, 0], [s, c, 0], [0, 0, 1]])


def hero(name):
    mod = importlib.import_module('fish_' + name)
    mesh, body = mod.build()
    lo, hi = bounds(mesh); c = (lo + hi) / 2; ext = np.max(hi - lo)
    d = np.array([0.62, 0.2, 0.76]); d /= np.linalg.norm(d)
    img = render(mesh, mod.material, 1200, 760, eye=c + d * ext * 1.5, target=c, fov=30, bg=bg_studio, bloom=0.8, ss=2)
    save(img, OUT + f'{name}_hero.png')
    # side reference, orthographic
    img = render(mesh, mod.material, 1200, 520, eye=c + np.array([0, 0, 5.0]), target=c, ortho=ext * 0.58, bg=bg_studio, bloom=0.6, ss=2)
    save(img, OUT + f'{name}_side.png')
    # icon master: side view tilted head-up, transparent
    m2 = transform(mesh, rotz(36), -c)
    lo2, hi2 = bounds(m2); c2 = (lo2 + hi2) / 2; ext2 = np.max(hi2 - lo2)
    lights = [((-0.4, 0.8, 0.6), (1.25, 1.12, 0.95)), ((0.7, 0.1, 0.6), (0.3, 0.36, 0.5)), ((0.2, 0.3, -1.0), (0.6, 0.7, 0.9))]
    img = render(m2, mod.material, 512, 512, eye=c2 + np.array([0.0, 0.0, 5.0]), target=c2, ortho=ext2 * 0.5, bloom=0.7, ss=2, lights=lights,
                 env_up=(0.42, 0.46, 0.56))
    save(img, OUT + f'{name}_icon_master.png')


def outline(im, width):
    a = np.asarray(im)[..., 3]
    grown = Image.fromarray(a).filter(ImageFilter.MaxFilter(width * 2 + 1))
    ol = Image.new('RGBA', im.size, (10, 8, 14, 0)); ol.putalpha(grown)
    ol = ol.point(lambda v: v)  # keep
    base = Image.new('RGBA', im.size, (0, 0, 0, 0))
    base.alpha_composite(ol); base.alpha_composite(im)
    return base


def icons(name):
    m = Image.open(OUT + f'{name}_icon_master.png').convert('RGBA')
    # HD 128: downsample, 2 px outline (alpha threshold so the glow halo doesn't thicken the line)
    hd = m.resize((120, 120), Image.LANCZOS)
    pad = Image.new('RGBA', (128, 128), (0, 0, 0, 0)); pad.alpha_composite(hd, (4, 4))
    a = np.asarray(pad).copy(); solid = a[..., 3] > 150
    core = pad.copy(); arr = np.asarray(core).copy(); arr[..., 3] = np.where(solid, arr[..., 3], arr[..., 3] // 2); core = Image.fromarray(arr)
    hd = outline(Image.fromarray(np.where(solid[..., None], a, 0).astype(np.uint8)), 2)
    hd.alpha_composite(pad)
    hd.save(OUT + f'{name}_icon128.png')
    # native 32 x 32, 8-bit palette like the game's item icons
    sm = m.resize((30, 30), Image.LANCZOS)
    pad = Image.new('RGBA', (32, 32), (0, 0, 0, 0)); pad.alpha_composite(sm, (1, 1))
    arr = np.asarray(pad).copy(); solid = arr[..., 3] > 110
    ic = outline(Image.fromarray(np.where(solid[..., None], arr, 0).astype(np.uint8)), 1)
    ic.alpha_composite(pad)
    q = ic.convert('RGBA').quantize(colors=255, method=Image.Quantize.FASTOCTREE).convert('RGBA')
    q.save(OUT + f'{name}_icon32.png')
    q.resize((128, 128), Image.NEAREST).save(OUT + f'{name}_icon32x4.png')


if __name__ == '__main__':
    names = sys.argv[1:] or FISH
    for n in names:
        hero(n); icons(n); print('done', n, flush=True)
