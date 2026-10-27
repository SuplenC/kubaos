import QtQuick
import KubaOS.Components as K

// A toggle tile: a symbol, a title and a status line, as tall as a two-line list row and with a
// control's corners, since it is clicked. On, it fills with the primary colour. Enter, Space or a
// click toggles it (`activated`). With `expandable`, an arrow at its end opens the tile's list
// (`expanded`); Right does too, from the keyboard.
K.Pressable {
    id: root

    property string icon
    property string title
    property string subtitle
    property bool checked: false
    property bool expandable: false

    signal expanded

    readonly property bool fillWidth: true
    readonly property color content: root.checked ? K.Tokens.color.on_primary : K.Tokens.color.on_surface
    readonly property color muted: root.checked ? K.Look.alpha(K.Tokens.color.on_primary, K.Tokens.opacity.subdued) : K.Tokens.color.on_surface_variant
    // Where the words end: before the arrow and its inset, or the tile's own inset.
    readonly property real end: root.expandable ? arrow.width + K.Tokens.space.s12 * 2 : K.Tokens.space.s12

    implicitHeight: K.Tokens.size.row_two_line
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
        radius: K.Tokens.radius.control
        color: root.checked ? K.Tokens.color.primary : K.Tokens.color.surface_container_high

        Behavior on color {
            ColorAnimation {
                duration: K.Tokens.duration.short
                easing.type: Easing.BezierSpline
                easing.bezierCurve: K.Tokens.easing.standard
            }
        }

        K.StateLayer {
            radius: parent.radius
            content: root.content
            hovered: root.hovered && !arrow.hovered
            pressed: root.down
        }
    }

    K.Row {
        id: line

        x: K.Tokens.space.s12
        y: K.Look.snap((root.height - line.implicitHeight) / 2, line.dpr)
        room: Math.max(0, root.width - K.Tokens.space.s12 - root.end)
        gap: K.Tokens.space.s12
        align: "centre"

        K.Icon {
            name: root.icon
            size: K.Tokens.icon.s20
            color: root.checked ? root.content : K.Tokens.color.on_surface_variant
        }

        K.Stack {
            gap: K.Tokens.space.s4

            K.Text {
                text: root.title
                style: K.Tokens.type.body_strong
                color: root.content
                maximumLines: 1
            }

            K.Text {
                text: root.subtitle
                style: K.Tokens.type.small
                color: root.muted
                maximumLines: 1
            }
        }
    }

    K.IconButton {
        id: arrow

        x: root.width - width - K.Tokens.space.s12
        y: K.Look.snap((root.height - height) / 2, line.dpr)
        visible: root.expandable
        activeFocusOnTab: false
        icon: "chevron_right"
        label: `${root.title} settings`
        // Tonal reads on the tile on and off alike; a text button vanishes into the primary fill.
        kind: "Tonal"
        size: "Sm"
        onActivated: root.expanded()
    }

    K.FocusRing {
        shown: root.focused && root.enabled
        controlRadius: K.Tokens.radius.control
    }
}
