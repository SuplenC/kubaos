# Checkbox

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Checkbox`

Rust `check::checkbox`; QML `Checkbox`.

Square, 20, the small radius (4), a hairline edge. Empty, partial (a dash, for some of a group) or full. For several independent picks that apply together.

Its label is `body`, `on_surface`, 12 from the box, and the box and the label are one press target. Empty, the box is `surface_container_highest` with an `outline` hairline. Partial and full, it is filled with the accent: at night the unlit fill (`primary_fill`) with an `on_primary_fill` dash or tick, as a Filled button's fill and label; by day the accent at lightness 0.80 with a dark mark.

**States** follow the neon (`docs/DESIGN.md`, "States"), each empty, partial and full, with the line on the box and no state layer:

- **Rest:** unlit at night, as above.
- **Hovered:** the pointer over it, quiet: over the box or its label, empty the box takes the state layer (`on_surface` at `opacity.hover`), picked the fill goes part way to `primary_fill_pressed`, with no line and no glow, the label at rest (`docs/DESIGN.md`, "States").
- **Lit:** focused from the keyboard. Signs on, at night while the lights burn, the line inside the box's edge burns with the glow outside it: empty the tube (`primary` at `focus.tube`) over the outline; picked the core (`primary_fill_line`, `stroke.thin` wide) inside the fill's edge, and the mark brightens to `on_primary_fill_lit`. Signs off, by day always and at night while a power change has the lights out, the line is glass (empty `primary_glass`, solid, as a Tonal or Text button's; picked `primary_fill_glass`) with no glow. The label doesn't light.
- **Pressed:** a step past hovered, as quiet: empty, the state layer at `opacity.press`; picked, the fill lifts 0.035 (`primary_fill_pressed`, 0.04 darker by day). A key's press on a focused box keeps the neon, its glow `focus.pressed_glow` times the lit one.
- **Disabled** (no `on_toggle`, `enabled: false`): grey, with no tone, no line and no glow: the box `on_surface` at `opacity.disabled_container` with no outline, the mark and the label at `opacity.disabled_content`.

**Motion.** Picked or cleared, the fill and the mark fade in as the outline fades out over `short` on `standard`. Lighting up and going back to rest are fades over `short`, a press over `micro`; the flicker is for power changes only.

*Not built yet: the libraries fill a picked box with `primary` and an `on_primary` mark, dim a disabled one as a whole, have no hover or press look, show no fade on picking and strike focus; the mockups' Checkbox page (`design/mockups/src/pages/atoms/checkbox.ts`) draws the rule.*
