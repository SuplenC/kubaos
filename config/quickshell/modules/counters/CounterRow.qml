import QtQuick
import KubaOS.Components as K

// A counter's row: its name, the line of its auto reset under it, then at its end its value,
// rolling as it changes, − (Tonal, disabled at 0) and + (Filled), which turns into a reset once
// the counter has reached its stop. A double-click on the name asks for a new one, on the value
// for a new value; a right-click opens its menu. The sheet does what each asks.
Item {
    id: root

    required property var modelData
    required property int index
    // The value box's widest value, the same for every row.
    property string widest: "0"
    // The keyboard is on it, or its menu is open.
    property bool lit: false

    readonly property var counter: root.modelData
    readonly property bool atStop: root.counter.stop > 0 && root.counter.value >= root.counter.stop
    // What a prompt for its name or its value floats under.
    readonly property Item nameAnchor: nameBox
    readonly property Item valueAnchor: number

    signal picked
    signal plus
    signal minus
    signal renameAsked
    signal setAsked
    // At `at`, in the window's content coordinates.
    signal menuAsked(point at)

    width: ListView.view ? ListView.view.width : 0
    // Every row is a two-line row's height, a counter with no auto reset too, so they share one.
    implicitHeight: K.Tokens.size.row_two_line

    K.ListRow {
        id: row

        width: root.width
        height: root.height
        title: root.counter.name
        subtitle: root.counter.detail
        subtitleIcon: root.counter.detail !== "" ? "repeat" : ""
        selected: root.lit
        onActivated: root.picked()

        K.RollingNumber {
            id: number

            value: root.counter.value
            widest: root.widest
            color: row.main

            HoverHandler {
                id: valueHover
            }

            TapHandler {
                onDoubleTapped: root.setAsked()
            }

            K.Hint {
                text: "Double-click to set"
                shown: valueHover.hovered
            }
        }

        K.Row {
            gap: K.Tokens.space.s8
            align: "centre"

            K.IconButton {
                icon: "remove"
                label: "−1"
                kind: "Tonal"
                size: "Sm"
                enabled: root.counter.value > 0
                activeFocusOnTab: false
                onActivated: root.minus()
            }

            K.IconButton {
                icon: root.atStop ? "restart_alt" : "add"
                label: root.atStop ? `Reached ${root.counter.stop}: reset to 0` : "+1"
                kind: "Filled"
                size: "Sm"
                activeFocusOnTab: false
                onActivated: root.plus()
            }
        }
    }

    // The name's room, as wide as the name shows: its hint, and a double-click to rename it.
    Item {
        id: nameBox

        parent: row.titleItem
        width: Math.min(row.titleItem.naturalWidth, row.titleItem.width)
        height: row.titleItem.height

        HoverHandler {
            id: nameHover
        }

        TapHandler {
            onDoubleTapped: root.renameAsked()
        }

        K.Hint {
            text: row.titleItem.cut ? `${root.counter.name}. Double-click to rename` : "Double-click to rename"
            shown: nameHover.hovered
            // It reads from where the name starts, which may run half the row.
            held: "start"
        }
    }

    // A handler, not a MouseArea over the row: a MouseArea always sets a cursor of its own, an
    // arrow, which would win over the hand of the − and + under it.
    TapHandler {
        acceptedButtons: Qt.RightButton
        onTapped: (point, button) => root.menuAsked(root.mapToItem(null, point.position.x, point.position.y))
    }
}
