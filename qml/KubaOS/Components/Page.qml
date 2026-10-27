import QtQuick
import KubaOS.Components as K

// A whole page on the surface, its content the container padding in at the sides (16 under 640,
// 24 from 640, 32 from 1024: `Tokens.containerPadding`), 32 below the top and 24 above the foot,
// where a footer's buttons sit (Kōbō's panel). `kubaos-ui`'s `layout::page`.
Rectangle {
    id: root

    default property alias content: slot.data
    property alias room: slot.room

    readonly property bool fillWidth: true
    // The padding at the sides, by the page's width.
    readonly property int padding: Tokens.containerPadding(root.width)
    readonly property var probe: ({
            role: "component",
            name: "Page",
            variant: "",
            fixed: {
                width: false,
                height: false,
                holds: true
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
        leftPadding: root.padding
        rightPadding: root.padding
        topPadding: Tokens.space.s32
        bottomPadding: Tokens.space.s24
    }
}
