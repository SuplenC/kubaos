import QtQuick
import KubaOS.Components as K

// The components page's "Steps" section, drawn with `KubaOS.Components`: the section
// `crates/kubaos-specimen/src/pages/components/steps.rs` draws with `kubaos-ui`, box for box.
K.Section {
    title: "Steps"
    description: "To do, done and waiting on an action each look their own, and the action carries the accent. A description wraps; a rail marks the state instead."

    K.Split {
        weights: [2, 1]
        gap: K.Tokens.space.s24

        K.List {
            focused: true

            K.Step {
                title: "TUXEDO drivers"
                subtitle: "Key queued. Reboot and enroll it on the blue MokManager screen, before KubaOS starts."
                icon: "computer"
                status: "action"
                actionLabel: "Reboot"
                actionIcon: "restart_alt"
                selected: true
            }

            K.Step {
                title: "Fingerprint"
                subtitle: "One finger: right index. sudo and password dialogs take it."
                icon: "fingerprint"
                status: "done"
            }

            K.Step {
                title: "Bitwarden"
                subtitle: "Not logged in."
                icon: "lock"
            }
        }

        K.Stack {
            K.Step {
                title: "Language"
                icon: "language"
                status: "done"
                mark: true
            }

            K.Step {
                title: "Disk"
                icon: "hard_drive"
                status: "current"
                mark: true
                enabled: false
            }

            K.Step {
                title: "Git"
                icon: "code"
                mark: true
                enabled: false
            }

            K.Step {
                title: "Reboot"
                icon: "computer"
                status: "action"
                actionLabel: "Reboot"
                actionIcon: "restart_alt"
                mark: true
                enabled: false
            }
        }
    }
}
