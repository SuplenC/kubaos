# List, ListRow and EmptyList

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `List`

Rust `list::list`; QML `List`.

Rows in a well: the surface radius (24), padding 12, rows 4 apart, so the rows take 24 − 12 = 12, the control radius (concentric). The focus tube lights the well's edge; the row shows where the keyboard is.

## `ListRow`

Rust `list::list_row(title)`; QML `ListRow`.

44 tall with one line (`row`), 56 with a subtitle (`row_two_line`). Padding 12, a 20 icon (24, `icon.two_line`, beside a title over a subtitle, so it spans both lines) or the icon on an `IconTile` (`tile`: `Sm` beside one line, `Md` beside two), where the symbol is what tells the rows of one kind apart (the keyboard layouts, the default's star in the accent), the title in `body_strong`, subtitle in `small`, each one line that ends in an ellipsis, the subtitle optionally after a 16 symbol 4 from its words (`subtitle_icon`: the lock of a secured network, a device's battery); chips near the end for facts that matter at a glance (Default, Too small, To do), a detail in `mono` at the end, an optional check whose room is kept when unchecked so row ends line up. Each part is centred on the row's whole height, so a chip moves nothing else. The selected row takes `secondary_container`, or `error` on a row whose action can't be taken back (Shut down). A list taller than its room scrolls inside its well (`list::scrolling_list`), its thumb in the well's padding. A list whose order means something (the keyboard layouts, the first the default) is reordered by dragging any row, with no handle: past the drag threshold the row follows the pointer, drawn over the others at `subdued` so the row it passes over shows through, and a thin line in `primary` (`stroke.thin`, a pill, inset 12 as the rows' words are) in the gap where it will land; the list changes once, on the drop.

## `EmptyList`

Rust `list::empty_list`; QML `EmptyList`.

The well with nothing in it: one centred line.
