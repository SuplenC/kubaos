pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io
import qs.settings

// ukiyo, the live wallpaper daemon (docs/UKIYO.md). A scene is the wallpaper
// when Wallpapers.current is `ukiyo:<id>`; dot-theme starts and stops the
// daemon. This side:
// - lists the installed scenes for the picker (`ukiyo scenes --json`)
// - watches the daemon's status over its socket, so the desktop keeps the
//   scene's poster up until ukiyo really draws (`drawing`)
// - maps Motion to the daemon: pause/resume (reason "motion") with
//   Motion.desktopOn, `still` while locked (the lock screen shows the frame
//   ukiyo saves then, `stillFor`), resume on unlock
// - keeps Motion's mode and ukiyo's config in step, both ways:
//     always <-> performance.on_battery full (or reduced)
//     ac     <-> performance.on_battery still
//     never  <-> performance.motion false
// - reads and writes ukiyo's config for the settings panel (`schema`,
//   `configSet`, `configReset`), through the CLI, which works with or
//   without a daemon
Singleton {
    id: root

    // The scene that is the wallpaper, "" for a still or a loop.
    readonly property string current: Wallpapers.current.startsWith("ukiyo:") ? Wallpapers.current.slice(6) : ""
    readonly property bool active: current !== ""
    // Installed scenes: [{ id, name, dir, poster, description }], sorted by id.
    property var scenes: []
    property bool listed: false
    // The last `status` answer, null when the daemon doesn't answer.
    property var status: null
    readonly property bool running: status !== null
    // The current scene is on every output ukiyo has.
    readonly property bool drawing: active && running && status.scene?.id === current && (status.outputs ?? []).length > 0 && status.outputs.every(o => o.configured)
    // ...and has been for a moment: what the desktop waits for to hide its poster.
    property bool shown: false
    // Every option, from `ukiyo config schema --json` (see the settings panel).
    property var schema: []
    property bool schemaLoaded: false
    // Lock frames by output name, and a counter that changes with each set.
    property var stillFrames: ({})
    property int stillSerial: 0

    readonly property string socketPath: {
        const own = Quickshell.env("UKIYO_SOCKET");
        if (own)
            return own;
        const display = (Quickshell.env("WAYLAND_DISPLAY") || "wayland-0").split("/").pop();
        return `${Quickshell.env("XDG_RUNTIME_DIR")}/ukiyo/${display}.sock`;
    }

    // Some screen may move: Motion's rules, applied to the daemon while unlocked.
    readonly property bool live: Quickshell.screens.some(s => Motion.desktopOn(s))
    readonly property bool locked: Session.locked

    function scene(id: string): var {
        return scenes.find(s => s.id === id) ?? null;
    }

    function poster(id: string): string {
        return scene(id)?.poster ?? "";
    }

    function name(id: string): string {
        return scene(id)?.name ?? id.replace(/[-_]+/g, " ");
    }

    // The lock frame for one output ("" until ukiyo has written it).
    function stillFor(output: string): string {
        return stillFrames[output] ?? "";
    }

    function refreshScenes(): void {
        sceneLister.running = true;
    }

    function refreshSchema(): void {
        if (schemaReader.running)
            schemaDirty = true;
        else
            schemaReader.running = true;
    }
    property bool schemaDirty: false

    // Option `key` as the user's own value (config.toml), live.
    function configSet(key: string, value: var): void {
        const text = Array.isArray(value) ? value.join(",") : typeof value === "boolean" ? (value ? "true" : "false") : String(value);
        run(["ukiyo", "config", "set", key, text], refreshSchema);
    }

    // Back to the default (or KubaOS's defaults.toml); `scenes.<id>` drops
    // all of a scene's overrides.
    function configReset(key: string): void {
        run(["ukiyo", "config", "reset", key], refreshSchema);
    }

    // Motion's mode -> ukiyo's config (Motion.setMode calls this).
    function pushMode(mode: string): void {
        // Before the first schema read there's nothing to compare with;
        // onSchemaChanged catches up.
        if (!available || !schemaLoaded)
            return;
        const motion = option("performance.motion")?.value ?? true;
        const battery = option("performance.on_battery")?.value ?? "still";
        if (mode === "never") {
            if (motion !== false)
                configSet("performance.motion", false);
            return;
        }
        if (motion !== true)
            configSet("performance.motion", true);
        if (mode === "ac" && battery !== "still")
            configSet("performance.on_battery", "still");
        else if (mode === "always" && battery === "still")
            configSet("performance.on_battery", "full");
    }

    // ukiyo's config -> Motion's mode.
    function modeFromConfig(): string {
        if (option("performance.motion")?.value === false)
            return "never";
        return option("performance.on_battery")?.value === "still" ? "ac" : "always";
    }

    function option(key: string): var {
        return schema.find(o => o.key === key) ?? null;
    }

    // One request on the daemon's socket; `done` gets the response object
    // (null if nothing answered). One connection, kept open: the daemon
    // answers each line in order, so the callbacks wait in a queue.
    function send(request: var, done: var): void {
        const line = `${JSON.stringify(request)}\n`;
        link.waiting.push(done ?? null);
        if (link.connected) {
            link.write(line);
            link.flush();
        } else {
            link.queue.push(line);
            link.connected = true;
        }
    }

    // A CLI call. Calls run one at a time, in order (they all write
    // config.toml); `done` runs when this one has exited.
    function run(command: list<string>, done: var): void {
        calls.push({
            command: command,
            done: done ?? null
        });
        if (!caller.running)
            caller.next();
    }
    property var calls: []

    function poll(): void {
        send({
            cmd: "status"
        }, r => {
            root.status = r?.ok ? r.data : null;
            if (root.status)
                root.reconcile();
        });
    }

    // Bring the daemon in line with Motion and the lock (each poll, so a
    // restarted daemon catches up too).
    function reconcile(): void {
        const st = status;
        if (!st || !active)
            return;
        if (locked) {
            if (!stillSent) {
                stillSent = true;
                send({
                    cmd: "still"
                }, () => root.readStills());
            } else if (st.still_serial !== stillSerial) {
                readStills();
            }
            return;
        }
        const paused = (st.reasons ?? []).includes("motion");
        if (live && paused)
            send({
                cmd: "resume",
                reason: "motion"
            });
        else if (!live && !paused)
            send({
                cmd: "pause",
                reason: "motion"
            });
    }
    property bool stillSent: false

    function readStills(): void {
        send({
            cmd: "status"
        }, r => {
            if (!r?.ok)
                return;
            root.status = r.data;
            const frames = {};
            for (const p of r.data.still_frames ?? []) {
                const m = /\/still-(.+)\.png$/.exec(p);
                if (m)
                    frames[m[1]] = p;
            }
            root.stillFrames = frames;
            root.stillSerial = r.data.still_serial ?? 0;
        });
    }

    readonly property bool available: listed && scenes.length > 0

    onLockedChanged: {
        if (locked) {
            // Last lock's frames are stale: the poster until the new ones exist.
            stillFrames = {};
            stillSent = false;
            if (active)
                poll();
        } else {
            stillSent = false;
            if (active)
                send({
                    cmd: "resume",
                    reason: "lock"
                }, () => root.poll());
        }
    }
    onLiveChanged: if (active && !locked)
        poll()
    onActiveChanged: {
        shown = false;
        status = null;
        if (active)
            poll();
    }
    onDrawingChanged: {
        if (drawing)
            settle.restart();
        else {
            settle.stop();
            shown = false;
        }
    }
    onSchemaChanged: {
        // ukiyo's config wins where the user set it there (the panel, or
        // `ukiyo config set`); otherwise Motion's mode goes to ukiyo.
        if (!schemaLoaded)
            return;
        const own = ["performance.motion", "performance.on_battery"].some(k => option(k)?.source === "user");
        const mode = modeFromConfig();
        if (mode === Motion.mode)
            return;
        if (own)
            Motion.adopt(mode);
        else
            pushMode(Motion.mode);
    }

    Component.onCompleted: {
        refreshScenes();
        if (active)
            poll();
    }

    // Poll fast while waiting for the first frame (for 10 s, then every 2 s:
    // a daemon that doesn't come up shouldn't cost 4 wakeups a second),
    // slowly once it draws.
    Timer {
        property int polls: 0

        running: root.active
        repeat: true
        interval: root.shown ? 3000 : polls < 40 ? 250 : 2000
        onRunningChanged: polls = 0
        onTriggered: {
            polls = root.shown ? 0 : polls + 1;
            root.poll();
        }
    }

    // A frame or two after ukiyo says it draws, so its surface is on screen.
    Timer {
        id: settle

        interval: 400
        onTriggered: root.shown = root.drawing
    }

    Process {
        id: sceneLister

        command: ["ukiyo", "scenes", "--json"]
        stdout: StdioCollector {
            onStreamFinished: {
                let list = [];
                try {
                    list = JSON.parse(text);
                } catch (e) {}
                root.scenes = (Array.isArray(list) ? list : []).filter(s => s.poster);
                root.listed = true;
                if (root.available && !root.schemaLoaded)
                    root.refreshSchema();
            }
        }
        // Not installed: no scenes, nothing else changes.
        onExited: code => {
            if (code !== 0)
                root.listed = true;
        }
    }

    Process {
        id: schemaReader

        command: ["ukiyo", "config", "schema", "--json", ...(root.current ? ["--scene", root.current] : [])]
        stdout: StdioCollector {
            onStreamFinished: {
                let list = null;
                try {
                    list = JSON.parse(text);
                } catch (e) {}
                if (Array.isArray(list)) {
                    root.schemaLoaded = true;
                    root.schema = list;
                }
                if (root.schemaDirty) {
                    root.schemaDirty = false;
                    schemaReader.running = true;
                }
            }
        }
    }

    Socket {
        id: link

        // Callbacks for the requests sent, oldest first; lines waiting for
        // the connection.
        property var waiting: []
        property var queue: []

        function drop(): void {
            const w = waiting;
            waiting = [];
            queue = [];
            for (const done of w)
                if (done)
                    done(null);
        }

        path: root.socketPath
        parser: SplitParser {
            onRead: data => {
                const done = link.waiting.shift();
                let r = null;
                try {
                    r = JSON.parse(data);
                } catch (e) {}
                if (done)
                    done(r);
            }
        }
        onConnectedChanged: {
            if (connected) {
                for (const line of queue)
                    write(line);
                queue = [];
                flush();
            } else {
                drop();
            }
        }
        // No daemon (or it went away): everyone waiting hears null.
        onError: {
            connected = false;
            drop();
        }
    }

    Process {
        id: caller

        property var done: null

        function next(): void {
            const call = root.calls.shift();
            if (!call)
                return;
            done = call.done;
            command = call.command;
            running = true;
        }

        stderr: StdioCollector {
            onStreamFinished: if (text.trim())
                console.warn(`ukiyo: ${text.trim()}`)
        }
        onExited: {
            const d = done;
            done = null;
            if (d)
                d();
            Qt.callLater(next);
        }
    }
}
