import QtQuick
import qs.theme
import qs.components
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

    implicitHeight: column.implicitHeight + Theme.spacing.lg

    function move(delta: int): void {
        const n = rows.length;
        current = ((current + delta) % n + n) % n;
    }

    function rowItem(i: int): Item {
        return repeater.count > i ? repeater.itemAt(i)?.item ?? null : null;
    }

    // Keep the focused row on screen, with a row of air around it.
    function reveal(): void {
        const it = repeater.itemAt(current);
        if (!it)
            return;
        const top = it.y - Theme.spacing.xxl;
        const bottom = it.y + it.height + Theme.spacing.xxl;
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
        contentHeight: column.implicitHeight + Theme.spacing.lg
        clip: true
        boundsBehavior: Flickable.StopAtBounds

        Behavior on contentY {
            enabled: !flick.moving

            Anim {
                duration: Theme.anim.normal
                easing.bezierCurve: Theme.anim.standard
            }
        }

        Column {
            id: column

            width: flick.width - Theme.spacing.md
            spacing: Theme.spacing.xxs

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
    // a focus ring while it's the current one.
    component SettingRow: Rectangle {
        id: row

        required property int rowIndex
        property string label
        property string help
        default property alias control: controlSlot.data
        readonly property bool current: root.current === rowIndex
        readonly property bool focused: current && root.activeFocus

        // Key -> handled? Overridden per kind.
        property var handler: (key, shift) => false
        function act(key: int, shift: bool): bool {
            return handler(key, shift);
        }

        implicitHeight: Math.max(52, texts.implicitHeight + Theme.spacing.md * 2)
        radius: Theme.rounding.md
        color: current ? Theme.m3surfaceContainerHigh : "transparent"

        Behavior on color {
            CAnim {}
        }

        FocusRing {
            shown: row.focused
        }

        MouseArea {
            anchors.fill: parent
            onPressed: {
                root.current = row.rowIndex;
                root.forceActiveFocus();
            }
        }

        Column {
            id: texts

            anchors.left: parent.left
            anchors.leftMargin: Theme.spacing.lg
            anchors.right: controlSlot.left
            anchors.rightMargin: Theme.spacing.lg
            anchors.verticalCenter: parent.verticalCenter
            spacing: Theme.spacing.xxs

            StyledText {
                width: parent.width
                text: row.label
                font.weight: Font.Medium
                color: Theme.m3onSurface
            }

            StyledText {
                width: parent.width
                visible: text !== ""
                text: row.help
                font.pixelSize: Theme.font.sm
                color: Theme.m3onSurfaceVariant
                elide: row.current ? Text.ElideNone : Text.ElideRight
                wrapMode: row.current ? Text.WordWrap : Text.NoWrap
                maximumLineCount: row.current ? 3 : 1
            }
        }

        Item {
            id: controlSlot

            anchors.right: parent.right
            anchors.rightMargin: Theme.spacing.md
            anchors.verticalCenter: parent.verticalCenter
            width: Math.round(row.width * 0.5)
            height: childrenRect.height
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

            Segmented {
                anchors.right: parent.right
                options: ["always", "ac", "never"]
                labels: ["Always", "On AC", "Never"]
                value: Motion.mode
                onPicked: v => Motion.setMode(v)
            }
        }
    }

    // Pills side by side, the selected one filled. Clicks pick; keys are
    // the row's.
    component Segmented: Row {
        id: seg

        property list<string> options
        property list<string> labels
        property string value

        signal picked(string value)

        spacing: Theme.spacing.xxs

        Repeater {
            model: seg.options

            delegate: Rectangle {
                id: pill

                required property string modelData
                required property int index
                readonly property bool selected: seg.value === modelData

                implicitWidth: label.implicitWidth + Theme.spacing.lg * 2
                implicitHeight: 28
                radius: 14
                color: selected ? Theme.m3secondaryContainer : Theme.m3surfaceContainerHighest

                Behavior on color {
                    CAnim {}
                }

                StyledText {
                    id: label

                    anchors.centerIn: parent
                    text: seg.labels[pill.index] ?? pill.modelData
                    font.pixelSize: Theme.font.sm
                    font.weight: pill.selected ? Font.DemiBold : Font.Normal
                    color: pill.selected ? Theme.m3onSecondaryContainer : Theme.m3onSurfaceVariant
                }

                StateLayer {
                    onClicked: seg.picked(pill.modelData)
                }
            }
        }
    }
}
