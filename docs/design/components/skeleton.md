# Skeleton

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Skeleton`

Rust `skeleton::skeleton`; QML `Skeleton`.

The shape of what is loading, held in its place: a `line` of text in a style (as wide as its room, the style's line tall, the small radius), a `circle` an icon size across, or a `block` a size tall at a radius role. In the placeholder colour a `Media` box holds, `surface_container_high`; in QML a sheen of `surface_container_highest` sweeps across it, left to right, over `slow` on `standard` and then rests for `slow`, three times as it shows, then it holds still, so a long wait draws no frames. The sheen is the shape's own fill, so its corners cut it. With reduce motion it holds still from the start. A Wi-Fi or Bluetooth scan with nothing found yet shows skeleton rows, and the system monitor's processes show skeleton bars until they are counted; a `Media` box's placeholder is a block skeleton of its own.
