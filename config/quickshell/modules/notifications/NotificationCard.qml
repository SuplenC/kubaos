import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Widgets
import Quickshell.Services.Notifications
import qs.theme
import qs.settings
import qs.components
import qs.services

// One notification. As a popup it hides itself after the timeout (paused on
// hover; a thin line along the bottom shows the time left). In the control
// center history it stays until dismissed.
Rectangle {
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
    readonly property color fg: critical ? Theme.m3onErrorContainer : Theme.m3onSurface
    readonly property color fgMuted: critical ? Theme.alpha(Theme.m3onErrorContainer, 0.8) : Theme.m3onSurfaceVariant

    implicitHeight: layout.implicitHeight + Theme.spacing.md * 2
    radius: Theme.rounding.lg
    color: critical ? Theme.m3errorContainer : popup ? Theme.m3surfaceContainer : Theme.m3surfaceContainerHigh
    border.width: selected ? 2 : 0
    border.color: Theme.m3primary
    clip: true

    // Time left before the popup hides (1 -> 0); paused while hovered.
    property real remaining: 1

    NumberAnimation on remaining {
        running: root.popup && !root.critical
        paused: running && stateLayer.containsMouse
        from: 1
        to: 0
        duration: root.timeout
        onFinished: Notifs.hidePopup(root.notification)
    }

    StateLayer {
        id: stateLayer

        onClicked: Notifs.activate(root.notification)
    }

    RowLayout {
        id: layout

        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.margins: Theme.spacing.md
        spacing: Theme.spacing.md

        Rectangle {
            Layout.alignment: Qt.AlignTop
            implicitWidth: 40
            implicitHeight: 40
            radius: Theme.rounding.md
            color: root.critical ? Theme.m3error : Theme.m3secondaryContainer

            ClippingRectangle {
                anchors.fill: parent
                radius: parent.radius
                color: "transparent"
                visible: root.iconSource !== "" && image.status === Image.Ready

                Image {
                    id: image

                    anchors.fill: parent
                    source: root.iconSource
                    fillMode: Image.PreserveAspectCrop
                    sourceSize: Qt.size(96, 96)
                    asynchronous: true
                    smooth: true
                    mipmap: true
                }
            }

            Icon {
                anchors.centerIn: parent
                visible: !(root.iconSource !== "" && image.status === Image.Ready)
                name: root.critical ? "warning" : "notifications"
                filled: true
                color: root.critical ? Theme.m3onError : Theme.m3onSecondaryContainer
            }
        }

        ColumnLayout {
            Layout.fillWidth: true
            spacing: 2

            RowLayout {
                Layout.fillWidth: true
                spacing: Theme.spacing.xs

                StyledText {
                    Layout.fillWidth: true
                    text: root.notification?.summary ?? ""
                    font.weight: Font.DemiBold
                    color: root.fg
                }

                StyledText {
                    text: root.notification?.appName ?? ""
                    font.pixelSize: Theme.font.xs
                    color: root.fgMuted
                    Layout.maximumWidth: 120
                }

                IconButton {
                    icon: "close"
                    size: 22
                    tint: root.fgMuted
                    activeFocusOnTab: false
                    onClicked: root.notification.dismiss()
                }
            }

            StyledText {
                Layout.fillWidth: true
                visible: text !== ""
                text: root.notification?.body ?? ""
                textFormat: Text.StyledText
                wrapMode: Text.Wrap
                maximumLineCount: root.popup ? 3 : 6
                color: root.fgMuted
                lineHeight: 1.1
            }

            Flow {
                Layout.fillWidth: true
                Layout.topMargin: Theme.spacing.sm
                spacing: Theme.spacing.xs
                visible: actions.count > 0

                Repeater {
                    id: actions

                    model: root.notification?.actions.filter(a => a.identifier !== "default") ?? []

                    delegate: Rectangle {
                        id: action

                        required property NotificationAction modelData

                        implicitWidth: actionLabel.implicitWidth + Theme.spacing.lg * 2
                        implicitHeight: 30
                        radius: Theme.rounding.full
                        color: root.critical ? Theme.m3error : Theme.m3secondaryContainer

                        StyledText {
                            id: actionLabel

                            anchors.centerIn: parent
                            text: action.modelData.text
                            color: root.critical ? Theme.m3onError : Theme.m3onSecondaryContainer
                            font.pixelSize: Theme.font.sm
                            font.weight: Font.Medium
                        }

                        StateLayer {
                            onClicked: action.modelData.invoke()
                        }
                    }
                }
            }
        }
    }

    Rectangle {
        anchors.left: parent.left
        anchors.bottom: parent.bottom
        anchors.leftMargin: Theme.spacing.md
        height: 2
        radius: 1
        width: (parent.width - Theme.spacing.md * 2) * root.remaining
        color: Theme.alpha(Theme.m3primary, 0.6)
        visible: root.popup && !root.critical
    }
}
