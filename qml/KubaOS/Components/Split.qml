import QtQuick
import KubaOS.Components as K

// Columns as wide as their share of `weights`, `gap` apart both ways, filled row by row:
// `kubaos-ui`'s `layout::split`. `[1, 3]` is a label column a quarter wide beside the rest. The
// width is split in device pixels, as a `Grid`'s is, so every edge lands on one.
K.Grid {
}
