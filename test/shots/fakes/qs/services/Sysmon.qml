pragma Singleton
import QtQuick

// Stand-in for Sysmon: one detailed sample of a busy laptop, and a minute of history rising and
// falling. The drivers set `pendingPid` for the process state.
QtObject {
    id: root

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
    readonly property bool available: true
    property string problem: ""
    property var settings: ({
            pill: true,
            style: "numbers",
            metrics: ["cpu", "ram", "temp"],
            alerts: true,
            thresholds: root.defaults.thresholds
        })
    readonly property var thresholds: settings.thresholds ?? defaults.thresholds
    readonly property bool open: true
    readonly property string mode: "popup"
    property int revision: 1
    property int pendingPid: 0
    property var sample: ({
            t: 60000,
            detail: true,
            uptime: 192840,
            kernel: "6.17.0-6-generic",
            cpu: {
                usage: 37,
                count: 16,
                mhz: 3412,
                load: [2.41, 1.98, 1.73],
                temp: 61,
                cores: [42, 18, 77, 9, 33, 51, 12, 64, 28, 5, 39, 88, 21, 47, 15, 60]
            },
            memory: {
                percent: 71,
                total: 33285996544,
                used: 18790000000,
                cache: 4900000000,
                available: 9600000000,
                swapTotal: 8589934592,
                swapUsed: 1288490188
            },
            gpus: [
                {
                    vendor: "AMD Radeon 780M",
                    discrete: false,
                    active: true,
                    usage: 23,
                    vramTotal: 2147483648,
                    vramUsed: 912261120,
                    temp: 54,
                    mhz: 1800
                }
            ],
            net: {
                interface: "wlp1s0",
                address: "192.168.1.42",
                down: 1240000,
                up: 84000
            },
            io: {
                read: 3400000,
                write: 812000
            },
            disks: [
                {
                    label: "/",
                    point: "/",
                    free: 214748364800,
                    total: 1000204886016,
                    percent: 79
                },
                {
                    label: "Backup",
                    point: "/media/kuba/Backup",
                    free: 51539607552,
                    total: 512110190592,
                    percent: 92
                }
            ],
            snapshots: {
                count: 14,
                bytes: 6442450944
            },
            battery: {
                percent: 64,
                status: "Discharging",
                watts: 11.4,
                secondsLeft: 15420,
                health: 93
            },
            fans: [
                {
                    label: "cpu_fan",
                    rpm: 2380
                }
            ],
            top: {
                cpu: [root.firefox, root.hyprland, root.analyzer, root.quickshell, root.ghostty],
                memory: [root.firefox, root.analyzer, root.hyprland, root.quickshell, root.ghostty]
            }
        })
    readonly property var firefox: ({
            pid: 4211,
            name: "firefox",
            cpu: 38.4,
            memory: 2254857830
        })
    readonly property var hyprland: ({
            pid: 1873,
            name: "Hyprland",
            cpu: 12.1,
            memory: 402653184
        })
    readonly property var analyzer: ({
            pid: 9031,
            name: "rust-analyzer",
            cpu: 9.7,
            memory: 1932735283
        })
    readonly property var quickshell: ({
            pid: 2210,
            name: "quickshell",
            cpu: 4.2,
            memory: 268435456
        })
    readonly property var ghostty: ({
            pid: 7712,
            name: "ghostty",
            cpu: 1.3,
            memory: 157286400
        })
    // A minute of samples, one every two seconds, as waves.
    readonly property var history: ({
            cpu: root.wave(34, 18, 0),
            temp: root.wave(58, 4, 0.5),
            ram: root.wave(71, 4, 1),
            swap: root.wave(15, 1, 1.5),
            gpu: root.wave(23, 10, 2),
            down: root.wave(820000, 600000, 2),
            up: root.wave(90000, 60000, 3),
            net: root.wave(910000, 620000, 2.5),
            power: root.wave(11, 3, 3.5),
            read: root.wave(3400000, 2000000, 4),
            write: root.wave(812000, 500000, 4.5)
        })
    readonly property int tempLevel: level(sample?.cpu?.temp, thresholds.cpuTemp, 10)
    readonly property int ramLevel: level(sample?.memory?.percent, thresholds.ram, 10)
    readonly property var fullDisks: (sample?.disks ?? []).filter(d => d.percent > thresholds.disk)

    signal processRequested(int pid)

    function wave(base: real, swing: real, phase: real): var {
        const points = [];
        for (let i = 0; i < 31; i++)
            points.push([i * 2000, base + swing * Math.sin(i / 3 + phase)]);
        return points;
    }

    function sendMode(): void {
    }

    function show(): void {
    }

    function toggle(): void {
    }

    function showProcess(pid: int): void {
        root.pendingPid = pid;
        root.processRequested(pid);
    }

    function request(op: string, fields: var, done: var): void {
        if (op === "process")
            done?.({
                process: {
                    pid: fields.pid,
                    name: "firefox",
                    command: "/usr/lib/firefox/firefox -contentproc -childID 12 -isForBrowser -prefsLen 31000 -prefMapSize 244000 -jsInitLen 231800 -parentBuildID 20261020 -greomni /usr/lib/firefox/omni.ja",
                    user: "kuba",
                    state: "Sleeping",
                    threads: 87,
                    memory: 2254857830,
                    started: 1793080000,
                    parent: 1
                }
            });
        else
            done?.({});
    }

    function refreshSettings(): void {
    }

    function set(key: string, value: string, done: var): void {
        done?.("");
    }

    function stop(pid: int, how: string, done: var): void {
        done?.("");
    }

    function record(s: var): void {
    }

    function level(value: var, threshold: real, margin: real): int {
        if (value === null || value === undefined)
            return 0;
        return value > threshold ? 2 : value > threshold - margin ? 1 : 0;
    }

    function metric(name: string): var {
        const s = root.sample;
        switch (name) {
        case "cpu":
            return {
                icon: "memory",
                text: percent(s.cpu.usage),
                fill: s.cpu.usage / 100,
                level: tempLevel,
                series: "cpu",
                max: 100
            };
        case "ram":
            return {
                icon: "memory_alt",
                text: percent(s.memory.percent),
                fill: s.memory.percent / 100,
                level: ramLevel,
                series: "ram",
                max: 100
            };
        case "temp":
            return {
                icon: "thermostat",
                text: `${Math.round(s.cpu.temp)}°`,
                fill: s.cpu.temp / (thresholds.cpuTemp + 10),
                level: tempLevel,
                series: "temp",
                max: thresholds.cpuTemp + 10
            };
        case "gpu":
            return {
                icon: "developer_board",
                text: percent(s.gpus[0].usage),
                fill: s.gpus[0].usage / 100,
                level: 0,
                series: "gpu",
                max: 100
            };
        case "net":
            return {
                icon: "swap_vert",
                text: rate(s.net.down + s.net.up),
                fill: 0,
                level: 0,
                series: "net",
                max: 0
            };
        case "power":
            return {
                icon: "bolt",
                text: `${s.battery.watts.toFixed(1)} W`,
                fill: s.battery.watts / thresholds.batteryDraw,
                level: 0,
                series: "power",
                max: 0
            };
        }
        return null;
    }

    function percent(value: var): string {
        return value === null || value === undefined ? "–" : `${Math.round(value)}%`;
    }

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

    function duration(seconds: real): string {
        const d = Math.floor(seconds / 86400), h = Math.floor(seconds % 86400 / 3600), m = Math.floor(seconds % 3600 / 60);
        if (d > 0)
            return `${d} d ${h} h`;
        return h > 0 ? `${h} h ${m} min` : `${m} min`;
    }

    function alert(a: var): void {
    }

    function act(action: string): void {
    }

    function status(): var {
        return ({});
    }
}
