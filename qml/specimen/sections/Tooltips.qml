import QtQuick
import KubaOS.Components as K

// The components page's "Tooltips" section, drawn with `KubaOS.Components`: the section
// `crates/kubaos-specimen/src/pages/components/tooltips.rs` draws with `kubaos-ui`, box for box.
K.Section {
    title: "Tooltips"
    description: "What a control does, on a popover 8 above it while the pointer or the keyboard is on it; drawn here in place, over the chip it explains."

    K.Stack {
        gap: K.Tokens.space.s8

        K.Tooltip {
            floating: false
            title: "Mac accents on Left Alt"
            body: "Type accents as a Mac does: hold Left Alt with the accent's key, then type the letter."

            K.Stack {
                gap: K.Tokens.space.s8

                K.KeyHints {
                    hints: [["Left Alt+E", "then"], ["E", "types é"]]
                }

                K.KeyHints {
                    hints: [["Left Alt+U", "then"], ["O", "types ö"]]
                }
            }
        }

        K.Chip {
            text: "Mac accents"
            kind: "Selected"
            icon: "keyboard_option_key"
            pressable: true
        }
    }
}
