pragma Singleton

import QtQuick
import Quickshell

// Where the shell finds the user's files in the user layer, ~/.config/kubaos,
// grouped by area (docs/CONTRACTS.md, "Defaults and the user layer"). The same
// table in the other languages: crates/kubaos-user-layer (Rust) and
// home/user-layer.nix (Nix and dot-config); only the apps the shell reads are
// here.
Singleton {
    id: root

    readonly property string home: Quickshell.env("HOME")
    // The user layer itself.
    readonly property string layer: `${Quickshell.env("XDG_CONFIG_HOME") || `${root.home}/.config`}/kubaos`
    // Each app's folder relative to the user layer.
    readonly property var apps: ({
            quickshell: "desktop/quickshell",
            clock: "desktop/clock",
            sysmon: "desktop/sysmon"
        })

    // dir("clock"): the app's folder.
    function dir(app: string): string {
        return `${root.layer}/${root.apps[app]}`;
    }

    // shown("clock", "clock.toml"): the file as a person would type it, `~/` for the home.
    function shown(app: string, name: string): string {
        const path = `${root.dir(app)}/${name}`;
        return path.startsWith(`${root.home}/`) ? `~${path.slice(root.home.length)}` : path;
    }
}
