pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io
import qs.settings

// What the shell's own panels change and remember across restarts, in one file,
// `~/.local/state/quickshell/kubaos.json`: the wallpaper motion mode (`Motion`) and whether
// Bluetooth lists its unnamed devices (`Bt`). One file has one writer, as each write puts the
// whole of it. The Command menu's on/off pairs read it too (crates/kotodama-kubaos, states.rs).
Singleton {
    id: root

    // "always", "ac" or "never"; empty for `Settings.wallpaperAnimate`.
    property alias animate: prefs.animate
    // Whether Bluetooth's Nearby lists devices that send no name.
    property alias bluetoothUnnamed: prefs.bluetoothUnnamed

    FileView {
        path: `${Quickshell.env("XDG_STATE_HOME") || `${Settings.home}/.local/state`}/quickshell/kubaos.json`
        printErrors: false
        atomicWrites: true
        onAdapterUpdated: writeAdapter()
        // First run: no file yet. Write the defaults so the next change has a place to go, and
        // the menu's pairs have a state to read.
        onLoadFailed: error => {
            if (error === FileViewError.FileNotFound)
                writeAdapter();
        }

        JsonAdapter {
            id: prefs

            property string animate: ""
            property bool bluetoothUnnamed: false
        }
    }
}
