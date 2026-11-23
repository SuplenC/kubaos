#!/usr/bin/env python3
"""Draws an Apps catalog icon for an app whose vendor publishes none, into apps/<id>/<id>.svg.

    design/app-icon.py <id> [<tabler symbol>]

With a symbol, the tile carries that KubaOS Symbol, named by its Tabler name as `symbol` in an
app.toml is; without one, the app's initial, by IconTile's rule: the first grapheme of its
`name`, upper-cased, in the headline style's weight of Geist, or of M PLUS 1 for a kana or a
kanji. The letter is outlined, so drawing the icon needs no font.

The tile is Colloid's plate, as every catalog icon's: 56 of 64 inset 4, corner 13, on a 500
grid. A symbol sits on it as IconTile's lg symbol sits on its tile, a `headline` icon in a
`control_lg` square. A letter is drawn larger than IconTile's, so it reads as heavy as a symbol
beside it at 32 and at 128 pixels. Its colours are fixed, a picture's own, not the
wallpaper's.

Run it through `just app-icon`, in the flake's `branding` shell, which has fontTools and names
M PLUS 1 in `MPLUS1_DIR`. It overwrites the app's icon file; the header records the command,
so the file can be drawn again."""
import os
import sys
import tomllib
import unicodedata
import xml.etree.ElementTree as ET

from fontTools.pens.boundsPen import BoundsPen
from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.pens.transformPen import TransformPen
from fontTools.ttLib import TTFont

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
with open(os.path.join(REPO, "design/tokens.toml"), "rb") as f:
    TOKENS = tomllib.load(f)

GRID = 500
# Colloid's plate, 56 of 64 inset 4 with a corner of 13, on the 500 grid, as
# docs/CONTRACTS.md gives it for every catalog icon.
PLATE_INSET = GRID * 4 / 64
PLATE = GRID * 56 / 64
PLATE_RADIUS = GRID * 13 / 64

HEADLINE = TOKENS["type"]["headline"]
GLYPH_BOX = PLATE * HEADLINE["icon"] / TOKENS["size"]["control_lg"]
TABLER_GRID = 24

# A letter's size, judged by eye beside the symbols at 32 and 128 pixels: a Latin capital is this
# much of the glyph box tall, by its font's cap height, and a kana's or a kanji's em is this much
# of it, as an ideograph fills its em where a capital fills its cap height.
LATIN_CAP = 0.64
CJK_EM = 0.84

# Colloid's own plate for a one-colour mark, its OBS icon's grey and the light grey of its
# glyph, so a generated icon reads as one of Colloid's beside them. They are a picture's
# colours, fixed, the same on a dark menu and a light one; no design token holds them, as no
# KubaOS surface is drawn in them.
TILE = "#4d4d4d"
INK = "#f2f2f2"

SYMBOLS_DIR = os.path.join(REPO, "crates/kubaos-ui/assets/symbols")
FONTS_DIR = os.path.join(REPO, "crates/kubaos-ui/assets/fonts")
WEIGHT_FILE = {"regular": "Regular", "medium": "Medium", "semibold": "SemiBold"}


def fail(message):
    print(f"app-icon: {message}", file=sys.stderr)
    sys.exit(1)


def num(value):
    return f"{value:.4f}".rstrip("0").rstrip(".")


def symbol_name(tabler):
    """The KubaOS Symbol drawn from Tabler's `tabler`, as `kubaos-core` looks it up."""
    for name, symbol in TOKENS["symbols"].items():
        if symbol.get("tabler") == tabler:
            return name
    fail(f"{tabler!r} is not the Tabler name of one of KubaOS Symbols (design/tokens.toml, [symbols])")


def symbol_paths(tabler):
    """The pinned Tabler drawing's strokes, its invisible 24 unit frame left out."""
    tree = ET.parse(os.path.join(SYMBOLS_DIR, f"{symbol_name(tabler)}.svg"))
    shapes = []
    for element in tree.getroot():
        if element.get("stroke") == "none":
            continue
        tag = element.tag.split("}")[-1]
        attributes = " ".join(f'{k}="{v}"' for k, v in element.attrib.items())
        shapes.append(f"<{tag} {attributes}/>")
    return shapes


def symbol_glyph(tabler, ink):
    """The symbol on Tabler's grid, scaled to the glyph box and centred on the tile.

    Tabler's grid is the drawing's optical centre (docs/DESIGN.md, "Iconography"), so the box,
    not the ink, is centred."""
    scale = GLYPH_BOX / TABLER_GRID
    offset = (GRID - GLYPH_BOX) / 2
    return (
        f'<g transform="translate({num(offset)} {num(offset)}) scale({num(scale)})" fill="none" '
        f'stroke="{ink}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">'
        + "".join(symbol_paths(tabler))
        + "</g>"
    )


def initial(name):
    """The first grapheme of `name`, upper-cased: a base character and the marks that follow it."""
    name = name.strip()
    if not name:
        return ""
    end = 1
    while end < len(name) and (
        unicodedata.combining(name[end]) or unicodedata.category(name[end]) in ("Mn", "Me", "Mc")
    ):
        end += 1
    return unicodedata.normalize("NFC", name[:end].upper())


def japanese(letter):
    """A kana or a kanji, which M PLUS 1 draws and Geist doesn't (IconTile's `japanese`)."""
    cp = ord(letter[0])
    return (
        0x3040 <= cp <= 0x30FF
        or 0x3400 <= cp <= 0x4DBF
        or 0x4E00 <= cp <= 0x9FFF
        or 0xF900 <= cp <= 0xFAFF
        or 0xFF66 <= cp <= 0xFF9F
        or 0x20000 <= cp <= 0x3134F
    )


def font_for(letter):
    weight = WEIGHT_FILE[HEADLINE["weight"]]
    if japanese(letter):
        directory = os.environ.get("MPLUS1_DIR") or fail("MPLUS1_DIR is unset: run it through `just app-icon`")
        return os.path.join(directory, f"Mplus1-{weight}.ttf"), "M PLUS 1"
    return os.path.join(FONTS_DIR, f"Geist-{weight}.ttf"), "Geist"


def letter_size(font, ideograph):
    """The size `font`'s letter is drawn at on the 500 grid: by its em for a kana or a kanji, else
    by its cap height."""
    em = font["head"].unitsPerEm
    if ideograph:
        return GLYPH_BOX * CJK_EM
    return GLYPH_BOX * LATIN_CAP * em / font["OS/2"].sCapHeight


def letter_glyph(letter, ink):
    """`letter` outlined, at `letter_size`: across on its advance, down on the top and bottom of
    its ink, as IconTile centres an initial."""
    if len(letter) != 1:
        fail(f"{letter!r} is more than one character, which no font here draws as one glyph")
    path, family = font_for(letter)
    font = TTFont(path)
    glyph_name = font.getBestCmap().get(ord(letter)) or fail(f"{family} has no glyph for {letter!r}")
    glyphs = font.getGlyphSet()
    glyph = glyphs[glyph_name]

    bounds = BoundsPen(glyphs)
    glyph.draw(bounds)
    if bounds.bounds is None:
        fail(f"{family}'s glyph for {letter!r} draws nothing")
    _, y_min, _, y_max = bounds.bounds

    scale = letter_size(font, japanese(letter)) / font["head"].unitsPerEm
    x = GRID / 2 - glyph.width * scale / 2
    baseline = GRID / 2 + (y_min + y_max) / 2 * scale

    svg = SVGPathPen(glyphs, ntos=num)
    glyph.draw(TransformPen(svg, (scale, 0, 0, -scale, x, baseline)))
    return f'<path d="{svg.getCommands()}" fill="{ink}"/>', family


def icon(glyph, tile):
    """The tile, Colloid's plate, with `glyph` on it."""
    return (
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{GRID}" height="{GRID}" viewBox="0 0 {GRID} {GRID}">'
        f'<rect x="{num(PLATE_INSET)}" y="{num(PLATE_INSET)}" width="{num(PLATE)}" height="{num(PLATE)}" '
        f'rx="{num(PLATE_RADIUS)}" fill="{tile}"/>'
        f"{glyph}</svg>\n"
    )


def draw(name, tabler=None, tile=TILE, ink=INK):
    """The icon's SVG body for an app called `name`, and what its glyph is, for the header."""
    if tabler:
        return icon(symbol_glyph(tabler, ink), tile), f"Tabler's {tabler} (@tabler/icons 3.49.0, MIT)"
    letter = initial(name)
    if not letter:
        fail(f"{name!r} has no letter to draw")
    glyph, family = letter_glyph(letter, ink)
    return icon(glyph, tile), f"its initial, {letter}, in {family} (OFL-1.1), outlined"


def main(argv):
    if len(argv) not in (1, 2) or argv[0] in ("-h", "--help"):
        print(__doc__.split("\n\n")[1], file=sys.stderr)
        sys.exit(2)
    app_id, tabler = argv[0], (argv[1] if len(argv) == 2 else None)
    folder = os.path.join(REPO, "apps", app_id)
    try:
        with open(os.path.join(folder, "app.toml"), "rb") as f:
            name = tomllib.load(f)["name"]
    except FileNotFoundError:
        fail(f"no app {app_id!r}: apps/{app_id}/app.toml is missing")

    body, what = draw(name, tabler)
    command = " ".join(["just app-icon", app_id] + ([tabler] if tabler else []))
    header = (
        f"<!-- {name}'s icon, generated by `{command}` (design/app-icon.py), as its vendor "
        f"publishes none: {what}, on KubaOS's tile, Colloid's plate. MIT, the repository's "
        f"licence. Drawn again by the same command; a vendor's icon replaces it. -->\n"
    )
    with open(os.path.join(folder, f"{app_id}.svg"), "w") as f:
        f.write(header + body)


if __name__ == "__main__":
    main(sys.argv[1:])
