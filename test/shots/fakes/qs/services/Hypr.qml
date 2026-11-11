pragma Singleton
import QtQuick

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

    function blockOf(id: int): int {
        return Math.floor((id - 1) / 10);
    }

    function slotOf(id: int): int {
        return (id - 1) % 10 + 1;
    }

    function blockOfMonitor(monitor: var): int {
        return 0;
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
