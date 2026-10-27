import QtQuick
import KubaOS.Components as K

// A page's header: the Japanese accent, the title, and one line of explanation 8 under it.
// `kubaos-ui`'s `layout::header`.
K.Stack {
    id: root

    property string accent
    property string title
    property string lede

    probe: ({
            role: "component",
            name: "Header",
            variant: "",
            fixed: {
                width: false,
                height: false
            }
        })

    K.Stack {
        K.Text {
            text: root.accent
            style: Tokens.type.accent
            color: Tokens.color.primary
        }

        K.Text {
            text: root.title
            style: Tokens.type.title
            color: Tokens.color.on_surface
        }

        K.Gap {
            space: Tokens.space.s8
        }

        K.Text {
            text: root.lede
            style: Tokens.type.body
            color: Tokens.color.on_surface_variant
        }
    }
}
