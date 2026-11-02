# Backdrop

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Backdrop`

QML `Backdrop` (the lock screen, polkit's prompt) and Rust `layout::backdrop` (Kōbō's welcome, done and step-rail screens), a template. Its mockup page is the source of truth: Templates, Backdrop, in `design/mockups/`. *Not built yet: the lock (`modules/lock/LockSurface.qml`), polkit (`modules/polkit/Polkit.qml`) and Kōbō (`crates/kubaos-kobo/src/view.rs`) each draw their own picture, scrim and layout.*

A full-screen moment: a picture, a scrim over it, and one thing in its middle, centred on whole device pixels with the odd one after it.

- **The picture** is the wallpaper, Kōbō's own, or the lock's blurred copy of it; polkit lies over the desktop it asks on and has none of its own. Before it has arrived the screen is `surface_container_lowest`, and it fades in over `slow`.
- **The scrim** is one of two. Behind words (the lock, Kōbō's welcome and done): a band of `surface` at `opacity.backdrop` across the screen, solid from `space.s64` above the thing to `space.s64` below it and fading to nothing over those `space.s64`, so every text role keeps 4.5:1 over any picture. Behind a dialog or a panel (polkit, Kōbō's step-rail screens): `scrim` at `opacity.scrim` over all of it.
- **The thing**: the lock's column, `size.dialog` wide and `space.s64` above the middle (its accent, the clock in `clock`, the date `space.s8` under it, the `size.password` field `space.s48` under that, and its line `space.s8` under the field); polkit's `Dialog`; Kōbō's welcome or done column, `size.column` wide, its words centred; or Kōbō's glass panel, `space.s32` in from the screen's edges (`space.s16` under two columns' width), no wider than a rail and two columns with `space.s40` beside each, its blurred copy of the picture under `surface` at `opacity.glass`, a soft hairline inside its edge, at the surface radius, casting `shadow.panel`, which StepRailPage fills.
- **A short room:** where the screen is shorter than the thing and `space.s32` above and below it, the parts marked as giving way go first (Kōbō's sign), so the words and the action always show.

Motion: it arrives over `scene` on `emphasized_decel`, the picture and scrim fading in and the thing fading in as it rises `space.s24` into place, and lifts over `scene` on `emphasized_accel`, fading off what is under it while the thing rises `space.s24` on. A prompt over the desktop arrives as a `Dialog` does, over `long`. With reduce motion each fades, capped at `short`, with no rise. A wrong password shakes the lock's field (`space.s12`, back `space.s8`, `space.s4`, each over `micro`, then home over `short` on `standard_decel`).

Keys: Enter answers or takes the screen's `Filled` action; Esc empties the lock's field, is polkit's Cancel, and does nothing on Kōbō's welcome and done; Tab goes round the thing's controls and never leaves it; typing on the lock goes into its field. Arriving lands on the lock's or polkit's field, or Kōbō's action; lifting gives the keyboard back to what had it.
