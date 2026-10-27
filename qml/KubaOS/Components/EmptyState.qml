import QtQuick
import KubaOS.Components as K

// A place with nothing in it, centred: a 32 icon, 12 below it a `subheading` title, 4 below that
// a `body` line saying what to do, and 16 below that an optional action. Never a bare "No items".
// `kubaos-ui`'s `empty::empty_state`.
K.Stack {
    id: root

    property string icon
    property string title
    property string body
    // The action under the words, or none.
    property Component action: null

    fill: true
    probe: ({
            role: "component",
            name: "EmptyState",
            variant: "",
            fixed: {
                width: false,
                height: false
            }
        })

    K.Stack {
        fill: true

        K.Stack {
            probe: null
            centred: true

            K.Icon {
                name: root.icon
                size: Tokens.icon.s32
                color: Tokens.color.on_surface_variant
            }
        }

        K.Gap {
            space: Tokens.space.s12
        }

        K.Stack {
            probe: null
            centred: true

            K.Text {
                text: root.title
                style: Tokens.type.subheading
                color: Tokens.color.on_surface
            }
        }

        K.Gap {
            space: Tokens.space.s4
        }

        K.Stack {
            probe: null
            centred: true

            K.Text {
                text: root.body
                style: Tokens.type.body
                color: Tokens.color.on_surface_variant
            }
        }

        K.Gap {
            visible: root.action !== null
            space: Tokens.space.s16
        }

        K.Stack {
            visible: root.action !== null
            probe: null
            centred: true

            Loader {
                active: root.action !== null
                sourceComponent: root.action
            }
        }
    }
}
