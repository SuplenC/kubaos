import QtQuick
import Quickshell
import Quickshell.Hyprland
import qs.theme
import qs.settings
import qs.components
import qs.services

// This screen's workspaces in groups, each numbered 1-10 as SUPER + 1-0 reach
// them, a thin rule between groups. The groups come in block order, which is
// the order Hyprland's three-finger swipe walks the screen's workspaces in and
// `groups()` in config/hypr/workspace_blocks.lua, which next and previous
// walk: the screen's own group first, unless a waiting group has lower numbers
// (the laptop's, while its lid is closed). Its own group always shows its
// first workspace. The group of a desk position no screen holds, waiting
// here, sits on a tint, so it reads as borrowed, and starts with its name
// when the user gave it one (the workspace groups file). Scroll to step
// through them.
//
// Each group keeps one empty workspace after its last with windows
// (config/hypr/workspaces.lua), where a swipe or next opens a new one. It
// draws as a faint `+` until it is shown.
//
// Every change of shape glides: a new workspace grows in place, one that goes
// away shrinks out, and the cells after it, the gaps between groups and the
// pill's width follow, so the title beside the pill never jumps. Each cell is
// one entry of a ListModel kept in step with the groups, so its item lives on
// and animates instead of being rebuilt.
//
// The active indicator stretches toward the new workspace: its leading edge
// moves fast, the trailing edge follows a little later.
Pill {
    id: root

    required property ShellScreen screen
    readonly property HyprlandMonitor monitor: Hyprland.monitorFor(screen)
    readonly property int block: Hypr.blockOfMonitor(monitor)
    readonly property int activeId: monitor?.activeWorkspace?.id ?? 0
    // Lists of ids, one per group, in block order.
    readonly property var groups: {
        const sets = {};
        sets[block] = [block * Hypr.slots + 1];
        for (const workspace of Hyprland.workspaces.values) {
            const id = workspace.id;
            const owner = Hypr.blockOf(id);
            if (id <= 0 || Hypr.isStale(id) || (owner !== block && workspace.monitor?.name !== monitor?.name))
                continue;
            sets[owner] = sets[owner] ?? [];
            if (!sets[owner].includes(id))
                sets[owner].push(id);
        }
        return Object.keys(sets).map(owner => parseInt(owner, 10)).sort((a, b) => a - b).map(owner => sets[owner].sort((a, b) => a - b));
    }
    readonly property var groupBlocks: groups.map(group => Hypr.blockOf(group[0]))
    // Workspace ids only, in the bar's order: what scrolling steps through.
    readonly property var ids: [].concat(...groups)
    // What the pill draws, in order: `{ key, kind, workspace, borrowed, head,
    // tail, label }`, `kind` one of "cell", "ghost" (the empty workspace a
    // group keeps after its last, not shown), "gap" (the rule before a group
    // after the first) and "label" (a borrowed group's name, before its
    // cells). `head` and `tail` mark a group's first and last entry, where
    // the tint of a borrowed group rounds off.
    readonly property var entries: {
        const result = [];
        groups.forEach((group, index) => {
            const borrowed = groupBlocks[index] !== block;
            const name = borrowed ? Hypr.names[groupBlocks[index]] ?? "" : "";
            if (index > 0)
                result.push({
                    key: `gap${groupBlocks[index]}`,
                    kind: "gap",
                    workspace: 0,
                    borrowed: false,
                    head: false,
                    tail: false,
                    label: ""
                });
            if (name !== "")
                result.push({
                    key: `label${groupBlocks[index]}`,
                    kind: "label",
                    workspace: group[0],
                    borrowed: true,
                    head: true,
                    tail: false,
                    label: name
                });
            group.forEach((id, at) => {
                const last = at === group.length - 1;
                const ghost = last && at > 0 && id !== activeId && !occupied(id);
                result.push({
                    key: `${id}`,
                    kind: ghost ? "ghost" : "cell",
                    workspace: id,
                    borrowed: borrowed,
                    head: at === 0 && name === "",
                    tail: last,
                    label: ""
                });
            });
        });
        return result;
    }
    // Where the active cell ends up once every width has settled, -1 while it
    // isn't drawn.
    readonly property real activeX: {
        let x = 0;
        for (const entry of entries) {
            if ((entry.kind === "cell" || entry.kind === "ghost") && entry.workspace === activeId)
                return x;
            x += widthOf(entry.kind, entry.label, entry.workspace);
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
    readonly property real borrowedOpacity: 0.6
    readonly property var kanji: ["一", "二", "三", "四", "五", "六", "七", "八", "九", "十"]
    readonly property var roles: ["kind", "borrowed", "head", "tail", "label"]
    property bool ready: false

    // By each window's own workspace, not a workspace's list of windows: a
    // workspace renumbered away from keeps its list, and the object is reused
    // when Hyprland creates that id again.
    function occupied(id: int): bool {
        return Hyprland.toplevels.values.some(t => t.workspace?.id === id);
    }

    // What the cell of workspace `id` draws: its number, or its kanji.
    function numberText(id: int): string {
        const slot = Hypr.slotOf(id);
        return Settings.workspaceKanji ? kanji[slot - 1] : `${slot}`;
    }

    // The room the cell of workspace `id` leaves before its number, which it
    // centres, unrounded.
    function numberRoom(id: int): real {
        return (cell - numberMetrics.advanceWidth(numberText(id))) / 2;
    }

    // Where a group's name starts: as far from the tint's rounded end as the
    // number of the group's first cell, `first`, is from the name's end.
    function labelInset(first: int): int {
        return Math.round(numberRoom(first));
    }

    // How wide a group's name draws, elided past `groupNameMax`.
    function labelWidth(label: string): real {
        return Math.min(labelMetrics.advanceWidth(label), Theme.bar.groupNameMax);
    }

    // An entry's width. A name's entry is its inset, the name, and the inset
    // again less the room its first cell leaves before the number, so the
    // name sits as far from the number after it as from the tint's end.
    function widthOf(kind: string, label: string, first: int): int {
        if (kind === "label")
            return Math.round(2 * labelInset(first) + labelWidth(label) - numberRoom(first));
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
                shapes.insert(at, Object.assign({
                    leaving: false
                }, entry));
            } else {
                if (found !== at)
                    shapes.move(found, at, 1);
                for (const role of roles) {
                    if (shapes.get(at)[role] !== entry[role])
                        shapes.setProperty(at, role, entry[role]);
                }
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

    // Measures a group's name with the font it is drawn in.
    FontMetrics {
        id: labelMetrics

        font.family: Theme.font.sans
        font.pixelSize: Theme.bar.cellFont
        font.weight: Font.Medium
    }

    // Measures a cell's number as a waiting group's first cell draws it: in
    // a group of windows, bold.
    FontMetrics {
        id: numberMetrics

        font.family: Settings.workspaceKanji ? Theme.font.jp : Theme.font.sans
        font.pixelSize: Theme.bar.cellFont
        font.weight: Font.DemiBold
        font.features: Theme.font.tabular
    }

    // One entry's place in a row, growing in when it appears and shrinking
    // out when it leaves. The tints and the cells are two rows of these over
    // the same model, so the indicator can sit between them.
    component Slot: Item {
        id: slotItem

        required property string key
        required property string kind
        required property string label
        required property int workspace
        required property bool leaving
        // False until the item exists, so a new one grows from nothing
        // rather than appearing at full width.
        property bool appeared: false
        readonly property bool present: appeared && !leaving

        Component.onCompleted: appeared = true

        width: present ? root.widthOf(kind, label, workspace) : 0
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
    }

    Item {
        id: cells

        implicitWidth: row.implicitWidth
        implicitHeight: root.cell
        // Whatever its two edges are animating, the indicator draws on the
        // cells only, never onto the window title beside the pill.
        clip: true

        Row {
            Repeater {
                model: shapes

                delegate: Slot {
                    id: tintSlot

                    required property bool borrowed
                    required property bool head
                    required property bool tail

                    // A pill as wide as the group: each entry draws its part,
                    // reaching past its inner edges into the clip, so only the
                    // group's two ends round off.
                    Rectangle {
                        visible: tintSlot.borrowed
                        x: tintSlot.head ? 0 : -root.cell / 2
                        width: root.widthOf(tintSlot.kind, tintSlot.label, tintSlot.workspace) + (tintSlot.head ? 0 : root.cell / 2) + (tintSlot.tail ? 0 : root.cell / 2)
                        height: root.cell
                        radius: height / 2
                        color: Theme.m3secondaryContainer
                        opacity: root.borrowedOpacity
                    }
                }
            }
        }

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

                delegate: Slot {
                    id: shape

                    readonly property bool isGap: kind === "gap"
                    readonly property bool isGhost: kind === "ghost"
                    readonly property bool isLabel: kind === "label"
                    readonly property bool isActive: !isGap && !isLabel && workspace === root.activeId
                    readonly property bool occupied: root.occupied(workspace)

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

                    Text {
                        visible: shape.isLabel
                        x: root.labelInset(shape.workspace)
                        width: Math.ceil(root.labelWidth(shape.label))
                        anchors.verticalCenter: parent.verticalCenter
                        text: shape.label
                        elide: Text.ElideRight
                        font: labelMetrics.font
                        renderType: Text.NativeRendering
                        color: Theme.m3onSecondaryContainer
                    }

                    // The name goes to its group's first workspace, as its
                    // first cell does.
                    MouseArea {
                        visible: shape.isLabel
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: Hypr.focusOn(root.monitor, shape.workspace)
                    }

                    Item {
                        visible: !shape.isGap && !shape.isLabel
                        anchors.horizontalCenter: parent.horizontalCenter
                        width: root.cell
                        height: root.cell

                        Text {
                            anchors.centerIn: parent
                            text: root.numberText(shape.workspace)
                            font.family: Settings.workspaceKanji ? Theme.font.jp : Theme.font.sans
                            font.pixelSize: Theme.bar.cellFont
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
                            onClicked: Hypr.focusOn(root.monitor, shape.workspace)
                        }
                    }
                }
            }
        }
    }
}
