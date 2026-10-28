# Sparkline

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Sparkline`

Rust `meter::sparkline`; QML `Sparkline`.

The last minute of a reading as a line in the thin stroke, filled under it with the line's colour at the tint's opacity, newest at the right edge (QML places each sample by its time over the last minute, so a graph with less than a minute of history starts part way across; iced spaces the samples evenly); a second reading over it, unfilled, in `tertiary` (upload over download). As wide as its room and 48 tall (`sparkline`), or a line of small text tall inline in the bar. The scale tops out at 100, or fits the highest point with room above it. It draws when a sample comes, never per frame.
