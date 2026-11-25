"""Animated preview of the v3 aquarium: the fish swimming (14 s at 6 frames a second)."""
import functools, sys
import numpy as np, tank3
from PIL import Image
from r3d import render, tonemap
tank3.STATE.update(lit=1.0, crystal=1.0)
tank3.fish_mesh = functools.lru_cache(None)(tank3.fish_mesh)
_tank = tank3.group_tank().build()
class _M:
    def build(self): return _tank
tank3.group_tank = lambda: _M()
W, H = (int(sys.argv[1]), int(sys.argv[2])) if len(sys.argv) > 2 else (720, 450)
frames = []
for i in range(84):
    t = 2.0 + i / 6.0
    mesh, mat = tank3.scene(tank3.FISH_ORDER, t)
    bg = lambda h, w: np.zeros((h, w, 3)) + 0.03
    lights = [((-0.3, 1.0, 0.45), (0.85, 0.85, 0.95)), ((0.8, 0.2, 0.6), (0.22, 0.26, 0.4)), ((0.1, 0.4, -1.0), (0.4, 0.5, 0.8))]
    img = render(mesh, mat, W, H, eye=(0.9, 1.25, 5.4), target=(0, 0.62, 0), fov=38, bg=bg, bloom=1.0, ss=1, lights=lights,
                 env_up=(0.2, 0.24, 0.36), env_down=(0.07, 0.07, 0.1))
    frames.append(Image.fromarray((np.clip(tonemap(img)[..., :3], 0, 1) * 255).astype(np.uint8)))
    print(i, flush=True)
frames[0].save('/home/claude/elder/out/aquarium3_swim.gif', save_all=True, append_images=frames[1:], duration=167, loop=0)
print('gif ok')
