import QtQuick
import KubaOS.Components as K

// The components page's "Errors" section, drawn with `KubaOS.Components`: the section
// `crates/kubaos-specimen/src/pages/components/errors.rs` draws with `kubaos-ui`, box for box.
K.Section {
    title: "Errors"
    description: "What broke, what to do next, and the log behind it."

    K.Stack {
        gap: K.Tokens.space.s16

        K.ErrorBlock {
            message: "efibootmgr failed to register the boot entry: Operation not permitted."
            hint: "The firmware refused a new boot entry. Turn off Secure Boot's \"lock boot order\" in the setup menu and try again."
        }

        K.ErrorBlock {
            message: "grub-install --target=x86_64-efi --efi-directory=/boot/efi --bootloader-id=ubuntu failed with exit status 1."
            hint: "A message this long drops to body size."
        }

        K.Log {
            lines: [
                {
                    level: "step",
                    text: "==> Bootloader"
                },
                {
                    level: "info",
                    text: "Installing for x86_64-efi platform."
                },
                {
                    level: "debug",
                    text: "efibootmgr -c -d /dev/nvme0n1 -p 1 -L ubuntu"
                },
                {
                    level: "warn",
                    text: "warning: the firmware reports Secure Boot's boot order as locked"
                },
                {
                    level: "error",
                    text: "grub-install: error: efibootmgr failed to register the boot entry: Operation not permitted."
                }
            ]
        }

        K.Row {
            gap: K.Tokens.space.s12
            align: "centre"

            K.Button {
                text: "Back to the review"
                kind: "Text"
                icon: "arrow_back"
            }

            K.Button {
                text: "Hide log"
                kind: "Text"
                icon: "terminal"
            }

            K.Fill {
            }

            K.Button {
                text: "Save log"
                kind: "Tonal"
                icon: "save"
            }

            K.Button {
                text: "Try again"
                icon: "restart_alt"
                focused: true
            }
        }
    }
}
