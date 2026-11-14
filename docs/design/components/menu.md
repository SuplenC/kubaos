# Popover, Menu and MenuItem

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Popover`

Rust `menu::popover`; QML `Popover`.

The floating surface a menu sits on: `surface_container`, a soft hairline, the panel's shadow, the surface radius (24), 12 inside, so its items take 24 − 12 = 12, the control radius. It floats over the window at its anchor (a tray icon, the pointer, under a select), a menu's width (240, `menu`) or its select's; `kubaos-ui` draws one where it is laid out.

## `Menu`, `MenuItem`

Rust `menu::menu`, `menu::menu_item`, `menu::menu_separator`; QML `Menu`, `MenuItem`.

A few actions or choices one under the other. An item is a small control tall (32), 12 inside: an optional 20 symbol (`icon`), its words in `body` on one line, an optional value of its own (`value`), the keys that do the same as keycaps, and a check whose room is kept when unchecked (`check`). A value is a figure the item is about, set apart 12 after its words in `mono_large`, in the item's ink: a counter's Auto reset reads "When it reaches" with its stop, 10, at the end, and taking the item opens the `Prompt` that changes it under that figure, lined up with its end (`valueItem` in QML), not under the whole menu. The words keep their room and are cut before the value is. The item the keyboard is on takes `secondary_container`; one that can't be taken is dimmed. One that destroys something (`danger`) is in `error`, and the keyboard on it lights it in `error` itself, its words, symbol and keys `on_error`, so lit it reads as what it is; pressed, it takes `error_pressed` in place of the state layer's press. A hairline 4 clear of each side separates groups. ↑ ↓ (j k, Ctrl+J/K/N/P), Home and End move over the items that can be taken, Enter takes one, Esc leaves. A menu taller than its room (`maxHeight`: a select's eight items, the room under a tray icon) scrolls, the item the keyboard is on kept in view. The tray's app menus, where a submenu opens in place with a Back item at its top and Esc steps back one level before it closes, and a select's list.

### Held items

Rust `menu_item(label).held(reach, start, end)`; QML a `held` entry (`Menu`), `MenuItem.held`.

An item that destroys something that can't be taken back (a counter's Hold to remove, with its history) is held, never clicked: it is taken only once it has been held for `duration.hold` (1100), by the pointer pressed on it or by Enter kept down while the keyboard is on it. It is a danger item (`danger`), in `error` lit or not; lit, it lies on `error` at `opacity.tint`, so the sweep has room to show. While it is held, `error` sweeps across it from its start on `linear`, its words, symbol and check `on_error` under the sweep and cut at its straight edge, so they read on either side of it; it is taken when the sweep reaches its end. Let go, left by the pointer or by the keyboard before then, the sweep runs back over `short` on `standard`. Pressed again part of the way back, the sweep goes on from where it is, over what is left of `hold`. The sweep is a timer, not a move: reduce motion keeps it. In `kubaos-ui` the app times the hold and gives the item how far along it is (`reach`), as with a held `Button`; `KubaOS.Components` times it itself and sends `activated` once it is taken.
