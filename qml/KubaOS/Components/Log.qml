import QtQuick
import KubaOS.Components as K

// The log behind an error or a progress bar, `mono`, newest line at the bottom, the surface
// radius: `kubaos-ui`'s `error::log`. It keeps to its bottom as lines arrive. The caller sets its
// height, a size token; `Tokens.size.log` is the one the specimen and Kōbō use.
Item {
    id: root

    // Each line as `{ level, text }`; the level is `error`, `warn`, `step`, `info` or `debug`.
    property var lines: []

    readonly property bool fillWidth: true
    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    readonly property var probe: ({
            role: "component",
            name: "Log",
            variant: "",
            fixed: {
                width: false,
                height: false
            }
        })

    function colour(level: string): color {
        const c = Tokens.color;
        switch (level) {
        case "error":
            return c.error;
        case "warn":
            return c.primary;
        case "step":
            return c.cold;
        case "debug":
            return c.on_surface_variant;
        default:
            return c.on_surface;
        }
    }

    implicitHeight: Tokens.size.log

    Rectangle {
        anchors.fill: parent
        radius: Tokens.radius.surface
        color: K.Look.alpha(Tokens.color.surface_container_lowest, Tokens.opacity.log)
        border.width: K.Look.hairline(root.dpr)
        border.color: K.Look.alpha(Tokens.color.outline_variant, Tokens.opacity.outline_soft)
    }

    Flickable {
        id: scroller

        x: Tokens.space.s16
        y: Tokens.space.s12
        width: root.width - Tokens.space.s16 * 2
        height: root.height - Tokens.space.s12 * 2
        contentWidth: lines.width
        contentHeight: lines.implicitHeight
        contentY: Math.max(lines.implicitHeight - scroller.height, 0)
        clip: true
        boundsBehavior: Flickable.StopAtBounds

        K.Stack {
            id: lines

            room: scroller.width

            Repeater {
                model: root.lines

                K.Text {
                    required property var modelData

                    text: modelData.text
                    style: Tokens.type.mono
                    color: root.colour(modelData.level)
                }
            }
        }
    }
}
