# List, ScrollingList, ListRows, ListRow, LiveList and EmptyList

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `List`

Rust `list::list`; QML `List`. Its mockup page is the source of truth, with `ListRow` and `ListRows`: Molecules, List, ListRow and ListRows, in `design/mockups/`.

Rows in a well: the surface radius (24), padding 12, rows 4 apart, so the rows take 24 − 12 = 12, the control radius (concentric). The focus tube lights the well's edge; the row shows where the keyboard is.

**States:** resting, and focused: the tube on the well's edge while the list has the keyboard.

## `ScrollingList`

Rust `list::scrolling_list`; QML `ScrollingList`. Its mockup page is the source of truth: Molecules, ScrollingList, in `design/mockups/`.

A `List` whose rows scroll inside its well, as tall as its place makes it. The rows scroll right up to the well's edge and are cut by its own rounded shape; the 12 above the first row and below the last is the scroll's padding (`DESIGN.md`, "Rules every component keeps"). The well's hairline and focus tube are drawn over the rows. In QML it holds a `ListRows` (or a `LiveList`) beside whatever else the well holds (a toolbar at its foot, a form that takes the list's place), 12 in at the sides and down its whole height, and the view's `topMargin` and `bottomMargin` are the 12.

**States:** resting, and focused: the tube on the well's edge while the list has the keyboard.

*Not built yet: QML's well shows no thumb, and iced's runs its thumb the well's whole height, where the corners cut it; neither keeps the row the keys reach the well's padding clear of its edge, and a wheel's notch scrolls each toolkit's own distance. The mockup floats the thumb `space.s4` in, along the rows' room `space.s12` clear of the top and bottom, and scrolls a row and its gap a notch.*

## `ListRows`

QML `ListRows`; in iced the app owns the keys and moves its list's `selected` row. The QML specimen's "Shell only" page shows it in a `ScrollingList`, at rest and with the keyboard on it.

A list's rows that scroll, 4 apart, moved through with the keys (`DESIGN.md`, "The keyboard"): ↑ ↓, Ctrl+N, Ctrl+P, Ctrl+J, Ctrl+K, j and k, Home and End, Page Up and Page Down, and Enter once per press, which presses the current row; in a list laid out across, ← → and h and l. A step past either end wraps round; Home, End and a page stop at the ends, and a page is as many rows as show whole. Rows that can't be selected (a section's label, a note) are passed over (`selectable`). `vimKeys: false` leaves the plain letters to the list's owner, and `fieldKeys: true` leaves Home and End to a text field that forwards its keys to the list; Alt and a key are the owner's too (a list that reorders moves its row with Alt+↑ ↓). The current row shows itself selected while `highlightShown`, as a rule while the list has the keyboard. A list with `highlightFollowsCurrentItem: false` scrolls only for the keys, which bring the row they select into view.

## `ListRow`

Rust `list::list_row(title)`; QML `ListRow`.

44 tall with one line (`row`), 56 with a subtitle (`row_two_line`). Padding 12, a 20 icon (24, `icon.two_line`, beside a title over a subtitle, so it spans both lines) or the icon on an `IconTile` (`tile`: sm beside one line, md beside two), where the symbol is what tells the rows of one kind apart (the keyboard layouts, the default's star in the accent), the title in `body_strong`, subtitle in `small`, each one line that ends in an ellipsis, the subtitle optionally after a 16 symbol 4 from its words (`subtitle_icon`: the lock of a secured network, a device's battery); chips near the end for facts that matter at a glance (Default, Too small, To do), a detail in `mono` at the end, an optional check whose room is kept when unchecked so row ends line up. Each part is centred on the row's whole height, so a chip moves nothing else. The selected row takes `secondary_container`, or `error` on a row whose action can't be taken back (Shut down). A list taller than its room scrolls inside its well (`list::scrolling_list`), its thumb in the well's padding. A list whose order means something (the keyboard layouts, the first the default) is reordered by dragging any row, with no handle: past the drag threshold the row follows the pointer, drawn over the others at `subdued` so the row it passes over shows through, and a thin line in `primary` (`stroke.thin`, a pill, inset 12 as the rows' words are) in the gap where it will land; the list changes once, on the drop.

A row still being found (`loading`) keeps the place of the tiled two-line row that will take it: its tile and its two lines as `Skeleton`s, the lines as much of the words' room as `share` says, so rows one under another don't read as one block, and it takes no press. A Wi-Fi or Bluetooth scan with nothing found yet shows them. *QML only: iced's `list_row` has no loading state yet. It is on the QML specimen's "Shell only" page, which the parity check leaves out.*

**States:** resting, hovered, pressed, selected (the secondary container, or the error colour when `danger`), checked, and loading (QML only, as above).

*Not built yet: both libraries light a row with a state layer under the pointer and fill only the selected row, draw a danger row's content in `on_surface` at rest, tick a checked row in its content colour with a plain title, keep a dimmed row selectable, and have no keycaps, switch or chevron at a row's end; the mockup lights the row the keyboard is on (which the pointer moves) on the secondary container as a menu item, picks in `primary_lit`, and passes over a disabled row.*

## `LiveList` and `LiveRow`

QML `LiveList`, its delegates `LiveRow`s (its mockup page is the source of truth: Molecules, LiveList and LiveRow, in `design/mockups/`); *iced has no live list yet, and the rule applies to it as it gets one. It is on the QML specimen's "Shell only" page, which the parity check leaves out.*

A list a scan updates while it shows, which never jumps (`DESIGN.md`, "Motion"): a `ListRows` whose rows (`rows`, each with a `key`) are kept in step by key (`Keyed`, the fewest removes, inserts and moves), so only what changed changes. A row that arrives opens its room over `medium` (`standard_decel`) and fades in over `slow`; one that leaves fades and gives its room back over `medium` (`standard_accel`), taking no press while it goes; one that changes place glides there over `medium` (`standard_decel`). Only while the list shows: an update nobody sees draws nothing. It scrolls only when the person scrolls it or moves with the keys, so a re-sort never carries the rows they were looking at out of sight. The `folded` rows at its end are a drawer under a disclosure line, open while `unfolded`: its edge slides over `medium` and its rows fade in together a `micro` after, or at once with reduced motion. The keys pass over rows that are neither a `row` nor a `disclosure`, and a closed drawer.

*Not built yet: a row arriving or leaving above the row under the pointer or the keyboard moves it; the mockup holds that row still, the list scrolling with the rooms above it, or the update waiting until the pointer and the keyboard let go. The mockup has no drawer yet.*

## `EmptyList`

Rust `list::empty_list`; QML `EmptyList`. Its mockup page is the source of truth: Molecules, EmptyList, in `design/mockups/`.

The well with nothing in it: one centred line.
