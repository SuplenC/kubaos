import QtQuick
import Quickshell
import KubaOS.Components as K
import qs.components
import qs.services
import qs.settings
import "panel.js" as Panel

// The system monitor's panel, out of the frame's right edge from the bar to the bottom edge, a
// twin of the notifications panel and the control center. A header with the title, a line saying
// how the machine is doing from its worst reading, and btop. Under it, one column: each reading in
// a card of its own (CPU, memory, GPUs, network, disks, power, fans), then the busiest processes,
// which take the room left; on a short screen the whole body scrolls. Uptime and the kernel sit
// behind the info button in the bottom right corner. While it is open the engine samples every
// second. A sensor, GPU, battery or fan the machine lacks leaves its line or card out.
K.Sheet {
    id: root

    readonly property int padding: K.Tokens.space.s16
    // The sample shown. It holds still while the panel leaves, so nothing changes shape as it goes:
    // the engine's lighter samples resume the moment it closes.
    property var s: null
    // The last detailed sample, whose fans show until this opening's first detailed one arrives:
    // the lighter samples leave fans out, and their card would push everything under it down.
    property var lastDetailed: null
    readonly property var th: Sysmon.thresholds
    readonly property var fans: root.s?.detail ? (root.s.fans ?? []) : (root.lastDetailed?.fans ?? [])

    // The colour of a reading's colour step (Sysmon.level).
    function tint(level: int, fine: color): color {
        return level === 2 ? K.Tokens.color.error : level === 1 ? K.Tokens.color.warning : fine;
    }

    edge: "right"
    flushStart: true
    flushEnd: true
    contentWidth: K.Tokens.size.panel
    onSChanged: {
        if (root.s?.detail)
            root.lastDetailed = root.s;
    }
    onOpened: {
        scroll.contentY = 0;
        processes.clear();
        if (Sysmon.pendingPid > 0)
            processes.show(Sysmon.pendingPid);
        Sysmon.pendingPid = 0;
    }

    Binding {
        target: root
        property: "s"
        value: Sysmon.sample
        when: root.shown
        restoreMode: Binding.RestoreNone
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

    // The header, the container padding in at the sides and 16 above.
    K.Container {
        id: head

        width: root.contentWidth

        K.Stack {
            fill: true
            topPadding: K.Tokens.space.s16

            K.Row {
                fill: true
                gap: K.Tokens.space.s8
                align: "centre"

                K.Stack {
                    fill: true

                    K.Text {
                        text: "System"
                        style: K.Tokens.type.heading
                        maximumLines: 1
                    }

                    // How the machine is doing, from its worst reading, in that reading's colour.
                    K.Text {
                        text: Panel.mood(Sysmon.worstLevel)
                        style: K.Tokens.type.small
                        color: root.tint(Sysmon.worstLevel, K.Tokens.color.on_surface_variant)
                        maximumLines: 1
                    }
                }

                K.IconButton {
                    id: btop

                    icon: "terminal"
                    label: "Open btop"
                    kind: "Text"
                    size: "Sm"
                    explained: true
                    onActivated: {
                        Panels.close();
                        Quickshell.execDetached(["ghostty", "--class=dot.float", "-e", "btop"]);
                    }

                    // Below the button: the header leaves no room above it.
                    K.Tooltip {
                        title: "Open btop"
                        body: "Every process and sensor, in a terminal."
                        within: panelBox
                        shown: btop.pointerOver || btop.focused
                    }
                }
            }
        }
    }

    // The body, under the header. It reaches 16 up into the gap under the header and down to the
    // panel's edge, so what scrolls past shows there.
    K.Scroll {
        id: scroll

        y: head.height + root.padding
        width: root.contentWidth
        height: Math.max(0, root.height - y - root.padding)
        roomAbove: root.padding
        roomBelow: root.padding

        K.Container {
            width: scroll.width

            K.Stack {
                id: body

                fill: true
                gap: K.Tokens.space.s16
                // Room to scroll the last row clear of the info button.
                bottomPadding: K.Tokens.size.control_sm + root.padding

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

                K.Stack {
                    fill: true
                    gap: K.Tokens.space.s8

                    Section {
                        icon: "memory"
                        title: "CPU"
                        value: Sysmon.percent(root.s?.cpu?.usage)
                        valueColor: root.tint(Sysmon.tempLevel, K.Tokens.color.on_surface)

                        K.Sparkline {
                            points: Sysmon.history.cpu ?? []
                            revision: Sysmon.revision
                            color: root.tint(Sysmon.tempLevel, K.Tokens.color.primary)
                        }

                        // One bar a core, filled from the bottom, the width split in device
                        // pixels. A bar a core shows before the first detailed sample, empty, so
                        // the card keeps its height as the cores come in.
                        K.Grid {
                            id: cores

                            readonly property var list: root.s?.cpu?.cores ?? []
                            readonly property int count: Math.max(cores.list.length, root.s?.cpu?.count ?? 0)

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

                        // In the temperature's colour step, the one this card's reading follows.
                        Line {
                            text: [root.s?.cpu?.count ? `${root.s.cpu.count} cores` : "", root.s?.cpu?.mhz ? `${(root.s.cpu.mhz / 1000).toFixed(1)} GHz` : "", root.s?.cpu?.temp != null ? `${Math.round(root.s.cpu.temp)} °C` : ""].filter(Boolean).join(" · ")
                            color: root.tint(Sysmon.tempLevel, K.Tokens.color.on_surface_variant)
                        }
                    }

                    Section {
                        icon: "memory_alt"
                        title: "Memory"
                        value: Sysmon.percent(root.s?.memory?.percent)
                        valueColor: root.tint(Sysmon.ramLevel, K.Tokens.color.on_surface)

                        K.Meter {
                            readonly property real total: Math.max(1, root.s?.memory?.total ?? 1)
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
                            text: root.s?.memory ? `${Sysmon.bytes(root.s.memory.used)} used · ${Sysmon.bytes(root.s.memory.cache)} cache of ${Sysmon.bytes(root.s.memory.total)}` : "Memory can't be read"
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
                        value: root.s?.gpus?.[0]?.active === false ? "Asleep" : Sysmon.percent(root.s?.gpus?.[0]?.usage)

                        Repeater {
                            model: (root.s?.gpus ?? []).length

                            K.Stack {
                                id: gpu

                                required property int index
                                readonly property var modelData: root.s?.gpus?.[index] ?? ({})

                                fill: true
                                gap: K.Tokens.space.s4

                                // A lone GPU's reading is the card's own.
                                Named {
                                    name: `${gpu.modelData.vendor}${gpu.modelData.discrete ? " · discrete" : ""}`
                                    reading: (root.s?.gpus ?? []).length < 2 ? "" : !gpu.modelData.active ? "Asleep" : Sysmon.percent(gpu.modelData.usage)
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
                        value: root.s?.net?.down != null ? `↓ ${Sysmon.rate(root.s.net.down)}  ↑ ${Sysmon.rate(root.s.net.up ?? 0)}` : ""

                        Line {
                            text: root.s?.net?.interface ? [root.s.net.interface, root.s.net.address ?? ""].filter(Boolean).join(" · ") : "Offline"
                        }
                    }

                    Section {
                        icon: "hard_drive"
                        title: "Disks"
                        value: root.s?.io ? `R ${Sysmon.rate(root.s.io.read)}  W ${Sysmon.rate(root.s.io.write)}` : ""

                        Repeater {
                            model: (root.s?.disks ?? []).length

                            K.Stack {
                                id: disk

                                required property int index
                                readonly property var modelData: root.s?.disks?.[index] ?? ({})
                                readonly property int level: Sysmon.diskLevel(modelData)

                                fill: true
                                gap: K.Tokens.space.s4

                                // By its label alone: a mount point beside it leaves its reading no
                                // room on the line in a panel's width.
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

                        K.Meter {
                            visible: root.s?.battery != null
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
                            color: root.tint(Sysmon.drawLevel, K.Tokens.color.on_surface_variant)
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
                            size: "Compact"
                        }
                    }

                    Section {
                        visible: root.fans.length > 0
                        icon: "mode_fan"
                        title: "Fans"

                        Repeater {
                            model: root.fans.length

                            Line {
                                required property int index
                                readonly property var modelData: root.fans[index] ?? ({})

                                text: `${modelData.label}  ${modelData.rpm} rpm`
                            }
                        }
                    }
                }

                Processes {
                    id: processes

                    sample: root.s
                    // The rows take what is left of the room the body shows, above its foot.
                    tallest: scroll.height - processes.y - body.bottomPadding
                }
            }
        }
    }

    // The panel's own box, which its tooltips keep inside.
    Item {
        id: panelBox

        anchors.fill: parent
    }

    // Uptime, the kernel and the load, behind a button floating in the bottom right corner over
    // the body.
    K.IconButton {
        id: about

        x: root.contentWidth - root.padding - width
        y: root.height - root.padding - height
        icon: "info"
        label: "Uptime and kernel"
        kind: "Tonal"
        size: "Sm"
        explained: true
        activeFocusOnTab: false

        K.Tooltip {
            title: root.s ? `Up ${Sysmon.duration(root.s.uptime)}` : "Uptime"
            body: root.s ? `Linux ${root.s.kernel}` : "Waiting for the monitor's first reading."
            within: panelBox
            shown: about.pointerOver

            // What waits for the CPU, over 1, 5 and 15 minutes.
            Line {
                visible: root.s?.cpu?.load != null
                text: root.s?.cpu?.load ? `Load ${root.s.cpu.load.map(l => l.toFixed(2)).join("  ")}` : ""
            }
        }
    }
}
