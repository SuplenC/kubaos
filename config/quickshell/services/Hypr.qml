pragma Singleton

import Quickshell
import Quickshell.Io
import Quickshell.Hyprland

// Small helpers on top of the Hyprland IPC, and each screen's workspaces.
//
// Every screen owns a block of ten workspace ids, the laptop panel 1-10, the
// next screen 11-20 (config/hypr/workspaces.lua). Hyprland keeps which block
// is whose in a state file, read here. The bar labels each block's workspaces
// 1-10.
Singleton {
    id: root

    readonly property int slots: 10

    // The ShellScreen of the monitor Hyprland considers focused.
    readonly property ShellScreen focusedScreen: Quickshell.screens.find(s => s.name === Hyprland.focusedMonitor?.name) ?? Quickshell.screens[0] ?? null

    // The lines "block<TAB>connector<TAB>identity" of the state file, as
    // fields; a screen that is unplugged has an empty connector.
    readonly property var screenLines: screensFile.text().split("\n").map(line => line.split("\t")).filter(fields => fields.length >= 3)

    // Connector -> block.
    readonly property var blocks: {
        const next = {};
        for (const fields of screenLines) {
            if (fields[1] !== "")
                next[fields[1]] = parseInt(fields[0], 10);
        }
        return next;
    }

    function blockOf(id: int): int {
        return Math.floor((id - 1) / slots);
    }

    function slotOf(id: int): int {
        return (id - 1) % slots + 1;
    }

    // A monitor's block. Until Hyprland has written the file, the block of the
    // workspace it shows.
    function blockOfMonitor(monitor: HyprlandMonitor): int {
        const block = blocks[monitor?.name ?? ""];
        if (block !== undefined)
            return block;
        const active = monitor?.activeWorkspace?.id ?? 1;
        return active > 0 ? blockOf(active) : 0;
    }

    // Works for empty (not yet created) workspaces too, unlike HyprlandWorkspace.activate().
    function focusWorkspace(id: int): void {
        Hyprland.dispatch(Hyprland.usingLua ? `hl.dsp.focus({ workspace = "${id}" })` : `workspace ${id}`);
    }

    // Goes to workspace `id` on `monitor`, through config/hypr/workspaces.lua,
    // which brings it back from another screen if an unplug left it there.
    function focusOn(monitor: HyprlandMonitor, id: int): void {
        if (Hyprland.usingLua && monitor)
            Hyprland.dispatch(`dot.workspaces.go_to(${id}, "${monitor.name}")`);
        else
            focusWorkspace(id);
    }

    // The next (`direction` 1) or previous (-1) workspace on `monitor`, as
    // SUPER + Tab walks them, through config/hypr/workspaces.lua. Without it,
    // `fallback`, the workspace beside in the bar.
    function step(monitor: HyprlandMonitor, direction: int, fallback: int): void {
        if (Hyprland.usingLua && monitor)
            Hyprland.dispatch(`dot.workspaces.${direction > 0 ? "next" : "previous"}("${monitor.name}")`);
        else
            focusWorkspace(fallback);
    }

    FileView {
        id: screensFile

        path: `${Quickshell.env("XDG_STATE_HOME") || Quickshell.env("HOME") + "/.local/state"}/kubaos/hypr/workspace-screens`
        watchChanges: true
        printErrors: false
        onFileChanged: reload()
    }
}
