# ListDetail

A template: [DESIGN.md, "Levels and where they live"](../../DESIGN.md#levels-and-where-they-live) says what a template is. Its mockup page is the source of truth: Templates, ListDetail, in `design/mockups/`. Its new design is sketched after the mockups; the page mocks what its two builds draw now.

## `ListDetail`

A list beside the detail of its selected row.

**The Command menu** (`config/quickshell/modules/menu/Menu.qml`) is a shell panel sized by its screen: half the screen's width, one to two `size.column`s, on the 4 grid; 8 to 14 rows for about two thirds of the screen's height; two fifths down the room it leaves. The frame's glass in the soft hairline at `radius.surface`, 12 inside and between its parts: the search field, the body, a hairline across the panel, and the footer's keys, a sm control tall. The body is the Results and, on a panel at least `size.dialog` plus `size.rail` wide, the pane: a `Card` two fifths of the room (a rail at the least) with the selected row's 48 symbol, title, subtitle, kind, Actions and keys, crossfading over `short` as the selection moves. Without it, the footer shows the row's keys. It opens over `short` (fading, dropping 8) and closes over `micro`; a pushed View slides 24 and settles from 0.4 over `medium`.

**Kōbō's setup** (`crates/kubaos-kobo/src/setup/view.rs`) is an app's window on `surface`: the side, a third of the window up to `size.column` on the well's fill, holds the header, the cards as two-line `Step`s in a `ScrollingList`, whether it is online, Done for now and the keys; beside it the picked card's page, its head, its words in a `Scroll` and its actions (a `Text` button at their start hangs its padding, its label on the page's text edge, as the [button bar](button-bar.md) says), rising 16 into place and fading in over `long` when a card is picked.

*Not built yet: neither library has the template; each surface draws its own.*
