import QtQuick
import KubaOS.Components as K
import qs.services

// The wallpaper settings, the picker's last tab: one row per setting, each a
// label with its help on the left and the control on the right.
//
// Keys: Tab/Shift+Tab or Up/Down move between rows, Left/Right change,
// Space or Enter steps forward, Esc goes back to the grid.
FocusScope {
    id: root

    // Esc on a row: back to the wallpapers.
    signal back

    // The rows, in order: { kind: "motion" }.
    readonly property var rows: [
        {
            kind: "motion"
        }
    ]
    property int current: 0

    implicitHeight: column.implicitHeight + K.Tokens.space.s16

    function move(delta: int): void {
        const n = rows.length;
        current = ((current + delta) % n + n) % n;
    }

    function rowItem(i: int): Item {
        return repeater.count > i ? repeater.itemAt(i)?.item ?? null : null;
    }

    // Keep the focused row on screen, with air around it.
    function reveal(): void {
        const it = repeater.itemAt(current);
        if (!it)
            return;
        const top = it.y - K.Tokens.space.s32;
        const bottom = it.y + it.height + K.Tokens.space.s32;
        if (top < flick.contentY)
            flick.contentY = Math.max(0, top);
        else if (bottom > flick.contentY + flick.height)
            flick.contentY = Math.min(flick.contentHeight - flick.height, bottom - flick.height);
    }

    onCurrentChanged: Qt.callLater(reveal)

    Keys.onPressed: event => {
        const k = event.key;
        if (k === Qt.Key_Tab || k === Qt.Key_Down || k === Qt.Key_J)
            move(1);
        else if (k === Qt.Key_Backtab || k === Qt.Key_Up || k === Qt.Key_K)
            move(-1);
        else if (k === Qt.Key_Escape)
            root.back();
        else {
            const it = rowItem(current);
            if (!it || !it.act(k, (event.modifiers & Qt.ShiftModifier) !== 0))
                return;
        }
        event.accepted = true;
    }

    Flickable {
        id: flick

        anchors.fill: parent
        contentWidth: width
        contentHeight: column.implicitHeight + K.Tokens.space.s16
        clip: true
        boundsBehavior: Flickable.StopAtBounds

        Behavior on contentY {
            enabled: !flick.moving

            NumberAnimation {
                duration: K.Tokens.duration.medium
                easing.type: Easing.BezierSpline
                easing.bezierCurve: K.Tokens.easing.standard
            }
        }

        // The rows stand off the flickable's edges by `focus.offset`; the clip cuts a focused
        // row's glow past that.
        Column {
            id: column

            x: K.Tokens.focus.offset
            y: K.Tokens.focus.offset
            width: flick.width - K.Tokens.focus.offset * 2
            spacing: K.Tokens.space.s4

            Repeater {
                id: repeater

                model: root.rows

                delegate: Loader {
                    id: slot

                    required property var modelData
                    required property int index

                    width: column.width
                    sourceComponent: ({
                            motion: motionRow
                        })[modelData.kind]
                }
            }
        }
    }

    // --- rows ---------------------------------------------------------------

    // A focusable row: label and help on the left, the control on the right,
    // the focus tube while it's the current one.
    component SettingRow: Rectangle {
        id: row

        required property int rowIndex
        property string label
        property string help
        default property alias control: controlSlot.data
        readonly property bool current: root.current === rowIndex
        readonly property bool focused: current && root.activeFocus
        readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1

        // Key -> handled? Overridden per kind.
        property var handler: (key, shift) => false
        function act(key: int, shift: bool): bool {
            return handler(key, shift);
        }

        implicitHeight: Math.max(K.Tokens.size.row_two_line, line.implicitHeight + K.Tokens.space.s12 * 2)
        radius: K.Tokens.radius.control
        color: current ? K.Tokens.color.surface_container_high : "transparent"

        Behavior on color {
            ColorAnimation {
                duration: K.Tokens.duration.micro
                easing.type: Easing.BezierSpline
                easing.bezierCurve: K.Tokens.easing.standard
            }
        }

        K.FocusRing {
            shown: row.focused
            controlRadius: K.Tokens.radius.control
        }

        MouseArea {
            anchors.fill: parent
            onPressed: {
                root.current = row.rowIndex;
                root.forceActiveFocus();
            }
        }

        K.Row {
            id: line

            x: K.Tokens.space.s16
            y: K.Look.snap((row.height - line.implicitHeight) / 2, row.dpr)
            width: row.width - K.Tokens.space.s16 - K.Tokens.space.s12
            gap: K.Tokens.space.s16
            align: "centre"

            K.Stack {
                fill: true
                gap: K.Tokens.space.s4

                K.Text {
                    text: row.label
                    style: K.Tokens.type.body_strong
                }

                K.Text {
                    visible: text !== ""
                    text: row.help
                    style: K.Tokens.type.small
                    color: K.Tokens.color.on_surface_variant
                    maximumLines: row.current ? 3 : 1
                }
            }

            K.Slot {
                id: controlSlot
            }
        }
    }

    Component {
        id: motionRow

        SettingRow {
            rowIndex: parent?.index ?? -1
            label: "Motion"
            help: Motion.mode === "never" ? "Never. Posters only." : Motion.mode === "always" ? "Always, battery be damned. Loops keep moving off the charger." : "On AC. Plugged in it moves; on battery it holds still."
            handler: (key, shift) => {
                const order = ["always", "ac", "never"];
                const i = order.indexOf(Motion.mode);
                if (key === Qt.Key_Left || key === Qt.Key_H)
                    Motion.setMode(order[(i + 2) % 3]);
                else if (key === Qt.Key_Right || key === Qt.Key_L || key === Qt.Key_Space || key === Qt.Key_Return || key === Qt.Key_Enter)
                    Motion.setMode(order[(i + 1) % 3]);
                else
                    return false;
                return true;
            }

            // Clicks pick; the keys are the row's, so the tabs are no stop of their own.
            K.Tabs {
                readonly property list<string> options: ["always", "ac", "never"]

                labels: ["Always", "On AC", "Never"]
                currentIndex: options.indexOf(Motion.mode)
                activeFocusOnTab: false
                onPicked: index => Motion.setMode(options[index])
            }
        }
    }
}
