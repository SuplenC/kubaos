import QtQuick
import KubaOS.Components as K
import "samples.js" as Samples

// The components page's "Icon buttons" section, drawn with `KubaOS.Components`: the section
// `crates/kubaos-specimen/src/pages/components/icon_buttons.rs` draws with `kubaos-ui`, box for box.
K.Section {
    title: "Icon buttons"
    description: "Circles as tall as the label buttons of each size. The label is required: it shows in a hint on hover and while focused."

    K.Stack {
        gap: K.Tokens.space.s32

        Repeater {
            model: Samples.sizes

            K.Stack {
                id: iconSize

                required property string modelData

                gap: K.Tokens.space.s12

                K.Text {
                    text: Samples.sizeName(iconSize.modelData, K.ButtonMetrics)
                    style: K.Tokens.type.subheading
                }

                // Rows 40 apart, so a focused button's hint has room under it.
                K.Stack {
                    gap: K.Tokens.space.s40

                    K.Grid {
                        weights: Samples.columns
                        gap: K.Tokens.space.s16
                        centred: true

                        StateHeads {
                        }
                    }

                    Repeater {
                        model: Samples.kinds

                        IconButtonRow {
                            required property string modelData

                            kind: modelData
                            size: iconSize.modelData
                        }
                    }
                }
            }
        }
    }
}
