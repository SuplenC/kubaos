# 5. Wallpaper depth: a background plate, so the city pushes in faster than the sky

Status: proposed (2026-11-04). Supersedes the part of ADR 0004 that says the lock needs no filled-in background, and its rejection of LaMa.

## Context

ADR 0004 has the lock draw three layers: the picture, the clock and the cut-out. It says a cut-out scaled at least as much as the picture only ever covers more of its own outline, so nothing behind it is revealed. That is wrong. A city's outline overhangs itself: Tokyo Tower's decks stand out over the shaft below them, and its legs arch. A cut-out grown about any origin moves off part of its own outline, and the copy of the city in the picture shows next to it. A probe measured doubled edges at a scale difference of 0.1%.

So with only a cut-out, the picture and the cut-out can only push in together, at one scale. The city then never comes forward, which was the point of the depth.

## Decision

**Each wallpaper with a cut-out also ships a background plate:** the picture with the cut-out's subject painted out by big-lama (LaMa, Apache-2.0). The painted area is the cut-out's alpha, grown a little past its soft edge. The plate keeps only that area and is transparent elsewhere, so it sits over the picture and never changes a pixel outside the subject. Like the cut-out, it is made once, offline, by the wallpapers repository's script, and checked by eye.

**The lock draws four layers**: the picture, the plate, the clock and the city. The city is the picture drawn through the cut-out's alpha, so picture, plate and city at one scale are the desktop's picture.
- **The lock motion:** the picture and the plate push in to `scale.push`, and the city to the stronger `scale.near`. All three grow about the bottom centre, where the city stands, so its roofs grow over the sky and the plate shows only in thin bands beside its towers. The plate fades in with the push, because inside the painted band it never equals the picture.
- **Unlocking** plays the motion in reverse, and its last frame is the desktop's picture. At rest nothing moves and nothing redraws.

**The Suite gains big-lama as its fourth model**, about 200 MB, so the models come to about 600 MB together. `kubaos-depth` makes the plate with the cut-out, in the same form the shipped files take. A picture with a cut-out but no plate pushes in at one scale.

## Considered options

- **Picture and cut-out at one scale, no plate.** Rejected: nothing reveals, but the city never comes forward.
- **A plate as a full picture.** Rejected: re-encoding changes every pixel, about 1.4% RMSE, so the swap at the lock's first frame would show. A patch over the picture leaves the rest exactly as it was.
- **Fading the cut-out in only after the push.** Rejected: the depth would play while nobody can see it.
