import QtQuick
import KubaOS.Components as K
import qs.services
import "feed.js" as Feed

// The bell's peek, in the popups' place: the most recent unread notifications as the popups'
// cards, one app's stacked into one, then "+N more" (which opens the panel) and two one-click
// actions on what it shows, Mark read and Clear. Looking marks nothing read; a click on a card
// unfolds its text, its buttons run its actions, and its × dismisses it.
Item {
    id: root

    // What there is to show now.
    readonly property var now: Feed.peek(Notifs.history, n => Notifs.isUnread(n))
    // What it shows: the last cards it had once there are none, so it closes with them on it
    // rather than empty.
    property var peek: ({
            stacks: [],
            more: 0
        })
    readonly property var shown: [].concat(...root.peek.stacks.map(stack => stack.items))
    readonly property bool hovered: hover.hovered
    // The notification whose card shows its whole text, one at a time, until the peek goes.
    property var unfolded: null

    implicitHeight: column.implicitHeight
    onNowChanged: {
        if (root.now.stacks.length > 0)
            root.peek = root.now;
    }
    onVisibleChanged: {
        if (!root.visible)
            root.unfolded = null;
    }
    Component.onCompleted: root.peek = root.now

    HoverHandler {
        id: hover
    }

    K.Stack {
        id: column

        width: root.width
        gap: K.Tokens.space.s8

        Repeater {
            // No cards while the peek is away, so a new notification makes none unseen.
            model: root.visible ? root.peek.stacks : []

            delegate: NotificationCard {
                required property var modelData

                implicitWidth: root.width
                notification: modelData.items[0]
                items: modelData.items
                unfolded: root.unfolded === modelData.items[0]
                onUnfoldRequested: open => root.unfolded = open ? modelData.items[0] : null
            }
        }

        K.Row {
            width: root.width
            fill: true
            gap: K.Tokens.space.s4
            align: "centre"

            K.Button {
                visible: root.peek.more > 0
                text: `+${root.peek.more} more`
                kind: "Text"
                size: "Sm"
                activeFocusOnTab: false
                onActivated: Panels.open("notifications")
            }

            K.Fill {}

            K.Button {
                text: "Mark read"
                kind: "Text"
                size: "Sm"
                activeFocusOnTab: false
                onActivated: Notifs.markRead(root.shown)
            }

            K.Button {
                text: "Clear"
                kind: "Text"
                size: "Sm"
                activeFocusOnTab: false
                onActivated: Notifs.dismissAll(root.shown)
            }
        }
    }
}
