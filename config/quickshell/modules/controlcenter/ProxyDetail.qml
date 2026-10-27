import QtQuick
import QtQuick.Layouts
import qs.theme
import qs.components
import qs.services

// The Proxy tile's detail, under the tiles while its arrow is open: the
// state, the tunnel's endpoint, when it last received, its last handshake
// and the bytes moved, read live while it shows (services/Proxy.qml).
Rectangle {
    id: root

    readonly property var rows: {
        const live = Proxy.liveTunnel;
        const list = [
            {
                label: "State",
                value: Proxy.busy ? "Switching…" : Proxy.on ? "On" : Proxy.problem !== "" ? Proxy.problem : "Off"
            }
        ];
        if (Proxy.tunnel) {
            list.push({
                label: "Tunnel",
                value: `${Proxy.kind} to ${Proxy.endpoint || "no endpoint"}`
            });
            if (Proxy.watching) {
                list.push({
                    label: "Last received",
                    value: Proxy.lastReceived > 0 ? Proxy.ago(Proxy.lastReceived, Proxy.now) : "Not since this opened"
                });
                if (live && live.handshake !== null && live.handshake !== undefined)
                    list.push({
                        label: "Last handshake",
                        value: Proxy.ago(live.handshake, Proxy.now)
                    });
                if (Proxy.rxBytes >= 0)
                    list.push({
                        label: "Moved",
                        value: `${Proxy.bytes(Proxy.rxBytes)} in · ${Proxy.bytes(Math.max(0, Proxy.txBytes))} out`
                    });
            }
        }
        return list;
    }

    implicitHeight: column.implicitHeight + Theme.spacing.md * 2
    radius: Theme.rounding.md
    color: Theme.m3surfaceContainerHigh

    Component.onCompleted: Proxy.live = visible
    onVisibleChanged: Proxy.live = visible
    Component.onDestruction: Proxy.live = false

    ColumnLayout {
        id: column

        anchors.fill: parent
        anchors.margins: Theme.spacing.md
        spacing: Theme.spacing.xs

        Repeater {
            model: root.rows

            RowLayout {
                required property var modelData

                Layout.fillWidth: true
                spacing: Theme.spacing.md

                StyledText {
                    text: modelData.label
                    font.pixelSize: Theme.font.sm
                    color: Theme.m3onSurfaceVariant
                }

                StyledText {
                    Layout.fillWidth: true
                    horizontalAlignment: Text.AlignRight
                    elide: Text.ElideMiddle
                    text: modelData.value
                    font.pixelSize: Theme.font.sm
                    font.features: Theme.font.tabular
                }
            }
        }
    }
}
