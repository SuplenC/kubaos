# Banner

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Banner`

Rust `banner::banner`; QML `Banner`. Its mockup page is the source of truth: Molecules, Banner, in `design/mockups/`.

A message that matters on the page itself, with what to do about it inside: a status message lying flat in the page, with the sunken soft hairline, the content radius (16), 16 inside, its left edge lit in its tone (the thin stroke, tapering into both corners: the tone's core with its near glow at night while the signs are on, the tone by day). In it a 24 symbol 16 from a `subheading` title over a `body` line and any paragraphs more, 4 apart, all wrapping; its actions 12 apart at its end beside the words, the one it is about last; an optional foot 16 under the rest (a held button's `timed_bar`, filling over `duration.hold`). Tones: `Info` (`surface_container_high`, its edge and symbol `primary`), `Warning` (the same with `warning`), `Danger` (really red: `error_container`, its edge and symbol `error`, its words in `on_error_container`), so the step that destroys something can't be skimmed; on its red a `Text` button takes `on_error_container` and the foot's bar `error`, the banner's own colours, not the accent (`banner::inside`; in QML they keep the accent). An empty body leaves the title alone, a line tall. It isn't raised, never goes by itself and has no close. Kōbō's Last chance is a `Danger` banner holding Back and the held Install.

*Not built yet: both libraries draw it at the surface radius (24) with a hairline in its tone and no lit edge.*
