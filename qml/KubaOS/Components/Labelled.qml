import QtQuick
import KubaOS.Components as K

// A control under its label, with an optional small action at the label's right (a Show toggle,
// a small button) and an optional note under it: `kubaos-ui`'s `field::labelled`. The label's row
// is as tall as a small button whether or not it has one, so fields side by side line up.
K.Stack {
    id: root

    default property alias control: slot.data
    property string label
    // The small action at the label's right, or none.
    property Component action: null
    // The note under the control, or none.
    property string note
    property bool invalid: false

    probe: ({
            role: "component",
            name: "Labelled",
            variant: "",
            fixed: {
                width: false,
                height: false
            }
        })

    K.Stack {
        gap: Tokens.space.s4

        K.Row {
            align: "centre"
            fixedHeight: Tokens.size.control_sm

            K.Text {
                text: root.label
                style: Tokens.type.label
                color: Tokens.color.on_surface_variant
            }

            K.Fill {
            }

            Loader {
                active: root.action !== null
                visible: root.action !== null
                sourceComponent: root.action
            }
        }

        K.Slot {
            id: slot
        }

        K.Note {
            visible: root.note !== ""
            text: root.note
            invalid: root.invalid
        }
    }
}
