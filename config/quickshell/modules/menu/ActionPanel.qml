import QtQuick
import qs.theme
import qs.components

// The Action panel, Ctrl+K: the selected row's Actions, the Item's own first,
// then the Contributed ones under the Extension offering each, with their
// shortcuts. Menu.qml gives it the keys; the pointer picks too.
Rectangle {
    id: panel

    property MenuLook look
    // Each `{title, group, shortcut}`, as `KotodamaSearch.actions` gives them.
    property var actions: []
    // The entry Enter runs.
    property int current: 0

    // The pointer chose entry `n`.
    signal picked(int n)

    implicitHeight: entries.implicitHeight + 2 * look.gap
    radius: look.cardRadius
    color: Theme.m3surfaceContainerHigh
    border.width: 1
    border.color: Theme.alpha(Theme.m3outlineVariant, 0.8)

    Column {
        id: entries

        x: panel.look.gap
        y: panel.look.gap
        width: panel.width - 2 * panel.look.gap

        Repeater {
            model: panel.actions

            Column {
                id: entry

                required property int index
                required property var modelData
                readonly property bool heading: modelData.group !== "" && (index === 0 || panel.actions[index - 1].group !== modelData.group)
                readonly property bool current: index === panel.current

                width: entries.width

                StyledText {
                    visible: entry.heading
                    height: visible ? panel.look.headingHeight : 0
                    leftPadding: panel.look.inset
                    bottomPadding: panel.look.nudge
                    verticalAlignment: Text.AlignBottom
                    text: panel.look.kindLabel(entry.modelData.group)
                    color: Theme.m3onSurfaceVariant
                    font.pixelSize: panel.look.font.small
                    font.weight: Font.Medium
                }

                Rectangle {
                    width: parent.width
                    height: panel.look.smallRowHeight
                    radius: panel.look.itemRadius
                    color: entry.current ? Theme.m3secondaryContainer : "transparent"

                    StyledText {
                        anchors.left: parent.left
                        anchors.leftMargin: panel.look.inset
                        anchors.right: shortcut.left
                        anchors.rightMargin: panel.look.gap
                        anchors.verticalCenter: parent.verticalCenter
                        text: entry.modelData.title
                        color: entry.current ? Theme.m3onSecondaryContainer : Theme.m3onSurface
                        font.pixelSize: panel.look.font.title
                    }

                    Keycap {
                        id: shortcut

                        anchors.right: parent.right
                        anchors.rightMargin: panel.look.inset
                        anchors.verticalCenter: parent.verticalCenter
                        look: panel.look
                        visible: key !== ""
                        tint: entry.current ? Theme.m3onSecondaryContainer : Theme.m3onSurfaceVariant
                        key: entry.index === 0 ? "↵" : panel.look.shortcutLabel(entry.modelData.shortcut)
                    }

                    MouseArea {
                        anchors.fill: parent
                        hoverEnabled: true
                        onPositionChanged: panel.current = entry.index
                        onClicked: panel.picked(entry.index)
                    }
                }
            }
        }
    }
}
