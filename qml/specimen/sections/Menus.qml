import QtQuick
import KubaOS.Components as K

// The components page's "Selects and menus" section, drawn with `KubaOS.Components`: the section
// `crates/kubaos-specimen/src/pages/components/menus.rs` draws with `kubaos-ui`, box for box.
K.Section {
    title: "Selects and menus"
    description: "A popover floats over the page: the surface radius, 12 inside, its items small controls. A select drops one from a field-like control."

    K.Grid {
        columns: 3
        gap: K.Tokens.space.s24

        K.Stack {
            fill: true
            gap: K.Tokens.space.s8

            K.Select {
                options: ["None", "Germany", "Italy", "Poland"]
                currentIndex: 3
                icon: "flag"
                open: true
            }

            K.Popover {
                fillWidth: true

                K.Menu {
                    currentIndex: 2
                    entries: ["None", "Germany", "Italy", "Poland"].map(name => ({
                                text: name,
                                checkable: true,
                                checked: name === "Poland"
                            }))
                }
            }
        }

        K.Popover {
            fixedWidth: K.Tokens.size.menu

            K.Menu {
                currentIndex: 0
                entries: [
                    {
                        text: "Open",
                        icon: "folder_open",
                        shortcut: ["Enter"]
                    },
                    {
                        text: "Copy path",
                        icon: "content_paste",
                        shortcut: ["Ctrl", "C"]
                    },
                    {
                        separator: true
                    },
                    {
                        text: "Pin to the top",
                        icon: "push_pin"
                    },
                    {
                        text: "Forget",
                        icon: "close",
                        enabled: false
                    },
                    {
                        separator: true
                    },
                    {
                        text: "Quit",
                        icon: "logout",
                        danger: true
                    }
                ]
            }
        }

        K.Stack {
            fill: true
            gap: K.Tokens.space.s16

            K.Select {
                options: ["Every 5 minutes"]
                currentIndex: 0
            }

            K.Select {
                options: ["Every 5 minutes"]
                currentIndex: 0
                focused: true
            }

            K.Select {
                options: ["Unavailable"]
                currentIndex: 0
                enabled: false
            }
        }
    }
}
