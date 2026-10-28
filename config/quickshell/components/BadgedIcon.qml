import QtQuick
import KubaOS.Components as K

// A bar's symbol with a badge centred on its top right corner: a count, or a dot when `count` is
// 0. While it shows, the box takes the half of the badge past the symbol, so the badge keeps the
// row's gap from the symbol beside it rather than running into it.
Item {
    id: root

    property alias name: glyph.name
    property alias color: glyph.color
    property alias size: glyph.size
    property bool badged: false
    property alias count: badge.count

    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1

    implicitWidth: glyph.width + (root.badged ? K.Look.snapUp(badge.width / 2, root.dpr) : 0)
    implicitHeight: glyph.height

    K.Icon {
        id: glyph

        size: K.Tokens.icon.s16
        color: K.Tokens.color.on_surface_variant
    }

    K.Badge {
        id: badge

        x: K.Look.snap(glyph.width - width / 2, root.dpr)
        y: K.Look.snap(-height / 2, root.dpr)
        visible: root.badged
    }
}
