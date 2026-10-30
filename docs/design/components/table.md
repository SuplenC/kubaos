# Table and TableRow

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Table`, `TableRow`

Rust `table::table`, `table::table_row`; QML `Table`, `TableRow`.

Readings in columns: a header in `label`, then a row per entry, each a line of body text with 4 above and below (28, `row_compact`), so many fit. Columns take set shares of the width, 12 apart, 12 in from the row's ends; words sit at their column's start on one line, ending in an ellipsis; numbers at its right edge in `mono_large`, so their digits line up. The whole row is one control: the pointer over it lays the state layer across every cell, and the picked row takes `secondary_container`. The system monitor's processes.

**States:** a row resting, hovered, pressed and picked (the secondary container).
