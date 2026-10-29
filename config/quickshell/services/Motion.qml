pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Wayland
import Quickshell.Hyprland
import Quickshell.Services.UPower
import qs.settings

// When moving wallpapers may decode. The user's mode (always / ac / never,
// remembered in `ShellState`) and what the machine is doing: on battery, locked,
// screens off (idle as long as hypridle waits before DPMS), a fullscreen
// window on that monitor. Anything that says no stops the video and shows
// its poster frame.
Singleton {
    id: root

    readonly property list<string> modes: ["ac", "always", "never"]
    readonly property string mode: modes.includes(ShellState.animate) ? ShellState.animate : Settings.wallpaperAnimate
    readonly property string modeLabel: mode === "always" ? "Always" : mode === "never" ? "Never" : "On AC"
    readonly property bool onBattery: UPower.onBattery
    // The mode and the power source allow motion at all.
    readonly property bool allowed: mode === "always" || (mode === "ac" && !onBattery)
    // No input for as long as hypridle waits before turning the screens off.
    readonly property bool screensOff: idle.isIdle
    // Desktop: allowed, not under the lock screen, screens on.
    readonly property bool desktop: allowed && !Session.locked && !screensOff
    // Lock screen: allowed and screens on.
    readonly property bool lock: allowed && !screensOff

    // screen name -> path of the video playing on it (for `wallpaper status`).
    property var playing: ({})

    function setMode(m: string): bool {
        if (!modes.includes(m))
            return false;
        ShellState.animate = m;
        return true;
    }

    // always -> ac -> never -> always.
    function cycle(): void {
        setMode(mode === "always" ? "ac" : mode === "ac" ? "never" : "always");
    }

    // A fullscreen window on this screen's active workspace hides the
    // wallpaper completely. Hyprland only; other compositors say false.
    function covered(screen: ShellScreen): bool {
        return Hyprland.monitorFor(screen)?.activeWorkspace?.hasFullscreen ?? false;
    }

    function desktopOn(screen: ShellScreen): bool {
        return desktop && !covered(screen);
    }

    function report(screen: string, path: string): void {
        const p = Object.assign({}, playing);
        if (path)
            p[screen] = path;
        else
            delete p[screen];
        playing = p;
    }

    IdleMonitor {
        id: idle

        timeout: Settings.screenOffAfter
        respectInhibitors: true
    }
}
