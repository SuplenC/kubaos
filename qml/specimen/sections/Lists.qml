import QtQuick
import KubaOS.Components as K

// The components page's "Lists" section, drawn with `KubaOS.Components`: the section
// `crates/kubaos-specimen/src/pages/components/lists.rs` draws with `kubaos-ui`, box for box.
K.Section {
    title: "Lists"
    description: "Rows in a well: a surface, 24, with 12 inside, so the rows take 24 − 12 = 12, the control radius. The focus tube lights the well's edge; the row shows where the keyboard is."

    K.Grid {
        columns: 3
        gap: K.Tokens.space.s24

        K.List {
            K.ListRow {
                title: "English (US)"
                chips: [{ kind: "Accent", text: "Default" }]
                detail: "us"
                checkable: true
                checked: true
            }

            K.ListRow {
                title: "Italian"
                detail: "it"
                checkable: true
                selected: true
            }

            K.ListRow {
                title: "German"
                detail: "de"
                checkable: true
            }
        }

        K.List {
            focused: true

            K.ListRow {
                title: "Home-5G"
                subtitle: "Secured · strong"
                icon: "wifi"
                selected: true
            }

            K.ListRow {
                title: "FRITZ!Box 7590 XL Gästenetz OG"
                subtitle: "Secured · fair"
                icon: "wifi"
            }

            K.ListRow {
                title: "Samsung SSD 990 PRO"
                subtitle: "nvme0n1 · 2 TB · Linux"
                icon: "hard_drive"
                detail: "2 TB"
            }
        }

        K.EmptyList {
            text: "Nothing matches. Typo?"
        }

        K.List {
            K.ListRow {
                title: "Reboot"
                icon: "restart_alt"
            }

            K.ListRow {
                title: "Shut down"
                icon: "power_settings_new"
                danger: true
                selected: true
            }
        }

        K.List {
            K.ListRow {
                title: "English (US)"
                subtitle: "us"
                icon: "star_filled"
                tile: true
                accent: true
                checkable: true
            }

            K.ListRow {
                title: "Italian"
                subtitle: "it"
                icon: "keyboard"
                tile: true
                checkable: true
                checked: true
                selected: true
            }
        }

        K.List {
            K.ListRow {
                title: "Shinjuku-5G"
                subtitle: "Connected · 5 GHz"
                subtitleIcon: "lock"
                icon: "signal_wifi_4_bar"
                tile: true
                accent: true
                checkable: true
                checked: true
            }

            K.ListRow {
                title: "Omoide Yokocho"
                subtitle: "Secured"
                subtitleIcon: "lock"
                icon: "network_wifi_2_bar"
                tile: true
                checkable: true
            }
        }
    }
}
