# Chip

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Chip`

Rust `chip::chip`; QML `Chip`.

A short fact or pick, 24 tall (`chip`) at the control radius, 12, which on a chip is a pill, `label` type, an optional 16 icon 4 from the label, an optional close. A pick may also be pressed, with the state layer: Kōbō's picked layouts, where a press makes one the default, and a setting a toolbar turns on and off (the keyboard section's Mac accents, `Selected` while on, `Neutral` while off), which in the shell takes the keyboard too and shows the focus tube. In QML a pressable chip may be no stop for Tab (`tabStop: false`) where the field or the row beside the chips is the stop and a click picks one: the clock's timer presets and settings. A pressable chip's `tooltip` (QML too) names it in a `Hint` under it while the pointer or the keyboard is on it. *iced's `chip::chip` has neither yet.* Its left side pads 12, or 8 with an icon; its right pads 12, or nothing when the close button (a 24 circle) ends it. Kinds: `Neutral` (a fact: `surface_container_high` under `on_surface`), `Selected` (one of several picks: `secondary_container` under a label dimmed at night at rest, `on_secondary_container_rest`, as a Tonal button's), `Accent` (a tag that matters, Recommended, Default: the accent at `opacity.tint` under `primary`), `Cold` (rarely, demo, preview: the cold accent at `opacity.tint` under `cold`), `Filled` (the accent's own fill: an action among states, Reboot beside Done and To do; at night the unlit fill, `primary_fill`, under `on_primary_fill`, as a Filled button's, and by day the accent at lightness 0.80 with a dark label). A chip that toggles a setting is `Selected` while on and `Neutral` while off, fading between them over `short`.

**States:** a plain chip has none: it is its kind's colours at rest. A pressable one follows the neon (`docs/DESIGN.md`, "States"), with no state layer:

- **Rest:** its kind's colours, as above.
- **Lit:** hovered or focused, one state. Signs on, at night while the lights burn, the line inside the pill's edge burns with the glow outside it in its kind's tone: the tube (`focus.tube`) in the accent on Neutral, Selected and Accent and in the cold accent on Cold, the core (`primary_fill_line`, `stroke.thin` wide) inside Filled's fill. Selected's label goes full (`on_secondary_container`) and Filled's brightens (`on_primary_fill_lit`). Signs off, by day always and at night while a power change has the lights out, the line is glass (the tone's `_unlit` at `focus.glass`, or `primary_fill_glass` on Filled) with no glow and the label at rest.
- **Pressed:** overdriven and lit, the glow `focus.pressed_glow` times the lit one. Neutral's, Accent's and Cold's tube runs to the core (by day the tone tints the pill at `opacity.press_tint`); Selected's container and Filled's fill lift 0.035 (0.04 darker by day).
- **Disabled** (`Chip::disabled`, `enabled: false`): grey, with no tone, no line and no glow: the pill `on_surface` at `opacity.disabled_container`, its content at `opacity.disabled_content`; neither its press nor its close button takes the pointer or the keyboard, and it is no stop for Tab.

Its close button is a control of its own, lit as a Text icon button is: the tube inside its circle in the chip's tone, the core on a Filled chip, with the glow outside it; pressed, the tube runs to the core. With the pointer over the close button, it lights alone.

**Motion.** Lighting up and going back to rest are fades over `short` on `standard`, a press over `micro`, and a toggling chip fades between Neutral and Selected over `short`. The flicker is for power changes only, where a lit chip flickers to its signs-off look and back and every other chip just recolours.

*Not built yet: the libraries fill `Filled` with `primary` under `on_primary`, draw hover and press as state layers (the close button's too), light a `Cold` chip in the accent and strike focus; the mockups' Chip page (`design/mockups/src/pages/atoms/chip.ts`) draws the rule.*
