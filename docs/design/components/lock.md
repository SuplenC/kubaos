# Lock

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Lock`

QML `Lock`, a template of the shell's alone: only the shell locks, so it is in `KubaOS.Components` only, with its section on the QML specimen's "Shell only" page. Its mockup page is the source of truth: Templates, Lock, in `design/mockups/`. *Not built yet: `modules/lock/LockSurface.qml` draws the lock as a column over a band of `opacity.backdrop`, with the clock in `clock` and the `size.password` field under it, in the light or dark scheme of the mode.*

The lock screen. It has two phases, rest and awake, and always draws in the night scheme matugen derives from its wallpaper (`scheme-fidelity` in dark, as `dot-theme` makes it, with its derived roles), whatever the light or dark mode, so it looks the same in both.

**At rest** it is the wallpaper and its clock, and nothing else: no frame, no bar, no field. Every screen of a setup shows rest, each with its own clock.

- **The clock** is the digits in `type.clock_rest` (384, Geist Light, about 280 tall), `on_surface`, under a kicker line `space.s16` above the digits' tops: where and which day in `accent`, `primary` (東京・月曜日), then ` · ` and the date in `body_strong`, `on_surface_variant`. The digits' box is their cap height down to their baseline. The kicker and the digits are each centred on the clock's middle, on whole device pixels, the odd one after. With a 12-hour clock the digits carry no meridiem; it takes its own slot at the kicker's end (· PM), so the digits keep one size.
- **Its place** is the wallpaper's: each wallpaper names the middle of the digits' baseline as shares of the picture's width and height, so the clock keeps its place against the city however a screen crops the picture (`object-fit: cover`), held `space.s32` inside the screen's edges. A picture with no place of its own (yours) puts it in the middle, a little above.
- **The cut-out**, where the wallpaper has one, is its foreground cut out (an alpha picture of the city, the wallpaper's size), lying exactly over the picture and in front of the clock, so the towers stand in front of the digits. The place covers part of the digits and leaves them readable (Shinjuku's skyline covers their feet; Tokyo Tower's spire crosses the last digit) and never covers the kicker. A picture without one has the clock in front of it.
- **The shade** behind the clock is `surface` in an ellipse through the clock's corners and `space.s64` past them, blurred by `space.s32`, at the least opacity, in hundredths, that keeps every word of the clock at 4.5:1 over the brightest of the picture under it: its 98th percentile by luminance, read off the picture. A dark sky needs none and gets none. The cut-out stands in front of the shade too, so only the sky darkens.

**Awake** is the screen with the pointer only; the others stay at rest.

- **The frame** closes in round the picture as the shell draws it: `surface` at `opacity.frame`, `size.bar` at the top and `space.s8` down the sides and along the bottom, its inner corners at `radius.surface`. Unlocking leaves it where it is, for the desktop's.
- **The picture** blurs to `size.lock_blur` (8) and dims under `scrim` at `opacity.panel_scrim`, as a panel's desktop does; the blur holds its edges, so it reaches the screen's.
- **The clock** lifts out from behind the cut-out, which fades, and settles at `type.clock` (80), centred, the kicker's top `space.s48` under the frame's top. Its shade follows it, worked out the same way over the blurred, dimmed picture.
- **The sheet** is a `Sheet` on the middle of the bottom edge, curving into the frame through concave fillets of `radius.surface`, `size.panel` (384) wide, with a PanelPage's padding: `space.s16` at its top and foot, the container's padding at its sides. It holds the md `Field` for the password (`lock` leading in the field's icon colour, Password as its placeholder, lit while the keyboard is in it); `space.s4` under it, as a field's note is, a line led by `fingerprint` in `primary`, "Or touch the fingerprint reader", in `small`, `on_surface_variant`; and `space.s16` under that the `KeyHints` ↵ Unlock and Esc Rest.
- No avatar, no initial, no emoji: only KubaOS Symbols.

States: checking, the field's symbol is `hourglass_top` and its placeholder Checking…; wrong, the field shakes, empties and turns Invalid, and the line under it says what went wrong in `error` in place of the reader's line, until the next key.

Motion:

- **Locking:** the lock arrives at rest on every screen; until the picture has arrived the screen is `surface_container_lowest`, and the picture, the clock and the cut-out fade in over `scene` on `emphasized_decel`.
- **Waking:** the frame closes in and the picture blurs over `long` on `emphasized_decel`, the dim comes over `medium` on `standard`, the cut-out fades over `short` on `standard`, the clock lifts over `long` on `emphasized_decel`, its digits going from `type.clock_rest` to `type.clock` on whole device pixels, and the sheet rises as a Sheet opens.
- **Going back to rest:** the sheet closes as a Sheet does and the frame opens out, both over `medium` on `emphasized_accel`; the dim lifts over `medium` on `standard`, the blur clears over `long` on `standard`, the clock settles back over `long` on `emphasized_decel`, and the cut-out fades back over `short` from `medium` on, as the clock lands. The field empties.
- **Unlocking:** the sheet closes, and the clock fades and the blur clears over `medium` on `emphasized_accel`; the dim lifts over `medium` on `standard`; the frame stays, and the desktop comes up in it. `medium` and no longer: the session unlocks the moment it ends.
- **A wrong password** shakes the field: `space.s12`, back `space.s8`, `space.s4`, each over `micro` on `standard`, then home over `short` on `standard_decel`.
- With reduce motion, waking fades the awake lock in over `short` (the frame, the sheet and the clock in their awake places; the cut-out gone at once), going back to rest fades it out over `short` and the rest clock and the cut-out in over the next `short`, unlocking fades over `short`, and the field doesn't shake.

Keys: at rest, any key on the screen with the pointer wakes it, a press too, and a touch of the reader, and a key that types a character lands in the field as its first. Awake, Enter checks what is typed; Esc empties the field and goes back to rest; the field is the only stop for Tab; typing goes into the field whatever had the keyboard, and each key, press or touch keeps it awake for another `duration.lock_idle` (20 s), after which it goes back to rest. Waking lands the keyboard in the field; going to rest lands it on the lock, so the next key wakes it; unlocking gives it back to what had it before.
