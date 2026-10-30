import QtQuick
import KubaOS.Components as K

// The "Shell only" page's live list: a scan's rows, its section labels and the rows still being
// found.
K.Section {
    title: "LiveList"
    description: "A list a scan updates while it shows, which never jumps: section labels the keyboard skips, rows on their tiles, and rows still being found keeping the place of the rows that will take it."

    K.Grid {
        columns: 2
        gap: K.Tokens.space.s24

        K.ScrollingList {
            implicitHeight: K.Tokens.size.log

            K.LiveList {
                id: list

                anchors.fill: parent
                rows: [
                    {
                        key: "known",
                        kind: "label",
                        title: "Known"
                    },
                    {
                        key: "home",
                        kind: "row",
                        title: "Shinjuku-5G",
                        subtitle: "Connected"
                    },
                    {
                        key: "nearby",
                        kind: "label",
                        title: "Nearby"
                    },
                    {
                        key: "cafe",
                        kind: "row",
                        title: "Golden Gai Free Wi-Fi",
                        subtitle: "Open"
                    },
                    {
                        key: "found-1",
                        kind: "skeleton",
                        title: "",
                        subtitle: ""
                    },
                    {
                        key: "found-2",
                        kind: "skeleton",
                        title: "",
                        subtitle: ""
                    }
                ]

                delegate: K.LiveRow {
                    id: live

                    required property string key
                    required property string kind
                    required property string title
                    required property string subtitle

                    // A label after rows stands 12 clear of them, with the 4 the row above carries.
                    readonly property int labelRoom: live.index > 0 ? K.Tokens.space.s12 : 0

                    rowHeight: live.kind === "label" ? live.labelRoom + K.Tokens.type.label.line : K.Tokens.size.row_two_line

                    Loader {
                        width: live.width
                        height: live.rowHeight
                        sourceComponent: ({
                                row: wifiRow,
                                label: sectionLabel,
                                skeleton: skeletonRow
                            })[live.kind] ?? null
                    }

                    Component {
                        id: wifiRow

                        K.ListRow {
                            title: live.title
                            subtitle: live.subtitle
                            icon: "wifi"
                            tile: true
                            checkable: true
                            checked: live.key === "home"
                        }
                    }

                    Component {
                        id: sectionLabel

                        Item {
                            K.Text {
                                x: K.Tokens.space.s12
                                y: live.labelRoom
                                text: live.title
                                style: K.Tokens.type.label
                                color: K.Tokens.color.on_surface_variant
                            }
                        }
                    }

                    Component {
                        id: skeletonRow

                        K.ListRow {
                            loading: true
                            share: live.index % 2 ? 0.45 : 0.7
                        }
                    }
                }
            }
        }
    }
}
