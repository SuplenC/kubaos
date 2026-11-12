import QtQuick
import KubaOS.Components as K

// The components page's "Rolling numbers" section, drawn with `KubaOS.Components`: the section
// `crates/kubaos-specimen/src/pages/components/rolling_numbers.rs` draws with `kubaos-ui`, box for
// box.
K.Section {
    title: "Rolling numbers"
    description: "A count that rolls only the figures that change, each in a slot a whole device pixel wide, in a box as wide as its widest value."

    K.Row {
        gap: K.Tokens.space.s32
        align: "centre"

        Repeater {
            model: [0, 42, 999, 1000]

            K.RollingNumber {
                required property int modelData

                value: modelData
                widest: "000"
            }
        }

        K.Row {
            gap: K.Tokens.space.s8
            align: "centre"

            K.RollingNumber {
                value: 128
                widest: "000"
            }

            K.Text {
                text: "glasses"
                style: K.Tokens.type.body
                color: K.Tokens.color.on_surface_variant
            }
        }
    }
}
