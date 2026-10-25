import QtQuick
import Quickshell
import qs.theme
import qs.components
import qs.services
import qs.modules.clock
import qs.settings

// The system monitor's popup, under the bar's pill: CPU, memory and GPUs on
// the left; network, disks, power and fans on the right; the busiest
// processes below, and uptime, kernel and btop at the foot. While it is open
// the engine samples every second. A sensor, GPU, battery or fan the machine
// lacks leaves its line or part out.
//
// It glides in as the clock popup does (Kōbō's 220 ms on emphasized
// decelerate).
Drawer {
    id: root

    readonly property int glide: 220
    readonly property list<real> arrive: Theme.anim.emphasizedDecel
    readonly property int pad: Theme.spacing.xl
    readonly property int gap: Theme.spacing.xxl
    readonly property int column: 340
    readonly property int graphHeight: 52
    readonly property int coreHeight: 22
    readonly property int buttonSize: 30
    readonly property var s: Sysmon.sample
    readonly property var th: Sysmon.thresholds

    // The colour of a reading's colour step (Sysmon.level).
    function tint(level: int, fine: color): color {
        return level === 2 ? Theme.m3error : level === 1 ? Theme.warning : fine;
    }

    edge: "top"
    align: 1
    enterDuration: glide
    enterCurve: arrive
    exitDuration: 120
    contentWidth: column * 2 + gap + pad * 2
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

    component Line: StyledText {
        width: parent?.width ?? 0
        font.pixelSize: Theme.font.sm
        font.features: Theme.font.tabular
        color: Theme.m3onSurfaceVariant
    }

    Column {
        id: body

        x: root.pad
        y: root.pad
        width: root.contentWidth - root.pad * 2
        spacing: Theme.spacing.xl

        Row {
            spacing: root.gap

            Column {
                width: root.column
                spacing: Theme.spacing.xl

                Section {
                    width: parent.width
                    icon: "memory"
                    title: "CPU"
                    value: Sysmon.percent(root.s?.cpu?.usage)
                    valueColor: root.tint(Sysmon.tempLevel, Theme.m3onSurface)

                    Graph {
                        width: parent.width
                        height: root.graphHeight
                        points: Sysmon.history.cpu ?? []
                        revision: Sysmon.revision
                        color: root.tint(Sysmon.tempLevel, Theme.m3primary)
                    }

                    // One bar a core, filled from the bottom.
                    Row {
                        id: cores

                        readonly property var list: root.s?.cpu?.cores ?? []

                        visible: list.length > 0
                        width: parent.width
                        height: root.coreHeight
                        spacing: Theme.spacing.xxs

                        Repeater {
                            model: cores.list

                            Rectangle {
                                required property real modelData

                                width: (cores.width - cores.spacing * (cores.list.length - 1)) / cores.list.length
                                height: cores.height
                                radius: Theme.rounding.xs / 2
                                color: Theme.m3surfaceContainerHighest

                                Rectangle {
                                    anchors.bottom: parent.bottom
                                    width: parent.width
                                    height: parent.height * Math.min(1, parent.modelData / 100)
                                    radius: parent.radius
                                    color: Theme.m3primary

                                    Behavior on height {
                                        Anim {
                                            duration: Theme.anim.normal
                                            easing.bezierCurve: Theme.anim.standard
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
                        color: root.tint(Sysmon.tempLevel, Theme.m3onSurfaceVariant)
                    }
                }

                Section {
                    width: parent.width
                    icon: "memory_alt"
                    title: "Memory"
                    value: Sysmon.percent(root.s?.memory?.percent)
                    valueColor: root.tint(Sysmon.ramLevel, Theme.m3onSurface)

                    Meter {
                        readonly property real total: Math.max(1, root.s?.memory?.total ?? 1)

                        width: parent.width
                        parts: [
                            {
                                value: (root.s?.memory?.used ?? 0) / total,
                                color: root.tint(Sysmon.ramLevel, Theme.m3primary)
                            },
                            {
                                value: (root.s?.memory?.cache ?? 0) / total,
                                color: Theme.m3tertiary
                            }
                        ]
                    }

                    Line {
                        text: root.s?.memory ? `${Sysmon.bytes(root.s.memory.used)} used · ${Sysmon.bytes(root.s.memory.cache)} cache · ${Sysmon.bytes(root.s.memory.available)} available of ${Sysmon.bytes(root.s.memory.total)}` : "Memory can't be read"
                    }

                    Graph {
                        width: parent.width
                        height: root.graphHeight
                        points: Sysmon.history.ram ?? []
                        revision: Sysmon.revision
                        color: root.tint(Sysmon.ramLevel, Theme.m3primary)
                    }

                    Meter {
                        visible: (root.s?.memory?.swapTotal ?? 0) > 0
                        width: parent.width
                        parts: [
                            {
                                value: (root.s?.memory?.swapUsed ?? 0) / Math.max(1, root.s?.memory?.swapTotal ?? 1),
                                color: Theme.m3secondary
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
                    width: parent.width
                    icon: "developer_board"
                    title: "GPU"
                    value: Sysmon.percent(root.s?.gpus?.[0]?.usage)

                    Repeater {
                        model: root.s?.gpus ?? []

                        Column {
                            id: gpu

                            required property var modelData

                            width: parent.width
                            spacing: Theme.spacing.xs

                            Item {
                                width: parent.width
                                height: gpuName.implicitHeight

                                StyledText {
                                    id: gpuName

                                    text: `${gpu.modelData.vendor}${gpu.modelData.discrete ? " · discrete" : ""}`
                                    font.weight: Font.Medium
                                    font.pixelSize: Theme.font.sm
                                }

                                StyledText {
                                    anchors.right: parent.right
                                    text: !gpu.modelData.active ? "Asleep" : Sysmon.percent(gpu.modelData.usage)
                                    font.pixelSize: Theme.font.sm
                                    font.features: Theme.font.tabular
                                    color: gpu.modelData.active ? Theme.m3onSurface : Theme.m3outline
                                }
                            }

                            Meter {
                                visible: gpu.modelData.active && gpu.modelData.usage != null
                                width: parent.width
                                parts: [
                                    {
                                        value: (gpu.modelData.usage ?? 0) / 100,
                                        color: Theme.m3primary
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

            Column {
                width: root.column
                spacing: Theme.spacing.xl

                Section {
                    width: parent.width
                    icon: "swap_vert"
                    title: "Network"
                    value: root.s?.net?.down != null ? `↓ ${Sysmon.rate(root.s.net.down)}  ↑ ${Sysmon.rate(root.s.net.up ?? 0)}` : ""

                    Graph {
                        width: parent.width
                        height: root.graphHeight
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
                    width: parent.width
                    icon: "hard_drive"
                    title: "Disks"
                    value: root.s?.io ? `R ${Sysmon.rate(root.s.io.read)}  W ${Sysmon.rate(root.s.io.write)}` : ""

                    Repeater {
                        model: root.s?.disks ?? []

                        Column {
                            id: disk

                            required property var modelData
                            readonly property int level: Sysmon.level(modelData.percent, root.th.disk, 5)

                            width: parent.width
                            spacing: Theme.spacing.xs

                            Item {
                                width: parent.width
                                height: diskName.implicitHeight

                                StyledText {
                                    id: diskName

                                    width: parent.width - diskFree.implicitWidth - Theme.spacing.md
                                    // The root and home disks are named by their mount point already.
                                    text: disk.modelData.label === disk.modelData.point ? disk.modelData.label : `${disk.modelData.label}  ${disk.modelData.point}`
                                    font.weight: Font.Medium
                                    font.pixelSize: Theme.font.sm
                                }

                                StyledText {
                                    id: diskFree

                                    anchors.right: parent.right
                                    text: `${Sysmon.bytes(disk.modelData.free)} free of ${Sysmon.bytes(disk.modelData.total)}`
                                    font.pixelSize: Theme.font.sm
                                    font.features: Theme.font.tabular
                                    color: root.tint(disk.level, Theme.m3onSurfaceVariant)
                                }
                            }

                            Meter {
                                width: parent.width
                                parts: [
                                    {
                                        value: disk.modelData.percent / 100,
                                        color: root.tint(disk.level, Theme.m3primary)
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
                    width: parent.width
                    icon: "bolt"
                    title: "Power"
                    value: root.s?.battery ? `${Math.round(root.s.battery.percent)}%` : ""

                    Meter {
                        visible: root.s?.battery != null
                        width: parent.width
                        parts: [
                            {
                                value: (root.s?.battery?.percent ?? 0) / 100,
                                color: (root.s?.battery?.percent ?? 100) < 15 ? Theme.m3error : Theme.m3primary
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
                        color: root.tint(Sysmon.level(root.s?.battery?.watts, root.th.batteryDraw, 5), Theme.m3onSurfaceVariant)
                    }

                    Line {
                        visible: root.s?.battery?.health != null
                        text: root.s?.battery?.health != null ? `Health ${Math.round(root.s.battery.health)}% of its design capacity` : ""
                    }

                    Line {
                        visible: root.s?.battery == null
                        text: "No battery: on mains power"
                    }

                    Row {
                        spacing: Theme.spacing.xs

                        Repeater {
                            model: Profiles.available

                            Chip {
                                required property int modelData

                                label: Profiles.nameFor(modelData)
                                selected: Profiles.current === modelData
                                onClicked: Profiles.set(modelData)
                            }
                        }
                    }
                }

                Section {
                    visible: (root.s?.fans ?? []).length > 0
                    width: parent.width
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

            width: parent.width
        }

        // Uptime, the kernel, and btop for the full picture.
        Item {
            width: parent.width
            height: btop.height

            Line {
                anchors.verticalCenter: parent.verticalCenter
                width: parent.width - btop.width - Theme.spacing.md
                text: root.s ? `Up ${Sysmon.duration(root.s.uptime)} · Linux ${root.s.kernel}` : ""
            }

            IconButton {
                id: btop

                anchors.right: parent.right
                icon: "terminal"
                size: root.buttonSize
                baseColor: Theme.m3surfaceContainer
                tooltip: "Open btop"
                onClicked: {
                    Panels.close();
                    Quickshell.execDetached(["ghostty", "-e", "btop"]);
                }
            }
        }

        StyledText {
            visible: !Sysmon.available
            width: parent.width
            text: "kubaos-sysmon isn't running: the monitor needs it"
            color: Theme.m3error
            font.pixelSize: Theme.font.sm
        }

        StyledText {
            visible: Sysmon.problem !== ""
            width: parent.width
            text: `${UserLayer.shown("sysmon", "sysmon.toml")}: ${Sysmon.problem}`
            color: Theme.m3error
            font.pixelSize: Theme.font.sm
            wrapMode: Text.Wrap
        }
    }
}
