pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io
import qs.settings

// The Proxy, the work network's switch: what `kubaos-proxy` last published
// ($XDG_RUNTIME_DIR/kubaos-proxy.json, crates/kubaos-proxy, replaced whole
// on every change), and switching it through `kubaos-proxy on|off`. Watched,
// never polled. While `live` is set and the tunnel is up, the bytes moved
// come from /sys every few seconds, which needs no root, and when the tunnel
// last received; the handshake needs root (`status --json --live`, through
// pkexec), so it is read as the detail opens and then once a minute. An edit
// of the profile is read again at once (`status --json`), so the tile shows
// as soon as it has a part.
Singleton {
    id: root

    readonly property string stateFile: `${Quickshell.env("XDG_RUNTIME_DIR")}/kubaos-proxy.json`
    // What the Proxy last published, null before it has.
    property var published: null
    // The last live read, with the handshake and the byte counts.
    property var liveStatus: null
    // A switch asked here that hasn't come back yet: the tile shows it at once.
    property bool pending: false
    // The control center's detail is open: read the tunnel live.
    property bool live: false
    readonly property bool watching: live && on && tunnel !== null
    readonly property string statistics: "/sys/class/net/wg-kubaos/statistics"
    property real rxBytes: -1
    property real txBytes: -1
    // When the received bytes last grew, in seconds since the epoch; 0 before
    // they have while the detail showed.
    property real lastReceived: 0
    // Seconds since the epoch, as of the last read, which "… ago" counts from.
    property real now: Date.now() / 1000

    onWatchingChanged: {
        rxBytes = -1;
        txBytes = -1;
        lastReceived = 0;
        now = Date.now() / 1000;
    }

    readonly property bool configured: published?.configured ?? false
    readonly property bool on: published?.on ?? false
    readonly property bool busy: pending || (published?.busy ?? false)
    readonly property var tunnel: published?.tunnel ?? null
    readonly property var liveTunnel: liveStatus?.tunnel ?? null
    readonly property string endpoint: tunnel?.endpoint ?? ""
    readonly property bool full: tunnel?.full ?? false
    readonly property bool killSwitch: tunnel?.killSwitch ?? false
    readonly property string problem: published?.problem ?? ""
    readonly property string kind: !tunnel ? "" : full ? (killSwitch ? "Full tunnel, kill switch" : "Full tunnel") : "Split tunnel"
    // The tile's status line.
    readonly property string subtitle: busy ? (on ? "Turning off…" : "Turning on…") : on ? (endpoint || "On") : problem !== "" ? "Didn't switch" : "Off"

    function toggle(): void {
        if (busy || !configured)
            return;
        pending = true;
        confirm.restart();
        Quickshell.execDetached(["kubaos-proxy", on ? "off" : "on"]);
    }

    // `seconds` since the epoch, as how long before `now`: "12 s ago".
    function ago(seconds: real, now: real): string {
        if (!seconds)
            return "never";
        const secs = Math.max(0, Math.round(now - seconds));
        return secs < 60 ? `${secs} s ago` : secs < 3600 ? `${Math.floor(secs / 60)} min ago` : `${Math.floor(secs / 3600)} h ago`;
    }

    function bytes(count: real): string {
        const units = ["B", "kB", "MB", "GB", "TB"];
        let value = count;
        let unit = 0;
        while (value >= 1000 && unit < units.length - 1) {
            value /= 1000;
            unit++;
        }
        return unit === 0 ? `${count} B` : `${value.toFixed(1)} ${units[unit]}`;
    }

    // A switch the Proxy never confirms (the command died) gives the tile back
    // to what is true.
    Timer {
        id: confirm

        interval: 30000
        onTriggered: root.pending = false
    }

    FileView {
        path: root.stateFile
        watchChanges: true
        printErrors: false
        onFileChanged: reload()
        onLoaded: {
            try {
                root.published = JSON.parse(text());
            } catch (e) {
                root.published = null;
            }
            if (!(root.published?.busy ?? false))
                root.pending = false;
        }
        onLoadFailed: root.published = null
    }

    // The profile, edited by hand or by the import: whether it has a part.
    FileView {
        path: `${UserLayer.dir("network")}/proxy.toml`
        watchChanges: true
        printErrors: false
        onFileChanged: if (!profile.running)
            profile.running = true
    }

    Process {
        id: profile

        command: ["kubaos-proxy", "status", "--json"]
        stdout: StdioCollector {
            onStreamFinished: {
                if (root.busy)
                    return;
                try {
                    root.published = JSON.parse(text);
                } catch (e) {}
            }
        }
    }

    Process {
        id: status

        command: ["kubaos-proxy", "status", "--json", "--live"]
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    root.liveStatus = JSON.parse(text);
                } catch (e) {
                    root.liveStatus = null;
                }
            }
        }
    }

    // The handshake, as the detail opens and then once a minute: each read is
    // a pkexec run.
    Timer {
        running: root.watching
        interval: 60000
        repeat: true
        triggeredOnStart: true
        onTriggered: if (!status.running)
            status.running = true
    }

    FileView {
        id: rx

        path: root.watching ? `${root.statistics}/rx_bytes` : ""
        printErrors: false
        onLoaded: {
            const count = parseInt(text());
            if (isNaN(count))
                return;
            if (root.rxBytes >= 0 && count > root.rxBytes)
                root.lastReceived = root.now;
            root.rxBytes = count;
        }
    }

    FileView {
        id: tx

        path: root.watching ? `${root.statistics}/tx_bytes` : ""
        printErrors: false
        onLoaded: {
            const count = parseInt(text());
            if (!isNaN(count))
                root.txBytes = count;
        }
    }

    Timer {
        running: root.watching
        interval: 5000
        repeat: true
        onTriggered: {
            root.now = Date.now() / 1000;
            rx.reload();
            tx.reload();
        }
    }
}
