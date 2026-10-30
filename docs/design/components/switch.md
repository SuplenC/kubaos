# Switch

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Switch`

Rust `switch::switch`; QML `Switch`.

On or off, 44 × 24, the knob 12 off and 16 on, a hairline track. It takes effect at once; Space, Enter or a click toggles.

Off, the track is `surface_container_highest` with an `outline` hairline inside its edge and an `outline` knob. On, the track is filled with the accent: at night the unlit fill (`primary_fill`) with an `on_primary_fill` knob, as a Filled button's fill and label; by day the accent at lightness 0.80 with a dark knob.

**States** follow the neon (`docs/DESIGN.md`, "States"), each off and on, with no state layer:

- **Rest:** unlit at night, as above.
- **Lit:** hovered or focused, one state. Signs on, at night while the lights burn, the line inside the edge burns with the glow outside it: off the tube (`primary` at `focus.tube`) over the outline, and the knob brightens to `on_surface_variant`; on the core (`primary_fill_line`, `stroke.thin` wide) inside the fill's edge, and the knob brightens to `on_primary_fill_lit`. Signs off, by day always and at night while a power change has the lights out, the line is glass (off `primary_glass`, solid, as a Tonal or Text button's; on `primary_fill_glass`) with no glow and the knob at rest.
- **Pressed:** overdriven and lit, the glow `focus.pressed_glow` times the lit one. Off, the tube runs to the core (by day the tone tints the track at `opacity.press_tint`); on, the fill lifts 0.035 (`primary_fill_pressed`, 0.04 darker by day).
- **Disabled** (no `on_toggle`, `enabled: false`): grey, with no tone, no line and no glow: the track `on_surface` at `opacity.disabled_container` with no outline, the knob at `opacity.disabled_content`.

**Motion.** Toggled, the knob moves and grows and the colours fade over `short` on `standard`. Lighting up and going back to rest are fades over `short`, a press over `micro`; the flicker is for power changes only, where a lit switch flickers to its signs-off look and back and one at rest just recolours.

*Not built yet: the libraries draw the track in `primary` with an `on_primary` knob, dim a disabled switch as a whole, mix `on_surface` over the track on hover and strike focus; the mockups' Switch page (`design/mockups/src/pages/atoms/switch.ts`) draws the rule.*
