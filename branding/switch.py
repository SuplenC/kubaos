#!/usr/bin/env python3
"""The neon sign switching on and off, as clips with no background.

Cut from the stills in branding/photo/neon-sign-states/: the sign dark, then
with one more group of tubes lit in each. Between two stills the group strikes
as a whole tube, stuttering between its dark and its lit still the way neon
does; the film the stills come from strikes with light crawling along the
glass, which this sign doesn't do. Writes:

  branding/video/kubaos-sign-on.webm    dark to lit, a group at a time
  branding/video/kubaos-sign-off.webm   lit to dark, the last group first

Both are 1920x1080 at 60 frames a second, VP9 with alpha: the sign and its
glow over whatever colour is put under them. The glow is light, so that
colour has to be a dark one.

Needs python3 with Pillow and numpy, and ffmpeg with libvpx:

  python3 branding/switch.py
"""

import subprocess
from pathlib import Path

import numpy as np
from PIL import Image

HERE = Path(__file__).resolve().parent
STATES = HERE / "photo" / "neon-sign-states"
VIDEO = HERE / "video"

# The switch-on order, and the still that has each group lit.
GROUPS = ("ku", "ba", "dakuten", "kuba", "os")
FPS = 60
# How long a strike holds each of its levels, in ms.
STEP = 1000 / 30
# A group's light while it strikes, and while it dies, a level per step.
STRIKE = (0.5, 0.0, 0.0, 0.8, 0.1, 0.0, 1.0, 0.3, 0.9, 0.0, 1.0, 0.7, 1.0, 0.9)
DIE = (1.0, 0.6, 1.0, 0.2, 0.9, 0.0, 0.5, 0.0, 0.25, 0.0)
# When each group strikes, in ms, and how long the lit sign then holds.
ON_AT = (300, 650, 1100, 1450, 1900)
ON_HOLD = 1200
# The lit sign holds, then a group goes out every OFF_STEP ms, and the dark
# sign holds.
OFF_FROM = 500
OFF_STEP = 220
OFF_HOLD = 700
# The film's black is not quite black: below this a pixel is clear.
FLOOR = 2 / 255
FAINT = 0.1
# How far in from the frame's edges the picture fades out, as a part of its
# size, so no edge shows over a colour.
EDGE = 0.05


def still(name: str) -> np.ndarray:
    return np.asarray(Image.open(STATES / f"{name}.jpg").convert("RGB")).astype(np.float32) / 255


def level(pattern: tuple[float, ...], start: float, rest: float, t: float) -> float:
    """A group's light at `t`: `rest` until it starts, the pattern's levels,
    then the other end."""
    since = t - start
    if since < 0:
        return rest
    k = int(since / STEP)
    return pattern[k] if k < len(pattern) else 1 - rest


def edge_fade(h: int, w: int) -> np.ndarray:
    def ramp(n: int) -> np.ndarray:
        t = np.clip(np.minimum(np.arange(n), np.arange(n)[::-1]) / (EDGE * n), 0, 1)
        return t * t * (3 - 2 * t)

    return ramp(h)[:, None] * ramp(w)[None, :]


def cut_out(im: np.ndarray, fade: np.ndarray) -> bytes:
    """Light on black as straight-alpha RGBA: alpha is the brightest channel
    above the black, so over black it gives the picture back."""
    a = np.clip((im.max(axis=2) - FLOOR) / (1 - FLOOR), 0, 1)
    # Colour is the picture over its alpha, which blows the film's noise up
    # to full range in the faintest glow and costs the clip most of its size:
    # under FAINT the division stops, and the glow's last trace fades a little
    # sooner.
    rgb = np.clip((im - FLOOR * (1 - a[..., None])) / np.maximum(a, FAINT)[..., None], 0, 1)
    px = (np.dstack([rgb, a * fade]) * 255 + 0.5).astype(np.uint8)
    px[px[..., 3] == 0] = 0
    return px.tobytes()


def clip(name: str, length: float, levels) -> None:
    stills = [still("off")] + [still(g) for g in GROUPS]
    h, w, _ = stills[0].shape
    fade = edge_fade(h, w)
    out = VIDEO / f"kubaos-sign-{name}.webm"
    # -auto-alt-ref 0: libvpx drops the alpha plane with alternate reference
    # frames on.
    cmd = ["ffmpeg", "-v", "error", "-y", "-f", "rawvideo", "-pix_fmt", "rgba", "-s", f"{w}x{h}", "-r", str(FPS)]
    cmd += ["-i", "-", "-c:v", "libvpx-vp9", "-pix_fmt", "yuva420p", "-crf", "32", "-b:v", "0"]
    cmd += ["-row-mt", "1", "-auto-alt-ref", "0", str(out)]
    with subprocess.Popen(cmd, stdin=subprocess.PIPE) as ffmpeg:
        for i in range(int(length * FPS / 1000)):
            frame = stills[0].copy()
            for g, on in enumerate(levels(i * 1000 / FPS)):
                frame += on * (stills[g + 1] - stills[g])
            ffmpeg.stdin.write(cut_out(np.clip(frame, 0, 1), fade))
        ffmpeg.stdin.close()
        if ffmpeg.wait():
            raise SystemExit(f"ffmpeg failed on {out.name}")


if __name__ == "__main__":
    VIDEO.mkdir(exist_ok=True)
    groups = range(len(GROUPS))
    off_at = [OFF_FROM + (len(GROUPS) - 1 - g) * OFF_STEP for g in groups]
    clip("on", ON_AT[-1] + len(STRIKE) * STEP + ON_HOLD, lambda t: [level(STRIKE, ON_AT[g], 0, t) for g in groups])
    clip("off", off_at[0] + len(DIE) * STEP + OFF_HOLD, lambda t: [level(DIE, off_at[g], 1, t) for g in groups])
    print(f"wrote {VIDEO.relative_to(HERE.parent)}")
