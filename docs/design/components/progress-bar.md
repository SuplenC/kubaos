# ProgressBar

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `ProgressBar`

Rust `progress::bar`; QML `ProgressBar`. Its mockup page is the source of truth: Molecules, ProgressBar, in `design/mockups/`.

Work that knows how far it is: a pill 8 tall (`size.progress`) across its room, the track `surface_container_highest` at `opacity.track`, the fill from the left edge to the work's fraction, rounded up to a device pixel, in `primary` with `shadow.glow` under it in its own colour. The glow is a light: it shows at night while the signs are on, flickers with them in a power change, and is out by day. The fill follows the work over `short` on the `linear` curve; a bar that shows up already part full starts there. A fraction the app already times against the clock (how long a button has been held) is drawn as given (`progress::timed_bar`), with no easing to trail behind it. In a `Danger` banner (`danger`, `progress::danger_bar` and `progress::danger_timed_bar`, `danger` in QML) the fill is `error` on a track of `on_error_container` at the tint's opacity, so the hold that erases a disk on Kōbō's Last chance reads as red all the way.

The bar has no words of its own: a `label` line `space.s8` above it says what is going on, the percent at its end as a figure that holds its width, or a `small` count sits `space.s4` under it. A bar can carry a caption over its end instead (how long is left, as Kōbō's install says it): `small` in `on_surface_variant` after its symbol at the text's icon size, `space.s8` above the bar on a line of its own as wide as the bar, its end on the bar's end and its words breaking from the end where the line is short. The line keeps a line's height with nothing on it, so the bar never moves as the caption comes and goes, and nothing beside or under the bar shares its room. Work that can't say how far it is shows a [Spinner](spinner.md) and its words in the bar's place until it can.

*Not built yet: both libraries draw the glow by day too.*
