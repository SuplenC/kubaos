pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io
import Quickshell.Services.Notifications
import "dnd.js" as Dnd
import "popups.js" as Popups

// Notification daemon. Every notification is kept in `history` until dismissed, unread until it is
// opened, marked read or seen in the notifications panel; `popups` is the subset on screen now.
// Opening the panel is seeing every popup (`popups.js`): they leave as it opens, and nothing pops
// up while it is open, as it lists what comes.
//
// Do not disturb (`dnd.js`) is on by hand, or by itself while Hyprland says the screen is shared
// or a window is fullscreen. While it is on, new notifications land in the history without popping
// up, critical ones apart. Its state is `$XDG_RUNTIME_DIR/kubaos-dnd.json`, which the Command
// menu's on/off pair reads and a restarted shell takes the hand's on back from. A reload keeps the
// notifications (`keepOnReload`), and `PersistentProperties` keeps what is unread, when each came
// and the whole Do not disturb state with them.
Singleton {
    id: root

    readonly property var history: server.trackedNotifications.values.slice().reverse()
    readonly property int count: history.length
    property var popupState: Popups.initial()
    readonly property var popups: root.popupState.popups
    // Whether the notifications panel is open, where everything is seen.
    readonly property bool listing: Panels.current === "notifications"
    // When each notification arrived, by its id: the spec carries no time, so it is the moment
    // the server took it.
    property var arrivals: ({})
    // The ids of the notifications not read yet, as a set.
    property var unreadIds: ({})
    readonly property int unreadCount: history.filter(n => root.unreadIds[n.id] === true).length

    property var dndState: Dnd.initial()
    readonly property bool dnd: Dnd.on(root.dndState)
    // Why it is on: "manual", "sharing" or "fullscreen"; "overridden" while it is off during a
    // share or a fullscreen window; "" while it is simply off.
    readonly property string dndReason: Dnd.reason(root.dndState)
    // Whether the state file has been read, so nothing writes over it before then.
    property bool dndRead: false
    // Whether a reload handed the state over, so the file's doesn't replace it.
    property bool dndKept: false

    // When `notification` arrived, or null for one from before this shell started.
    function arrived(notification): var {
        return root.arrivals[notification?.id] ?? null;
    }

    function isUnread(notification): bool {
        return root.unreadIds[notification?.id] === true;
    }

    function markRead(list: var): void {
        const kept = Object.assign({}, root.unreadIds);
        for (const n of list)
            delete kept[n.id];
        root.unreadIds = kept;
    }

    function markAllRead(): void {
        root.unreadIds = {};
    }

    function toggleDnd(): void {
        root.dndState = Dnd.toggle(root.dndState);
    }

    function setDnd(on: bool): void {
        root.dndState = Dnd.set(root.dndState, on);
    }

    function clear(): void {
        root.popupState = Object.assign({}, root.popupState, {
            popups: []
        });
        for (const n of server.trackedNotifications.values.slice())
            n.dismiss();
    }

    function dismissAll(list: var): void {
        for (const n of list.slice())
            n.dismiss();
    }

    // Lets the transient notifications seen in the panel go, once it has gone: not before, so no
    // row leaves the panel as it slides away.
    function letGoSeen(): void {
        const seen = root.popupState.seen;
        root.popupState = Popups.gone(root.popupState);
        for (const n of seen)
            n.expire();
    }

    function hidePopup(notification): void {
        root.popupState = Popups.left(root.popupState, notification);
        if (notification?.transient)
            notification.expire();
    }

    function hidePopups(list: var): void {
        for (const n of list.slice())
            root.hidePopup(n);
    }

    // Runs `action`, one of `notification`'s, and marks it read. A notification that isn't
    // resident goes as its action is taken.
    // HAZARD: invoke before anything closes it. Closing a notification destroys it, and a destroyed
    // one refuses its action; `hidePopup` closes a transient one.
    function run(notification, action): void {
        root.markRead([notification]);
        root.popupState = Popups.left(root.popupState, notification);
        action.invoke();
    }

    // Runs the default action if there is one, and marks it read.
    function activate(notification): void {
        const action = notification.actions.find(a => a.identifier === "default");
        if (action) {
            root.run(notification, action);
            return;
        }
        root.markRead([notification]);
        root.popupState = Popups.left(root.popupState, notification);
        if (notification.transient)
            notification.expire();
    }

    onListingChanged: root.popupState = Popups.listed(root.popupState, root.listing)
    onDndStateChanged: {
        persisted.dnd = JSON.stringify(root.dndState);
        if (root.dndRead)
            dndFile.setText(Dnd.saved(root.dndState));
    }
    onUnreadIdsChanged: persisted.unread = JSON.stringify(root.unreadIds)
    onArrivalsChanged: {
        const times = {};
        for (const id in root.arrivals)
            times[id] = root.arrivals[id].getTime();
        persisted.arrivals = JSON.stringify(times);
    }

    // What the previous generation had, after a reload: the notifications themselves stay with
    // the server.
    PersistentProperties {
        id: persisted

        property string unread: "{}"
        property string arrivals: "{}"
        property string dnd: ""

        reloadableId: "notifications"
        onReloaded: {
            root.unreadIds = JSON.parse(persisted.unread);
            const arrivals = {};
            const times = JSON.parse(persisted.arrivals);
            for (const id in times)
                arrivals[id] = new Date(times[id]);
            root.arrivals = arrivals;
            if (persisted.dnd !== "") {
                root.dndState = JSON.parse(persisted.dnd);
                root.dndKept = true;
                root.sense();
            }
        }
    }

    // What Hyprland says now: whether the screen is shared, whether a window is fullscreen.
    function sense(): void {
        root.dndState = Dnd.sense(root.dndState, {
            sharing: Hypr.sharing,
            fullscreen: Hypr.fullscreen
        });
    }

    Connections {
        target: Hypr

        function onSharingChanged(): void {
            root.sense();
        }

        function onFullscreenChanged(): void {
            root.sense();
        }
    }

    FileView {
        id: dndFile

        path: `${Quickshell.env("XDG_RUNTIME_DIR")}/kubaos-dnd.json`
        printErrors: false
        onLoaded: {
            // A reload's state is the whole one, the file's only the hand's on.
            if (!root.dndKept)
                root.dndState = Dnd.restore(text());
            root.dndRead = true;
            root.sense();
            dndFile.setText(Dnd.saved(root.dndState));
        }
        onLoadFailed: {
            root.dndRead = true;
            root.sense();
            dndFile.setText(Dnd.saved(root.dndState));
        }
    }

    NotificationServer {
        id: server

        keepOnReload: true
        actionsSupported: true
        imageSupported: true
        bodySupported: true
        persistenceSupported: true

        onNotification: notification => {
            notification.tracked = true;
            const id = notification.id;
            root.arrivals = Object.assign({}, root.arrivals, {
                [id]: new Date()
            });
            root.unreadIds = Object.assign({}, root.unreadIds, {
                [id]: true
            });
            notification.closed.connect(() => {
                root.popupState = Popups.left(root.popupState, notification);
                const kept = Object.assign({}, root.arrivals);
                delete kept[id];
                root.arrivals = kept;
                const unread = Object.assign({}, root.unreadIds);
                delete unread[id];
                root.unreadIds = unread;
            });
            root.popupState = Popups.arrived(root.popupState, notification, Dnd.popsUp(root.dndState, notification.urgency));
        }
    }
}
