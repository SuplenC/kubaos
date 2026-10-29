# Select

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Select`

Rust `select::select`; QML `Select`.

One of a list too long for radio rows or tabs: a field's look (40, the control radius, a hairline, `surface_container_high`), the picked option in `body_large`, an optional 20 symbol before it and a chevron at its end. Enter, Space or a click opens its menu on a popover 4 under it, as wide as it, the picked option checked and the keyboard on it; open, its hairline takes the accent and its chevron turns up. ↑ ↓ and Enter pick, Esc or a click elsewhere closes it, and so does whatever closes or hides what holds the select (its panel closed by any route, its page swapped out): in QML the owner calls `dismiss()` as its panel goes, and a select hidden closes its menu itself. In iced the app opens the menu and draws it. The clock's holidays.
