pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io
import Quickshell.Hyprland
import qs.settings
import KubaOS.Components as K
import "dnd.js" as Dnd

// Small helpers on top of the Hyprland IPC, and each screen's workspaces.
//
// Every position on the desk owns a block of ten workspace ids, the laptop
// panel 1-10, the next 11-20, and a screen shows the block of the position it
// stands at (config/hypr/workspaces.lua). Hyprland keeps which block is whose
// in a state file, read here. The bar labels each block's workspaces 1-10.
Singleton {
    id: root

    readonly property int slots: 10

    // The ShellScreen of the monitor Hyprland considers focused.
    readonly property ShellScreen focusedScreen: Quickshell.screens.find(s => s.name === Hyprland.focusedMonitor?.name) ?? Quickshell.screens[0] ?? null

    // The Hyprland monitor of `screen`, null while Hyprland lists none under its name. Found in
    // the list by name, so a binding on it follows the list. `Hyprland.monitorFor` answers once:
    // null for a screen Wayland announces before Hyprland's event for it is read, and an object
    // Quickshell deletes when the monitor goes, though the screen may stay or come back.
    function monitorOf(screen: ShellScreen): HyprlandMonitor {
        return Hyprland.monitors.values.find(monitor => monitor.name === screen?.name) ?? null;
    }

    // The lines "block<TAB>connector<TAB>position<TAB>screen" of the state
    // file, and "<TAB>name" for a group the user named, as fields; a
    // position no screen holds has an empty connector.
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

    // Block -> the name the user gave its group, for the groups that have one.
    readonly property var names: {
        const next = {};
        for (const fields of screenLines) {
            if ((fields[4] ?? "") !== "")
                next[parseInt(fields[0], 10)] = fields[4];
        }
        return next;
    }

    // How many screen shares run, counted from Hyprland's `screencast` events: one more as a client
    // (Meet, Discord, OBS) starts taking frames of a screen or a window, one less as it stops
    // (`dnd.js`). Unknown at start, so none until the next share starts.
    property int shares: 0
    readonly property bool sharing: root.shares > 0

    // How many windows are really fullscreen on a workspace a screen shows: Hyprland's fullscreen
    // mode 2, never a maximised window (mode 1), which a workspace's `hasFullscreen` counts too.
    // The mode is the client list's, read again on every `fullscreen` event.
    readonly property int fullscreenWindows: Dnd.fullscreenWindows(Hyprland.toplevels.values.map(t => ({
                shown: t.workspace?.active ?? false,
                mode: t.lastIpcObject?.fullscreen ?? 0
            })))
    readonly property bool fullscreen: root.fullscreenWindows > 0

    function blockOf(id: int): int {
        return Math.floor((id - 1) / slots);
    }

    function slotOf(id: int): int {
        return (id - 1) % slots + 1;
    }

    // The block of the screen named `name`, whose Hyprland monitor is `monitor`, looked up by name
    // so a screen Hyprland doesn't list yet still finds its own. Until Hyprland has written the
    // file, the block of the workspace the monitor shows; once it has, -1 for a screen the file
    // doesn't list yet, which holds no block.
    function blockOfScreen(name: string, monitor: HyprlandMonitor): int {
        const block = blocks[name];
        if (block !== undefined)
            return block;
        if (screenLines.length > 0)
            return -1;
        const active = monitor?.activeWorkspace?.id ?? 1;
        return active > 0 ? blockOf(active) : 0;
    }

    // The ids of the workspaces that hold windows, sorted, each between commas. Worked out once
    // here and kept as a string, which signals only when its text changes, so a window event
    // reaches the bar's cells only when the set changes: each cell scanning the windows itself
    // took about 1.2 ms of script per window event across three screens, the set about 30 µs.
    // Each window's own workspace counts, not a workspace's list of windows: a workspace
    // renumbered away from keeps its list, and the object is reused when Hyprland creates that id
    // again.
    readonly property string occupiedIds: {
        const ids = [];
        for (const toplevel of Hyprland.toplevels.values) {
            const id = toplevel.workspace?.id;
            if (id !== undefined && !ids.includes(id))
                ids.push(id);
        }
        return `,${ids.sort((a, b) => a - b).join(",")},`;
    }

    function occupied(id: int): bool {
        return root.occupiedIds.includes(`,${id},`);
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

    // The workspace groups the user pinned screens to. Hyprland reads the
    // file when its config loads; a hand edit is passed on as it is saved,
    // with no reload. Hyprland reads the file itself and moves nothing when
    // its text is the same, so the Command menu's edits, which tell Hyprland
    // themselves, are harmless to pass on again.
    FileView {
        path: `${UserLayer.dir("workspaces")}/groups.toml`
        watchChanges: true
        printErrors: false
        onFileChanged: groupsSaved.restart()
    }

    // An editor saving by rename moves the file away, then writes a new one
    // in its place, and each step is a change; the last one is passed on,
    // once the file is back.
    Timer {
        id: groupsSaved

        interval: K.Tokens.duration.short
        onTriggered: {
            if (Hyprland.usingLua)
                Hyprland.dispatch("dot.workspaces.reload_groups()");
        }
    }

    // The client list read once more after a burst of `fullscreen` events has settled. Quickshell
    // drops a read asked for while another is in flight, so the last change of a burst (a window
    // leaving fullscreen right after entering it) could otherwise be missed until the next one.
    Timer {
        id: fullscreenRead

        interval: K.Tokens.duration.short
        onTriggered: Hyprland.refreshToplevels()
    }

    // Hyprland's workspaces, screens and windows read again once its workspace and screen events
    // have held still for a moment, as one burst. Quickshell (v0.3.2) puts a workspace switch on
    // the focused screen whichever screen it happened on, and moves the workspace there with it:
    // the reconcile in `config/hypr/workspaces.lua` switching a screen that just arrived would leave the
    // workspace on the focused screen's bar. A list asked for before an event and read after it
    // can drop a screen just added or leave windows on a renumbered workspace's old object, and a
    // read asked for while one is in flight is dropped. The read after the burst puts all of it
    // right, and an ordinary switch costs one burst of three reads, a window event none.
    //
    // Each move and switch the reconcile makes starts the wait again, so the read follows its last
    // one whatever the interval. `medium` outlasts the reconcile's own wait for the screens to
    // settle (250 ms, `SETTLE_MS` there), so a screen arriving and its reconcile are read once.
    //
    // HAZARD: the workspaces before the screens. The workspace list makes again a screen that
    // Quickshell dropped, from the one each workspace names, and only then can the screen list,
    // which makes none, fill in what it shows. Hyprland answers in the order asked.
    Timer {
        id: settledRead

        interval: K.Tokens.duration.medium
        onTriggered: {
            Hyprland.refreshWorkspaces();
            Hyprland.refreshMonitors();
            Hyprland.refreshToplevels();
        }
    }

    // The events after which `settledRead` reads again.
    readonly property var unsettling: ["workspacev2", "focusedmon", "moveworkspacev2", "monitoraddedv2", "monitorremoved", "changeworkspaceid"]

    // The ids Hyprland has renumbered away from (`changeworkspaceid`, which
    // `dot.workspaces.move_to_monitor` causes), as a set. Quickshell (v0.3.2)
    // keys a workspace by the id it first saw and never changes it, so the
    // object under the old id stays in `Hyprland.workspaces`, with its old
    // monitor and its windows, until a config reload reads the list afresh;
    // the new id gets an object with the move that follows the renumber,
    // which names it. The bar leaves these out. An old id comes back when
    // Hyprland creates a workspace under it again, since Quickshell then
    // reuses the object; its monitor is read again then, as the object still
    // carries the old one.
    property var staleIds: ({})

    function isStale(id: int): bool {
        return staleIds[id] === true;
    }

    // Id -> the connector of the screen Hyprland last moved the workspace to.
    // Quickshell sets a workspace's screen from move events and from each
    // workspace list it reads. A list asked for before a move can be read
    // after the move's event, and puts the workspace back where it was: on a
    // hotplug, the list asked for as a screen arrives puts the groups the
    // reconcile then moves off the primary back on it, until the next list is
    // read. Hyprland sends every move of a workspace as an event, in order, so
    // the last event's screen wins here. An id destroyed, created, or
    // renumbered away from is placed with no move, so it loses its entry.
    property var movedTo: ({})

    // The connector of the screen workspace `workspace` is on, "" when unknown.
    function monitorNameOf(workspace: HyprlandWorkspace): string {
        return movedTo[workspace.id] ?? workspace.monitor?.name ?? "";
    }

    function forgetMove(id: int): void {
        if (movedTo[id] === undefined)
            return;
        const next = Object.assign({}, movedTo);
        delete next[id];
        movedTo = next;
    }

    Connections {
        target: Hyprland

        function onRawEvent(event: HyprlandEvent): void {
            if (root.unsettling.includes(event.name))
                settledRead.restart();
            if (event.name === "screencast") {
                root.shares = Dnd.screencast(root.shares, event.data);
            } else if (event.name === "fullscreen") {
                Hyprland.refreshToplevels();
                fullscreenRead.restart();
            } else if (event.name === "moveworkspacev2") {
                const fields = event.parse(3);
                const next = Object.assign({}, root.movedTo);
                next[parseInt(fields[0], 10)] = fields[2];
                root.movedTo = next;
            } else if (event.name === "destroyworkspacev2") {
                root.forgetMove(parseInt(event.parse(2)[0], 10));
            } else if (event.name === "changeworkspaceid") {
                const id = parseInt(event.parse(2)[0], 10);
                const next = Object.assign({}, root.staleIds);
                next[id] = true;
                root.staleIds = next;
                root.forgetMove(id);
                // The windows stay on the old id's object until read again;
                // the reply comes after the move that creates the new id's.
                // Should it not, the windows' read makes the object with no id
                // yet, and the workspaces' read, served next, gives it one.
                // The move names no active workspace for the screen it lands
                // on, so the screens are read again too. A read still in
                // flight drops the new one; then the first's reply, served
                // after the move's event, has what it needs.
                Hyprland.refreshToplevels();
                Hyprland.refreshWorkspaces();
                Hyprland.refreshMonitors();
            } else if (event.name === "createworkspacev2") {
                const id = parseInt(event.parse(2)[0], 10);
                root.forgetMove(id);
                if (root.staleIds[id] === true) {
                    const next = Object.assign({}, root.staleIds);
                    delete next[id];
                    root.staleIds = next;
                    Hyprland.refreshWorkspaces();
                }
            } else if (event.name === "configreloaded" && Object.keys(root.staleIds).length > 0) {
                root.staleIds = {};
            }
        }
    }
}
