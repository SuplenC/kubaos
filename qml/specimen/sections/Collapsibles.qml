import QtQuick
import KubaOS.Components as K

// The components page's "Collapsibles" section, drawn with `KubaOS.Components`: the section
// `crates/kubaos-specimen/src/pages/components/collapsibles.rs` draws with `kubaos-ui`, box for box.
K.Section {
    title: "Collapsibles"
    description: "A titled part that opens to show more; the last is disabled."

    K.Grid {
        columns: 2
        gap: K.Tokens.space.s24

        K.Collapsible {
            title: "firefox · pid 4127"
            open: true

            K.Stack {
                gap: K.Tokens.space.s4

                K.Text {
                    text: "/usr/lib/firefox/firefox -contentproc"
                    style: K.Tokens.type.mono
                    color: K.Tokens.color.on_surface_variant
                }

                K.Text {
                    text: "kuba · sleeping · 92 threads · 2.1 GB"
                    style: K.Tokens.type.small
                    color: K.Tokens.color.on_surface_variant
                }
            }
        }

        K.Collapsible {
            title: "Hyprland · pid 1203"
            focused: true

            K.Text {
                text: "Hidden while closed."
                style: K.Tokens.type.small
                color: K.Tokens.color.on_surface_variant
            }
        }

        K.Collapsible {
            title: "systemd · pid 1"
            enabled: false

            K.Text {
                text: "Hidden while closed."
                style: K.Tokens.type.small
                color: K.Tokens.color.on_surface_variant
            }
        }
    }
}
