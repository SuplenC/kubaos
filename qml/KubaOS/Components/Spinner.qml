import QtQuick
import QtQuick.Shapes
import KubaOS.Components as K

// Work that doesn't know how far it is: `kubaos-ui`'s `progress::spinner`. An icon size across, a
// thin arc that turns and breathes over a faint full ring, so its shortest arc still reads as a
// spinner. It turns on the `linear` curve while `running`.
Item {
    id: root

    // An icon size: 16, 20 or 24 beside text; 32 and 48 for a page's one spinner.
    property int size: Tokens.icon.s24
    property color color: Tokens.color.primary
    property bool running: true
    // Seconds into the turn: the clock the arc follows.
    property real seconds: 0

    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    readonly property real stroke: K.Look.thin(root.dpr)
    readonly property real ring: root.size / 2 - root.stroke
    // How long until the arc is where it started: 4 turns at 0.8 a second and 3 breaths at 0.6,
    // so the loop never jumps.
    readonly property real cycle: 5
    readonly property var probe: ({
            role: "component",
            name: "Spinner",
            variant: `${root.size}`,
            fixed: {
                width: true,
                height: true
            }
        })

    implicitWidth: root.size
    implicitHeight: root.size

    NumberAnimation on seconds {
        running: root.running && root.visible
        from: 0
        to: root.cycle
        loops: Animation.Infinite
        // The clock runs in real time: a second of it is a second.
        duration: root.cycle * 1000
        easing.type: Easing.BezierSpline
        easing.bezierCurve: Tokens.easing.linear
    }

    Shape {
        anchors.fill: parent
        preferredRendererType: Shape.CurveRenderer

        ShapePath {
            fillColor: "transparent"
            strokeColor: K.Look.alpha(root.color, Tokens.opacity.spinner_track)
            strokeWidth: root.stroke

            PathAngleArc {
                centerX: root.size / 2
                centerY: root.size / 2
                radiusX: root.ring
                radiusY: root.ring
                startAngle: 0
                sweepAngle: 360
            }
        }

        ShapePath {
            fillColor: "transparent"
            strokeColor: root.color
            strokeWidth: root.stroke
            capStyle: ShapePath.RoundCap

            PathAngleArc {
                centerX: root.size / 2
                centerY: root.size / 2
                radiusX: root.ring
                radiusY: root.ring
                // Material's spinner: 0.8 turns a second, the arc breathing between a quarter and
                // 0.85 of a half turn.
                startAngle: root.seconds * 360 * 0.8
                sweepAngle: 180 * (0.45 + 0.6 * (0.5 + 0.5 * Math.sin(root.seconds * Math.PI * 1.2)))
            }
        }
    }
}
