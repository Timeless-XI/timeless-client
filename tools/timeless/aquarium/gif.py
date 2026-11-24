import numpy as np, tank2
from PIL import Image
from r3d import render, tonemap
tank2.STATE.update(lit=1.0, crystal=1.0)
frames = []
for i in range(36):
    t = i * (9.0 / 36)  # 9 seconds
    mesh, mat = tank2.scene(tank2.FISH_ORDER, t)
    bg = lambda h, w: np.zeros((h, w, 3)) + 0.03
    lights = [((-0.3, 1.0, 0.45), (0.85, 0.85, 0.95)), ((0.8, 0.2, 0.6), (0.2, 0.26, 0.42)), ((0.1, 0.4, -1.0), (0.4, 0.5, 0.8))]
    img = render(mesh, mat, 720, 440, eye=(0.2, 0.7, 2.6), target=(0, 0.52, 0), fov=40, bg=bg, bloom=1.0, ss=1, lights=lights,
                 env_up=(0.2, 0.24, 0.36), env_down=(0.07, 0.07, 0.1))
    frames.append(Image.fromarray((np.clip(tonemap(img)[..., :3], 0, 1) * 255).astype(np.uint8)))
frames[0].save('/home/claude/elder/out/aquarium_anim.gif', save_all=True, append_images=frames[1:], duration=250, loop=0)
print('gif ok')
