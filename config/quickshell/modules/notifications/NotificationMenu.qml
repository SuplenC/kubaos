import QtQuick
import KubaOS.Components as K
import qs.services

// A notification's right-click menu: its actions, then Open, Dismiss and Clear all, as a `Menu` on
// a `Popover` floating over the window under the pointer. A press anywhere else or Esc closes it.
// Over the corner stack, which is no panel, it opens a modal one (`notificationmenu`), so the
// press elsewhere reaches it and the keys go to the menu; inside the control center, already
// modal, it stays its own.
Item {
    id: root

    // Whether it opens a modal panel of its own while it shows.
    property bool modal: false
    property var notification: null
    // What had the keyboard before the menu took it, given back when it closes.
    property Item returnTo: null
    readonly property var actions: root.notification?.actions.filter(a => a.identifier !== "default") ?? []
    readonly property var entries: [...root.actions.map(a => ({
                    text: a.text
                })), ...(root.actions.length > 0 ? [
                {
                    separator: true
                }
            ] : []),
        {
            text: "Open"
        },
        {
            text: "Dismiss"
        },
        {
            text: "Clear all",
            danger: true
        }
    ]

    // Opens `notification`'s menu with its corner at `at`, in the window's content coordinates.
    function open(notification: var, at: point): void {
        const window = root.Window.window;
        if (!window)
            return;
        root.returnTo = window.activeFocusItem;
        root.parent = window.contentItem;
        root.notification = notification;
        menu.currentIndex = -1;
        popover.x = Math.max(K.Tokens.space.s8, Math.min(at.x, window.width - K.Tokens.size.menu - K.Tokens.space.s8));
        popover.y = Math.max(K.Tokens.space.s8, Math.min(at.y, window.height - popover.height - K.Tokens.space.s8));
        if (root.modal)
            Panels.open("notificationmenu");
        menu.forceActiveFocus();
    }

    function close(): void {
        root.notification = null;
        if (root.modal && Panels.current === "notificationmenu")
            Panels.close();
        const back = root.returnTo;
        root.returnTo = null;
        if (back)
            back.forceActiveFocus();
    }

    // HAZARD: read what was picked before closing. Closing clears the notification, which takes
    // its actions out of `entries` and moves every item after them.
    function take(index: int): void {
        const n = root.notification;
        const actions = root.actions;
        const name = root.entries[index]?.text ?? "";
        root.close();
        if (!n)
            return;
        if (index < actions.length) {
            actions[index].invoke();
            return;
        }
        if (name === "Open")
            Notifs.activate(n);
        else if (name === "Dismiss")
            n.dismiss();
        else if (name === "Clear all")
            Notifs.clear();
    }

    anchors.fill: parent
    visible: root.notification !== null
    z: 1000

    Connections {
        target: Panels

        function onCurrentChanged(): void {
            // Another panel took over: the keyboard is its, so nothing is given back.
            if (root.modal && Panels.current !== "notificationmenu") {
                root.notification = null;
                root.returnTo = null;
            }
        }
    }

    // A notification gone while its menu shows takes the menu with it.
    Connections {
        target: root.notification

        function onClosed(): void {
            root.close();
        }
    }

    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        onPressed: root.close()
    }

    K.Popover {
        id: popover

        floating: true
        fixedWidth: K.Tokens.size.menu

        K.Menu {
            id: menu

            entries: root.entries
            onActivated: index => root.take(index)
            onDismissed: root.close()
        }
    }
}
