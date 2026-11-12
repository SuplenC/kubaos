pragma Singleton
import QtQuick

// Stand-in for Players: Spotify playing, the active one, and Firefox paused.
QtObject {
    id: root

    component Player: QtObject {
        property string identity
        property string desktopEntry
        property string dbusName
        property bool isPlaying
        property bool canTogglePlaying: true
        property bool canGoNext: true
        property bool canGoPrevious: true
        property bool canRaise: true
    }

    readonly property Player spotify: Player {
        identity: "Spotify"
        desktopEntry: "spotify"
        dbusName: "org.mpris.MediaPlayer2.spotify"
        isPlaying: true
    }
    readonly property Player firefox: Player {
        identity: "Mozilla Firefox"
        desktopEntry: "firefox"
        dbusName: "org.mpris.MediaPlayer2.firefox.instance_1_42"
        isPlaying: false
        canGoNext: false
        canGoPrevious: false
    }

    property var players: [root.spotify, root.firefox]
    property var active: root.spotify

    function pick(player: var): void {
        root.active = player;
    }

    function togglePlaying(player: var): void {
        player.isPlaying = !player.isPlaying;
    }

    function next(player: var): void {
    }

    function previous(player: var): void {
    }

    function name(player: var): string {
        return player.identity;
    }

    function icon(player: var): string {
        return player.desktopEntry;
    }

    function show(player: var): void {
    }
}
