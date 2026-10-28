import QtQuick
import KubaOS.Components as K

// A reading split into parts: `kubaos-ui`'s `meter::meter`. Each part is a 0..1 share and a
// colour (memory as used and cache, a disk as used, a core as busy); what is left is the track.
// It has a progress bar's look: 8 tall, a pill, the track in the progress bar's colour.
Rectangle {
    id: root

    // [{ value, color }], left to right.
    property var parts: []
    // Whether it takes the room a `K.Row` or `K.Stack` gives it.
    property bool fillWidth: true
    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    readonly property var probe: ({
            role: "component",
            name: "Meter",
            variant: "",
            fixed: {
                width: false,
                height: true
            }
        })

    implicitHeight: Tokens.size.progress
    radius: Tokens.radius.full
    color: K.Look.alpha(Tokens.color.surface_container_highest, Tokens.opacity.track)

    // Each part is drawn from the left edge to the end of its share, the last first, so every part
    // keeps the rounded ends of the bar.
    Repeater {
        model: {
            let end = 0;
            return root.parts.map(p => ({
                        end: end = Math.min(1, end + Math.max(0, p.value)),
                        color: p.color
                    })).reverse();
        }

        Rectangle {
            required property var modelData

            width: K.Look.snapUp(root.width * modelData.end, root.dpr)
            height: root.height
            radius: Tokens.radius.full
            color: modelData.color
            visible: width > 0

            // A reading moves the fill over `short`, as a progress bar follows its work.
            Behavior on width {
                NumberAnimation {
                    duration: Tokens.duration.short
                    easing.type: Easing.BezierSpline
                    easing.bezierCurve: Tokens.easing.linear
                }
            }
        }
    }
}
