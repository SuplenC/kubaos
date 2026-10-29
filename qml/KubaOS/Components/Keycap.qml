import QtQuick
import KubaOS.Components as K

// A key drawn as a key: `kubaos-ui`'s `keycap::keycap`. 20 tall, as a line of body text, so it
// sits inline with body and caption text; the `keycap` type on the highest container, a hairline
// edge with a heavier bottom, the small radius, 4 either side of the key and never narrower than
// tall, so a one-letter key is square.
//
// HAZARD: no layout primitives in here; the key is placed with token arithmetic. The Command
// menu's footer, Action panel and detail pane make their keycaps again as the selection moves, on
// the keystroke's frame.
Item {
    id: root

    // The key, as it is written on the keyboard: `Esc`, `K`, `↵`.
    property string key

    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    readonly property var probe: ({
            role: "component",
            name: "Keycap",
            variant: "",
            fixed: {
                width: false,
                height: true
            }
        })

    implicitWidth: Math.max(Tokens.size.keycap, label.implicitWidth + Tokens.space.s4 * 2)
    implicitHeight: Tokens.size.keycap

    // The edge fills the whole key; the face stands on it one hairline short of the bottom, so
    // the edge shows a hairline at the top and sides and two at the bottom.
    Rectangle {
        anchors.fill: parent
        radius: Tokens.radius.small
        color: Tokens.color.outline_variant
    }

    Rectangle {
        width: parent.width
        height: parent.height - K.Look.hairline(root.dpr)
        radius: Tokens.radius.small
        color: Tokens.color.surface_container_highest
        border.width: K.Look.hairline(root.dpr)
        border.color: Tokens.color.outline_variant
    }

    K.Text {
        id: label

        x: K.Look.snap((root.width - label.implicitWidth) / 2, root.dpr)
        text: root.key
        style: Tokens.type.keycap
        color: Tokens.color.on_surface_variant
    }
}
