import QtQuick
import Quickshell
import Quickshell.Hyprland
import KubaOS.Components as K
import qs.services

// Title of the focused window, shown only on the monitor that holds it: one line, cut with an
// ellipsis where the room up to the clock ends. A new title slides in from below. A window drawn
// through XWayland has a warning before its title, with a hint saying so: the session asks every
// toolkit for Wayland first (`config/hypr/env.lua`), so such an app ignored that or has no
// Wayland backend.
Item {
    id: root

    required property ShellScreen screen
    // The widest the title may draw.
    property real room: Infinity
    readonly property HyprlandToplevel toplevel: Hyprland.activeToplevel
    // The screen comes from the window's workspace through `Hypr.monitorNameOf`, which a
    // workspace list Quickshell reads late can't put back on a screen the workspace has left.
    readonly property string title: (toplevel?.workspace ? Hypr.monitorNameOf(toplevel.workspace) : "") === screen?.name ? (toplevel?.title ?? "") : ""
    // Hyprland's client list says which windows are XWayland's; Quickshell fills a window's
    // `lastIpcObject` only from a read of that list.
    readonly property bool x11: root.title !== "" && root.toplevel?.lastIpcObject?.xwayland === true
    // Where the title starts: after the warning and its gap while there is one.
    readonly property real lead: root.x11 ? warning.width + K.Tokens.space.s8 : 0
    // With KUBAOS_WORKSPACES_PROBE=1, the title is logged on every change, for
    // `workspaces_hotplug_bar` in `test/lib/e2e.sh`.
    readonly property bool probed: Quickshell.env("KUBAOS_WORKSPACES_PROBE") === "1"

    // The room only cuts the drawn width: laying the words out again as the workspaces glide
    // would run script every frame.
    implicitWidth: Math.min(root.lead + label.implicitWidth, root.room)
    implicitHeight: label.implicitHeight
    onTitleChanged: {
        swap.restart();
        if (probed)
            console.info(`active window probe: ${screen.name} ${JSON.stringify(title)}`);
    }
    // A window opened since the last read has an empty `lastIpcObject` until the next one.
    onToplevelChanged: {
        if (root.toplevel && root.toplevel.lastIpcObject?.address === undefined)
            Hyprland.refreshToplevels();
    }

    Item {
        id: line

        width: root.width
        height: root.height

        transform: Translate {
            id: shift
        }

        K.IconInLine {
            id: warning

            visible: root.x11
            name: "warning"
            size: K.Tokens.type.body_strong.icon
            line: K.Tokens.type.body_strong.line
            color: K.Tokens.color.error

            MouseArea {
                id: pointer

                anchors.fill: parent
                hoverEnabled: true
            }

            K.Hint {
                text: "Runs through XWayland, not native Wayland"
                shown: pointer.containsMouse
            }
        }

        K.Text {
            id: label

            x: root.lead
            width: root.width - root.lead
            text: root.title
            style: K.Tokens.type.body_strong
            color: K.Tokens.color.on_surface_variant
            maximumLines: 1
        }
    }

    ParallelAnimation {
        id: swap

        NumberAnimation {
            target: line
            property: "opacity"
            from: 0
            to: 1
            duration: K.Tokens.duration.short
            easing.type: Easing.BezierSpline
            easing.bezierCurve: K.Tokens.easing.standard
        }

        NumberAnimation {
            target: shift
            property: "y"
            from: K.Tokens.space.s8
            to: 0
            duration: K.Tokens.duration.medium
            easing.type: Easing.BezierSpline
            easing.bezierCurve: K.Tokens.easing.emphasized_decel
        }
    }
}
