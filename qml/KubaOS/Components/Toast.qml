import QtQuick
import KubaOS.Components as K

// One `body` line saying what just happened, in the inverse colours so it reads over anything,
// an optional 20 icon 12 from the text, 16 by 12 inside, the surface radius, which on one line is
// a pill: `kubaos-ui`'s `toast::toast`. It fades in over `short` while `shown`, and out over
// `medium`; it never carries an action the user must take.
Item {
    id: root

    property string text
    property string icon: ""
    property bool shown: true

    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    readonly property var probe: ({
            role: "component",
            name: "Toast",
            variant: "",
            fixed: {
                width: false,
                height: false
            }
        })

    implicitWidth: K.Look.snapUp(line.implicitWidth + Tokens.space.s16 * 2, root.dpr)
    implicitHeight: K.Look.snapUp(line.implicitHeight + Tokens.space.s12 * 2, root.dpr)
    opacity: root.shown ? 1 : 0
    visible: root.opacity > 0

    // HAZARD: pick the timing from where the fade is going, not from `shown`. The animation starts
    // before bindings on `shown` update, so those would fade out at the fade-in's timing.
    Behavior on opacity {
        id: fade

        NumberAnimation {
            duration: fade.targetValue > 0 ? Tokens.duration.short : Tokens.duration.medium
            easing.type: Easing.BezierSpline
            easing.bezierCurve: fade.targetValue > 0 ? Tokens.easing.standard : Tokens.easing.standard_accel
        }
    }

    Rectangle {
        anchors.fill: parent
        radius: Tokens.radius.surface
        color: Tokens.color.on_surface
    }

    K.Row {
        id: line

        x: Tokens.space.s16
        y: Tokens.space.s12
        gap: Tokens.space.s12
        align: "centre"

        K.Icon {
            visible: root.icon !== ""
            name: root.icon
            size: Tokens.icon.s20
            color: Tokens.color.surface
        }

        K.Text {
            text: root.text
            style: Tokens.type.body
            color: Tokens.color.surface
        }
    }
}
