import QtQuick
import KubaOS.Components as K

// What is new about something, over its icon's corner: `kubaos-ui`'s `avatar::badge`. A count in a
// pill a line of label text tall, or a dot when `count` is 0, in the error colour so it reads over
// any icon. One digit makes a circle; more pad 4 either side, and past 99 it reads 99+.
Item {
    id: root

    // How many, or 0 for a dot.
    property int count: 0

    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    readonly property bool dot: root.count <= 0
    readonly property string words: root.count > 99 ? "99+" : String(root.count)
    readonly property bool oneDigit: root.words.length === 1
    readonly property var probe: ({
            role: "component",
            name: "Badge",
            variant: root.dot ? "dot" : "count",
            fixed: {
                width: false,
                height: true
            }
        })

    implicitWidth: root.dot ? Tokens.size.badge_dot : root.oneDigit ? Tokens.size.badge : K.Look.snapUp(label.width + Tokens.space.s4 * 2, root.dpr)
    implicitHeight: root.dot ? Tokens.size.badge_dot : Tokens.size.badge

    Rectangle {
        anchors.fill: parent
        radius: Tokens.radius.full
        color: Tokens.color.error
    }

    K.Text {
        id: label

        x: K.Look.centre(root.width - width, root.dpr)
        visible: !root.dot
        text: root.words
        style: Tokens.type.label
        color: Tokens.color.on_error
    }
}
