# KubaOS branding

The mark is クバ, Kuba in katakana, drawn as neon tubes: amber strokes with a
pale lit core and a soft halo, and the dakuten as a second, cyan tube on the
same sign. The wordmark is KubaOS in a geometric round-stroke alphabet:
"Kuba" in off-white, "OS" in amber. Nothing here uses a font. Every glyph is
a stroked path built in `make.py`, so the SVGs render the same everywhere.

Rules for the name and the voice are in `docs/BRAND.md`.

## Files

| File | Use it for |
|---|---|
| `svg/kubaos-mark.svg` | The mark alone, neon, transparent background. Splash screens, about boxes, big stuff. |
| `svg/kubaos-icon.svg` | App icon: the mark on a dark rounded tile with the desktop's amber-to-cyan border. |
| `svg/kubaos-icon-small.svg` | The icon for 16 and 32 px: no glow, fatter strokes. |
| `svg/kubaos-mark-white.svg`, `-black.svg` | Single colour. Stamps, monochrome tray icons, anywhere you get one colour. |
| `svg/kubaos-mark-flat.svg` | Flat two-colour drawing; source of the terminal logo. |
| `svg/kubaos-horizontal.svg` | Mark and wordmark side by side, transparent. The default lockup. |
| `svg/kubaos-horizontal-white.svg` | The same in white. |
| `svg/kubaos-horizontal-dark.svg` | The same on its own dark plate, for light pages. |
| `svg/kubaos-wordmark.svg` | "KubaOS" alone, for setting next to a mark you already have. |
| `svg/kubaos-stacked.svg` | Mark over wordmark. Square-ish spaces. |
| `svg/kubaos-splash.svg` | 1920x1080 boot splash: stacked logo and SHINJUKU on near-black. |
| `png/kubaos-icon-{16..1024}.png` | Rendered icon at 16, 32, 64, 128, 256, 512, 1024. |
| `png/kubaos-mark-{size}.png`, `-white-`, `-black-` | Rendered mark, same sizes. |
| `png/kubaos-horizontal*.png`, `png/kubaos-stacked*.png` | Lockups, 256 px high (stacked dark: 512). |
| `png/kubaos-splash-1920x1080.png` | Wallpaper-style splash still. The real boot splash and GRUB theme are drawn from `boot.py`'s assets. |
| `fastfetch.txt` | Half-block terminal logo with fastfetch colour slots. |

Change the drawing in `make.py`, then run `./render.sh` (needs python3,
`resvg` and ImageMagick). Don't hand-edit the SVGs or PNGs, they get
overwritten.

The boot assets (Plymouth splash and GRUB theme PNGs, plus the fonts both
use) come from `boot.py` and land in `system/files/{plymouth/kubaos,grub/kubaos,fonts}`:
`python3 branding/boot.py` (needs Pillow and `resvg`). The theme scripts
next to them (`kubaos.script`, `theme.txt`) are hand-written.

## Colours

From the default wallpaper's Material You scheme (`config/matugen/default-colors.json`):

| Role | Hex | Where |
|---|---|---|
| Amber (primary) | `#fdb882` | Mark tubes, "OS" |
| Tube core | `#ffe6cf` | The lit line inside the amber tubes |
| Cyan (tertiary) | `#8dd1e2` | Dakuten, icon border gradient end |
| Off-white (on_surface) | `#ece0da` | "Kuba" |
| Night (surface) | `#18120f` | Icon tile |
| Deep (surface_container_lowest) | `#120d0a` | Dark plates, splash |

On a light background use the dark plate version or the black mark. Don't
put the neon version on anything lighter than `#3a332f`: neon needs the dark.

## Clear space and size

- Keep empty space around the logo of at least the height of the バ dakuten
  tube, about a quarter of the mark's height. In the horizontal lockup that's
  the gap between the mark and the K.
- Smallest sizes: the icon down to 16 px (use `icon-small` below 48 px), the
  horizontal lockup down to 20 px high. Below that, use the mark alone.
- Don't stretch it, recolour the tubes, add a drop shadow, put it on a busy
  photo without a scrim, or set "KubaOS" in some other font next to the mark.

## fastfetch

`fastfetch.txt` uses `$1` for amber and `$2` for cyan:

```jsonc
"logo": {
  "type": "file",
  "source": "~/.config/fastfetch/kubaos.txt",
  "color": { "1": "38;2;253;184;130", "2": "38;2;141;209;226" }
}
```
