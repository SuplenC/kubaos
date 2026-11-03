# Step

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Step`

Rust `step::step(title)`; QML `Step`. Its mockup page is the source of truth: Molecules, Step, in `design/mockups/`.

One step of a stepper or a checklist: an optional symbol (24 beside two lines), a `body_strong` title and a `small` subtitle that wraps to as many lines as it needs, never cut, then where it stands. To do is a `Neutral` chip, done a quiet `Accent` chip with a check (on a selected row a `Neutral` one with its check, since the accent's tint over `secondary_container` leaves its label under 4.5:1), an action (Reboot) a `Filled` chip with its symbol, the loudest of the three, since it is what to do next, and the current step is the selected row (`secondary_container`); the three never look alike. A narrow rail takes a `mark` instead of a chip: a check when done, the action's symbol in `primary`, its room kept otherwise. In a rail the symbol is a bead on the rail's progress wire ([StepRailPage](step-rail-page.md)): `primary` once its step is reached (done or current), the current one with a lit control's glow at night (`focus.near` and `focus.far`, while the signs are on) and none by day, and, while its work runs, humming down to `opacity.hum` and back over `duration.hum` each way (still with reduce motion); a step not reached keeps its ink. In a rail the current step is a quiet fill, `on_surface` at `opacity.hover`, its words as they rest, not the selected row. A step that failed shows `error` in `error` in its symbol's place, with no glow. 44 tall with one line, 56 with a subtitle's first line and a line taller for each line more; padding 12 across, the corners its box's less the box's padding: the row radius (8) in a list's well, the surface radius less 12 in Kōbō's rail. Kōbō's step rail and first boot's checklist.

**States:** to do, current (the selected row; in a rail the quiet fill and the lit symbol), running (in a rail, the symbol humming), done, an action and failed; a pressable step resting, lit (the accent's tube inside its edge, the glow outside at night) and pressed (the tube white-hot at night, the tone's tint by day).

*Not built yet: both libraries draw a step at 12 in a well too, light it with a state layer, and draw a failed step's symbol in the row's own colour; neither draws a rail step's symbol as a bead, nor its current row as a quiet fill.*
