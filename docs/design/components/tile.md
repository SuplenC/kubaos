# Tile

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Tile`

Rust `tile::tile`; QML `Tile`.

A quick toggle: a 20 symbol, a `body_strong` title over a `small` status line, as tall as a two-line row (56) at the control radius, 12 inside. On, it fills with `primary` and its content takes `on_primary`; off, `surface_container_high`. A press toggles it. A tile with more behind it ends in a detail arrow (`detail`), a 32 circle 12 from its end with a 16 chevron, which opens it; → does too. The arrow is a small control and stays quiet: no fill, in the tile's own content colour, its state layer under the pointer, and the tile's own layer gives way while the pointer is on it. The control center's Wi-Fi, Bluetooth and the rest.

**States:** off and on, each resting, hovered, pressed, focused and disabled (no action, `enabled: false`); its detail arrow resting, hovered and pressed.
