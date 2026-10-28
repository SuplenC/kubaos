import QtQuick
import KubaOS.Components as K

// One item of a menu, or the hairline between two groups: `kubaos-ui`'s `menu::menu_item` and
// `menu::menu_separator`. An item is a small control tall, 12 inside: an optional symbol, its
// words on one line, the keys that do the same as keycaps, and a check whose room is kept when
// unchecked. Its corners are concentric inside the popover. The item the keyboard is on takes
// the secondary container; one that destroys something (Kill) is in the error colour.
K.Pressable {
    id: root

    property string text
    property string icon: ""
    // The keys that do the same, one keycap each.
    property list<string> shortcut
    // Whether it shows a check at its end, and whether it is ticked.
    property bool checkable: false
    property bool checked: false
    property bool danger: false
    property bool highlighted: false
    // A hairline between two groups, in place of an item.
    property bool separator: false

    readonly property bool fillWidth: true
    readonly property color ink: !root.enabled ? K.Look.alpha(Tokens.color.on_surface, Tokens.opacity.disabled_content) : root.highlighted ? Tokens.color.on_secondary_container : root.danger ? Tokens.color.error : Tokens.color.on_surface
    readonly property var probe: root.separator ? null : {
        role: "component",
        name: "MenuItem",
        variant: [root.icon === "" ? "" : "icon", root.checkable ? "check" : "", root.danger ? "danger" : ""].filter(word => word !== "").join(" "),
        fixed: {
            width: false,
            height: true
        }
    }

    function relayout(): void {
        if (root.separator)
            rule.relayout();
        else
            line.relayout();
    }

    implicitHeight: root.separator ? rule.implicitHeight : Tokens.size.control_sm
    activeFocusOnTab: false

    Rectangle {
        anchors.fill: parent
        visible: !root.separator
        radius: K.Look.nested(Tokens.radius.surface, Tokens.space.s12)
        color: root.highlighted && root.enabled ? Tokens.color.secondary_container : "transparent"

        K.StateLayer {
            radius: parent.radius
            content: root.ink
            active: root.enabled
            hovered: root.hovered
            pressed: root.down
        }
    }

    K.Stack {
        id: rule

        visible: root.separator
        width: root.width
        fill: true
        topPadding: Tokens.space.s4
        bottomPadding: Tokens.space.s4

        K.Divider {
        }
    }

    K.Row {
        id: line

        visible: !root.separator
        x: Tokens.space.s12
        width: root.width - Tokens.space.s12 * 2
        fixedHeight: root.height
        gap: Tokens.space.s12
        align: "centre"

        K.Icon {
            visible: root.icon !== ""
            name: root.icon
            size: Tokens.type.body.icon
            color: root.ink
        }

        K.Stack {
            fill: true

            K.Text {
                text: root.text
                style: Tokens.type.body
                color: root.ink
                maximumLines: 1
            }
        }

        K.Row {
            visible: root.shortcut.length > 0
            gap: Tokens.space.s4
            align: "centre"

            Repeater {
                model: root.shortcut

                K.Keycap {
                    required property string modelData

                    key: modelData
                }
            }
        }

        Item {
            visible: root.checkable
            implicitWidth: Tokens.type.body.icon
            implicitHeight: Tokens.type.body.icon

            K.Icon {
                visible: root.checked
                name: "check"
                size: Tokens.type.body.icon
                color: root.ink
            }
        }
    }
}
