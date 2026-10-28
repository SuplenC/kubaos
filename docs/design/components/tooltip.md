# Tooltip

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Tooltip`

Rust `tooltip::tooltip`, `tooltip::with_tooltip`; QML `Tooltip`.

- **A `Tooltip` explains a control** where a `Hint` only names it: a title in `body_strong`, the body in `small` and `on_surface_variant` 4 under it, and what it holds 8 below (the keys it is about, as `KeyHints`), on a `Popover` as wide as a menu (240). It shows while the pointer is over its control or the keyboard is on it, 8 above the control and starting where it starts, held inside the window: above, since the controls it explains sit in a toolbar at a panel's foot, and from the control's start, since a card centred on a chip at a toolbar's start would hang past the panel. Where there is no room above (a button in a header at the window's top, the notifications panel's), it opens 8 below instead, still from the control's start. In QML it may be held inside a panel rather than the window (`within`), so a card from a button in the panel's corner stays on the panel. Like the hint it floats over the window, no part of the page's layout. It is for a control whose words can't say enough on their own (the keyboard section's Mac accents chip), never for a sentence the page should show.
