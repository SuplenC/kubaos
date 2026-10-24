import QtQuick
import Quickshell
import qs.theme
import qs.components

// The cards of a toast stack (`Popups.qml`, `PopupsBelow.qml`). New cards
// slide in from the right, gone ones slide out, and the rest close up.
ListView {
    id: list

    required property Drawer drawer
    // The notifications to show; empty while the drawer is away, so other
    // screens and hidden stacks don't run their countdown animations.
    required property var notifications
    // The cards on screen now: none while the drawer or the list is faded out.
    readonly property int drawn: drawer.active && drawer.contentOpacity > 0 && opacity > 0 ? count : 0

    height: contentHeight
    interactive: false
    spacing: Theme.spacing.sm
    model: ScriptModel {
        values: list.notifications
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

    // A card displaced mid-slide-in finishes the slide and the fade, or it stays faded and
    // off to the side (see `components/NavList.qml`).
    displaced: Transition {
        Anim {
            property: "y"
            duration: Theme.anim.large
            easing.bezierCurve: Theme.anim.spatial
        }

        Anim {
            property: "x"
            to: 0
            duration: Theme.anim.large
            easing.bezierCurve: Theme.anim.emphasizedDecel
        }

        Anim {
            property: "opacity"
            to: 1
            duration: Theme.anim.small
            easing.bezierCurve: Theme.anim.effects
        }
    }

    delegate: NotificationCard {
        required property var modelData

        width: ListView.view.width
        notification: modelData
        popup: true
    }
}
