import QtQuick
import KubaOS.Components as K

// The components page's "Chips" section, drawn with `KubaOS.Components`: the section
// `crates/kubaos-specimen/src/pages/components/chips.rs` draws with `kubaos-ui`, box for box.
K.Section {
    title: "Chips"

    K.Row {
        gap: K.Tokens.space.s8
        align: "centre"

        K.Chip {
            text: "2 TB"
        }

        K.Chip {
            text: "English (US)"
            kind: "Selected"
        }

        K.Chip {
            text: "Recommended"
            kind: "Accent"
        }

        K.Chip {
            text: "Demo"
            kind: "Cold"
        }

        K.Chip {
            text: "Reboot"
            kind: "Filled"
        }

        K.Chip {
            text: "Default"
            kind: "Selected"
            icon: "check"
        }

        K.Chip {
            text: "Demo"
            kind: "Cold"
            icon: "bolt"
        }

        K.Chip {
            text: "Italian"
            pressable: true
            removable: true
        }

        K.Chip {
            text: "English (US)"
            kind: "Selected"
            icon: "check"
            removable: true
        }
    }
}
