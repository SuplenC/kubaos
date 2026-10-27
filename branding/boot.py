#!/usr/bin/env python3
"""Boot assets: the Plymouth theme's and the GRUB theme's PNGs.

Writes into system/files/ (the payload and a hand install both read from
there; system/25-boot.sh installs them):

  system/files/plymouth/kubaos/   field, bullet (1x and @2x)
  system/files/grub/kubaos/       background, the menu's 9-slice boxes
  system/files/fonts/             Geist and JetBrains Mono (Kōbō's fonts):
                                  Plymouth's label in the initramfs, and the
                                  source of GRUB's .pf2 fonts (grub-mkfont)

The neon sign on both (Plymouth's switch-on layers, GRUB's logo) comes from
neon.py. The theme scripts (kubaos.script, theme.txt) are written by hand next
to them. Needs python3 with Pillow and resvg on PATH. Run it from anywhere:

  python3 branding/boot.py
"""

import shutil
import subprocess
import tempfile
from pathlib import Path

from PIL import Image

HERE = Path(__file__).resolve().parent
REPO = HERE.parent
PLY = REPO / "system/files/plymouth/kubaos"
GRUB = REPO / "system/files/grub/kubaos"
GUI_FONTS = REPO / "crates/kubaos-ui/assets/fonts"
FONTS = REPO / "system/files/fonts"

# Roles from config/matugen/default-colors.json, as in design/tokens.toml.
AMBER = "#ffb779"  # primary
ERROR = "#ffb4ab"  # error
FIELD = "#261e17"  # surface_container
SELECT = "#312821"  # surface_container_high
OUTLINE_VARIANT = "#544436"
DEEP = (0x14, 0x0D, 0x07)  # surface_container_lowest

# Plymouth sizes at 1x (the script picks @2x on screens 1600 px and taller).
FIELD_W, FIELD_H, FIELD_R = 320, 44, 12
BULLET = 8


def resvg(svg: str, out: Path, width: int) -> None:
    with tempfile.NamedTemporaryFile("w", suffix=".svg", delete=False) as f:
        f.write(svg)
        src = f.name
    try:
        subprocess.run(["resvg", "-w", str(width), src, str(out)], check=True)
    finally:
        Path(src).unlink()


def field_svg(border: str, border_opacity: float) -> str:
    # 1 px border at 1x, inset half a pixel so it lands on whole pixels.
    w, h, r = FIELD_W, FIELD_H, FIELD_R
    return f"""<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">
<rect x="0.5" y="0.5" width="{w - 1}" height="{h - 1}" rx="{r}" fill="{FIELD}"
 stroke="{border}" stroke-opacity="{border_opacity}" stroke-width="1"/></svg>"""


def bullet_svg() -> str:
    b = BULLET
    return f"""<svg xmlns="http://www.w3.org/2000/svg" width="{b}" height="{b}" viewBox="0 0 {b} {b}">
<circle cx="{b / 2}" cy="{b / 2}" r="{b / 2}" fill="{AMBER}"/></svg>"""


def plymouth() -> None:
    PLY.mkdir(parents=True, exist_ok=True)
    for scale, suffix in ((1, ""), (2, "@2x")):
        resvg(field_svg(AMBER, 0.55), PLY / f"field{suffix}.png", FIELD_W * scale)
        resvg(field_svg(OUTLINE_VARIANT, 1.0), PLY / f"field-busy{suffix}.png", FIELD_W * scale)
        resvg(field_svg(ERROR, 0.9), PLY / f"field-error{suffix}.png", FIELD_W * scale)
        resvg(bullet_svg(), PLY / f"bullet{suffix}.png", BULLET * scale)


def nine_slice(src: Path, prefix: Path, corner: int) -> None:
    """GRUB's styled box: prefix_{nw,n,ne,w,c,e,sw,s,se}.png."""
    im = Image.open(src).convert("RGBA")
    w, h = im.size
    c = corner
    boxes = {
        "nw": (0, 0, c, c), "n": (c, 0, c + 1, c), "ne": (w - c, 0, w, c),
        "w": (0, c, c, c + 1), "c": (c, c, c + 1, c + 1), "e": (w - c, c, w, c + 1),
        "sw": (0, h - c, c, h), "s": (c, h - c, c + 1, h), "se": (w - c, h - c, w, h),
    }
    for name, box in boxes.items():
        im.crop(box).save(f"{prefix}_{name}.png")


def grub() -> None:
    GRUB.mkdir(parents=True, exist_ok=True)
    # One flat colour, as an image: theme.txt's desktop-image, and
    # GRUB_BACKGROUND, which makes GRUB's terminal draw black as transparent,
    # so the editor and the shell sit on terminal_box instead of a black slab.
    Image.new("RGB", (16, 16), DEEP).save(GRUB / "background.png")
    # GRUB draws a box's border slices outside the item, so theme.txt's
    # item_spacing has to leave room for two of them (2 x 14 px).
    r = 12
    sel = f"""<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 64 64">
<rect x="1" y="1" width="62" height="62" rx="{r}" fill="{SELECT}" stroke="{AMBER}" stroke-opacity="0.45" stroke-width="2"/></svg>"""
    # The box around the editor and the GRUB shell (e, c).
    term = f"""<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 64 64">
<rect x="1" y="1" width="62" height="62" rx="{r}" fill="{FIELD}" stroke="{OUTLINE_VARIANT}" stroke-width="2"/></svg>"""
    with tempfile.TemporaryDirectory() as d:
        png = Path(d) / "select.png"
        resvg(sel, png, 64)
        nine_slice(png, GRUB / "select", r + 2)
        # Unselected items get an invisible box with the same borders: GRUB
        # puts an item's text below its box's top border, so without it the
        # selected entry sits lower than the rest.
        Image.new("RGBA", (64, 64), (0, 0, 0, 0)).save(Path(d) / "item.png")
        nine_slice(Path(d) / "item.png", GRUB / "item", r + 2)
        png = Path(d) / "terminal.png"
        resvg(term, png, 64)
        nine_slice(png, GRUB / "terminal_box", r + 2)


def fonts() -> None:
    # JetBrains Mono, the boot's monospace, is kept in FONTS itself: no front end
    # carries a copy to take it from.
    FONTS.mkdir(parents=True, exist_ok=True)
    for f in ("Geist-Regular.ttf", "Geist-Medium.ttf", "Geist-OFL.txt"):
        shutil.copy(GUI_FONTS / f, FONTS / f)


if __name__ == "__main__":
    fonts()
    plymouth()
    grub()
    print(f"wrote {PLY.relative_to(REPO)}, {GRUB.relative_to(REPO)} and {FONTS.relative_to(REPO)}")
