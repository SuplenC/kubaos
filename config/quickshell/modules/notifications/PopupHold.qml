import QtQml

// What a stack of popups shows while the pointer is on it: every card it showed when the pointer
// came, each with its time stopped where it was, and none leaving by itself. A new one waits, so
// nothing moves under the pointer. When the pointer leaves, each goes on with the time it had left
// and the waiting ones come in. A card dismissed or acted on while held still goes.
QtObject {
    id: root

    // The popups to show, newest first.
    property var notifications: []
    // Whether the pointer is anywhere on the stack, the gaps between its cards included.
    property bool pointed: false
    // The cards when the pointer came.
    property var kept: []
    readonly property var still: root.kept.filter(n => root.notifications.includes(n))
    readonly property bool holding: root.pointed && root.still.length > 0
    readonly property var shown: root.holding ? root.still : root.notifications

    onPointedChanged: root.kept = root.pointed ? root.notifications.slice() : []
}
