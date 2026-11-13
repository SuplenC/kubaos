# Tabs

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Tabs`

Rust `tabs::tabs`; QML `Tabs`. Its mockup page is the source of truth: Molecules, Tabs, in `design/mockups/`.

One of a few views or values, picked along a rail of neon. It is the segmented control too: a setting's few choices side by side (the wallpaper's motion, Always, On AC, Never). The control center's sections, the clock's tools, the wallpaper picker's views (Picture, Desktop, Lock), the system monitor's order. Choices that don't fit their room are a `Select` instead: tabs never wrap or scroll.

**The well.** The choices sit side by side in one well: a pill (`radius.full`) in `surface_container_highest` at `opacity.track`, as a slider's track is, `size.control_sm` (32) tall, with `space.s4` round the choices. Each choice is the well's height less that padding (24), `space.s12` inside its sides, its words `button_small` with an optional symbol `space.s4` before them in the words' colour. Nothing fills the picked choice, and the well is as wide as its choices.

**The rail.** An unlit rail of neon runs straight along the well's bottom edge, on the edge with nothing below it, across the flat part between the round ends (half the well's height in from each side): the tone at the spinner's ring's `opacity.spinner_track`. The picked choice's stretch of it is lit, under its words (inset `space.s12` from the choice's sides), square at both ends: at night the core (`primary_core`) with the content glow round it, as the spinner's arc; by day the tone, with no glow. It is `tabs.rail` (2) thick at night and `tabs.rail_day` (3) by day, whole device pixels rounded half up.

**The words.** Picked, they are `primary_lit`: the core at night, fitted to 4.5:1 on the well, with the label glow round them (`focus.label_glow` over `focus.label_near`); the tone by day, with no glow. The weight never changes, so nothing shifts. The rest are `on_surface_variant`.

**A tip.** A choice may carry a tip, what picking it means: the library's `Tooltip`, the choice's name as its title and the tip, a short sentence about that choice alone, as its muted body under it, `space.s8` above the well, lined up with the well's start for a choice in the row's first half and its end for one in the second, so the card stays over the row. It shows while the pointer is on the choice or the keyboard is, fading over `short` as every tooltip does, and takes no room: nothing round the row moves. Calls and Music each say what they mean.

**Picking.** The lit stretch travels from the old choice to the new one and stretches as it goes: its leading edge, the one toward the new choice, gets there over `short` on `emphasized_decel`, and its trailing edge follows a `micro` later over `medium` on the same curve, catching up. As the pick switches, its core flashes to `primary_flash` over `micro` (at night most of the way to white, its glow swelling to `focus.pressed_glow` times its own; by day a brighter tint of the tone) and settles back over `long`, on `standard`. This isn't `neon.strike`: nothing powers on. The words cross-fade over `short` on `standard`. A pick made while the stretch is still travelling starts from where it is. With reduce motion the stretch is on the new choice at once; the flash, a change of colour, stays, settling over `short`, and the words still fade.

**States.**

- **Under the pointer**, a choice that isn't picked brightens its words to `on_surface` over `short`. The rail doesn't change.
- **Pressed**, a choice's words go to the picked colour over `micro`; letting go picks it.
- **The keyboard:** the row is one stop for Tab, on the picked choice, and while it has the keyboard the focus tube lights the whole well's edge, with its glow, under the rail; the lit stretch already says which choice is picked. ← and → (or h and l) pick the neighbour, round from either end, and the keyboard moves with the pick (the owner moves the pick in iced; `Tabs` in QML does it). Space and Enter do nothing more.
- **Signs off** (by day always, at night while a power change has the lights out), the focus tube and the lit stretch are solid glass, `primary_glass`, with no glow; the picked words keep `primary_lit`, with no glow round them.
- **Disabled** (`Tabs::disabled`, `enabled: false`): every part at `opacity.disabled_content` (the well, the rail, the lit stretch and the words), with no glow, the pick still shown; no press, and no stop for Tab.

*Not built yet: `tabs::tabs` and `Tabs` draw a row of small buttons, the picked one `Tonal` and the rest `Text`; the mockups draw the rail.*
