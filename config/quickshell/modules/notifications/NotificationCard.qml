import QtQuick
import Quickshell
import Quickshell.Services.Notifications
import KubaOS.Components as K
import qs.settings
import qs.services

// One notification. As a popup it hides itself after the timeout (paused on hover; a thin line
// along the bottom shows the time left). In the control center history it stays until dismissed.
//
// A header band says who it is from and when: the app's icon (or the notification's image) on a
// small icon tile, its name, and the day and time it came, with the dismiss button in the band's
// far corner. Under it, across the whole card, the summary, the body and the actions.
//
// The card is one step above what holds it, with its corners nested in it: the panel's surface
// radius 8 in as a popup on the stack, and the well's 12 in as a row of the history. A click
// opens what it is about; the dismiss button and the actions take their own clicks.
K.Pressable {
    id: root

    required property Notification notification
    property bool popup: false
    property bool selected: false
    readonly property bool critical: notification?.urgency === NotificationUrgency.Critical
    // expireTimeout is the raw D-Bus value in milliseconds (Quickshell's docs say seconds).
    readonly property int timeout: notification?.expireTimeout > 0 ? notification.expireTimeout : Settings.notificationTimeout
    readonly property string appIcon: {
        const icon = notification?.appIcon ?? "";
        if (icon === "")
            return "";
        return icon.startsWith("/") ? `file://${icon}` : Quickshell.iconPath(icon, true);
    }
    readonly property color fg: critical ? K.Tokens.color.on_error_container : K.Tokens.color.on_surface
    readonly property color fgMuted: critical ? K.Look.alpha(K.Tokens.color.on_error_container, K.Tokens.opacity.subdued_text) : K.Tokens.color.on_surface_variant
    // In the control center's well: one of its rows, so 12 in from its surface.
    property bool inWell: false
    readonly property real radius: K.Look.nested(K.Tokens.radius.surface, root.inWell ? K.Tokens.space.s12 : K.Tokens.space.s8)
    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    // The band's inset from the card's top and from its corner: the dismiss button sits as close
    // to the corner as the curve lets it, so the words below keep the room.
    readonly property int band: K.Tokens.space.s8
    readonly property var arrived: root.notification ? Notifs.arrived(root.notification) : null
    // The day and time it came: the time alone today, the day's name before it otherwise.
    readonly property string when: {
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

    // Time left before the popup hides (1 -> 0); paused while hovered.
    property real remaining: 1

    // `body` as plain words. A notification's body may carry the spec's small set of markup
    // (bold, italics, links, line breaks, entities), which a type style draws as written. A number
    // keeps its unit on its line ("41 s", "20 %"), so a line never ends in a lone number.
    // https://specifications.freedesktop.org/notification-spec/latest/markup.html
    function plain(body: string): string {
        return body.replace(/<br\s*\/?>/gi, "\n").replace(/<[^>]*>/g, "").replace(/&lt;/g, "<").replace(/&gt;/g, ">").replace(/&quot;/g, "\"").replace(/&apos;|&#39;/g, "'").replace(/&amp;/g, "&").replace(/(\d) (?=[A-Za-z%°]{1,3}\b)/g, "$1 ");
    }

    // Every height here is a binding, never a layout pass: a list places the cards after this one
    // by its height as it arrives, and doesn't move them for a change while they slide in.
    implicitHeight: K.Look.snapUp(words.y + words.height + K.Tokens.space.s12, root.dpr)
    activeFocusOnTab: false
    onActivated: Notifs.activate(root.notification)

    NumberAnimation on remaining {
        running: root.popup && !root.critical
        paused: running && root.pointerOver
        from: 1
        to: 0
        duration: root.timeout
        onFinished: Notifs.hidePopup(root.notification)
    }

    Rectangle {
        anchors.fill: parent
        radius: root.radius
        color: root.critical ? K.Tokens.color.error_container : root.popup ? K.Tokens.color.surface_container : K.Tokens.color.surface_container_high

        K.StateLayer {
            radius: parent.radius
            content: root.fg
            hovered: root.hovered && !dismiss.hovered
            pressed: root.down
        }
    }

    // The header band, a small control tall: who it is from and when.
    K.IconTile {
        id: tile

        x: K.Tokens.space.s12
        y: root.band
        size: "Sm"
        icon: root.critical ? "warning" : "notifications"
        source: root.appIcon
        photo: root.notification?.image ?? ""
        danger: root.critical
    }

    K.Text {
        id: app

        x: tile.x + tile.width + K.Tokens.space.s8
        y: root.band + K.Look.snap((K.Tokens.size.control_sm - height) / 2, root.dpr)
        room: Math.max(0, dismiss.x - K.Tokens.space.s8 - x - (time.visible ? time.width + K.Tokens.space.s8 : 0))
        // One line, cut at the band's room: an app's name past it breaks at a space onto a line
        // that isn't shown.
        height: K.Tokens.type.label.line
        clip: true
        text: root.notification?.appName || "Notification"
        style: K.Tokens.type.label
        color: root.fg
    }

    K.Text {
        id: time

        x: app.x + app.width + K.Tokens.space.s8
        y: root.band + K.Look.snap((K.Tokens.size.control_sm - height) / 2, root.dpr)
        visible: text !== ""
        text: root.when
        style: K.Tokens.type.small
        color: root.fgMuted
    }

    K.IconButton {
        id: dismiss

        x: root.width - root.band - width
        y: root.band
        icon: "close"
        label: qsTr("Dismiss")
        kind: "Text"
        size: "Sm"
        activeFocusOnTab: false
        onActivated: root.notification.dismiss()
    }

    // Under the band, across the card: the summary, the body, then the actions.
    Item {
        id: words

        readonly property real summaryHeight: summary.text !== "" ? summary.height : 0
        readonly property real bodyHeight: body.text !== "" ? K.Tokens.space.s4 + body.height : 0
        readonly property real actionsHeight: actions.count > 0 ? K.Tokens.space.s12 + K.Tokens.size.control_sm : 0

        x: K.Tokens.space.s12
        y: root.band + K.Tokens.size.control_sm + K.Tokens.space.s8
        width: Math.max(0, root.width - K.Tokens.space.s12 * 2)
        height: words.summaryHeight + words.bodyHeight + words.actionsHeight

        K.Text {
            id: summary

            visible: text !== ""
            room: words.width
            text: root.notification?.summary ?? ""
            style: K.Tokens.type.body_strong
            color: root.fg
            maximumLines: 2
        }

        K.Text {
            id: body

            y: words.summaryHeight + K.Tokens.space.s4
            visible: text !== ""
            room: words.width
            text: root.plain(root.notification?.body ?? "")
            style: K.Tokens.type.body
            color: root.fgMuted
            maximumLines: root.popup ? 4 : 6
        }

        K.Row {
            y: words.summaryHeight + words.bodyHeight + K.Tokens.space.s12
            visible: actions.count > 0
            gap: K.Tokens.space.s8

            Repeater {
                id: actions

                model: root.notification?.actions.filter(a => a.identifier !== "default") ?? []

                delegate: K.Button {
                    required property NotificationAction modelData

                    text: modelData.text
                    kind: root.critical ? "Danger" : "Tonal"
                    size: "Sm"
                    activeFocusOnTab: false
                    onActivated: modelData.invoke()
                }
            }
        }
    }

    // The time left, a thin line along the bottom inside the padding, on whole device pixels so it
    // repaints only when it loses one.
    Rectangle {
        x: K.Tokens.space.s12
        y: root.height - height
        width: K.Look.snap((root.width - K.Tokens.space.s12 * 2) * root.remaining, root.dpr)
        height: K.Look.thin(root.dpr)
        radius: K.Tokens.radius.full
        color: K.Tokens.color.primary
        visible: root.popup && !root.critical
    }

    K.FocusRing {
        shown: root.selected
        controlRadius: root.radius
    }
}
