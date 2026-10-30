# Radio

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Radio`

Rust `check::radio`; QML `Radio`.

A 20 disc with an 8 dot. One of a group, exactly one picked.

Its label is `body`, `on_surface`, 12 from the disc, and the disc and the label are one press target. Not picked, the disc is `surface_container_highest` with an `outline` hairline. Picked, it is filled with the accent: at night the unlit fill (`primary_fill`) with an `on_primary_fill` dot, as a Filled button's fill and label; by day the accent at lightness 0.80 with a dark dot.

**States** follow the neon (`docs/DESIGN.md`, "States"), picked and not, with the line on the disc and no state layer:

- **Rest:** unlit at night, as above.
- **Lit:** hovered or focused, over the disc or its label, one state. Signs on, at night while the lights burn, the line inside the disc's edge burns with the glow outside it: not picked the tube (`primary` at `focus.tube`) over the outline; picked the core (`primary_fill_line`, `stroke.thin` wide) inside the fill's edge, and the dot brightens to `on_primary_fill_lit`. Signs off, by day always and at night while a power change has the lights out, the line is glass (`primary_glass`, solid, as a Tonal or Text button's, or `primary_fill_glass` picked) with no glow. The label doesn't light.
- **Pressed:** overdriven and lit, the glow `focus.pressed_glow` times the lit one. Not picked, the tube runs to the core (by day the tone tints the disc at `opacity.press_tint`); picked, the fill lifts 0.035 (`primary_fill_pressed`, 0.04 darker by day).
- **Disabled** (no action, `enabled: false`): grey, with no tone, no line and no glow: the disc `on_surface` at `opacity.disabled_container` with no outline, the dot and the label at `opacity.disabled_content`.

**Motion.** Picking one fades its fill and dot in over `short` on `standard` as the one it unpicks fades back. Lighting up and going back to rest are fades over `short`, a press over `micro`; the flicker is for power changes only.

*Not built yet: the libraries fill a picked disc with `primary` and an `on_primary` dot, dim a disabled one as a whole, have no hover or press look, show no fade on picking and strike focus; the mockups' Radio page (`design/mockups/src/pages/atoms/radio.ts`) draws the rule.*
