import QtQuick
import KubaOS.Components as K

// The components page's "Selects and menus" section, drawn with `KubaOS.Components`: the section
// `crates/kubaos-specimen/src/pages/components/menus.rs` draws with `kubaos-ui`, box for box.
K.Section {
    title: "Selects and menus"
    description: "A popover floats over the page: the surface radius, 12 inside, its items small controls. A select drops one from a field-like control. A danger item lights in the error, and presses deeper. A held item is taken only once held: the error sweeps across it. An item may end in a value of its own."

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

        // A counter's menu, its held item lit and held part of the way.
        K.Popover {
            fixedWidth: K.Tokens.size.menu

            K.Menu {
                currentIndex: 4
                entries: [
                    {
                        text: "Rename"
                    },
                    {
                        text: "Reset now"
                    },
                    {
                        text: "History"
                    },
                    {
                        separator: true
                    },
                    {
                        text: "Hold to remove",
                        held: true,
                        reach: 0.4
                    }
                ]
            }
        }

        // A danger item lit, then lit and pressed.
        Repeater {
            model: ["", "pressed"]

            K.Popover {
                required property string modelData

                fixedWidth: K.Tokens.size.menu

                K.Menu {
                    currentIndex: 2
                    entries: [
                        {
                            text: "Copy command",
                            icon: "content_paste"
                        },
                        {
                            separator: true
                        },
                        {
                            text: "Kill",
                            icon: "close",
                            danger: true,
                            preview: modelData
                        }
                    ]
                }
            }
        }

        // A counter's Auto reset choices: When it reaches carries its stop as a value of its own.
        K.Popover {
            fixedWidth: K.Tokens.size.menu

            K.Menu {
                currentIndex: 1
                entries: [
                    {
                        text: "Never",
                        checkable: true
                    },
                    {
                        text: "When it reaches",
                        value: "10",
                        checkable: true,
                        checked: true
                    }
                ]
            }
        }
    }
}
