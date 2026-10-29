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
    // While the pointer is on the stack, every card stays with its time stopped.
    readonly property bool holding: hold.holding
    // What the stack keeps while the pointer is on it; the shots hold it without a pointer.
    readonly property PopupHold hold: hold
    // The notification whose card shows its whole text, one at a time.
    property var unfolded: null

    height: contentHeight
    interactive: false
    spacing: K.Tokens.space.s8
    model: ScriptModel {
        values: hold.shown
    }

    PopupHold {
        id: hold

        notifications: list.notifications
        pointed: stackHover.hovered
    }

    // On the list, not on each card, so the pointer crossing the gap between two cards doesn't
    // let them go for a moment.
    HoverHandler {
        id: stackHover
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

    // The card in a box of its own, which the transitions above move and fade.
    delegate: Item {
        id: slot

        required property var modelData

        width: ListView.view.width
        height: card.implicitHeight

        NotificationCard {
            id: card

            width: slot.width
            notification: slot.modelData
            popup: true
            unfolded: list.unfolded === slot.modelData
            held: menu.notification === slot.modelData || list.holding
            onMenuRequested: at => menu.open(slot.modelData, at)
            onUnfoldRequested: open => list.unfolded = open ? slot.modelData : null
        }
    }

    NotificationMenu {
        id: menu

        modal: true
    }
}
