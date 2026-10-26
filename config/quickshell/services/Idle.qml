pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io

// Keep awake and the scheduled lock, from the files `kubaos-idle` writes in
// $XDG_RUNTIME_DIR (crates/kubaos-idle: kubaos-keep-awake.json,
// kubaos-scheduled-lock.json, each replaced whole on every change). Watched,
// never polled; the clock only ticks while either is on. Both outlive a shell
// restart: the files and the systemd user units behind them are kubaos-idle's.
Singleton {
    id: root

    readonly property string runtime: Quickshell.env("XDG_RUNTIME_DIR")
    property var keepAwakeInfo: ({})
    property var lockInfo: ({})

    // Seconds since the epoch, to the second while anything counts down.
    readonly property int now: Math.floor(clock.date.getTime() / 1000)

    // When Keep awake ends, in seconds since the epoch; 0 for until turned off.
    readonly property int keepAwakeUntil: keepAwakeInfo.until ?? 0
    readonly property bool keepAwake: (keepAwakeInfo.active ?? false) && (keepAwakeUntil === 0 || keepAwakeUntil > now)

    // When the scheduled lock comes, in seconds since the epoch.
    readonly property int lockAt: lockInfo.at ?? 0
    readonly property bool lockPending: lockAt > now
    readonly property int lockLeft: lockPending ? lockAt - now : 0

    // "4:59" under an hour, "1:05:00" over, "2 d" from two days.
    function countdown(seconds: int): string {
        if (seconds >= 172800)
            return `${Math.round(seconds / 86400)} d`;
        const h = Math.floor(seconds / 3600), m = Math.floor(seconds / 60) % 60, s = seconds % 60;
        const mmss = `${String(m).padStart(h ? 2 : 1, "0")}:${String(s).padStart(2, "0")}`;
        return h ? `${h}:${mmss}` : mmss;
    }

    function stopKeepAwake(): void {
        Quickshell.execDetached(["kubaos-idle", "awake", "off"]);
    }

    function cancelLock(): void {
        Quickshell.execDetached(["kubaos-idle", "lock", "cancel"]);
    }

    SystemClock {
        id: clock

        // On while either could be on, so `keepAwake` and `lockPending` turn
        // false on time, not at the next file change. Seconds only for the
        // lock's countdown; Keep awake shows no time in the bar.
        enabled: (root.keepAwakeInfo.active ?? false) || root.lockAt > 0
        precision: root.lockAt > 0 ? SystemClock.Seconds : SystemClock.Minutes
    }

    FileView {
        id: keepAwakeFile

        path: `${root.runtime}/kubaos-keep-awake.json`
        watchChanges: true
        printErrors: false
        onFileChanged: reload()
        onLoaded: root.keepAwakeInfo = root.parse(text())
        // Not there before the first Keep awake of the login: create it, so
        // the watch has a file to follow.
        onLoadFailed: {
            root.keepAwakeInfo = {};
            keepAwakeCreator.running = true;
        }
    }

    FileView {
        id: lockFile

        path: `${root.runtime}/kubaos-scheduled-lock.json`
        watchChanges: true
        printErrors: false
        onFileChanged: reload()
        onLoaded: root.lockInfo = root.parse(text())
        onLoadFailed: {
            root.lockInfo = {};
            lockCreator.running = true;
        }
    }

    function parse(text: string): var {
        try {
            return JSON.parse(text) ?? {};
        } catch (e) {
            return {};
        }
    }

    Process {
        id: keepAwakeCreator

        command: ["sh", "-c", '[ -e "$1" ] || printf \'{"active":false,"until":0}\\n\' > "$1"', "sh", keepAwakeFile.path]
        onExited: keepAwakeFile.reload()
    }

    Process {
        id: lockCreator

        command: ["sh", "-c", '[ -e "$1" ] || printf \'{"at":0}\\n\' > "$1"', "sh", lockFile.path]
        onExited: lockFile.reload()
    }
}
