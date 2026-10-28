import QtQuick
import KubaOS.Components as K

// The components page's "Tiles" section, drawn with `KubaOS.Components`: the section
// `crates/kubaos-specimen/src/pages/components/tiles.rs` draws with `kubaos-ui`, box for box.
K.Section {
    title: "Tiles"
    description: "Quick toggles as tall as a two-line row, filled with the accent while on. The detail arrow stays quiet: no fill, a state layer under the pointer."

    K.Grid {
        columns: 4
        gap: K.Tokens.space.s16

        K.Tile {
            icon: "signal_wifi_4_bar"
            title: "Wi-Fi"
            subtitle: "Shinjuku-5G"
            checked: true
        }

        K.Tile {
            icon: "bluetooth"
            title: "Bluetooth"
            subtitle: "Off"
        }

        K.Tile {
            icon: "keyboard"
            title: "Keyboard"
            subtitle: "US · 1 of 2"
            checked: true
            expandable: true
        }

        K.Tile {
            icon: "key"
            title: "Proxy"
            subtitle: "Off"
            expandable: true
            focused: true
        }
    }
}
