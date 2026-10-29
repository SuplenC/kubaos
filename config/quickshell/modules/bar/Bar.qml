import QtQuick
import Quickshell
import KubaOS.Components as K
import qs.services

// The top edge of the frame. Left: workspaces, the live preview (while the home is built from a
// KubaOS checkout) and the window title.
// Center: clock (opens the clock popup). Right: recording indicator (while recording), Keep
// awake and a scheduled action (while on), the Proxy (while on), the keyboard layout (with two
// or more), the update marker (while a KubaOS release waits), tray, the system monitor (opens its
// popup), the bell (while there is a notification; opens the notifications panel), status (opens
// the control center), power menu.
//
// Every group is a pill as tall as a small control, 4 inside the bar's 40, so each lands on whole
// device pixels at every scale.
Item {
    id: root

    required property ShellScreen screen
    required property QsWindow window
    // The pointer has rested on the bell: the frame shows the peek.
    readonly property bool bellPeeking: bell.peeking

    Workspaces {
        id: workspaces

        anchors.left: parent.left
        anchors.leftMargin: K.Tokens.space.s8
        anchors.verticalCenter: parent.verticalCenter
        screen: root.screen
    }

    // Beside the workspaces, where the window title gives way to it, so the right side never
    // reaches the clock.
    PreviewIndicator {
        id: preview

        anchors.left: workspaces.right
        anchors.leftMargin: K.Tokens.space.s8
        anchors.verticalCenter: parent.verticalCenter
    }

    // Anchored to the pill rather than laid out beside it, so its place follows the pill's width
    // as it animates, and its width depends on nothing it sets itself.
    ActiveWindow {
        anchors.left: preview.visible ? preview.right : workspaces.right
        anchors.leftMargin: K.Tokens.space.s12
        anchors.verticalCenter: parent.verticalCenter
        room: Math.max(0, clock.x - x - K.Tokens.space.s40)
        screen: root.screen
    }

    Clock {
        id: clock

        anchors.centerIn: parent
    }

    // A positioner, not `K.Row`: a pill's width glides when its content changes, and a positioner
    // follows it each frame without running script.
    Row {
        anchors.right: parent.right
        anchors.verticalCenter: parent.verticalCenter
        anchors.rightMargin: K.Tokens.space.s8
        spacing: K.Tokens.space.s8

        RecordingIndicator {}

        IdleIndicator {}

        ProxyIndicator {}

        KeyboardLayout {}

        UpdateIndicator {}

        Tray {
            window: root.window
        }

        SysmonPill {}

        Bell {
            id: bell
        }

        Status {}

        K.IconButton {
            icon: "power_settings_new"
            label: "Power"
            kind: "Tonal"
            size: "Sm"
            activeFocusOnTab: false
            onActivated: Panels.toggle("powermenu")
        }
    }
}
