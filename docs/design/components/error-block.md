# ErrorBlock

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `ErrorBlock`

Rust `error::error_block`; QML `ErrorBlock`. Its mockup page is the source of truth: Molecules, ErrorBlock, in `design/mockups/`.

What broke and what to do next, shown where it broke: a status message lying flat in the page, on the error container at `opacity.error_fill` with the sunken soft hairline, the content radius (16), 16 inside, its left edge lit in `error` (the thin stroke, tapering into both corners: `error_core` with the error's near glow at night while the signs are on, `error` by day). In it a 24 error icon, 16 from the message in `subheading` (`body_strong` past 90 characters, since a whole command line in 16 px semibold is a wall), the hint in `body` 4 under it. It holds no buttons: the screen's own actions follow it (Show log, Save log, Try again), and the [Log](log.md) behind it opens under it.

*Not built yet: both libraries draw it at the surface radius (24), 20 inside, with an error hairline at `opacity.error_edge` and no lit edge.*
