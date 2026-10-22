import QtQuick
import qs.theme
import qs.components

// The pane right of the list: the selected row, in full. A new selection
// crossfades in over the last; the same row changing in place, as its
// matched characters do while typing, just updates.
Item {
    id: pane

    property MenuLook look
    // Which of the two contents shows the entry now.
    property bool flipped: false
    readonly property PaneContent shown: flipped ? second : first

    // Shows `entry`, an object as `Menu.qml`'s `paneEntry` builds it, or null
    // for no row.
    function show(entry: var): void {
        if ((entry?.key ?? "") === (shown.entry?.key ?? "")) {
            shown.entry = entry;
            return;
        }
        (flipped ? first : second).entry = entry;
        flipped = !flipped;
    }

    Rectangle {
        anchors.fill: parent
        color: Theme.alpha(Theme.m3surfaceContainerLow, 0.6)
    }

    Rectangle {
        width: 1
        height: parent.height
        color: Theme.alpha(Theme.m3outlineVariant, 0.55)
    }

    PaneContent {
        id: first

        anchors.fill: parent
        look: pane.look
        opacity: pane.flipped ? 0 : 1
        visible: opacity > 0

        Behavior on opacity {
            NumberAnimation {
                duration: pane.look.motion.fade
                easing.type: Easing.BezierSpline
                easing.bezierCurve: pane.look.motion.effects
            }
        }
    }

    PaneContent {
        id: second

        anchors.fill: parent
        look: pane.look
        opacity: pane.flipped ? 1 : 0
        visible: opacity > 0

        Behavior on opacity {
            NumberAnimation {
                duration: pane.look.motion.fade
                easing.type: Easing.BezierSpline
                easing.bezierCurve: pane.look.motion.effects
            }
        }
    }

    EmptyState {
        anchors.centerIn: parent
        visible: pane.shown.entry === null
        icon: "manage_search"
        text: "Nothing selected"
    }
}
