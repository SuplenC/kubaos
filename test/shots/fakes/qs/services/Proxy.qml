pragma Singleton
import QtQuick

// Stand-in for Proxy: a full WireGuard tunnel with the kill switch, on,
// letting the local network through. `openvpn` makes it an OpenVPN one,
// which has no handshake to read.
QtObject {
    id: root

    property bool live: false
    property bool openvpn: false
    readonly property bool configured: true
    property bool on: true
    readonly property bool busy: false
    readonly property bool pending: false
    readonly property string problem: ""
    readonly property var tunnel: ({
            type: openvpn ? "openvpn" : "wireguard",
            typeName: typeName,
            interface: openvpn ? "tun-kubaos" : "wg-kubaos",
            endpoint: endpoint,
            full: true,
            killSwitch: true,
            allowLocalNetwork: true,
            localNetworks: ["192.168.1.0/24"]
        })
    readonly property var liveTunnel: openvpn ? ({
            handshake: null
        }) : ({
            handshake: 1793137620
        })
    readonly property string endpoint: openvpn ? "vpn.example.org:1194" : "vpn.example.org:51820"
    readonly property string typeName: openvpn ? "OpenVPN" : "WireGuard"
    readonly property bool full: true
    readonly property bool killSwitch: true
    property bool allowLocalNetwork: true
    readonly property string kind: "Full tunnel, kill switch"
    readonly property string subtitle: on ? endpoint : "Off"
    readonly property bool watching: live && on
    readonly property real lastReceived: 1793137640
    readonly property real now: 1793137652
    readonly property real rxBytes: 48230000
    readonly property real txBytes: 3120000

    function toggle(): void {
        root.on = !root.on;
    }

    function setLocalNetwork(allowed: bool): void {
        root.allowLocalNetwork = allowed;
    }

    function ago(seconds: real, now: real): string {
        return `${Math.round(now - seconds)} s ago`;
    }

    function bytes(count: real): string {
        return `${(count / 1e6).toFixed(1)} MB`;
    }
}
