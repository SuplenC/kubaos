import QtQuick
import KubaOS.Components as K

// The components page's "Prompts" section, drawn with `KubaOS.Components`: the section
// `crates/kubaos-specimen/src/pages/components/prompts.rs` draws with `kubaos-ui`, box for box.
K.Section {
    title: "Prompts"
    description: "One value asked for in a popover, a menu wide: its label and Enter, the field with all of it selected, its note. It floats under the text it changes, or sits inside a menu."

    K.Grid {
        columns: 3
        gap: K.Tokens.space.s24

        K.Prompt {
            label: "Rename"
            text: "Coffee"
            focused: true
        }

        K.Prompt {
            label: "Set value"
            text: "3.5"
            focused: true
            invalid: true
            check: typed => "A whole number, 0 or more."
        }

        K.Popover {
            fixedWidth: K.Tokens.size.menu

            K.Stack {
                fill: true

                K.Menu {
                    entries: [
                        {
                            text: "Back",
                            icon: "arrow_back"
                        }
                    ]
                }

                K.MenuItem {
                    separator: true
                }

                K.Prompt {
                    inMenu: true
                    label: "When it resets"
                    text: "every day at 9"
                    focused: true
                    note: typed => "Resets every day at 9:00."
                }

                K.MenuItem {
                    separator: true
                }

                K.Menu {
                    entries: [
                        {
                            text: "Never",
                            checkable: true,
                            checked: true
                        },
                        {
                            text: "When it reaches",
                            checkable: true
                        }
                    ]
                }
            }
        }
    }
}
