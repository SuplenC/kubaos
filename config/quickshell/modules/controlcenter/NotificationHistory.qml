import QtQuick
import QtQuick.Layouts
import Quickshell
import qs.theme
import qs.components
import qs.services
import qs.modules.notifications

// The control center's notifications, above the tabs: one slim line saying
// how many there are and whether do not disturb is on, and under it the
// notifications themselves, up to `maxListHeight` before the list scrolls.
// Enter opens one, Delete dismisses it.
Rectangle {
    id: root

    // The most the list takes, so the tabs below keep room on a small screen.
    property real maxListHeight: 240
    readonly property bool empty: Notifs.count === 0
    // The line's height, the clear-all button's, so the line keeps its height
    // when the button goes with the last notification.
    readonly property real lineHeight: 30

    implicitHeight: column.implicitHeight + Theme.spacing.sm * 2
    radius: Theme.rounding.xl
    color: Theme.m3surfaceContainer

    ColumnLayout {
        id: column

        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.margins: Theme.spacing.sm
        spacing: Theme.spacing.sm

        RowLayout {
            Layout.fillWidth: true
            Layout.preferredHeight: root.lineHeight
            Layout.leftMargin: Theme.spacing.sm
            spacing: Theme.spacing.sm

            Icon {
                name: Notifs.dnd ? "notifications_off" : root.empty ? "notifications_none" : "notifications"
                size: 20
                color: Theme.m3onSurfaceVariant
            }

            StyledText {
                Layout.fillWidth: true
                text: root.empty ? "No notifications" : Notifs.count === 1 ? "1 notification" : `${Notifs.count} notifications`
                color: Theme.m3onSurfaceVariant
                elide: Text.ElideRight
            }

            StyledText {
                visible: Notifs.dnd
                text: "Do not disturb"
                color: Theme.m3tertiary
                font.pixelSize: Theme.font.sm
                font.weight: Font.Medium
            }

            IconButton {
                size: root.lineHeight
                icon: "clear_all"
                tooltip: "Clear all"
                visible: !root.empty
                onClicked: Notifs.clear()
            }
        }

        NavList {
            id: list

            Layout.fillWidth: true
            Layout.preferredHeight: Math.min(contentHeight, root.maxListHeight)
            visible: count > 0
            spacing: Theme.spacing.sm
            highlightShown: false
            model: ScriptModel {
                values: Notifs.history
            }
            onActivated: index => Notifs.activate(Notifs.history[index])
            Keys.onDeletePressed: Notifs.history[currentIndex]?.dismiss()

            delegate: NotificationCard {
                required property var modelData

                width: ListView.view.width
                notification: modelData
                selected: ListView.isCurrentItem && ListView.view.activeFocus
            }
        }
    }
}
