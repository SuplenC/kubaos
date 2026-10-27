import QtQuick
import KubaOS.Components as K

// A titled part of a page: the title, an optional line under it 4 down, then the content 16
// below. `kubaos-ui`'s `layout::section`.
K.Stack {
    id: root

    default property alias content: body.data
    property string title
    property string description

    probe: ({
            role: "component",
            name: "Section",
            variant: "",
            fixed: {
                width: false,
                height: false
            }
        })

    K.Stack {
        id: body

        gap: Tokens.space.s16

        K.Stack {
            gap: Tokens.space.s4

            K.Text {
                text: root.title
                style: Tokens.type.heading
                color: Tokens.color.on_surface
            }

            K.Text {
                visible: root.description !== ""
                text: root.description
                style: Tokens.type.body
                color: Tokens.color.on_surface_variant
            }
        }
    }
}
