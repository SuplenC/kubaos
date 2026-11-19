.pragma library

// Which workspaces hold windows, as `Hypr` works it out from Quickshell's windows.

// The ids of the workspaces that hold windows among `toplevels` (`HyprlandToplevel`s), sorted,
// each between commas. Each window's own workspace counts, not a workspace's list of windows: a
// workspace renumbered away from keeps its list, and the object is reused when Hyprland creates
// that id again. A pinned window counts nowhere: it follows its screen from workspace to
// workspace, so every empty one it passes through would read as holding a window. Whether it is
// pinned is the client list's (`lastIpcObject`), as fresh as the last read of it.
function ids(toplevels) {
    const found = [];
    for (const toplevel of toplevels) {
        const id = toplevel.workspace?.id;
        if (id !== undefined && toplevel.lastIpcObject?.pinned !== true && !found.includes(id))
            found.push(id);
    }
    return `,${found.sort((a, b) => a - b).join(",")},`;
}
