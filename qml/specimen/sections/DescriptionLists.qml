import QtQuick
import KubaOS.Components as K

// The components page's "Description lists" section, drawn with `KubaOS.Components`: the section
// `crates/kubaos-specimen/src/pages/components/description_lists.rs` draws with `kubaos-ui`, box for box.
K.Section {
    title: "Description lists"
    description: "Facts beside their labels, a quarter of the width; a path or an email gets a mono line of its own."

    K.Grid {
        columns: 2
        gap: K.Tokens.space.s24

        K.Card {
            K.DescriptionList {
                facts: [
                    {
                        label: "Name",
                        value: "Kuba Example"
                    },
                    {
                        label: "Login",
                        value: "kuba@tuxedo"
                    },
                    {
                        label: "Git",
                        value: "Kuba Example",
                        sub: "kuba@example.com"
                    }
                ]
            }
        }

        K.Card {
            K.DescriptionList {
                facts: [
                    {
                        label: "Disk",
                        value: "Samsung SSD 990 PRO 2TB",
                        sub: "/dev/nvme0n1 · 2.0 TB"
                    },
                    {
                        label: "Mode",
                        value: "Erase everything"
                    },
                    {
                        label: "Encryption",
                        value: "LUKS2, account password"
                    }
                ]
            }
        }
    }
}
