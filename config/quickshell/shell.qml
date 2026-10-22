//@ pragma UseQApplication

import QtQuick
import Quickshell
import qs.settings
import qs.modules.frame
import qs.modules.lock
import qs.modules.menu
import qs.modules.polkit
import qs.modules.wallpaper

// Entry point. Each screen gets a wallpaper and a frame (bar, border and every
// panel; see modules/frame/Frame.qml). The lock screen, polkit agent and IPC
// exist once, and so does the Command menu: `kotodama toggle` reaches one menu
// only. See README.md for the file map.
ShellRoot {
    Variants {
        model: Quickshell.screens

        Scope {
            id: perScreen

            required property ShellScreen modelData

            Wallpaper {
                screen: perScreen.modelData
            }

            Frame {
                screen: perScreen.modelData
            }
        }
    }

    Lock {}

    Menu {}

    LazyLoader {
        active: Settings.polkitAgent

        Polkit {}
    }

    IpcHandlers {}
}
