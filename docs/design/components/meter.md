# Meter

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Meter`

Rust `meter::meter`; QML `Meter`.

A reading in parts on a progress bar's track: 8 tall, a pill, each part a share and its colour, left to right (memory as used and cache), every part keeping the bar's rounded ends. The QML one eases a new reading over `short` on `linear`, as a progress bar follows its work. The system monitor's memory, disks and battery, and the bar pill's bars.
