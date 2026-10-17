pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io

// Screen recording state, from the file scripts/dot-hypr-record writes
// ($XDG_RUNTIME_DIR/dot-record.json, replaced atomically on every change).
// Watched, never polled; the elapsed-time clock only ticks while recording.
Singleton {
    id: root

    readonly property string stateFile: `${Quickshell.env("XDG_RUNTIME_DIR")}/dot-record.json`
    property var info: ({})
    readonly property string state: info.state ?? "idle"
    readonly property bool recording: state === "recording"
    readonly property bool replay: state === "replay"
    readonly property bool active: recording || replay
    readonly property string mode: info.mode ?? ""
    readonly property int started: info.started ?? 0
    // Seconds since start, ticking once a second while active.
    readonly property int elapsed: active ? Math.max(0, Math.floor(clock.date.getTime() / 1000) - started) : 0
    readonly property string elapsedText: {
        const h = Math.floor(elapsed / 3600), m = Math.floor(elapsed / 60) % 60, s = elapsed % 60;
        const mmss = `${String(m).padStart(h ? 2 : 1, "0")}:${String(s).padStart(2, "0")}`;
        return h ? `${h}:${mmss}` : mmss;
    }

    // Commands go through the script, which owns the recorder and the state.
    // Region selection runs after the panel has closed, so slurp sees the desktop.
    function run(args: list<string>, delayed: bool): void {
        if (delayed)
            Quickshell.execDetached(["sh", "-c", 'sleep 0.4; exec "$@"', "sh", "dot-hypr-record", ...args]);
        else
            Quickshell.execDetached(["dot-hypr-record", ...args]);
    }
    function stop(): void {
        run(["stop"], false);
    }
    function toggle(): void {
        if (active)
            stop();
        else
            run(["output"], false);
    }
    function record(mode: string): void {
        run([mode], true);
    }
    function toggleReplay(): void {
        run(["replay", "toggle"], false);
    }
    function saveReplay(): void {
        run(["replay", "save"], false);
    }

    SystemClock {
        id: clock

        enabled: root.active
        precision: SystemClock.Seconds
    }

    FileView {
        id: file

        path: root.stateFile
        watchChanges: true
        printErrors: false
        onFileChanged: reload()
        onLoaded: {
            try {
                root.info = JSON.parse(text()) ?? {};
            } catch (e) {
                root.info = {};
            }
        }
        // Not there before the first recording of the session: create it, so
        // the watch has a file to follow.
        onLoadFailed: {
            root.info = {};
            creator.running = true;
        }
    }

    Process {
        id: creator

        command: ["sh", "-c", '[ -e "$1" ] || printf \'{"state":"idle"}\\n\' > "$1"', "sh", root.stateFile]
        onExited: file.reload()
    }
}
