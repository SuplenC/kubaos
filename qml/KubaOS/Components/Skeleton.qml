import QtQuick
import KubaOS.Components as K

// The shape of what is loading, held in its place until it arrives: `kubaos-ui`'s
// `skeleton::skeleton`. In the placeholder colour a `Media` box holds until its picture loads, so
// what arrives takes the room kept for it and moves nothing. It breathes as it shows, so it reads
// as on its way rather than empty, then holds still: a wait that goes on draws no frames for it.
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

    implicitWidth: root.circle ? root.size : 0
    implicitHeight: root.bone === "line" ? root.style.line : root.size

    Rectangle {
        id: shape

        anchors.fill: parent
        radius: root.bone === "line" ? Tokens.radius.small : root.circle ? Tokens.radius.full : root.radius
        color: Tokens.color.surface_container_high

        SequentialAnimation on opacity {
            running: root.visible && root.loading
            // A few breaths say it is coming; a scan that takes longer costs no frames.
            loops: 3
            // Back to full when it stops, so a still placeholder is the plain colour.
            onRunningChanged: {
                if (!running)
                    shape.opacity = 1;
            }

            NumberAnimation {
                to: Tokens.opacity.subdued
                duration: Tokens.duration.slow
                easing.type: Easing.BezierSpline
                easing.bezierCurve: Tokens.easing.standard
            }

            NumberAnimation {
                to: 1
                duration: Tokens.duration.slow
                easing.type: Easing.BezierSpline
                easing.bezierCurve: Tokens.easing.standard
            }
        }
    }
}
