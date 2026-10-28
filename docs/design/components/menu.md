# Popover, Menu and MenuItem

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Popover`

Rust `menu::popover`; QML `Popover`.

The floating surface a menu sits on: `surface_container`, a soft hairline, the panel's shadow, the surface radius (24), 12 inside, so its items take 24 − 12 = 12, the control radius. It floats over the window at its anchor (a tray icon, the pointer, under a select), a menu's width (240, `menu`) or its select's; `kubaos-ui` draws one where it is laid out.

## `Menu`, `MenuItem`

Rust `menu::menu`, `menu::menu_item`, `menu::menu_separator`; QML `Menu`, `MenuItem`.

A few actions or choices one under the other. An item is a small control tall (32), 12 inside: an optional 20 symbol (`icon`), its words in `body` on one line, the keys that do the same as keycaps, and a check whose room is kept when unchecked (`check`). The item the keyboard is on takes `secondary_container`; one that destroys something (`danger`) is in `error`; one that can't be taken is dimmed. A hairline 4 clear of each side separates groups. ↑ ↓ (j k, Ctrl+J/K/N/P), Home and End move over the items that can be taken, Enter takes one, Esc leaves. A menu taller than its room (`maxHeight`: a select's eight items, the room under a tray icon) scrolls, the item the keyboard is on kept in view. The tray's app menus, where a submenu opens in place with a Back item at its top and Esc steps back one level before it closes, and a select's list.
