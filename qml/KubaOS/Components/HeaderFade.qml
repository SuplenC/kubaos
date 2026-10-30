import QtQuick
import KubaOS.Components as K

// The gap under a panel's header (or the Command menu's field) and the top of the list or form
// that scrolls beneath it, laid over it: content scrolled up keeps the gap clear of it and fades
// into the panel's surface over the next 16, instead of being cut flat under the header's text. It
// shows only while content is under it, so at rest a focused first row keeps its whole glow.
//
// Give it the box of the scroll under it (its `x` and `width`): it spans that box but for the
// scrollbar's width at its right edge, so the thumb is never covered, whether or not the scroll
// shows one now.
Item {
    id: root

    // Whether the list is scrolled away from its top.
    property bool scrolled: false
    // The room between the header and the list's first row, kept clear: a panel's step, or the
    // Command menu's padding under its field.
    property int gap: K.Tokens.space.s16

    // The gap, then the fade.
    height: root.gap + K.Tokens.space.s16
    opacity: root.scrolled ? 1 : 0

    Behavior on opacity {
        NumberAnimation {
            duration: K.Tokens.duration.micro
            easing.type: Easing.BezierSpline
            easing.bezierCurve: K.Tokens.easing.standard
        }
    }

    Rectangle {
        width: Math.max(0, root.width - K.Tokens.size.scrollbar)
        height: root.height
        gradient: Gradient {
            GradientStop {
                position: 0
                color: K.Tokens.color.surface
            }

            GradientStop {
                position: root.gap / root.height
                color: K.Tokens.color.surface
            }

            GradientStop {
                position: 1
                color: K.Look.alpha(K.Tokens.color.surface, 0)
            }
        }
    }
}
