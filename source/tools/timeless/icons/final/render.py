#!/usr/bin/env python3
"""Renders the Timeless View icon masters (final/svg/*.svg) to 128 x 128 PNGs for tools/timeless/icon_map.py.

Usage: render.py <png dir>   (needs rsvg-convert: librsvg; on Windows use WSL or MSYS2's mingw-w64-x86_64-librsvg)
"""
import os
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor

SRC = os.path.join(os.path.dirname(os.path.abspath(__file__)), "svg")
OUT = sys.argv[1]
os.makedirs(OUT, exist_ok=True)


def one(f):
    subprocess.run(["rsvg-convert", "-w", "128", "-h", "128", os.path.join(SRC, f), "-o", os.path.join(OUT, f[:-4] + ".png")], check=True)


with ThreadPoolExecutor(8) as ex:
    list(ex.map(one, [f for f in os.listdir(SRC) if f.endswith(".svg")]))
print(len(os.listdir(OUT)), "png")
