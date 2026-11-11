#!/usr/bin/env python3
"""The neon-sign logo: every asset drawn from the photos in branding/photo/.

The photos are the logo wherever it shows large. Small sizes (icons, tray,
fastfetch) keep the vector mark from make.py. Writes:

  branding/png/kubaos-sign-1024.png          the sign cut out of the wall, transparent
  branding/png/kubaos-sign-{mark,stacked}-1024.png  クバ alone, and over the name, the same
  branding/png/kubaos-hero.jpg               the README banner, from its own photo
  branding/png/kubaos-splash-1920x1080.png   the sign on the dark, as the boot splash shows it
  system/files/plymouth/kubaos/sign-*.png    the switch-on layers, 1x and @2x
  system/files/grub/kubaos/logo.png          the lit sign over the boot menu

Needs python3 with Pillow and numpy:

  python3 branding/neon.py
"""

import tomllib
from pathlib import Path

import numpy as np
from PIL import Image

HERE = Path(__file__).resolve().parent
REPO = HERE.parent
PHOTO = HERE / "photo"
# The sign lit, and the same sign switched off, every tube where the lit one
# has it: the boot splash's dead glass is the off photo.
LIT = PHOTO / "neon-sign-lit.jpg"
OFF = PHOTO / "neon-sign-off.jpg"
BANNER = PHOTO / "neon-sign-banner.jpg"
# The same sign filmed switching on: one still per group, each with that group
# and the ones before it lit, 1920x1080 with the sign across the full width.
# They say how the light spreads as the groups strike, a lit tube's shine on
# the dead glass beside it included.
STATES = PHOTO / "neon-sign-states"
PNG = HERE / "png"
PLY = REPO / "system/files/plymouth/kubaos"
GRUB = REPO / "system/files/grub/kubaos"

# surface_container_lowest of the dark palette (design/tokens.toml): the boot
# splash's background, and what the cut-out is checked against.
with open(REPO / "design/tokens.toml", "rb") as f:
    _deep = tomllib.load(f)["color"]["dark"]["surface_container_lowest"].lstrip("#")
DEEP = tuple(int(_deep[i : i + 2], 16) for i in (0, 2, 4))

# The photo's geometry, in its pixels (3136x1344). Lit glass is anything
# brighter than TUBE; it spans these columns and rows.
TUBE = 0.6
TUBES_X = (322, 2797)
TUBES_Y = (393, 898)
# Wall darker than this is cut away; the warm glow the tubes throw on the
# wall around them stays, as part of the sign.
FLOOR = 0.05

# How far past the glass the cut-out reaches, and how wide its edges fade
# out, in the photo's pixels. The photo ends 339 px right of the S, so the
# sides fade inside the margin it has.
PAD_Y = 260
FADE_X = 145
FADE_Y = 185

# The switch-on order; kubaos.script strikes the groups in this order.
GROUPS = ("ku", "ba", "dakuten", "kuba", "os")
# The stills are a film's frames and carry its noise; blurred by this, in the
# photo's pixels, what is left is the light.
STATE_BLUR = 4.0
# Light a group adds below this says nothing about whose it is: there the
# share falls back to the group's place in the order.
STATE_FLOOR = 0.02
# How alike the light a pixel has gained and the colour it burns have to be,
# as the cosine between the two colours, to count as its own: the film's
# white-hot glass against the photo's is over 0.95 and counts in full, amber
# falling on the dakuten's blue glass is under 0.5 and doesn't count.
UNLIKE = 0.55
ALIKE = 0.9
# The dead glass is the off photo at this brightness, so the sign shows
# faintly before it strikes instead of standing out as a grey sign.
DEAD_LIGHT = 0.35
# In the off photo, glass is brighter than this: three quarters of the dead
# tubes are over 0.4, and the wall right beside them stays under 0.17.
DEAD_GLASS = 0.2

# Display sizes, as the width of the lit glass in pixels.
PLYMOUTH_TUBES = 720
SPLASH_TUBES = 860
# GRUB's at its scale (design/tokens.toml, `boot.grub_scale`), smaller than
# the splash's, so the menu fits under it on a 1080p screen.
with open(REPO / "design/tokens.toml", "rb") as f:
    GRUB_TUBES = 440 * tomllib.load(f)["boot"]["grub_scale"]


def load(path: Path) -> np.ndarray:
    return np.asarray(Image.open(path).convert("RGB")).astype(np.float32) / 255


def box_blur(m: np.ndarray, r: int, axis: int) -> np.ndarray:
    pad = [(0, 0), (0, 0)]
    pad[axis] = (r + 1, r)
    c = np.cumsum(np.pad(m, pad, mode="edge"), axis=axis, dtype=np.float64)
    hi = np.take(c, range(2 * r + 1, c.shape[axis]), axis=axis)
    lo = np.take(c, range(0, c.shape[axis] - 2 * r - 1), axis=axis)
    return ((hi - lo) / (2 * r + 1)).astype(np.float32)


def blur(m: np.ndarray, sigma: float) -> np.ndarray:
    """A Gaussian blur of a float mask: three box passes, the usual
    approximation. Wide blurs run at a reduced size, since their result is
    smooth anyway."""
    h, w = m.shape
    f = max(1, int(sigma // 6))
    small = np.asarray(Image.fromarray(m).resize((max(w // f, 1), max(h // f, 1)), Image.BOX))
    s = sigma / f
    # Three passes of width 2r+1 add up to the Gaussian's variance.
    r = max(1, int(round((np.sqrt(4 * s * s + 1) - 1) / 2)))
    for _ in range(3):
        small = box_blur(box_blur(small, r, 0), r, 1)
    if f == 1:
        return small
    # Bicubic overshoots below zero next to an edge; a share must not.
    return np.maximum(np.asarray(Image.fromarray(small).resize((w, h), Image.BICUBIC)), 0)


def cut_out(im: np.ndarray, floor: float = FLOOR) -> tuple[np.ndarray, np.ndarray]:
    """The sign without the wall: straight colour and alpha from luminance.
    Over a wall of grey `floor` it gives back the photo; over the dark it is
    the sign glowing on nothing."""
    lum = im.max(axis=2)
    a = np.clip((lum - floor) / (1 - floor), 0, 1)
    safe = np.maximum(a, 1e-4)[..., None]
    rgb = np.clip((im - floor * (1 - a[..., None])) / safe, 0, 1)
    return rgb, a


def state(name: str, like: np.ndarray) -> np.ndarray:
    """A still of the switch-on, laid over the photo: the stills hold the photo
    at their width, centred, so the middle rows scaled back up are its pixels."""
    im = Image.open(STATES / f"{name}.jpg").convert("RGB")
    h, w, _ = like.shape
    rows = round(im.width * h / w)
    top = (im.height - rows) // 2
    im = im.crop((0, top, im.width, top + rows)).resize((w, h), Image.BICUBIC)
    px = np.asarray(im).astype(np.float32) / 255
    return np.dstack([blur(px[..., c], STATE_BLUR) for c in range(3)])


def hue(rgb: np.ndarray) -> np.ndarray:
    """Colour without brightness, as a direction."""
    return rgb / (np.linalg.norm(rgb, axis=2, keepdims=True) + 1e-4)


def lit_so_far(lit: np.ndarray, off: np.ndarray) -> dict[str, np.ndarray]:
    """How much of each pixel's light is on once a group has struck, from none
    to all: what the still with that group lit has gained over the dark one,
    out of what the fully lit one has. It follows the film, so a lit group's
    glow carries on past a dark neighbour instead of stopping short of it.

    The light a still has gained only counts where it is the colour the pixel
    burns in the photo: amber falling on the dakuten's dead glass is not the
    dakuten striking, and would tint it blue before it does."""
    dark = state("off", lit)
    burns = hue(np.dstack([blur(c, STATE_BLUR) for c in np.moveaxis(np.clip(lit - off, 0, None), 2, 0)]))
    full = np.clip(state(GROUPS[-1], lit) - dark, 0, None).sum(axis=2)
    out, so_far = {}, np.zeros_like(full)
    for i, g in enumerate(GROUPS):
        gained = np.clip(state(g, lit) - dark, 0, None)
        t = np.clip(((hue(gained) * burns).sum(axis=2) - UNLIKE) / (ALIKE - UNLIKE), 0, 1)
        alike = t * t * (3 - 2 * t)
        share = (gained.sum(axis=2) * alike + STATE_FLOOR * (i + 1) / len(GROUPS)) / (full + STATE_FLOOR)
        # A group never takes light back from the ones before it, and the last
        # one leaves the sign fully lit.
        so_far = np.maximum(so_far, np.clip(share, 0, 1)) if i < len(GROUPS) - 1 else np.ones_like(full)
        out[g] = so_far
    return out


def edge_fade(h: int, w: int, fade_x: float, fade_y: float) -> np.ndarray:
    """1 inside, easing to 0 at the crop's edges, so no edge of the image shows
    on the screen."""

    def ramp(n: int, fade: float) -> np.ndarray:
        d = np.minimum(np.arange(n), np.arange(n)[::-1]).astype(np.float32)
        t = np.clip(d / fade, 0, 1)
        return t * t * (3 - 2 * t)

    return ramp(h, fade_y)[:, None] * ramp(w, fade_x)[None, :]


class Sign:
    """The cut-out sign and its switch-on layers, at the photo's size."""

    def __init__(self) -> None:
        rows = slice(TUBES_Y[0] - PAD_Y, TUBES_Y[1] + PAD_Y)
        lit, off = load(LIT), load(OFF)
        self.rgb, a = cut_out(lit[rows])
        self.fade = edge_fade(*a.shape, FADE_X, FADE_Y)
        self.alpha = a * self.fade
        self.lit_so_far = {g: m[rows] for g, m in lit_so_far(lit, off).items()}
        self.off = off[rows]

    def tubes_width(self) -> float:
        return TUBES_X[1] - TUBES_X[0]

    def lit(self) -> np.ndarray:
        """The sign fully lit, as RGBA in 0..1."""
        return np.dstack([self.rgb, self.alpha])

    def dead(self) -> np.ndarray:
        """The off photo, dimmed, cut from its wall the way the lit one is, and
        kept to its glass: the off photo's wall still carries a trace of the lit
        sign's glow (blue round the dakuten), which dead glass doesn't throw."""
        rgb, a = cut_out(self.off * DEAD_LIGHT, FLOOR * DEAD_LIGHT)
        glass = np.clip(blur((self.off.max(axis=2) > DEAD_GLASS).astype(np.float32), 4.0) * 2, 0, 1)
        return np.dstack([rgb, a * glass * self.fade])

    def layers(self) -> dict[str, np.ndarray]:
        """The dead glass, then one layer per group, in drawing order.

        Each group's layer is what turns the sign with the groups before it
        lit into the sign with it lit too, drawn over them at full opacity.
        Layers that each carried their share of the light as alpha would stack
        darker than the photo wherever two groups share the glow, since two
        half-alpha layers over each other don't make a whole. Drawn in order,
        these add up to the lit sign, so the lit sprite the script swaps to
        once the sign is on doesn't pop. They carry no background of their
        own, so they do so over any colour the splash draws under them.
        """
        dead = premultiplied(self.dead())
        lit = premultiplied(self.lit())
        out = {"base": self.dead()}
        before = dead
        for g in GROUPS:
            after = dead + self.lit_so_far[g][..., None] * (lit - dead)
            out[g] = step(before, after)
            before = after
        return out


def premultiplied(rgba: np.ndarray) -> np.ndarray:
    """Colour times alpha: the form in which a mix of two layers, drawn over
    anything, is the mix of the two drawn over it."""
    return np.dstack([rgba[..., :3] * rgba[..., 3:], rgba[..., 3]])


def step(before: np.ndarray, after: np.ndarray) -> np.ndarray:
    """The straight-alpha layer that, drawn over `before`, gives `after`, both
    premultiplied. Its alpha is the least that gets there, so where nothing
    changes it is clear: enough to let the right amount of the background
    through, to carry the light the group adds, and to hide what the dead
    glass has more of than the lit (its grey's blue under an amber tube)."""
    was, now = before[..., :3], after[..., :3]
    a = np.clip(
        np.maximum.reduce(
            [
                (after[..., 3] - before[..., 3]) / np.maximum(1 - before[..., 3], 1e-6),
                ((now - was) / np.maximum(1 - was, 1e-6)).max(axis=2),
                np.where(was > now + 1 / 255, 1 - now / np.maximum(was, 1e-6), 0).max(axis=2),
            ]
        ),
        0,
        1,
    )
    rgb = (now - was * (1 - a[..., None])) / np.maximum(a, 1e-6)[..., None]
    return np.dstack([np.clip(rgb, 0, 1), a])


def to_image(rgba: np.ndarray) -> Image.Image:
    px = (np.clip(rgba, 0, 1) * 255 + 0.5).astype(np.uint8)
    # The colour under a fully transparent pixel never shows; zero compresses
    # better than the wall texture it would otherwise keep.
    px[px[..., 3] == 0] = 0
    return Image.fromarray(px)


def resized(rgba: np.ndarray, scale: float) -> Image.Image:
    """Scaled in premultiplied colour, so the dark fringe of a transparent
    pixel doesn't bleed into its neighbours."""
    h, w, _ = rgba.shape
    size = (round(w * scale), round(h * scale))
    pre = rgba.copy()
    pre[..., :3] *= pre[..., 3:]
    chans = [np.asarray(Image.fromarray(pre[..., c]).resize(size, Image.LANCZOS)) for c in range(4)]
    out = np.clip(np.dstack(chans), 0, 1)
    out[..., :3] /= np.maximum(out[..., 3:], 1e-6)
    return to_image(out)


def on_deep(img: Image.Image, size: tuple[int, int]) -> Image.Image:
    bg = Image.new("RGBA", size, DEEP + (255,))
    bg.alpha_composite(img, ((size[0] - img.width) // 2, (size[1] - img.height) // 2))
    return bg.convert("RGB")


def save(img: Image.Image, path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    img.save(path, optimize=True)


def save_indexed(img: Image.Image, path: Path) -> None:
    """Saved as a 256-colour palette with alpha, dithered. The Plymouth layers
    go into the initramfs, and this keeps all of them under 2 MB.
    libimagequant (pngquant's) picks the palette for colour and alpha
    together, so the glow's long fade stays smooth where an octree's palette
    steps it. Plymouth's PNG loader expands palettes and tRNS; GRUB's doesn't,
    so GRUB's logo stays truecolour."""
    q = img.quantize(256, method=Image.Quantize.LIBIMAGEQUANT, dither=Image.Dither.FLOYDSTEINBERG)
    save(q, path)


def branding(sign: Sign, lit: np.ndarray) -> None:
    save(resized(lit, 1024 / lit.shape[1]), PNG / "kubaos-sign-1024.png")
    splash = resized(lit, SPLASH_TUBES / sign.tubes_width())
    save(on_deep(splash, (1920, 1080)), PNG / "kubaos-splash-1920x1080.png")
    for lockup in ("mark", "stacked"):
        cut = lockup_cut(PHOTO / f"neon-{lockup}-lit.jpg")
        save(resized(cut, 1024 / cut.shape[1]), PNG / f"kubaos-sign-{lockup}-1024.png")
    hero()


def lockup_cut(path: Path) -> np.ndarray:
    """クバ alone or over the name, cut from its wall like the sign and cropped
    round its glass with room for the glow, as RGBA in 0..1."""
    im = load(path)
    ys, xs = np.nonzero(im.max(axis=2) > TUBE)
    # The glow reaches about a fifth of the glass's width past it.
    pad = round(0.2 * (xs.max() - xs.min()))
    h, w, _ = im.shape
    im = im[max(ys.min() - pad, 0) : min(ys.max() + pad, h), max(xs.min() - pad, 0) : min(xs.max() + pad, w)]
    rgb, a = cut_out(im)
    return np.dstack([rgb, a * edge_fade(*a.shape, 0.6 * pad, 0.6 * pad)])


def hero() -> None:
    """The README banner: its own photo, the sign on a long wall, cropped to
    the banner's shape."""
    im = Image.open(BANNER).convert("RGB")
    h = round(im.width * 667 / 1600)
    top = (im.height - h) // 2
    banner = im.crop((0, top, im.width, top + h)).resize((1600, 667), Image.LANCZOS)
    PNG.mkdir(parents=True, exist_ok=True)
    banner.save(PNG / "kubaos-hero.jpg", quality=88, optimize=True, progressive=True, subsampling=0)


def plymouth(sign: Sign, layers: dict[str, np.ndarray], lit: np.ndarray) -> None:
    for scale, suffix in ((1, ""), (2, "@2x")):
        f = PLYMOUTH_TUBES * scale / sign.tubes_width()
        for name, img in layers.items():
            save_indexed(resized(img, f), PLY / f"sign-{name}{suffix}.png")
        save_indexed(resized(lit, f), PLY / f"sign-lit{suffix}.png")


def grub(sign: Sign, lit: np.ndarray) -> None:
    save(resized(lit, GRUB_TUBES / sign.tubes_width()), GRUB / "logo.png")


if __name__ == "__main__":
    sign = Sign()
    layers = sign.layers()
    branding(sign, sign.lit())
    plymouth(sign, layers, sign.lit())
    grub(sign, sign.lit())
    print(f"wrote {PNG.relative_to(REPO)}, {PLY.relative_to(REPO)} and {GRUB.relative_to(REPO)}")
