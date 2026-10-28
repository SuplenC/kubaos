pragma Singleton
import QtQuick
import Quickshell

// Stand-in for the StatusNotifierItem tray: Adwaita's own symbolic icon (no fill, so it draws in
// black), the same kind of icon filled #2e3436 as GTK's light theme draws it, and a colour icon.
// `@ICONS@` is the harness's icon directory, filled in when a tree is staged. The network's item
// has a menu as nm-applet's reads: checks, a separator, a disabled entry and a submenu.
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
            menu: QsMenuEntry {
                entries: [
                    QsMenuEntry {
                        text: "Enable Networking"
                        buttonType: QsMenuButtonType.CheckBox
                        checkState: Qt.Checked
                    },
                    QsMenuEntry {
                        text: "Enable Wi-Fi"
                        buttonType: QsMenuButtonType.CheckBox
                        checkState: Qt.Checked
                    },
                    QsMenuEntry {
                        isSeparator: true
                    },
                    QsMenuEntry {
                        text: "VPN Connections"
                        hasChildren: true
                    },
                    QsMenuEntry {
                        text: "Connection Information"
                        enabled: false
                    },
                    QsMenuEntry {
                        isSeparator: true
                    },
                    QsMenuEntry {
                        text: "Edit Connections…"
                    }
                ]
            }
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
