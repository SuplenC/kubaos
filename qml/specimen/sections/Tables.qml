import QtQuick
import KubaOS.Components as K

// The components page's "Tables" section, drawn with `KubaOS.Components`: the section
// `crates/kubaos-specimen/src/pages/components/tables.rs` draws with `kubaos-ui`, box for box.
K.Section {
    title: "Tables"
    description: "Readings in columns, a line of text with 4 above and below a row; the whole row takes the pointer, numbers line up at their column's right edge."

    K.Table {
        columns: [
            {
                title: "Name",
                weight: 4
            },
            {
                title: "CPU",
                weight: 1,
                numeric: true
            },
            {
                title: "Memory",
                weight: 1,
                numeric: true
            }
        ]
        rows: [
            {
                cells: ["firefox", "38.4%", "2.1 GB"]
            },
            {
                cells: ["Hyprland", "12.1%", "384 MB"],
                selected: true
            },
            {
                cells: ["rust-analyzer", "9.7%", "1.8 GB"]
            },
            {
                cells: ["quickshell", "4.2%", "256 MB"]
            }
        ]
    }
}
