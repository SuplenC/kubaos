pragma Singleton
import QtQuick

// Stand-in for Motion: wallpapers move on AC, and the machine is on battery, so nothing plays.
QtObject {
    id: root

    readonly property list<string> modes: ["ac", "always", "never"]
    property string mode: "ac"
    readonly property string modeLabel: mode === "always" ? "Always" : mode === "never" ? "Never" : "On AC"
    readonly property bool onBattery: true
    readonly property bool allowed: false
    readonly property bool screensOff: false
    readonly property bool desktop: false
    readonly property bool lock: false
    property var playing: ({})

    function setMode(m: string): bool {
        root.mode = m;
        return true;
    }

    function cycle(): void {
    }

    function covered(screen: var): bool {
        return false;
    }

    function desktopOn(screen: var): bool {
        return false;
    }

    function report(screen: string, path: string): void {
    }
}
