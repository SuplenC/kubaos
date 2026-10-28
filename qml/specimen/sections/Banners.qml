import QtQuick
import KubaOS.Components as K

// The components page's "Banners" section, drawn with `KubaOS.Components`: the section
// `crates/kubaos-specimen/src/pages/components/banners.rs` draws with `kubaos-ui`, box for box.
K.Section {
    title: "Banners"
    description: "A message that matters on the page itself, its actions inside: info, a warning, and danger, really red, holding the button that destroys."

    K.Stack {
        fill: true
        gap: K.Tokens.space.s16

        K.Banner {
            tone: "Info"
            icon: "storage"
            title: "KubaOS goes into the free space on Samsung SSD 990 PRO."
            body: "Windows stays where it is."
        }

        K.Banner {
            tone: "Warning"
            icon: "warning"
            title: "12% left. Plug the damn thing in."
            body: "An install on battery may not finish."
        }

        K.Banner {
            tone: "Danger"
            icon: "warning"
            title: "Everything on Samsung SSD 990 PRO (nvme0n1, 2 TB) gets erased."
            body: "All 4 partitions, Windows included. There's no undo."
            foot: [
                K.ProgressBar {
                    value: 0.4
                    danger: true
                }
            ]

            K.Button {
                text: "Back"
                kind: "Text"
                icon: "arrow_back"
            }

            K.Stack {
                gap: K.Tokens.space.s8

                K.Button {
                    text: "Hold to erase and install"
                    kind: "Danger"
                    icon: "download"
                }

                K.Row {
                    gap: K.Tokens.space.s4
                    align: "centre"

                    K.Text {
                        text: "or hold"
                        style: K.Tokens.type.caption
                        color: K.Look.alpha(K.Tokens.color.on_error_container, K.Tokens.opacity.subdued_text)
                    }

                    K.Keycap {
                        key: "Enter"
                    }
                }
            }
        }
    }
}
