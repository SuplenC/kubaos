import QtQuick

// Empty room of one spacing token both ways, where a stack needs one larger step:
// `kubaos-ui`'s `layout::gap`.
Item {
    id: root

    property int space: Tokens.space.s8

    readonly property var probe: ({
            role: "spacer",
            fill: false
        })

    implicitWidth: root.space
    implicitHeight: root.space
}
