import QtQuick
import KubaOS.Components as K

// The components page's "Sliders" section, drawn with `KubaOS.Components`: the section
// `crates/kubaos-specimen/src/pages/components/sliders.rs` draws with `kubaos-ui`, box for box.
K.Section {
    title: "Sliders"
    description: "A pill whose fill is the value: its round end marks it. Steps show on both sides, and the mark the value rests on turns solid. Regular above, compact below."

    K.Stack {
        gap: K.Tokens.space.s32

        Repeater {
            model: ["Regular", "Compact"]

            K.Grid {
                id: sizes

                required property string modelData

                columns: 3
                gap: K.Tokens.space.s32

                Repeater {
                    model: [
                        {
                            label: "Volume, 62%",
                            icon: "volume_up",
                            value: 0.62
                        },
                        {
                            label: "Under the pointer",
                            icon: "volume_up",
                            value: 0.62,
                            preview: "hovered"
                        },
                        {
                            label: "Held: the fill lights up",
                            icon: "volume_up",
                            value: 0.62,
                            preview: "pressed"
                        },
                        {
                            label: "Muted: the icon mutes",
                            icon: "volume_off",
                            value: 0,
                            words: "Muted"
                        },
                        {
                            label: "Auto: the sensor's level",
                            icon: "motion_photos_auto",
                            value: 0.55,
                            words: "Auto · 55%",
                            soft: true
                        },
                        {
                            label: "Five steps, resting on one",
                            icon: "balance",
                            stops: 5,
                            index: 2,
                            names: ["Super Saver", "Power Saver", "Balanced", "Performance", "Max Performance"]
                        },
                        {
                            label: "Sticky at 100% of 150%",
                            icon: "volume_up",
                            value: 2 / 3,
                            sticky: 2 / 3,
                            number: "100%",
                            widest: "150%"
                        },
                        {
                            label: "Focused",
                            icon: "volume_up",
                            value: 0.3,
                            focused: true
                        },
                        {
                            label: "Disabled",
                            icon: "volume_up",
                            value: 0.3,
                            enabled: false
                        }
                    ]

                    K.Stack {
                        id: slid

                        required property var modelData

                        gap: K.Tokens.space.s8

                        K.Slider {
                            size: sizes.modelData
                            icon: slid.modelData.icon
                            value: slid.modelData.value ?? 0
                            stops: slid.modelData.stops ?? 0
                            index: slid.modelData.index ?? 0
                            names: slid.modelData.names ?? []
                            words: slid.modelData.words ?? ""
                            number: slid.modelData.number ?? ""
                            widest: slid.modelData.widest ?? "100%"
                            soft: slid.modelData.soft ?? false
                            sticky: slid.modelData.sticky ?? -1
                            iconButton: true
                            focused: slid.modelData.focused ?? false
                            enabled: slid.modelData.enabled ?? true
                            preview: slid.modelData.preview ?? ""
                        }

                        K.Text {
                            text: slid.modelData.label
                            style: K.Tokens.type.small
                            color: K.Tokens.color.on_surface_variant
                        }
                    }
                }
            }
        }
    }
}
