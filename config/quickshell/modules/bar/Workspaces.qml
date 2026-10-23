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
// where those screens last sat, left to right, with a thin rule between groups. The
// same order as `groups()` in config/hypr/workspace_blocks.lua, which next and
// previous walk. Scroll to step through them.
//
// The group this screen shows ends in a ghost slot, a faint `+` where next
// creates the new workspace after the group's last; a click goes there. A
// group with all ten taken has none. Only the shown group has one, on every
// screen whether focused or not: next, previous and SUPER + 1-0 address the
// group a screen shows, and a pill that kept its shape as focus moved
// between screens never shifts the title beside it.
//
// Every change of shape glides: a new workspace fills the ghost in place and
// a new ghost grows after it, a workspace that goes away shrinks out, and the
// cells after it, the gaps between groups and the pill's width follow, so the
// title beside the pill never jumps. Each cell is one entry of a ListModel kept
// in step with the groups, so its item lives on and animates instead of being
// rebuilt.
//
// The active indicator stretches toward the new workspace: its leading edge
// moves fast, the trailing edge follows a little later.
Pill {
    id: root

    required property ShellScreen screen
    readonly property HyprlandMonitor monitor: Hyprland.monitorFor(screen)
    readonly property int block: Hypr.blockOfMonitor(monitor)
    readonly property int activeId: monitor?.activeWorkspace?.id ?? 0
    readonly property bool activeOccupied: (monitor?.activeWorkspace?.toplevels?.values?.length ?? 0) > 0
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
    readonly property var groupBlocks: groups.map(group => Hypr.blockOf(group[0]))
    // The group of the workspace this screen shows. Found by block rather than
    // by id, so a workspace focused before Hyprland lists it keeps its group.
    readonly property int activeGroup: Math.max(0, groupBlocks.indexOf(Hypr.blockOf(activeId)))
    // The workspace the ghost stands for, 0 for none.
    readonly property int ghostId: {
        const id = Hypr.appendSlot(groupBlocks[activeGroup], Hyprland.workspaces.values.map(w => w.id));
        return groups[activeGroup].includes(id) ? 0 : id;
    }
    // Workspace ids only, in the bar's order: what scrolling steps through.
    readonly property var ids: [].concat(...groups)
    // What the pill draws, in order: `{ key, kind, workspace }`, `kind` one of
    // "cell", "ghost" and "gap" (the dot before a group after the first). A
    // cell and its ghost share a key, so the ghost turns into the cell.
    readonly property var entries: {
        const result = [];
        groups.forEach((group, index) => {
            if (index > 0)
                result.push({
                    key: `gap${groupBlocks[index]}`,
                    kind: "gap",
                    workspace: 0
                });
            const shown = index === activeGroup && ghostId > 0 ? group.concat(ghostId).sort((a, b) => a - b) : group;
            for (const id of shown)
                result.push({
                    key: `${id}`,
                    kind: group.includes(id) ? "cell" : "ghost",
                    workspace: id
                });
        });
        return result;
    }
    // Where the active cell (or the ghost being filled) ends up once every
    // width has settled, -1 while it isn't drawn.
    readonly property real activeX: {
        let x = 0;
        for (const entry of entries) {
            if (entry.kind !== "gap" && entry.workspace === activeId)
                return x;
            x += widthOf(entry.kind);
        }
        return -1;
    }
    readonly property int slot: Math.max(0, ids.indexOf(activeId))
    readonly property int cell: Theme.bar.pillHeight - 6
    readonly property int groupGap: Theme.spacing.md
    // A thin rule between groups, not a dot: a dot reads as a notification.
    readonly property int separator: Math.round(cell * 0.5)
    // How long a change of shape takes. The indicator follows a cell that
    // moves with the same duration and curve, so it stays on it.
    readonly property int glide: Theme.anim.large
    readonly property real ghostOpacity: 0.45
    readonly property var kanji: ["一", "二", "三", "四", "五", "六", "七", "八", "九", "十"]
    property bool ready: false

    function widthOf(kind: string): int {
        return kind === "gap" ? groupGap : cell;
    }

    function indexOfKey(key: string): int {
        for (let i = 0; i < shapes.count; i++) {
            if (shapes.get(i).key === key)
                return i;
        }
        return -1;
    }

    // Brings `shapes` in line with `entries`. An entry that is no longer
    // wanted stays in place, marked leaving, until it has shrunk out.
    function sync(): void {
        const wanted = new Set(entries.map(entry => entry.key));
        for (let i = 0; i < shapes.count; i++) {
            if (!wanted.has(shapes.get(i).key))
                shapes.setProperty(i, "leaving", true);
        }
        let at = 0;
        for (const entry of entries) {
            while (at < shapes.count && !wanted.has(shapes.get(at).key))
                at++;
            const found = indexOfKey(entry.key);
            if (found < 0) {
                shapes.insert(at, {
                    key: entry.key,
                    kind: entry.kind,
                    workspace: entry.workspace,
                    leaving: false
                });
            } else {
                if (found !== at)
                    shapes.move(found, at, 1);
                if (shapes.get(at).kind !== entry.kind)
                    shapes.setProperty(at, "kind", entry.kind);
                if (shapes.get(at).leaving)
                    shapes.setProperty(at, "leaving", false);
            }
            at++;
        }
    }

    function drop(key: string): void {
        const index = indexOfKey(key);
        if (index >= 0 && shapes.get(index).leaving)
            shapes.remove(index);
    }

    // The ghost does what next does from the group's last workspace. When the
    // shown workspace is empty and last, it already is that new workspace.
    function openGhost(): void {
        const group = groups[activeGroup];
        if (!activeOccupied && activeId === group[group.length - 1])
            return;
        Hypr.focusOn(monitor, ghostId);
    }

    padding: 3
    spacing: 0
    interactive: true
    animatesWidth: false
    onScrolled: steps => Hypr.step(monitor, steps > 0 ? -1 : 1, ids[(slot - steps + ids.length) % ids.length])
    onEntriesChanged: sync()
    Component.onCompleted: {
        sync();
        ready = true;
    }

    ListModel {
        id: shapes
    }

    Item {
        id: cells

        implicitWidth: row.implicitWidth
        implicitHeight: root.cell
        // Whatever its two edges are animating, the indicator draws on the
        // cells only, never onto the window title beside the pill.
        clip: true

        Rectangle {
            id: indicator

            // Plain values, set in place(), so the durations are chosen
            // before the Behaviors start.
            property real start: 0
            property real end: root.cell
            property int lastId: 0
            property bool ready: false
            property int lead: Theme.anim.normal
            property int trail: Theme.anim.large

            // Both edges come from the target. Under its Behavior, `start`
            // still reads the old value right after it is set, and an end
            // taken from it lands a cell off: the indicator vanished on the
            // way right and covered two cells on the way back. A new active
            // workspace stretches the indicator; the same one moved by the
            // cells before it glides with them.
            function place(): void {
                const target = root.activeX;
                if (target < 0)
                    return;
                const stretch = root.activeId !== lastId;
                const right = target > start;
                lastId = root.activeId;
                startAnim.duration = !stretch ? root.glide : right ? trail : lead;
                endAnim.duration = !stretch ? root.glide : right ? lead : trail;
                start = target;
                end = target + root.cell;
            }

            Component.onCompleted: {
                place();
                ready = true;
            }

            x: Math.max(0, start)
            width: Math.max(0, Math.min(end, cells.width) - x)
            height: root.cell
            radius: height / 2
            color: Theme.m3primary

            // Deferred, so a change of the active id and of the layout in the
            // same turn place the indicator once, from both.
            Connections {
                target: root

                function onActiveXChanged() {
                    Qt.callLater(indicator.place);
                }

                function onActiveIdChanged() {
                    Qt.callLater(indicator.place);
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
            id: row

            Repeater {
                model: shapes

                delegate: Item {
                    id: shape

                    required property string key
                    required property string kind
                    required property int workspace
                    required property bool leaving
                    readonly property bool isGap: kind === "gap"
                    readonly property bool isGhost: kind === "ghost"
                    readonly property int number: Hypr.slotOf(workspace)
                    readonly property bool isActive: !isGap && workspace === root.activeId
                    readonly property bool occupied: Hyprland.workspaces.values.some(w => w.id === workspace && w.toplevels.values.length > 0)
                    // False until the item exists, so a new one grows from
                    // nothing rather than appearing at full width.
                    property bool appeared: false
                    readonly property bool present: appeared && !leaving

                    Component.onCompleted: appeared = true

                    width: present ? root.widthOf(kind) : 0
                    height: root.cell
                    opacity: present ? 1 : 0
                    clip: true

                    Behavior on width {
                        enabled: root.ready

                        Anim {
                            duration: root.glide
                            easing.bezierCurve: Theme.anim.emphasizedDecel
                        }
                    }

                    Behavior on opacity {
                        enabled: root.ready

                        Anim {
                            duration: root.glide
                            easing.bezierCurve: Theme.anim.effects
                        }
                    }

                    Timer {
                        running: shape.leaving
                        interval: root.glide
                        onTriggered: Qt.callLater(root.drop, shape.key)
                    }

                    Rectangle {
                        visible: shape.isGap
                        anchors.centerIn: parent
                        width: 1
                        height: root.separator
                        color: Theme.m3outlineVariant
                    }

                    Item {
                        visible: !shape.isGap
                        anchors.horizontalCenter: parent.horizontalCenter
                        width: root.cell
                        height: root.cell

                        Text {
                            anchors.centerIn: parent
                            text: Settings.workspaceKanji ? root.kanji[shape.number - 1] : shape.number
                            font.family: Settings.workspaceKanji ? Theme.font.jp : Theme.font.sans
                            font.pixelSize: Theme.font.sm + 1
                            font.weight: shape.isActive || shape.occupied ? Font.DemiBold : Font.Medium
                            font.features: Theme.font.tabular
                            renderType: Text.NativeRendering
                            color: shape.isActive ? Theme.m3onPrimary : shape.occupied ? Theme.m3onSurface : Theme.m3outline
                            opacity: shape.isGhost ? 0 : 1

                            Behavior on color {
                                CAnim {}
                            }

                            Behavior on opacity {
                                Anim {
                                    easing.bezierCurve: Theme.anim.effects
                                }
                            }
                        }

                        Icon {
                            anchors.centerIn: parent
                            name: "add"
                            size: Theme.font.md
                            color: Theme.m3outline
                            opacity: shape.isGhost ? root.ghostOpacity : 0

                            Behavior on opacity {
                                Anim {
                                    easing.bezierCurve: Theme.anim.effects
                                }
                            }
                        }

                        MouseArea {
                            anchors.fill: parent
                            cursorShape: Qt.PointingHandCursor
                            onClicked: shape.isGhost ? root.openGhost() : Hypr.focusOn(root.monitor, shape.workspace)
                        }
                    }
                }
            }
        }
    }
}
