pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io

// What the daily update check found, from the file `kubaos update --check`
// writes (~/.local/state/kubaos/update-available, `key=value` lines, empty
// when there is nothing to install). Watched, never polled.
Singleton {
    id: root

    readonly property string stateFile: `${Quickshell.env("XDG_STATE_HOME") || Quickshell.env("HOME") + "/.local/state"}/kubaos/update-available`
    property var info: ({})
    // "main at 1a2b3c4 (0.1.1)" or "0.1.2": what kubaos update installs.
    readonly property string update: info.update ?? ""
    // "0.2.0": the next series, which only kubaos upgrade installs.
    readonly property string upgrade: info.upgrade ?? ""
    readonly property string upgradeCodename: info["upgrade-codename"] ?? ""
    readonly property bool available: update !== "" || upgrade !== ""

    // In a terminal, so sudo's prompt and apt's output stay visible.
    function run(command: string): void {
        Quickshell.execDetached(["ghostty", "--wait-after-command=true", "-e", "kubaos", command]);
    }

    function parse(text: string): var {
        const out = {};
        for (const line of text.split("\n")) {
            const i = line.indexOf("=");
            if (i > 0)
                out[line.slice(0, i)] = line.slice(i + 1);
        }
        return out;
    }

    FileView {
        id: file

        path: root.stateFile
        watchChanges: true
        printErrors: false
        onFileChanged: reload()
        onLoaded: root.info = root.parse(text())
        // Not there before the first check: create it empty, so the watch has
        // a file to follow.
        onLoadFailed: {
            root.info = {};
            creator.running = true;
        }
    }

    Process {
        id: creator

        command: ["sh", "-c", 'mkdir -p "$(dirname "$1")" && { [ -e "$1" ] || : > "$1"; }', "sh", root.stateFile]
        onExited: file.reload()
    }
}
