import QtQuick
import KubaOS.Components as K

// A symbol in a box as wide as itself and as tall as the label's line beside it, centred on a
// whole device pixel, so an icon never changes a button's height.
Item {
    id: root

    property string name
    property int size: Tokens.type.button.icon
    property int line: Tokens.type.button.line
    property color color: Tokens.color.on_surface

    implicitWidth: root.size
    implicitHeight: root.line

    K.Icon {
        id: glyph

        y: K.Look.snap((root.line - root.size) / 2, glyph.dpr)
        name: root.name
        size: root.size
        color: root.color
    }
}
