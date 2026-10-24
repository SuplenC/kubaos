import QtQuick
import QtQuick.Layouts
import Quickshell
import qs.theme
import qs.components
import qs.services

// The top edge of the frame. Left: workspaces and window title.
// Center: clock (opens the clock popup). Right: recording indicator (while recording), the update
// marker (while a KubaOS release waits), tray, status
// (opens the control center), power menu.
Item {
    id: root

    required property ShellScreen screen
    required property QsWindow window

    Workspaces {
        id: workspaces

        anchors.left: parent.left
        anchors.leftMargin: Theme.bar.padding
        anchors.verticalCenter: parent.verticalCenter
        screen: root.screen
    }

    // Anchored to the pill rather than laid out beside it, so its place
    // follows the pill's width as it animates, and its width depends on
    // nothing it sets itself.
    ActiveWindow {
        anchors.left: workspaces.right
        anchors.leftMargin: Theme.spacing.md
        anchors.verticalCenter: parent.verticalCenter
        width: Math.max(0, Math.min(implicitWidth, clock.x - x - Theme.spacing.xxxl))
        screen: root.screen
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

        KeyboardLayout {}

        UpdateIndicator {}

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
