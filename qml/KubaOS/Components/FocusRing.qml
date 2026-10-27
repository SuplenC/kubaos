import QtQuick
import KubaOS.Components as K

// The focus ring around its parent, a control whose corners have radius `controlRadius`: outside
// it in a band `Tokens.focus.offset` wide, concentric, its stroke whole device pixels. It takes no
// layout space, so focus moving never moves anything. It fades in and out over `short`, the way
// focus moves from one control to the next.
Rectangle {
    id: root

    // Whether the keyboard is on the control.
    property bool shown: false
    // The control's corner radius.
    property int controlRadius: Tokens.radius.control
    // The cold accent, on cards where the accent already means picked.
    property bool cold: false

    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1

    x: -Tokens.focus.offset
    y: -Tokens.focus.offset
    width: root.parent ? root.parent.width + Tokens.focus.offset * 2 : 0
    height: root.parent ? root.parent.height + Tokens.focus.offset * 2 : 0
    radius: K.Look.ringRadius(root.controlRadius)
    color: "transparent"
    border.color: root.cold ? Tokens.color.cold : Tokens.color.primary
    border.width: K.Look.thin(root.dpr)
    opacity: root.shown ? 1 : 0
    visible: root.opacity > 0

    Behavior on opacity {
        NumberAnimation {
            duration: Tokens.duration.short
            easing.type: Easing.BezierSpline
            easing.bezierCurve: Tokens.easing.standard
        }
    }
}
