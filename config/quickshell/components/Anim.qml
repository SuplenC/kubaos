import QtQuick
import qs.theme

// Number animation with the shell's defaults: standard curve, normal duration.
// Override duration/easing.bezierCurve with Theme.anim tokens where needed.
NumberAnimation {
    duration: Theme.anim.normal
    easing.type: Easing.BezierSpline
    easing.bezierCurve: Theme.anim.standard
}
