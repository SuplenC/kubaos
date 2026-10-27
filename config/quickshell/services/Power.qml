pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io
import Quickshell.Services.UPower

// The power stops, Super Saver to Max Performance: the stop the power
// service (`kubaos-power serve`, crates/kubaos-power) is on, from the file it
// replaces whole on every change ($XDG_RUNTIME_DIR/kubaos-power.json), and
// moving to another through `kubaos-power set`. Watched, never polled.
// Without the service, the stop named like power-profiles-daemon's profile.
Singleton {
    id: root

    readonly property string stateFile: `${Quickshell.env("XDG_RUNTIME_DIR")}/kubaos-power.json`
    // In slider order; the ids are kubaos-power's, the titles its too.
    readonly property var stops: [
        {
            id: "super-saver",
            title: "Super Saver",
            icon: "bedtime"
        },
        {
            id: "power-saver",
            title: "Power Saver",
            icon: "eco"
        },
        {
            id: "balanced",
            title: "Balanced",
            icon: "balance"
        },
        {
            id: "performance",
            title: "Performance",
            icon: "bolt"
        },
        {
            id: "max-performance",
            title: "Max Performance",
            icon: "rocket_launch"
        }
    ]
    readonly property var titles: stops.map(stop => stop.title)
    // What the service last wrote, null while none runs.
    property var published: null
    // A stop picked here that hasn't come back from the service yet, -1 for
    // none: the slider shows the pick at once, and a step on the way doesn't
    // pull it back.
    property int pending: -1
    // The process that wrote the file still runs: a service killed outright
    // leaves its file behind, which then says nothing true.
    property bool alive: false
    readonly property bool service: published !== null && alive
    readonly property int profileIndex: PowerProfiles.profile === PowerProfile.PowerSaver ? 1 : PowerProfiles.profile === PowerProfile.Performance ? 3 : 2
    readonly property int index: pending >= 0 ? pending : service ? published.index : profileIndex
    readonly property string title: stops[index].title
    readonly property string icon: stops[index].icon
    // A saver stop holds background work back: the system monitor samples less often.
    readonly property bool quiet: published?.quiet ?? false

    function set(to: int): void {
        const clamped = Math.max(0, Math.min(stops.length - 1, to));
        if (clamped === index)
            return;
        pending = clamped;
        confirm.restart();
        Quickshell.execDetached(["kubaos-power", "set", stops[clamped].id]);
    }

    // By its id: `power-saver`.
    function setId(id: string): void {
        set(stops.findIndex(stop => stop.id === id));
    }

    onProfileIndexChanged: if (!service && pending === profileIndex)
        pending = -1

    // A pick the service never confirms (it stopped, or refused) gives the
    // slider back to what is true.
    Timer {
        id: confirm

        interval: 3000
        onTriggered: root.pending = -1
    }

    FileView {
        id: file

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
            process.reload();
            if (root.published?.index === root.pending)
                root.pending = -1;
        }
        onLoadFailed: root.published = null
    }

    // The writer's name, from /proc: `kubaos-power` while it runs.
    FileView {
        id: process

        path: root.published?.pid ? `/proc/${root.published.pid}/comm` : ""
        // Read at once, so the slider never shows the stale fallback for a frame.
        blockLoading: true
        printErrors: false
        onLoaded: root.alive = text().trim() === "kubaos-power"
        onLoadFailed: root.alive = false
    }

    // A file that isn't there can't be watched: the service starts beside
    // the shell at login and after a restart, so while there is none, look
    // again now and then. One failed open; nothing runs once it's there.
    Timer {
        interval: 2000
        repeat: true
        running: root.published === null
        onTriggered: file.reload()
    }

    // A dead writer leaves no event behind, so its name is asked again now
    // and then while the file stands: one small read.
    Timer {
        interval: 10000
        repeat: true
        running: root.published !== null
        onTriggered: process.reload()
    }
}
