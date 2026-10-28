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
    // An ambient light sensor: an IIO device reading illuminance, which iio-sensor-proxy serves.
    property bool sensor: false
    // A service that hands the backlight to the sensor. None runs, so no slider offers Auto.
    readonly property bool autoBackend: false
    // Whether the brightness slider's icon switches Auto.
    readonly property bool autoAvailable: root.sensor && root.autoBackend

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

    Process {
        running: true
        command: ["sh", "-c", "ls /sys/bus/iio/devices/*/in_illuminance_* >/dev/null 2>&1"]
        onExited: code => root.sensor = code === 0
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
