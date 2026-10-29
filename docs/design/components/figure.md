# Figure

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Figure`

QML `Figure`; *QML only: the clock popup's times, its timer and its stopwatch. It is in neither specimen, so the parity check leaves it out; `kubaos-ui` gets one with the first app that shows a ticking number.*

A number that ticks: a time, a timer, a stopwatch. Geist's figures are proportional, so a ticking number would jitter what sits beside it as its digits change. A `Figure` is a `Text` (`text`, `style`, `color`) in a box as wide as the same text with every digit a 0, Geist's widest; the text starts at the box's left edge, and what sits beside or around the box holds still as the digits change. Only the digits are held: a colon, a sign or a unit still takes its own width, so a box grows only when the text's shape does (59:59 to 1:00:00).
