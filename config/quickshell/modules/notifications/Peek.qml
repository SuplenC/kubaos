import QtQuick
import KubaOS.Components as K
import qs.services
import "feed.js" as Feed

// The bell's peek, in the popups' place: the most recent unread notifications as the popups'
// cards, one app's in a group (`CardGroup.qml`), then "+N more" (which opens the panel) and two
// one-click actions on what it shows, Mark read and Clear. Looking marks nothing read; a click on
// a group opens it, a click on a card unfolds its text, its buttons run its actions, and its ×
// dismisses it. A group's preview floats past the peek's end (`overhang`), which the popups make
// room for.
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
    readonly property bool hovered: hover.hovered || (root.lifted?.pointed ?? false)
    // The notification whose card shows its whole text, one at a time, until the peek goes.
    property var unfolded: null
    // The groups opened, by app, until the peek goes.
    property var opened: ({})
    // The group whose preview shows, which may reach past the peek's end.
    property CardGroup lifted: null
    readonly property real overhang: root.lifted ? Math.max(0, root.lifted.y + root.lifted.extent - root.implicitHeight) : 0
    // How far what isn't a group's preview sits back behind it.
    readonly property real sitBack: 1 - K.Tokens.opacity.recede * (root.lifted?.recede ?? 0)
    // How many groups are moving between two states, which the popups' size follows frame by frame.
    property int settling: 0

    function setOpened(app: string, open: bool): void {
        const next = Object.assign({}, root.opened);
        if (open)
            next[app] = true;
        else
            delete next[app];
        root.opened = next;
    }

    implicitHeight: column.implicitHeight
    onNowChanged: {
        if (root.now.stacks.length > 0)
            root.peek = root.now;
    }
    onVisibleChanged: {
        if (root.visible)
            return;
        root.unfolded = null;
        root.opened = {};
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

            delegate: CardGroup {
                id: group

                required property var modelData

                implicitWidth: root.width
                z: group.raised ? 1 : 0
                items: group.modelData.items
                open: root.opened[group.modelData.app] === true
                // Every other group sits back behind one's preview.
                opacity: root.lifted && root.lifted !== group ? root.sitBack : 1
                onOpenRequested: open => root.setOpened(group.modelData.app, open)
                onRaisedChanged: {
                    if (group.raised)
                        root.lifted = group;
                    else if (root.lifted === group)
                        root.lifted = null;
                }
                onSettlingChanged: root.settling += group.settling ? 1 : -1
                Component.onDestruction: {
                    if (group.settling)
                        root.settling -= 1;
                    if (root.lifted === group)
                        root.lifted = null;
                }

                delegate: NotificationCard {
                    required property int index

                    notification: group.items[index] ?? null
                    items: group.standing[index] ?? []
                    unfolded: root.unfolded !== null && root.unfolded === notification
                    onUnfoldRequested: open => root.unfolded = open ? notification : null
                }
            }
        }

        K.Row {
            width: root.width
            opacity: root.sitBack
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
