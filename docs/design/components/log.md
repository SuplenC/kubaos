# Log

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Log`

Rust `error::log`; QML `Log`. Its mockup page is the source of truth: Molecules, Log, in `design/mockups/`.

The log behind an error or a progress bar, opened on demand by a `Text` button (Show log, `terminal`): a sunken well on `surface_container_lowest` at `opacity.log` with the soft hairline, the content radius (16), its own thumb in its padding, its lines `mono`, 16 in at the sides and 12 above and below, newest at the bottom. Each line takes its level's colour: an error `error`, a warning `warning`, a step starting `cold`, the rest `on_surface`, debug `on_surface_variant`. Lines wrap between words; a log of wide output can run each on one line and scroll across. The caller sets its height as an `Extent`; `log` (232) is the size the specimen and Kōbō use. It opens and closes over `medium` on `emphasized_decel`.

It never jumps: at its end it follows the newest line; scrolled up, what is read holds still as lines arrive, and lines dropped from its top give their room back without moving it. End, or scrolling back to the end, follows again. Its lines select as text and copy (Ctrl+A, Ctrl+C). It is a stop for Tab, lit with the accent's tube inside its edge; the arrows, Page Up and Page Down, Home and End scroll it.

*Not built yet: both libraries draw it at the surface radius (24), warnings in `primary`; their lines don't select, it takes no keyboard, and scrolled up it moves as lines arrive (iced) or stops following for good (QML).*
