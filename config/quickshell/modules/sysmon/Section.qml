import QtQuick
import KubaOS.Components as K
import "panel.js" as Panel

// One reading of the system monitor's panel, in a card of its own so the readings don't run into
// each other: an icon and a title, the headline reading on the right, and what it holds below,
// 4 apart. While the headline reading is on its way (`waiting`), a placeholder holds its room and
// it fades in over it.
K.Card {
    id: root

    property string icon
    property string title
    property string value
    property color valueColor: K.Tokens.color.on_surface
    property bool waiting: false
    // Whether the reading fades in over its placeholder (`Line`, `fades`).
    property bool fades: true
    default property alias body: content.data

    readonly property int fade: K.Motion.reduced ? K.Tokens.duration.short : K.Tokens.duration.slow

    K.Stack {
        fill: true
        gap: K.Tokens.space.s8

        K.Row {
            id: head

            fill: true
            gap: K.Tokens.space.s8
            align: "centre"

            K.Icon {
                name: root.icon
                size: K.Tokens.type.body_strong.icon
                color: K.Tokens.color.primary
            }

            K.Text {
                text: root.title
                style: K.Tokens.type.body_strong
            }

            K.Fill {
            }

            // The reading ticks, so it is in mono: its width holds still. Its line is there with no
            // reading too, so one arriving doesn't make the card taller.
            Item {
                id: reading

                readonly property real placeholderWidth: K.Look.snap(head.width * Panel.headline(), head.dpr)

                implicitWidth: root.waiting ? reading.placeholderWidth : value.implicitWidth
                implicitHeight: K.Tokens.type.mono_large.line

                K.Skeleton {
                    x: reading.width - width
                    width: reading.placeholderWidth
                    style: K.Tokens.type.mono_large
                    loading: root.waiting
                    visible: opacity > 0
                    opacity: root.waiting ? 1 : 0

                    Behavior on opacity {
                        enabled: root.fades

                        NumberAnimation {
                            duration: root.fade
                            easing.type: Easing.BezierSpline
                            easing.bezierCurve: K.Tokens.easing.standard
                        }
                    }
                }

                K.Text {
                    id: value

                    x: reading.width - width
                    text: root.value
                    style: K.Tokens.type.mono_large
                    color: root.valueColor
                    visible: opacity > 0
                    opacity: root.waiting ? 0 : 1

                    Behavior on opacity {
                        enabled: root.fades

                        NumberAnimation {
                            duration: root.fade
                            easing.type: Easing.BezierSpline
                            easing.bezierCurve: K.Tokens.easing.standard
                        }
                    }
                }
            }
        }

        K.Stack {
            id: content

            fill: true
            gap: K.Tokens.space.s4
        }
    }
}
