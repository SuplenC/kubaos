pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io
import KubaOS.Components as K

// The shell's theme: feeds the design tokens, `KubaOS.Components`' `Tokens` (generated from
// design/tokens.toml), the colours matugen derived from the wallpaper
// (~/.local/state/theme/colors.json, written by `dot-theme`, reloaded when it changes), and says
// where that state lives. Everything the shell draws reads `KubaOS.Components` directly.
Singleton {
    id: root

    readonly property string stateDir: (Quickshell.env("XDG_STATE_HOME") || `${Quickshell.env("HOME")}/.local/state`) + "/theme"

    // matugen's colours as colors.json holds them, or nothing before it has been read.
    readonly property var palette: K.Tokens.matugen ?? ({})

    function load(text: string): void {
        try {
            K.Tokens.matugen = JSON.parse(text);
        } catch (e) {
            console.warn(`Theme: ignoring unreadable colors.json: ${e}`);
        }
    }

    FileView {
        path: `${root.stateDir}/colors.json`
        watchChanges: true
        printErrors: false
        onFileChanged: reload()
        onLoaded: root.load(text())
        onLoadFailed: K.Tokens.matugen = null
    }
}
