#!/usr/bin/env python3
"""Boot assets: the Plymouth theme's and the GRUB theme's pictures, drawn from
the design tokens (design/tokens.toml) in the dark palette, as the components
of docs/DESIGN.md draw them.

Writes into system/files/ (the payload and a hand install both read from
there; system/25-boot.sh installs them):

  system/files/plymouth/kubaos/   the passphrase field in its three states,
                                  its icons, bullet and caret, the
                                  placeholder and the notes under it, each at
                                  1x and @2x
  system/files/grub/kubaos/       background, the menu's 9-slice boxes, the
                                  key hints
  system/files/fonts/             Geist, Geist Medium and Geist Mono
                                  (kubaos-ui's): Plymouth's text in the
                                  initramfs, and the source of GRUB's .pf2
                                  fonts (grub-mkfont)

Plymouth's label plugin has one regular and one monospace font in the
initramfs, and no UTF-8 locale, so every fixed piece of text (the medium
keycaps, the bullet, the ellipsis) is drawn here, in its type style. Only the
messages other programs send are drawn at boot.

The neon sign on both (Plymouth's switch-on layers, GRUB's logo) comes from
neon.py. The values the scripts read (kubaos.script's tokens, theme.txt) are
written by `cargo run -p kubaos-tokens -- write`, which also measures the
pictures written here: run it after this. Needs python3 (3.11 or later) with
Pillow, and resvg on PATH. Run it from anywhere:

  python3 branding/boot.py
"""

import math
import shutil
import subprocess
import tempfile
import tomllib
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

HERE = Path(__file__).resolve().parent
REPO = HERE.parent
PLY = REPO / "system/files/plymouth/kubaos"
GRUB = REPO / "system/files/grub/kubaos"
GUI_FONTS = REPO / "crates/kubaos-ui/assets/fonts"
SYMBOLS = REPO / "crates/kubaos-ui/assets/symbols"
FONTS = REPO / "system/files/fonts"

with open(REPO / "design/tokens.toml", "rb") as f:
    T = tomllib.load(f)
DARK = T["color"]["dark"]
SIZE = T["size"]
TYPE = T["type"]
SPACE = {f"s{v}": v for v in T["space"]["values"]}
RADIUS = T["radius"]
OPACITY = T["opacity"]

# The type families' files, by family and weight, as kubaos-ui embeds them.
FONT_FILES = {
    ("sans", "regular"): "Geist-Regular.ttf",
    ("sans", "medium"): "Geist-Medium.ttf",
    ("sans", "semibold"): "Geist-SemiBold.ttf",
    ("mono", "regular"): "GeistMono-Regular.ttf",
    ("mono", "medium"): "GeistMono-Medium.ttf",
}

# The words, as docs/BRAND.md ("Boot") has them.
PLACEHOLDER = "Disk passphrase"
HINT = ("The one you set when you installed KubaOS.", "Enter", "unlocks.")
CAPS = ("Caps Lock", "is on.")
BUSY = "Unlocking…"
WRONG = "Wrong passphrase. Try again."
# GRUB's keys: each hint's keys, then what they do.
GRUB_HINTS = ((("↑", "↓"), "Pick"), (("Enter",), "Boot"), (("E",), "Edit the entry"), (("C",), "GRUB shell"))


def rgb(role: str) -> tuple[int, int, int]:
    h = DARK[role].lstrip("#")
    return tuple(int(h[i : i + 2], 16) for i in (0, 2, 4))


def rgba(role: str, alpha: float = 1.0) -> tuple[int, int, int, int]:
    return rgb(role) + (round(alpha * 255),)


def px(logical: float, s: float) -> int:
    """`logical` pixels in device pixels at scale `s`. Every length on the 4 px
    grid is whole at every supported scale; anything else is a mistake."""
    device = logical * s
    if device != int(device):
        raise ValueError(f"{logical} px at {s} is {device}, between device pixels")
    return int(device)


def device(logical: float, s: float) -> int:
    """A stroke of `logical` pixels in whole device pixels at scale `s`, rounded
    half up and never under one (kubaos_ui::scale::Scale::stroke)."""
    return max(1, math.floor(logical * s + 0.5))


def resvg(svg: str, out: Path, width: int) -> None:
    with tempfile.NamedTemporaryFile("w", suffix=".svg", delete=False) as f:
        f.write(svg)
        src = f.name
    try:
        subprocess.run(["resvg", "-w", str(width), src, str(out)], check=True)
    finally:
        Path(src).unlink()


def svg_image(svg: str, width: int) -> Image.Image:
    with tempfile.TemporaryDirectory() as d:
        png = Path(d) / "out.png"
        resvg(svg, png, width)
        return Image.open(png).convert("RGBA")


def font(style: str, s: float) -> ImageFont.FreeTypeFont:
    t = TYPE[style]
    return ImageFont.truetype(str(GUI_FONTS / FONT_FILES[(t["family"], t["weight"])]), t["size"] * s)


def text(words: str, style: str, colour: tuple, s: int) -> Image.Image:
    """`words` in `style`, on a box its line tall and as wide as the words,
    rounded up to a whole pixel, centred on the line as Qt and iced centre
    text: half the room the font's ascent and descent leave above."""
    f = font(style, s)
    line = px(TYPE[style]["line"], s)
    ascent, descent = f.getmetrics()
    width = math.ceil(f.getlength(words))
    im = Image.new("RGBA", (max(width, 1), line), (0, 0, 0, 0))
    baseline = (line - (ascent + descent)) / 2 + ascent
    ImageDraw.Draw(im).text((0, baseline), words, font=f, fill=colour, anchor="ls")
    return im


def keycap(key: str, s: int) -> Image.Image:
    """A `Keycap`: the key in `keycap` type on the highest container, a
    hairline edge with a second at the bottom, the small radius, 4 either side
    of the key and never narrower than tall."""
    label = text(key, "keycap", rgba("on_surface_variant"), s)
    h = px(SIZE["keycap"], s)
    pad = px(SPACE["s4"], s)
    w = max(h, label.width + 2 * pad)
    r = px(RADIUS["small"], s)
    edge, face = DARK["outline_variant"], DARK["surface_container_highest"]
    svg = f"""<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">
<rect width="{w}" height="{h}" rx="{r}" fill="{edge}"/>
<rect x="0.5" y="0.5" width="{w - 1}" height="{h - 2}" rx="{r - 0.5}" fill="{face}" stroke="{edge}" stroke-width="1"/></svg>"""
    im = svg_image(svg, w)
    im.alpha_composite(label, ((w - label.width) // 2, (h - label.height) // 2))
    return im


def row(parts: list[tuple[int, Image.Image]]) -> Image.Image:
    """Pictures side by side, each `gap` after the one before, centred on the
    row's height; an odd spare pixel goes below, never above."""
    h = max(im.height for _, im in parts)
    w = sum(gap + im.width for gap, im in parts)
    out = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    x = 0
    for gap, im in parts:
        x += gap
        out.alpha_composite(im, (x, (h - im.height) // 2))
        x += im.width
    return out


def symbol(name: str, size: int, role: str, s: int) -> Image.Image:
    """A KubaOS Symbol at `size`, its 2 unit stroke rounded to whole device
    pixels as kubaos_ui::icon::device_stroke rounds it."""
    side = px(size, s)
    stroke = device(T["stroke"]["symbol"] * size / 24, s) * 24 / side
    svg = (SYMBOLS / f"{name}.svg").read_text()
    svg = svg.replace('stroke-width="2"', f'stroke-width="{stroke}"').replace("currentColor", DARK[role])
    return svg_image(svg, side)


def field(s: int, fill: str, edge: str, ring: bool) -> Image.Image:
    """A `Field` the size of the password field alone on a screen, with the
    focus ring round it when the keyboard is on it: the picture is the focus
    ring's box, `focus.offset` larger than the field on every side."""
    off = px(T["focus"]["offset"], s)
    fw, fh = px(SIZE["password"], s), px(SIZE["control_md"], s)
    w, h = fw + 2 * off, fh + 2 * off
    r = px(RADIUS["control"], s)
    thin = device(T["stroke"]["thin"], s)
    ring_svg = ""
    if ring:
        ring_r = r + off - thin / 2
        ring_svg = (
            f'<rect x="{thin / 2}" y="{thin / 2}" width="{w - thin}" height="{h - thin}" rx="{ring_r}" '
            f'fill="none" stroke="{DARK["primary"]}" stroke-width="{thin}"/>'
        )
    svg = f"""<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">
{ring_svg}<rect x="{off + 0.5}" y="{off + 0.5}" width="{fw - 1}" height="{fh - 1}" rx="{r - 0.5}"
 fill="{DARK[fill]}" stroke="{DARK[edge]}" stroke-width="1"/></svg>"""
    return svg_image(svg, w)


def bullet(role: str, s: int) -> Image.Image:
    """One character of a password: the field's bullet in `body_large`, in a
    box as wide as its advance (rounded to a pixel), so the script steps by the
    picture's width."""
    f = font("body_large", s)
    line = px(TYPE["body_large"]["line"], s)
    ascent, descent = f.getmetrics()
    im = Image.new("RGBA", (round(f.getlength("•")), line), (0, 0, 0, 0))
    baseline = (line - (ascent + descent)) / 2 + ascent
    ImageDraw.Draw(im).text((0, baseline), "•", font=f, fill=rgba(role), anchor="ls")
    return im


def note(*parts: str | tuple[str, str], s: int) -> Image.Image:
    """A line under the field: `small` words and keycaps 4 apart, centred on a
    keycap's height, so every note is the same height and swapping one for
    another moves nothing. A word part is (words, role)."""
    gap = px(SPACE["s4"], s)
    pieces = []
    for part in parts:
        im = text(part[0], "small", rgba(part[1]), s) if isinstance(part, tuple) else keycap(part, s)
        pieces.append((gap if pieces else 0, im))
    out = row(pieces)
    tall = Image.new("RGBA", (out.width, px(SIZE["keycap"], s)), (0, 0, 0, 0))
    tall.alpha_composite(out, (0, (tall.height - out.height) // 2))
    return tall


def plymouth() -> None:
    PLY.mkdir(parents=True, exist_ok=True)
    muted = "on_surface_variant"
    icon = TYPE["body_large"]["icon"]
    for s, suffix in ((1, ""), (2, "@2x")):
        pictures = {
            "field": field(s, "surface_container_highest", "primary", ring=True),
            "field-error": field(s, "surface_container_high", "error", ring=True),
            "field-busy": field(s, "surface_container_high", "outline_variant", ring=False),
            "icon-lock": symbol("lock", icon, muted, s),
            "icon-busy": symbol("hourglass_top", icon, muted, s),
            "bullet": bullet("on_surface", s),
            "bullet-busy": bullet(muted, s),
            "caret": Image.new(
                "RGBA", (device(T["stroke"]["thin"], s), px(TYPE["body_large"]["line"], s)), rgba("primary")
            ),
            "placeholder": text(PLACEHOLDER, "body_large", rgba(muted, OPACITY["placeholder_text"]), s),
            "note-hint": note((HINT[0], muted), HINT[1], (HINT[2], muted), s=s),
            "note-caps": note(CAPS[0], (CAPS[1], muted), s=s),
            "note-busy": note((BUSY, muted), s=s),
            "note-wrong": note((WRONG, "error"), s=s),
        }
        for name, im in pictures.items():
            im.save(PLY / f"{name}{suffix}.png", optimize=True)


def nine_slice(im: Image.Image, prefix: Path, corner: int) -> None:
    """GRUB's styled box: prefix_{nw,n,ne,w,c,e,sw,s,se}.png."""
    w, h = im.size
    c = corner
    boxes = {
        "nw": (0, 0, c, c), "n": (c, 0, c + 1, c), "ne": (w - c, 0, w, c),
        "w": (0, c, c, c + 1), "c": (c, c, c + 1, c + 1), "e": (w - c, c, w, c + 1),
        "sw": (0, h - c, c, h), "s": (c, h - c, c + 1, h), "se": (w - c, h - c, w, h),
    }
    for name, box in boxes.items():
        im.crop(box).save(f"{prefix}_{name}.png", optimize=True)


def grub() -> None:
    GRUB.mkdir(parents=True, exist_ok=True)
    s = T["boot"]["grub_scale"]
    # One flat colour, as an image: theme.txt's desktop-image, and
    # GRUB_BACKGROUND, which makes GRUB's terminal draw black as transparent,
    # so the editor and the shell sit on terminal_box instead of a black slab.
    Image.new("RGB", (16, 16), rgb("surface_container_lowest")).save(GRUB / "background.png")
    # A menu entry is a `ListRow`, `row` tall at the control radius. GRUB draws
    # an entry's box round its text, its borders outside the text's
    # `item_height`: a border as tall as the room the row leaves round its
    # line centres the text on the row, and holds the corner.
    pad = px((SIZE["row"] - TYPE["body_strong"]["line"]) / 2, s)
    r = px(RADIUS["control"], s)
    side = 2 * pad + 2
    selected = svg_image(
        f"""<svg xmlns="http://www.w3.org/2000/svg" width="{side}" height="{side}" viewBox="0 0 {side} {side}">
<rect width="{side}" height="{side}" rx="{r}" fill="{DARK["secondary_container"]}"/></svg>""",
        side,
    )
    nine_slice(selected, GRUB / "select", pad)
    # Entries not picked get an empty box with the same borders: GRUB puts an
    # entry's text inside its box's top border, so without it the picked entry
    # would sit lower than the rest.
    nine_slice(Image.new("RGBA", (side, side), (0, 0, 0, 0)), GRUB / "item", pad)
    # The box round the editor and the GRUB shell (e, c): a surface's hairline
    # edge, one device pixel wide, the surface radius in from it. GRUB paints
    # the terminal's own background, the backdrop, over the box's middle, so
    # the box has no fill of its own.
    r = px(RADIUS["surface"], s)
    side = 2 * r + 2
    nine_slice(
        svg_image(
            f"""<svg xmlns="http://www.w3.org/2000/svg" width="{side}" height="{side}" viewBox="0 0 {side} {side}">
<rect x="0.5" y="0.5" width="{side - 1}" height="{side - 1}" rx="{r - 0.5}" fill="none"
 stroke="{DARK["outline_variant"]}" stroke-width="1"/></svg>""",
            side,
        ),
        GRUB / "terminal_box",
        r,
    )
    grub_hints(s).save(GRUB / "hints.png", optimize=True)


def grub_hints(s: int) -> Image.Image:
    """`KeyHints` for the menu: each hint's keys as keycaps 4 apart, what they
    do in `caption` 8 after them, the hints 16 apart."""
    parts: list[tuple[int, Image.Image]] = []
    for keys, does in GRUB_HINTS:
        for i, key in enumerate(keys):
            gap = SPACE["s16"] if parts and i == 0 else SPACE["s4"] if i else 0
            parts.append((px(gap, s), keycap(key, s)))
        parts.append((px(SPACE["s8"], s), text(does, "caption", rgba("on_surface_variant"), s)))
    return row(parts)


def fonts() -> None:
    FONTS.mkdir(parents=True, exist_ok=True)
    for f in ("Geist-Regular.ttf", "Geist-Medium.ttf", "Geist-OFL.txt", "GeistMono-Regular.ttf"):
        shutil.copy(GUI_FONTS / f, FONTS / f)


if __name__ == "__main__":
    fonts()
    plymouth()
    grub()
    print(f"wrote {PLY.relative_to(REPO)}, {GRUB.relative_to(REPO)} and {FONTS.relative_to(REPO)}")
