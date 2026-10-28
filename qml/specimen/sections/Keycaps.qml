import QtQuick
import KubaOS.Components as K

// The components page's "Keycaps and key hints" section, drawn with `KubaOS.Components`: the section
// `crates/kubaos-specimen/src/pages/components/keycaps.rs` draws with `kubaos-ui`, box for box.
K.Section {
    title: "Keycaps and key hints"
    description: "Keys drawn as keys, as tall as a line of body text so they sit inline with it, then what each does."

    K.Stack {
        gap: K.Tokens.space.s16

        K.Row {
            gap: K.Tokens.space.s8
            align: "centre"

            Repeater {
                model: ["Esc", "Tab", "Enter", "↵", "K", "Ctrl", "← →", "+"]

                K.Keycap {
                    required property string modelData

                    key: modelData
                }
            }
        }

        K.KeyHints {
            hints: [["Enter", "continue"], ["Ctrl+K", "actions"], ["Esc", "back"]]
        }

        K.KeyHints {
            text: "↵ Open · Tab Enter app · Ctrl+K Actions"
        }

        K.Row {
            gap: K.Tokens.space.s4
            align: "centre"

            K.Text {
                text: "Press"
                style: K.Tokens.type.body
            }

            K.Keycap {
                key: "Space"
            }

            K.Text {
                text: "to pick a layout, or"
                style: K.Tokens.type.body
            }

            K.Keycap {
                key: "Del"
            }

            K.Text {
                text: "to drop it."
                style: K.Tokens.type.body
            }
        }
    }
}
