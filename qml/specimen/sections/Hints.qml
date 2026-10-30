import QtQuick
import KubaOS.Components as K

// The components page's "Hints" section, drawn with `KubaOS.Components`: the section
// `crates/kubaos-specimen/src/pages/components/hints.rs` draws with `kubaos-ui`, box for box.
K.Section {
    title: "Hints"
    description: "A control's name in a small bubble 8 under it, while the pointer or the keyboard is on it: an icon button's label, a chip's tooltip."

    K.Row {
        gap: K.Tokens.space.s32
        align: "centre"

        K.IconButton {
            icon: "settings"
            label: "Settings"
            kind: "Tonal"
            focused: true
        }

        K.Chip {
            text: "Day first"
            pressable: true

            K.Hint {
                text: "Your locale's order"
                shown: true
            }
        }
    }
}
