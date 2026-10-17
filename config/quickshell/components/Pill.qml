import QtQuick
import qs.theme

// Rounded container used for bar modules. Children are laid out in a row.
// Set `interactive` for hover/press feedback plus clicked() and scrolled(steps).
Rectangle {
    id: root

    default property alias content: row.data
    property int padding: Theme.spacing.md
    property alias spacing: row.spacing
    property bool interactive: false
    property string tooltip
    readonly property bool hovered: stateLayer.containsMouse

    signal clicked
    signal scrolled(int steps)

    implicitWidth: row.implicitWidth + padding * 2
    implicitHeight: Theme.bar.pillHeight
    radius: Theme.rounding.full
    color: Theme.m3surfaceContainer

    Behavior on implicitWidth {
        Anim {
            duration: Theme.anim.large
            easing.bezierCurve: Theme.anim.spatial
        }
    }

    Row {
        id: row

        anchors.centerIn: parent
        spacing: Theme.spacing.sm
    }

    StateLayer {
        id: stateLayer

        enabled: root.interactive
        onClicked: root.clicked()
        onWheel: wheel => root.scrolled(wheel.angleDelta.y > 0 ? 1 : -1)
    }

    Tooltip {
        text: root.tooltip
        shown: root.tooltip !== "" && stateLayer.containsMouse
    }
}
