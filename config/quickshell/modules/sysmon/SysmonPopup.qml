import QtQuick
import Quickshell
import KubaOS.Components as K
import qs.components
import qs.services
import qs.settings

// The system monitor's popup, under the bar's pill: CPU, memory and GPUs on
// the left; network, disks, power and fans on the right; the busiest
// processes below, and uptime, kernel and btop at the foot. While it is open
// the engine samples every second. A sensor, GPU, battery or fan the machine
// lacks leaves its line or part out. It is two panels wide.
Drawer {
    id: root

    readonly property int pad: K.Tokens.space.s24
    readonly property var s: Sysmon.sample
    readonly property var th: Sysmon.thresholds

    // The colour of a reading's colour step (Sysmon.level).
    function tint(level: int, fine: color): color {
        return level === 2 ? K.Tokens.color.error : level === 1 ? K.Tokens.color.warning : fine;
    }

    edge: "top"
    align: 1
    contentWidth: K.Tokens.size.panel * 2
    contentHeight: body.implicitHeight + pad * 2

    onOpened: {
        processes.clear();
        if (Sysmon.pendingPid > 0)
            processes.show(Sysmon.pendingPid);
        Sysmon.pendingPid = 0;
    }

    Connections {
        target: Sysmon

        function onProcessRequested(pid: int): void {
            if (root.shown) {
                processes.show(pid);
                Sysmon.pendingPid = 0;
            }
        }
    }

    // A line of readings under a section's headline. In mono: the numbers in it tick, and their
    // widths hold still.
    component Line: K.Text {
        style: K.Tokens.type.mono
        color: K.Tokens.color.on_surface_variant
    }

    // A thing's name with its reading at the end of the row, the reading in its colour step.
    component Named: K.Row {
        id: named

        property string name
        property string reading
        property color fine: K.Tokens.color.on_surface
        property int level: 0
        readonly property color readingColor: named.level === 2 ? K.Tokens.color.error : named.level === 1 ? K.Tokens.color.warning : named.fine

        fill: true
        gap: K.Tokens.space.s12
        align: "centre"

        K.Text {
            text: named.name
            style: K.Tokens.type.label
            maximumLines: 1
        }

        K.Fill {
        }

        K.Text {
            text: named.reading
            style: K.Tokens.type.mono
            color: named.readingColor
        }
    }

    K.Stack {
        id: body

        x: root.pad
        y: root.pad
        width: root.contentWidth - root.pad * 2
        room: width
        fill: true
        gap: K.Tokens.space.s20

        K.Grid {
            columns: 2
            gap: K.Tokens.space.s20

            K.Stack {
                fill: true
                gap: K.Tokens.space.s20

                Section {
                    icon: "memory"
                    title: "CPU"
                    value: Sysmon.percent(root.s?.cpu?.usage)
                    valueColor: root.tint(Sysmon.tempLevel, K.Tokens.color.on_surface)

                    Graph {
                        fillWidth: true
                        implicitHeight: K.Tokens.space.s48
                        points: Sysmon.history.cpu ?? []
                        revision: Sysmon.revision
                        color: root.tint(Sysmon.tempLevel, K.Tokens.color.primary)
                    }

                    // One bar a core, filled from the bottom, the width split in device pixels.
                    K.Grid {
                        id: cores

                        readonly property var list: root.s?.cpu?.cores ?? []

                        visible: list.length > 0
                        columns: Math.max(1, list.length)
                        gap: K.Tokens.space.s4

                        Repeater {
                            model: cores.list

                            Rectangle {
                                required property real modelData
                                readonly property bool fillWidth: true
                                // How busy the core is, 0 to 1, from its percent.
                                readonly property real busy: Math.min(1, modelData / 100)

                                implicitHeight: K.Tokens.space.s24
                                radius: K.Tokens.radius.small
                                color: K.Look.alpha(K.Tokens.color.surface_container_highest, K.Tokens.opacity.track)

                                Rectangle {
                                    anchors.bottom: parent.bottom
                                    width: parent.width
                                    height: K.Look.snapUp(parent.height * parent.busy, cores.dpr)
                                    radius: parent.radius
                                    color: K.Tokens.color.primary

                                    Behavior on height {
                                        NumberAnimation {
                                            duration: K.Tokens.duration.short
                                            easing.type: Easing.BezierSpline
                                            easing.bezierCurve: K.Tokens.easing.linear
                                        }
                                    }
                                }
                            }
                        }
                    }

                    Line {
                        text: [root.s?.cpu?.count ? `${root.s.cpu.count} cores` : "", root.s?.cpu?.mhz ? `${(root.s.cpu.mhz / 1000).toFixed(1)} GHz` : "", root.s?.cpu?.load ? `load ${root.s.cpu.load.map(l => l.toFixed(2)).join(" ")}` : ""].filter(Boolean).join(" · ")
                    }

                    Line {
                        visible: root.s?.cpu?.temp != null
                        text: root.s?.cpu?.temp != null ? `${Math.round(root.s.cpu.temp)} °C` : ""
                        color: root.tint(Sysmon.tempLevel, K.Tokens.color.on_surface_variant)
                    }
                }

                Section {
                    icon: "memory_alt"
                    title: "Memory"
                    value: Sysmon.percent(root.s?.memory?.percent)
                    valueColor: root.tint(Sysmon.ramLevel, K.Tokens.color.on_surface)

                    Meter {
                        readonly property real total: Math.max(1, root.s?.memory?.total ?? 1)

                        fillWidth: true
                        parts: [
                            {
                                value: (root.s?.memory?.used ?? 0) / total,
                                color: root.tint(Sysmon.ramLevel, K.Tokens.color.primary)
                            },
                            {
                                value: (root.s?.memory?.cache ?? 0) / total,
                                color: K.Tokens.color.tertiary
                            }
                        ]
                    }

                    Line {
                        text: root.s?.memory ? `${Sysmon.bytes(root.s.memory.used)} used · ${Sysmon.bytes(root.s.memory.cache)} cache · ${Sysmon.bytes(root.s.memory.available)} available of ${Sysmon.bytes(root.s.memory.total)}` : "Memory can't be read"
                    }

                    Graph {
                        fillWidth: true
                        implicitHeight: K.Tokens.space.s48
                        points: Sysmon.history.ram ?? []
                        revision: Sysmon.revision
                        color: root.tint(Sysmon.ramLevel, K.Tokens.color.primary)
                    }

                    Meter {
                        visible: (root.s?.memory?.swapTotal ?? 0) > 0
                        fillWidth: true
                        parts: [
                            {
                                value: (root.s?.memory?.swapUsed ?? 0) / Math.max(1, root.s?.memory?.swapTotal ?? 1),
                                color: K.Tokens.color.secondary
                            }
                        ]
                    }

                    Line {
                        visible: (root.s?.memory?.swapTotal ?? 0) > 0
                        text: root.s?.memory ? `Swap ${Sysmon.bytes(root.s.memory.swapUsed)} of ${Sysmon.bytes(root.s.memory.swapTotal)}` : ""
                    }
                }

                Section {
                    visible: (root.s?.gpus ?? []).length > 0
                    icon: "developer_board"
                    title: "GPU"
                    value: Sysmon.percent(root.s?.gpus?.[0]?.usage)

                    Repeater {
                        model: root.s?.gpus ?? []

                        K.Stack {
                            id: gpu

                            required property var modelData

                            fill: true
                            gap: K.Tokens.space.s4

                            Named {
                                name: `${gpu.modelData.vendor}${gpu.modelData.discrete ? " · discrete" : ""}`
                                reading: !gpu.modelData.active ? "Asleep" : Sysmon.percent(gpu.modelData.usage)
                                fine: gpu.modelData.active ? K.Tokens.color.on_surface : K.Tokens.color.outline
                            }

                            Meter {
                                visible: gpu.modelData.active && gpu.modelData.usage != null
                                fillWidth: true
                                parts: [
                                    {
                                        value: (gpu.modelData.usage ?? 0) / 100,
                                        color: K.Tokens.color.primary
                                    }
                                ]
                            }

                            Line {
                                visible: text !== ""
                                text: [gpu.modelData.vramTotal ? `VRAM ${Sysmon.bytes(gpu.modelData.vramUsed ?? 0)} of ${Sysmon.bytes(gpu.modelData.vramTotal)}` : "", gpu.modelData.temp != null ? `${Math.round(gpu.modelData.temp)} °C` : "", gpu.modelData.mhz ? `${gpu.modelData.mhz} MHz` : ""].filter(Boolean).join(" · ")
                            }
                        }
                    }
                }
            }

            K.Stack {
                fill: true
                gap: K.Tokens.space.s20

                Section {
                    icon: "swap_vert"
                    title: "Network"
                    value: root.s?.net?.down != null ? `↓ ${Sysmon.rate(root.s.net.down)}  ↑ ${Sysmon.rate(root.s.net.up ?? 0)}` : ""

                    Graph {
                        fillWidth: true
                        implicitHeight: K.Tokens.space.s48
                        points: Sysmon.history.down ?? []
                        second: Sysmon.history.up ?? []
                        revision: Sysmon.revision
                        max: 0
                    }

                    Line {
                        text: root.s?.net?.interface ? [root.s.net.interface, root.s.net.address ?? ""].filter(Boolean).join(" · ") : "Offline"
                    }
                }

                Section {
                    icon: "hard_drive"
                    title: "Disks"
                    value: root.s?.io ? `R ${Sysmon.rate(root.s.io.read)}  W ${Sysmon.rate(root.s.io.write)}` : ""

                    Repeater {
                        model: root.s?.disks ?? []

                        K.Stack {
                            id: disk

                            required property var modelData
                            readonly property int level: Sysmon.level(modelData.percent, root.th.disk, 5)

                            fill: true
                            gap: K.Tokens.space.s4

                            Named {
                                // The root and home disks are named by their mount point already.
                                name: disk.modelData.label === disk.modelData.point ? disk.modelData.label : `${disk.modelData.label}  ${disk.modelData.point}`
                                reading: `${Sysmon.bytes(disk.modelData.free)} free of ${Sysmon.bytes(disk.modelData.total)}`
                                fine: K.Tokens.color.on_surface_variant
                                level: disk.level
                            }

                            Meter {
                                fillWidth: true
                                parts: [
                                    {
                                        value: disk.modelData.percent / 100,
                                        color: root.tint(disk.level, K.Tokens.color.primary)
                                    }
                                ]
                            }
                        }
                    }

                    Line {
                        visible: root.s?.snapshots != null
                        text: {
                            const snaps = root.s?.snapshots;
                            if (!snaps)
                                return "";
                            const count = snaps.count != null ? `${snaps.count} snapshots` : "Snapshots";
                            return snaps.bytes != null ? `${count} · ${Sysmon.bytes(snaps.bytes)}` : `${count} · size unknown without Btrfs quotas`;
                        }
                    }
                }

                Section {
                    icon: "bolt"
                    title: "Power"
                    value: root.s?.battery ? `${Math.round(root.s.battery.percent)}%` : ""

                    Meter {
                        visible: root.s?.battery != null
                        fillWidth: true
                        parts: [
                            {
                                value: (root.s?.battery?.percent ?? 0) / 100,
                                color: (root.s?.battery?.percent ?? 100) < 15 ? K.Tokens.color.error : K.Tokens.color.primary
                            }
                        ]
                    }

                    Line {
                        visible: root.s?.battery != null
                        text: {
                            const b = root.s?.battery;
                            if (!b)
                                return "";
                            const left = b.secondsLeft ? `${Sysmon.duration(b.secondsLeft)} to ${b.status === "Charging" ? "full" : "empty"}` : "";
                            return [b.status, b.watts != null ? `${b.watts.toFixed(1)} W` : "", left].filter(Boolean).join(" · ");
                        }
                        color: root.tint(Sysmon.level(root.s?.battery?.watts, root.th.batteryDraw, 5), K.Tokens.color.on_surface_variant)
                    }

                    Line {
                        visible: root.s?.battery?.health != null
                        text: root.s?.battery?.health != null ? `Health ${Math.round(root.s.battery.health)}% of its design capacity` : ""
                    }

                    Line {
                        visible: root.s?.battery == null
                        text: "No battery: on mains power"
                    }

                    PowerSlider {
                    }
                }

                Section {
                    visible: (root.s?.fans ?? []).length > 0
                    icon: "mode_fan"
                    title: "Fans"

                    Repeater {
                        model: root.s?.fans ?? []

                        Line {
                            required property var modelData

                            text: `${modelData.label}  ${modelData.rpm} rpm`
                        }
                    }
                }
            }
        }

        Processes {
            id: processes
        }

        // Uptime, the kernel, and btop for the full picture.
        K.Row {
            fill: true
            gap: K.Tokens.space.s12
            align: "centre"

            Line {
                text: root.s ? `Up ${Sysmon.duration(root.s.uptime)} · Linux ${root.s.kernel}` : ""
            }

            K.Fill {
            }

            K.IconButton {
                icon: "terminal"
                label: "Open btop"
                kind: "Tonal"
                size: "Sm"
                onActivated: {
                    Panels.close();
                    Quickshell.execDetached(["ghostty", "--class=dot.float", "-e", "btop"]);
                }
            }
        }

        K.Text {
            visible: !Sysmon.available
            text: "kubaos-sysmon isn't running: the monitor needs it"
            style: K.Tokens.type.small
            color: K.Tokens.color.error
        }

        K.Text {
            visible: Sysmon.problem !== ""
            text: `${UserLayer.shown("sysmon", "sysmon.toml")}: ${Sysmon.problem}`
            style: K.Tokens.type.small
            color: K.Tokens.color.error
        }
    }
}
