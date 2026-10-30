import QtQuick
import KubaOS.Components as K

// A titled part that opens to show more: `kubaos-ui`'s `collapsible::collapsible`. The header is
// a control as tall as a list row: its title, and a chevron at its end, down while closed and up
// while open. Open, the content sits 8 under it, revealed over `medium` as a list reordering
// settles (`standard_decel`); closing hides it over `short`. Enter, Space or a click on the header
// toggles it (`toggled`); the owner keeps `open`. Disabled (`enabled: false`), the header's title
// and chevron are at the disabled opacity, and it takes no hover, press or focus; what it holds is
// the owner's to disable with it.
Item {
    id: root

    property string title
    property bool open: false
    // Whether the keyboard is on the header; Qt's focus by default, or the app's where it owns
    // focus.
    property alias focused: header.focused
    default property alias content: slot.data

    signal toggled

    readonly property bool fillWidth: true
    readonly property color disabledContent: K.Look.alpha(Tokens.color.on_surface, Tokens.opacity.disabled_content)
    readonly property var probe: ({
            role: "component",
            name: "Collapsible",
            variant: root.open ? "open" : "",
            fixed: {
                width: false,
                height: false,
                holds: true
            }
        })

    implicitHeight: column.implicitHeight

    function relayout(): void {
        column.relayout();
    }

    K.Stack {
        id: column

        width: root.width
        fill: true
        gap: Tokens.space.s8

        K.Pressable {
            id: header

            readonly property bool fillWidth: true

            implicitHeight: Tokens.size.row
            Accessible.role: Accessible.Button
            Accessible.name: root.title
            onActivated: root.toggled()

            function relayout(): void {
                line.relayout();
            }

            K.StateLayer {
                radius: Tokens.radius.control
                hovered: header.hovered
                pressed: header.down
            }

            K.Row {
                id: line

                x: Tokens.space.s12
                width: header.width - Tokens.space.s12 * 2
                fixedHeight: header.height
                gap: Tokens.space.s12
                align: "centre"

                K.Stack {
                    fill: true

                    K.Text {
                        text: root.title
                        style: Tokens.type.body_strong
                        color: root.enabled ? Tokens.color.on_surface : root.disabledContent
                        maximumLines: 1
                    }
                }

                K.Icon {
                    name: root.open ? "expand_less" : "expand_more"
                    size: Tokens.type.body_strong.icon
                    color: root.enabled ? Tokens.color.on_surface_variant : root.disabledContent
                }
            }

            K.FocusRing {
                shown: header.focused
                controlRadius: Tokens.radius.control
            }
        }

        // The content's room: as tall as the content while open, nothing while closed, moving
        // between the two.
        Item {
            id: body

            readonly property bool fillWidth: true
            property real shown: root.open ? slot.implicitHeight : 0

            implicitHeight: body.shown
            visible: root.open || body.shown > 0
            clip: true

            function relayout(): void {
                slot.relayout();
            }

            Behavior on shown {
                NumberAnimation {
                    duration: root.open ? Tokens.duration.medium : Tokens.duration.short
                    easing.type: Easing.BezierSpline
                    easing.bezierCurve: root.open ? Tokens.easing.standard_decel : Tokens.easing.standard_accel
                }
            }

            // In line with the title: the header's own inset at the sides, once, so the content
            // brings none of its own.
            K.Slot {
                id: slot

                width: body.width
                fill: true
                leftPadding: Tokens.space.s12
                rightPadding: Tokens.space.s12
            }
        }
    }
}
