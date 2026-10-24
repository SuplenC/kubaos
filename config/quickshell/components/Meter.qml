import QtQuick
import qs.theme

// A rounded bar split into parts, each a 0..1 share and a colour: memory as
// used and cache, a disk as used, a core as busy. What is left is the track.
Rectangle {
    id: root

    // [{ value, color }], left to right.
    property var parts: []

    implicitHeight: Theme.spacing.xs + 2
    radius: height / 2
    color: Theme.m3surfaceContainerHighest

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

            width: root.width * modelData.end
            height: root.height
            radius: root.radius
            color: modelData.color
            visible: width > 0

            Behavior on width {
                Anim {
                    duration: Theme.anim.normal
                    easing.bezierCurve: Theme.anim.standard
                }
            }
        }
    }
}
