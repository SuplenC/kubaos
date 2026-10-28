# Banner

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Banner`

Rust `banner::banner`; QML `Banner`.

A message that matters on the page itself, with what to do about it inside: a 24 symbol 16 from a `subheading` title over a `body` line and any paragraphs more, 4 apart, all wrapping, 16 inside at the surface radius; its actions 12 apart at its end beside the words, the one it is about last; an optional foot 16 under the rest (a held button's `timed_bar`). Tones: `Info` (`surface_container_high` with a `primary` hairline and symbol), `Warning` (the same with a `warning` hairline and symbol), `Danger` (really red: `error_container` with an `error` hairline and symbol, its words in `on_error_container`), so the step that destroys something can't be skimmed; on its red a `Text` button takes `on_error_container` and the foot's bar `error`, the banner's own colours, not the accent (`banner::inside`; in QML they keep the accent). An empty body leaves the title alone, a line tall. Kōbō's Last chance is a `Danger` banner holding Back and the held Install.
