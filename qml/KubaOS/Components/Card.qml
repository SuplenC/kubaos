import QtQuick
import KubaOS.Components as K

// A surface inside the page: the well's surface and a soft hairline, the surface radius, 16
// inside. `kubaos-ui`'s `card::card`.
Item {
    id: root

    default property alias content: slot.data
    property alias room: slot.room

    readonly property bool fillWidth: true
    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    readonly property var probe: ({
            role: "component",
            name: "Card",
            variant: "",
            fixed: {
                width: false,
                height: false
            }
        })

    function relayout(): void {
        slot.relayout();
    }

    implicitHeight: slot.implicitHeight

    Rectangle {
        anchors.fill: parent
        radius: Tokens.radius.surface
        color: K.Look.alpha(Tokens.color.surface_container_low, Tokens.opacity.well)
        border.width: K.Look.hairline(root.dpr)
        border.color: K.Look.alpha(Tokens.color.outline_variant, Tokens.opacity.outline_soft)
    }

    K.Slot {
        id: slot

        width: root.width
        fill: true
        leftPadding: Tokens.space.s16
        rightPadding: Tokens.space.s16
        topPadding: Tokens.space.s16
        bottomPadding: Tokens.space.s16
    }
}
