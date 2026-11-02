# Step

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Step`

Rust `step::step(title)`; QML `Step`. Its mockup page is the source of truth: Molecules, Step, in `design/mockups/`.

One step of a stepper or a checklist: an optional symbol (24 beside two lines), a `body_strong` title and a `small` subtitle that wraps to as many lines as it needs, never cut, then where it stands. To do is a `Neutral` chip, done a quiet `Accent` chip with a check (on a selected row a `Neutral` one with its check, since the accent's tint over `secondary_container` leaves its label under 4.5:1), an action (Reboot) a `Filled` chip with its symbol, the loudest of the three, since it is what to do next, and the current step is the selected row (`secondary_container`); the three never look alike. A narrow rail takes a `mark` instead of a chip: a check when done, the action's symbol in `primary`, its room kept otherwise. A step that failed shows `error` in `error` in its symbol's place. 44 tall with one line, 56 with a subtitle's first line and a line taller for each line more; padding 12 across, the corners its box's less the box's padding: the row radius (8) in a list's well, the surface radius less 12 in Kōbō's rail. Kōbō's step rail and first boot's checklist.

**States:** to do, current (the selected row), done, an action and failed; a pressable step resting, lit (the accent's tube inside its edge, the glow outside at night) and pressed (the tube white-hot at night, the tone's tint by day).

*Not built yet: both libraries draw a step at 12 in a well too, light it with a state layer, and draw a failed step's symbol in the row's own colour.*
