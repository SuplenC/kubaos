import QtQuick
import KubaOS.Components as K

// The components page's "Toasts and progress" section, drawn with `KubaOS.Components`: the section
// `crates/kubaos-specimen/src/pages/components/progress.rs` draws with `kubaos-ui`, box for box.
K.Section {
    title: "Toasts and progress"

    K.Stack {
        gap: K.Tokens.space.s32

        K.Row {
            gap: K.Tokens.space.s16
            align: "centre"

            K.Toast {
                text: "Copied. Evidence secured."
            }

            K.Toast {
                text: "Replay saved."
                icon: "check_circle"
            }
        }

        K.Row {
            gap: K.Tokens.space.s48

            K.Stack {
                fill: true
                gap: K.Tokens.space.s16

                K.Stack {
                    gap: K.Tokens.space.s16

                    K.ProgressBar {
                        value: 0
                    }

                    K.ProgressBar {
                        value: 0.42
                    }

                    K.ProgressBar {
                        value: 1
                    }
                }

                K.Row {
                    gap: K.Tokens.space.s24
                    align: "centre"

                    Repeater {
                        model: [K.Tokens.icon.s16, K.Tokens.icon.s20, K.Tokens.icon.s24]

                        K.Spinner {
                            required property int modelData

                            size: modelData
                            running: false
                            seconds: 0.7
                        }
                    }

                    K.Text {
                        text: "Poking at the hardware…"
                        color: K.Tokens.color.on_surface_variant
                    }
                }
            }

            K.Stack {
                fill: true

                K.Stack {
                    gap: K.Tokens.space.s12

                    K.Row {
                        gap: K.Tokens.space.s12
                        align: "end"

                        K.Text {
                            text: "Installing"
                            style: K.Tokens.type.title
                        }

                        K.Fill {
                        }

                        K.Text {
                            text: "42%"
                            style: K.Tokens.type.hero_number
                        }
                    }

                    K.ProgressBar {
                        value: 0.42
                    }

                    K.Row {
                        gap: K.Tokens.space.s12
                        align: "centre"

                        K.Text {
                            text: "Unpacking the system"
                            style: K.Tokens.type.body_strong
                        }

                        K.Text {
                            text: "1.8 / 4.2 GB"
                            style: K.Tokens.type.mono
                            color: K.Tokens.color.primary
                        }

                        K.Fill {
                        }

                        K.Text {
                            text: "01:42"
                            style: K.Tokens.type.mono
                            color: K.Tokens.color.on_surface_variant
                        }
                    }
                }
            }
        }
    }
}
