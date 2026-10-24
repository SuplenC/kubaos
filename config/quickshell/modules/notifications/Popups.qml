import QtQuick
import Quickshell
import qs.theme
import qs.components
import qs.services

// Notification popups, in a panel that grows down from the top-right corner
// of the frame. New cards slide in from the right; the panel's height follows
// the stack.
//
// While an open panel covers the corner (`covered`), the cards wait in
// `Notifs.popups`, still in the control center's history, and come back
// when the panel has gone; critical ones show under it (`PopupsBelow.qml`).
Drawer {
    id: root

    readonly property int padding: Theme.spacing.sm
    property bool covered: false
    readonly property int drawn: list.drawn

    edge: "top"
    flushEnd: true
    takesFocus: false
    contentWidth: 380
    contentHeight: Math.max(list.contentHeight, 1) + padding * 2

    ToastList {
        id: list

        x: root.padding
        y: root.padding
        width: root.contentWidth - root.padding * 2
        drawer: root
        notifications: root.shown || root.active ? Notifs.popups.slice(0, 5) : []
        // HAZARD: the cards go at once when a panel covers them, not with the
        // surface's fade. The sysmon and clock popups show their text within
        // a few frames of opening, long before this panel's content has faded.
        opacity: root.covered ? 0 : 1
    }
}
