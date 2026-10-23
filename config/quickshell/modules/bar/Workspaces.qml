import QtQuick
import Quickshell
import Quickshell.Hyprland
import qs.theme
import qs.settings
import qs.components
import qs.services

// This screen's workspaces in groups, each numbered 1-10 as SUPER + 1-0 reach
// them. Its own group first: its first workspace always, then those of its own
// that exist. Then the group of each unplugged screen waiting here, ordered by
// where those screens last sat, left to right, with a dot between groups. The
// same order as `groups()` in config/hypr/workspace_blocks.lua, which next and
// previous walk. Scroll to step through them.
//
// The active indicator stretches toward the new workspace: its leading edge
// moves fast, the trailing edge follows a little later.
Pill {
    id: root

    required property ShellScreen screen
    readonly property HyprlandMonitor monitor: Hyprland.monitorFor(screen)
    readonly property int block: Hypr.blockOfMonitor(monitor)
    readonly property int activeId: monitor?.activeWorkspace?.id ?? 0
    // Lists of ids, one per group.
    readonly property var groups: {
        const sets = {};
        sets[block] = [block * Hypr.slots + 1];
        const guests = [];
        for (const workspace of Hyprland.workspaces.values) {
            const id = workspace.id;
            const owner = Hypr.blockOf(id);
            if (id <= 0 || (owner !== block && workspace.monitor?.name !== monitor?.name))
                continue;
            if (!sets[owner]) {
                sets[owner] = [];
                guests.push(owner);
            }
            if (!sets[owner].includes(id))
                sets[owner].push(id);
        }
        const x = owner => Hypr.positions[owner] ?? Infinity;
        guests.sort((left, right) => (x(left) - x(right)) || (left - right));
        return [block].concat(guests).map(owner => sets[owner].sort((a, b) => a - b));
    }
    readonly property var ids: [].concat(...groups)
    // The x of each cell in `ids`, a gap before each group after the first.
    readonly property var xs: {
        const result = [];
        let x = 0;
        groups.forEach((group, index) => {
            if (index > 0)
                x += groupGap;
            for (const _ of group) {
                result.push(x);
                x += cell;
            }
        });
        return result;
    }
    readonly property int slot: Math.max(0, ids.indexOf(activeId))
    readonly property real activeX: xs[slot] ?? 0
    readonly property int cell: Theme.bar.pillHeight - 6
    readonly property int groupGap: Theme.spacing.md
    readonly property int separator: Theme.spacing.xs
    readonly property var kanji: ["一", "二", "三", "四", "五", "六", "七", "八", "九", "十"]

    padding: 3
    spacing: 0
    interactive: true
    onScrolled: steps => Hypr.step(monitor, steps > 0 ? -1 : 1, ids[(slot - steps + ids.length) % ids.length])

    Item {
        id: cells

        implicitWidth: root.ids.length * root.cell + (root.groups.length - 1) * root.groupGap
        implicitHeight: root.cell
        // Whatever its two edges are animating, the indicator draws on the
        // cells only, never onto the window title beside the pill.
        clip: true

        Rectangle {
            id: indicator

            // Plain values, set in onActiveXChanged, so the durations are
            // chosen before the Behaviors start.
            property real start: 0
            property real end: root.cell
            property bool ready: false
            property int lead: Theme.anim.normal
            property int trail: Theme.anim.large

            Component.onCompleted: {
                start = root.activeX;
                end = start + root.cell;
                ready = true;
            }

            x: Math.max(0, start)
            width: Math.max(0, Math.min(end, cells.width) - x)
            height: root.cell
            radius: height / 2
            color: Theme.m3primary

            Connections {
                target: root

                // Both edges come from the target. Under its Behavior, `start`
                // still reads the old value right after it is set, and an end
                // taken from it lands a cell off: the indicator vanished on
                // the way right and covered two cells on the way back.
                function onActiveXChanged() {
                    const target = root.activeX;
                    const right = target > indicator.start;
                    startAnim.duration = right ? indicator.trail : indicator.lead;
                    endAnim.duration = right ? indicator.lead : indicator.trail;
                    indicator.start = target;
                    indicator.end = target + root.cell;
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
                    required property int index
                    readonly property int workspaceId: modelData
                    readonly property int number: Hypr.slotOf(workspaceId)
                    readonly property bool isActive: workspaceId === root.activeId
                    readonly property bool occupied: Hyprland.workspaces.values.some(w => w.id === workspaceId && w.toplevels.values.length > 0)
                    // The first cell of a group after the first carries the gap
                    // and the dot before it.
                    readonly property bool opensGroup: index > 0 && root.xs[index] - root.xs[index - 1] > root.cell

                    width: (opensGroup ? root.groupGap : 0) + root.cell
                    height: root.cell

                    Rectangle {
                        visible: cell.opensGroup
                        x: (root.groupGap - width) / 2
                        anchors.verticalCenter: parent.verticalCenter
                        width: root.separator
                        height: width
                        radius: width / 2
                        color: Theme.m3outlineVariant
                    }

                    Item {
                        x: cell.opensGroup ? root.groupGap : 0
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
}
