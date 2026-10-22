import QtQuick
import qs.theme

// How far a job has got, as Kōbō draws it (`bar` in
// crates/kubaos-kobo/src/widgets.rs): a rounded track filled in the primary
// color. When the Extension can't tell, a short segment sweeps the track.
Rectangle {
    id: bar

    property MenuLook look
    // 0 to 1, or -1 when the Extension can't tell.
    property real progress: -1
    property bool failed: false
    readonly property bool known: progress >= 0 || failed

    implicitHeight: look.barHeight
    radius: height / 2
    color: Theme.m3surfaceContainerHighest
    clip: true

    Rectangle {
        height: bar.height
        radius: bar.radius
        visible: bar.known
        width: bar.failed ? bar.width : Math.max(bar.height, bar.width * Math.min(1, bar.progress))
        color: bar.failed ? Theme.m3error : Theme.m3primary

        Behavior on width {
            enabled: bar.look.motion.moves
            NumberAnimation {
                duration: bar.look.motion.glide
                easing.type: Easing.BezierSpline
                easing.bezierCurve: bar.look.motion.arrive
            }
        }
    }

    Rectangle {
        id: sweep

        height: bar.height
        radius: bar.radius
        width: bar.width * 0.3
        visible: !bar.known
        color: Theme.m3primary
        opacity: bar.look.motion.moves ? 1 : 0.5

        NumberAnimation on x {
            running: sweep.visible && bar.look.motion.moves
            loops: Animation.Infinite
            from: -sweep.width
            to: bar.width
            duration: bar.look.motion.sweep
            easing.type: Easing.InOutQuad
        }
    }
}
