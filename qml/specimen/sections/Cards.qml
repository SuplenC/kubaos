import QtQuick
import KubaOS.Components as K

// The components page's "Cards" section, drawn with `KubaOS.Components`: the section
// `crates/kubaos-specimen/src/pages/components/cards.rs` draws with `kubaos-ui`, box for box.
K.Section {
    title: "Cards"
    description: "Picked cards take the accent, so their focus tube is the cold one. A card that can't be picked now is disabled."

    K.Grid {
        columns: 4
        gap: K.Tokens.space.s24

        K.Card {
            CardText {
                title: "Plain card"
                body: "The well's surface and a soft hairline, 16 inside."
            }
        }

        K.ChoiceCard {
            checked: true

            CardText {
                title: "Erase the disk"
                body: "Everything on it goes."
            }
        }

        K.ChoiceCard {
            CardText {
                title: "Install alongside"
                body: "Keep Windows, shrink it."
            }
        }

        K.ChoiceCard {
            checked: true
            focused: true

            CardText {
                title: "Focused, picked"
                body: "The cold tube."
            }
        }

        K.ChoiceCard {
            enabled: false

            CardText {
                title: "Keep it as it is"
                body: "No room left on this disk."
            }
        }
    }

    // A card's title over a line under it.
    component CardText: K.Stack {
        id: words

        property string title
        property string body

        gap: K.Tokens.space.s4

        K.Text {
            text: words.title
            style: K.Tokens.type.subheading
        }

        K.Text {
            text: words.body
            style: K.Tokens.type.small
            color: K.Tokens.color.on_surface_variant
        }
    }
}
