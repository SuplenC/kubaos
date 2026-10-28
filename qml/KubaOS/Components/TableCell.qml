import QtQuick
import KubaOS.Components as K

// One cell of a `Table`'s header or rows: words where its column starts, or a number at its
// column's right edge.
Item {
    id: root

    property alias text: label.text
    property alias style: label.style
    property alias color: label.color
    property bool numeric: false
    // Whether the words stay on one line, ending in an ellipsis where they don't fit.
    property bool oneLine: !root.numeric

    readonly property bool fillWidth: true

    implicitHeight: label.implicitHeight

    K.Text {
        id: label

        x: root.numeric ? root.width - width : 0
        room: root.width
        maximumLines: root.oneLine ? 1 : 0
    }
}
