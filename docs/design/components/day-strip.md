# DayStrip

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `DayStrip`

Its mockup page is the source of truth: Molecules, DayStrip, in `design/mockups/` (`src/lib/day-strip.ts`). The clock popup's week in its weather view.

A row of days to pick one of, each its forecast at a glance, sharing the row's width equally, `space.s4` apart. A day is a control at the control radius, `space.s8` above and below its words: its name (`label`, `on_surface_variant`) over its weather's symbol (`body`'s icon size, `on_surface`) over its high (`body_strong`, `on_surface`) over its low (`small`, `on_surface_variant`), centred, `space.s4` apart.

- **Rest:** no surface.
- **Hovered and pressed:** the state layer, `on_surface` at `opacity.hover` and `opacity.press`, a faint shape at its radius.
- **Picked:** `secondary_container`, its words `on_secondary_container`, as a Selected chip; hovered and pressed, the container lifts toward `secondary_container_pressed`.
- **Keyboard focus:** the accent's neon line inside the picked day's edge and its glow, only from the keyboard.

**Keys.** The row is one stop for Tab, on the picked day. ← → (h l) pick the day before or after, Home and End the first and last; a press picks the day pressed. Its owner says what a pick shows (the weather view: that day's banner, hours and tiles).

*Not built yet: no library has it; it is a mockup only.*
