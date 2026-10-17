import QtQuick
import Quickshell
import qs.theme
import qs.components
import qs.services

// Notification popups, in a panel that grows down from the top-right corner
// of the frame. New cards slide in from the right; the panel's height follows
// the stack.
Drawer {
    id: root

    readonly property int padding: Theme.spacing.sm

    edge: "top"
    flushEnd: true
    takesFocus: false
    contentWidth: 380
    contentHeight: Math.max(list.contentHeight, 1) + padding * 2

    ListView {
        id: list

        x: root.padding
        y: root.padding
        width: root.contentWidth - root.padding * 2
        height: contentHeight
        interactive: false
        spacing: Theme.spacing.sm
        // Only the screen showing the panel builds cards, so other screens
        // don't run their countdown animations.
        model: ScriptModel {
            values: root.shown || root.active ? Notifs.popups.slice(0, 5) : []
        }

        add: Transition {
            ParallelAnimation {
                Anim {
                    property: "x"
                    from: 80
                    to: 0
                    duration: Theme.anim.large
                    easing.bezierCurve: Theme.anim.emphasizedDecel
                }

                Anim {
                    property: "opacity"
                    from: 0
                    to: 1
                    duration: Theme.anim.small
                    easing.bezierCurve: Theme.anim.effects
                }
            }
        }

        remove: Transition {
            ParallelAnimation {
                Anim {
                    property: "x"
                    to: 120
                    duration: Theme.anim.normal
                    easing.bezierCurve: Theme.anim.emphasizedAccel
                }

                Anim {
                    property: "opacity"
                    to: 0
                    duration: Theme.anim.small
                }
            }
        }

        displaced: Transition {
            Anim {
                property: "y"
                duration: Theme.anim.large
                easing.bezierCurve: Theme.anim.spatial
            }
        }

        delegate: NotificationCard {
            required property var modelData

            width: ListView.view.width
            notification: modelData
            popup: true
        }
    }
}
