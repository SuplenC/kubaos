pragma Singleton

import Quickshell
import Quickshell.Io
import Quickshell.Hyprland

// Small helpers on top of the Hyprland IPC, and each screen's workspaces.
//
// Every screen owns a block of ten workspace ids, the laptop panel 1-10, the
// next screen 11-20 (config/hypr/workspaces.lua). Hyprland keeps which block
// is whose, and where each screen last sat, in a state file, read here. The
// bar labels each block's workspaces 1-10 and orders the groups of unplugged
// screens by where those sat.
Singleton {
    id: root

    readonly property int slots: 10

    // The ShellScreen of the monitor Hyprland considers focused.
    readonly property ShellScreen focusedScreen: Quickshell.screens.find(s => s.name === Hyprland.focusedMonitor?.name) ?? Quickshell.screens[0] ?? null

    // The lines "block<TAB>connector<TAB>identity<TAB>x" of the state file, as
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

    // Block -> x where its screen last sat, for the screens that have one.
    readonly property var positions: {
        const next = {};
        for (const fields of screenLines) {
            if (fields.length >= 4 && fields[3] !== "")
                next[parseInt(fields[0], 10)] = parseInt(fields[3], 10);
        }
        return next;
    }

    function blockOf(id: int): int {
        return Math.floor((id - 1) / slots);
    }

    function slotOf(id: int): int {
        return (id - 1) % slots + 1;
    }

    // The id next creates after the last workspace of `block`'s group, given
    // the ids in use: `append_slot` in `config/hypr/workspace_blocks.lua`, which
    // this mirrors. 0 when all ten are taken.
    function appendSlot(block: int, taken: var): int {
        const first = block * slots + 1;
        const last = first + slots - 1;
        const own = taken.filter(id => id >= first && id <= last);
        const highest = Math.max(first - 1, ...own);
        if (highest < last)
            return highest + 1;
        for (let id = first; id <= last; id++) {
            if (!own.includes(id))
                return id;
        }
        return 0;
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
