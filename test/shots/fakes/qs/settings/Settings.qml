pragma Singleton
import QtQuick

// Stand-in for Settings: the defaults, with the pinned wallpaper sets (`@WALLPAPERS@`, filled in
// when a tree is staged).
QtObject {
    readonly property string home: "/home/kuba"
    readonly property string wallpaperSets: "@WALLPAPERS@"
    readonly property string wallpaperAnimate: "ac"
    readonly property string lockLoop: ""
    readonly property int screenOffAfter: 600
    readonly property bool workspaceKanji: false
    readonly property string menuMotion: "full"
    readonly property bool focusStutter: true
    readonly property bool u2f: true
    readonly property bool fingerprint: false
    readonly property bool polkitAgent: true
    readonly property int notificationTimeout: 6000
    property var user: ({})

    function readUser(text: string): void {
    }
}
