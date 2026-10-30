import QtQuick
import KubaOS.Components as K

// The components page's "Split, Flow and Gap" section, drawn with `KubaOS.Components`: the section
// `crates/kubaos-specimen/src/pages/components/layouts.rs` draws with `kubaos-ui`, box for box.
K.Section {
    title: "Split, Flow and Gap"
    description: "Columns by their share of the width, a label a quarter wide beside its value; chips flowing onto a new line where the next doesn't fit; and one larger step in a stack."

    K.Grid {
        columns: 3
        gap: K.Tokens.space.s24

        K.Split {
            weights: [1, 3]
            gap: K.Tokens.space.s8
            centred: true

            K.Text {
                text: "Hostname"
                style: K.Tokens.type.small
                color: K.Tokens.color.on_surface_variant
            }

            K.Text {
                text: "tuxedo"
                style: K.Tokens.type.body_strong
            }

            K.Text {
                text: "Kernel"
                style: K.Tokens.type.small
                color: K.Tokens.color.on_surface_variant
            }

            K.Text {
                text: "6.17.0-42"
                style: K.Tokens.type.body_strong
            }

            K.Text {
                text: "Uptime"
                style: K.Tokens.type.small
                color: K.Tokens.color.on_surface_variant
            }

            K.Text {
                text: "3 days"
                style: K.Tokens.type.body_strong
            }
        }

        K.Flow {
            gap: K.Tokens.space.s8

            Repeater {
                model: ["English (US)", "Italian", "German", "Japanese", "Polish", "French", "Spanish", "Portuguese (Brazil)"]

                K.Chip {
                    required property string modelData

                    text: modelData
                }
            }
        }

        K.Stack {
            K.Text {
                text: "A title"
                style: K.Tokens.type.body_strong
            }

            K.Gap {
                space: K.Tokens.space.s16
            }

            K.Text {
                text: "16 under it, with no gap of the stack's own."
                style: K.Tokens.type.small
                color: K.Tokens.color.on_surface_variant
            }
        }
    }
}
