import QtQuick
import KubaOS.Components as K

// A list's well as tall as its parent makes it, holding a scrolling view: `K.List`'s look round a
// `NavList`, which lays out and scrolls rows the well alone can't. The ring goes round the well
// while `focused`.
K.List {
    id: root

    default property alias view: holder.data

    rows: Item {
        id: holder

        readonly property bool fillWidth: true

        implicitHeight: Math.max(0, root.height - K.Tokens.space.s12 * 2)
    }
}
