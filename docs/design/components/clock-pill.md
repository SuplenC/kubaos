# ClockPill

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `ClockPill`

Its mockup pages are the source of truth: Shell, Bar, and Shell, Clock popup, in `design/mockups/` (`src/lib/clock-pill.ts`). The bar's clock, which the Lock's bar draws too, for the rest clock to fly into ([Lock](lock.md)). Where the bar places it and what it gives way to as the bar narrows is the [Bar](bar.md)'s.

A [Pill](pill.md) lying flat on the bar's glass like every other item: the weekday's seal (its kanji on a disc of `cold`, `on_cold`), the date plain (`body`, `on_surface_variant`), the time, and home's weather plain like the date (its symbol and the temperature; none with no home city). While its card is open the time sits in a filled capsule of `primary` (`on_primary`), `size.bar_capsule` tall, coming in over `short` on `standard`; at rest it is plain. The capsule's place is the screen's exact middle, whether it shows or not, and the rest of the pill is laid out round it.

- **The time** is `body`, Geist's own proportional figures, centred in a slot as wide as the widest time the clock shows (24-hour or 12-hour), so the pill keeps its width as the minutes turn.
- **Two press targets**, nothing between them: the time's half (the seal, the date and the time) opens the clock's card on the calendar, the weather's half its weather view. One card, centred under the clock, serves both. The bar's edge light lies under the half the pointer is on, or the half whose view is open, and follows the card as it turns from one view to the other.
- **Recording merged in**, on a narrow bar: the capsule stays filled in `error` with `on_error`, the seal's disc turns `on_error_container`, and the recording's `error` dot leads.
- **What it shows** (the seal, the date full or short, seconds, the weather) is set in Settings › Date & time.

*Not built yet: no library has it; `config/quickshell/modules/bar/Clock.qml` draws the old clock, a pill on `surface_container` with the time always in its capsule.*
