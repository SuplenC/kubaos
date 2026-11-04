# Backdrop

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Backdrop`

QML `Backdrop` (polkit's prompt) and Rust `layout::backdrop` (Kōbō's welcome, done and step-rail screens), a template. Its mockup page is the source of truth: Templates, Backdrop, in `design/mockups/`. The lock screen is a template of its own, [Lock](lock.md). *Not built yet: polkit (`modules/polkit/Polkit.qml`) and Kōbō (`crates/kubaos-kobo/src/view.rs`) each draw their own picture, scrim and layout.*

A full-screen moment: a picture, a scrim over it, and one thing in its middle, centred on whole device pixels with the odd one after it.

- **The picture** is Kōbō's wallpaper; polkit lies over the desktop it asks on and has none of its own. Before it has arrived the screen is `surface_container_lowest`, and it fades in over `slow`.
- **The scrim** is one of two. Behind words (Kōbō's welcome and done): a band of `surface` at `opacity.backdrop` across the screen, solid from `space.s64` above the thing to `space.s64` below it and fading to nothing over those `space.s64`, so every text role keeps 4.5:1 over any picture. Behind a dialog or a panel (polkit, Kōbō's step-rail screens): `scrim` at `opacity.scrim` over all of it.
- **The thing**: polkit's `Dialog`; Kōbō's welcome or done column, `size.column` wide, its words centred; or Kōbō's glass panel, `space.s32` in from the screen's edges (`space.s16` under two columns' width), no wider than a rail and two columns with `space.s40` beside each, its blurred copy of the picture under `surface` at `opacity.glass`, a soft hairline inside its edge, at the surface radius, casting `shadow.panel`, which StepRailPage fills.
- **A short room:** where the screen is shorter than the thing and `space.s32` above and below it, the parts marked as giving way go first (Kōbō's sign), so the words and the action always show.

Motion: it arrives over `scene` on `emphasized_decel`, the picture and scrim fading in and the thing fading in as it rises `space.s24` into place, and lifts over `scene` on `emphasized_accel`, fading off what is under it while the thing rises `space.s24` on. A prompt over the desktop arrives as a `Dialog` does, over `long`. With reduce motion each fades, capped at `short`, with no rise.

Keys: Enter answers or takes the screen's `Filled` action; Esc is polkit's Cancel and does nothing on Kōbō's welcome and done; Tab goes round the thing's controls and never leaves it. Arriving lands on polkit's field or Kōbō's action; lifting gives the keyboard back to what had it.
