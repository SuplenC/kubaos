import QtQuick
import KubaOS.Components as K

// The components page's "Notes" section, drawn with `KubaOS.Components`: the section
// `crates/kubaos-specimen/src/pages/components/notes.rs` draws with `kubaos-ui`, box for box.
K.Section {
    title: "Notes"
    description: "The small line under a field: muted, or the error colour when the value is wrong."

    K.Stack {
        gap: K.Tokens.space.s8

        K.Note {
            text: "Letters, digits and dashes."
        }

        K.Note {
            text: "That's not an email. Typo?"
            invalid: true
        }
    }
}
