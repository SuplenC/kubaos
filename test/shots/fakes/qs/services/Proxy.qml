pragma Singleton
import QtQuick

// Stand-in for Proxy: a split tunnel, on.
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
            full: false,
            killSwitch: false
        })
    readonly property var liveTunnel: ({
            handshake: 1793137620
        })
    readonly property string endpoint: "vpn.example.org:51820"
    readonly property bool full: false
    readonly property bool killSwitch: false
    readonly property string kind: "Split tunnel"
    readonly property string subtitle: on ? endpoint : "Off"
    readonly property bool watching: live && on
    readonly property real lastReceived: 1793137640
    readonly property real now: 1793137652
    readonly property real rxBytes: 48230000
    readonly property real txBytes: 3120000

    function toggle(): void {
        root.on = !root.on;
    }

    function ago(seconds: real, now: real): string {
        return `${Math.round(now - seconds)} s ago`;
    }

    function bytes(count: real): string {
        return `${(count / 1e6).toFixed(1)} MB`;
    }
}
