import QtQuick
import KubaOS.Components as K

// A panel's or a page's body: its content the container padding in on every side, by the width
// it is given (Tailwind's container: 16 under 640, 24 from 640, 32 from 1024), as tall as the
// content. `kubaos-ui`'s `layout::container`. The body of every panel that isn't a whole `Page`.
Item {
    id: root

    default property alias content: slot.data
    property alias room: slot.room

    readonly property bool fillWidth: true
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
        topPadding: root.padding
        bottomPadding: root.padding
    }
}
