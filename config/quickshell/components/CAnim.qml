import QtQuick
import qs.theme

// Color animation on the M3 "effects" curve.
ColorAnimation {
    duration: Theme.anim.small
    easing.type: Easing.BezierSpline
    easing.bezierCurve: Theme.anim.effects
}
