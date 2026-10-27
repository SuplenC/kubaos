import QtQuick

// Room that grows across a row, pushing what follows to its far end: `kubaos-ui`'s `layout::fill`.
// It never grows down, so the row stays as tall as its content.
Item {
    readonly property bool fillWidth: true
    readonly property var probe: ({
            role: "spacer",
            fill: true
        })
}
