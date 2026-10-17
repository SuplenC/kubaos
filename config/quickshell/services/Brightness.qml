pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io

// Screen backlight. Reads sysfs (watched, so key presses handled by Hyprland
// show up here) and writes through brightnessctl, which needs no root.
Singleton {
    id: root

    property string device: ""
    readonly property int max: parseInt(maxFile.text()) || 0
    readonly property real value: max > 0 ? (parseInt(currentFile.text()) || 0) / max : 0
    readonly property bool available: max > 0
    readonly property string icon: value < 0.34 ? "brightness_low" : value < 0.67 ? "brightness_medium" : "brightness_high"

    function set(v: real): void {
        const percent = Math.round(Math.max(0.01, Math.min(1, v)) * 100);
        Quickshell.execDetached(["brightnessctl", "--quiet", "set", `${percent}%`]);
    }

    Process {
        running: true
        command: ["sh", "-c", "ls /sys/class/backlight | head -n1"]
        stdout: StdioCollector {
            onStreamFinished: root.device = text.trim()
        }
    }

    FileView {
        id: maxFile

        path: root.device ? `/sys/class/backlight/${root.device}/max_brightness` : ""
    }

    FileView {
        id: currentFile

        path: root.device ? `/sys/class/backlight/${root.device}/brightness` : ""
        watchChanges: true
        onFileChanged: reload()
    }
}
