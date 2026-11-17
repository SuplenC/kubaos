pragma Singleton
import QtQuick
import Quickshell.Hyprland

// Stand-in for Hypr. The drivers set `focusedScreen` to the screen they shoot.
QtObject {
    readonly property int slots: 10
    property var focusedScreen: null
    readonly property var screenLines: []
    readonly property var blocks: [0]
    readonly property var names: ({
            "1": "Desk"
        })
    property var staleIds: ({})
    // As the service works it out (`occupied.js`): each window's own workspace, pinned windows
    // left out.
    readonly property string occupiedIds: {
        const ids = [];
        for (const toplevel of Hyprland.toplevels.values) {
            const id = toplevel.workspace?.id;
            if (id !== undefined && toplevel.lastIpcObject?.pinned !== true && !ids.includes(id))
                ids.push(id);
        }
        return `,${ids.sort((a, b) => a - b).join(",")},`;
    }

    function blockOf(id: int): int {
        return Math.floor((id - 1) / 10);
    }

    function slotOf(id: int): int {
        return (id - 1) % 10 + 1;
    }

    // The stand-in's one monitor, whichever the screen.
    function monitorOf(screen: var): var {
        return Hyprland.monitor;
    }

    function blockOfScreen(name: string, monitor: var): int {
        return 0;
    }

    function occupied(id: int): bool {
        return occupiedIds.includes(`,${id},`);
    }

    function focusWorkspace(id: int): void {
    }

    function focusOn(monitor: var, id: int): void {
    }

    function step(monitor: var, direction: int, fallback: int): void {
    }

    // The screen a workspace is on, as the service reads it: the stand-in's workspaces carry their
    // monitor, and none is ever moved.
    function monitorNameOf(workspace: var): string {
        return workspace?.monitor?.name ?? "";
    }

    function isStale(id: int): bool {
        return false;
    }
}
