import QtQuick
import KubaOS.Components as K

// A panel's body or a toolbar band: its content the container padding in at the sides, by the
// width it is given (Tailwind's container, `px-4 sm:px-6 lg:px-8`: 16 under 640, 24 from 640, 32
// from 1024), as tall as the content, which keeps its own spacing above and below.
// `kubaos-ui`'s `layout::container`.
Item {
    id: root

    default property alias content: slot.data
    property alias room: slot.room

    readonly property bool fillWidth: true
    // The padding at the sides, by the container's width.
    readonly property int padding: Tokens.containerPadding(root.width)
    readonly property var probe: ({
            role: "component",
            name: "Container",
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

    K.Slot {
        id: slot

        width: root.width
        fill: true
        leftPadding: root.padding
        rightPadding: root.padding
    }
}
