import QtQuick
import KubaOS.Components as K

// One row of a list: `kubaos-ui`'s `list::list_row`. 44 tall with one line, 56 with a subtitle;
// 12 inside, a 20 icon, the title in `body_strong`, the subtitle in `small`, chips and a detail in
// `mono` at the end, and an optional check whose room is kept when unchecked, so the rows' ends
// line up. Each part is centred on the row's whole height, so a chip moves nothing else.
// Its corners are concentric inside the list's well. The selected row takes the secondary
// container, or the error colour on a row whose action can't be taken back (`danger`).
K.Pressable {
    id: root

    property string title
    property string subtitle
    // A short fact at the row's end, in mono: a device path, a size, a code.
    property string detail
    // Chips near the row's end, as `{ kind, text }`: facts that matter at a glance (Default).
    property var chips: []
    property string icon: ""
    // The row the keyboard is on, or the one picked in a list of one.
    property bool selected: false
    // Its action can't be taken back (Shut down).
    property bool danger: false
    // Whether the row shows a check at its end, and whether it is ticked.
    property bool checkable: false
    property bool checked: false

    readonly property bool fillWidth: true
    readonly property color onFill: root.danger ? Tokens.color.on_error : Tokens.color.on_secondary_container
    readonly property color main: root.selected ? root.onFill : Tokens.color.on_surface
    readonly property color muted: root.selected ? K.Look.alpha(root.onFill, Tokens.opacity.subdued) : Tokens.color.on_surface_variant
    readonly property var probe: ({
            role: "component",
            name: "ListRow",
            variant: [root.subtitle === "" ? "one-line" : "two-line", root.icon === "" ? "" : "icon"].filter(word => word !== "").join(" "),
            fixed: {
                width: false,
                height: true
            }
        })

    implicitHeight: root.subtitle === "" ? Tokens.size.row : Tokens.size.row_two_line
    // The list's well shows where the keyboard is; a row draws no ring of its own.
    activeFocusOnTab: false

    Rectangle {
        anchors.fill: parent
        radius: K.Look.nested(Tokens.radius.surface, Tokens.space.s12)
        color: !root.selected ? "transparent" : root.danger ? Tokens.color.error : Tokens.color.secondary_container

        K.StateLayer {
            radius: parent.radius
            content: Tokens.color.on_surface
            active: root.enabled
            hovered: root.hovered
            pressed: root.down
        }
    }

    K.Row {
        id: line

        x: Tokens.space.s12
        width: root.width - Tokens.space.s12 * 2
        fixedHeight: root.height
        gap: Tokens.space.s12
        align: "centre"

        K.Icon {
            visible: root.icon !== ""
            name: root.icon
            size: Tokens.type.body_strong.icon
            color: root.main
        }

        K.Stack {
            fill: true
            gap: Tokens.space.s4

            K.Text {
                text: root.title
                style: Tokens.type.body_strong
                maximumLines: 1
                color: root.main
            }

            K.Text {
                visible: root.subtitle !== ""
                text: root.subtitle
                style: Tokens.type.small
                maximumLines: 1
                color: root.muted
            }
        }

        Repeater {
            model: root.chips

            K.Chip {
                required property var modelData

                kind: modelData.kind
                text: modelData.text
            }
        }

        K.Text {
            visible: root.detail !== ""
            text: root.detail
            style: Tokens.type.mono
            color: root.muted
        }

        Item {
            visible: root.checkable
            implicitWidth: Tokens.type.body_strong.icon
            implicitHeight: Tokens.type.body_strong.icon

            K.Icon {
                visible: root.checked
                name: "check"
                size: Tokens.type.body_strong.icon
                color: root.main
            }
        }
    }
}
