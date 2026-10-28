import QtQuick
import Quickshell
import KubaOS.Components as K
import qs.components
import qs.services
import qs.modules.notifications

// The control center's notifications, above the tabs: one slim line saying
// how many there are and whether do not disturb is on, and under it the
// notifications themselves, up to `maxListHeight` before the list scrolls.
// Enter opens one, Delete dismisses it.
K.Stack {
    id: root

    // The most the list takes, about four notifications, so the tabs below keep room on a small
    // screen.
    property real maxListHeight: K.Tokens.size.row_two_line * 4
    readonly property bool empty: Notifs.count === 0

    fill: true
    gap: K.Tokens.space.s8

    // As tall as the clear-all button whether or not it shows, so the line keeps its height when
    // the button goes with the last notification.
    K.Row {
        fill: true
        fixedHeight: K.Tokens.size.control_sm
        gap: K.Tokens.space.s8
        align: "centre"

        K.Icon {
            name: Notifs.dnd ? "notifications_off" : "notifications"
            size: K.Tokens.icon.s20
            color: K.Tokens.color.on_surface_variant
        }

        K.Stack {
            fill: true

            K.Text {
                text: root.empty ? "No notifications" : Notifs.count === 1 ? "1 notification" : `${Notifs.count} notifications`
                style: K.Tokens.type.body
                color: K.Tokens.color.on_surface_variant
                maximumLines: 1
            }
        }

        K.Chip {
            visible: Notifs.dnd
            text: "Do not disturb"
            kind: "Accent"
        }

        K.IconButton {
            visible: !root.empty
            icon: "clear_all"
            label: "Clear all"
            kind: "Text"
            size: "Sm"
            onActivated: Notifs.clear()
        }
    }

    // A list of like items, so in a well, as Wi-Fi's networks and the Bluetooth devices are.
    Well {
        visible: list.count > 0
        implicitHeight: Math.min(list.contentHeight, root.maxListHeight) + K.Tokens.space.s12 * 2
        focused: list.activeFocus

        NavList {
            id: list

            anchors.fill: parent
            // The well's padding above the first row and below the last, scrolled with them.
            topMargin: K.Tokens.space.s12
            bottomMargin: K.Tokens.space.s12
            spacing: K.Tokens.space.s4
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
                inWell: true
                selected: ListView.isCurrentItem && ListView.view.activeFocus
                onMenuRequested: at => menu.open(modelData, at)
            }
        }
    }

    NotificationMenu {
        id: menu
    }
}
