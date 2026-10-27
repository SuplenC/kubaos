import QtQuick
import KubaOS.Components as K

// A key drawn as a key, as `K.Keycap` draws it: 20 tall, the `keycap` type on the highest
// container, a hairline edge with a heavier bottom, 4 either side of the key and never narrower
// than tall. The same boxes, placed with token arithmetic on the menu's own `Label`.
//
// HAZARD: no `K.Text` and no layout primitives in here. The footer, the Action panel and the
// detail pane remake their keycaps as the selection moves, on the keystroke's frame; see
// `KeyHints.qml` for what that costs.
Item {
    id: root

    // The key, as it is written on the keyboard: `Esc`, `K`, `↵`.
    property string key

    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1

    implicitWidth: Math.max(K.Tokens.size.keycap, K.Look.snapUp(label.implicitWidth + K.Tokens.space.s4 * 2, root.dpr))
    implicitHeight: K.Tokens.size.keycap
    width: implicitWidth
    height: implicitHeight

    // The edge fills the whole key; the face stands on it one hairline short of the bottom, so the
    // edge shows a hairline at the top and sides and two at the bottom.
    Rectangle {
        anchors.fill: parent
        radius: K.Tokens.radius.small
        color: K.Tokens.color.outline_variant
    }

    Rectangle {
        width: parent.width
        height: parent.height - K.Look.hairline(root.dpr)
        radius: K.Tokens.radius.small
        color: K.Tokens.color.surface_container_highest
        border.width: K.Look.hairline(root.dpr)
        border.color: K.Tokens.color.outline_variant
    }

    Label {
        id: label

        x: K.Look.snap((root.width - label.implicitWidth) / 2, root.dpr)
        height: root.height
        elide: Text.ElideNone
        type: K.Tokens.type.keycap
        text: root.key
        color: K.Tokens.color.on_surface_variant
    }
}
