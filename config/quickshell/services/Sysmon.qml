pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io
import qs.settings

// The system monitor's state: the bar's pill, the popup and the alerts all
// read the one stream.
//
// Samples come from `kubaos-sysmon serve`, one process for the shell's life,
// which reads /proc and /sys itself: a sample costs a line, not a process.
// The shell tells it the pace: every 2 s for the pill, every second while the
// popup is open, and nothing while the screen is locked or nothing shows. A
// request is a line of JSON on its stdin; the reply a line with its id. The
// settings file (~/.config/kubaos/desktop/sysmon/sysmon.toml) is watched, so a change
// from the Command menu shows at once.
Singleton {
    id: root

    readonly property string configDir: UserLayer.dir("sysmon")
    // The graphs' window, in ms.
    readonly property int window: 60000
    readonly property var defaults: ({
            pill: true,
            style: "numbers",
            metrics: ["cpu", "ram"],
            alerts: true,
            thresholds: {
                cpuTemp: 90,
                ram: 90,
                disk: 95,
                batteryDraw: 30
            }
        })
    readonly property var allMetrics: ["cpu", "ram", "temp", "gpu", "net", "power"]

    // The last sample, or null before the first.
    property var sample: null
    property var settings: defaults
    // The engine's reason the settings file was not read, or "".
    property string problem: ""
    // The engine is running; false where kubaos-sysmon isn't installed.
    readonly property bool available: engine.running

    readonly property bool open: Panels.current === "sysmon"
    readonly property var thresholds: settings.thresholds ?? defaults.thresholds
    readonly property string mode: {
        if (Session.locked)
            return "paused";
        if (open)
            return "popup";
        if (!settings.pill && !settings.alerts)
            return "paused";
        return Power.quiet ? "quiet" : "pill";
    }

    // The last minute of each series, as [t, value] pairs, oldest first. The
    // arrays change in place, so `revision` is what the graphs repaint on.
    // Every series exists from the start: a graph binds to its array once,
    // and a key added later would never reach it.
    property var history: ({
            cpu: [],
            temp: [],
            ram: [],
            swap: [],
            gpu: [],
            down: [],
            up: [],
            net: [],
            power: [],
            read: [],
            write: []
        })
    property int revision: 0

    // The process the popup shows the details of, as `qs ipc call sysmon
    // process` or an alert asked, until it opens.
    property int pendingPid: 0

    signal processRequested(int pid)

    onModeChanged: sendMode()

    function sendMode(): void {
        request("mode", { mode: root.mode }, null);
    }

    function show(): void {
        Panels.open("sysmon");
    }

    function toggle(): void {
        Panels.toggle("sysmon");
    }

    // Opens the popup on process `pid`'s details.
    function showProcess(pid: int): void {
        pendingPid = pid;
        Panels.open("sysmon");
        processRequested(pid);
    }

    property int nextId: 1
    property var waiting: ({})

    // Sends `op` with `fields`, and calls `done(reply)` with the reply.
    function request(op: string, fields: var, done: var): void {
        if (!engine.running) {
            done?.({ error: "kubaos-sysmon isn't running" });
            return;
        }
        const id = root.nextId++;
        if (done)
            root.waiting[id] = done;
        engine.write(JSON.stringify(Object.assign({ id: id, op: op }, fields ?? {})) + "\n");
    }

    function refreshSettings(): void {
        request("settings", {}, reply => {
            if (reply.settings) {
                root.settings = reply.settings;
                root.problem = reply.problem ?? "";
            }
        });
    }

    // Changes setting `key` to `value`; `done(error)`, "" when it took.
    function set(key: string, value: string, done: var): void {
        request("set", { key: key, value: value }, reply => {
            if (reply.settings)
                root.settings = reply.settings;
            done?.(reply.error ?? "");
        });
    }

    // Asks process `pid` to exit (`term`) or stops it (`kill`); `done(error)`.
    function stop(pid: int, how: string, done: var): void {
        request("signal", { pid: pid, signal: how }, reply => done?.(reply.error ?? ""));
    }

    function record(s: var): void {
        const series = {
            cpu: s.cpu?.usage,
            temp: s.cpu?.temp,
            ram: s.memory?.percent,
            swap: s.memory?.swapTotal > 0 ? s.memory.swapUsed * 100 / s.memory.swapTotal : null,
            gpu: s.gpus?.[0]?.usage,
            down: s.net?.down,
            up: s.net?.up,
            net: s.net?.down != null ? s.net.down + (s.net.up ?? 0) : null,
            power: s.battery?.watts,
            read: s.io?.read,
            write: s.io?.write
        };
        const since = s.t - root.window;
        for (const key in series) {
            const points = root.history[key];
            if (series[key] !== null && series[key] !== undefined)
                points.push([s.t, series[key]]);
            while (points.length > 0 && points[0][0] < since)
                points.shift();
        }
        root.sample = s;
        root.revision++;
    }

    // The colour step of a reading: 0 fine, 1 close to the threshold
    // (amber), 2 over it (red).
    function level(value: var, threshold: real, margin: real): int {
        if (value === null || value === undefined)
            return 0;
        return value > threshold ? 2 : value > threshold - margin ? 1 : 0;
    }

    readonly property int tempLevel: level(sample?.cpu?.temp, thresholds.cpuTemp, 10)
    readonly property int ramLevel: level(sample?.memory?.percent, thresholds.ram, 10)
    readonly property var fullDisks: (sample?.disks ?? []).filter(d => d.percent > thresholds.disk)

    // A pill metric: its icon, its text, a 0..1 fill for the bars, its
    // colour step and the series it draws.
    function metric(name: string): var {
        const s = root.sample;
        switch (name) {
        case "cpu":
            return { icon: "memory", text: percent(s?.cpu?.usage), fill: (s?.cpu?.usage ?? 0) / 100, level: tempLevel, series: "cpu", max: 100 };
        case "ram":
            return { icon: "memory_alt", text: percent(s?.memory?.percent), fill: (s?.memory?.percent ?? 0) / 100, level: ramLevel, series: "ram", max: 100 };
        case "temp":
            return { icon: "thermostat", text: s?.cpu?.temp != null ? `${Math.round(s.cpu.temp)}°` : "–", fill: (s?.cpu?.temp ?? 0) / Math.max(1, thresholds.cpuTemp + 10), level: tempLevel, series: "temp", max: thresholds.cpuTemp + 10 };
        case "gpu":
            return { icon: "developer_board", text: percent(s?.gpus?.[0]?.usage), fill: (s?.gpus?.[0]?.usage ?? 0) / 100, level: 0, series: "gpu", max: 100 };
        case "net":
            return { icon: "swap_vert", text: s?.net?.down != null ? rate(s.net.down + (s.net.up ?? 0)) : "–", fill: 0, level: 0, series: "net", max: 0 };
        case "power":
            return { icon: "bolt", text: s?.battery?.watts != null ? `${s.battery.watts.toFixed(1)} W` : "–", fill: (s?.battery?.watts ?? 0) / Math.max(1, thresholds.batteryDraw), level: level(s?.battery?.watts, thresholds.batteryDraw, 5), series: "power", max: 0 };
        }
        return null;
    }

    function percent(value: var): string {
        return value === null || value === undefined ? "–" : `${Math.round(value)}%`;
    }

    // "512 MB", "9.1 GB": powers of 1024, one decimal under ten, as
    // `kubaos_sysmon::format::bytes` writes them for the menu.
    function bytes(value: real): string {
        const units = ["B", "KB", "MB", "GB", "TB"];
        let v = value, unit = 0;
        while (v >= 1024 && unit < units.length - 1) {
            v /= 1024;
            unit++;
        }
        return unit === 0 || v >= 10 ? `${Math.round(v)} ${units[unit]}` : `${v.toFixed(1)} ${units[unit]}`;
    }

    function rate(value: real): string {
        return `${bytes(value)}/s`;
    }

    // "3 h 12 min", "45 min", "2 d 4 h".
    function duration(seconds: real): string {
        const d = Math.floor(seconds / 86400), h = Math.floor(seconds % 86400 / 3600), m = Math.floor(seconds % 3600 / 60);
        if (d > 0)
            return `${d} d ${h} h`;
        return h > 0 ? `${h} h ${m} min` : `${m} min`;
    }

    // Shows alert `a` as a notification; its button runs its action through
    // `act`, back in this shell.
    function alert(a: var): void {
        Quickshell.execDetached(["sh", "-c", 'a=$(notify-send -a "System monitor" -i utilities-system-monitor -A "go=$3" "$1" "$2") && [ "$a" = go ] && qs ipc call sysmon act "$4"', "sh", a.title, a.body, a.actionLabel, a.action]);
    }

    // What an alert's button asked for: `open`, `process:<pid>`,
    // `files:<mount>` or `power-saver`.
    function act(action: string): void {
        if (action.startsWith("process:"))
            showProcess(Number(action.slice(8)));
        else if (action.startsWith("files:"))
            Quickshell.execDetached(["xdg-open", action.slice(6)]);
        else if (action === "power-saver")
            Power.setId("power-saver");
        else
            show();
    }

    // What `qs ipc call sysmon status` says.
    function status(): var {
        return {
            available: root.available,
            open: root.open,
            mode: root.mode,
            pill: !!root.settings.pill,
            metrics: root.settings.metrics ?? [],
            problem: root.problem,
            sample: root.sample !== null,
            detail: root.sample?.detail ?? false,
            cpu: root.sample?.cpu?.usage ?? null,
            ram: root.sample?.memory?.percent ?? null
        };
    }

    Process {
        id: engine

        command: ["kubaos-sysmon", "serve"]
        running: true
        stdinEnabled: true
        stdout: SplitParser {
            onRead: line => {
                let message;
                try {
                    message = JSON.parse(line);
                } catch (e) {
                    console.warn(`Sysmon: unreadable line from kubaos-sysmon: ${line}`);
                    return;
                }
                restart.failures = 0;
                if (message.type === "sample") {
                    root.record(message);
                } else if (message.type === "alert") {
                    root.alert(message);
                } else {
                    const done = root.waiting[message.id];
                    delete root.waiting[message.id];
                    done?.(message);
                }
            }
        }
        onStarted: {
            root.sendMode();
            root.refreshSettings();
        }
        // A crash or an update of the binary: start again, a little later,
        // and later each time it fails before a line, up to a minute.
        onExited: (code, status) => {
            root.waiting = {};
            restart.interval = Math.min(60000, 2000 * Math.pow(2, restart.failures++));
            restart.restart();
        }
    }

    Timer {
        id: restart

        property int failures: 0

        onTriggered: engine.running = true
    }

    FileView {
        path: `${root.configDir}/sysmon.toml`
        watchChanges: true
        printErrors: false
        onFileChanged: root.refreshSettings()
    }
}
