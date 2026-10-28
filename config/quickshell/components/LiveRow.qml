import QtQuick
import KubaOS.Components as K

// A `LiveList`'s delegate: its row (the child), `rowHeight` tall, and the 4 after it. Added to the
// list while it shows, it opens its room over `medium` and fades in over `slow`; removed, it stays
// in the view, unpressable, while it fades and gives its room back over `medium`, then goes.
Item {
    id: root

    // The row's own height, without the gap after it.
    property real rowHeight: K.Tokens.size.row_two_line
    // How much of its room the row has: 0 while it arrives or leaves, else 1.
    property real grow: 1
    // Whether this is the list's current row, for a row's `selected`.
    readonly property bool current: ListView.isCurrentItem
    readonly property bool leaving: leave.running
    // The leave, which a screenshot holds half-way.
    readonly property alias leaveAnimation: leave

    width: ListView.view ? ListView.view.width : 0
    height: (root.rowHeight + K.Tokens.space.s4) * root.grow
    clip: root.grow < 1

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
            property: "opacity"
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
                property: "opacity"
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
