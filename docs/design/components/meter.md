# Meter

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Meter`

Rust `meter::meter`; QML `Meter`. Its mockup page is the source of truth: Molecules, Meter, in `design/mockups/`.

A reading in parts on a progress bar's track: 8 tall, a pill, each part a share and its colour, left to right (memory as used and cache), every part keeping the bar's rounded ends. It has no glow. A new reading eases each part over `short` on `linear`, as a progress bar follows its work. The system monitor's memory, disks and battery across a card, and the bar pill's bars, a small control (`control_sm`) wide beside a 16 symbol.

A reading steps by its threshold in `sysmon.toml`: `primary` while fine, `warning` within its margin of the threshold (10 for memory and the CPU's temperature, 5 for a disk and the battery's draw), `error` past it; its figure beside it takes the same step, and cache keeps `tertiary`. Those are the fixed palette's roles, the same on every wallpaper.

*Not built yet: `kubaos-ui`'s meter doesn't ease a new reading.*
