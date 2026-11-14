# ProgressBar

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `ProgressBar`

Rust `progress::bar`; QML `ProgressBar`. Its mockup page is the source of truth: Molecules, ProgressBar, in `design/mockups/`.

Work that knows how far it is: a pill 8 tall (`size.progress`) across its room, the track `surface_container_highest` at `opacity.track`, the fill from the left edge to the work's fraction, rounded up to a device pixel, in `primary` with `shadow.glow` under it in its own colour. The glow is a light: it shows at night while the signs are on, flickers with them in a power change, and is out by day. The fill follows the work over `short` on the `linear` curve; a bar that shows up already part full starts there. A fraction the app already times against the clock (how long a button has been held) is drawn as given (`progress::timed_bar`), with no easing to trail behind it. In a `Danger` banner (`danger`, `progress::danger_bar` and `progress::danger_timed_bar`, `danger` in QML) the fill is `error` on a track of `on_error_container` at the tint's opacity, so the hold that erases a disk on Kōbō's Last chance reads as red all the way.

The bar has no words of its own: a `label` line `space.s8` above it says what is going on, the percent at its end as a figure that holds its width, or a `small` count sits `space.s4` under it. Work that can't say how far it is shows a [Spinner](spinner.md) and its words in the bar's place until it can.

*Not built yet: both libraries draw the glow by day too.*
