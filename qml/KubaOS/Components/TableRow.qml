import QtQuick
import KubaOS.Components as K

// One row of a `Table`: `kubaos-ui`'s `table::table_row`. A line of body text with 4 above and
// below, its `cells` in the table's `columns`: words on one line, ending in an ellipsis where
// they don't fit; numbers at their column's right edge in `mono_large`, so their digits line up.
// The whole row is one control: the pointer over it lays the state layer across every cell, and
// the picked row takes the secondary container.
K.Pressable {
    id: root

    // `[{ title, weight, numeric }]`, the table's.
    property var columns: []
    property list<string> cells
    property bool selected: false

    readonly property bool fillWidth: true
    readonly property color main: root.selected ? Tokens.color.on_secondary_container : Tokens.color.on_surface
    readonly property color muted: root.selected ? K.Look.alpha(Tokens.color.on_secondary_container, Tokens.opacity.subdued_text) : Tokens.color.on_surface_variant
    readonly property var probe: ({
            role: "component",
            name: "TableRow",
            variant: "",
            fixed: {
                width: false,
                height: true
            }
        })

    function relayout(): void {
        grid.relayout();
    }

    implicitHeight: Tokens.size.row_compact
    activeFocusOnTab: false

    Rectangle {
        anchors.fill: parent
        radius: Tokens.radius.control
        color: root.selected ? Tokens.color.secondary_container : "transparent"

        K.StateLayer {
            radius: parent.radius
            active: root.enabled
            hovered: root.hovered
            pressed: root.down
        }
    }

    K.Grid {
        id: grid

        x: Tokens.space.s12
        y: Tokens.space.s4
        width: root.width - Tokens.space.s12 * 2
        weights: root.columns.map(column => column.weight)
        gap: Tokens.space.s12

        Repeater {
            model: root.columns.length

            K.TableCell {
                required property int index

                text: root.cells[index] ?? ""
                numeric: root.columns[index].numeric ?? false
                style: numeric ? Tokens.type.mono_large : Tokens.type.body
                color: numeric ? root.muted : root.main
            }
        }
    }
}
