import QtQuick
import KubaOS.Components as K

// Facts beside their labels, as a summary reads them: `kubaos-ui`'s
// `description::description_list`. One under the other, 8 apart: each label in `small` a quarter
// of the width, its value in `body` the rest, wrapping, and an optional mono line under the value
// for a path, an email or a version, which break badly wrapped mid-token.
Item {
    id: root

    // `[{ label, value, sub }]`, `sub` optional.
    property var facts: []
    // The widest it may grow, offered by its parent.
    property alias room: column.room

    readonly property bool fillWidth: true
    readonly property var probe: ({
            role: "component",
            name: "DescriptionList",
            variant: "",
            fixed: {
                width: false,
                height: false
            }
        })

    implicitHeight: column.implicitHeight

    function relayout(): void {
        column.relayout();
    }

    K.Stack {
        id: column

        width: root.width
        fill: true
        gap: Tokens.space.s8

        Repeater {
            model: root.facts

            K.Grid {
                id: fact

                required property var modelData

                weights: [1, 3]
                gap: Tokens.space.s8

                K.Text {
                    text: fact.modelData.label
                    style: Tokens.type.small
                    color: Tokens.color.on_surface_variant
                }

                K.Stack {
                    gap: Tokens.space.s4

                    K.Text {
                        text: fact.modelData.value
                        style: Tokens.type.body
                    }

                    K.Text {
                        visible: !!fact.modelData.sub
                        text: fact.modelData.sub ?? ""
                        style: Tokens.type.mono
                        color: Tokens.color.on_surface_variant
                        maximumLines: 1
                    }
                }
            }
        }
    }
}
