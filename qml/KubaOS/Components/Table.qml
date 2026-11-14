import QtQuick
import KubaOS.Components as K

// Readings in columns: `kubaos-ui`'s `table::table`. A header naming `columns`
// (`[{ title, weight, numeric }]`, each a share of the width), then a compact `TableRow` per
// entry of `rows` (`[{ cells, selected }]`). A press on a row sends `activated` with its index.
Item {
    id: root

    property var columns: []
    property var rows: []

    signal activated(int index)

    readonly property bool fillWidth: true
    readonly property var probe: ({
            role: "component",
            name: "Table",
            variant: "",
            fixed: {
                width: false,
                height: false
            }
        })

    implicitHeight: column.implicitHeight

    function relayout(): void {
        column.relayout();
    }

    K.Stack {
        id: column

        width: root.width
        fill: true

        Item {
            id: header

            readonly property bool fillWidth: true

            implicitHeight: titles.implicitHeight + Tokens.space.s4 * 2

            function relayout(): void {
                titles.relayout();
            }

            K.Split {
                id: titles

                x: Tokens.space.s12
                y: Tokens.space.s4
                width: header.width - Tokens.space.s12 * 2
                weights: root.columns.map(column => column.weight)
                gap: Tokens.space.s12

                Repeater {
                    model: root.columns

                    K.TableCell {
                        required property var modelData

                        text: modelData.title
                        numeric: modelData.numeric ?? false
                        oneLine: false
                        style: Tokens.type.label
                        color: Tokens.color.on_surface_variant
                    }
                }
            }
        }

        // A row per entry, by count: new readings refill the rows there are, and only a count that
        // changes makes or drops one. A list as the model would make every row again each time.
        Repeater {
            model: root.rows.length

            K.TableRow {
                required property int index

                columns: root.columns
                cells: root.rows[index]?.cells ?? []
                selected: root.rows[index]?.selected ?? false
                onActivated: root.activated(index)
            }
        }
    }
}
