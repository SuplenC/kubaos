import QtQuick
import Quickshell
import Quickshell.Hyprland
import KubaOS.Components as K
import qs.services

// Title of the focused window, shown only on the monitor that holds it: one line, cut with an
// ellipsis where the room up to the clock ends. A new title slides in from below.
Item {
    id: root

    required property ShellScreen screen
    // The widest the title may draw.
    property real room: Infinity
    readonly property HyprlandToplevel toplevel: Hyprland.activeToplevel
    // The screen comes from the window's workspace through `Hypr.monitorNameOf`, which a
    // workspace list Quickshell reads late can't put back on a screen the workspace has left.
    readonly property string title: (toplevel?.workspace ? Hypr.monitorNameOf(toplevel.workspace) : "") === screen?.name ? (toplevel?.title ?? "") : ""
    // With KUBAOS_WORKSPACES_PROBE=1, the title is logged on every change, for
    // `workspaces_hotplug_bar` in `test/lib/e2e.sh`.
    readonly property bool probed: Quickshell.env("KUBAOS_WORKSPACES_PROBE") === "1"

    // The room only cuts the drawn width: laying the words out again as the workspaces glide
    // would run script every frame.
    implicitWidth: Math.min(label.implicitWidth, root.room)
    implicitHeight: label.implicitHeight
    onTitleChanged: {
        swap.restart();
        if (probed)
            console.info(`active window probe: ${screen.name} ${JSON.stringify(title)}`);
    }

    K.Text {
        id: label

        width: root.width
        text: root.title
        style: K.Tokens.type.body_strong
        color: K.Tokens.color.on_surface_variant
        maximumLines: 1

        transform: Translate {
            id: shift
        }
    }

    ParallelAnimation {
        id: swap

        NumberAnimation {
            target: label
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
