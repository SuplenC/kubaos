import QtQuick
import Quickshell
import Quickshell.Services.Notifications
import qs.settings
import qs.services
import "feed.js" as Feed

// One notification, or the newest of a group standing for the whole of it, the same card in the
// popups, the bell's peek and the notifications panel: `CardFace` fed from the notification, its
// buttons wired to it. As a popup it hides itself, and what it stands for, after the timeout
// (stopped while the pointer is on it; a thin line along the bottom shows the time left), from
// the whole time again when a newer notification takes it.
//
// The card is one step above what holds it, its corners the panel's surface radius 8 in.
CardFace {
    id: root

    // The newest of `items`, which the card shows.
    required property Notification notification
    // The notifications the card stands for, newest first: one, or a group.
    property var items: [root.notification]
    // Whether something keeps it, as its open menu does: a popup waits while it is held.
    property bool held: false
    // expireTimeout is the raw D-Bus value in milliseconds (Quickshell's docs say seconds).
    readonly property int timeout: notification?.expireTimeout > 0 ? notification.expireTimeout : Settings.notificationTimeout
    readonly property var buttons: Feed.actions(root.notification?.actions ?? [])
    readonly property var arrived: root.notification ? Notifs.arrived(root.notification) : null

    // `body` as plain words. A notification's body may carry the spec's small set of markup
    // (bold, italics, links, line breaks, entities), which a type style draws as written. A number
    // keeps its unit on its line ("41 s", "20 %"), so a line never ends in a lone number.
    // https://specifications.freedesktop.org/notification-spec/latest/markup.html
    function plain(body: string): string {
        return body.replace(/<br\s*\/?>/gi, "\n").replace(/<[^>]*>/g, "").replace(/&lt;/g, "<").replace(/&gt;/g, ">").replace(/&quot;/g, "\"").replace(/&apos;|&#39;/g, "'").replace(/&amp;/g, "&").replace(/(\d) (?=[A-Za-z%°]{1,3}\b)/g, "$1 ");
    }

    // Runs `action` of the newest and marks every notification the card stands for read.
    function run(action): void {
        Notifs.markRead(root.items);
        Notifs.run(root.notification, action);
    }

    onNotificationChanged: countdown.restart()

    function dismiss(): void {
        Notifs.dismissAll(root.items);
    }

    appName: root.notification?.appName ?? ""
    icon: root.critical ? "warning" : "notifications"
    appIcon: {
        const icon = notification?.appIcon ?? "";
        if (icon === "")
            return "";
        return icon.startsWith("/") ? `file://${icon}` : Quickshell.iconPath(icon, true);
    }
    photo: root.notification?.image ?? ""
    // The day and time it came: the time alone today, the day's name before it otherwise.
    when: {
        const at = root.arrived;
        if (!at)
            return "";
        const now = Time.now;
        const day = d => new Date(d.getFullYear(), d.getMonth(), d.getDate()).getTime();
        const days = Math.round((day(now) - day(at)) / 86400000);
        const time = Qt.formatDateTime(at, Time.timeFormat);
        if (days === 0)
            return time;
        if (days === 1)
            return `Yesterday ${time}`;
        return `${Qt.formatDateTime(at, Time.shortDateFormat)} ${time}`;
    }
    count: root.items.length
    unread: root.items.some(n => Notifs.isUnread(n))
    critical: root.notification?.urgency === NotificationUrgency.Critical
    remaining: countdown.remaining
    summary: root.notification?.summary ?? ""
    body: root.plain(root.notification?.body ?? "")
    mainAction: root.buttons.label
    otherActions: root.buttons.others.map(a => a.text)
    onDismissRequested: root.dismiss()
    onMainPicked: root.run(root.buttons.main)
    onOtherPicked: index => root.run(root.buttons.others[index])
    onMarkReadPicked: {
        Notifs.markRead(root.items);
        if (root.popup)
            Notifs.hidePopup(root.notification);
    }

    Countdown {
        id: countdown

        running: root.popup && !root.critical
        held: root.pointed || root.held
        duration: root.timeout
        onFinished: Notifs.hidePopups(root.items)
    }
}
