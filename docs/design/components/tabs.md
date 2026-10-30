# Tabs

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Tabs`

Rust `tabs::tabs`; QML `Tabs`.

One of a few views or values, picked from a row of small buttons (`Sm`) 4 apart: the picked one `Tonal`, the rest `Text`, each with an optional symbol before its label. It is the segmented control too: a setting's few choices side by side (the wallpaper's motion, Always, On AC, Never). The row is one stop for Tab; ← and → (or h and l) pick the neighbour (the owner moves the pick in iced; `Tabs` in QML does it), and while the row has the keyboard the focus tube lights the picked one. The control center's sections, the clock's tools, the wallpaper picker's folders, the system monitor's order.

**States:** one choice picked (`Tonal`) and the rest not (`Text`), each choice resting, hovered and pressed; focused, the tube on the picked one; and disabled (`Tabs::disabled`, `enabled: false`): every choice dimmed, the picked one still tonal, no press, and no stop for Tab.
