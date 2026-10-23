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
    // Off for content that animates its own width: the pill then follows it
    // frame by frame, where a second animation would lag behind it.
    property bool animatesWidth: true
    property string tooltip
    readonly property bool hovered: stateLayer.containsMouse

    signal clicked
    signal scrolled(int steps)

    implicitWidth: row.implicitWidth + padding * 2
    implicitHeight: Theme.bar.pillHeight
    radius: Theme.rounding.full
    color: Theme.m3surfaceContainer

    Behavior on implicitWidth {
        enabled: root.animatesWidth

        Anim {
            duration: Theme.anim.large
            easing.bezierCurve: Theme.anim.spatial
        }
    }

    // While the pill's width animates to fit new content, the content is cut
    // at the pill's edge rather than drawn over its neighbours. Tooltips are
    // popups, so this clip never cuts them.
    Item {
        anchors.fill: parent
        clip: true

        Row {
            id: row

            anchors.centerIn: parent
            spacing: Theme.spacing.sm
        }
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
