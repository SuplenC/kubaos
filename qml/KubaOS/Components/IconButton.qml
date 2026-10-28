import QtQuick
import KubaOS.Components as K

// A circle with a symbol, as tall as the label button of its size: `kubaos-ui`'s
// `button::icon_button`. Its `label` is required: an icon alone says nothing out loud, so the
// label is its accessible name and shows in a hint while the pointer is over it or the keyboard
// is on it.
K.Pressable {
    id: root

    required property string icon
    required property string label
    // `Filled`, `Tonal`, `Text` or `Danger`.
    property string kind: "Filled"
    // `Sm`, `Md` or `Lg`.
    property string size: "Md"

    readonly property var metrics: K.ButtonMetrics.of(root.size)
    readonly property var colours: K.ButtonMetrics.colours(root.kind, root.enabled)
    readonly property var probe: ({
            role: "component",
            name: "IconButton",
            variant: `${root.kind} ${root.size}`,
            fixed: {
                width: true,
                height: true
            }
        })

    implicitWidth: root.metrics.height
    implicitHeight: root.metrics.height
    Accessible.name: root.label
    Accessible.role: Accessible.Button

    Rectangle {
        anchors.fill: parent
        radius: Tokens.radius.full
        color: root.colours.fill

        K.StateLayer {
            radius: parent.radius
            content: root.colours.content
            active: root.enabled
            hovered: root.hovered
            pressed: root.down
        }
    }

    K.Icon {
        id: glyph

        x: K.Look.snap((root.width - glyph.size) / 2, glyph.dpr)
        y: K.Look.snap((root.height - glyph.size) / 2, glyph.dpr)
        name: root.icon
        size: root.metrics.icon
        color: root.colours.shown
    }

    K.FocusRing {
        shown: root.focused && root.enabled
        filled: root.kind === "Filled"
        controlRadius: Tokens.radius.full
    }

    K.Hint {
        text: root.label
        shown: (root.focused && root.enabled) || root.pointerOver
    }
}
