# IconTile

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `IconTile`

Rust `avatar::icon_tile`; QML `IconTile`.

What an app or a sender looks like in a row: a symbol or a picture on a tinted square of one size, `Sm` 32 beside one line (a 20 icon), `Md` 40 beside a title over a subtitle (24, `icon.two_line`) and `Lg` 48 over a `headline` name, the one thing a view is about (32, the headline's: a network or a device being connected), at the control radius, on `surface_container_highest`, which reads on a list, a card and a selected row alike (`error` for a critical notification). Its symbol is `on_surface_variant`, or `primary` on the one tile of a list that comes first (`accent`: the default keyboard layout's filled star). An app's icon sits inset at the icon size, keeping its own margins; a photo (a notification's image) covers the tile. The symbol shows until the picture has loaded and stays when there is none, so every row reads the same size whatever each vendor drew. The Command menu's rows, the notifications', a `ListRow`'s with `tile` and the control center's connection form.
