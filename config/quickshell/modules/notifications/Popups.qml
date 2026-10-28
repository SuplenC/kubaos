import QtQuick
import Quickshell
import KubaOS.Components as K
import qs.services

// Notification popups, in a panel that grows down from the top-right corner
// of the frame. New cards slide in from the right; the panel's height follows
// the stack. With no popup to show, the bell's peek (`Peek.qml`) takes the
// same place while the bar asks for it (`peeking`).
//
// While an open panel covers the corner (`covered`), the cards wait in
// `Notifs.popups`, still in the history, and come back when the panel has
// gone; critical ones show under it (`PopupsBelow.qml`).
K.Sheet {
    id: root

    readonly property int padding: K.Tokens.space.s8
    property bool covered: false
    property bool peeking: false
    readonly property bool hasPopups: Notifs.popups.length > 0
    readonly property bool hasPeek: !root.hasPopups && root.peeking && peek.peek.stacks.length > 0
    // Whether the pointer is on the peek, which keeps it open. An unseen peek takes no hover.
    readonly property bool peekHovered: peek.hovered
    readonly property int drawn: list.drawn

    edge: "top"
    flushEnd: true
    takesFocus: false
    contentWidth: K.Tokens.size.panel
    contentHeight: Math.max(root.hasPopups || !root.hasPeek ? list.contentHeight : peek.implicitHeight, 0) + padding * 2

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

    Peek {
        id: peek

        x: root.padding
        y: root.padding
        width: root.contentWidth - root.padding * 2
        visible: root.hasPeek && !root.covered
    }
}
