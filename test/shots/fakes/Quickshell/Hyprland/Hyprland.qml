pragma Singleton
import QtQuick

// Stand-in for Hyprland's IPC: one monitor with workspaces 1, 2, 3, 11 and 12, the second one
// focused, and three windows. Workspaces are plain objects: a QML type can't declare `id`.
QtObject {
    id: hyprland

    readonly property HyprlandMonitor monitor: HyprlandMonitor {
        activeWorkspace: hyprland.spaces[1]
    }
    readonly property HyprlandMonitor focusedMonitor: hyprland.monitor
    readonly property var spaces: [1, 2, 3, 11, 12].map(n => ({
                id: n,
                name: `${n}`,
                monitor: hyprland.monitor,
                active: n === 2,
                focused: n === 2,
                urgent: false,
                hasFullscreen: false
            }))
    readonly property var focusedWorkspace: hyprland.spaces[1]
    readonly property var workspaces: ({
            values: hyprland.spaces
        })
    property list<HyprlandToplevel> windows: [
        HyprlandToplevel {
            title: "kubaos — nvim config/quickshell/modules/bar/Workspaces.qml"
            workspace: hyprland.spaces[1]
            monitor: hyprland.monitor
        },
        HyprlandToplevel {
            title: "Firefox"
            workspace: hyprland.spaces[0]
            monitor: hyprland.monitor
        },
        HyprlandToplevel {
            title: "Notes"
            workspace: hyprland.spaces[3]
            monitor: hyprland.monitor
        }
    ]
    readonly property var toplevels: ({
            values: Array.from(hyprland.windows)
        })
    readonly property var monitors: ({
            values: [hyprland.monitor]
        })
    readonly property HyprlandToplevel activeToplevel: hyprland.windows[0]

    signal rawEvent(var event)

    function monitorFor(screen: var): HyprlandMonitor {
        return hyprland.monitor;
    }

    function dispatch(request: string): void {
        console.log(`hyprland: dispatch ${request}`);
    }

    function refreshWorkspaces(): void {
    }

    function refreshToplevels(): void {
    }

    function refreshMonitors(): void {
    }
}
