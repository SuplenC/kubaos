import QtQuick
import KubaOS.Components as K
import "samples.js" as Samples

// The components page's "Page scroll" section, drawn with `KubaOS.Components`: the section
// `crates/kubaos-specimen/src/pages/components/page_scroll.rs` draws with `kubaos-ui`, box for box.
// Every kind of button, twice, in a page's scroll a log tall: more than it shows, the first focused
// at its top edge.
K.Section {
    title: "Page scroll"
    description: "A page's or a panel's content taller than its room. It has no edge of its own, so it reaches past its box into the room around it, as far as a glow does and up to the text above it, with its content that far in: the focused button at its top keeps its whole glow, and what runs past that room is cut there."

    K.Grid {
        columns: 3
        gap: K.Tokens.space.s24

        K.Scroll {
            implicitHeight: K.Tokens.size.log
            roomAbove: K.Tokens.space.s16

            K.Stack {
                gap: K.Tokens.space.s8

                Repeater {
                    model: Samples.kinds.concat(Samples.kinds)

                    SampleButton {
                        required property string modelData
                        required property int index

                        kind: modelData
                        size: "Md"
                        // The first is focused, `stateNames`' fourth; the rest are resting.
                        shownState: index === 0 ? 3 : 0
                    }
                }
            }
        }
    }
}
