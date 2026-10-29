import QtQuick
import KubaOS.Components as K

// The gap under a panel's header and the top of the list that scrolls beneath it, laid over the
// list: content scrolled up keeps the gap clear of it and fades into the panel's surface over the
// next 16, instead of being cut flat under the header's text. It shows only while content is
// under it, so at rest a focused first row keeps its whole glow.
Rectangle {
    id: root

    // Whether the list is scrolled away from its top.
    property bool scrolled: false

    // The gap, as tall as the room between the header and the list's first row, then the fade.
    height: K.Tokens.space.s32
    opacity: root.scrolled ? 1 : 0
    gradient: Gradient {
        GradientStop {
            position: 0
            color: K.Tokens.color.surface
        }

        GradientStop {
            position: K.Tokens.space.s16 / root.height
            color: K.Tokens.color.surface
        }

        GradientStop {
            position: 1
            color: K.Look.alpha(K.Tokens.color.surface, 0)
        }
    }

    Behavior on opacity {
        NumberAnimation {
            duration: K.Tokens.duration.micro
            easing.type: Easing.BezierSpline
            easing.bezierCurve: K.Tokens.easing.standard
        }
    }
}
