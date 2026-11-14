# ErrorBlock

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `ErrorBlock`

Rust `error::error_block`; QML `ErrorBlock`. Its mockup page is the source of truth: Molecules, ErrorBlock, in `design/mockups/`.

What broke and what to do next, shown where it broke: a status message lying flat in the page, on the error container at `opacity.error_fill` with the sunken soft hairline, the content radius (16), 16 inside. Down its start side runs the [Banner](banner.md)'s rail in `error`, from the top padding to the bottom one with round ends, `banner.rail` (4) at night and `banner.rail_day` (4) by day in whole device pixels, filling a lane 4 wide 12 in from the box's edge, so the words start 12 past the lane, 28 in: `error_core` with a lit control's glow round it, near and far, at night while the signs are on, glass while they are off, `error` by day. In it a 24 error icon, 16 from the message in `subheading` (`body_strong` past 90 characters, since a whole command line in 16 px semibold is a wall), the hint in `body` 4 under it. It holds no buttons: the screen's own actions follow it, 16 under it (Show log, Save log, Try again), and the [Log](log.md) behind it opens under them, 16 below, over `medium` on `emphasized_decel` (with reduce motion, at once and fading in over `short`). Opening the log moves nothing above it: the actions stay under the pointer, and the screen grows to hold the log.

*Not built yet: both libraries draw it at the surface radius (24), 20 inside, with an error hairline at `opacity.error_edge` and no rail.*
