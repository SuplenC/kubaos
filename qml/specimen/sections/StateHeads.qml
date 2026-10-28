import QtQuick
import KubaOS.Components as K
import "samples.js" as Samples

// A button table's first row: an empty corner, then the states' names.
Repeater {
    model: [""].concat(Samples.stateNames)

    K.Text {
        required property string modelData

        text: modelData
        style: K.Tokens.type.label
        color: K.Tokens.color.on_surface_variant
    }
}
