import QtQuick
import KubaOS.Components as K

// A control's name in a small bubble under it, while the pointer is over it or the keyboard is on
// it: `kubaos-ui`'s `hint::hint`. The inverse colours, `label` type, 8 by 4 inside, the small
// radius, centred under the control past its focus ring and 4 more, on whole device pixels. It is
// no part of the page's layout; leave it room below the control, as the specimen's rows do.
Item {
    id: root

    property string text
    property bool shown: false

    // What a geometry dump leaves out: the bubble floats, as iced's overlay does.
    readonly property bool overlay: true
    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1

    anchors.fill: root.parent
    z: 1

    Rectangle {
        x: K.Look.snap((root.width - width) / 2, root.dpr)
        y: root.height + Tokens.focus.offset + Tokens.space.s4
        width: K.Look.snapUp(label.implicitWidth + Tokens.space.s8 * 2, root.dpr)
        height: K.Look.snapUp(label.implicitHeight + Tokens.space.s4 * 2, root.dpr)
        radius: Tokens.radius.small
        color: Tokens.color.on_surface
        opacity: root.shown ? 1 : 0
        visible: opacity > 0

        Behavior on opacity {
            NumberAnimation {
                duration: Tokens.duration.short
                easing.type: Easing.BezierSpline
                easing.bezierCurve: Tokens.easing.standard
            }
        }

        K.Text {
            id: label

            x: Tokens.space.s8
            y: Tokens.space.s4
            text: root.text
            style: Tokens.type.label
            color: Tokens.color.surface
        }
    }
}
