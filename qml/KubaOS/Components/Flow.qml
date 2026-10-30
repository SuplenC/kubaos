import QtQuick
import KubaOS.Components as K

// Children side by side, `gap` apart both ways, a new line started where the next one doesn't fit:
// chips, tags. `kubaos-ui`'s `layout::flow`, laid out as iced's wrapping row is: each child is
// measured in the whole room, and goes on the line it fits after the ones before it. Each box is
// rounded up to whole device pixels, so every child starts on one. As wide as its widest line; the
// lines tell the geometry check nothing, and each child still says what it is.
K.Layout {
    id: root

    // The space between children and between lines, a spacing token or 0.
    property int gap: 0

    function arrange(kids: list<Item>): void {
        const snapUp = v => K.Look.snapUp(v, root.dpr);
        const room = Math.max((root.fillWidth ? root.width : root.room) - root.leftPadding - root.rightPadding, 0);
        let x = 0;
        let y = 0;
        let line = 0;
        let widest = 0;
        for (const kid of kids) {
            root.fit(kid, room, NaN);
            kid.height = snapUp(kid.implicitHeight);
            if (x > 0 && x + kid.width > room) {
                widest = Math.max(widest, x - root.gap);
                y += line + root.gap;
                x = 0;
                line = 0;
            }
            line = Math.max(line, kid.height);
            kid.x = root.leftPadding + x;
            kid.y = root.topPadding + y;
            x += kid.width + root.gap;
        }
        if (x > 0)
            widest = Math.max(widest, x - root.gap);
        root.implicitWidth = snapUp(root.leftPadding + widest + root.rightPadding);
        root.implicitHeight = snapUp(root.topPadding + y + line + root.bottomPadding);
    }
}
