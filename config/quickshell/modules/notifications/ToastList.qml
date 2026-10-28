import QtQuick
import Quickshell
import KubaOS.Components as K

// The cards of a toast stack (`Popups.qml`, `PopupsBelow.qml`). New cards
// slide in from the right as a panel arrives, gone ones slide out as one
// leaves, and the rest close up as a list reorders.
ListView {
    id: list

    required property K.Sheet drawer
    // The notifications to show; empty while the drawer is away, so other
    // screens and hidden stacks don't run their countdown animations.
    required property var notifications
    // The cards on screen now: none while the drawer or the list is faded out.
    readonly property int drawn: drawer.active && drawer.contentOpacity > 0 && opacity > 0 ? count : 0
    // The card the pointer is on. While it is, it shows alone: the others fade out where they are,
    // their time stopped, and a new one waits, so nothing moves under the pointer.
    property var pointed: null
    // The cards when the pointer came onto one.
    property var held: []
    readonly property bool holding: list.pointed !== null && list.notifications.includes(list.pointed)

    height: contentHeight
    interactive: false
    spacing: K.Tokens.space.s8
    model: ScriptModel {
        values: list.holding ? list.held.filter(n => list.notifications.includes(n)) : list.notifications
    }

    add: Transition {
        ParallelAnimation {
            NumberAnimation {
                property: "x"
                from: K.Tokens.space.s64
                to: 0
                duration: K.Tokens.duration.long
                easing.type: Easing.BezierSpline
                easing.bezierCurve: K.Tokens.easing.emphasized_decel
            }

            NumberAnimation {
                property: "opacity"
                from: 0
                to: 1
                duration: K.Tokens.duration.short
                easing.type: Easing.BezierSpline
                easing.bezierCurve: K.Tokens.easing.standard
            }
        }
    }

    remove: Transition {
        ParallelAnimation {
            NumberAnimation {
                property: "x"
                to: K.Tokens.space.s64 * 2
                duration: K.Tokens.duration.medium
                easing.type: Easing.BezierSpline
                easing.bezierCurve: K.Tokens.easing.emphasized_accel
            }

            NumberAnimation {
                property: "opacity"
                to: 0
                duration: K.Tokens.duration.medium
                easing.type: Easing.BezierSpline
                easing.bezierCurve: K.Tokens.easing.standard_accel
            }
        }
    }

    // A card displaced mid-slide-in finishes the slide and the fade, or it stays faded and
    // off to the side (see `components/NavList.qml`).
    displaced: Transition {
        NumberAnimation {
            property: "y"
            duration: K.Tokens.duration.medium
            easing.type: Easing.BezierSpline
            easing.bezierCurve: K.Tokens.easing.standard_decel
        }

        NumberAnimation {
            property: "x"
            to: 0
            duration: K.Tokens.duration.long
            easing.type: Easing.BezierSpline
            easing.bezierCurve: K.Tokens.easing.emphasized_decel
        }

        NumberAnimation {
            property: "opacity"
            to: 1
            duration: K.Tokens.duration.short
            easing.type: Easing.BezierSpline
            easing.bezierCurve: K.Tokens.easing.standard
        }
    }

    // The card in a box of its own: the transitions above move and fade the box, and the card
    // fades by itself while another is pointed at.
    delegate: Item {
        id: slot

        required property var modelData

        width: ListView.view.width
        height: card.implicitHeight

        NotificationCard {
            id: card

            readonly property bool aside: list.holding && list.pointed !== slot.modelData

            width: slot.width
            notification: slot.modelData
            popup: true
            held: menu.notification === slot.modelData || list.holding
            enabled: !card.aside
            opacity: card.aside ? 0 : 1
            onMenuRequested: at => menu.open(slot.modelData, at)
            onPointedChanged: {
                if (card.pointed) {
                    list.held = list.notifications.slice();
                    list.pointed = slot.modelData;
                } else if (list.pointed === slot.modelData) {
                    list.pointed = null;
                }
            }

            Behavior on opacity {
                NumberAnimation {
                    duration: K.Tokens.duration.micro
                    easing.type: Easing.BezierSpline
                    easing.bezierCurve: K.Tokens.easing.standard
                }
            }
        }
    }

    NotificationMenu {
        id: menu

        modal: true
    }
}
