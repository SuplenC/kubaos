import QtQuick
import qs.theme

// Outline shown around the parent while it has keyboard focus.
Rectangle {
    property bool shown: parent?.activeFocus ?? false

    anchors.fill: parent
    anchors.margins: -3
    radius: (parent?.radius ?? 0) + 3
    color: "transparent"
    border.width: 2
    border.color: Theme.m3primary
    opacity: shown ? 1 : 0
    visible: opacity > 0

    Behavior on opacity {
        Anim {
            duration: Theme.anim.fast
        }
    }
}
