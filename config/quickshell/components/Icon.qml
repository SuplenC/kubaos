import QtQuick
import qs.theme

// Material Symbols Rounded glyph, addressed by ligature name ("wifi",
// "volume_up", ...). Browse names at https://fonts.google.com/icons.
// `filled` animates the font's FILL axis, so toggles morph instead of swap.
Text {
    id: root

    property string name
    property real size: 20
    property bool filled: false
    property real fill: filled ? 1 : 0

    text: name
    color: Theme.m3onSurface
    font.family: Theme.font.icons
    font.pixelSize: size
    font.variableAxes: ({
            FILL: root.fill,
            opsz: Math.max(20, Math.min(48, root.size)),
            wght: 400
        })
    textFormat: Text.PlainText
    renderType: Text.NativeRendering
    horizontalAlignment: Text.AlignHCenter
    verticalAlignment: Text.AlignVCenter

    Behavior on fill {
        Anim {
            duration: Theme.anim.small
        }
    }
}
