import QtQuick
import KubaOS.Components as K

// A quick toggle: `kubaos-ui`'s `tile::tile`. A symbol, a title and a status line, as tall as a
// two-line row and with a control's corners, since it is clicked. On, it fills with the accent.
// Enter, Space or a click toggles it (`activated`). With `expandable`, a detail arrow at its end
// opens what is behind it (`expanded`), and Right does too, from the keyboard. The arrow is a
// small control that stays quiet: no fill, in the tile's own content colour, its state layer
// showing under the pointer.
K.Pressable {
    id: root

    property string icon
    property string title
    property string subtitle
    property bool checked: false
    property bool expandable: false

    signal expanded

    readonly property bool fillWidth: true
    readonly property color content: !root.enabled ? K.Look.alpha(Tokens.color.on_surface, Tokens.opacity.disabled_content) : root.checked ? Tokens.color.on_primary : Tokens.color.on_surface
    readonly property color muted: !root.enabled ? root.content : root.checked ? K.Look.alpha(Tokens.color.on_primary, Tokens.opacity.subdued_text) : Tokens.color.on_surface_variant
    // The symbol and the arrow: the content colour while on, quieter while off.
    readonly property color glyph: root.checked || !root.enabled ? root.content : Tokens.color.on_surface_variant
    // Where the words end: before the arrow and its inset, or at the tile's own inset.
    readonly property real end: root.expandable ? Tokens.space.s12 * 2 + Tokens.size.control_sm : Tokens.space.s12
    readonly property var probe: ({
            role: "component",
            name: "Tile",
            variant: root.expandable ? "detail" : "",
            fixed: {
                width: false,
                height: true
            }
        })

    implicitHeight: Tokens.size.row_two_line
    Accessible.name: root.title
    Accessible.role: Accessible.CheckBox
    Accessible.checked: root.checked
    Keys.onRightPressed: event => {
        event.accepted = root.expandable;
        if (root.expandable)
            root.expanded();
    }

    Rectangle {
        anchors.fill: parent
        radius: Tokens.radius.control
        color: !root.enabled ? K.Look.alpha(Tokens.color.on_surface, Tokens.opacity.disabled_container) : root.checked ? Tokens.color.primary : Tokens.color.surface_container_high

        Behavior on color {
            ColorAnimation {
                duration: Tokens.duration.short
                easing.type: Easing.BezierSpline
                easing.bezierCurve: Tokens.easing.standard
            }
        }

        K.StateLayer {
            radius: parent.radius
            content: root.content
            hovered: root.hovered && !arrow.hovered
            pressed: root.down && !arrow.down
        }
    }

    K.Row {
        id: line

        x: Tokens.space.s12
        width: root.width - Tokens.space.s12 - root.end
        fixedHeight: root.height
        gap: Tokens.space.s12
        align: "centre"

        K.Icon {
            name: root.icon
            size: Tokens.type.body_strong.icon
            color: root.glyph
        }

        K.Stack {
            fill: true
            gap: Tokens.space.s4

            K.Text {
                text: root.title
                style: Tokens.type.body_strong
                color: root.content
                maximumLines: 1
            }

            K.Text {
                text: root.subtitle
                style: Tokens.type.small
                color: root.muted
                maximumLines: 1
            }
        }
    }

    K.Pressable {
        id: arrow

        x: root.width - Tokens.space.s12 - width
        y: K.Look.snap((root.height - height) / 2, line.dpr)
        width: Tokens.size.control_sm
        height: Tokens.size.control_sm
        visible: root.expandable
        activeFocusOnTab: false
        Accessible.name: `${root.title} settings`
        onActivated: root.expanded()

        K.StateLayer {
            radius: Tokens.radius.full
            content: root.glyph
            hovered: arrow.hovered
            pressed: arrow.down
        }

        K.Icon {
            x: K.Look.snap((arrow.width - width) / 2, line.dpr)
            y: K.Look.snap((arrow.height - height) / 2, line.dpr)
            name: "chevron_right"
            size: Tokens.type.button_small.icon
            color: root.glyph
        }
    }

    K.FocusRing {
        shown: root.focused && root.enabled
        filled: root.checked
        controlRadius: Tokens.radius.control
    }
}
