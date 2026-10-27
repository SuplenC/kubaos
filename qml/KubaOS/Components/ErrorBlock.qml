import QtQuick
import KubaOS.Components as K

// What broke and what to do next: `kubaos-ui`'s `error::error_block`. A 24 error icon 16 from the
// message in `subheading`, the hint in `body` 4 under it, on the error container with an error
// hairline, the surface radius, 20 inside. A message past 90 characters drops to `body_strong`,
// since a whole command line in 16 px semibold is a wall.
Item {
    id: root

    property string message
    property string hint
    property alias room: slot.room

    // Past this many characters, the message is long.
    readonly property int longMessage: 90
    readonly property bool long: root.message.length > root.longMessage
    readonly property bool fillWidth: true
    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    readonly property var probe: ({
            role: "component",
            name: "ErrorBlock",
            variant: root.long ? "long" : "",
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
        color: K.Look.alpha(Tokens.color.error_container, Tokens.opacity.error_fill)
        border.width: K.Look.hairline(root.dpr)
        border.color: K.Look.alpha(Tokens.color.error, Tokens.opacity.error_edge)
    }

    K.Slot {
        id: slot

        width: root.width
        fill: true
        leftPadding: Tokens.space.s20
        rightPadding: Tokens.space.s20
        topPadding: Tokens.space.s20
        bottomPadding: Tokens.space.s20

        K.Row {
            gap: Tokens.space.s16
            align: "centre"

            K.Icon {
                name: "error"
                size: Tokens.icon.s24
                color: Tokens.color.error
            }

            K.Stack {
                fill: true
                gap: Tokens.space.s4

                K.Text {
                    text: root.message
                    style: root.long ? Tokens.type.body_strong : Tokens.type.subheading
                    color: Tokens.color.on_surface
                }

                K.Text {
                    text: root.hint
                    style: Tokens.type.body
                    color: Tokens.color.on_surface_variant
                }
            }
        }
    }
}
