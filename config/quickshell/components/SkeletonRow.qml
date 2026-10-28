import QtQuick
import KubaOS.Components as K

// A two-line list row still being found: a tiled `K.ListRow`'s box, its icon tile and its two
// lines as skeletons, so a scan's first results take the place the skeletons kept and nothing moves.
// `share` is how much of the row the lines take, so rows side by side don't read as one block.
Item {
    id: root

    property real share: 0.6

    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    readonly property real words: root.width - K.Tokens.space.s12 * 3 - K.Tokens.size.control_md

    implicitHeight: K.Tokens.size.row_two_line

    K.Skeleton {
        x: K.Tokens.space.s12
        y: K.Look.snap((root.height - height) / 2, root.dpr)
        width: K.Tokens.size.control_md
        bone: "block"
        size: K.Tokens.size.control_md
        radius: K.Tokens.radius.control
    }

    Column {
        x: K.Tokens.space.s12 * 2 + K.Tokens.size.control_md
        y: K.Look.snap((root.height - height) / 2, root.dpr)
        spacing: K.Tokens.space.s4

        K.Skeleton {
            width: K.Look.snap(root.words * root.share, root.dpr)
            style: K.Tokens.type.body_strong
        }

        K.Skeleton {
            width: K.Look.snap(root.words * root.share * root.share, root.dpr)
            style: K.Tokens.type.small
        }
    }
}
