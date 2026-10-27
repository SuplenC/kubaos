import QtQuick
import KubaOS.Components as K

// A whole page on the surface, with the page's insets around its content: 40 at the sides, 32
// above, 24 below, where a footer's buttons sit (Kōbō's panel). `kubaos-ui`'s `layout::page`.
Rectangle {
    id: root

    default property alias content: slot.data
    property alias room: slot.room

    readonly property bool fillWidth: true
    readonly property var probe: ({
            role: "component",
            name: "Page",
            variant: "",
            fixed: {
                width: false,
                height: false
            }
        })

    function relayout(): void {
        slot.relayout();
    }

    implicitWidth: slot.implicitWidth
    implicitHeight: slot.implicitHeight
    color: Tokens.color.surface

    K.Slot {
        id: slot

        width: root.width
        fill: true
        leftPadding: Tokens.space.s40
        rightPadding: Tokens.space.s40
        topPadding: Tokens.space.s32
        bottomPadding: Tokens.space.s24
    }
}
