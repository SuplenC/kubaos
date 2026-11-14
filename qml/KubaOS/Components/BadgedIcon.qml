import QtQuick
import KubaOS.Components as K

// A bar's symbol with a badge centred on its top right corner: a count, or a dot when `count` is
// 0. While it shows, the box takes the half of the badge past the symbol on both sides, so the
// badge keeps the row's gap from the symbol beside it and the symbol stays centred in a pill of
// its own.
Item {
    id: root

    property alias name: glyph.name
    property alias color: glyph.color
    property alias size: glyph.size
    property bool badged: false
    property alias count: badge.count

    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1

    // The room the badge takes past the symbol, on each side.
    readonly property real overhang: root.badged ? K.Look.snapUp(badge.width / 2, root.dpr) : 0

    implicitWidth: glyph.width + root.overhang * 2
    implicitHeight: glyph.height

    K.Icon {
        id: glyph

        x: root.overhang
        size: K.Tokens.icon.s16
        color: K.Tokens.color.on_surface_variant
    }

    K.Badge {
        id: badge

        x: K.Look.snap(glyph.x + glyph.width - width / 2, root.dpr)
        y: K.Look.snap(-height / 2, root.dpr)
        visible: root.badged
    }
}
