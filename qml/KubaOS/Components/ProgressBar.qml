import QtQuick
import QtQuick.Effects
import KubaOS.Components as K

// Work that knows how far it is: `kubaos-ui`'s `progress::bar`. 8 tall, a pill, the fill glowing
// in its own colour. The fill follows the work on the `linear` curve over `short`, so it moves
// with the work instead of jumping.
Item {
    id: root

    // How far the work is, 0 to 1.
    property real value: 0

    readonly property bool fillWidth: true
    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    readonly property real shown: Math.max(0, Math.min(1, root.value))
    readonly property var probe: ({
            role: "component",
            name: "ProgressBar",
            variant: "",
            fixed: {
                width: false,
                height: true
            }
        })

    implicitHeight: Tokens.size.progress

    Rectangle {
        anchors.fill: parent
        radius: Tokens.radius.full
        color: K.Look.alpha(Tokens.color.surface_container_highest, Tokens.opacity.track)
    }

    // The row the fill sits in, as `kubaos-ui` lays it out.
    Item {
        readonly property var probe: ({
                role: "layout",
                axis: "x",
                gap: 0,
                align: "start"
            })

        anchors.fill: parent

        RectangularShadow {
            anchors.fill: fill
            visible: fill.visible
            radius: fill.radius
            offset.y: Tokens.shadow.glow.y
            blur: Tokens.shadow.glow.blur
            color: K.Look.alpha(Tokens.color.primary, Tokens.shadow.glow.opacity)
        }

        Rectangle {
            id: fill

            property real fraction: root.shown

            visible: fill.fraction > 0
            width: K.Look.snapUp(root.width * fill.fraction, root.dpr)
            height: root.height
            radius: Tokens.radius.full
            color: Tokens.color.primary

            Behavior on fraction {
                NumberAnimation {
                    duration: Tokens.duration.short
                    easing.type: Easing.BezierSpline
                    easing.bezierCurve: Tokens.easing.linear
                }
            }
        }
    }
}
