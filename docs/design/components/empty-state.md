# EmptyState

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `EmptyState`

Rust `empty::empty_state`; QML `EmptyState`. Its mockup page is the source of truth: Molecules, EmptyState, in `design/mockups/`.

A place with nothing in it that isn't a list (a page, a panel, a picker, the Command menu's results), centred: a 32 icon in `on_surface_variant`, 12 below it a `subheading` title, 4 below that a `body` line in `on_surface_variant` saying what to do, and 16 below that an optional action, a `Tonal` button where something fixes it at once (Bluetooth's Turn on). Its words wrap, centred, in a column `size.column` wide at most. Never a bare "No items". It sits in the page, so it has no box and isn't raised. A list's well with nothing in it is an `EmptyList` instead ([List](list.md)): one line in the well, no symbol, since the list round it already says what it holds.

*Not built yet: neither library holds the words to `size.column`.*
