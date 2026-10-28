import QtQuick
import KubaOS.Components as K

// Rows in a well: `kubaos-ui`'s `list::list`. A surface (24) with 12 inside and rows 4 apart, so
// the rows take 24 - 12 = 12, the control radius: concentric with the well. The focus tube lights
// the well's edge while the list has the keyboard; the rows show where it is themselves.
Item {
    id: root

    default property alias rows: stack.data
    // Whether the list has the keyboard.
    property bool focused: false
    property alias room: well.room
    // The name the geometry check reads it by: `EmptyList` reads its own well as one.
    property string probeName: "List"

    readonly property bool fillWidth: true
    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    readonly property var probe: ({
            role: "component",
            name: root.probeName,
            variant: "",
            fixed: {
                width: false,
                height: false
            }
        })

    function relayout(): void {
        well.relayout();
    }

    implicitHeight: well.implicitHeight

    Rectangle {
        anchors.fill: parent
        radius: Tokens.radius.surface
        color: K.Look.alpha(Tokens.color.surface_container_low, Tokens.opacity.well)
        border.width: K.Look.hairline(root.dpr)
        border.color: K.Look.alpha(Tokens.color.outline_variant, Tokens.opacity.outline_soft)
    }

    K.Slot {
        id: well

        width: root.width
        fill: true
        leftPadding: Tokens.space.s12
        rightPadding: Tokens.space.s12
        topPadding: Tokens.space.s12
        bottomPadding: Tokens.space.s12

        K.Stack {
            id: stack

            fill: true
            gap: Tokens.space.s4
        }
    }

    K.FocusRing {
        shown: root.focused
        controlRadius: Tokens.radius.surface
    }
}
