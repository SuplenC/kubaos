import QtQuick
import Quickshell
import KubaOS.Components as K
import "feed.js" as Feed

// The cards of a toast stack (`Popups.qml`, `PopupsBelow.qml`), one app's in a group
// (`CardGroup.qml`). New groups slide in from the right as a panel arrives, gone ones slide out as
// one leaves, and the rest close up as a list reorders. A group's preview floats past the list's
// end (`overhang`), which the stack makes room for.
ListView {
    id: list

    required property K.Sheet drawer
    // The notifications to show, newest first; empty while the drawer is away, so other screens
    // and hidden stacks don't run their countdown animations.
    required property var notifications
    // The groups, at most five, each keyed by its app so it keeps its card as its notifications
    // come and go.
    readonly property var groups: Feed.groups(hold.shown).slice(0, 5)
    // The notifications on screen now: none while the drawer or the list is faded out.
    readonly property int drawn: drawer.active && drawer.contentOpacity > 0 && opacity > 0 ? list.groups.reduce((sum, g) => sum + g.items.length, 0) : 0
    // While the pointer is on the stack, every card stays with its time stopped.
    readonly property bool holding: hold.holding
    // What the stack keeps while the pointer is on it; the shots hold it without a pointer.
    readonly property PopupHold hold: hold
    // The notification whose card shows its whole text, one at a time.
    property var unfolded: null
    // The groups opened, by app.
    property var opened: ({})
    // The group whose preview shows, which may reach past the list's end.
    property CardGroup lifted: null
    readonly property real overhang: list.lifted ? Math.max(0, list.lifted.parent.y + list.lifted.extent - list.contentHeight) : 0
    // How many groups are moving between two states, which the stack's size follows frame by frame.
    property int settling: 0

    function setOpened(key: string, open: bool): void {
        const next = Object.assign({}, list.opened);
        if (open)
            next[key] = true;
        else
            delete next[key];
        list.opened = next;
    }

    height: contentHeight
    interactive: false
    spacing: K.Tokens.space.s8
    model: ScriptModel {
        values: list.groups
        objectProp: "key"
    }

    PopupHold {
        id: hold

        notifications: list.notifications
        pointed: stackHover.hovered || (list.lifted?.pointed ?? false)
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

    // The group in a box of its own, which the transitions above move and fade.
    delegate: Item {
        id: slot

        required property var modelData

        width: ListView.view.width
        height: group.implicitHeight
        z: group.raised ? 1 : 0

        CardGroup {
            id: group

            width: slot.width
            // Every other group sits back behind one's preview, and goes where the preview covers
            // it: on the group, as the slot's own opacity is the transitions'. A lifted group's
            // parent is its slot.
            opacity: list.lifted && list.lifted !== group ? Feed.behind(slot.y, slot.y + slot.height, list.lifted.parent.y, list.lifted.parent.y + list.lifted.extent, list.lifted.recede, K.Tokens.opacity.recede) : 1
            items: slot.modelData.items
            open: list.opened[slot.modelData.key] === true
            onOpenRequested: open => list.setOpened(slot.modelData.key, open)
            onRaisedChanged: {
                if (group.raised)
                    list.lifted = group;
                else if (list.lifted === group)
                    list.lifted = null;
            }
            onSettlingChanged: list.settling += group.settling ? 1 : -1
            Component.onDestruction: {
                if (group.settling)
                    list.settling -= 1;
                if (list.lifted === group)
                    list.lifted = null;
            }

            delegate: NotificationCard {
                required property int index

                notification: group.items[index] ?? null
                items: group.standing[index] ?? []
                popup: true
                unfolded: list.unfolded !== null && list.unfolded === notification
                held: menu.notification === notification || list.holding
                onMenuRequested: at => menu.open(notification, at)
                onUnfoldRequested: open => list.unfolded = open ? notification : null
            }
        }
    }

    NotificationMenu {
        id: menu

        modal: true
    }
}
