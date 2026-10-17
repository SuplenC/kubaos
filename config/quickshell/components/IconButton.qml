import QtQuick
import qs.theme

// Round icon button. `toggled` fills it with the primary color (an "on"
// state). Pressing squashes the circle into a rounded square, M3-expressive style.
Rectangle {
    id: root

    property string icon
    property bool toggled: false
    property real size: 36
    property color baseColor: "transparent"
    property string tooltip
    property color tint: Theme.m3onSurfaceVariant
    readonly property color iconColor: toggled ? Theme.m3onPrimary : tint

    signal clicked

    implicitWidth: size
    implicitHeight: size
    radius: stateLayer.pressed ? size * 0.3 : size / 2
    color: toggled ? Theme.m3primary : baseColor
    activeFocusOnTab: true

    Keys.onReturnPressed: clicked()
    Keys.onEnterPressed: clicked()
    Keys.onSpacePressed: clicked()

    Behavior on color {
        CAnim {}
    }

    Behavior on radius {
        Anim {
            duration: Theme.anim.small
            easing.bezierCurve: Theme.anim.spatialFast
        }
    }

    Icon {
        anchors.centerIn: parent
        name: root.icon
        size: Math.round(root.size * 0.55)
        color: root.iconColor
        filled: root.toggled

        Behavior on color {
            CAnim {}
        }
    }

    StateLayer {
        id: stateLayer

        color: root.iconColor
        onClicked: root.clicked()
    }

    Tooltip {
        text: root.tooltip
        shown: root.tooltip !== "" && stateLayer.containsMouse && !stateLayer.pressed
    }

    FocusRing {}
}
