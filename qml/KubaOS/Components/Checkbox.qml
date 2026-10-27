import QtQuick
import KubaOS.Components as K

// A square, 20, at the small radius, then its label, all one press target: `kubaos-ui`'s
// `check::checkbox`. Empty, partial (a dash, for some of a group) or full. For several
// independent picks that apply together; Space, Enter or a click toggles it.
K.Pressable {
    id: root

    signal toggled

    property string text
    // `Empty`, `Partial` or `Full`.
    property string tick: "Empty"

    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    readonly property bool empty: root.tick === "Empty"
    readonly property var probe: ({
            role: "component",
            name: "Checkbox",
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
    onActivated: root.toggled()

    K.Row {
        id: line

        gap: Tokens.space.s12
        align: "centre"

        Item {
            implicitWidth: Tokens.size.check
            implicitHeight: Tokens.size.check

            Rectangle {
                anchors.fill: parent
                radius: Tokens.radius.small
                color: root.dimmed(root.empty ? Tokens.color.surface_container_highest : Tokens.color.primary)
                border.width: root.empty ? K.Look.hairline(root.dpr) : 0
                border.color: root.dimmed(Tokens.color.outline)
            }

            K.Icon {
                visible: root.tick === "Full"
                x: K.Look.snap((Tokens.size.check - Tokens.icon.s16) / 2, root.dpr)
                y: K.Look.snap((Tokens.size.check - Tokens.icon.s16) / 2, root.dpr)
                name: "check"
                size: Tokens.icon.s16
                color: root.dimmed(Tokens.color.on_primary)
            }

            // A partial tick: two thirds of the box wide, a thin stroke tall, on whole device
            // pixels.
            Rectangle {
                visible: root.tick === "Partial"
                width: Tokens.size.check - Tokens.space.s4 * 2
                height: K.Look.thin(root.dpr)
                x: Tokens.space.s4
                y: K.Look.snap((Tokens.size.check - height) / 2, root.dpr)
                color: root.dimmed(Tokens.color.on_primary)
            }

            K.FocusRing {
                shown: root.focused && root.enabled
                controlRadius: Tokens.radius.small
            }
        }

        K.Text {
            text: root.text
            style: Tokens.type.body
            color: root.dimmed(Tokens.color.on_surface)
        }
    }
}
