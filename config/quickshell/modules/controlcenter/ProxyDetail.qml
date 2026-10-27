import QtQuick
import KubaOS.Components as K
import qs.services

// The Proxy tile's detail, under the tiles while its arrow is open: the
// state, the tunnel's endpoint, when it last received, its last handshake
// and the bytes moved, read live while it shows (services/Proxy.qml). The
// values tick while it shows, so they are mono and keep their width.
K.Card {
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

    Component.onCompleted: Proxy.live = visible
    onVisibleChanged: Proxy.live = visible
    Component.onDestruction: Proxy.live = false

    K.Stack {
        fill: true
        gap: K.Tokens.space.s4

        // By count, so a row lives on while its value ticks instead of being made again each
        // second.
        Repeater {
            model: root.rows.length

            K.Row {
                required property int index
                readonly property var modelData: root.rows[index] ?? ({})

                fill: true
                gap: K.Tokens.space.s12
                align: "centre"

                K.Text {
                    text: modelData.label ?? ""
                    style: K.Tokens.type.small
                    color: K.Tokens.color.on_surface_variant
                }

                K.Fill {
                }

                K.Text {
                    text: modelData.value ?? ""
                    style: K.Tokens.type.mono
                    color: K.Tokens.color.on_surface
                    maximumLines: 1
                }
            }
        }
    }
}
