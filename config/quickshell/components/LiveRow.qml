import QtQuick
import KubaOS.Components as K

// A `LiveList`'s delegate: its row (the child), `rowHeight` tall, and the 4 after it. Added to the
// list while it shows, it opens its room over `medium` and fades in over `slow`; removed, it stays
// in the view, unpressable, while it fades and gives its room back over `medium`, then goes. A row
// that `folds` is in the list's drawer: it has the room the drawer's edge leaves it, and its row
// is cut at that edge, not squeezed, and it shows as much as the drawer's `drawerShown`.
Item {
    id: root

    // The row's own height, without the gap after it.
    property real rowHeight: K.Tokens.size.row_two_line
    // How much of its room the row has: 0 while it arrives or leaves, else 1.
    property real grow: 1
    // How much of it shows while it arrives or leaves, which the drawer's `drawerShown` scales.
    property real shown: 1
    // Its place in the list, which puts it under or past the drawer's edge.
    required property int index
    // Whether the row is in the list's drawer.
    property bool folds: false
    readonly property var view: ListView.view
    // How much of its room the drawer's edge uncovers: all of the rows above the edge, none
    // below it, and part of the one it crosses. A row leaving the model has no index, and goes
    // as the drawer's first, which the edge of an open drawer still uncovers when it was the last.
    readonly property real uncovered: !root.folds || !root.view ? 1 : Math.max(0, Math.min(1, root.view.drawerEdge * Math.max(1, root.view.drawerCount) - Math.max(0, root.index - root.view.drawerStart)))
    readonly property real drawerShown: root.folds && root.view ? root.view.drawerShown : 1
    // Whether this is the list's current row, for a row's `selected`.
    readonly property bool current: ListView.isCurrentItem
    readonly property bool leaving: leave.running
    // The leave, which a screenshot holds half-way.
    readonly property alias leaveAnimation: leave

    width: root.view ? root.view.width : 0
    height: (root.rowHeight + K.Tokens.space.s4) * root.grow * root.uncovered
    opacity: root.shown * root.drawerShown
    visible: root.uncovered > 0 && root.drawerShown > 0
    enabled: root.uncovered === 1 && root.drawerShown === 1
    clip: root.grow < 1 || root.uncovered < 1

    ListView.onAdd: if (ListView.view.animated)
        enter.start()
    ListView.onRemove: if (ListView.view.animated)
        leave.start()

    ParallelAnimation {
        id: enter

        NumberAnimation {
            target: root
            property: "grow"
            from: 0
            to: 1
            duration: K.Tokens.duration.medium
            easing.type: Easing.BezierSpline
            easing.bezierCurve: K.Tokens.easing.standard_decel
        }

        NumberAnimation {
            target: root
            property: "shown"
            from: 0
            to: 1
            duration: K.Tokens.duration.slow
            easing.type: Easing.BezierSpline
            easing.bezierCurve: K.Tokens.easing.standard_decel
        }
    }

    SequentialAnimation {
        id: leave

        PropertyAction {
            target: root
            property: "ListView.delayRemove"
            value: true
        }

        PropertyAction {
            target: root
            property: "enabled"
            value: false
        }

        ParallelAnimation {
            NumberAnimation {
                target: root
                property: "grow"
                to: 0
                duration: K.Tokens.duration.medium
                easing.type: Easing.BezierSpline
                easing.bezierCurve: K.Tokens.easing.standard_accel
            }

            NumberAnimation {
                target: root
                property: "shown"
                to: 0
                duration: K.Tokens.duration.medium
                easing.type: Easing.BezierSpline
                easing.bezierCurve: K.Tokens.easing.standard_accel
            }
        }

        PropertyAction {
            target: root
            property: "ListView.delayRemove"
            value: false
        }
    }
}
