import QtQuick
import KubaOS.Components as K
import "panel.js" as Panel

// A line of readings under a section's headline, in mono: the numbers in it tick, and their widths
// hold still. It is one line, ending in an ellipsis past its room. While its reading is on its way
// (`waiting`), a placeholder holds the line's room and the text fades in over it, so nothing under
// it moves.
Item {
    id: root

    property string text
    property color color: K.Tokens.color.on_surface_variant
    property bool waiting: false
    // Whether the text fades in over the placeholder. Not while the panel is still coming in: the
    // text comes in with it.
    property bool fades: true
    // Which line it is, so placeholders one above the other differ in length.
    property int place: 0
    // The widest the line may be, offered by its layout.
    property real room: Infinity

    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    readonly property int fade: K.Motion.reduced ? K.Tokens.duration.short : K.Tokens.duration.slow
    readonly property real placeholderWidth: Number.isFinite(root.room) ? K.Look.snap(root.room * Panel.share(root.place, false), root.dpr) : 0

    implicitWidth: root.waiting ? root.placeholderWidth : label.implicitWidth
    implicitHeight: K.Tokens.type.mono.line

    K.Skeleton {
        width: root.placeholderWidth
        style: K.Tokens.type.mono
        loading: root.waiting
        visible: opacity > 0
        opacity: root.waiting ? 1 : 0

        Behavior on opacity {
            enabled: root.fades

            NumberAnimation {
                duration: root.fade
                easing.type: Easing.BezierSpline
                easing.bezierCurve: K.Tokens.easing.standard
            }
        }
    }

    K.Text {
        id: label

        text: root.text
        style: K.Tokens.type.mono
        color: root.color
        room: root.room
        maximumLines: 1
        visible: opacity > 0
        opacity: root.waiting ? 0 : 1

        Behavior on opacity {
            enabled: root.fades

            NumberAnimation {
                duration: root.fade
                easing.type: Easing.BezierSpline
                easing.bezierCurve: K.Tokens.easing.standard
            }
        }
    }
}
