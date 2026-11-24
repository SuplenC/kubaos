# RollingNumber

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `RollingNumber`

Rust `rolling::rolling_number(value)`; QML `RollingNumber`.

A count that rolls as it changes, as an odometer does: a counter's value beside its buttons. Its figures are tabular whatever its style (a proportional style's figures are set `tnum`, and its slot is a tabular figure wide), so a narrow figure leaves no hole before a unit or a percent sign after it; `mono_large` in `on_surface` unless given another style and colour, each in a slot one figure wide and the style's line tall (20) that cuts what leaves it. A slot is a figure's advance rounded up to a whole device pixel, so every figure starts on one at every scale. The box is as many slots wide as the widest value it is given (`widest`, its figures all 0: `"000"` for a count up to 999), its figures held to its end, so the words beside it hold still as the count grows a figure; a value wider than that widens the box. Its figures are the count's own, never grouped or padded.

Its figures are the digits unless it is given others (`figures` in the mockups): a kanji numeral's, 一 to 十, one glyph to a slot as wide as the widest of them, rolling as a digit does, up or down by the value. Ten is 十, one glyph, so a kanji number never changes its count of figures. The bar's workspaces in kanji roll theirs this way. *Not built yet: neither library takes figures; only the mockups do.*

A number can be written with marks between its figures (`format` in the mockups): a clock's colons, a minus before the time left. A mark sits in a slot as wide as its own advance, rounded up to a whole device pixel, and never rolls; the box is as wide as the widest value's figures and marks, so a clock that grows an hour keeps its place. The media player's times are clocks. *Not built yet: neither library writes marks; only the mockups do.*

Its figures are centred for the mass of every figure in its own style. A number set among others on one baseline is centred for theirs instead (`centre` in the mockups: a style and the words to weigh): the bar's workspace digits are each centred for a 0 in `workspace_digit`, so they stand on one baseline. *Not built yet: neither library takes a centre; only the mockups do.*

It isn't a control: it takes no pointer and no focus, and has no disabled, selected, loading or error state. A count that is nothing shows 0.

**Motion.** Only the figures that change roll, the leaving and arriving figure moving together over `short` on `standard`. Going up, the new figure comes in from below as the old one leaves upward; going down, from above as the old one leaves downward. A figure the count grows comes in from nothing, and one it sheds leaves into nothing. Its first value shows in place. Reduce motion changes the figures in place.

**Its width follows its figures.** At rest the box is exactly its figures' width, never under its widest value's. When a roll changes how many figures there are (9 to 10, 10 to 9), the box eases from the old figures' width to the new over the same `short` on `standard`, starting with the roll, so an owner that centres the number moves it as smoothly as the figures roll and it already stands centred when the roll ends: a workspace's number in the bar, the carried workspace's disc, an Overview workspace's badge. A figure it sheds rolls out just before the figures that stay, drawn past the box's edge, never cut, and goes as its roll ends. Reduce motion changes the width with the figures, at once.
