import QtQuick
import KubaOS.Components as K
import qs.services

// The Proxy tile's detail, under the tiles while its arrow is open: the
// state, the VPN's type and kind, its server, whether a full tunnel's kill
// switch lets the local network through (a switch), when it last received, a
// WireGuard VPN's last handshake and the bytes moved, read live while it
// shows (services/Proxy.qml). The values tick while it shows, so they are
// mono and keep their width.
K.Card {
    id: root

    // The rows above the switch, which hold still while the detail shows.
    readonly property var steadyRows: {
        const list = [
            {
                label: "State",
                value: Proxy.busy ? "Switching…" : Proxy.on ? "On" : Proxy.problem !== "" ? Proxy.problem : "Off"
            }
        ];
        if (Proxy.tunnel) {
            list.push({
                label: "VPN",
                value: Proxy.typeName !== "" ? `${Proxy.typeName}, ${Proxy.kind.toLowerCase()}` : Proxy.kind
            });
            list.push({
                label: "Server",
                value: Proxy.endpoint || "None"
            });
        }
        return list;
    }
    // The rows read live, below the switch, so they come and go without
    // moving it.
    readonly property var liveRows: {
        const live = Proxy.liveTunnel;
        const list = [];
        if (!Proxy.tunnel || !Proxy.watching)
            return list;
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
        return list;
    }

    Component.onCompleted: Proxy.live = visible
    onVisibleChanged: Proxy.live = visible
    Component.onDestruction: Proxy.live = false

    // A label on the left and its value on the right.
    component ValueRow: K.Row {
        id: valueRow

        property string label
        property string value

        fill: true
        gap: K.Tokens.space.s12
        align: "centre"

        K.Text {
            text: valueRow.label
            style: K.Tokens.type.small
            color: K.Tokens.color.on_surface_variant
        }

        K.Fill {
        }

        K.Text {
            text: valueRow.value
            style: K.Tokens.type.mono
            color: K.Tokens.color.on_surface
            maximumLines: 1
        }
    }

    K.Stack {
        fill: true
        gap: K.Tokens.space.s4

        // By count, so a row lives on while its value ticks instead of being made again each
        // second.
        Repeater {
            model: root.steadyRows.length

            ValueRow {
                required property int index

                label: root.steadyRows[index]?.label ?? ""
                value: root.steadyRows[index]?.value ?? ""
            }
        }

        K.Row {
            visible: Proxy.killSwitch
            fill: true
            gap: K.Tokens.space.s12
            align: "centre"

            K.Text {
                text: "Allow local network"
                style: K.Tokens.type.small
                color: K.Tokens.color.on_surface_variant
            }

            K.Fill {
            }

            K.Switch {
                checked: Proxy.allowLocalNetwork
                enabled: !Proxy.busy
                onToggled: Proxy.setLocalNetwork(!checked)
            }
        }

        Repeater {
            model: root.liveRows.length

            ValueRow {
                required property int index

                label: root.liveRows[index]?.label ?? ""
                value: root.liveRows[index]?.value ?? ""
            }
        }
    }
}
