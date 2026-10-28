import QtQuick
import KubaOS.Components as K
import "samples.js" as Samples

// Every kind of label button of `size`, in every state.
K.Grid {
    id: table

    property string size

    weights: Samples.columns
    gap: K.Tokens.space.s16
    centred: true

    StateHeads {
    }

    Repeater {
        model: Samples.kinds.length * (Samples.stateNames.length + 1)

        Loader {
            required property int index

            readonly property string kind: Samples.kinds[Math.floor(index / (Samples.stateNames.length + 1))]
            readonly property int shownState: index % (Samples.stateNames.length + 1) - 1

            sourceComponent: shownState < 0 ? kindName : sample

            Component {
                id: kindName

                K.Text {
                    text: kind
                    style: K.Tokens.type.label
                    color: K.Tokens.color.on_surface_variant
                }
            }

            Component {
                id: sample

                SampleButton {
                    kind: parent.kind
                    size: table.size
                    shownState: parent.shownState
                }
            }
        }
    }
}
