import QtQuick
import Quickshell
import Quickshell.Services.Notifications
import KubaOS.Components as K
import qs.settings
import qs.services
import qs.components

// One notification, or a stack of one app's, the same card in the popups, the bell's peek and the
// notifications panel. As a popup it hides itself after the timeout (stopped while the pointer is
// on it; a thin line along the bottom shows the time left).
//
// A header band says who it is from and when: the app's icon (or the notification's image) on a
// small icon tile, an accent dot on its corner while unread, its name, the stack's count, and the
// day and time it came; the dismiss button shows in the band's far corner while the pointer is on
// the card or the keyboard is. Under it, across the whole card, the summary, two lines of the body
// and the notification's own actions, the first filled.
//
// The card is one step above what holds it, its corners the panel's surface radius 8 in. A click
// opens what it is about, or asks for the stack to open (`expandRequested`) where a stack opens;
// the dismiss button and the actions take their own clicks.
K.Pressable {
    id: root

    // The newest of `items`, which the card shows.
    required property Notification notification
    // The notifications the card stands for, newest first: one, or a stack.
    property var items: [root.notification]
    readonly property bool stacked: root.items.length > 1
    // Whether a click on a stack asks for it to open rather than opening its newest.
    property bool expands: false
    property bool popup: false
    property bool selected: false
    readonly property bool unread: root.items.some(n => Notifs.isUnread(n))
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
    readonly property real radius: K.Look.nested(K.Tokens.radius.surface, K.Tokens.space.s8)
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

    // Whether something keeps it, as its open menu does: a popup waits while it is held.
    property bool held: false
    // Time left before the popup hides (1 -> 0).
    readonly property real remaining: countdown.remaining

    // `body` as plain words. A notification's body may carry the spec's small set of markup
    // (bold, italics, links, line breaks, entities), which a type style draws as written. A number
    // keeps its unit on its line ("41 s", "20 %"), so a line never ends in a lone number.
    // https://specifications.freedesktop.org/notification-spec/latest/markup.html
    function plain(body: string): string {
        return body.replace(/<br\s*\/?>/gi, "\n").replace(/<[^>]*>/g, "").replace(/&lt;/g, "<").replace(/&gt;/g, ">").replace(/&quot;/g, "\"").replace(/&apos;|&#39;/g, "'").replace(/&amp;/g, "&").replace(/(\d) (?=[A-Za-z%°]{1,3}\b)/g, "$1 ");
    }

    // What a click or Enter does: open the stack where stacks open, else run the newest's main
    // action and mark every notification the card stands for read.
    function open(): void {
        if (root.stacked && root.expands) {
            root.expandRequested();
            return;
        }
        Notifs.markRead(root.items);
        Notifs.activate(root.notification);
    }

    function dismiss(): void {
        Notifs.dismissAll(root.items);
    }

    // Every height here is a binding, never a layout pass: a list places the cards after this one
    // by its height as it arrives, and doesn't move them for a change while they slide in.
    implicitHeight: K.Look.snapUp(words.y + words.height + K.Tokens.space.s12, root.dpr)
    activeFocusOnTab: false
    onActivated: root.open()

    // A right-click on the card asks for its menu, at the pointer in the window's coordinates.
    signal menuRequested(point at)
    signal expandRequested

    Countdown {
        id: countdown

        running: root.popup && !root.critical
        held: root.pointerOver || root.held
        duration: root.timeout
        onFinished: Notifs.hidePopup(root.notification)
    }

    // Under the card's buttons, which take their own clicks; it takes the right button alone, so
    // a left click and the pointer's hover pass through to the card.
    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.RightButton
        onClicked: mouse => root.menuRequested(mapToItem(null, mouse.x, mouse.y))
    }

    Rectangle {
        anchors.fill: parent
        radius: root.radius
        color: root.critical ? K.Tokens.color.error_container : K.Tokens.color.surface_container

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

    K.Badge {
        x: K.Look.snap(tile.x + tile.width - width / 2, root.dpr)
        y: K.Look.snap(tile.y - height / 2, root.dpr)
        visible: root.unread
        accent: true
    }

    K.Text {
        id: app

        x: tile.x + tile.width + K.Tokens.space.s8
        y: root.band + K.Look.snap((K.Tokens.size.control_sm - height) / 2, root.dpr)
        room: Math.max(0, dismiss.x - K.Tokens.space.s8 - x - (count.visible ? count.width + K.Tokens.space.s8 : 0) - (time.visible ? time.width + K.Tokens.space.s8 : 0))
        // One line, cut at the band's room: an app's name past it breaks at a space onto a line
        // that isn't shown.
        height: K.Tokens.type.label.line
        clip: true
        text: root.notification?.appName || "Notification"
        style: K.Tokens.type.label
        color: root.fg
    }

    // How many the stack holds.
    K.Chip {
        id: count

        x: app.x + app.width + K.Tokens.space.s8
        y: root.band + K.Look.snap((K.Tokens.size.control_sm - height) / 2, root.dpr)
        visible: root.stacked
        text: String(root.items.length)
    }

    K.Text {
        id: time

        x: (count.visible ? count.x + count.width : app.x + app.width) + K.Tokens.space.s8
        y: root.band + K.Look.snap((K.Tokens.size.control_sm - height) / 2, root.dpr)
        visible: text !== ""
        text: root.when
        style: K.Tokens.type.small
        color: root.fgMuted
    }

    // Shown while the pointer is on the card or the keyboard is, its room kept at rest so the band
    // never moves.
    K.IconButton {
        id: dismiss

        x: root.width - root.band - width
        y: root.band
        opacity: root.pointerOver || dismiss.pointerOver || root.selected ? 1 : 0
        enabled: opacity > 0
        icon: "close"
        label: root.stacked ? qsTr("Dismiss all %1").arg(root.items.length) : qsTr("Dismiss")
        kind: "Text"
        size: "Sm"
        activeFocusOnTab: false
        onActivated: root.dismiss()

        Behavior on opacity {
            NumberAnimation {
                duration: K.Tokens.duration.micro
                easing.type: Easing.BezierSpline
                easing.bezierCurve: K.Tokens.easing.standard
            }
        }
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
            maximumLines: 2
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
                    required property int index

                    text: modelData.text
                    kind: index > 0 ? "Text" : root.critical ? "Danger" : "Filled"
                    size: "Sm"
                    activeFocusOnTab: false
                    onActivated: {
                        Notifs.markRead(root.items);
                        modelData.invoke();
                    }
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
