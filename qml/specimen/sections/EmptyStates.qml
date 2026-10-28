import QtQuick
import KubaOS.Components as K

// The components page's "Empty states" section, drawn with `KubaOS.Components`: the section
// `crates/kubaos-specimen/src/pages/components/empty_states.rs` draws with `kubaos-ui`, box for box.
K.Section {
    title: "Empty states"

    K.Grid {
        columns: 2
        gap: K.Tokens.space.s24

        K.Card {
            K.EmptyState {
                icon: "wifi"
                title: "No networks"
                body: "Nothing in range. Plug in a cable, or skip it."
                action: Component {
                    K.Button {
                        text: "Skip"
                        kind: "Tonal"
                    }
                }
            }
        }

        K.Card {
            K.EmptyState {
                icon: "search"
                title: "Nothing matches"
                body: "Typo? Try fewer letters."
            }
        }
    }
}
