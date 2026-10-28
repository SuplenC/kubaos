# Skeleton

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Skeleton`

Rust `skeleton::skeleton`; QML `Skeleton`.

The shape of what is loading, held in its place: a `line` of text in a style (as wide as its room, the style's line tall, the small radius), a `circle` an icon size across, or a `block` a size tall at a radius role. In the placeholder colour a `Media` box holds, `surface_container_high`; in QML it breathes between full and `subdued` over `slow` three times as it shows, then holds still, so a long wait draws no frames. A Wi-Fi or Bluetooth scan with nothing found yet shows skeleton rows; a `Media` box's placeholder is a block skeleton of its own.
