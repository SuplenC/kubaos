pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Services.Notifications

// Notification daemon. Every notification is kept in `history` until dismissed;
// `popups` is the subset currently shown on screen. Do-not-disturb suppresses
// popups (except critical ones) but still records history.
Singleton {
    id: root

    readonly property var history: server.trackedNotifications.values.slice().reverse()
    readonly property int count: history.length
    property var popups: []
    readonly property bool dnd: persisted.dnd
    // When each notification arrived, by its id: the spec carries no time, so it is the moment
    // the server took it.
    property var arrivals: ({})

    // When `notification` arrived, or null for one from before this shell started.
    function arrived(notification): var {
        return root.arrivals[notification?.id] ?? null;
    }

    function toggleDnd(): void {
        persisted.dnd = !persisted.dnd;
    }

    function clear(): void {
        root.popups = [];
        for (const n of server.trackedNotifications.values.slice())
            n.dismiss();
    }

    function hidePopup(notification): void {
        root.popups = root.popups.filter(n => n !== notification);
        if (notification?.transient)
            notification.expire();
    }

    // Runs the default action if there is one, then dismisses.
    function activate(notification): void {
        const action = notification.actions.find(a => a.identifier === "default");
        if (action)
            action.invoke();
        notification.dismiss();
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
            root.arrivals = Object.assign({}, root.arrivals, {
                [notification.id]: new Date()
            });
            const id = notification.id;
            notification.closed.connect(() => {
                root.popups = root.popups.filter(n => n !== notification);
                const kept = Object.assign({}, root.arrivals);
                delete kept[id];
                root.arrivals = kept;
            });
            if (!root.dnd || notification.urgency === NotificationUrgency.Critical)
                root.popups = [notification, ...root.popups];
        }
    }

    PersistentProperties {
        id: persisted

        reloadableId: "notifications"

        property bool dnd: false
    }
}
