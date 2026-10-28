import QtQuick
import KubaOS.Components as K

// The components page's "Skeletons" section, drawn with `KubaOS.Components`: the section
// `crates/kubaos-specimen/src/pages/components/skeletons.rs` draws with `kubaos-ui`, box for box.
K.Section {
    title: "Skeletons"
    description: "The shape of what is loading, in the placeholder colour, so what arrives moves nothing: a row being scanned, a thumbnail being decoded."

    K.Grid {
        columns: 2
        gap: K.Tokens.space.s24

        K.Row {
            gap: K.Tokens.space.s12
            align: "centre"

            K.Skeleton {
                bone: "circle"
                size: K.Tokens.icon.s24
            }

            K.Stack {
                fill: true
                gap: K.Tokens.space.s4

                K.Skeleton {
                    style: K.Tokens.type.body_strong
                }

                K.Skeleton {
                    style: K.Tokens.type.small
                }
            }
        }

        K.Skeleton {
            bone: "block"
            size: K.Tokens.size.control_lg
        }
    }
}
