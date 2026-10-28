import QtQuick
import KubaOS.Components as K

// The components page's "Dialogs" section, drawn with `KubaOS.Components`: the section
// `crates/kubaos-specimen/src/pages/components/dialogs.rs` draws with `kubaos-ui`, box for box.
K.Section {
    title: "Dialogs"
    description: "A question, the confirmation before something is destroyed, and a prompt for what only you can type."

    K.Grid {
        columns: 2
        gap: K.Tokens.space.s24

        K.Scrim {
            implicitHeight: K.Tokens.size.column / 2 + K.Tokens.space.s64

            K.Dialog {
                // A picture of a dialog: it leaves the keyboard where it is.
                takesFocus: false
                icon: "power_settings_new"
                title: "Restart now?"
                body: "Everything open closes. I'll be back in a few seconds."
                dismiss: K.Button {
                    text: "Cancel"
                    kind: "Text"
                }
                actions: [
                    K.Button {
                        text: "Shut down"
                        kind: "Tonal"
                        icon: "power_settings_new"
                    },
                    K.Button {
                        text: "Restart"
                        icon: "restart_alt"
                        focused: true
                    }
                ]
            }
        }

        K.Scrim {
            implicitHeight: K.Tokens.size.column / 2 + K.Tokens.space.s64

            K.Dialog {
                // A picture of a dialog: it leaves the keyboard where it is.
                takesFocus: false
                tone: "Danger"
                icon: "warning"
                title: "Last chance"
                body: "Everything on Samsung SSD 990 PRO (nvme0n1, 2 TB) gets erased. There's no undo."
                dismiss: K.Button {
                    text: "Cancel"
                    kind: "Text"
                    focused: true
                }
                actions: [
                    K.Button {
                        text: "Erase and install"
                        kind: "Danger"
                        icon: "warning"
                    }
                ]
            }
        }

        K.Scrim {
            implicitHeight: K.Tokens.size.column / 2 + K.Tokens.space.s64

            K.Dialog {
                // A picture of a dialog: it leaves the keyboard where it is.
                takesFocus: false
                icon: "admin_panel_settings"
                title: "Authentication required"
                body: "Installing btop needs your password."
                dismiss: K.Button {
                    text: "Cancel"
                    kind: "Text"
                }
                actions: [
                    K.Button {
                        text: "Authenticate"
                    }
                ]

                K.Labelled {
                    label: "Password for Kuba"
                    note: "That didn't work. Fat fingers? Try again."
                    invalid: true

                    K.Field {
                        text: "hunter"
                        icon: "key"
                        secret: true
                        status: "Invalid"
                    }
                }
            }
        }
    }
}
