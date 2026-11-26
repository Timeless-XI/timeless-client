"""Inventory icon for the Timeless Aquarium item (v3): the full show piece, three-quarter view, transparent."""
import numpy as np
from PIL import Image
import tank3 as tank
from r3d import render, save
from render_all import outline

OUT = '/home/claude/elder/out/'
tank.STATE.update(lit=1.0, crystal=1.0)
mesh, material = tank.scene(tank.FISH_ORDER)
lights = [((-0.3, 1.0, 0.6), (1.2, 1.12, 1.0)), ((0.8, 0.2, 0.6), (0.3, 0.38, 0.5)), ((0.1, 0.4, -1.0), (0.4, 0.6, 0.8))]
img = render(mesh, material, 512, 512, eye=(3.2, 2.2, 6.4), target=(0, 0.6, 0), fov=40, bloom=0.6, ss=2, lights=lights,
             env_up=(0.4, 0.46, 0.56))
save(img, OUT + 'aquarium_icon_master.png')
m = Image.open(OUT + 'aquarium_icon_master.png').convert('RGBA')
bbox = m.getbbox(); m = m.crop(bbox)
side = max(m.size); sq = Image.new('RGBA', (side, side)); sq.alpha_composite(m, ((side - m.size[0]) // 2, (side - m.size[1]) // 2))
for size, pad, ow, name in ((128, 4, 2, 'aquarium_icon128.png'), (32, 1, 1, 'aquarium_icon32.png')):
    s = sq.resize((size - 2 * pad, size - 2 * pad), Image.LANCZOS)
    p = Image.new('RGBA', (size, size)); p.alpha_composite(s, (pad, pad))
    a = np.asarray(p).copy(); solid = a[..., 3] > 120
    ic = outline(Image.fromarray(np.where(solid[..., None], a, 0).astype(np.uint8)), ow)
    ic.alpha_composite(p)
    ic.save(OUT + name)
Image.open(OUT + 'aquarium_icon32.png').resize((128, 128), Image.NEAREST).save(OUT + 'aquarium_icon32x4.png')
print('ok')
