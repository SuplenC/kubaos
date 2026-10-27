import QtQuick

// The state layer of a pressable surface: its content colour laid over its fill while hovered
// (`Tokens.opacity.hover`) or pressed (`Tokens.opacity.press`), fading in and out over `micro`.
// Fills its parent, at the corners it is given.
Rectangle {
    id: root

    property bool hovered: false
    property bool pressed: false
    property bool active: true
    // The colour laid over: the control's content colour.
    property color content: Tokens.color.on_surface

    anchors.fill: root.parent
    color: root.content
    opacity: !root.active ? 0 : root.pressed ? Tokens.opacity.press : root.hovered ? Tokens.opacity.hover : 0
    visible: root.opacity > 0

    Behavior on opacity {
        NumberAnimation {
            duration: Tokens.duration.micro
            easing.type: Easing.BezierSpline
            easing.bezierCurve: Tokens.easing.standard
        }
    }
}
