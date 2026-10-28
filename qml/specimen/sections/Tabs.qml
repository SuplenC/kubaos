import QtQuick
import KubaOS.Components as K

// The components page's "Tabs" section, drawn with `KubaOS.Components`: the section
// `crates/kubaos-specimen/src/pages/components/tabs.rs` draws with `kubaos-ui`, box for box.
K.Section {
    title: "Tabs"
    description: "One of a few views or values, picked from small buttons: the picked one tonal. The same row is a setting's segmented control."

    K.Row {
        gap: K.Tokens.space.s32
        align: "centre"

        K.Tabs {
            labels: ["Wi-Fi", "Bluetooth", "Output", "Keyboard"]
            currentIndex: 0
        }

        K.Tabs {
            labels: ["Timer", "Alarm"]
            icons: ["timer", "alarm"]
            currentIndex: 1
            focused: true
        }

        K.Tabs {
            labels: ["Always", "On AC", "Never"]
            currentIndex: 1
        }
    }
}
