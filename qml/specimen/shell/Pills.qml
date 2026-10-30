import QtQuick
import KubaOS.Components as K

// The "Shell only" page's pills: the bar's groups.
K.Section {
    title: "Pill"
    description: "A group in the bar, as tall as a small control. One that takes a click lays the state layer over itself under the pointer and is never a stop for Tab; its tooltip shows in a hint under it."

    K.Row {
        gap: K.Tokens.space.s8
        align: "centre"

        K.Pill {
            K.Text {
                text: "Tue 27 Oct 21:47"
                style: K.Tokens.type.label
            }
        }

        K.Pill {
            interactive: true
            tooltip: "Wi-Fi, sound and battery"

            K.Icon {
                name: "wifi"
                size: K.Tokens.type.label.icon
            }

            K.Icon {
                name: "volume_up"
                size: K.Tokens.type.label.icon
            }

            K.Icon {
                name: "battery_full"
                size: K.Tokens.type.label.icon
            }
        }

        K.Pill {
            interactive: true

            K.BadgedIcon {
                name: "notifications"
                badged: true
            }
        }
    }
}
