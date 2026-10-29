pragma Singleton
import QtQuick

// Stand-in for Proxy: a full tunnel with the kill switch, on, letting the
// local network through.
QtObject {
    id: root

    property bool live: false
    readonly property bool configured: true
    property bool on: true
    readonly property bool busy: false
    readonly property bool pending: false
    readonly property string problem: ""
    readonly property var tunnel: ({
            endpoint: "vpn.example.org:51820",
            full: true,
            killSwitch: true,
            allowLocalNetwork: true,
            localNetworks: ["192.168.1.0/24"]
        })
    readonly property var liveTunnel: ({
            handshake: 1793137620
        })
    readonly property string endpoint: "vpn.example.org:51820"
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
