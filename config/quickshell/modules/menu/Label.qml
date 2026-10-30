import QtQuick
import KubaOS.Components as K

// Text in a type style for the Results' rows, which are made fresh with every key: a row's title,
// subtitle and Alias. One line cut short with an ellipsis; `textFormat: Text.StyledText` takes
// Kotodama's lit matches.
//
// HAZARD: keep the Results' rows on Qt's own text. Made headless, a keystroke's fourteen fresh
// rows of three texts take 1.1 ms in `K.Text` against 0.65 ms here, and the Results as
// `K.ListRow`s took the menu's keystroke-to-frame median from 9 ms to 14 ms (the e2e `menu` check,
// nine interleaved runs).
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
