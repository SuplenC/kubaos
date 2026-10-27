import QtQuick
import KubaOS.Components as K

// Children side by side, `gap` apart, each box rounded up to whole device pixels, so every child
// starts on one: `kubaos-ui`'s `layout::row`. A child that fills (`fillWidth`, as `Fill` does)
// takes the room the others leave, and makes the row take all the room it is given.
//
// Laid out as iced's flex lays out a row: the children that don't fill are measured one after
// another, each with the room the ones before it left; those that fill share what remains.
K.Layout {
    id: root

    // The space between children, a spacing token or 0.
    property int gap: 0
    // How the children line up down the row: `start`, `centre` or `end`.
    property string align: "start"
    // A height of its own, a size token, or 0 for its children's.
    property real fixedHeight: 0

    probe: ({
            role: "layout",
            axis: "x",
            gap: root.gap,
            align: root.align
        })

    function arrange(kids: list<Item>): void {
        const dpr = root.dpr;
        const snapUp = v => K.Look.snapUp(v, dpr);
        const fills = root.fillWidth;
        let available = (fills ? root.width : root.room) - root.leftPadding - root.rightPadding - root.gap * Math.max(kids.length - 1, 0);
        let cross = 0;
        let filling = 0;
        for (const kid of kids) {
            if (fills && kid.fillWidth === true) {
                filling += 1;
                continue;
            }
            root.fit(kid, Math.max(available, 0), NaN);
            kid.height = snapUp(kid.implicitHeight);
            available -= kid.width;
            cross = Math.max(cross, kid.height);
        }
        const share = filling > 0 ? Math.max(available, 0) / filling : 0;
        for (const kid of kids) {
            if (!(fills && kid.fillWidth === true))
                continue;
            root.fit(kid, share, snapUp(share));
            kid.height = snapUp(kid.implicitHeight);
            cross = Math.max(cross, kid.height);
        }
        if (root.fixedHeight > 0)
            cross = root.fixedHeight - root.topPadding - root.bottomPadding;
        let x = root.leftPadding;
        for (const kid of kids) {
            const spare = cross - kid.height;
            const down = root.align === "centre" ? K.Look.snap(spare / 2, dpr) : root.align === "end" ? spare : 0;
            kid.x = x;
            kid.y = root.topPadding + down;
            x += kid.width + root.gap;
        }
        const content = kids.length > 0 ? x - root.gap : x;
        root.implicitWidth = snapUp(content + root.rightPadding);
        root.implicitHeight = root.fixedHeight > 0 ? root.fixedHeight : snapUp(root.topPadding + cross + root.bottomPadding);
    }
}
