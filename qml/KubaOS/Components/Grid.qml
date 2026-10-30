import QtQuick
import KubaOS.Components as K

// `columns` equal columns, `gap` apart both ways, filled row by row: `kubaos-ui`'s `layout::grid`.
// The width is split in device pixels, so every edge lands on one and the columns differ by one
// device pixel at most. A grid takes all the room it is given. `Split` is a grid whose columns
// take their share of `weights`.
K.Layout {
    id: root

    property int columns: 1
    // Each column's share of the width: a `Split`'s. Equal columns when empty.
    property list<int> weights: []
    property int gap: 0
    // Each row's cells centred down it, on whole device pixels.
    property bool centred: false

    readonly property list<int> shares: root.weights.length > 0 ? root.weights : Array(Math.max(root.columns, 1)).fill(1)

    fill: true
    probe: ({
            role: "grid",
            columns: root.shares.length,
            gap: root.gap,
            align: root.centred ? "centre" : "start"
        })

    // The columns' left edges and widths in logical pixels: the width split in whole device pixels,
    // the leftover pixels going one each to the first columns.
    function spans(width: real): var {
        const dpr = root.dpr;
        const weights = root.shares.map(w => Math.max(w, 1));
        const total = Math.max(Math.floor(width * dpr + 1e-3), 0);
        const gap = Math.round(root.gap * dpr);
        const room = Math.max(total - gap * (weights.length - 1), 0);
        const sum = weights.reduce((a, b) => a + b, 0);
        const widths = weights.map(w => Math.floor(room * w / sum));
        let leftover = room - widths.reduce((a, b) => a + b, 0);
        for (let i = 0; i < widths.length && leftover > 0; i += 1, leftover -= 1)
            widths[i] += 1;
        let x = 0;
        return widths.map(w => {
            const span = {
                x: x / dpr,
                width: w / dpr
            };
            x += w + gap;
            return span;
        });
    }

    function arrange(kids: list<Item>): void {
        const dpr = root.dpr;
        const snapUp = v => K.Look.snapUp(v, dpr);
        const spans = root.spans(root.width);
        let y = 0;
        for (let first = 0; first < kids.length; first += spans.length) {
            if (first > 0)
                y += root.gap;
            const cells = kids.slice(first, first + spans.length);
            let height = 0;
            cells.forEach((kid, column) => {
                const span = spans[column];
                root.fit(kid, span.width, span.width);
                kid.height = snapUp(kid.implicitHeight);
                kid.x = span.x;
                height = Math.max(height, kid.height);
            });
            height = snapUp(height);
            for (const kid of cells)
                kid.y = y + (root.centred ? K.Look.snap((height - kid.height) / 2, dpr) : 0);
            y += height;
        }
        root.implicitWidth = 0;
        root.implicitHeight = y;
    }
}
