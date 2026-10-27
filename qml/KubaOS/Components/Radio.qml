import QtQuick
import KubaOS.Components as K

// A 20 disc with an 8 dot when picked, then its label, all one press target: `kubaos-ui`'s
// `check::radio`. One of a group, exactly one picked; Space, Enter or a click picks it.
K.Pressable {
    id: root

    signal picked

    property string text
    property bool checked: false

    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    readonly property var probe: ({
            role: "component",
            name: "Radio",
            variant: "",
            fixed: {
                width: false,
                height: true
            }
        })

    function dimmed(c: color): color {
        return root.enabled ? c : K.Look.alpha(c, Tokens.opacity.disabled_content);
    }

    implicitWidth: line.implicitWidth
    implicitHeight: line.implicitHeight
    onActivated: root.picked()

    K.Row {
        id: line

        gap: Tokens.space.s12
        align: "centre"

        Item {
            implicitWidth: Tokens.size.check
            implicitHeight: Tokens.size.check

            Rectangle {
                anchors.fill: parent
                radius: Tokens.radius.full
                color: root.dimmed(root.checked ? Tokens.color.primary : Tokens.color.surface_container_highest)
                border.width: root.checked ? 0 : K.Look.hairline(root.dpr)
                border.color: root.dimmed(Tokens.color.outline)
            }

            Rectangle {
                visible: root.checked
                x: K.Look.snap((Tokens.size.check - Tokens.size.radio_dot) / 2, root.dpr)
                y: K.Look.snap((Tokens.size.check - Tokens.size.radio_dot) / 2, root.dpr)
                width: Tokens.size.radio_dot
                height: Tokens.size.radio_dot
                radius: Tokens.radius.full
                color: root.dimmed(Tokens.color.on_primary)
            }

            K.FocusRing {
                shown: root.focused && root.enabled
                controlRadius: Tokens.radius.full
            }
        }

        K.Text {
            text: root.text
            style: Tokens.type.body
            color: root.dimmed(Tokens.color.on_surface)
        }
    }
}
