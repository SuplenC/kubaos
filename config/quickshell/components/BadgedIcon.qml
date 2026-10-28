import QtQuick
import KubaOS.Components as K

// A bar's symbol with a badge centred on its top right corner: a count, or a dot when `count` is
// 0. The badge stands out of the symbol's box without taking room, so the symbols beside it stay
// where they are whether or not it shows.
Item {
    id: root

    property alias name: glyph.name
    property alias color: glyph.color
    property alias size: glyph.size
    property bool badged: false
    property alias count: badge.count

    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1

    implicitWidth: glyph.width
    implicitHeight: glyph.height

    K.Icon {
        id: glyph

        size: K.Tokens.icon.s16
        color: K.Tokens.color.on_surface_variant
    }

    K.Badge {
        id: badge

        x: K.Look.snap(root.width - width / 2, root.dpr)
        y: K.Look.snap(-height / 2, root.dpr)
        visible: root.badged
    }
}
