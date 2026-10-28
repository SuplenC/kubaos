import QtQuick
import KubaOS.Components as K

// A key drawn as a key: `kubaos-ui`'s `keycap::keycap`. 20 tall, as a line of body text, so it
// sits inline with body and caption text; the `keycap` type on the highest container, a hairline
// edge with a heavier bottom, the small radius, 4 either side of the key and never narrower than
// tall, so a one-letter key is square.
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

    implicitWidth: Math.max(Tokens.size.keycap, K.Look.snapUp(label.implicitWidth, root.dpr))
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

    K.Row {
        id: label

        x: K.Look.snap((root.width - label.implicitWidth) / 2, root.dpr)
        leftPadding: Tokens.space.s4
        rightPadding: Tokens.space.s4

        K.Text {
            text: root.key
            style: Tokens.type.keycap
            color: Tokens.color.on_surface_variant
        }
    }
}
