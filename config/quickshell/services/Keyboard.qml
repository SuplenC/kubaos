pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io
import Quickshell.Hyprland

// The keyboard layouts Hyprland types with, and the one in use, from
// `hyprctl devices -j`. Read again on each layout switch and config reload,
// never polled; `kubaos keyboard` reloads the config after a change.
Singleton {
    id: root

    // "us", "it": the layouts of the main keyboard, the default first.
    property var layouts: []
    property int active: 0
    readonly property bool several: layouts.length > 1
    // "US": the layout in use, as the bar shows it.
    readonly property string code: (layouts[active] ?? "").toUpperCase()

    function next(): void {
        Quickshell.execDetached(["hyprctl", "switchxkblayout", "all", "next"]);
    }

    // The keyboard Hyprland calls main, or the first: every keyboard has the
    // same layouts, as `switchxkblayout all` moves them together.
    function parse(text: string): void {
        let keyboards = [];
        try {
            keyboards = JSON.parse(text).keyboards ?? [];
        } catch (e) {
            return;
        }
        const keyboard = keyboards.find(k => k.main) ?? keyboards[0];
        if (!keyboard)
            return;
        root.layouts = (keyboard.layout ?? "").split(",").map(l => l.trim()).filter(l => l !== "");
        root.active = keyboard.active_layout_index ?? 0;
    }

    Process {
        id: devices

        running: true
        command: ["hyprctl", "devices", "-j"]
        stdout: StdioCollector {
            onStreamFinished: root.parse(text)
        }
    }

    // A switch sends `activelayout` once a keyboard, so the reads wait for the last.
    Timer {
        id: settle

        interval: 50
        onTriggered: devices.running = true
    }

    Connections {
        target: Hyprland

        function onRawEvent(event: HyprlandEvent): void {
            if (event.name === "activelayout" || event.name === "configreloaded")
                settle.restart();
        }
    }
}
