import QtQuick
import QtQuick.Layouts
import Quickshell
import qs.theme
import qs.components
import qs.services
import qs.modules.notifications

// Notifications received while the shell runs. Enter opens, Delete dismisses.
ColumnLayout {
    spacing: Theme.spacing.sm

    RowLayout {
        Layout.fillWidth: true

        StyledText {
            Layout.fillWidth: true
            text: Notifs.count > 0 ? `${Notifs.count} notifications` : "No notifications"
            color: Theme.m3onSurfaceVariant
        }

        IconButton {
            icon: "clear_all"
            tooltip: "Clear all"
            enabled: Notifs.count > 0
            opacity: enabled ? 1 : 0.4
            onClicked: Notifs.clear()
        }
    }

    NavList {
        id: list

        Layout.fillWidth: true
        Layout.fillHeight: true
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

    Item {
        Layout.fillWidth: true
        Layout.fillHeight: true
        visible: list.count === 0

        EmptyState {
            anchors.centerIn: parent
            icon: Notifs.dnd ? "notifications_off" : "notifications_none"
            text: Notifs.dnd ? "Do not disturb is on" : "Nothing here. Enjoy the quiet."
        }
    }
}
