import QtQuick
import KubaOS.Components as K
import "samples.js" as Samples

// The components page's "Buttons" section, drawn with `KubaOS.Components`: the section
// `crates/kubaos-specimen/src/pages/components/buttons.rs` draws with `kubaos-ui`, box for box.
K.Section {
    title: "Buttons"
    description: "Filled, Tonal, Text and Danger, in small, regular and large, each resting, hovered, pressed, focused and disabled."

    K.Stack {
        gap: K.Tokens.space.s32

        Repeater {
            model: Samples.sizes

            K.Stack {
                id: buttonSize

                required property string modelData

                gap: K.Tokens.space.s12

                K.Text {
                    text: Samples.sizeName(buttonSize.modelData, K.ButtonMetrics)
                    style: K.Tokens.type.subheading
                }

                ButtonTable {
                    size: buttonSize.modelData
                }
            }
        }
    }
}
