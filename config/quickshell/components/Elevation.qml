import QtQuick
import QtQuick.Effects
import qs.theme

// Soft drop shadow behind a floating rounded surface. Put it before the
// surface (same parent) and point `target` at it. level 0-5, animatable.
RectangularShadow {
    id: root

    required property Item target
    property real level: 2

    x: target.x
    y: target.y + level * 0.6
    width: target.width
    height: target.height
    radius: target.radius ?? 0
    opacity: target.opacity
    visible: target.visible && level > 0
    blur: Math.pow(level * 6, 0.85)
    spread: -level * 0.4
    color: Theme.alpha(Theme.m3shadow, Theme.elevation.opacity)

    Behavior on level {
        Anim {}
    }
}
