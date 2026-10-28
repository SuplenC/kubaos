# ProgressBar

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `ProgressBar`

Rust `progress::bar`; QML `ProgressBar`.

Work that knows how far it is: 8 tall (`progress`), the fill glowing in its own colour. The fill follows the work over `short` on the `linear` curve; a bar that shows up already part full starts there. A fraction the app already times against the clock (how long a button has been held) is drawn as given (`progress::timed_bar`), with no easing to trail behind it. On a danger banner's red (`danger`, `progress::danger_bar` and `progress::danger_timed_bar`, `danger` in QML) the fill is `error` on a track of `on_error_container` at the tint's opacity, so the hold that erases a disk on Kōbō's Last chance reads as red all the way.
