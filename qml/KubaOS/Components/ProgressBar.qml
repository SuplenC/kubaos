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
    // On a danger banner's red: the fill in the error colour on a pale track, so the work that
    // destroys something reads as red all the way.
    property bool danger: false
    readonly property color fillColour: root.danger ? Tokens.color.error : Tokens.color.primary

    readonly property bool fillWidth: true
    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    readonly property real shown: Math.max(0, Math.min(1, root.value))
    readonly property var probe: ({
            role: "component",
            name: "ProgressBar",
            variant: root.danger ? "danger" : "",
            fixed: {
                width: false,
                height: true
            }
        })

    implicitHeight: Tokens.size.progress

    Rectangle {
        anchors.fill: parent
        radius: Tokens.radius.full
        color: root.danger ? K.Look.alpha(Tokens.color.on_error_container, Tokens.opacity.tint) : K.Look.alpha(Tokens.color.surface_container_highest, Tokens.opacity.track)
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
            color: K.Look.alpha(root.fillColour, Tokens.shadow.glow.opacity)
        }

        Rectangle {
            id: fill

            property real fraction: root.shown

            visible: fill.fraction > 0
            width: K.Look.snapUp(root.width * fill.fraction, root.dpr)
            height: root.height
            radius: Tokens.radius.full
            color: root.fillColour

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
