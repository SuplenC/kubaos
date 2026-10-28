import QtQuick
import KubaOS.Components as K

// A button with a label: `kubaos-ui`'s `button::button`. Four kinds by what it is for, three sizes;
// an optional icon leads the label, or trails it on a button that goes forward. The control
// radius; the label's line fills the control's content height, so it centres on the grid.
K.Pressable {
    id: root

    property string text
    // `Filled`, `Tonal`, `Text` or `Danger`.
    property string kind: "Filled"
    // `Sm`, `Md` or `Lg`.
    property string size: "Md"
    // A symbol beside the label, or none.
    property string icon: ""
    // The icon after the label: a button that goes forward.
    property bool trailing: false

    readonly property var metrics: K.ButtonMetrics.of(root.size)
    readonly property real padding: root.kind === "Text" ? root.metrics.textPadding : root.metrics.padding
    readonly property var colours: K.ButtonMetrics.colours(root.kind, root.enabled)
    readonly property var probe: ({
            role: "component",
            name: "Button",
            variant: [root.kind, root.size, root.icon === "" ? "" : root.trailing ? "trailing-icon" : "leading-icon"].filter(word => word !== "").join(" "),
            fixed: {
                width: false,
                height: true
            }
        })

    implicitWidth: K.Look.snapUp(content.implicitWidth + root.padding * 2, content.dpr)
    implicitHeight: root.metrics.height

    Rectangle {
        anchors.fill: parent
        radius: Tokens.radius.control
        color: root.colours.fill

        K.StateLayer {
            radius: parent.radius
            content: root.colours.content
            active: root.enabled
            hovered: root.hovered
            pressed: root.down
        }
    }

    K.Row {
        id: content

        x: root.padding
        y: (root.metrics.height - root.metrics.label.line) / 2
        gap: root.metrics.gap

        K.IconInLine {
            visible: root.icon !== "" && !root.trailing
            name: root.icon
            size: root.metrics.icon
            line: root.metrics.label.line
            color: root.colours.shown
        }

        K.Text {
            text: root.text
            style: root.metrics.label
            color: root.colours.shown
        }

        K.IconInLine {
            visible: root.icon !== "" && root.trailing
            name: root.icon
            size: root.metrics.icon
            line: root.metrics.label.line
            color: root.colours.shown
        }
    }

    K.FocusRing {
        shown: root.focused && root.enabled
        filled: root.kind === "Filled"
        controlRadius: Tokens.radius.control
    }
}
