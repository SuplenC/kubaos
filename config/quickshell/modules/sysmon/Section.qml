import QtQuick
import KubaOS.Components as K

// One reading of the system monitor's panel, in a card of its own so the readings don't run into
// each other: an icon and a title, the headline reading on the right, and what it holds below,
// 4 apart.
K.Card {
    id: root

    property string icon
    property string title
    property string value
    property color valueColor: K.Tokens.color.on_surface
    default property alias body: content.data

    K.Stack {
        fill: true
        gap: K.Tokens.space.s8

        K.Row {
            fill: true
            gap: K.Tokens.space.s8
            align: "centre"

            K.Icon {
                name: root.icon
                size: K.Tokens.type.body_strong.icon
                color: K.Tokens.color.primary
            }

            K.Text {
                text: root.title
                style: K.Tokens.type.body_strong
            }

            K.Fill {
            }

            // The reading ticks, so it is in mono: its width holds still.
            K.Text {
                visible: root.value !== ""
                text: root.value
                style: K.Tokens.type.mono_large
                color: root.valueColor
            }
        }

        K.Stack {
            id: content

            fill: true
            gap: K.Tokens.space.s4
        }
    }
}
