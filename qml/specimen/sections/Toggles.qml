import QtQuick
import KubaOS.Components as K

// The components page's "Switches, checkboxes and radio buttons" section, drawn with `KubaOS.Components`: the section
// `crates/kubaos-specimen/src/pages/components/toggles.rs` draws with `kubaos-ui`, box for box.
K.Section {
    title: "Switches, checkboxes and radio buttons"

    K.Stack {
        gap: K.Tokens.space.s24

        K.Row {
            gap: K.Tokens.space.s32
            align: "centre"

            Repeater {
                model: [
                    {
                        label: "Off",
                        on: false,
                        position: 0,
                        focused: false,
                        enabled: true
                    },
                    {
                        label: "On",
                        on: true,
                        position: 1,
                        focused: false,
                        enabled: true
                    },
                    {
                        label: "Halfway",
                        on: true,
                        position: 0.5,
                        focused: false,
                        enabled: true
                    },
                    {
                        label: "Focused",
                        on: true,
                        position: 1,
                        focused: true,
                        enabled: true
                    },
                    {
                        label: "Disabled off",
                        on: false,
                        position: 0,
                        focused: false,
                        enabled: false
                    },
                    {
                        label: "Disabled on",
                        on: true,
                        position: 1,
                        focused: false,
                        enabled: false
                    }
                ]

                K.Stack {
                    id: sample

                    required property var modelData

                    gap: K.Tokens.space.s8

                    K.Switch {
                        checked: sample.modelData.on
                        position: sample.modelData.position
                        focused: sample.modelData.focused
                        enabled: sample.modelData.enabled
                    }

                    K.Text {
                        text: sample.modelData.label
                        style: K.Tokens.type.small
                        color: K.Tokens.color.on_surface_variant
                    }
                }
            }
        }

        K.Row {
            gap: K.Tokens.space.s32
            align: "centre"

            K.Checkbox {
                text: "Bluetooth"
            }

            K.Checkbox {
                text: "Some wallpapers"
                tick: "Partial"
            }

            K.Checkbox {
                text: "Encrypt the disk"
                tick: "Full"
            }

            K.Checkbox {
                text: "Focused"
                tick: "Full"
                focused: true
            }

            K.Checkbox {
                text: "Disabled"
                tick: "Full"
                enabled: false
            }
        }

        K.Row {
            gap: K.Tokens.space.s32
            align: "centre"

            K.Radio {
                text: "Keep Windows"
            }

            K.Radio {
                text: "Erase everything"
                checked: true
            }

            K.Radio {
                text: "Focused"
                focused: true
            }

            K.Radio {
                text: "Disabled"
                checked: true
                enabled: false
            }
        }
    }
}
