import QtQuick
import KubaOS.Components as K

// The line under a field: muted, or the error colour when the value is invalid. `kubaos-ui`'s
// `field::note`.
K.Stack {
    id: root

    property string text
    property bool invalid: false
    // Lines centred on one another, under a centred field (the lock screen's); left otherwise.
    property bool centred: false

    probe: ({
            role: "component",
            name: "Note",
            variant: "",
            fixed: {
                width: false,
                height: false
            }
        })

    K.Text {
        text: root.text
        style: Tokens.type.small
        color: root.invalid ? Tokens.color.error : Tokens.color.on_surface_variant
        horizontalAlignment: root.centred ? Text.AlignHCenter : Text.AlignLeft
    }
}
