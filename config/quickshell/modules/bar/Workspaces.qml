import QtQuick
import Quickshell
import Quickshell.Hyprland
import qs.theme
import qs.settings
import qs.components
import qs.services

// This screen's workspaces, numbered 1-10 as SUPER + 1-0 reach them: its
// first always, then those of its own that exist, then the guests, which are
// workspaces of an unplugged screen that found no room among this one's
// (config/hypr/workspaces.lua). Scroll to step through them.
//
// The active indicator stretches toward the new workspace: its leading edge
// moves fast, the trailing edge follows a little later.
Pill {
    id: root

    required property ShellScreen screen
    readonly property HyprlandMonitor monitor: Hyprland.monitorFor(screen)
    readonly property int block: Hypr.blockOfMonitor(monitor)
    readonly property int activeId: monitor?.activeWorkspace?.id ?? 0
    readonly property var ids: {
        const own = [block * Hypr.slots + 1];
        const guests = [];
        for (const workspace of Hyprland.workspaces.values) {
            const id = workspace.id;
            if (id <= 0)
                continue;
            if (Hypr.blockOf(id) === block) {
                if (!own.includes(id))
                    own.push(id);
            } else if (workspace.monitor?.name === monitor?.name) {
                guests.push(id);
            }
        }
        own.sort((a, b) => a - b);
        guests.sort((a, b) => a - b);
        return own.concat(guests);
    }
    readonly property int slot: Math.max(0, ids.indexOf(activeId))
    readonly property int cell: Theme.bar.pillHeight - 6
    readonly property var kanji: ["一", "二", "三", "四", "五", "六", "七", "八", "九", "十"]

    padding: 3
    spacing: 0
    interactive: true
    onScrolled: steps => Hypr.focusOn(monitor, ids[(slot - steps + ids.length) % ids.length])

    Item {
        implicitWidth: root.ids.length * root.cell
        implicitHeight: root.cell

        Rectangle {
            id: indicator

            // Plain values, set in onSlotChanged, so the durations are chosen
            // before the Behaviors start.
            property real start: 0
            property real end: root.cell
            property bool ready: false
            property int lead: Theme.anim.normal
            property int trail: Theme.anim.large

            Component.onCompleted: {
                start = root.slot * root.cell;
                end = start + root.cell;
                ready = true;
            }

            x: start
            width: end - start
            height: root.cell
            radius: height / 2
            color: Theme.m3primary

            Connections {
                target: root

                function onSlotChanged() {
                    const right = root.slot * root.cell > indicator.start;
                    startAnim.duration = right ? indicator.trail : indicator.lead;
                    endAnim.duration = right ? indicator.lead : indicator.trail;
                    indicator.start = root.slot * root.cell;
                    indicator.end = indicator.start + root.cell;
                }
            }

            Behavior on start {
                enabled: indicator.ready

                Anim {
                    id: startAnim

                    easing.bezierCurve: Theme.anim.emphasizedDecel
                }
            }

            Behavior on end {
                enabled: indicator.ready

                Anim {
                    id: endAnim

                    easing.bezierCurve: Theme.anim.emphasizedDecel
                }
            }
        }

        Row {
            Repeater {
                model: root.ids

                delegate: Item {
                    id: cell

                    required property int modelData
                    readonly property int workspaceId: modelData
                    readonly property int number: Hypr.slotOf(workspaceId)
                    readonly property bool isActive: workspaceId === root.activeId
                    readonly property bool occupied: Hyprland.workspaces.values.some(w => w.id === workspaceId && w.toplevels.values.length > 0)

                    width: root.cell
                    height: root.cell

                    Text {
                        anchors.centerIn: parent
                        text: Settings.workspaceKanji ? root.kanji[cell.number - 1] : cell.number
                        font.family: Settings.workspaceKanji ? Theme.font.jp : Theme.font.sans
                        font.pixelSize: Theme.font.sm + 1
                        font.weight: cell.isActive || cell.occupied ? Font.DemiBold : Font.Medium
                        font.features: Theme.font.tabular
                        renderType: Text.NativeRendering
                        color: cell.isActive ? Theme.m3onPrimary : cell.occupied ? Theme.m3onSurface : Theme.m3outline

                        Behavior on color {
                            CAnim {}
                        }
                    }

                    // Occupied but not active: a small dot under the number.
                    Rectangle {
                        anchors.horizontalCenter: parent.horizontalCenter
                        anchors.bottom: parent.bottom
                        anchors.bottomMargin: 2
                        width: 3
                        height: 3
                        radius: 1.5
                        color: Theme.m3tertiary
                        opacity: cell.occupied && !cell.isActive ? 1 : 0

                        Behavior on opacity {
                            Anim {
                                duration: Theme.anim.fast
                            }
                        }
                    }

                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: Hypr.focusOn(root.monitor, cell.workspaceId)
                    }
                }
            }
        }
    }
}
