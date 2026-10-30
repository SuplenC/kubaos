import QtQuick
import KubaOS.Components as K
import "sections" as S

// The specimen's components page, drawn with `KubaOS.Components`: the same page
// `kubaos-specimen` draws with `kubaos-ui` (`crates/kubaos-specimen/src/pages/components.rs` and
// the bar above it), box for box, so the cross-toolkit test can hold the two to the same frame and
// their screenshots can be laid side by side. Each section is a file of its own in sections/.
K.Stack {
    id: root

    // The scale the page is shown at, named in the bar.
    property real shownScale: 1

    fill: true

    // The bar above the page: the pages, then the mode and the scale it is shown at. A toolbar
    // band: the container padding at the sides, as the page under it, and a small control's room
    // above and below.
    K.Container {
        K.Row {
            gap: K.Tokens.space.s16
            align: "centre"
            fill: true
            topPadding: K.Tokens.space.s16
            bottomPadding: K.Tokens.space.s16

            K.Text {
                text: "KubaOS design specimen"
                style: K.Tokens.type.subheading
            }

            K.Row {
                gap: K.Tokens.space.s4
                align: "centre"

                Repeater {
                    model: ["Type", "Colour", "Space and shape", "Components", "Icons", "Motion", "Templates"]

                    K.Button {
                        required property string modelData

                        text: modelData
                        kind: modelData === "Components" ? "Tonal" : "Text"
                        size: "Sm"
                    }
                }
            }

            K.Fill {
            }

            K.KeyHints {
                hints: [["Tab", "next page"], ["← →", "pages"], ["M", "mode"]]
            }

            K.Chip {
                text: `${root.shownScale}×`
            }

            K.IconButton {
                icon: K.Tokens.mode === "dark" ? "light_mode" : "dark_mode"
                label: K.Tokens.mode === "dark" ? "Light mode" : "Dark mode"
                kind: "Text"
                size: "Sm"
            }
        }
    }

    K.Divider {
    }

    K.Page {
        K.Stack {
            gap: K.Tokens.space.s40

            K.Header {
                accent: "部品"
                title: "Components"
                lede: "The closed set: every kind and size, resting, hovered, pressed, focused and disabled."
            }

            K.Stack {
                gap: K.Tokens.space.s48

                // One file each under sections/, in the iced page's order
                // (`crates/kubaos-specimen/src/pages/components.rs`): a section adds its file and its
                // line in both.
                S.Buttons {
                }

                S.IconButtons {
                }

                S.Fields {
                }

                S.Toggles {
                }

                S.Sliders {
                }

                S.Tabs {
                }

                S.Tiles {
                }

                S.IconTiles {
                }

                S.Skeletons {
                }

                S.Lists {
                }

                S.PageScroll {
                }

                S.Chips {
                }

                S.Keycaps {
                }

                S.Tooltips {
                }

                S.Cards {
                }

                S.Sheets {
                }

                S.Calendars {
                }

                S.Media {
                }

                S.Banners {
                }

                S.Steps {
                }

                S.DescriptionLists {
                }

                S.Dialogs {
                }

                S.Progress {
                }

                S.Meters {
                }

                S.EmptyStates {
                }

                S.Menus {
                }

                S.Tables {
                }

                S.Collapsibles {
                }

                S.Errors {
                }
            }
        }
    }
}
