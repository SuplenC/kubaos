import QtQuick
import Quickshell
import Quickshell.Hyprland
import qs.theme
import qs.components

// Title of the focused window, shown only on the monitor that holds it.
// A new title slides in from below.
StyledText {
    id: root

    required property ShellScreen screen
    readonly property HyprlandToplevel toplevel: Hyprland.activeToplevel
    readonly property string title: toplevel?.monitor?.name === screen?.name ? (toplevel?.title ?? "") : ""

    text: title
    color: Theme.m3onSurfaceVariant
    font.weight: Font.Medium
    onTitleChanged: swap.restart()

    transform: Translate {
        id: shift
    }

    ParallelAnimation {
        id: swap

        Anim {
            target: root
            property: "opacity"
            from: 0
            to: 1
            duration: Theme.anim.small
            easing.bezierCurve: Theme.anim.effects
        }

        Anim {
            target: shift
            property: "y"
            from: 6
            to: 0
            duration: Theme.anim.normal
            easing.bezierCurve: Theme.anim.emphasizedDecel
        }
    }
}
