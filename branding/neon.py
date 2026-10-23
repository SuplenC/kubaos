#!/usr/bin/env python3
"""The neon-sign logo: every asset drawn from the photo in branding/photo/.

The photo is the logo wherever it shows large. Small sizes (icons, tray,
fastfetch) keep the vector mark from make.py. Writes:

  branding/png/kubaos-sign-1024.png       the sign cut out of the wall, transparent
  branding/png/kubaos-hero.jpg              the README banner: the photo, cropped
  branding/png/kubaos-splash-1920x1080.png  the sign on the dark, as the boot splash shows it
  system/files/plymouth/kubaos/sign-*.png   the switch-on layers, 1x and @2x
  system/files/grub/kubaos/logo.png         the lit sign over the boot menu

Run it after branding/photo/upscale.sh changes the master. Needs python3 with
Pillow and numpy:

  python3 branding/neon.py
"""

from pathlib import Path

import numpy as np
from PIL import Image

HERE = Path(__file__).resolve().parent
REPO = HERE.parent
MASTER = HERE / "photo/neon-sign-front@2x.jpg"
PNG = HERE / "png"
PLY = REPO / "system/files/plymouth/kubaos"
GRUB = REPO / "system/files/grub/kubaos"

# surface_container_lowest of config/matugen/default-colors.json: the boot
# splash's background, and what the cut-out is checked against.
DEEP = (0x14, 0x0D, 0x07)

# The photo's geometry, in the source photo's pixels (neon-sign-front.png,
# 1536 wide); the master is scaled from it. Lit glass is anything brighter
# than TUBE, and the empty columns between the groups fall at these x.
SOURCE_WIDTH = 1536
TUBE = 0.6
KU_END = 336
BA_END = 660
KUBA_END = 1183
# The lit glass spans these columns and rows.
TUBES_X = (94, 1457)
TUBES_Y = (343, 677)

# Wall darker than this is cut away; the warm glow the tubes throw on the
# wall around them stays, as part of the sign.
FLOOR = 0.12

# How far past the glass the cut-out reaches, and how wide its edges fade
# out, in source pixels. The photo ends 79 px right of the S, so the sides
# fade over the margin the photo has.
PAD_Y = 130
FADE_X = 70
FADE_Y = 90

# The switch-on order; kubaos.script strikes the groups in this order.
GROUPS = ("ku", "ba", "dakuten", "kuba", "os")
# How far a group's share of the wall glow reaches past its glass, in source
# pixels. One Gaussian would hand the glow from one group to the next over a
# few pixels (the ratio of two Gaussians is a steep logistic), which shows as
# a seam when one of them is dark. Summed over growing radii the tails fall
# off slowly, so the handover is as wide as the gap between the groups.
GLOW_SIGMAS = (45.0, 120.0, 300.0)
# Unlit glass: a cool grey at low alpha, so the dead sign shows faintly
# before it strikes.
DEAD_GLASS = (0.55, 0.58, 0.62)
DEAD_ALPHA = 0.16

# Display sizes, as the width of the lit glass in pixels.
PLYMOUTH_TUBES = 720
GRUB_TUBES = 880
SPLASH_TUBES = 860


def load_master() -> np.ndarray:
    return np.asarray(Image.open(MASTER).convert("RGB")).astype(np.float32) / 255


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


def cut_out(im: np.ndarray) -> tuple[np.ndarray, np.ndarray]:
    """The sign without the wall: straight colour and alpha from luminance.
    Over a wall of grey FLOOR it gives back the photo; over the dark it is the
    sign glowing on nothing."""
    lum = im.max(axis=2)
    a = np.clip((lum - FLOOR) / (1 - FLOOR), 0, 1)
    safe = np.maximum(a, 1e-4)[..., None]
    rgb = np.clip((im - FLOOR * (1 - a[..., None])) / safe, 0, 1)
    return rgb, a


def tube_groups(im: np.ndarray, k: float) -> dict[str, np.ndarray]:
    """Which lit glass belongs to which group. The groups sit in their own
    columns, except the dakuten over バ, which is the only blue glass."""
    h, w, _ = im.shape
    lit = im.max(axis=2) > TUBE
    blue = im[..., 2] > im[..., 0]
    x = np.broadcast_to(np.arange(w, dtype=np.float32)[None, :] / k, (h, w))
    return {
        "ku": lit & ~blue & (x < KU_END),
        "ba": lit & ~blue & (x >= KU_END) & (x < BA_END),
        "dakuten": lit & blue & (x < BA_END),
        "kuba": lit & (x >= BA_END) & (x < KUBA_END),
        "os": lit & (x >= KUBA_END),
    }


def glass_mask(tubes: dict[str, np.ndarray], sigma: float) -> np.ndarray:
    """Every group's glass, softened by `sigma` so its edge isn't jagged."""
    return np.clip(blur(sum(t.astype(np.float32) for t in tubes.values()), sigma) * 2, 0, 1)


def group_shares(tubes: dict[str, np.ndarray], k: float) -> dict[str, np.ndarray]:
    """Each group's share of every pixel's light, summing to one.

    On the glass the share is the group that owns it. Off the glass it comes
    from each group's glass blurred wide, so the boundary between two groups
    follows the tubes' shapes and blends over tens of pixels.
    """
    glass = glass_mask(tubes, 1.5 * k)
    sharp = {g: blur(t.astype(np.float32), 1.5 * k) for g, t in tubes.items()}
    wide = {g: sum(blur(t.astype(np.float32), s * k) for s in GLOW_SIGMAS) for g, t in tubes.items()}
    eps = 1e-6
    sharp_sum = sum(sharp.values()) + eps
    wide_sum = sum(wide.values()) + eps
    return {
        g: glass * sharp[g] / sharp_sum + (1 - glass) * (wide[g] + eps / len(GROUPS)) / wide_sum
        for g in tubes
    }


def crop_box(k: float) -> tuple[int, int, int, int]:
    return (0, int((TUBES_Y[0] - PAD_Y) * k), int(SOURCE_WIDTH * k), int((TUBES_Y[1] + PAD_Y) * k))


def edge_fade(h: int, w: int, k: float) -> np.ndarray:
    """1 inside, easing to 0 at the crop's edges, so no edge of the image shows
    on the screen."""

    def ramp(n: int, fade: float) -> np.ndarray:
        d = np.minimum(np.arange(n), np.arange(n)[::-1]).astype(np.float32)
        t = np.clip(d / fade, 0, 1)
        return t * t * (3 - 2 * t)

    return ramp(h, FADE_Y * k)[:, None] * ramp(w, FADE_X * k)[None, :]


class Sign:
    """The cut-out sign and its switch-on layers, at the master's size."""

    def __init__(self) -> None:
        im = load_master()
        self.k = im.shape[1] / SOURCE_WIDTH
        x0, y0, x1, y1 = crop_box(self.k)
        im = im[y0:y1, x0:x1]
        tubes = tube_groups(im, self.k)
        self.rgb, a = cut_out(im)
        self.alpha = a * edge_fade(*a.shape, self.k)
        self.shares = group_shares(tubes, self.k)
        self.glass = glass_mask(tubes, 1.0 * self.k)

    def tubes_width(self) -> float:
        return (TUBES_X[1] - TUBES_X[0]) * self.k

    def lit(self) -> np.ndarray:
        """The sign fully lit, as RGBA in 0..1."""
        return np.dstack([self.rgb, self.alpha])

    def dead(self) -> np.ndarray:
        rgb = np.broadcast_to(np.array(DEAD_GLASS, np.float32), self.rgb.shape)
        return np.dstack([rgb, self.glass * DEAD_ALPHA])

    def layer(self, group: str) -> np.ndarray:
        return np.dstack([self.rgb, self.alpha * self.shares[group]])

    def layers(self) -> dict[str, np.ndarray]:
        """The dead glass, then one layer per group, in drawing order."""
        return {"base": self.dead()} | {g: self.layer(g) for g in GROUPS}


def over(dst: np.ndarray, src: np.ndarray, opacity: float = 1.0) -> np.ndarray:
    """`src` drawn over `dst`, both straight-alpha RGBA, the way Plymouth draws
    one sprite over another."""
    sa = src[..., 3:] * opacity
    da = dst[..., 3:]
    out_a = sa + da * (1 - sa)
    out_rgb = (src[..., :3] * sa + dst[..., :3] * da * (1 - sa)) / np.maximum(out_a, 1e-6)
    return np.dstack([out_rgb, out_a])


def merged(layers: dict[str, np.ndarray]) -> np.ndarray:
    """Every layer drawn at full opacity. The lit sprite the script shows once
    the sign is on is exactly this, so swapping to it doesn't pop."""
    out = np.zeros_like(next(iter(layers.values())))
    for img in layers.values():
        out = over(out, img)
    return out


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
    go into the initramfs: this cuts them from 7.5 MB to under 2 MB, and the
    dither disappears into the wall's texture. Plymouth's PNG loader expands
    palettes and tRNS; GRUB's doesn't, so GRUB's logo stays truecolour."""
    q = img.quantize(256, method=Image.Quantize.FASTOCTREE, dither=Image.Dither.FLOYDSTEINBERG)
    save(q, path)


def branding(sign: Sign, lit: np.ndarray) -> None:
    save(resized(lit, 1024 / lit.shape[1]), PNG / "kubaos-sign-1024.png")
    splash = resized(lit, SPLASH_TUBES / sign.tubes_width())
    save(on_deep(splash, (1920, 1080)), PNG / "kubaos-splash-1920x1080.png")
    hero()


def hero() -> None:
    """The README banner: the photo itself, wall and all, cropped wide."""
    im = Image.open(MASTER).convert("RGB")
    k = im.width / SOURCE_WIDTH
    banner = im.crop((0, int(200 * k), im.width, int(840 * k))).resize((1600, 667), Image.LANCZOS)
    PNG.mkdir(parents=True, exist_ok=True)
    banner.save(PNG / "kubaos-hero.jpg", quality=85, optimize=True, progressive=True)


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
    plymouth(sign, layers, merged(layers))
    grub(sign, sign.lit())
    print(f"wrote {PNG.relative_to(REPO)}, {PLY.relative_to(REPO)} and {GRUB.relative_to(REPO)}")
