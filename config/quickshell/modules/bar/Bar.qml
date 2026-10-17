import QtQuick
import QtQuick.Layouts
import Quickshell
import qs.theme
import qs.components
import qs.services

// The top edge of the frame. Left: workspaces and window title.
// Center: clock. Right: recording indicator (while recording), tray, status
// (opens the control center), power menu.
Item {
    id: root

    required property ShellScreen screen
    required property QsWindow window

    RowLayout {
        anchors.left: parent.left
        anchors.verticalCenter: parent.verticalCenter
        anchors.leftMargin: Theme.bar.padding
        spacing: Theme.spacing.md

        Workspaces {
            screen: root.screen
        }

        ActiveWindow {
            screen: root.screen
            Layout.maximumWidth: Math.max(0, clock.x - x - Theme.spacing.xxxl - Theme.bar.padding)
        }
    }

    Clock {
        id: clock

        anchors.centerIn: parent
    }

    RowLayout {
        anchors.right: parent.right
        anchors.verticalCenter: parent.verticalCenter
        anchors.rightMargin: Theme.bar.padding
        spacing: Theme.spacing.sm

        RecordingIndicator {}

        Tray {
            window: root.window
        }

        Status {}

        IconButton {
            icon: "power_settings_new"
            size: Theme.bar.pillHeight
            baseColor: Theme.m3surfaceContainer
            tint: Theme.m3error
            tooltip: "Power"
            activeFocusOnTab: false
            onClicked: Panels.toggle("powermenu")
        }
    }
}
