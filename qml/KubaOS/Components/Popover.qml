import QtQuick
import QtQuick.Effects
import KubaOS.Components as K

// The floating surface a menu sits on: `kubaos-ui`'s `menu::popover`. The surface radius, a soft
// hairline, the panel's shadow, 12 inside, so the items in it take 24 − 12 = 12, the control
// radius. It floats where its owner puts it, over the window (`floating`), or sits in place where
// it is laid out (the specimen).
Item {
    id: root

    default property alias content: slot.data
    property alias room: slot.room
    // Over the window, above everything in it; its owner makes it a child of the window's content
    // item, so no clipping panel cuts it.
    property bool floating: false
    // Whether it takes the width it is given, as a select's menu takes the select's.
    property bool fillWidth: false
    // A width of its own, a size token (a menu's), or 0 for its content's.
    property real fixedWidth: 0

    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    // A floating popover is no part of the page the geometry check reads.
    readonly property bool overlay: root.floating
    readonly property var probe: ({
            role: "component",
            name: "Popover",
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

    implicitWidth: root.fixedWidth > 0 ? root.fixedWidth : root.fillWidth ? 0 : slot.implicitWidth
    implicitHeight: slot.implicitHeight
    z: root.floating ? 1000 : 0

    RectangularShadow {
        anchors.fill: surface
        radius: surface.radius
        offset.y: Tokens.shadow.panel.y
        blur: Tokens.shadow.panel.blur
        color: K.Look.alpha(Tokens.color.shadow, Tokens.shadow.panel.opacity)
    }

    Rectangle {
        id: surface

        anchors.fill: parent
        radius: Tokens.radius.surface
        color: Tokens.color.surface_container
        border.width: K.Look.hairline(root.dpr)
        border.color: K.Look.alpha(Tokens.color.outline_variant, Tokens.opacity.outline_soft)
    }

    // Clicks on the surface stay on it.
    MouseArea {
        anchors.fill: parent
    }

    K.Slot {
        id: slot

        width: root.width
        fill: root.fillWidth
        leftPadding: Tokens.space.s12
        rightPadding: Tokens.space.s12
        topPadding: Tokens.space.s12
        bottomPadding: Tokens.space.s12
    }
}
