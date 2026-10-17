import QtQuick
import qs.theme

// Hover/press overlay plus click handling for any rounded surface.
// Place inside the surface; it fills the parent and follows its radius.
MouseArea {
    id: root

    property color color: Theme.m3onSurface
    property real radius: parent?.radius ?? 0

    anchors.fill: parent
    hoverEnabled: true
    cursorShape: enabled ? Qt.PointingHandCursor : Qt.ArrowCursor

    Rectangle {
        anchors.fill: parent
        radius: root.radius
        color: root.color
        opacity: !root.enabled ? 0 : root.pressed ? Theme.state.press : root.containsMouse ? Theme.state.hover : 0

        Behavior on opacity {
            Anim {
                duration: Theme.anim.fast
                easing.bezierCurve: Theme.anim.effects
            }
        }
    }
}
