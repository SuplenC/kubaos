import QtQuick
import KubaOS.Components as K

// The components page's "Fields" section, drawn with `KubaOS.Components`: the section
// `crates/kubaos-specimen/src/pages/components/fields.rs` draws with `kubaos-ui`, box for box.
K.Section {
    title: "Fields"
    description: "As tall as a regular button. The label's row fits a small button, so fields side by side line up."

    K.Grid {
        columns: 3
        gap: K.Tokens.space.s24

        K.Labelled {
            label: "Full name"

            K.Field {
                placeholder: "Your name"
            }
        }

        K.Labelled {
            label: "Hostname"
            note: "Letters, digits and dashes."

            K.Field {
                text: "tuxedo"
                icon: "computer"
            }
        }

        K.Labelled {
            label: "Search"
            note: "Focused: the caret, and the tube on its own edge."

            K.Field {
                text: "Shinj"
                placeholder: "Search"
                icon: "search"
                focused: true
            }
        }

        K.Labelled {
            label: "Password"
            action: Component {
                K.Button {
                    text: "Show"
                    kind: "Text"
                    size: "Sm"
                    icon: "visibility"
                }
            }

            K.Field {
                text: "hunter2"
                secret: true
            }
        }

        K.Labelled {
            label: "Email for git"
            note: "That's not an email. Typo?"
            invalid: true

            K.Field {
                text: "not-an-email"
                icon: "mail"
                status: "Invalid"
            }
        }

        K.Labelled {
            label: "User name"
            note: "Set when you installed; read-only here."

            K.Field {
                text: "kuba"
                icon: "lock"
                status: "Locked"
            }
        }

        K.Labelled {
            label: "Proxy"
            note: "Disabled: turn the proxy on to set it."

            K.Field {
                placeholder: "proxy.example.com:3128"
                icon: "lan"
                status: "Disabled"
            }
        }
    }
}
