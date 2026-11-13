# Popover, Menu and MenuItem

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Popover`

Rust `menu::popover`; QML `Popover`.

The floating surface a menu sits on: `surface_container`, raised to the ladder's step 2 (its shadow and rim), at the control radius (12). Holding a menu it is 4 inside, so its items take 12 − 4 = 8, the row radius, and run parallel to its corners; holding words of its own it is 12 inside. It floats over the window at its anchor (a tray icon, the pointer), a menu's width (240, `menu`); `kubaos-ui` draws one where it is laid out. It fades in over `short` on `emphasized_decel` and out over `micro` on `emphasized_accel`.

A popover's menu and a select's open list (`select.md`) are one build, the rule for every raised list ([DESIGN.md, "Radii"](../../DESIGN.md#radii-by-role)): 12 corners and 4 inside, rows 32 tall at the row radius (8) and 8 inside, so their words start 12 from the surface's edge, and a hairline between groups inset to the words.

## `Menu`, `MenuItem`

Rust `menu::menu`, `menu::menu_item`, `menu::menu_separator`; QML `Menu`, `MenuItem`.

A few actions or choices one under the other. An item is a small control tall (32), 8 inside, at the row radius (8): an optional 16 symbol (`icon`, `icon.s16`) 8 from its words, its words in `body` on one line, the keys that do the same as keycaps at its end, and a 16 check whose room is kept when unchecked (`check`). The item the keyboard is on takes `secondary_container`, and its keycaps' keys and edges turn `primary` with it, over the same `short`; one that destroys something (`danger`) is in `error` and lights in `error` itself with `on_error`, not the secondary container, its keycaps unchanged; pressed, it takes `error_pressed` in place of the state layer's press; one that can't be taken is dimmed, its keycaps with it. A hairline 4 clear of each side separates groups, inset 8 so it lines up with the words, 12 from the popover's edge. ↑ ↓ (j k, Ctrl+J/K/N/P), Home and End move over the items that can be taken, Enter takes one, Esc leaves. A menu taller than its room (`maxHeight`: the room under a tray icon) scrolls, the item the keyboard is on kept in view. An item that opens a submenu ends in a 16 `chevron_right` in its ink (`submenu`). The tray's app menus, where a submenu opens in place with a Back item at its top and Esc steps back one level before it closes; the popover is padded 4 round the menu, as every menu's is. A select's options are menu items too, on the select's own surface as it grows into its list (`select.md`), not on a popover.

**States:** an item resting, hovered, pressed, highlighted (the keyboard on it: the secondary container), checked, `danger`, and disabled (no action, `enabled: false`).

### Held items

Rust `menu_item(label).held(reach, start, end)`; QML a `held` entry (`Menu`), `MenuItem.held`.

An item that destroys something that can't be taken back (a counter's Hold to remove, with its history) is held, never clicked: it is taken only once it has been held for `duration.hold` (1100), by the pointer pressed on it or by Enter kept down while the keyboard is on it. It is a danger item (`danger`), in `error` lit or not; lit, it lies on `error` at `opacity.tint`, so the sweep has room to show. While it is held, `error` sweeps across it from its start on `linear`, its words, symbol and check `on_error` under the sweep and cut at its straight edge, so they read on either side of it; it is taken when the sweep reaches its end. Let go, left by the pointer or by the keyboard before then, the sweep runs back over `short` on `standard`. Pressed again part of the way back, the sweep goes on from where it is, over what is left of `hold`. The sweep is a timer, not a move: reduce motion keeps it. In `kubaos-ui` the app times the hold and gives the item how far along it is (`reach`), as with a held `Button`; `KubaOS.Components` times it itself and sends `activated` once it is taken.
