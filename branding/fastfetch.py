#!/usr/bin/env python3
"""Prints the クバ mark as half-block art for fastfetch. `$1` is the amber,
`$2` the cyan; fastfetch's logo colors fill them in (see README)."""
import os
import subprocess
import tempfile

COLS = 30  # pixels wide = terminal columns; two pixel rows per text row

os.chdir(os.path.dirname(os.path.abspath(__file__)))
with tempfile.TemporaryDirectory() as tmp:
    png = os.path.join(tmp, "m.png")
    # Flat drawing (no glow) so every pixel is either tube or nothing.
    subprocess.run(["resvg", "-w", "400", "svg/kubaos-mark-flat.svg", png], check=True)
    ppm = subprocess.run(
        ["magick", png, "-trim", "-resize", f"{COLS}x", "-background", "black",
         "-alpha", "remove", "-depth", "8", "ppm:-"],
        check=True, capture_output=True,
    ).stdout

# Parse binary PPM: P6\n<w> <h>\n255\n<data>
parts = ppm.split(b"\n", 3)
w, h = map(int, parts[1].split())
data = parts[3]

def px(x, y):
    if y >= h:
        return None
    r, g, b = data[(y * w + x) * 3:(y * w + x) * 3 + 3]
    if max(r, g, b) < 90:
        return None
    return "2" if b > r else "1"

lines = []
for y in range(0, h, 2):
    line, cur = "", None
    for x in range(w):
        top, bot = px(x, y), px(x, y + 1)
        col = top or bot
        if col and col != cur:
            line += f"${col}"
            cur = col
        line += "█" if top and bot else "▀" if top else "▄" if bot else " "
    lines.append(line.rstrip())
print("\n".join(lines))
