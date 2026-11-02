# Toast

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Toast`

Rust `toast::toast`, `toast::toast_at`; QML `Toast`. Its mockup page is the source of truth: Molecules, Toast, in `design/mockups/`.

One `body` line saying what just happened, in the inverse colours (`surface` on `on_surface`) so it reads over anything, an optional 20 symbol `space.s12` before the words, padding 12, the control radius (12), a small floating thing's. It floats over its window or panel, raised to the ladder's step 2 as a popover and a menu are (`docs/DESIGN.md`, "Elevation"): step 2's contact and ambient shadow, its fill the inverse `on_surface` in both modes, and the rim in `surface`, the colour of its words, since a rim of `on_surface` would be its fill. It is as wide as what it says, up to its room less `space.s24` at either side; past that its words wrap. It confirms something just done that left no other trace (`docs/DESIGN.md`, "Notify or show inline"); it never asks for anything and never takes the keyboard. Where what it says can be undone, the surface's own key for it shows as key hints `space.s16` after the words (`Ctrl` `Z` Undo), in `surface`.

**Where.** At the bottom of the surface it confirms in (Kōbō's window, the Command menu's list), centred across it on a whole device pixel, `space.s24` above the edge, over what is there.

**Motion.** It arrives over `short`, rising 32 into place a whole device pixel at a time as it fades in (`emphasized_decel`), stays `duration.toast` (3.6 s), and leaves over `medium`, fading where it is (`standard_accel`): `toast::toast_at` draws it at an age. Its shadow fades in and out with it, never on its own. While the pointer is on it, it stays, and counts its stay again once the pointer leaves. One shows at a time: a new one takes the place of the one showing, which fades out over `micro`. With reduce motion it fades in with no rise, and the fades are capped at `short`; the stay isn't.

*Not built yet: `Toast` in QML fades in over `short` on `standard` with no rise, the Command menu writes its undo key as words ("⌃Z undoes it."), nothing holds a toast under the pointer, `kubaos-ui` keeps its stay as `toast::DEFAULT_STAY` rather than `duration.toast`, and both libraries draw it flat, 16 × 12 inside at the surface radius (24); the mockup draws the rule.*
