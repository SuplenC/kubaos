#!/usr/bin/env python3
"""Generates the KubaOS logo SVGs. No fonts: every glyph is a stroked path
drawn on a grid here. Run ./render.sh afterwards for the PNGs.

The mark's strokes, its tubes' widths and every colour are the design tokens'
(design/tokens.toml, `[neon.mark]` and the dark palette), which Kōbō's done
screen draws the same mark from."""
import os
import tomllib

with open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "../design/tokens.toml"), "rb") as f:
    TOKENS = tomllib.load(f)
DARK = TOKENS["color"]["dark"]
MARK = TOKENS["neon"]["mark"]

AMBER = DARK["primary"]
CYAN = DARK["cold"]
INK = DARK["on_surface"]
NIGHT = DARK["surface"]
DEEP = DARK["surface_container_lowest"]

# --- the mark: クバ in a 100x100 box ---------------------------------------
# ク: a short tick, then one stroke that runs right and sweeps down-left.
# バ: two diverging strokes (both warm); the dakuten are two cold ticks, like a
# second neon tube on the same sign.
WARM = [s["path"] for s in MARK["strokes"] if s["tone"] == "warm"]
DAKUTEN = [s["path"] for s in MARK["strokes"] if s["tone"] == "cold"]
# Lit core of the neon tube: the logo's own, paler than a control's.
CORE_AMBER = DARK["logo_core_warm"]
CORE_CYAN = DARK["logo_core_cold"]


def strokes(paths, color, width):
    return "".join(
        f'<path d="{d}" fill="none" stroke="{color}" stroke-width="{width}" '
        f'stroke-linecap="round" stroke-linejoin="round"/>'
        for d in paths
    )


def mark(main=AMBER, accent=CYAN, width=MARK["width"], glow=True):
    """The glyphs, in a 100x100 coordinate space. With `glow`, drawn as neon:
    a soft halo, the tube, and a pale lit core."""
    g = ""
    if glow:
        g += f'<g filter="url(#glow)" opacity="0.7">{strokes(WARM, main, width * 1.3)}{strokes(DAKUTEN, accent, width)}</g>'
    g += strokes(WARM, main, width) + strokes(DAKUTEN, accent, width * MARK["cold_width"])
    if glow:
        g += strokes(WARM, CORE_AMBER, width * MARK["core"]) + strokes(DAKUTEN, CORE_CYAN, width * MARK["cold_core"])
    return g


GLOW = (
    '<defs><filter id="glow" x="-30%" y="-30%" width="160%" height="160%">'
    '<feGaussianBlur stdDeviation="4"/></filter>'
    # The window border gradient from the desktop: primary to tertiary.
    f'<linearGradient id="edge" x1="0" y1="0" x2="1" y2="1">'
    f'<stop offset="0" stop-color="{AMBER}"/><stop offset="1" stop-color="{CYAN}"/></linearGradient></defs>'
)

# --- the wordmark: KubaOS, geometric, cap height 60, x-height 42 ----------
# Each letter: (advance, paths) in a box whose baseline is y=60.
def K(x):
    return 40, [f"M{x+5} 0 V60", f"M{x+36} 0 L{x+8} 34", f"M{x+17} 25 L{x+38} 60"]

def u(x):
    return 44, [f"M{x+5} 18 V41 A16 16 0 0 0 {x+37} 41 V18", f"M{x+37} 41 V60"]

def b(x):
    return 46, [f"M{x+5} 0 V60", f"M{x+5} 39 A18 21 0 1 1 {x+5} 39.01"]

def a(x):
    return 46, [f"M{x+41} 39 A18 21 0 1 0 {x+41} 39.01", f"M{x+41} 18 V60"]

def O(x):
    return 64, [f"M{x+33} 0 A28 30 0 1 1 {x+32.99} 0"]

def S(x):
    return 44, [f"M{x+38} 10 C{x+32} 1 {x+7} 0 {x+6} 15 C{x+5} 30 {x+39} 27 {x+39} 44 C{x+39} 61 {x+9} 62 {x+3} 49"]


def wordmark(x0=0, kuba=INK, os_=AMBER, width=9.5):
    out, x = "", x0
    for fn, col in [(K, kuba), (u, kuba), (b, kuba), (a, kuba), (O, os_), (S, os_)]:
        adv, ps = fn(x)
        gap = 11 if fn is a else 6  # a little air between Kuba and OS
        out += strokes(ps, col, width)
        x += adv + gap
    return out, x - x0


def svg(w, h, body, bg=None, rx=0, defs=GLOW):
    back = f'<rect width="{w}" height="{h}" rx="{rx}" fill="{bg}"/>' if bg else ""
    return (
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">'
        f"{defs}{back}{body}</svg>\n"
    )


def write(name, s):
    os.makedirs("svg", exist_ok=True)
    with open(f"svg/{name}.svg", "w") as f:
        f.write(s)


os.chdir(os.path.dirname(os.path.abspath(__file__)))

# Mark alone, transparent, with a neon glow.
write("kubaos-mark", svg(120, 120, f'<g transform="translate(10 10)">{mark()}</g>'))
# App-icon tile: the mark on a dark squircle-ish rounded square.
tile = (
    f'<rect x="2" y="2" width="116" height="116" rx="30" fill="{NIGHT}"/>'
    f'<rect x="2" y="2" width="116" height="116" rx="30" fill="none" stroke="url(#edge)" stroke-opacity="0.55" stroke-width="2.5"/>'
    f'<g transform="translate(16 15) scale(0.88)">{mark(width=11)}</g>'
)
write("kubaos-icon", svg(120, 120, tile))
# 16 and 32 px: no glow, no core, fatter strokes, so it stays crisp.
small = (
    f'<rect x="0" y="0" width="120" height="120" rx="28" fill="{NIGHT}"/>'
    f'<g transform="translate(9 8) scale(1.02)">{mark(width=14.5, glow=False)}</g>'
)
write("kubaos-icon-small", svg(120, 120, small))
# Flat two-colour drawing, used to make the terminal logo.
write("kubaos-mark-flat", svg(120, 120, f'<g transform="translate(10 10)">{mark(width=7.5, glow=False)}</g>'))
# Single colour (white and black), no glow, for stamping and 16 px.
write("kubaos-mark-white", svg(120, 120, f'<g transform="translate(10 10)">{mark("#ffffff", "#ffffff", glow=False)}</g>'))
write("kubaos-mark-black", svg(120, 120, f'<g transform="translate(10 10)">{mark("#000000", "#000000", glow=False)}</g>'))

# Horizontal: mark, then the wordmark on the same baseline.
wm, wmw = wordmark()
W = 120 + 22 + int(wmw) + 12
write(
    "kubaos-horizontal",
    svg(W, 120, f'<g transform="translate(10 10)">{mark()}</g><g transform="translate(142 30)">{wm}</g>'),
)
wmw_white, _ = wordmark(kuba="#ffffff", os_="#ffffff")
write(
    "kubaos-horizontal-white",
    svg(W, 120, f'<g transform="translate(10 10)">{mark("#ffffff", "#ffffff", glow=False)}</g><g transform="translate(142 30)">{wmw_white}</g>'),
)
# The wordmark alone (Kōbō sets it next to its own mark).
write("kubaos-wordmark", svg(int(wmw) + 12, 72, f'<g transform="translate(6 6)">{wm}</g>', defs=""))
# Stacked: mark over the wordmark, centred.
SW = max(int(wmw) + 24, 200)
write(
    "kubaos-stacked",
    svg(SW, 230, f'<g transform="translate({(SW-100)/2} 8)">{mark()}</g><g transform="translate({(SW-wmw)/2} 150)">{wm}</g>'),
)
# Dark-background versions (for docs on light pages, previews).
write("kubaos-horizontal-dark", svg(W + 40, 160, f'<g transform="translate(30 30)">{mark()}</g><g transform="translate(162 50)">{wm}</g>', bg=DEEP, rx=24))
print("wrote svg/*.svg")
