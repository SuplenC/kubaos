pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Hyprland
import Quickshell.Services.Mpris
import "players.js" as Logic

// Media players over MPRIS, and the active one the bar controls: the one picked, else the one
// that played last.
Singleton {
    id: root

    readonly property var players: Mpris.players.values
    readonly property MprisPlayer active: Logic.active(root.players, root.activeKey)

    // The D-Bus name of the player picked or that played last.
    property string activeKey: ""

    function pick(player: MprisPlayer): void {
        if (player)
            root.activeKey = player.dbusName;
    }

    function togglePlaying(player: MprisPlayer): void {
        if (player?.canTogglePlaying)
            player.togglePlaying();
    }

    function next(player: MprisPlayer): void {
        if (player?.canGoNext)
            player.next();
    }

    function previous(player: MprisPlayer): void {
        if (player?.canGoPrevious)
            player.previous();
    }

    function name(player: MprisPlayer): string {
        return player?.identity || root.entryOf(player)?.name || player?.desktopEntry || "Unknown player";
    }

    // An icon name for the theme, from the player's desktop entry; empty without one.
    function icon(player: MprisPlayer): string {
        return root.entryOf(player)?.icon ?? "";
    }

    // The player's desktop entry. Chromium's players name none, so their identity ("Chrome") is
    // the next best name to find one by.
    // Reading the entries makes a binding over this look again once they are scanned, which
    // finishes seconds after the shell starts.
    function entryOf(player: MprisPlayer): DesktopEntry {
        const entries = DesktopEntries.applications.values;
        if (player?.desktopEntry)
            return DesktopEntries.heuristicLookup(player.desktopEntry);
        if (!player?.identity)
            return null;
        return DesktopEntries.heuristicLookup(player.identity) ?? Logic.entryNamed(entries, player.identity);
    }

    // Brings the player's app forward: by MPRIS where the player can raise itself, else by
    // focusing its window in Hyprland.
    function show(player: MprisPlayer): void {
        if (!player)
            return;
        if (player.canRaise) {
            player.raise();
            return;
        }
        const windows = Hyprland.toplevels.values.map(t => ({
                    address: t.address,
                    appClass: t.wayland?.appId ?? t.lastIpcObject?.class ?? ""
                }));
        const entry = root.entryOf(player);
        const window = Logic.windowOf(windows, [player.desktopEntry, entry?.id, entry?.startupClass, player.identity]);
        if (window)
            Hyprland.dispatch(Hyprland.usingLua ? `hl.dsp.focus({ window = "address:0x${window.address}" })` : `focuswindow address:0x${window.address}`);
    }

    // A player that starts playing becomes the active one.
    Instantiator {
        model: Mpris.players

        delegate: Connections {
            required property MprisPlayer modelData

            target: modelData

            function onIsPlayingChanged(): void {
                root.activeKey = Logic.played(root.activeKey, modelData);
            }
        }
    }
}
