import QtQuick
import KubaOS.Components as K

// A bar split into parts, each a 0..1 share and a colour: memory as used and
// cache, a disk as used, a core as busy. What is left is the track. It has a
// progress bar's look: 8 tall, a pill, the track in the progress bar's colour.
Rectangle {
    id: root

    // [{ value, color }], left to right.
    property var parts: []
    // Whether it takes the room a `K.Row` or `K.Stack` gives it.
    property bool fillWidth: false
    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1

    implicitHeight: K.Tokens.size.progress
    radius: K.Tokens.radius.full
    color: K.Look.alpha(K.Tokens.color.surface_container_highest, K.Tokens.opacity.track)

    // Each part is drawn from the left edge to the end of its share, the
    // last first, so every part keeps the rounded ends of the bar.
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
            radius: K.Tokens.radius.full
            color: modelData.color
            visible: width > 0

            // A reading moves the fill over `short`, as a progress bar follows its work.
            Behavior on width {
                NumberAnimation {
                    duration: K.Tokens.duration.short
                    easing.type: Easing.BezierSpline
                    easing.bezierCurve: K.Tokens.easing.linear
                }
            }
        }
    }
}
