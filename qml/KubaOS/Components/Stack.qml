import QtQuick
import KubaOS.Components as K

// Children one above the other, `gap` apart, each box rounded up to whole device pixels:
// `kubaos-ui`'s `layout::stack`. Start-aligned, or `centred` across all the room there is on whole
// device pixels. A stack grows to the room it is given when a child fills, as iced's column does.
K.Layout {
    id: root

    // The space between children, a spacing token or 0.
    property int gap: 0
    // Each child centred across the stack, which then takes all the room it is given.
    property bool centred: false

    fill: root.centred
    probe: ({
            role: "layout",
            axis: "y",
            gap: root.gap,
            align: root.centred ? "centre" : "start"
        })

    function arrange(kids: list<Item>): void {
        const dpr = root.dpr;
        const snapUp = v => K.Look.snapUp(v, dpr);
        const fills = root.fillWidth;
        const across = (fills ? root.width : root.room) - root.leftPadding - root.rightPadding;
        let widest = 0;
        for (const kid of kids) {
            root.fit(kid, Math.max(across, 0), fills ? snapUp(across) : NaN);
            kid.height = snapUp(kid.implicitHeight);
            widest = Math.max(widest, kid.width);
        }
        const cross = fills ? across : widest;
        let y = root.topPadding;
        for (const kid of kids) {
            kid.x = root.leftPadding + (root.centred ? Math.max(K.Look.snap((cross - kid.width) / 2, dpr), 0) : 0);
            kid.y = y;
            y += kid.height + root.gap;
        }
        const content = kids.length > 0 ? y - root.gap : y;
        root.implicitWidth = snapUp(root.leftPadding + cross + root.rightPadding);
        root.implicitHeight = snapUp(content + root.bottomPadding);
    }
}
