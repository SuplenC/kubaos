pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io

// The live preview the home is on, from the record `kubaos switch --from` writes
// (~/.local/state/kubaos/preview.json, `crates/kubaos-core/src/preview.rs`): empty
// while the home is the release. Watched, never polled.
Singleton {
    id: root

    readonly property string stateFile: `${Quickshell.env("XDG_STATE_HOME") || Quickshell.env("HOME") + "/.local/state"}/kubaos/preview.json`
    property var record: ({})
    readonly property bool active: (record.path ?? "") !== ""
    // The checkout's branch, empty on a detached head.
    readonly property string branch: record.branch ?? ""
    readonly property string commit: (record.commit ?? "").slice(0, 7)
    // What the bar names it by: the branch, else the commit.
    readonly property string name: branch !== "" ? branch : commit
    readonly property bool dirty: record.dirty ?? false
    // When the home first switched to the checkout, in milliseconds since the epoch.
    readonly property real since: record.since ?? 0

    function parse(text: string): var {
        if (text.trim() === "")
            return {};
        try {
            return JSON.parse(text);
        } catch (e) {
            return {};
        }
    }

    FileView {
        id: file

        path: root.stateFile
        watchChanges: true
        printErrors: false
        onFileChanged: reload()
        onLoaded: root.record = root.parse(text())
        // Not there before the first preview: create it empty, so the watch
        // has a file to follow.
        onLoadFailed: {
            root.record = {};
            creator.running = true;
        }
    }

    Process {
        id: creator

        command: ["sh", "-c", 'mkdir -p "$(dirname "$1")" && { [ -e "$1" ] || : > "$1"; }', "sh", root.stateFile]
        onExited: file.reload()
    }
}
