import QtQuick
import QtQuick.Effects

// Clips its children to its rounded shape, as Quickshell's does: a plain `clip` would cut them
// square, and a shot would show corners the shell never draws. The children stay visible, so
// positioners inside lay them out as they would in the shell.
Item {
    id: root

    property real contentInsideBorder: 0
    property real radius: 0
    property color color: "transparent"
    property alias border: back.border
    default property alias content: inner.data

    Rectangle {
        id: back

        anchors.fill: parent
        radius: root.radius
        color: root.color
    }

    Rectangle {
        id: shape

        anchors.fill: parent
        radius: root.radius
        visible: false
        layer.enabled: true
    }

    Item {
        id: inner

        anchors.fill: parent
        layer.enabled: true
        layer.effect: MultiEffect {
            maskEnabled: true
            maskSource: shape
            maskThresholdMin: 0.5
            maskSpreadAtMin: 1
        }
    }
}
