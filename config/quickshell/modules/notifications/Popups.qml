import QtQuick
import Quickshell
import KubaOS.Components as K
import qs.services

// Notification popups, in a panel that grows down from the top-right corner
// of the frame, one app's in a group. New cards slide in from the right; the
// panel's height follows the stack, and a group's preview past its end. With
// no popup to show on this screen, the bell's peek (`Peek.qml`) takes the same
// place while the bar asks for it (`peeking`).
//
// While an open panel covers the corner (`covered`), the cards wait in
// `Notifs.popups`, still in the history, and come back when the panel has
// gone; critical ones show under it (`PopupsBelow.qml`). The notifications
// panel is the exception: opening it is seeing them, so they leave for good.
K.Sheet {
    id: root

    readonly property int padding: K.Tokens.space.s8
    property bool covered: false
    property bool peeking: false
    // Whether this screen shows the popups: the one with the keyboard.
    property bool here: false
    readonly property bool hasPopups: root.here && Notifs.popups.length > 0
    readonly property bool hasPeek: !root.hasPopups && root.peeking && peek.now.stacks.length > 0
    // The peek is what the panel shows: from the moment it is asked for until the panel has gone,
    // so it closes with its cards on it.
    property bool showingPeek: false
    // Whether the pointer is on the peek, which keeps it open. An unseen peek takes no hover.
    readonly property bool peekHovered: peek.hovered
    readonly property int drawn: list.drawn

    edge: "top"
    flushEnd: true
    takesFocus: false
    contentWidth: K.Tokens.size.panel
    contentHeight: Math.max(root.showingPeek ? peek.implicitHeight + peek.overhang : list.contentHeight + list.overhang, 0) + padding * 2
    // A group moving between two states sizes the panel frame by frame, rather than one easing after
    // the other.
    animateSize: list.settling === 0 && peek.settling === 0
    onHasPeekChanged: {
        if (root.hasPeek)
            root.showingPeek = true;
    }
    onHasPopupsChanged: {
        if (root.hasPopups)
            root.showingPeek = false;
    }
    onActiveChanged: {
        if (!root.active)
            root.showingPeek = root.hasPeek;
    }

    ToastList {
        id: list

        x: root.padding
        y: root.padding
        width: root.contentWidth - root.padding * 2
        drawer: root
        notifications: (root.shown || root.active) && !root.showingPeek ? Notifs.popups : []
        // HAZARD: the cards go at once when a panel covers them, not with the
        // surface's fade. The system monitor and the clock popup show their text
        // within a few frames of opening, long before this panel's content has
        // faded.
        opacity: root.covered ? 0 : 1
    }

    Peek {
        id: peek

        x: root.padding
        y: root.padding
        width: root.contentWidth - root.padding * 2
        visible: root.showingPeek && !root.covered
    }
}
