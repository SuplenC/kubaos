import QtQuick
import KubaOS.Components as K

// Rows in a well: `kubaos-ui`'s `list::list`. A surface (24) with 12 inside and rows 4 apart, so
// the rows take 24 - 12 = 12, the control radius: concentric with the well. The focus tube lights
// the well's edge while the list has the keyboard; the rows show where it is themselves.
//
// A flush list (`flush`) has no well: its rows lie straight on the surface under it, with no
// fill, edge or padding of its own, where the surface is the list's only content (a Sheet of
// counters). The focus tube still lights its edge, at its rows' corner, as its edge is theirs: the
// tube and a lit or hovered row at it are one shape.
Item {
    id: root

    default property alias rows: stack.data
    // Whether the list has the keyboard.
    property bool focused: false
    property alias room: well.room
    // The name the geometry check reads it by: `EmptyList` reads its own well as one.
    property string probeName: "List"
    // Its rows straight on the surface: no fill, edge or padding.
    property bool flush: false

    readonly property bool fillWidth: true
    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    readonly property var probe: ({
            role: "component",
            name: root.probeName,
            variant: root.flush ? "Flush" : "",
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
        visible: !root.flush
        radius: Tokens.radius.surface
        color: K.Look.alpha(Tokens.color.surface_container_low, Tokens.opacity.well)
        border.width: K.Look.hairline(root.dpr)
        border.color: K.Look.alpha(Tokens.color.outline_variant, Tokens.opacity.outline_soft)
    }

    K.Slot {
        id: well

        width: root.width
        fill: true
        leftPadding: root.flush ? 0 : Tokens.space.s12
        rightPadding: root.flush ? 0 : Tokens.space.s12
        topPadding: root.flush ? 0 : Tokens.space.s12
        bottomPadding: root.flush ? 0 : Tokens.space.s12

        K.Stack {
            id: stack

            fill: true
            gap: Tokens.space.s4
        }
    }

    K.FocusRing {
        shown: root.focused
        controlRadius: root.flush ? K.Look.nested(Tokens.radius.surface, Tokens.space.s12) : Tokens.radius.surface
    }
}
