# Calendar

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Calendar`

Rust `calendar::calendar`; QML `Calendar`. Its mockup page is the source of truth: Molecules, Calendar, in `design/mockups/`.

A month as six weeks, each led by its ISO week number (the week's Thursday's, `caption`, a small control wide), under the weekdays' names (`label`, two letters, weekends in `tertiary`). A day is a regular control (40), its disc a small one (32), a pill: today filled with `primary`, its number `body_strong` in `on_primary`; the picked day ringed with `primary` in the thin stroke; a holiday dotted in `tertiary` with its name in a hint while it is lit; the days of the months around dimmed to the disabled content's 0.38. The first weekday is the clock's setting (Monday by default, ISO 8601's). Between the arrows (`Text` sm icon buttons) the title, `subheading` at the control radius (12) as the year's months are, opens the year (`year`): the twelve months three to a row, the one shown filled, the title Tonal; the month and the year cross-fade over `short`. The app works the month out (`kubaos-clock` for the shell); a new month slides in `space.s64` from the side it lies on as it fades in, over `short` on `emphasized_decel` (with reduce motion it only fades), and a sideways turn of the wheel (Shift and the wheel, or a touchpad's sideways swipe) moves a month, a notch at a time; a turn up or down scrolls past it. The clock popup's.

**States.** A day, a month of the year and the title follow the neon as a Text button does: lit (under the pointer, or the keyboard's day), the tube inside the disc's edge (today's core inside its fill) with the glow outside at night; pressed, the tube runs white-hot and the glow brightens, and by day the tone tints the disc; letting go picks it. It has no disabled state.

**Keys.** The arrows and the title are stops of their own; the month is one stop, on the picked day or today. ← → (h l) move a day, ↑ ↓ (k j) a week, crossing into the month around; Page Up and Page Down a month; Home and End the week's first and last day; Enter or Space picks. The pointer moving onto a day puts the keyboard there. With the year open, ← → move a month, ↑ ↓ three, Enter picks, Esc goes back to the month.

*Not built yet: the days take no keyboard of their own (the clock popup moves a month with ← → from its search field), they light with a state layer, the year swaps in with a scale, and the title is a pill; the mockup draws the rule.*
