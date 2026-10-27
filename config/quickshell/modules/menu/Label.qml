import QtQuick
import KubaOS.Components as K

// Text in a type style that changes with every key or selection: a row's title and subtitle, the
// detail pane, the footer. One line cut short with an ellipsis, or wrapped where a caller sets
// `wrapMode`; `textFormat: Text.StyledText` takes Kotodama's lit matches.
//
// HAZARD: keep this on Qt's own text. `K.Text` measures every word the way iced lays it out, which
// cost 0.23 ms a text against 0.008 ms here; a keystroke's fourteen fresh rows of three texts
// would spend 10 ms on it.
Text {
    id: root

    // The type style, one of `K.Tokens.type`; Text's own `style` is its outline.
    property QtObject type: K.Tokens.type.body

    // Whether the text is shaped: as `K.Text` does, plain ASCII is laid out unkerned and unshaped,
    // so the menu's text sets the same as the library's.
    readonly property bool shaped: /[^\x00-\x7f]/.test(root.text)

    color: K.Tokens.color.on_surface
    elide: Text.ElideRight
    maximumLineCount: 1
    verticalAlignment: Text.AlignVCenter
    lineHeightMode: Text.FixedHeight
    lineHeight: root.type.line
    font.family: root.type.family
    font.weight: root.type.weight
    font.pixelSize: root.type.size
    font.kerning: root.shaped
    font.preferShaping: root.shaped
    font.hintingPreference: Font.PreferNoHinting
}
