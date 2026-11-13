# Popover, Menu and MenuItem

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Popover`

Rust `menu::popover`; QML `Popover`.

The floating surface a menu sits on: `surface_container`, raised to the ladder's step 2 (its shadow and rim), at the control radius (12). Holding a menu it is 4 inside, so its items take 12 − 4 = 8, the row radius, and run parallel to its corners; holding words of its own it is 12 inside. It floats over the window at its anchor (a tray icon, the pointer), a menu's width (240, `menu`); `kubaos-ui` draws one where it is laid out. It fades in over `short` on `emphasized_decel` and out over `micro` on `emphasized_accel`.

A popover's menu and a select's open list (`select.md`) are one build, the rule for every raised list ([DESIGN.md, "Radii"](../../DESIGN.md#radii-by-role)): 12 corners and 4 inside, rows 32 tall at the row radius (8) and 8 inside, so their words start 12 from the surface's edge, and a hairline between groups inset to the words.

## `Menu`, `MenuItem`

Rust `menu::menu`, `menu::menu_item`, `menu::menu_separator`; QML `Menu`, `MenuItem`.

A few actions or choices one under the other. An item is a small control tall (32), 8 inside, at the row radius (8): an optional 16 symbol (`icon`, `icon.s16`) 8 from its words, its words in `body` on one line, the keys that do the same as keycaps at its end, and a 16 check whose room is kept when unchecked (`check`). The item the keyboard is on takes `secondary_container`, and its keycaps' keys and edges turn `primary` with it, over the same `short`; one that destroys something (`danger`) is in `error` and lights in `error` itself with `on_error`, not the secondary container, its keycaps unchanged; one that can't be taken is dimmed, its keycaps with it. A hairline 4 clear of each side separates groups, inset 8 so it lines up with the words, 12 from the popover's edge. ↑ ↓ (j k, Ctrl+J/K/N/P), Home and End move over the items that can be taken, Enter takes one, Esc leaves. A menu taller than its room (`maxHeight`: the room under a tray icon) scrolls, the item the keyboard is on kept in view. An item that opens a submenu ends in a 16 `chevron_right` in its ink (`submenu`). An item may carry a short description, a second line under its words in `small`, which makes it a two-line row's height (`row_two_line`, 56; `two-line`), and a 16 symbol at its end (`trailing`, before its keys or check); both are `on_surface_variant`, and lit `on_secondary_container` at `opacity.subdued_text`, a list row's subtitle's. The tray's app menus, where a submenu opens in place with a Back item at its top and Esc steps back one level before it closes; the popover is padded 4 round the menu, as every menu's is. A select's options are menu items too, on the select's own surface as it grows into its list (`select.md`), not on a popover.

**States:** an item resting, hovered, pressed, highlighted (the keyboard on it: the secondary container), checked, `danger`, and disabled (no action, `enabled: false`).
