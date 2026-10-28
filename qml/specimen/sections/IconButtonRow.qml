import QtQuick
import KubaOS.Components as K
import "samples.js" as Samples

// One kind of icon button of `size`, in every state, under the label buttons' columns.
K.Grid {
    id: line

    property string kind
    property string size

    weights: Samples.columns
    gap: K.Tokens.space.s16
    centred: true

    K.Text {
        text: line.kind
        style: K.Tokens.type.label
        color: K.Tokens.color.on_surface_variant
    }

    Repeater {
        model: Samples.stateNames.length

        K.IconButton {
            required property int index

            icon: line.kind === "Filled" ? "arrow_forward" : line.kind === "Tonal" ? "settings" : line.kind === "Text" ? "close" : "power_settings_new"
            label: line.kind === "Filled" ? "Continue" : line.kind === "Tonal" ? "Settings" : line.kind === "Text" ? "Close" : "Shut down"
            kind: line.kind
            size: line.size
            preview: index === 1 ? "hovered" : index === 2 ? "pressed" : ""
            focused: index === 3
            enabled: index !== 4
        }
    }
}
