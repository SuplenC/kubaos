pragma Singleton
import QtQuick
import Quickshell.Services.Notifications

// Stand-in for Notifs: six notifications in the history, newest first: three from Slack, which
// stack, and the build, unread; lunch and the battery, read. The popups are empty unless a driver
// shows them (`showPopups`): the build, with an image and actions, and the battery, a plain one.
// A driver sets `dndReason` for Do not disturb.
QtObject {
    id: root

    property Notification ania: Notification {
        summary: "Ania"
        body: "Pushed the fix for the bar, can you check it before the stand-up?"
        appName: "Slack"
        expireTimeout: 600000
        actions: [
            NotificationAction {
                identifier: "reply"
                text: "Reply"
            },
            NotificationAction {
                identifier: "read"
                text: "Mark read"
            }
        ]
    }
    property Notification marco: Notification {
        summary: "Marco"
        body: "Lunch at 13:00?"
        appName: "Slack"
        expireTimeout: 600000
    }
    property Notification deploy: Notification {
        summary: "#deploys"
        body: "kubaos-ui 0.4 is out on the staging mirror."
        appName: "Slack"
        expireTimeout: 600000
    }
    property Notification build: Notification {
        summary: "Build finished"
        body: "kubaos-ui and kubaos-specimen built in 41 s; 312 tests passed, none skipped. The specimen's shots changed for the components page."
        appName: "Ghostty"
        image: "file://@WALLPAPERS@/shinjuku/kabukicho-red-gate.jpg"
        expireTimeout: 600000
        actions: [
            NotificationAction {
                identifier: "open"
                text: "Open log"
            },
            NotificationAction {
                identifier: "default"
                text: "Default"
            }
        ]
    }
    property Notification lunch: Notification {
        summary: "Kuba"
        body: "Lunch at Fuunji? The queue is short today."
        appName: "Signal"
        expireTimeout: 600000
    }
    property Notification battery: Notification {
        summary: "Battery at 20%"
        body: "Plug in soon, or switch to Power Saver."
        appName: "KubaOS"
        expireTimeout: 600000
    }
    property var history: [root.ania, root.marco, root.build, root.deploy, root.lunch, root.battery]
    readonly property int count: history.length
    property var popups: []
    property var unread: [root.ania, root.marco, root.build, root.deploy]
    readonly property int unreadCount: root.unread.filter(n => root.history.includes(n)).length
    readonly property var unreadIds: ({})
    property string dndReason: ""
    readonly property bool dnd: ["manual", "sharing", "fullscreen"].includes(root.dndReason)

    // When each arrived: a few minutes before the shots' frozen clock, and one the day before.
    function arrived(notification: var): var {
        const at = {
            "Ania": new Date(2026, 9, 27, 21, 46),
            "Marco": new Date(2026, 9, 27, 21, 44),
            "Build finished": new Date(2026, 9, 27, 21, 41),
            "#deploys": new Date(2026, 9, 27, 20, 58),
            "Kuba": new Date(2026, 9, 27, 12, 8)
        }[notification?.summary];
        return at ?? new Date(2026, 9, 26, 19, 30);
    }

    function isUnread(notification: var): bool {
        return root.unread.includes(notification);
    }

    function markRead(list: var): void {
    }

    function markAllRead(): void {
    }

    function showPopups(): void {
        root.popups = [root.build, root.battery];
    }

    function toggleDnd(): void {
        root.dndReason = root.dnd ? "" : "manual";
    }

    function setDnd(on: bool): void {
        root.dndReason = on ? "manual" : "";
    }

    function clear(): void {
        root.history = [];
    }

    function dismissAll(list: var): void {
    }

    function hidePopup(notification: var): void {
    }

    function activate(notification: var): void {
    }
}
