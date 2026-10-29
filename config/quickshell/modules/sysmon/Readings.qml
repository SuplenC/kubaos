import QtQuick
import KubaOS.Components as K
import qs.components
import qs.services
import "panel.js" as Panel

// The system monitor's readings, each in a card of its own: CPU, memory, GPUs, network, disks,
// power and fans. A sensor, GPU, battery or fan the machine lacks leaves its line or card out,
// which the lighter samples already say, so every line is in place from the panel's first frame.
// A line only a detailed sample fills (the clock, the address, disk traffic, snapshots, fans) is a
// placeholder until one comes, and fades in over it: nothing moves as the readings arrive.
K.Stack {
    id: root

    // The sample shown.
    property var sample: null
    // Whether what arrives fades in over its placeholder (`Line`, `fades`).
    property bool fades: true
    // The box the tooltips keep inside.
    property Item within: null

    // Whether the sample is detailed: the lines only a detailed sample fills wait until it is.
    readonly property bool detailed: root.sample?.detail === true
    readonly property var fans: root.sample?.fans ?? []
    // How many fans have lines: as many as the sample says there are, before a detailed one lists
    // them.
    readonly property int fanLines: root.detailed ? root.fans.length : (root.sample?.fanCount ?? 0)

    // The colour of a reading's colour step (Sysmon.level).
    function tint(level: int, fine: color): color {
        return level === 2 ? K.Tokens.color.error : level === 1 ? K.Tokens.color.warning : fine;
    }

    fill: true
    gap: K.Tokens.space.s8

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

    Section {
        icon: "memory"
        title: "CPU"
        value: Sysmon.percent(root.sample?.cpu?.usage)
        valueColor: root.tint(Sysmon.tempLevel, K.Tokens.color.on_surface)
        fades: root.fades

        K.Sparkline {
            points: Sysmon.history.cpu ?? []
            revision: Sysmon.revision
            color: root.tint(Sysmon.tempLevel, K.Tokens.color.primary)
        }

        // One bar a core, filled from the bottom, the width split in device pixels. A bar a core
        // shows before the first detailed sample, empty, so the card keeps its height as the cores
        // come in.
        K.Grid {
            id: cores

            readonly property var list: root.sample?.cpu?.cores ?? []
            readonly property int count: Math.max(cores.list.length, root.sample?.cpu?.count ?? 0)

            visible: cores.count > 0
            columns: Math.max(1, cores.count)
            gap: K.Tokens.space.s4

            Repeater {
                model: cores.count

                Rectangle {
                    required property int index
                    readonly property real modelData: cores.list[index] ?? 0
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

        // In the temperature's colour step, the one this card's reading follows. The clock is
        // only in a detailed sample.
        Line {
            waiting: !root.detailed
            fades: root.fades
            text: [root.sample?.cpu?.count ? `${root.sample.cpu.count} cores` : "", root.sample?.cpu?.mhz ? `${(root.sample.cpu.mhz / 1000).toFixed(1)} GHz` : "", root.sample?.cpu?.temp != null ? `${Math.round(root.sample.cpu.temp)} °C` : ""].filter(Boolean).join(" · ")
            color: root.tint(Sysmon.tempLevel, K.Tokens.color.on_surface_variant)
        }
    }

    Section {
        icon: "memory_alt"
        title: "Memory"
        value: Sysmon.percent(root.sample?.memory?.percent)
        valueColor: root.tint(Sysmon.ramLevel, K.Tokens.color.on_surface)
        fades: root.fades

        K.Meter {
            readonly property real total: Math.max(1, root.sample?.memory?.total ?? 1)
            parts: [
                {
                    value: (root.sample?.memory?.used ?? 0) / total,
                    color: root.tint(Sysmon.ramLevel, K.Tokens.color.primary)
                },
                {
                    value: (root.sample?.memory?.cache ?? 0) / total,
                    color: K.Tokens.color.tertiary
                }
            ]
        }

        Line {
            text: root.sample?.memory ? `${Sysmon.bytes(root.sample.memory.used)} used · ${Sysmon.bytes(root.sample.memory.cache)} cache of ${Sysmon.bytes(root.sample.memory.total)}` : "Memory can't be read"
        }

        Line {
            visible: (root.sample?.memory?.swapTotal ?? 0) > 0
            text: root.sample?.memory ? `Swap ${Sysmon.bytes(root.sample.memory.swapUsed)} of ${Sysmon.bytes(root.sample.memory.swapTotal)}` : ""
        }
    }

    Section {
        visible: (root.sample?.gpus ?? []).length > 0
        icon: "developer_board"
        title: "GPU"
        value: root.sample?.gpus?.[0]?.active === false ? "Asleep" : Sysmon.percent(root.sample?.gpus?.[0]?.usage)
        fades: root.fades

        Repeater {
            model: (root.sample?.gpus ?? []).length

            K.Stack {
                id: gpu

                required property int index
                readonly property var modelData: root.sample?.gpus?.[index] ?? ({})

                fill: true
                gap: K.Tokens.space.s4

                // A lone GPU's reading is the card's own.
                Named {
                    name: `${gpu.modelData.vendor}${gpu.modelData.discrete ? " · discrete" : ""}`
                    reading: (root.sample?.gpus ?? []).length < 2 ? "" : !gpu.modelData.active ? "Asleep" : Sysmon.percent(gpu.modelData.usage)
                    fine: gpu.modelData.active ? K.Tokens.color.on_surface : K.Tokens.color.on_surface_variant
                }

                K.Meter {
                    visible: gpu.modelData.active && gpu.modelData.usage != null
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

    Section {
        icon: "swap_vert"
        title: "Network"
        value: root.sample?.net?.down != null ? `↓ ${Sysmon.rate(root.sample.net.down)}  ↑ ${Sysmon.rate(root.sample.net.up ?? 0)}` : ""
        // The speeds need two samples on one interface.
        waiting: root.sample?.net?.interface != null && root.sample?.net?.down == null
        fades: root.fades

        // The address is only in a detailed sample.
        Line {
            waiting: !root.detailed && root.sample?.net?.interface != null
            fades: root.fades
            place: 1
            text: root.sample?.net?.interface ? [root.sample.net.interface, root.sample.net.address ?? ""].filter(Boolean).join(" · ") : "Offline"
        }
    }

    Section {
        icon: "hard_drive"
        title: "Disks"
        value: root.sample?.io ? `R ${Sysmon.rate(root.sample.io.read)}  W ${Sysmon.rate(root.sample.io.write)}` : ""
        // Disk traffic is only in a detailed sample.
        waiting: !root.detailed
        fades: root.fades

        Repeater {
            model: (root.sample?.disks ?? []).length

            K.Stack {
                id: disk

                required property int index
                readonly property var modelData: root.sample?.disks?.[index] ?? ({})
                readonly property int level: Sysmon.diskLevel(modelData)

                fill: true
                gap: K.Tokens.space.s4

                // By its label alone: a mount point beside it leaves its reading no room on the
                // line in a panel's width.
                Named {
                    name: disk.modelData.label
                    reading: `${Sysmon.bytes(disk.modelData.free)} free of ${Sysmon.bytes(disk.modelData.total)}`
                    fine: K.Tokens.color.on_surface_variant
                    level: disk.level
                }

                K.Meter {
                    parts: [
                        {
                            value: disk.modelData.percent / 100,
                            color: root.tint(disk.level, K.Tokens.color.primary)
                        }
                    ]
                }
            }
        }

        // As tall as the info button, whether it shows or not, so the size arriving moves
        // nothing.
        K.Row {
            id: snapshots

            readonly property var figures: root.sample?.snapshots ?? null
            readonly property bool unknown: root.detailed && snapshots.figures !== null && snapshots.figures.bytes == null

            visible: root.sample?.hasSnapshots === true || snapshots.figures !== null
            fill: true
            fixedHeight: K.Tokens.size.control_sm
            gap: K.Tokens.space.s4
            align: "centre"

            Line {
                waiting: !root.detailed
                fades: root.fades
                place: 2
                text: snapshots.figures ? Panel.snapshots(snapshots.figures, bytes => Sysmon.bytes(bytes)) : ""
            }

            K.IconButton {
                id: quotas

                icon: "info"
                label: "Why the size is unknown"
                kind: "Text"
                size: "Sm"
                explained: true
                enabled: snapshots.unknown
                visible: opacity > 0
                opacity: snapshots.unknown ? 1 : 0

                Behavior on opacity {
                    enabled: root.fades

                    NumberAnimation {
                        duration: K.Motion.reduced ? K.Tokens.duration.short : K.Tokens.duration.slow
                        easing.type: Easing.BezierSpline
                        easing.bezierCurve: K.Tokens.easing.standard
                    }
                }

                K.Tooltip {
                    title: "Snapshot size unknown"
                    body: "Btrfs counts the space snapshots hold only with its quotas on, and they are off. sudo btrfs quota enable / turns them on; deleting snapshots then takes longer."
                    within: root.within
                    shown: quotas.pointerOver || quotas.focused
                }
            }
        }
    }

    Section {
        icon: "bolt"
        title: "Power"
        value: root.sample?.battery ? `${Math.round(root.sample.battery.percent)}%` : ""
        fades: root.fades

        K.Meter {
            visible: root.sample?.battery != null
            parts: [
                {
                    value: (root.sample?.battery?.percent ?? 0) / 100,
                    color: (root.sample?.battery?.percent ?? 100) < 15 ? K.Tokens.color.error : K.Tokens.color.primary
                }
            ]
        }

        Line {
            visible: root.sample?.battery != null
            text: {
                const b = root.sample?.battery;
                if (!b)
                    return "";
                const left = b.secondsLeft ? `${Sysmon.duration(b.secondsLeft)} to ${b.status === "Charging" ? "full" : "empty"}` : "";
                return [b.status, b.watts != null ? `${b.watts.toFixed(1)} W` : "", left].filter(Boolean).join(" · ");
            }
            color: root.tint(Sysmon.drawLevel, K.Tokens.color.on_surface_variant)
        }

        Line {
            visible: root.sample?.battery?.health != null
            text: root.sample?.battery?.health != null ? `Health ${Math.round(root.sample.battery.health)}% of its design capacity` : ""
        }

        Line {
            visible: root.sample?.battery == null
            text: "No battery: on mains power"
        }

        PowerSlider {
            size: "Compact"
        }
    }

    Section {
        visible: root.fanLines > 0
        icon: "mode_fan"
        title: "Fans"
        fades: root.fades

        // Their speeds are only in a detailed sample.
        Repeater {
            model: root.fanLines

            Line {
                required property int index
                readonly property var modelData: root.fans[index] ?? null

                place: index
                waiting: !root.detailed
                fades: root.fades
                text: modelData ? `${modelData.label}  ${modelData.rpm} rpm` : ""
            }
        }
    }
}
