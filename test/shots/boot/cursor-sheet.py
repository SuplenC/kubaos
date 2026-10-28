"""cursor-sheet.py TOKENS DARK ACCENT OUT: sheets of the pointer's shapes from
two XCursor themes' `cursors/` directories, the dark and the accent style, a
row each, on the dark and the light `surface` of TOKENS (design/tokens.toml),
at every display scale: cursor-{dark,light}-{1,1.25,1.5,2}x.png in OUT.

At each scale a shape is the image of the size libXcursor gives an app there:
the nominal size nearest 24 times the scale, the first of two as near, drawn
as it is, unscaled."""

import struct
import sys
import tomllib
from pathlib import Path

from PIL import Image

tokens_path, dark_dir, accent_dir, out = sys.argv[1:5]
with open(tokens_path, "rb") as f:
    T = tomllib.load(f)
SIZE = 24
SCALES = T["rules"]["scales"]
# The shapes a desktop shows most, as Bibata names its XCursor files.
SHAPES = ["left_ptr", "hand2", "xterm", "watch", "left_ptr_watch", "grabbing", "sb_h_double_arrow",
          "bottom_right_corner", "crosshair", "circle"]
IMAGE = 0xFFFD0002


def images(path: Path) -> list[tuple[int, Image.Image]]:
    """Every image of an XCursor file, (nominal size, picture), in file order."""
    data = path.read_bytes()
    magic, header, _, ntoc = struct.unpack_from("<4sIII", data, 0)
    if magic != b"Xcur":
        raise ValueError(f"{path} isn't an XCursor file")
    found = []
    for i in range(ntoc):
        kind, nominal, at = struct.unpack_from("<III", data, header + i * 12)
        if kind != IMAGE:
            continue
        _, _, _, _, w, h, _, _, _ = struct.unpack_from("<IIIIIIIII", data, at)
        argb = data[at + 36 : at + 36 + w * h * 4]
        # Little-endian ARGB is B, G, R, A in memory, premultiplied.
        rgba = bytearray(len(argb))
        rgba[0::4], rgba[1::4], rgba[2::4], rgba[3::4] = argb[2::4], argb[1::4], argb[0::4], argb[3::4]
        found.append((nominal, Image.frombytes("RGBa", (w, h), bytes(rgba)).convert("RGBA")))
    return found


def best(found: list[tuple[int, Image.Image]], wanted: int) -> Image.Image:
    """libXcursor's pick: the nearest nominal size, the first on a tie, then its first frame."""
    size = min((nominal for nominal, _ in found), key=lambda n: abs(n - wanted))
    return next(im for nominal, im in found if nominal == size)


def rgb(hex_colour: str) -> tuple[int, int, int, int]:
    return tuple(int(hex_colour[i : i + 2], 16) for i in (1, 3, 5)) + (255,)


themes = [Path(dark_dir), Path(accent_dir)]
shapes = [s for s in SHAPES if all((t / s).exists() for t in themes)]
for scale in SCALES:
    cell, margin = round(40 * scale), round(16 * scale)
    wanted = round(SIZE * scale)
    for mode in ("dark", "light"):
        sheet = Image.new(
            "RGBA",
            (2 * margin + len(shapes) * cell, 2 * margin + len(themes) * cell),
            rgb(T["color"][mode]["surface"]),
        )
        for row, theme in enumerate(themes):
            for col, shape in enumerate(shapes):
                im = best(images((theme / shape).resolve()), wanted)
                x = margin + col * cell + (cell - im.width) // 2
                y = margin + row * cell + (cell - im.height) // 2
                sheet.alpha_composite(im, (x, y))
        name = f"cursor-{mode}-{scale:g}x.png"
        sheet.save(Path(out) / name, optimize=True)
