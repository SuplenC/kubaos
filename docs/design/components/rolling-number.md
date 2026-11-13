# RollingNumber

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `RollingNumber`

Rust `rolling::rolling_number(value)`; QML `RollingNumber`.

A count that rolls as it changes, as an odometer does: a counter's value beside its buttons. Its figures are tabular, `mono_large` in `on_surface` unless given another style and colour, each in a slot one figure wide and the style's line tall (20) that cuts what leaves it. A slot is a figure's advance rounded up to a whole device pixel, so every figure starts on one at every scale. The box is as many slots wide as the widest value it is given (`widest`, its figures all 0: `"000"` for a count up to 999), its figures held to its end, so the words beside it hold still as the count grows a figure; a value wider than that widens the box. Its figures are the count's own, never grouped or padded.

It isn't a control: it takes no pointer and no focus, and has no disabled, selected, loading or error state. A count that is nothing shows 0.

**Motion.** Only the figures that change roll, the leaving and arriving figure moving together over `short` on `standard`. Going up, the new figure comes in from below as the old one leaves upward; going down, from above as the old one leaves downward. Within its widest value, a figure the count grows comes in from nothing, and one it sheds leaves into nothing. Its first value shows in place. Reduce motion changes the figures in place.
