import QtQuick
import Quickshell
import Quickshell.Widgets
import Quickshell.Services.Notifications
import KubaOS.Components as K
import qs.settings
import qs.services

// One notification. As a popup it hides itself after the timeout (paused on
// hover; a thin line along the bottom shows the time left). In the control
// center history it stays until dismissed.
//
// As a popup it fills its stack 8 in from the panel's edge, so its corners are
// the panel's surface radius nested 8 in; in the history's well, 12 in. A click
// opens what it is about; the dismiss button and the actions take their own
// clicks.
K.Pressable {
    id: root

    required property Notification notification
    property bool popup: false
    property bool selected: false
    readonly property bool critical: notification?.urgency === NotificationUrgency.Critical
    // expireTimeout is the raw D-Bus value in milliseconds (Quickshell's docs say seconds).
    readonly property int timeout: notification?.expireTimeout > 0 ? notification.expireTimeout : Settings.notificationTimeout
    readonly property string iconSource: {
        const n = notification;
        if (!n)
            return "";
        if (n.image)
            return n.image;
        if (n.appIcon.startsWith("/"))
            return `file://${n.appIcon}`;
        return n.appIcon ? Quickshell.iconPath(n.appIcon, true) : "";
    }
    readonly property color fg: critical ? K.Tokens.color.on_error_container : K.Tokens.color.on_surface
    readonly property color fgMuted: critical ? K.Look.alpha(K.Tokens.color.on_error_container, K.Tokens.opacity.subdued_text) : K.Tokens.color.on_surface_variant
    // In the control center's well: one of its rows, so 12 in from its surface.
    property bool inWell: false
    readonly property real radius: K.Look.nested(K.Tokens.radius.surface, root.inWell ? K.Tokens.space.s12 : K.Tokens.space.s8)
    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    readonly property bool imageShown: root.iconSource !== "" && image.status === Image.Ready

    // Time left before the popup hides (1 -> 0); paused while hovered.
    property real remaining: 1

    // Where a header text sits: centred on the dismiss button, on a whole device pixel.
    function headerLine(text: Item): real {
        return K.Tokens.space.s12 + K.Look.snap((K.Tokens.size.control_sm - text.implicitHeight) / 2, root.dpr);
    }

    // Every height here is a binding, never a layout pass: a list places the cards after this one
    // by its height as it arrives, and doesn't move them for a change while they slide in.
    implicitHeight: K.Look.snapUp(K.Tokens.space.s12 * 2 + Math.max(K.Tokens.size.control_md, K.Tokens.size.control_sm + words.height), root.dpr)
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
            hovered: root.hovered
            pressed: root.down
        }
    }

    // The app's icon or the notification's image on a tile, or a bell.
    Item {
        id: tile

        x: K.Tokens.space.s12
        y: K.Tokens.space.s12
        width: K.Tokens.size.control_md
        height: K.Tokens.size.control_md

        Rectangle {
            anchors.fill: parent
            radius: K.Tokens.radius.control
            color: root.critical ? K.Tokens.color.error : K.Tokens.color.secondary_container
        }

        ClippingRectangle {
            anchors.fill: parent
            radius: K.Tokens.radius.control
            color: "transparent"
            visible: root.imageShown

            Image {
                id: image

                anchors.fill: parent
                source: root.iconSource
                fillMode: Image.PreserveAspectCrop
                sourceSize: Qt.size(Math.ceil(width * root.dpr), Math.ceil(height * root.dpr))
                asynchronous: true
                smooth: true
                mipmap: true
            }
        }

        K.Icon {
            x: K.Look.snap((K.Tokens.size.control_md - size) / 2, root.dpr)
            y: K.Look.snap((K.Tokens.size.control_md - size) / 2, root.dpr)
            visible: !root.imageShown
            name: root.critical ? "warning" : "notifications"
            size: K.Tokens.icon.s20
            color: root.critical ? K.Tokens.color.on_error : K.Tokens.color.on_secondary_container
        }
    }

    // The header, a small control tall: the summary, the app's name and the dismiss button.
    K.IconButton {
        id: dismiss

        x: root.width - K.Tokens.space.s12 - width
        y: K.Tokens.space.s12
        icon: "close"
        label: qsTr("Dismiss")
        kind: "Text"
        size: "Sm"
        activeFocusOnTab: false
        onActivated: root.notification.dismiss()
    }

    K.Text {
        id: app

        x: dismiss.x - K.Tokens.space.s8 - width
        y: root.headerLine(app)
        room: words.width / 2
        text: root.notification?.appName ?? ""
        style: K.Tokens.type.caption
        color: root.fgMuted
        maximumLines: 1
    }

    K.Text {
        id: summary

        x: words.x
        y: root.headerLine(summary)
        width: Math.max(0, app.x - K.Tokens.space.s8 - x)
        room: width
        text: root.notification?.summary ?? ""
        style: K.Tokens.type.body_strong
        color: root.fg
        maximumLines: 1
    }

    // Under the header, beside the tile: the body, then the actions.
    Item {
        id: words

        readonly property real bodyHeight: body.text !== "" ? K.Tokens.space.s4 + body.implicitHeight : 0
        // `body` as plain words. A notification's body may carry the spec's small set of markup
        // (bold, italics, links, line breaks, entities), which a type style draws as written.
        // https://specifications.freedesktop.org/notification-spec/latest/markup.html
        function plain(body: string): string {
            return body.replace(/<br\s*\/?>/gi, "\n").replace(/<[^>]*>/g, "").replace(/&lt;/g, "<").replace(/&gt;/g, ">").replace(/&quot;/g, "\"").replace(/&apos;|&#39;/g, "'").replace(/&amp;/g, "&");
        }

        readonly property real actionsHeight: actions.count > 0 ? K.Tokens.space.s12 + K.Tokens.size.control_sm : 0

        x: tile.x + tile.width + K.Tokens.space.s12
        y: K.Tokens.space.s12 + K.Tokens.size.control_sm
        width: Math.max(0, root.width - K.Tokens.space.s12 - x)
        height: words.bodyHeight + words.actionsHeight

        K.Text {
            id: body

            y: K.Tokens.space.s4
            visible: text !== ""
            room: words.width
            text: words.plain(root.notification?.body ?? "")
            style: K.Tokens.type.body
            color: root.fgMuted
            maximumLines: root.popup ? 3 : 6
        }

        K.Row {
            y: words.bodyHeight + K.Tokens.space.s12
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
