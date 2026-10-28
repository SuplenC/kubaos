"""cursor-palette.py TOKENS: the pointer's palette file, as matugen writes it from
config/matugen/templates/cursor.toml, for the dark fallback palette of TOKENS
(design/tokens.toml). The shots' stand-in for a wallpaper's colours."""

import colorsys
import sys
import tomllib

with open(sys.argv[1], "rb") as f:
    dark = tomllib.load(f)["color"]["dark"]


def lightness(hex_colour: str, percent: float) -> str:
    """matugen's `set_lightness`: the colour at that HSL lightness."""
    r, g, b = (int(hex_colour[i : i + 2], 16) / 255 for i in (1, 3, 5))
    h, _, s = colorsys.rgb_to_hls(r, g, b)
    r, g, b = colorsys.hls_to_rgb(h, percent / 100, s)
    return "#" + "".join(f"{round(c * 255):02x}" for c in (r, g, b))


primary = dark["primary"]
spinner = [primary, lightness(primary, 48), lightness(primary, 88), lightness(primary, 64)]
listed = ", ".join(f'"{c}"' for c in spinner)
print(f"""[dark]
fill = "{dark["surface"]}"
outline = "{dark["on_surface"]}"
watch = "{dark["surface"]}"
spinner = [{listed}]

[accent]
fill = "{primary}"
outline = "{dark["on_primary"]}"
watch = "{dark["on_primary"]}"
spinner = [{listed}]""")
