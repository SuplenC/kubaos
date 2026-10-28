import QtQuick
import QtQuick.Shapes
import KubaOS.Components as K

// The components page's "Sheets" section, drawn with `KubaOS.Components`: the section
// `crates/kubaos-specimen/src/pages/components/sheets.rs` draws with `kubaos-ui`, box for box.
K.Section {
    title: "Sheets"
    description: "A panel growing out of the frame's edge, one surface with it: concave fillets where it meets the edge, the surface radius at its far corners."

    Item {
        id: stage

        readonly property bool fillWidth: true
        readonly property color frameColour: K.Look.alpha(K.Tokens.color.surface, K.Tokens.opacity.frame)

        implicitHeight: K.Tokens.space.s8 + sheet.contentHeight + K.Tokens.space.s32

        Rectangle {
            anchors.fill: parent
            radius: K.Tokens.radius.surface
            color: K.Look.alpha(K.Tokens.color.surface_container_highest, K.Tokens.opacity.well)
        }

        // The frame's top edge, which the sheet hangs from.
        Rectangle {
            width: stage.width
            height: K.Tokens.space.s8
            topLeftRadius: K.Tokens.radius.surface
            topRightRadius: K.Tokens.radius.surface
            color: stage.frameColour
        }

        Shape {
            anchors.fill: parent
            preferredRendererType: Shape.CurveRenderer

            ShapePath {
                fillColor: stage.frameColour
                // design-allow(size): -1 is ShapePath's "no stroke", not a width.
                strokeWidth: -1

                PathSvg {
                    path: sheet.path
                }
            }
        }

        K.Sheet {
            id: sheet

            frame: QtObject {
                readonly property real l: 0
                readonly property real t: K.Tokens.space.s8
                readonly property real r: stage.width
                readonly property real b: stage.height
            }
            // Shown open, without the motion that opens it.
            progress: 1
            takesFocus: false
            animateSize: false
            contentWidth: sheetBody.implicitWidth
            contentHeight: sheetBody.implicitHeight

            K.Stack {
                id: sheetBody

                gap: K.Tokens.space.s4
                leftPadding: K.Tokens.space.s16
                rightPadding: K.Tokens.space.s16
                topPadding: K.Tokens.space.s16
                bottomPadding: K.Tokens.space.s16

                K.Text {
                    text: "Control center"
                    style: K.Tokens.type.subheading
                }

                K.Text {
                    text: "It grows out of the frame's edge."
                    style: K.Tokens.type.small
                    color: K.Tokens.color.on_surface_variant
                }
            }
        }
    }
}
