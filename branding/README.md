# KubaOS branding

The logo is a neon sign: クバ in amber tubes, the dakuten as a blue tube on
the same sign, then "Kuba" in white and "OS" in amber, on a dark wall. The
photo of it (`photo/`) is the logo wherever it shows large: the boot splash,
where it switches on, the GRUB menu, Kōbō's welcome screen, the README.

Small sizes can't carry a photo, so they use the vector mark: クバ drawn as
neon strokes in `make.py`, with the wordmark in a geometric round-stroke
alphabet. App icons from 16 to 256 px, the single-colour marks, the tray and
fastfetch all come from it. Nothing here uses a font. Every glyph is a stroked
path, so the SVGs render the same everywhere.

Rules for the name and the voice are in `docs/BRAND.md`.

## The photo

`photo/neon-sign-front.png` is the sign, front-on. It was generated with
ChatGPT for KubaOS and is released under CC0. `photo/neon-sign-front@2x.jpg`
is the master everything is drawn from: the photo upscaled with Upscayl's
remacri model (`photo/upscale.sh`). The angled shot of the same sign is the
default wallpaper (`shinjuku/kubaos-neon-sign.jpg` in kubaos-wallpapers).

`neon.py` cuts the sign out of the wall: alpha from brightness, so the tubes
and the glow they throw on the wall stay and the dark wall goes. For the boot
splash it also splits the sign into its five groups of tubes, in switch-on
order: ク, バ, the dakuten, Kuba, OS. Each group owns the glass it lights.
Its share of the wall glow comes from its glass blurred wide, so a lit group
next to a dark one fades out across the wall instead of ending in a seam.

## Files

| File | Use it for |
|---|---|
| `photo/neon-sign-front.png` | The source photo. |
| `photo/neon-sign-front@2x.jpg` | The upscaled master, 3072x2048. |
| `png/kubaos-sign-1024.png` | The sign cut out of the wall, transparent, 1024x396. Anywhere the logo is big, on the dark. |
| `png/kubaos-hero.jpg` | The README banner: the photo, wall and all, 1600x667. |
| `png/kubaos-splash-1920x1080.png` | The sign on the dark, as the boot splash shows it once it's on. |
| `svg/kubaos-mark.svg` | The vector mark alone, neon, transparent background. About boxes, anything too small for the photo. |
| `svg/kubaos-icon.svg` | App icon: the mark on a dark rounded tile with the desktop's amber-to-blue border. |
| `svg/kubaos-icon-small.svg` | The icon for 16 and 32 px: no glow, fatter strokes. |
| `svg/kubaos-mark-white.svg`, `-black.svg` | Single colour. Stamps, monochrome tray icons, anywhere you get one colour. |
| `svg/kubaos-mark-flat.svg` | Flat two-colour drawing; source of the terminal logo. |
| `svg/kubaos-horizontal.svg` | Vector mark and wordmark side by side, transparent. The small lockup. |
| `svg/kubaos-horizontal-white.svg` | The same in white. |
| `svg/kubaos-horizontal-dark.svg` | The same on its own dark plate, for light pages. |
| `svg/kubaos-wordmark.svg` | "KubaOS" alone, for setting next to a mark you already have. |
| `svg/kubaos-stacked.svg` | Mark over wordmark. Square-ish spaces. |
| `png/kubaos-icon-{16..1024}.png` | Rendered icon at 16, 32, 64, 128, 256, 512, 1024. |
| `png/kubaos-mark-{size}.png`, `-white-`, `-black-` | Rendered mark, same sizes. |
| `png/kubaos-horizontal*.png`, `png/kubaos-stacked*.png` | Lockups, 256 px high (stacked dark: 512). |
| `fastfetch.txt` | Half-block terminal logo with fastfetch colour slots. |

The boot assets land in `system/files/`. From `neon.py`: the Plymouth sign
layers (`plymouth/kubaos/sign-{base,ku,ba,dakuten,kuba,os,lit}{,@2x}.png`,
2 MB in all, palette PNGs because they ride in the initramfs) and GRUB's
`grub/kubaos/logo.png`. From `boot.py`: the passphrase field in its states,
its icons, bullet, caret, placeholder and notes, GRUB's boxes, background and
key hints, and the fonts both use, all drawn from the design tokens. The
values the theme files read (`kubaos.script`'s token block, `kubaos.plymouth`,
GRUB's `theme.txt` from `system/files/grub/theme.txt.in`) are written by
`cargo run -p kubaos-tokens -- write`, which measures `boot.py`'s pictures:
run it after `render.sh`. Kōbō carries copies of the mark, the wordmark and
`kubaos-sign-1024.png` (`crates/kubaos-kobo/assets/make-assets.sh`).

## Regenerating

Change the drawing in `make.py` or the cut in `neon.py`, then run:

```sh
PYTHON=python3 branding/render.sh
```

It runs `make.py`, renders the SVGs, writes `fastfetch.txt`, then runs
`neon.py` and `boot.py`. It needs `resvg`, ImageMagick and a python3 (3.11 or
later, for `tomllib`) with Pillow and numpy (`PYTHON` picks it). Only a new source photo needs
`photo/upscale.sh` first, which needs Upscayl. Don't hand-edit the SVGs or
PNGs, they get overwritten.

## Colours

The vector marks, Kōbō and the boot screens take their colours from the
design tokens' dark palette (`design/tokens.toml`, `[color.dark]`: matugen's
scheme for the default wallpaper, the angled photo of this sign), and the
mark's strokes and tube widths from `[neon.mark]`, which Kōbō's done screen
draws from too:

| Role | Where |
|---|---|
| `primary` (amber) | Mark tubes, "OS" |
| `cold` (blue) | Dakuten, icon border gradient end |
| `on_surface` (off-white) | "Kuba" |
| `surface` | Icon tile |
| `surface_container_lowest` | Dark plates, the boot splash and GRUB |
| `logo_core_warm`, `logo_core_cold` | The lit core inside the amber and the blue tubes |

The photo keeps its own colours.

On a light background use the dark plate version or the black mark. Don't
put the neon (photo or vector) on anything lighter than `#3d332b`: neon needs
the dark.

## Clear space and size

- Keep empty space around the logo of at least the height of the バ dakuten
  tube, about a quarter of the mark's height. In the horizontal lockup that's
  the gap between the mark and the K. The sign's cut-out already carries its
  glow as margin.
- Smallest sizes: the sign down to 400 px wide; below that, the vector
  lockup. The icon goes down to 16 px (use `icon-small` below 48 px), the
  horizontal lockup down to 20 px high. Below that, use the mark alone.
- Don't stretch it, recolour the tubes, add a drop shadow, put it on a busy
  photo without a scrim, or set "KubaOS" in some other font next to the mark.

## fastfetch

`fastfetch.txt` uses `$1` for amber and `$2` for blue:

```jsonc
"logo": {
  "type": "file",
  "source": "~/.config/fastfetch/kubaos.txt",
  "color": { "1": "38;2;255;183;121", "2": "38;2;133;207;255" }
}
```
