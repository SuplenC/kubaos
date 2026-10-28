pragma Singleton
import QtQuick
import Quickshell.Services.Notifications

// Stand-in for Notifs: three notifications in the history. The popups are empty unless a driver
// shows them (`showPopups`): the first with an image and actions, the second a plain one.
QtObject {
    id: root

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
    property var history: [root.build, root.lunch, root.battery]
    readonly property int count: history.length
    property var popups: []
    property bool dnd: false

    function showPopups(): void {
        root.popups = [root.build, root.battery];
    }

    function toggleDnd(): void {
        root.dnd = !root.dnd;
    }

    function clear(): void {
        root.history = [];
    }

    function hidePopup(notification: var): void {
    }

    function activate(notification: var): void {
    }
}
