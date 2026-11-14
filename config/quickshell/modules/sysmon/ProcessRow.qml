import QtQuick
import KubaOS.Components as K
import qs.services
import "panel.js" as Panel

// One row of the system monitor's processes: a process, or its placeholder while the list is
// counted, the process fading in over it so nothing moves; or empty, past the end of a short list.
// While its process is the one shown, the details open under it.
Item {
    id: root

    // The processes it is a row of: their columns, the process shown, and what a click does.
    required property Processes owner
    property var process: null
    // `process`, `placeholder` or `empty` (`panel.js`, `slot`).
    property string kind: "process"
    // Which row it is, so neighbouring placeholders differ.
    property int slotIndex: 0

    readonly property bool picked: root.process !== null && root.owner.details !== null && root.owner.details.pid === root.process.pid
    readonly property bool fillWidth: true
    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    // How much of the details shows: their height once open, 0 shut.
    property real opened: root.picked && details.item ? details.item.implicitHeight : 0
    // The fade from placeholder to process, a live list's new row's; with reduce motion, short.
    readonly property int fade: K.Motion.reduced ? K.Tokens.duration.short : K.Tokens.duration.slow

    implicitHeight: K.Tokens.size.row_compact + root.opened

    // Lays out what it holds when the processes lay it out, as a `Table` does its rows: their
    // cells take the width they are given only then.
    function relayout(): void {
        bars.relayout();
        row.relayout();
        details.item?.relayout();
    }

    Behavior on opened {
        NumberAnimation {
            duration: K.Motion.reduced ? K.Tokens.duration.instant : K.Tokens.duration.medium
            easing.type: Easing.BezierSpline
            easing.bezierCurve: K.Tokens.easing.standard_decel
        }
    }

    // The row's shape while the processes are counted: a bar in each column where its words or
    // its number will be, on the row's line, as a `TableRow` lays its cells out.
    K.Split {
        id: bars

        x: K.Tokens.space.s12
        y: K.Tokens.space.s4
        width: root.width - K.Tokens.space.s12 * 2
        weights: root.owner.columns.map(column => column.weight)
        gap: K.Tokens.space.s12
        visible: opacity > 0
        opacity: root.kind === "placeholder" ? 1 : 0

        Behavior on opacity {
            enabled: root.owner.fades

            NumberAnimation {
                duration: root.fade
                easing.type: Easing.BezierSpline
                easing.bezierCurve: K.Tokens.easing.standard
            }
        }

        Repeater {
            model: root.owner.columns.length

            Item {
                id: cell

                required property int index
                readonly property bool numeric: root.owner.columns[cell.index].numeric ?? false
                readonly property bool fillWidth: true

                implicitHeight: K.Tokens.type.body.line

                K.Skeleton {
                    x: cell.numeric ? cell.width - width : 0
                    width: K.Look.snap(cell.width * Panel.share(root.slotIndex + cell.index, cell.numeric), root.dpr)
                    style: cell.numeric ? K.Tokens.type.mono_large : K.Tokens.type.body
                    reported: false
                    loading: root.kind === "placeholder"
                }
            }
        }
    }

    K.TableRow {
        id: row

        width: root.width
        columns: root.owner.columns
        cells: root.process ? Panel.cells(root.process, Sysmon.bytes(root.process.memory ?? 0)) : []
        selected: root.picked
        visible: opacity > 0
        enabled: root.kind === "process"
        opacity: root.kind === "process" ? 1 : 0
        onActivated: root.owner.toggle(root.process.pid)

        Behavior on opacity {
            enabled: root.owner.fades

            NumberAnimation {
                duration: root.fade
                easing.type: Easing.BezierSpline
                easing.bezierCurve: K.Tokens.easing.standard
            }
        }
    }

    Item {
        y: K.Tokens.size.row_compact
        width: root.width
        height: root.opened
        // Shown as soon as its process is, so the details lay out and give their height to open to.
        visible: root.picked || root.opened > 0
        clip: true

        // Made while its process is shown, and kept while it closes.
        Loader {
            id: details

            width: parent.width
            active: root.picked || root.opened > 0

            sourceComponent: ProcessDetails {
                owner: root.owner
                width: details.width
            }
        }
    }
}
