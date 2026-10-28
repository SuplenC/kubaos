pragma Singleton
import QtQuick

// Stand-in for the StatusNotifierItem tray: Adwaita's own symbolic icon (no fill, so it draws in
// black), the same kind of icon filled #2e3436 as GTK's light theme draws it, and a colour icon.
// `@ICONS@` is the harness's icon directory, filled in when a tree is staged.
QtObject {
    id: tray

    property list<SystemTrayItem> list: [
        SystemTrayItem {
            icon: "file://@ICONS@/audio-volume-high-symbolic.svg"
            title: "Volume"
        },
        SystemTrayItem {
            icon: "file://@ICONS@/network-wireless-dark-symbolic.svg"
            title: "Network"
        },
        SystemTrayItem {
            icon: "file://@ICONS@/folder.svg"
            title: "Files"
        }
    ]
    readonly property TrayModel items: TrayModel {
        source: tray.list
    }
}
