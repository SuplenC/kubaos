import QtQuick
import KubaOS.Components as K

// A list's well with nothing in it: one centred line saying so. `kubaos-ui`'s `list::empty_list`.
K.Stack {
    id: root

    property string text

    fill: true
    probe: ({
            role: "component",
            name: "EmptyList",
            variant: "",
            fixed: {
                width: false,
                height: false
            }
        })

    K.List {
        K.Stack {
            probe: null
            centred: true
            topPadding: Tokens.space.s12
            bottomPadding: Tokens.space.s12

            K.Text {
                text: root.text
                style: Tokens.type.body
                color: Tokens.color.on_surface_variant
            }
        }
    }
}
