import QtQuick
import Quickshell
import KubaOS.Components as K
import qs.services
import qs.settings
import "panel.js" as Panel

// The system monitor's panel, out of the frame's right edge from the bar to the bottom edge, a
// twin of the notifications panel and the control center. A header with the title, a line saying
// how the machine is doing from its worst reading, and btop. Under it, one column: each reading in
// a card of its own (`Readings`), then the busiest processes, which take the room left; on a short
// screen the whole body scrolls. Uptime and the kernel sit behind the info button in the bottom
// right corner. While it is open the engine samples every second, and as it opens the engine sends
// its last detailed sample, so it opens on its last readings. Every line is in place from the
// first frame: one still on its way is a placeholder, and nothing moves as the readings arrive.
K.Sheet {
    id: root

    readonly property int padding: K.Tokens.space.s16
    // The sample shown. It holds still while the panel leaves, so nothing changes shape as it goes:
    // the engine's lighter samples resume the moment it closes.
    property var s: null
    // Whether readings fade in over their placeholders: once the content shows whole. While it is
    // still coming in with the panel, or from a twin's place, the engine's last detailed sample
    // takes their place at once.
    readonly property bool fades: root.contentOpacity >= 1 && root.presence >= 1

    // The colour of a reading's colour step (Sysmon.level).
    function tint(level: int, fine: color): color {
        return level === 2 ? K.Tokens.color.error : level === 1 ? K.Tokens.color.warning : fine;
    }

    edge: "right"
    flushStart: true
    flushEnd: true
    contentWidth: K.Tokens.size.panel
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

    // The header, the container padding in at the sides and 16 above.
    K.Container {
        id: head

        // Over the list's fade, so a tooltip hanging under the header is never faded.
        z: 1
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

                Readings {
                    sample: root.s
                    fades: root.fades
                    within: panelBox
                }

                Processes {
                    id: processes

                    sample: root.s
                    fades: root.fades
                    // The rows take what is left of the room the body shows, above its foot.
                    tallest: scroll.height - processes.y - body.bottomPadding
                }
            }
        }
    }

    K.HeaderFade {
        y: head.height
        width: root.contentWidth
        scrolled: scroll.contentY > 0
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
