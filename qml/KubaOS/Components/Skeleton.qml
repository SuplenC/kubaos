import QtQuick
import KubaOS.Components as K

// The shape of what is loading, held in its place until it arrives: `kubaos-ui`'s
// `skeleton::skeleton`. In the placeholder colour a `Media` box holds until its picture loads, so
// what arrives takes the room kept for it and moves nothing. A sheen sweeps across it as it shows,
// so it reads as on its way rather than empty, then it holds still: a wait that goes on draws no
// frames for it. With reduce motion it holds still from the start.
Item {
    id: root

    // `line` (a line of text in `style`), `circle` (an icon `size` across) or `block` (`size`
    // tall, at `radius`).
    property string bone: "line"
    property QtObject style: Tokens.type.body
    property int size: Tokens.icon.s24
    property int radius: Tokens.radius.control
    // Whether the geometry check sees it: a `Media` box's placeholder is the box's own.
    property bool reported: true
    // Whether what it stands in for is on its way. A picture that failed or has no source keeps
    // its placeholder still, so nothing animates for good.
    property bool loading: true

    readonly property bool circle: root.bone === "circle"
    readonly property bool fillWidth: !root.circle
    readonly property var probe: root.reported ? {
        role: "component",
        name: "Skeleton",
        variant: root.bone,
        fixed: {
            width: false,
            height: false
        }
    } : null

    // How far either side of its middle the sheen reaches, as a share of the shape's width.
    readonly property real band: 0.4
    // Where the sheen's middle is across the shape: 0 its left edge, 1 its right. It starts and
    // ends a band clear of the shape, so a still placeholder is the plain colour.
    property real sweep: -root.band

    implicitWidth: root.circle ? root.size : 0
    implicitHeight: root.bone === "line" ? root.style.line : root.size

    // The sheen's stops: where its band starts, peaks and ends, held to the shape.
    readonly property real rise: Math.max(0, Math.min(1, root.sweep - root.band))
    readonly property real peak: Math.max(0, Math.min(1, root.sweep))
    readonly property real fall: Math.max(0, Math.min(1, root.sweep + root.band))
    // The colour at each stop and at the shape's two edges: the placeholder colour, lit toward the
    // highest container as the band nears.
    readonly property color atLeft: K.Look.mix(Tokens.color.surface_container_high, Tokens.color.surface_container_highest, Math.max(0, 1 - Math.abs(root.sweep) / root.band))
    readonly property color atRise: K.Look.mix(Tokens.color.surface_container_high, Tokens.color.surface_container_highest, Math.max(0, 1 - Math.abs(root.rise - root.sweep) / root.band))
    readonly property color atPeak: K.Look.mix(Tokens.color.surface_container_high, Tokens.color.surface_container_highest, Math.max(0, 1 - Math.abs(root.peak - root.sweep) / root.band))
    readonly property color atFall: K.Look.mix(Tokens.color.surface_container_high, Tokens.color.surface_container_highest, Math.max(0, 1 - Math.abs(root.fall - root.sweep) / root.band))
    readonly property color atRight: K.Look.mix(Tokens.color.surface_container_high, Tokens.color.surface_container_highest, Math.max(0, 1 - Math.abs(1 - root.sweep) / root.band))

    // The sheen is the shape's own fill, so its rounded corners cut it as they cut the shape: its
    // stops are the shape's edges and the band's, each in the colour the band gives it there.
    Rectangle {
        id: shape

        anchors.fill: parent
        radius: root.bone === "line" ? Tokens.radius.small : root.circle ? Tokens.radius.full : root.radius
        gradient: Gradient {
            orientation: Gradient.Horizontal

            GradientStop {
                position: 0
                color: root.atLeft
            }

            GradientStop {
                position: root.rise
                color: root.atRise
            }

            GradientStop {
                position: root.peak
                color: root.atPeak
            }

            GradientStop {
                position: root.fall
                color: root.atFall
            }

            GradientStop {
                position: 1
                color: root.atRight
            }
        }
    }

    SequentialAnimation {
        running: root.visible && root.loading && !K.Motion.reduced
        // A few sweeps say it is coming; a scan that takes longer costs no frames.
        loops: 3
        onRunningChanged: {
            if (!running)
                root.sweep = -root.band;
        }

        NumberAnimation {
            target: root
            property: "sweep"
            from: -root.band
            to: 1 + root.band
            duration: Tokens.duration.slow
            easing.type: Easing.BezierSpline
            easing.bezierCurve: Tokens.easing.standard
        }

        PauseAnimation {
            duration: Tokens.duration.slow
        }
    }
}
