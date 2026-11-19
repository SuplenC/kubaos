# DescriptionList

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `DescriptionList`

Rust `description::description_list`; QML `DescriptionList`. Its mockup page is the source of truth: Molecules, DescriptionList, in `design/mockups/`.

Facts beside their labels, 8 apart: each label in `small` a quarter of the width (never narrower than its longest word, so it wraps between words), on its value's first baseline, its value in `body` the rest, wrapping, and an optional `mono` line under the value for a path, an email or a version, which break badly wrapped mid-token. Kōbō's Last chance cards. A line may carry one action at its right end, an [IconButton](icon-button.md) (`Text`, sm): copy this fact (Settings › About). A list with any action keeps a column `size.control_sm` (32) wide at the right of every line, 8 from the values, so the action never covers a value and a value too long for what is left wraps; a line without one keeps its column empty. The action is centred down its line and adds nothing to its height; it shows while the pointer is on its line or the keyboard is on it, fading in over `short` on `standard`, and hidden it takes no press. Tab reaches each action in reading order; without actions the list takes no keyboard.

*Not built yet: a line's action, in either library.*
