import QtQuick
import KubaOS.Components as K

// An app's intro card, before anything is typed in it: what the Extension says now (why it can't
// do anything, say), what the app does, and a few queries with what they give.
K.Card {
    id: card

    property string note: ""
    property string about: ""
    // Each `{query, result}`.
    property var examples: []

    K.Stack {
        gap: K.Tokens.space.s8

        K.Text {
            visible: card.note !== ""
            text: card.note
            style: K.Tokens.type.body
            color: K.Tokens.color.primary
        }

        K.Text {
            text: card.about
            style: K.Tokens.type.body
        }

        Repeater {
            model: card.examples

            // A query as typed, then what it gives.
            K.Row {
                id: example

                required property var modelData

                gap: K.Tokens.space.s8
                align: "centre"

                K.Text {
                    text: example.modelData.query
                    style: K.Tokens.type.mono
                }

                K.Icon {
                    name: "arrow_forward"
                    size: K.Tokens.icon.s16
                    color: K.Tokens.color.on_surface_variant
                }

                K.Text {
                    text: example.modelData.result
                    style: K.Tokens.type.small
                    color: K.Tokens.color.on_surface_variant
                }
            }
        }
    }
}
